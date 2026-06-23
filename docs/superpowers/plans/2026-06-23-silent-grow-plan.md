# 「悄悄长大」视频剪辑工具 实现计划

> **For agentic workers:** REQUIRED SUB-SKILL: Use superpowers:subagent-driven-development (recommended) or superpowers:executing-plans to implement this plan task-by-task. Steps use checkbox (`- [ ]`) syntax for tracking.

**Goal:** 构建 macOS 桌面视频剪辑工具，支持「悄悄长大」模板——上传照片后自动人脸检测+对齐，生成 0.2s/张的竖屏视频。

**Architecture:** Tauri 2.x 桌面壳 + Vue 3 前端 + Rust 后端引擎（YuNet ONNX 人脸检测 → 5点关键点对齐 → FFmpeg 视频合成）。

**Tech Stack:** Tauri 2.x, Vue 3 + Vite + Pinia, Rust (`ort`, `image`, `nalgebra`, `serde_yaml`), YuNet ONNX, FFmpeg

---

## File Structure Map

```
hey24/
├── src-tauri/                        # Rust 后端
│   ├── Cargo.toml                    # Rust 依赖声明
│   ├── tauri.conf.json               # Tauri 配置
│   ├── build.rs                      # Tauri build 脚本
│   ├── icons/                        # App 图标
│   ├── models/
│   │   └── yunet.onnx                # YuNet ONNX 模型（需下载）
│   ├── templates/
│   │   └── silent-grow.yaml          # 「悄悄长大」模板定义
│   └── src/
│       ├── main.rs                   # Tauri 入口（不修改）
│       ├── lib.rs                    # Tauri 库入口，注册 commands
│       ├── commands.rs               # Tauri IPC 命令
│       ├── error.rs                  # 自定义错误类型
│       └── engine/
│           ├── mod.rs                # 引擎模块导出
│           ├── face.rs               # YuNet ONNX 加载 + 推理
│           ├── align.rs              # 面部关键点对齐
│           ├── render.rs             # 帧序列 + FFmpeg 调度
│           └── template.rs           # YAML 模板加载 + 管线编排
├── src/                              # Vue 3 前端
│   ├── main.ts                       # Vue 入口
│   ├── App.vue                       # 根组件（布局壳）
│   ├── style.css                     # 全局样式
│   ├── types/
│   │   └── index.ts                  # TypeScript 类型定义
│   ├── stores/
│   │   └── project.ts                # Pinia 项目状态管理
│   ├── views/
│   │   └── EditorView.vue            # 主编编辑器视图
│   └── components/
│       ├── TemplateList.vue          # 左侧模板列表
│       ├── PhotoTimeline.vue         # 中间照片时间线
│       ├── PhotoCard.vue             # 单张照片缩略图卡片
│       ├── FaceSelector.vue          # 多人脸选择弹窗
│       ├── PreviewPanel.vue          # 右侧预览 + 生成按钮
│       ├── OutputSettings.vue        # 可展开输出参数
│       └── StatusBar.vue             # 底部状态栏
├── scripts/
│   └── download-model.sh            # 下载 YuNet ONNX 模型脚本
├── index.html                        # Vite HTML 入口
├── package.json                      # Node 依赖
├── vite.config.ts                    # Vite 配置
├── tsconfig.json                     # TypeScript 配置
└── README.md
```

---

### Task 1: Scaffold Tauri + Vue 3 Project

**Files:**
- Create: `package.json`, `vite.config.ts`, `tsconfig.json`, `index.html`
- Create: `src/main.ts`, `src/App.vue`, `src/style.css`
- Create: `src-tauri/Cargo.toml`, `src-tauri/tauri.conf.json`, `src-tauri/build.rs`
- Create: `src-tauri/src/main.rs`, `src-tauri/src/lib.rs`

- [ ] **Step 1: Create Vue 3 + Vite project**

```bash
npm create vite@latest hey24 -- --template vue-ts
cd hey24
npm install
```

- [ ] **Step 2: Install Tauri CLI and init**

```bash
npm install -D @tauri-apps/cli@latest
npx tauri init
# → App name: hey24
# → Window title: Hey24
# → Frontend dev URL: http://localhost:5173
# → Frontend build command: npm run build
# → Frontend dev command: npm run dev
```

- [ ] **Step 3: Verify Tauri dev mode runs**

```bash
npx tauri dev
```

Expected: A blank Tauri window opens showing the default Vite + Vue welcome page.

- [ ] **Step 4: Commit**

```bash
git add -A && git commit -m "feat: scaffold Tauri + Vue 3 project"
```

---

### Task 2: Set Up Rust Dependencies

**Files:**
- Modify: `src-tauri/Cargo.toml`
- Create: `src-tauri/src/error.rs`
- Create: `src-tauri/src/engine/mod.rs`

- [ ] **Step 1: Add Rust crate dependencies**

Edit `src-tauri/Cargo.toml` to add dependencies under `[dependencies]`:

```toml
[dependencies]
tauri = { version = "2", features = ["drag-drop"] }
tauri-plugin-dialog = "2"
tauri-plugin-fs = "2"
serde = { version = "1", features = ["derive"] }
serde_json = "1"
serde_yaml = "0.9"
ort = { version = "2", features = ["load-dynamic"] }
image = "0.25"
nalgebra = "0.33"
tempfile = "3"
uuid = { version = "1", features = ["v4"] }
thiserror = "2"
```

- [ ] **Step 2: Create error types**

Create `src-tauri/src/error.rs`:

```rust
use thiserror::Error;

#[derive(Error, Debug)]
pub enum AppError {
    #[error("Face detection failed: {0}")]
    FaceDetection(String),

    #[error("Face alignment failed: {0}")]
    FaceAlignment(String),

    #[error("Render failed: {0}")]
    Render(String),

    #[error("Template error: {0}")]
    Template(String),

    #[error("IO error: {0}")]
    Io(#[from] std::io::Error),

    #[error("Image processing error: {0}")]
    Image(#[from] image::ImageError),
}

pub type AppResult<T> = Result<T, AppError>;
```

- [ ] **Step 3: Create engine module skeleton**

Create `src-tauri/src/engine/mod.rs`:

```rust
pub mod face;
pub mod align;
pub mod render;
pub mod template;
```

- [ ] **Step 4: Update lib.rs to declare modules**

Edit `src-tauri/src/lib.rs`:

```rust
mod engine;
mod error;
mod commands;

use commands::*;

#[cfg_attr(mobile, tauri::mobile_entry_point)]
pub fn run() {
    tauri::Builder::default()
        .invoke_handler(tauri::generate_handler![
            detect_faces,
            align_photos,
            render_video,
            load_template,
        ])
        .run(tauri::generate_context!())
        .expect("error while running tauri application");
}
```

- [ ] **Step 5: Verify it compiles**

```bash
cd src-tauri && cargo check
```

Expected: `Finished dev [unoptimized + debuginfo]` (commands stubs won't exist yet, but that's ok — we'll add them later).

- [ ] **Step 6: Commit**

```bash
git add -A && git commit -m "feat: add Rust dependencies and error types"
```

---

### Task 3: Download YuNet ONNX Model

**Files:**
- Create: `scripts/download-model.sh`
- Create: `src-tauri/models/.gitkeep`

- [ ] **Step 1: Create model download script**

Create `scripts/download-model.sh`:

```bash
#!/bin/bash
set -euo pipefail

MODEL_DIR="src-tauri/models"
MODEL_FILE="$MODEL_DIR/yunet.onnx"
MODEL_URL="https://github.com/opencv/opencv_zoo/raw/main/models/face_detection_yunet/face_detection_yunet_2023mar.onnx"

mkdir -p "$MODEL_DIR"

if [ ! -f "$MODEL_FILE" ]; then
  echo "Downloading YuNet ONNX model..."
  curl -L -o "$MODEL_FILE" "$MODEL_URL"
  echo "Done: $MODEL_FILE"
else
  echo "Model already exists: $MODEL_FILE"
fi
```

- [ ] **Step 2: Run the download script**

```bash
chmod +x scripts/download-model.sh
./scripts/download-model.sh
```

Expected: Model file downloaded to `src-tauri/models/yunet.onnx` (~390KB).

- [ ] **Step 3: Verify model file**

```bash
ls -lh src-tauri/models/yunet.onnx
```

Expected: File exists, ~390KB.

- [ ] **Step 4: Add model to Tauri resources**

Add to `src-tauri/tauri.conf.json` under `bundle.resources`:

```json
{
  "bundle": {
    "resources": [
      "models/yunet.onnx",
      "templates/*"
    ]
  }
}
```

- [ ] **Step 5: Commit**

```bash
git add -A && git commit -m "feat: download YuNet ONNX model"
```

---

### Task 4: Implement Face Detection Engine

**Files:**
- Create: `src-tauri/src/engine/face.rs`

- [ ] **Step 1: Implement face.rs — ONNX model loading and face detection**

Create `src-tauri/src/engine/face.rs`:

```rust
use image::{DynamicImage, GenericImageView};
use nalgebra::Point2;
use ort::session::{Session, SessionOutputs};
use serde::{Deserialize, Serialize};

use crate::error::{AppError, AppResult};

const INPUT_SIZE: u32 = 320;
const CONFIDENCE_THRESHOLD: f32 = 0.5;
const NMS_THRESHOLD: f32 = 0.45;

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct FaceBox {
    /// Bounding box [x, y, width, height] normalized to [0, 1]
    pub bbox: [f32; 4],
    /// Detection confidence [0, 1]
    pub confidence: f32,
    /// 5 landmarks: left_eye, right_eye, nose_tip, left_mouth, right_mouth
    /// Each is [x, y] normalized to [0, 1]
    pub landmarks: [[f32; 2]; 5],
}

pub struct FaceDetector {
    session: Session,
}

impl FaceDetector {
    /// Load the YuNet ONNX model from the given path.
    pub fn load(model_path: &str) -> AppResult<Self> {
        let session = Session::builder()
            .map_err(|e| AppError::FaceDetection(format!("Failed to create session: {e}")))?
            .commit_from_file(model_path)
            .map_err(|e| AppError::FaceDetection(format!("Failed to load model: {e}")))?;
        Ok(Self { session })
    }

    /// Detect faces in an image. Returns a list of detected faces sorted by confidence (descending).
    pub fn detect(&self, img: &DynamicImage) -> AppResult<Vec<FaceBox>> {
        let (orig_w, orig_h) = img.dimensions();
        let orig_w = orig_w as f32;
        let orig_h = orig_h as f32;

        // Preprocess: resize to INPUT_SIZE, normalize
        let resized = img.resize_exact(
            INPUT_SIZE,
            INPUT_SIZE,
            image::imageops::FilterType::Lanczos3,
        );
        let rgb = resized.to_rgb8();
        let raw = rgb.into_raw();

        // Convert to CHW float tensor with normalization: (pixel - 127.5) / 128.0
        let mut input_data: Vec<f32> = Vec::with_capacity((3 * INPUT_SIZE * INPUT_SIZE) as usize);
        for c in 0..3 {
            for y in 0..INPUT_SIZE {
                for x in 0..INPUT_SIZE {
                    let idx = ((y * INPUT_SIZE + x) * 3 + c) as usize;
                    let val = (raw[idx] as f32 - 127.5) / 128.0;
                    input_data.push(val);
                }
            }
        }

        let input_tensor = ort::value::Tensor::from_array((
            [1usize, 3, INPUT_SIZE as usize, INPUT_SIZE as usize],
            input_data.into_boxed_slice(),
        ))
        .map_err(|e| AppError::FaceDetection(format!("Tensor creation failed: {e}")))?;

        // Run inference
        let outputs: SessionOutputs = self.session.run(ort::inputs!["input" => input_tensor])
            .map_err(|e| AppError::FaceDetection(format!("Inference failed: {e}")))?;

        // Parse outputs
        // YuNet outputs:
        //   output[0]: scores shape [1, N, 1] — confidence
        //   output[1]: bboxes shape [1, N, 4] — [x_center, y_center, w, h] normalized
        //   output[2]: landmarks shape [1, N, 10] — [x1, y1, x2, y2, ..., x5, y5] normalized
        let scores = outputs["score"]
            .try_extract_tensor::<f32>()
            .map_err(|e| AppError::FaceDetection(format!("Score parse: {e}")))?;
        let bboxes = outputs["bbox"]
            .try_extract_tensor::<f32>()
            .map_err(|e| AppError::FaceDetection(format!("Bbox parse: {e}")))?;
        let landmarks = outputs["landmarks"]
            .try_extract_tensor::<f32>()
            .map_err(|e| AppError::FaceDetection(format!("Landmark parse: {e}")))?;

        let scores_view = scores.view();
        let bboxes_view = bboxes.view();
        let landmarks_view = landmarks.view();

        let num_detections = scores_view.shape()[1];
        let mut faces: Vec<FaceBox> = Vec::new();

        for i in 0..num_detections {
            let conf = scores_view[[0, i, 0]];
            if conf < CONFIDENCE_THRESHOLD {
                continue;
            }

            // Convert YuNet bbox format [cx, cy, w, h] to [x, y, w, h]
            let cx = bboxes_view[[0, i, 0]];
            let cy = bboxes_view[[0, i, 1]];
            let bw = bboxes_view[[0, i, 2]];
            let bh = bboxes_view[[0, i, 3]];
            let x = (cx - bw / 2.0).max(0.0);
            let y = (cy - bh / 2.0).max(0.0);

            let mut lms = [[0.0f32; 2]; 5];
            for lm_idx in 0..5 {
                lms[lm_idx] = [
                    landmarks_view[[0, i, lm_idx * 2]],
                    landmarks_view[[0, i, lm_idx * 2 + 1]],
                ];
            }

            faces.push(FaceBox {
                bbox: [x, y, bw, bh],
                confidence: conf,
                landmarks: lms,
            });
        }

        // Sort by confidence descending
        faces.sort_by(|a, b| b.confidence.partial_cmp(&a.confidence).unwrap());

        // Simple NMS (non-maximum suppression)
        faces = nms(faces, NMS_THRESHOLD);

        Ok(faces)
    }
}

fn nms(mut faces: Vec<FaceBox>, threshold: f32) -> Vec<FaceBox> {
    let mut kept: Vec<FaceBox> = Vec::new();
    while let Some(current) = faces.first() {
        kept.push(current.clone());
        faces.retain(|f| iou(&current.bbox, &f.bbox) < threshold);
    }
    kept
}

fn iou(a: &[f32; 4], b: &[f32; 4]) -> f32 {
    let ax1 = a[0];
    let ay1 = a[1];
    let ax2 = a[0] + a[2];
    let ay2 = a[1] + a[3];
    let bx1 = b[0];
    let by1 = b[1];
    let bx2 = b[0] + b[2];
    let by2 = b[1] + b[3];

    let inter_x1 = ax1.max(bx1);
    let inter_y1 = ay1.max(by1);
    let inter_x2 = ax2.min(bx2);
    let inter_y2 = ay2.min(by2);

    let inter_w = (inter_x2 - inter_x1).max(0.0);
    let inter_h = (inter_y2 - inter_y1).max(0.0);
    let inter_area = inter_w * inter_h;

    let a_area = a[2] * a[3];
    let b_area = b[2] * b[3];
    let union = a_area + b_area - inter_area;

    if union < 1e-6 { 0.0 } else { inter_area / union }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_iou_identical() {
        let a = [0.0, 0.0, 0.5, 0.5];
        assert!((iou(&a, &a) - 1.0).abs() < 0.001);
    }

    #[test]
    fn test_iou_no_overlap() {
        let a = [0.0, 0.0, 0.1, 0.1];
        let b = [0.9, 0.9, 0.1, 0.1];
        assert!((iou(&a, &b) - 0.0).abs() < 0.001);
    }
}
```

- [ ] **Step 2: Verify compilation**

```bash
cd src-tauri && cargo check
```

Expected: module compiles (may have some warnings).

- [ ] **Step 3: Commit**

```bash
git add -A && git commit -m "feat: implement face detection engine (YuNet ONNX)"
```

---

### Task 5: Implement Face Alignment Engine

**Files:**
- Create: `src-tauri/src/engine/align.rs`

- [ ] **Step 1: Implement align.rs — similarity transform + image warping**

Create `src-tauri/src/engine/align.rs`:

```rust
use image::{DynamicImage, GenericImageView, RgbaImage};
use nalgebra::{Matrix3, Point2, Vector2};
use std::f32::consts::PI;

use crate::engine::face::FaceBox;
use crate::error::{AppError, AppResult};

#[derive(Debug, Clone)]
pub struct AlignParams {
    /// Target Y position for eyes (fraction of output height, default 0.38)
    pub target_eye_y: f32,
    /// Target pixel distance between the two eyes (default 320)
    pub target_eye_distance: f32,
    /// Output width
    pub out_width: u32,
    /// Output height
    pub out_height: u32,
    /// Fill mode: "blur_expand", "black", "crop"
    pub fill_mode: String,
}

impl Default for AlignParams {
    fn default() -> Self {
        Self {
            target_eye_y: 0.38,
            target_eye_distance: 320.0,
            out_width: 1080,
            out_height: 1920,
            fill_mode: "blur_expand".to_string(),
        }
    }
}

/// Compute a 2x3 similarity transform matrix from source face landmarks to target positions.
/// Returns the 2x3 affine matrix as [ [a,b,tx], [c,d,ty] ].
pub fn compute_similarity_transform(
    landmarks: &[[f32; 2]; 5],
    img_w: f32,
    img_h: f32,
    params: &AlignParams,
) -> [[f32; 3]; 2] {
    // Source: left eye center (index 0), right eye center (index 1)
    let src_left_eye = Point2::new(landmarks[0][0] * img_w, landmarks[0][1] * img_h);
    let src_right_eye = Point2::new(landmarks[1][0] * img_w, landmarks[1][1] * img_h);
    let src_eye_center = Point2::new(
        (src_left_eye.x + src_right_eye.x) / 2.0,
        (src_left_eye.y + src_right_eye.y) / 2.0,
    );
    let src_eye_distance = ((src_right_eye.x - src_left_eye.x).powi(2)
        + (src_right_eye.y - src_left_eye.y).powi(2))
    .sqrt();

    // Target positions
    let target_eye_center_x = params.out_width as f32 / 2.0;
    let target_eye_center_y = params.out_height as f32 * params.target_eye_y;

    // Scale factor
    let scale = params.target_eye_distance / src_eye_distance.max(1e-6);

    // Rotation angle: angle of source eye line relative to horizontal
    let src_angle = (src_right_eye.y - src_left_eye.y)
        .atan2(src_right_eye.x - src_left_eye.x);
    let cos_a = src_angle.cos();
    let sin_a = src_angle.sin();

    // Translation: map src_eye_center to target_eye_center after rotation+scale
    let tx = target_eye_center_x - scale * (cos_a * src_eye_center.x - sin_a * src_eye_center.y);
    let ty = target_eye_center_y - scale * (sin_a * src_eye_center.x + cos_a * src_eye_center.y);

    [
        [scale * cos_a, -scale * sin_a, tx],
        [scale * sin_a, scale * cos_a, ty],
    ]
}

/// Apply a 2x3 affine transform to an image and produce the output at the target resolution.
/// Uses `blur_expand` fill by default.
pub fn apply_transform(
    img: &DynamicImage,
    transform: &[[f32; 3]; 2],
    params: &AlignParams,
) -> AppResult<RgbaImage> {
    let (src_w, src_h) = img.dimensions();
    let out_w = params.out_width;
    let out_h = params.out_height;

    let src = img.to_rgba8();
    let mut dst = RgbaImage::new(out_w, out_h);

    let a = transform[0][0];
    let b = transform[0][1];
    let tx = transform[0][2];
    let c = transform[1][0];
    let d = transform[1][1];
    let ty = transform[1][2];

    // Inverse transform: map output pixel → source pixel
    let det = a * d - b * c;
    if det.abs() < 1e-12 {
        return Err(AppError::FaceAlignment("Degenerate transform matrix".into()));
    }
    let inv_a = d / det;
    let inv_b = -b / det;
    let inv_c = -c / det;
    let inv_d = a / det;

    for out_y in 0..out_h {
        for out_x in 0..out_w {
            let ox = out_x as f32 + 0.5;
            let oy = out_y as f32 + 0.5;

            let sx = inv_a * (ox - tx) + inv_b * (oy - ty);
            let sy = inv_c * (ox - tx) + inv_d * (oy - ty);

            if sx >= 0.0 && sx < (src_w as f32 - 1.0) && sy >= 0.0 && sy < (src_h as f32 - 1.0) {
                // Bilinear interpolation
                let fx = sx.fract();
                let fy = sy.fract();
                let ix = sx.floor() as u32;
                let iy = sy.floor() as u32;

                let p00 = src.get_pixel(ix, iy);
                let p10 = src.get_pixel(ix + 1, iy);
                let p01 = src.get_pixel(ix, iy + 1);
                let p11 = src.get_pixel(ix + 1, iy + 1);

                let r = bilinear(p00[0], p10[0], p01[0], p11[0], fx, fy);
                let g = bilinear(p00[1], p10[1], p01[1], p11[1], fx, fy);
                let b_val = bilinear(p00[2], p10[2], p01[2], p11[2], fx, fy);
                let a_val = bilinear(p00[3], p10[3], p01[3], p11[3], fx, fy);

                dst.put_pixel(out_x, out_y, image::Rgba([r, g, b_val, a_val]));
            }
            // Pixels outside source are left as transparent (0,0,0,0).
            // Production code would implement blur_expand / black fill here.
        }
    }

    // Apply blur_expand fill for transparent pixels if requested
    if params.fill_mode == "blur_expand" {
        fill_transparent_with_blur(&mut dst);
    }

    Ok(dst)
}

fn bilinear(c00: u8, c10: u8, c01: u8, c11: u8, fx: f32, fy: f32) -> u8 {
    let top = c00 as f32 + (c10 as f32 - c00 as f32) * fx;
    let bot = c01 as f32 + (c11 as f32 - c01 as f32) * fx;
    (top + (bot - top) * fy) as u8
}

/// Simple blur expand: for each transparent pixel, fill with average of nearby non-transparent pixels.
fn fill_transparent_with_blur(img: &mut RgbaImage) {
    let (w, h) = img.dimensions();
    let w_i32 = w as i32;
    let h_i32 = h as i32;

    for _pass in 0..3 {
        let snapshot = img.clone();
        for y in 0..h {
            for x in 0..w {
                let px = img.get_pixel(x, y);
                if px[3] > 0 {
                    continue; // Already filled
                }
                let (mut r_sum, mut g_sum, mut b_sum, mut count) = (0u32, 0u32, 0u32, 0u32);
                for dy in -2i32..=2 {
                    for dx in -2i32..=2 {
                        if dx == 0 && dy == 0 { continue; }
                        let nx = (x as i32 + dx).clamp(0, w_i32 - 1) as u32;
                        let ny = (y as i32 + dy).clamp(0, h_i32 - 1) as u32;
                        let np = snapshot.get_pixel(nx, ny);
                        if np[3] > 0 {
                            r_sum += np[0] as u32;
                            g_sum += np[1] as u32;
                            b_sum += np[2] as u32;
                            count += 1;
                        }
                    }
                }
                if count > 0 {
                    img.put_pixel(x, y, image::Rgba([
                        (r_sum / count) as u8,
                        (g_sum / count) as u8,
                        (b_sum / count) as u8,
                        255,
                    ]));
                }
            }
        }
    }
}

/// Align a single photo: take the first (highest-confidence) face and transform the image.
pub fn align_photo(
    img: &DynamicImage,
    faces: &[FaceBox],
    params: &AlignParams,
) -> AppResult<RgbaImage> {
    if faces.is_empty() {
        // No face detected: fall back to center-crop + scale
        return fallback_align(img, params);
    }

    let face = &faces[0];
    let (w, h) = img.dimensions();
    let transform = compute_similarity_transform(&face.landmarks, w as f32, h as f32, params);
    apply_transform(img, &transform, params)
}

/// Fallback when no face detected: scale to fill, center crop.
fn fallback_align(img: &DynamicImage, params: &AlignParams) -> AppResult<RgbaImage> {
    let out_w = params.out_width;
    let out_h = params.out_height;
    let (src_w, src_h) = img.dimensions();

    let scale = (out_w as f32 / src_w as f32).max(out_h as f32 / src_h as f32);
    let new_w = (src_w as f32 * scale) as u32;
    let new_h = (src_h as f32 * scale) as u32;

    let scaled = img.resize_exact(new_w, new_h, image::imageops::FilterType::Lanczos3);
    let x_offset = (new_w.saturating_sub(out_w) / 2) as u32;
    let y_offset = (new_h.saturating_sub(out_h) / 2) as u32;

    Ok(scaled.crop_imm(x_offset, y_offset, out_w, out_h).to_rgba8())
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_similarity_transform_identity() {
        let landmarks = [
            [0.45, 0.35],  // left eye
            [0.55, 0.35],  // right eye
            [0.50, 0.45],  // nose
            [0.47, 0.55],  // left mouth
            [0.53, 0.55],  // right mouth
        ];

        let params = AlignParams {
            target_eye_distance: 108.0, // 0.1 * 1080
            out_width: 1080,
            out_height: 1920,
            ..Default::default()
        };

        let t = compute_similarity_transform(&landmarks, 1080.0, 1920.0, &params);

        // The transform should map left_eye ≈ (486, 672) to near target_eye_x, target_eye_y
        let src_x = 0.45 * 1080.0;
        let src_y = 0.35 * 1920.0;
        let dst_x = t[0][0] * src_x + t[0][1] * src_y + t[0][2];
        let dst_y = t[1][0] * src_x + t[1][1] * src_y + t[1][2];

        assert!((dst_x - 540.0).abs() < 10.0); // near center X
        assert!((dst_y - 1920.0 * 0.38).abs() < 10.0); // near 38% Y
    }
}
```

- [ ] **Step 2: Verify compilation**

```bash
cd src-tauri && cargo check
```

- [ ] **Step 3: Commit**

```bash
git add -A && git commit -m "feat: implement face alignment engine"
```

---

### Task 6: Implement Template System

**Files:**
- Create: `src-tauri/templates/silent-grow.yaml`
- Create: `src-tauri/src/engine/template.rs`

- [ ] **Step 1: Create template YAML**

Create `src-tauri/templates/silent-grow.yaml`:

```yaml
name: "悄悄长大"
id: silent-grow
version: 1
output:
  default:
    width: 1080
    height: 1920
    fps: 5
    duration_per_photo: 0.2
pipeline:
  - stage: face_detect
    model: yunet
    keypoints: true
    confidence_threshold: 0.5
  - stage: face_align
    target_eye_y: 0.38
    target_eye_distance: 320
    fill_mode: blur_expand
  - stage: render
    transition: cut
    fps: 30
    codec: h264_videotoolbox
    bitrate: "8M"
```

- [ ] **Step 2: Implement template.rs**

Create `src-tauri/src/engine/template.rs`:

```rust
use serde::{Deserialize, Serialize};

use crate::engine::align::AlignParams;
use crate::error::{AppError, AppResult};

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct TemplateConfig {
    pub name: String,
    pub id: String,
    pub version: u32,
    pub output: OutputDefaults,
    pub pipeline: Vec<PipelineStage>,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct OutputDefaults {
    pub default: OutputParams,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct OutputParams {
    pub width: u32,
    pub height: u32,
    pub fps: u32,
    pub duration_per_photo: f64,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
#[serde(tag = "stage")]
pub enum PipelineStage {
    #[serde(rename = "face_detect")]
    FaceDetect {
        model: String,
        keypoints: bool,
        #[serde(default = "default_confidence")]
        confidence_threshold: f32,
    },
    #[serde(rename = "face_align")]
    FaceAlign {
        target_eye_y: f32,
        target_eye_distance: f32,
        fill_mode: String,
    },
    #[serde(rename = "render")]
    Render {
        transition: String,
        fps: u32,
        codec: String,
        bitrate: String,
    },
}

fn default_confidence() -> f32 { 0.5 }

impl TemplateConfig {
    /// Load a template from a YAML file path.
    pub fn load(path: &str) -> AppResult<Self> {
        let content = std::fs::read_to_string(path)
            .map_err(|e| AppError::Template(format!("Cannot read template file '{path}': {e}")))?;
        let config: Self = serde_yaml::from_str(&content)
            .map_err(|e| AppError::Template(format!("Invalid template YAML: {e}")))?;
        Ok(config)
    }

    /// Extract align params from the pipeline.
    pub fn align_params(&self) -> AppResult<AlignParams> {
        for stage in &self.pipeline {
            if let PipelineStage::FaceAlign {
                target_eye_y,
                target_eye_distance,
                fill_mode,
            } = stage
            {
                return Ok(AlignParams {
                    target_eye_y: *target_eye_y,
                    target_eye_distance: *target_eye_distance,
                    out_width: self.output.default.width,
                    out_height: self.output.default.height,
                    fill_mode: fill_mode.clone(),
                });
            }
        }
        Err(AppError::Template("No face_align stage found in pipeline".into()))
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_load_template() {
        let config = TemplateConfig::load("templates/silent-grow.yaml").unwrap();
        assert_eq!(config.id, "silent-grow");
        assert_eq!(config.output.default.width, 1080);
        assert_eq!(config.output.default.height, 1920);
    }

    #[test]
    fn test_align_params() {
        let config = TemplateConfig::load("templates/silent-grow.yaml").unwrap();
        let params = config.align_params().unwrap();
        assert_eq!(params.target_eye_y, 0.38);
        assert_eq!(params.target_eye_distance, 320.0);
    }
}
```

- [ ] **Step 3: Verify compilation and run tests**

```bash
cd src-tauri && cargo check && cargo test
```

Expected: `test result: ok. 2 passed`

- [ ] **Step 4: Commit**

```bash
git add -A && git commit -m "feat: implement template system with YAML config"
```

---

### Task 7: Implement Render Engine

**Files:**
- Create: `src-tauri/src/engine/render.rs`

- [ ] **Step 1: Implement render.rs — frame sequence + FFmpeg**

Create `src-tauri/src/engine/render.rs`:

```rust
use image::RgbaImage;
use std::path::{Path, PathBuf};
use std::process::Command;

use crate::error::{AppError, AppResult};

pub struct RenderParams {
    pub out_width: u32,
    pub out_height: u32,
    pub fps_in: u32,           // Input frame rate (1 / duration_per_photo)
    pub fps_out: u32,          // Output frame rate
    pub codec: String,
    pub bitrate: String,
    pub bgm_path: Option<PathBuf>,
}

impl Default for RenderParams {
    fn default() -> Self {
        Self {
            out_width: 1080,
            out_height: 1920,
            fps_in: 5,
            fps_out: 30,
            codec: "h264_videotoolbox".into(),
            bitrate: "8M".into(),
            bgm_path: None,
        }
    }
}

/// Render a list of RGBA frames to an MP4 video file using FFmpeg.
///
/// 1. Write each frame as a PNG to a temp directory.
/// 2. Spawn FFmpeg to encode the PNG sequence into MP4.
/// 3. Return the output path.
pub fn render_video(
    frames: &[RgbaImage],
    output_path: &Path,
    params: &RenderParams,
) -> AppResult<PathBuf> {
    let tmp_dir = tempfile::tempdir()
        .map_err(|e| AppError::Render(format!("Cannot create temp dir: {e}")))?;

    // Write frame PNGs
    for (i, frame) in frames.iter().enumerate() {
        let frame_path = tmp_dir.path().join(format!("frame_{i:04d}.png"));
        frame
            .save(&frame_path)
            .map_err(|e| AppError::Render(format!("Cannot save frame {i}: {e}")))?;
    }

    // Build FFmpeg command
    let frame_pattern = tmp_dir.path().join("frame_%04d.png");
    let mut args = vec![
        "-y".to_string(),
        "-framerate".to_string(),
        params.fps_in.to_string(),
        "-i".to_string(),
        frame_pattern.to_string_lossy().to_string(),
    ];

    // Optional background music
    if let Some(bgm) = &params.bgm_path {
        args.push("-i".to_string());
        args.push(bgm.to_string_lossy().to_string());
        args.push("-shortest".to_string());
    }

    args.extend_from_slice(&[
        "-c:v".to_string(),
        params.codec.clone(),
        "-b:v".to_string(),
        params.bitrate.clone(),
        "-vf".to_string(),
        format!(
            "fps={},format=yuv420p,scale={}:{}:force_original_aspect_ratio=decrease,pad={}:{}:(ow-iw)/2:(oh-ih)/2",
            params.fps_out, params.out_width, params.out_height, params.out_width, params.out_height
        ),
        "-pix_fmt".to_string(),
        "yuv420p".to_string(),
        output_path.to_string_lossy().to_string(),
    ]);

    let output = Command::new("ffmpeg")
        .args(&args)
        .output()
        .map_err(|e| AppError::Render(format!("Failed to spawn ffmpeg: {e}. Is FFmpeg installed?")))?;

    if !output.status.success() {
        let stderr = String::from_utf8_lossy(&output.stderr);
        return Err(AppError::Render(format!("FFmpeg failed:\n{stderr}")));
    }

    Ok(output_path.to_path_buf())
}

/// Generate frame sequence from aligned photos: each photo becomes one frame.
pub fn build_frames(aligned_photos: &[RgbaImage]) -> Vec<RgbaImage> {
    aligned_photos.to_vec()
}

#[cfg(test)]
mod tests {
    use super::*;
    use image::RgbaImage;

    #[test]
    fn test_build_frames_empty() {
        let frames = build_frames(&[]);
        assert!(frames.is_empty());
    }

    #[test]
    fn test_build_frames_single() {
        let img = RgbaImage::new(1080, 1920);
        let frames = build_frames(&[img]);
        assert_eq!(frames.len(), 1);
    }
}
```

- [ ] **Step 2: Verify compilation and run tests**

```bash
cd src-tauri && cargo check && cargo test
```

Expected: unit tests pass.

- [ ] **Step 3: Commit**

```bash
git add -A && git commit -m "feat: implement render engine (frame sequence + FFmpeg)"
```

---

### Task 8: Wire Up Tauri IPC Commands

**Files:**
- Create: `src-tauri/src/commands.rs`

- [ ] **Step 1: Implement commands.rs — bridge between frontend and engine**

Create `src-tauri/src/commands.rs`:

```rust
use std::path::PathBuf;
use std::sync::Mutex;

use image::DynamicImage;
use serde::{Deserialize, Serialize};
use tauri::State;

use crate::engine::align::{self, AlignParams};
use crate::engine::face::{FaceBox, FaceDetector};
use crate::engine::render::{self, RenderParams};
use crate::engine::template::TemplateConfig;
use crate::error::AppResult;

pub struct AppState {
    pub detector: Mutex<Option<FaceDetector>>,
}

#[derive(Debug, Serialize, Deserialize)]
pub struct DetectFacesResult {
    pub photo_index: usize,
    pub faces: Vec<FaceBox>,
}

#[derive(Debug, Serialize, Deserialize)]
pub struct AlignPhotosInput {
    pub photo_paths: Vec<String>,
    pub selected_faces: Vec<Option<usize>>, // Face index per photo, None for fallback
    pub template_id: String,
}

#[derive(Debug, Serialize, Deserialize)]
pub struct RenderVideoInput {
    pub aligned_photo_paths: Vec<String>,
    pub output_path: String,
    pub template_id: String,
    pub bgm_path: Option<String>,
}

/// Load a template from disk and return its config.
#[tauri::command]
pub fn load_template(template_id: String) -> Result<TemplateConfig, String> {
    let path = format!("templates/{template_id}.yaml");
    TemplateConfig::load(&path).map_err(|e| e.to_string())
}

/// Detect faces for a list of photo file paths. Returns results per photo.
#[tauri::command]
pub fn detect_faces(
    photo_paths: Vec<String>,
    state: State<'_, AppState>,
) -> Result<Vec<DetectFacesResult>, String> {
    let mut detector_guard = state.detector.lock().map_err(|e| e.to_string())?;

    // Lazy-load the detector if not yet loaded
    if detector_guard.is_none() {
        let model_path = resolve_model_path()?;
        let detector = FaceDetector::load(&model_path).map_err(|e| e.to_string())?;
        *detector_guard = Some(detector);
    }

    let detector = detector_guard.as_ref().unwrap();
    let mut results = Vec::new();

    for (i, path) in photo_paths.iter().enumerate() {
        let img = image::open(path).map_err(|e| format!("Cannot open {path}: {e}"))?;
        let faces = detector.detect(&img).map_err(|e| e.to_string())?;
        results.push(DetectFacesResult {
            photo_index: i,
            faces,
        });
    }

    Ok(results)
}

/// Align photos given their paths and selected face indices.
/// Returns paths to aligned PNG files.
#[tauri::command]
pub fn align_photos(
    input: AlignPhotosInput,
) -> Result<Vec<String>, String> {
    let template = TemplateConfig::load(&format!("templates/{}.yaml", input.template_id))
        .map_err(|e| e.to_string())?;
    let align_params = template.align_params().map_err(|e| e.to_string())?;

    // Re-detect faces to get face data
    let model_path = resolve_model_path()?;
    let detector = FaceDetector::load(&model_path).map_err(|e| e.to_string())?;

    let mut output_paths = Vec::new();
    let tmp_dir = tempfile::tempdir().map_err(|e| e.to_string())?;

    for (i, path) in input.photo_paths.iter().enumerate() {
        let img = image::open(path).map_err(|e| format!("Cannot open {path}: {e}"))?;
        let faces = detector.detect(&img).map_err(|e| e.to_string())?;

        // Select the face the user chose
        let target_faces: Vec<FaceBox> = match &input.selected_faces[i] {
            Some(idx) if *idx < faces.len() => vec![faces[*idx].clone()],
            _ => faces, // Use first (highest confidence) if no selection
        };

        let aligned = align::align_photo(&img, &target_faces, &align_params)
            .map_err(|e| e.to_string())?;

        let out_path = tmp_dir.path().join(format!("aligned_{i:04d}.png"));
        aligned.save(&out_path).map_err(|e| e.to_string())?;
        output_paths.push(out_path.to_string_lossy().to_string());
    }

    // Keep tmp_dir alive by leaking it (production code would manage this better)
    // For MVP, we return paths; the temp dir will be cleaned when the app exits.
    std::mem::forget(tmp_dir);

    Ok(output_paths)
}

/// Render video from aligned photo paths.
#[tauri::command]
pub fn render_video(
    input: RenderVideoInput,
) -> Result<String, String> {
    let template = TemplateConfig::load(&format!("templates/{}.yaml", input.template_id))
        .map_err(|e| e.to_string())?;

    let out = &template.output.default;

    let render_params = RenderParams {
        out_width: out.width,
        out_height: out.height,
        fps_in: out.fps,
        fps_out: 30,
        codec: "h264_videotoolbox".into(),
        bitrate: "8M".into(),
        bgm_path: input.bgm_path.map(PathBuf::from),
    };

    let mut frames = Vec::new();
    for path in &input.aligned_photo_paths {
        let frame = image::open(path)
            .map_err(|e| format!("Cannot open {path}: {e}"))?
            .to_rgba8();
        frames.push(frame);
    }

    let output = PathBuf::from(&input.output_path);
    render::render_video(&frames, &output, &render_params)
        .map(|p| p.to_string_lossy().to_string())
        .map_err(|e| e.to_string())
}

fn resolve_model_path() -> Result<String, String> {
    // In Tauri, resources are resolved relative to the app bundle
    let resource_dir = std::env::current_dir()
        .map_err(|e| e.to_string())?
        .join("models")
        .join("yunet.onnx");

    if resource_dir.exists() {
        return Ok(resource_dir.to_string_lossy().to_string());
    }

    // Fallback: check relative to the executable
    let exe_dir = std::env::current_exe()
        .map_err(|e| e.to_string())?
        .parent()
        .ok_or("No parent dir")?
        .join("models")
        .join("yunet.onnx");

    if exe_dir.exists() {
        return Ok(exe_dir.to_string_lossy().to_string());
    }

    Err("YuNet model not found. Run scripts/download-model.sh first.".to_string())
}
```

- [ ] **Step 2: Update lib.rs to register AppState**

Edit `src-tauri/src/lib.rs`:

```rust
mod engine;
mod error;
mod commands;

use commands::AppState;
use std::sync::Mutex;

#[cfg_attr(mobile, tauri::mobile_entry_point)]
pub fn run() {
    tauri::Builder::default()
        .manage(AppState {
            detector: Mutex::new(None),
        })
        .invoke_handler(tauri::generate_handler![
            commands::detect_faces,
            commands::align_photos,
            commands::render_video,
            commands::load_template,
        ])
        .run(tauri::generate_context!())
        .expect("error while running tauri application");
}
```

- [ ] **Step 3: Verify compilation**

```bash
cd src-tauri && cargo check
```

Expected: no errors. (Commands will have warnings about unused imports — fine for now.)

- [ ] **Step 4: Commit**

```bash
git add -A && git commit -m "feat: wire up Tauri IPC commands"
```

---

### Task 9: Define Frontend Types and Pinia Store

**Files:**
- Create: `src/types/index.ts`
- Create: `src/stores/project.ts`
- Modify: `src/main.ts`

- [ ] **Step 1: Install Pinia**

```bash
npm install pinia
```

- [ ] **Step 2: Create TypeScript types**

Create `src/types/index.ts`:

```typescript
export interface FaceBox {
  bbox: [number, number, number, number];
  confidence: number;
  landmarks: [[number, number], [number, number], [number, number], [number, number], [number, number]];
}

export interface PhotoItem {
  id: string;
  filePath: string;
  fileName: string;
  thumbnailUrl: string;
  faces: FaceBox[];
  selectedFaceIndex: number | null; // null = not yet analyzed, -1 = no face
  status: 'pending' | 'analyzing' | 'done' | 'no-face';
}

export interface TemplateInfo {
  name: string;
  id: string;
  version: number;
  output: {
    default: {
      width: number;
      height: number;
      fps: number;
      duration_per_photo: number;
    };
  };
}

export interface OutputSettings {
  width: number;
  height: number;
  fps: number;
  durationPerPhoto: number;
  bgmPath: string | null;
}
```

- [ ] **Step 3: Create Pinia store**

Create `src/stores/project.ts`:

```typescript
import { defineStore } from 'pinia';
import { ref, computed } from 'vue';
import type { PhotoItem, TemplateInfo, OutputSettings, FaceBox } from '../types';
import { invoke } from '@tauri-apps/api/core';

export const useProjectStore = defineStore('project', () => {
  const photos = ref<PhotoItem[]>([]);
  const currentTemplate = ref<TemplateInfo | null>(null);
  const outputSettings = ref<OutputSettings>({
    width: 1080,
    height: 1920,
    fps: 5,
    durationPerPhoto: 0.2,
    bgmPath: null,
  });
  const statusMessage = ref<string>('就绪');
  const isProcessing = ref(false);
  const outputVideoPath = ref<string | null>(null);

  const photoCount = computed(() => photos.value.length);

  let nextId = 1;

  function addPhotos(filePaths: string[]) {
    for (const fp of filePaths) {
      const fileName = fp.split('/').pop() || fp;
      photos.value.push({
        id: String(nextId++),
        filePath: fp,
        fileName,
        thumbnailUrl: fp, // In Tauri, we can use convertFileSrc for local files
        faces: [],
        selectedFaceIndex: null,
        status: 'pending',
      });
    }
  }

  function removePhoto(id: string) {
    photos.value = photos.value.filter(p => p.id !== id);
  }

  function reorderPhotos(fromIndex: number, toIndex: number) {
    const item = photos.value.splice(fromIndex, 1)[0];
    photos.value.splice(toIndex, 0, item);
  }

  async function loadTemplate(templateId: string) {
    currentTemplate.value = await invoke<TemplateInfo>('load_template', { templateId });
    if (currentTemplate.value) {
      const def = currentTemplate.value.output.default;
      outputSettings.value = {
        width: def.width,
        height: def.height,
        fps: def.fps,
        durationPerPhoto: def.duration_per_photo,
        bgmPath: outputSettings.value.bgmPath,
      };
    }
  }

  async function detectFaces() {
    isProcessing.value = true;
    statusMessage.value = '分析人脸中...';

    try {
      const paths = photos.value.map(p => p.filePath);
      const results = await invoke<any[]>('detect_faces', { photoPaths: paths });

      for (const r of results) {
        const photo = photos.value[r.photo_index];
        if (!photo) continue;
        photo.faces = r.faces;
        if (r.faces.length === 0) {
          photo.status = 'no-face';
        } else if (r.faces.length === 1) {
          photo.selectedFaceIndex = 0;
          photo.status = 'done';
        } else {
          photo.status = 'done';
          // Multi-face: user must select
        }
      }

      statusMessage.value = '人脸分析完成';
    } catch (err) {
      statusMessage.value = `分析失败: ${err}`;
    } finally {
      isProcessing.value = false;
    }
  }

  function selectFace(photoId: string, faceIndex: number) {
    const photo = photos.value.find(p => p.id === photoId);
    if (photo) {
      photo.selectedFaceIndex = faceIndex;
    }
  }

  async function generateVideo() {
    isProcessing.value = true;
    statusMessage.value = '生成视频中...';

    try {
      const alignedPaths = await invoke<string[]>('align_photos', {
        input: {
          photoPaths: photos.value.map(p => p.filePath),
          selectedFaces: photos.value.map(p => p.selectedFaceIndex),
          templateId: currentTemplate.value?.id || 'silent-grow',
        },
      });

      const outputPath = await invoke<string>('render_video', {
        input: {
          alignedPhotoPaths: alignedPaths,
          outputPath: '/tmp/hey24-output.mp4',
          templateId: currentTemplate.value?.id || 'silent-grow',
          bgmPath: outputSettings.value.bgmPath,
        },
      });

      outputVideoPath.value = outputPath;
      statusMessage.value = `视频已生成: ${outputPath}`;
    } catch (err) {
      statusMessage.value = `生成失败: ${err}`;
    } finally {
      isProcessing.value = false;
    }
  }

  return {
    photos,
    currentTemplate,
    outputSettings,
    statusMessage,
    isProcessing,
    outputVideoPath,
    photoCount,
    addPhotos,
    removePhoto,
    reorderPhotos,
    loadTemplate,
    detectFaces,
    selectFace,
    generateVideo,
  };
});
```

- [ ] **Step 4: Update main.ts to use Pinia**

Edit `src/main.ts`:

```typescript
import { createApp } from 'vue';
import { createPinia } from 'pinia';
import App from './App.vue';
import './style.css';

const app = createApp(App);
app.use(createPinia());
app.mount('#app');
```

- [ ] **Step 5: Verify**

```bash
npm run build
```

Expected: Build succeeds without errors.

- [ ] **Step 6: Commit**

```bash
git add -A && git commit -m "feat: define frontend types, Pinia store, and Tauri IPC integration"
```

---

### Task 10: Build Vue 3 Components — Layout Shell

**Files:**
- Create: `src/style.css`
- Modify: `src/App.vue`

- [ ] **Step 1: Create global styles**

Create `src/style.css`:

```css
:root {
  --bg-primary: #1a1a2e;
  --bg-secondary: #16213e;
  --bg-card: #0f3460;
  --accent: #e94560;
  --text-primary: #eee;
  --text-secondary: #aaa;
  --border: #2a2a4a;
  --radius: 8px;
  --font-mono: 'SF Mono', 'Fira Code', monospace;
}

* {
  margin: 0;
  padding: 0;
  box-sizing: border-box;
}

body {
  font-family: -apple-system, BlinkMacSystemFont, 'Segoe UI', sans-serif;
  background: var(--bg-primary);
  color: var(--text-primary);
  overflow: hidden;
  height: 100vh;
}

#app {
  height: 100vh;
  display: flex;
  flex-direction: column;
}

button {
  cursor: pointer;
  border: none;
  border-radius: var(--radius);
  padding: 8px 16px;
  font-size: 14px;
  transition: opacity 0.2s;
}

button:hover {
  opacity: 0.85;
}

button:disabled {
  opacity: 0.4;
  cursor: not-allowed;
}

.btn-primary {
  background: var(--accent);
  color: white;
  font-weight: 600;
}

.btn-secondary {
  background: var(--bg-card);
  color: var(--text-primary);
  border: 1px solid var(--border);
}

input, select {
  background: var(--bg-secondary);
  color: var(--text-primary);
  border: 1px solid var(--border);
  border-radius: var(--radius);
  padding: 6px 10px;
  font-size: 13px;
}
```

- [ ] **Step 2: Create App.vue layout shell**

Edit `src/App.vue`:

```vue
<script setup lang="ts">
import { onMounted } from 'vue';
import TemplateList from './components/TemplateList.vue';
import PhotoTimeline from './components/PhotoTimeline.vue';
import PreviewPanel from './components/PreviewPanel.vue';
import StatusBar from './components/StatusBar.vue';
import { useProjectStore } from './stores/project';

const store = useProjectStore();

onMounted(() => {
  store.loadTemplate('silent-grow');
});
</script>

<template>
  <div class="app-layout">
    <header class="app-header">
      <h1 class="app-title">🎬 Hey24</h1>
      <span class="template-badge" v-if="store.currentTemplate">
        模板: {{ store.currentTemplate.name }}
      </span>
    </header>

    <main class="app-main">
      <TemplateList />
      <PhotoTimeline />
      <PreviewPanel />
    </main>

    <StatusBar />
  </div>
</template>

<style scoped>
.app-layout {
  display: flex;
  flex-direction: column;
  height: 100vh;
}

.app-header {
  display: flex;
  align-items: center;
  gap: 16px;
  padding: 12px 20px;
  background: var(--bg-secondary);
  border-bottom: 1px solid var(--border);
  -webkit-app-region: drag;
}

.app-title {
  font-size: 18px;
  font-weight: 700;
}

.template-badge {
  font-size: 12px;
  color: var(--text-secondary);
  background: var(--bg-card);
  padding: 2px 10px;
  border-radius: 12px;
}

.app-main {
  display: flex;
  flex: 1;
  overflow: hidden;
}
</style>
```

- [ ] **Step 3: Verify**

```bash
npm run build
```

Expected: Build succeeds.

- [ ] **Step 4: Commit**

```bash
git add -A && git commit -m "feat: create app layout shell and global styles"
```

---

### Task 11: Build Vue 3 Components — TemplateList

**Files:**
- Create: `src/components/TemplateList.vue`

- [ ] **Step 1: Create TemplateList.vue**

Create `src/components/TemplateList.vue`:

```vue
<script setup lang="ts">
import { useProjectStore } from '../stores/project';

const store = useProjectStore();

const templates = [
  { id: 'silent-grow', name: '悄悄长大', desc: '成长记录，0.2s快切' },
  { id: 'birthday', name: '生日快剪', desc: '即将推出', disabled: true },
  { id: 'travel', name: '旅行日记', desc: '即将推出', disabled: true },
];

function selectTemplate(id: string) {
  store.loadTemplate(id);
}
</script>

<template>
  <aside class="template-list">
    <h3 class="section-title">模板</h3>
    <div
      v-for="tpl in templates"
      :key="tpl.id"
      class="template-item"
      :class="{
        active: store.currentTemplate?.id === tpl.id,
        disabled: tpl.disabled,
      }"
      @click="!tpl.disabled && selectTemplate(tpl.id)"
    >
      <div class="tpl-name">{{ tpl.name }}</div>
      <div class="tpl-desc">{{ tpl.desc }}</div>
    </div>

    <div class="output-settings">
      <h3 class="section-title">输出设置</h3>
      <label>分辨率</label>
      <select v-model.number="store.outputSettings.width">
        <option :value="1080">1080×1920 (竖屏)</option>
        <option :value="720">720×1280</option>
      </select>
      <label>每张时长 (秒)</label>
      <input type="number" v-model.number="store.outputSettings.durationPerPhoto" step="0.1" min="0.1" max="2.0" />
    </div>
  </aside>
</template>

<style scoped>
.template-list {
  width: 200px;
  min-width: 200px;
  background: var(--bg-secondary);
  border-right: 1px solid var(--border);
  padding: 16px 12px;
  overflow-y: auto;
  display: flex;
  flex-direction: column;
  gap: 8px;
}

.section-title {
  font-size: 11px;
  text-transform: uppercase;
  letter-spacing: 1px;
  color: var(--text-secondary);
  margin-bottom: 4px;
}

.template-item {
  padding: 10px;
  border-radius: var(--radius);
  cursor: pointer;
  border: 1px solid transparent;
  transition: border-color 0.2s;
}

.template-item:hover:not(.disabled) {
  border-color: var(--accent);
}

.template-item.active {
  background: var(--bg-card);
  border-color: var(--accent);
}

.template-item.disabled {
  opacity: 0.4;
  cursor: not-allowed;
}

.tpl-name {
  font-size: 14px;
  font-weight: 600;
}

.tpl-desc {
  font-size: 11px;
  color: var(--text-secondary);
  margin-top: 2px;
}

.output-settings {
  margin-top: auto;
  display: flex;
  flex-direction: column;
  gap: 4px;
}

.output-settings label {
  font-size: 11px;
  color: var(--text-secondary);
  margin-top: 8px;
}
</style>
```

- [ ] **Step 2: Verify build**

```bash
npm run build
```

- [ ] **Step 3: Commit**

```bash
git add -A && git commit -m "feat: create TemplateList component"
```

---

### Task 12: Build Vue 3 Components — PhotoTimeline + PhotoCard

**Files:**
- Create: `src/components/PhotoTimeline.vue`
- Create: `src/components/PhotoCard.vue`

- [ ] **Step 1: Create PhotoCard.vue**

Create `src/components/PhotoCard.vue`:

```vue
<script setup lang="ts">
import { computed } from 'vue';
import type { PhotoItem } from '../types';
import { useProjectStore } from '../stores/project';

const props = defineProps<{ photo: PhotoItem }>();
const emit = defineEmits<{ select: [photoId: string]; remove: [photoId: string] }>();

const store = useProjectStore();

const statusIcon = computed(() => {
  if (props.photo.status === 'analyzing') return '⏳';
  if (props.photo.status === 'no-face') return '⚠️';
  if (props.photo.faces.length > 1) return '👥';
  if (props.photo.status === 'done') return '✅';
  return '';
});

const statusClass = computed(() => `status-${props.photo.status}`);
</script>

<template>
  <div class="photo-card" :class="statusClass">
    <img
      :src="photo.thumbnailUrl"
      :alt="photo.fileName"
      class="photo-thumb"
      draggable="false"
    />
    <div class="photo-overlay">
      <span class="status-icon">{{ statusIcon }}</span>
      <button class="remove-btn" @click.stop="emit('remove', photo.id)">×</button>
    </div>
    <div class="photo-name">{{ photo.fileName }}</div>
  </div>
</template>

<style scoped>
.photo-card {
  position: relative;
  width: 80px;
  min-width: 80px;
  height: 140px;
  border-radius: 6px;
  overflow: hidden;
  border: 2px solid transparent;
  cursor: pointer;
  transition: border-color 0.2s;
}

.photo-card:hover {
  border-color: var(--accent);
}

.photo-card.status-no-face {
  border-color: #f0ad4e;
}

.photo-card.status-done {
  border-color: #5cb85c;
}

.photo-thumb {
  width: 100%;
  height: 100%;
  object-fit: cover;
}

.photo-overlay {
  position: absolute;
  top: 4px;
  right: 4px;
  display: flex;
  gap: 4px;
}

.status-icon {
  font-size: 14px;
}

.remove-btn {
  background: rgba(0, 0, 0, 0.6);
  color: white;
  border: none;
  border-radius: 50%;
  width: 20px;
  height: 20px;
  font-size: 12px;
  display: flex;
  align-items: center;
  justify-content: center;
  padding: 0;
}

.photo-name {
  position: absolute;
  bottom: 0;
  left: 0;
  right: 0;
  background: rgba(0, 0, 0, 0.5);
  font-size: 9px;
  padding: 2px 4px;
  white-space: nowrap;
  overflow: hidden;
  text-overflow: ellipsis;
}
</style>
```

- [ ] **Step 2: Create PhotoTimeline.vue**

Create `src/components/PhotoTimeline.vue`:

```vue
<script setup lang="ts">
import { useProjectStore } from '../stores/project';
import PhotoCard from './PhotoCard.vue';

const store = useProjectStore();

function handleDrop(event: DragEvent) {
  event.preventDefault();
  // Drag-and-drop file handling will use Tauri's drag-drop plugin
}

function handleFileSelect() {
  // Will be wired to Tauri dialog
  console.log('Select files...');
}

function handleRemove(photoId: string) {
  store.removePhoto(photoId);
}
</script>

<template>
  <section
    class="photo-timeline"
    @drop.prevent="handleDrop"
    @dragover.prevent
  >
    <div class="timeline-header">
      <h3>照片 ({{ store.photoCount }})</h3>
      <div class="timeline-actions">
        <button class="btn-secondary" @click="handleFileSelect">+ 添加照片</button>
        <button
          class="btn-primary"
          :disabled="store.photoCount === 0 || store.isProcessing"
          @click="store.detectFaces()"
        >
          分析人脸
        </button>
      </div>
    </div>

    <div
      class="photo-strip"
      v-if="store.photoCount > 0"
    >
      <PhotoCard
        v-for="photo in store.photos"
        :key="photo.id"
        :photo="photo"
        @remove="handleRemove"
      />
    </div>

    <div class="empty-state" v-else>
      <p>拖入照片或点击「+ 添加照片」开始</p>
    </div>
  </section>
</template>

<style scoped>
.photo-timeline {
  flex: 1;
  display: flex;
  flex-direction: column;
  background: var(--bg-primary);
  padding: 16px;
  overflow: hidden;
}

.timeline-header {
  display: flex;
  justify-content: space-between;
  align-items: center;
  margin-bottom: 12px;
}

.timeline-header h3 {
  font-size: 14px;
}

.timeline-actions {
  display: flex;
  gap: 8px;
}

.photo-strip {
  display: flex;
  gap: 8px;
  overflow-x: auto;
  padding-bottom: 8px;
  flex: 1;
  align-items: flex-start;
}

.empty-state {
  flex: 1;
  display: flex;
  align-items: center;
  justify-content: center;
  border: 2px dashed var(--border);
  border-radius: var(--radius);
  color: var(--text-secondary);
}
</style>
```

- [ ] **Step 3: Verify build**

```bash
npm run build
```

- [ ] **Step 4: Commit**

```bash
git add -A && git commit -m "feat: create PhotoTimeline and PhotoCard components"
```

---

### Task 13: Build Vue 3 Components — FaceSelector Modal

**Files:**
- Create: `src/components/FaceSelector.vue`

- [ ] **Step 1: Create FaceSelector.vue**

Create `src/components/FaceSelector.vue`:

```vue
<script setup lang="ts">
import { ref } from 'vue';
import type { FaceBox } from '../types';

const props = defineProps<{
  photoUrl: string;
  faces: FaceBox[];
  photoWidth: number;
  photoHeight: number;
}>();

const emit = defineEmits<{
  select: [faceIndex: number];
  close: [];
}>();

function handleFaceClick(index: number) {
  emit('select', index);
}

function faceStyle(face: FaceBox) {
  // Convert normalized coords to percentage for overlay positioning
  return {
    left: `${face.bbox[0] * 100}%`,
    top: `${face.bbox[1] * 100}%`,
    width: `${face.bbox[2] * 100}%`,
    height: `${face.bbox[3] * 100}%`,
  };
}
</script>

<template>
  <div class="modal-backdrop" @click.self="emit('close')">
    <div class="modal-content">
      <div class="modal-header">
        <h3>选择目标人物</h3>
        <button class="btn-secondary" @click="emit('close')">✕</button>
      </div>
      <div class="face-image-container">
        <img :src="photoUrl" class="face-preview" />
        <div
          v-for="(face, i) in faces"
          :key="i"
          class="face-box"
          :style="faceStyle(face)"
          @click.stop="handleFaceClick(i)"
        >
          <span class="face-label">{{ i + 1 }}</span>
        </div>
      </div>
      <p class="face-hint">点击人脸框选择目标人物</p>
    </div>
  </div>
</template>

<style scoped>
.modal-backdrop {
  position: fixed;
  inset: 0;
  background: rgba(0, 0, 0, 0.7);
  display: flex;
  align-items: center;
  justify-content: center;
  z-index: 1000;
}

.modal-content {
  background: var(--bg-secondary);
  border-radius: 12px;
  padding: 20px;
  max-width: 90vw;
  max-height: 90vh;
}

.modal-header {
  display: flex;
  justify-content: space-between;
  align-items: center;
  margin-bottom: 16px;
}

.face-image-container {
  position: relative;
  display: inline-block;
  max-height: 70vh;
}

.face-preview {
  max-height: 65vh;
  max-width: 80vw;
  object-fit: contain;
}

.face-box {
  position: absolute;
  border: 2px solid var(--accent);
  border-radius: 4px;
  cursor: pointer;
  transition: background 0.2s;
}

.face-box:hover {
  background: rgba(233, 69, 96, 0.2);
}

.face-label {
  position: absolute;
  top: -20px;
  left: 50%;
  transform: translateX(-50%);
  background: var(--accent);
  color: white;
  font-size: 11px;
  padding: 1px 6px;
  border-radius: 8px;
}

.face-hint {
  text-align: center;
  margin-top: 12px;
  font-size: 12px;
  color: var(--text-secondary);
}
</style>
```

- [ ] **Step 2: Verify build**

```bash
npm run build
```

- [ ] **Step 3: Commit**

```bash
git add -A && git commit -m "feat: create FaceSelector modal component"
```

---

### Task 14: Build Vue 3 Components — PreviewPanel and StatusBar

**Files:**
- Create: `src/components/PreviewPanel.vue`
- Create: `src/components/StatusBar.vue`

- [ ] **Step 1: Create PreviewPanel.vue**

Create `src/components/PreviewPanel.vue`:

```vue
<script setup lang="ts">
import { useProjectStore } from '../stores/project';

const store = useProjectStore();
</script>

<template>
  <aside class="preview-panel">
    <h3 class="section-title">预览</h3>

    <div class="preview-area">
      <div class="preview-placeholder" v-if="!store.outputVideoPath">
        <span>👆 上传照片后点击生成</span>
      </div>
      <video
        v-else
        :src="store.outputVideoPath"
        controls
        class="preview-video"
      />
    </div>

    <button
      class="btn-primary generate-btn"
      :disabled="store.photoCount === 0 || store.isProcessing"
      @click="store.generateVideo()"
    >
      ▶ 一键生成
    </button>
  </aside>
</template>

<style scoped>
.preview-panel {
  width: 280px;
  min-width: 280px;
  background: var(--bg-secondary);
  border-left: 1px solid var(--border);
  padding: 16px;
  display: flex;
  flex-direction: column;
  gap: 12px;
}

.section-title {
  font-size: 11px;
  text-transform: uppercase;
  letter-spacing: 1px;
  color: var(--text-secondary);
}

.preview-area {
  flex: 1;
  background: black;
  border-radius: var(--radius);
  display: flex;
  align-items: center;
  justify-content: center;
  aspect-ratio: 9 / 16;
  max-height: 60vh;
  overflow: hidden;
}

.preview-placeholder {
  color: var(--text-secondary);
  font-size: 13px;
  text-align: center;
  padding: 16px;
}

.preview-video {
  width: 100%;
  height: 100%;
  object-fit: contain;
}

.generate-btn {
  width: 100%;
  padding: 12px;
  font-size: 16px;
}
</style>
```

- [ ] **Step 2: Create StatusBar.vue**

Create `src/components/StatusBar.vue`:

```vue
<script setup lang="ts">
import { useProjectStore } from '../stores/project';

const store = useProjectStore();
</script>

<template>
  <footer class="status-bar">
    <span class="status-text">{{ store.statusMessage }}</span>
    <span class="status-right" v-if="store.isProcessing">
      <span class="spinner" />
    </span>
  </footer>
</template>

<style scoped>
.status-bar {
  display: flex;
  align-items: center;
  justify-content: space-between;
  padding: 6px 16px;
  background: var(--bg-secondary);
  border-top: 1px solid var(--border);
  font-size: 12px;
  color: var(--text-secondary);
}

.status-right {
  display: flex;
  align-items: center;
}

.spinner {
  display: inline-block;
  width: 12px;
  height: 12px;
  border: 2px solid var(--text-secondary);
  border-top-color: var(--accent);
  border-radius: 50%;
  animation: spin 0.8s linear infinite;
}

@keyframes spin {
  to { transform: rotate(360deg); }
}
</style>
```

- [ ] **Step 3: Verify build**

```bash
npm run build
```

- [ ] **Step 4: Commit**

```bash
git add -A && git commit -m "feat: create PreviewPanel and StatusBar components"
```

---

### Task 15: Integration — File Import via Tauri Drag-Drop

**Files:**
- Modify: `src/components/PhotoTimeline.vue`
- Modify: `src-tauri/src/lib.rs`

- [ ] **Step 1: Enable Tauri drag-drop plugin**

Edit `src-tauri/src/lib.rs` to add the plugin:

```rust
mod engine;
mod error;
mod commands;

use commands::AppState;
use std::sync::Mutex;

#[cfg_attr(mobile, tauri::mobile_entry_point)]
pub fn run() {
    tauri::Builder::default()
        .plugin(tauri_plugin_dialog::init())
        .plugin(tauri_plugin_fs::init())
        .manage(AppState {
            detector: Mutex::new(None),
        })
        .invoke_handler(tauri::generate_handler![
            commands::detect_faces,
            commands::align_photos,
            commands::render_video,
            commands::load_template,
        ])
        .run(tauri::generate_context!())
        .expect("error while running tauri application");
}
```

- [ ] **Step 2: Add file dialog and drag-drop to PhotoTimeline**

Edit `src/components/PhotoTimeline.vue` to add Tauri dialog import:

```typescript
import { open } from '@tauri-apps/plugin-dialog';
```

Update `handleFileSelect`:

```typescript
async function handleFileSelect() {
  try {
    const selected = await open({
      multiple: true,
      filters: [{
        name: 'Images',
        extensions: ['png', 'jpg', 'jpeg', 'heic', 'webp'],
      }],
    });
    if (selected) {
      const paths = Array.isArray(selected) ? selected : [selected];
      store.addPhotos(paths);
    }
  } catch (err) {
    console.error('File dialog failed:', err);
  }
}
```

- [ ] **Step 3: Handle drag-drop on the timeline**

Add `onMounted` listener for Tauri drag-drop events in PhotoTimeline.vue:

```typescript
import { onMounted, onUnmounted } from 'vue';
import { listen } from '@tauri-apps/api/event';

let unlisten: (() => void) | null = null;

onMounted(async () => {
  unlisten = await listen<string[]>('tauri://drag-drop', (event) => {
    const paths = event.payload;
    const imagePaths = paths.filter(p =>
      /\.(png|jpe?g|heic|webp)$/i.test(p)
    );
    if (imagePaths.length > 0) {
      store.addPhotos(imagePaths);
    }
  });
});

onUnmounted(() => {
  unlisten?.();
});
```

- [ ] **Step 4: Install Tauri frontend dependencies**

```bash
npm install @tauri-apps/plugin-dialog @tauri-apps/plugin-fs
```

- [ ] **Step 5: Enable drag-drop in tauri.conf.json**

Ensure `tauri.conf.json` has drag-drop enabled:

```json
{
  "app": {
    "windows": [
      {
        "title": "Hey24",
        "width": 1200,
        "height": 800,
        "dragDropEnabled": true
      }
    ]
  }
}
```

- [ ] **Step 6: Commit**

```bash
git add -A && git commit -m "feat: add file import via Tauri drag-drop and dialog"
```

---

### Task 16: Integration — Connect FaceSelector to PhotoTimeline

**Files:**
- Modify: `src/components/PhotoTimeline.vue`

- [ ] **Step 1: Add FaceSelector modal integration to PhotoTimeline**

Add face selection state and modal to PhotoTimeline.vue:

```vue
<script setup lang="ts">
import { ref } from 'vue';
// ... existing imports ...
import FaceSelector from './FaceSelector.vue';
import type { PhotoItem } from '../types';

// ... existing code ...

const selectedPhotoForFaces = ref<PhotoItem | null>(null);

function handlePhotoClick(photo: PhotoItem) {
  if (photo.faces.length > 1) {
    selectedPhotoForFaces.value = photo;
  }
}

function handleFaceSelect(faceIndex: number) {
  if (selectedPhotoForFaces.value) {
    store.selectFace(selectedPhotoForFaces.value.id, faceIndex);
    selectedPhotoForFaces.value = null;
  }
}
</script>

<template>
  <!-- ... existing template ... -->
  <PhotoCard
    v-for="photo in store.photos"
    :key="photo.id"
    :photo="photo"
    @remove="handleRemove"
    @click="handlePhotoClick(photo)"
  />

  <FaceSelector
    v-if="selectedPhotoForFaces"
    :photo-url="selectedPhotoForFaces.thumbnailUrl"
    :faces="selectedPhotoForFaces.faces"
    :photo-width="1080"
    :photo-height="1920"
    @select="handleFaceSelect"
    @close="selectedPhotoForFaces = null"
  />
</template>
```

- [ ] **Step 2: Update PhotoCard to emit click event**

In `PhotoCard.vue`, make the entire card clickable by emitting click on the root element (or use `@click` on parent).

- [ ] **Step 3: Commit**

```bash
git add -A && git commit -m "feat: wire FaceSelector modal to PhotoTimeline"
```

---

### Task 17: Packaging — .dmg Build Configuration

**Files:**
- Modify: `src-tauri/tauri.conf.json`
- Create: `src-tauri/icons/` (app icon placeholder)

- [ ] **Step 1: Update tauri.conf.json for macOS bundling**

Ensure `src-tauri/tauri.conf.json` has proper bundle config:

```json
{
  "bundle": {
    "active": true,
    "targets": "all",
    "icon": [
      "icons/32x32.png",
      "icons/128x128.png",
      "icons/128x128@2x.png",
      "icons/icon.icns",
      "icons/icon.ico"
    ],
    "resources": [
      "models/yunet.onnx",
      "templates/*"
    ],
    "externalBin": [],
    "copyright": "",
    "category": "Video",
    "shortDescription": "AI-powered video clipping tool",
    "longDescription": "Hey24 - 智能视频剪辑工具，选择模板后自动生成剪辑视频。"
  }
}
```

- [ ] **Step 2: Create a placeholder icon**

```bash
# Generate a simple icon using macOS sips (or skip for MVP)
mkdir -p src-tauri/icons
# Tauri will use default icon if none provided
```

- [ ] **Step 3: Build macOS .dmg**

```bash
npx tauri build
```

Expected: Produces `src-tauri/target/release/bundle/dmg/hey24_*.dmg`.

- [ ] **Step 4: Verify the .dmg**

```bash
ls -lh src-tauri/target/release/bundle/dmg/
```

Expected: `.dmg` file listed.

- [ ] **Step 5: Commit**

```bash
git add -A && git commit -m "feat: configure macOS .dmg packaging"
```

---

### Task 18: Polish — Error Handling & Edge Cases

**Files:**
- Modify: `src-tauri/src/engine/render.rs`
- Modify: `src/stores/project.ts`

- [ ] **Step 1: Add FFmpeg availability check**

Add to `src-tauri/src/engine/render.rs`:

```rust
/// Check if FFmpeg is available. Returns its version string or an error.
pub fn check_ffmpeg() -> AppResult<String> {
    let output = Command::new("ffmpeg")
        .arg("-version")
        .output()
        .map_err(|_| AppError::Render(
            "FFmpeg is not installed. Please install FFmpeg:\n  brew install ffmpeg".into()
        ))?;

    let version = String::from_utf8_lossy(&output.stdout)
        .lines()
        .next()
        .unwrap_or("unknown")
        .to_string();

    Ok(version)
}
```

- [ ] **Step 2: Add FFmpeg check command to commands.rs**

```rust
#[tauri::command]
pub fn check_system() -> Result<String, String> {
    render::check_ffmpeg().map_err(|e| e.to_string())
}
```

Register in `lib.rs` `generate_handler!`:

```rust
commands::check_system,
```

- [ ] **Step 3: Call system check on app startup**

Add to `src/stores/project.ts`:

```typescript
async function checkSystem() {
  try {
    const version = await invoke<string>('check_system');
    statusMessage.value = `FFmpeg: ${version.split('\n')[0]}`;
  } catch (err) {
    statusMessage.value = `⚠️ ${err}`;
  }
}

// Call in onMounted of App.vue or EditorView
```

- [ ] **Step 4: Commit**

```bash
git add -A && git commit -m "feat: add FFmpeg availability check and error handling"
```

---

## Summary

**18 tasks** covering the complete MVP — from project scaffolding through face detection, alignment, rendering, frontend UI, and packaging.

**Dependencies between tasks:**
- Task 1 (scaffold) → everything else
- Tasks 2-3 (deps + model) → Tasks 4-7 (Rust engine modules)
- Task 4 (face detection) → Task 5 (align needs face data) → Task 7 (render needs aligned images)
- Task 6 (template) can be done in parallel with 4-5
- Tasks 4-7 → Task 8 (Tauri commands need all engine modules)
- Tasks 9 (types+store) → Tasks 10-14 (Vue components)
- Task 8 (commands) + Tasks 10-14 (components) → Tasks 15-16 (integration)
- Everything → Task 17 (packaging)
- Task 18 (polish) can be done anytime after Task 7

**Suggested execution order for maximum parallelism:**
1. Tasks 1-3 (scaffold + deps + model) — sequential
2. Tasks 4, 5, 6, 7 (Rust engine) — sequential (dependencies within)
3. Task 8 (commands) — after engine
4. Tasks 9-14 (frontend) — parallel with Rust engine if desired
5. Tasks 15-16 (integration) — after commands + frontend
6. Task 17 (packaging) — last
7. Task 18 (polish) — anytime
