use std::path::PathBuf;
use std::sync::Mutex;
use serde::{Deserialize, Serialize};
use tauri::State;

use crate::engine::align;
use crate::engine::face::{FaceBox, FaceDetector};
use crate::engine::render::{self, RenderParams};
use crate::engine::template::TemplateConfig;

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
    pub selected_faces: Vec<Option<usize>>,
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
    let path = format!("templates/{}.yaml", template_id);
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

    let detector = detector_guard.as_mut().unwrap();
    let mut results = Vec::new();

    for (i, path) in photo_paths.iter().enumerate() {
        let img = image::open(path).map_err(|e| format!("Cannot open {}: {}", path, e))?;
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
pub fn align_photos(input: AlignPhotosInput) -> Result<Vec<String>, String> {
    let template = TemplateConfig::load(&format!("templates/{}.yaml", input.template_id))
        .map_err(|e| e.to_string())?;
    let align_params = template.align_params().map_err(|e| e.to_string())?;

    let model_path = resolve_model_path()?;
    let mut detector = FaceDetector::load(&model_path).map_err(|e| e.to_string())?;

    let mut output_paths = Vec::new();
    let tmp_dir = tempfile::tempdir().map_err(|e| e.to_string())?;

    for (i, path) in input.photo_paths.iter().enumerate() {
        let img = image::open(path).map_err(|e| format!("Cannot open {}: {}", path, e))?;
        let faces = detector.detect(&img).map_err(|e| e.to_string())?;

        let target_faces: Vec<FaceBox> = match &input.selected_faces.get(i) {
            Some(Some(idx)) if *idx < faces.len() => vec![faces[*idx].clone()],
            _ => faces,
        };

        let aligned = align::align_photo(&img, &target_faces, &align_params)
            .map_err(|e| e.to_string())?;

        let out_path = tmp_dir.path().join(format!("aligned_{:04}.png", i));
        aligned.save(&out_path).map_err(|e| e.to_string())?;
        output_paths.push(out_path.to_string_lossy().to_string());
    }

    std::mem::forget(tmp_dir);
    Ok(output_paths)
}

/// Render video from aligned photo paths.
#[tauri::command]
pub fn render_video(input: RenderVideoInput) -> Result<String, String> {
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
            .map_err(|e| format!("Cannot open {}: {}", path, e))?
            .to_rgba8();
        frames.push(frame);
    }

    let output = PathBuf::from(&input.output_path);
    render::render_video(&frames, &output, &render_params)
        .map(|p| p.to_string_lossy().to_string())
        .map_err(|e| e.to_string())
}

/// Check system requirements (FFmpeg availability).
#[tauri::command]
pub fn check_system() -> Result<String, String> {
    render::check_ffmpeg().map_err(|e| e.to_string())
}

fn resolve_model_path() -> Result<String, String> {
    let cwd = std::env::current_dir().map_err(|e| e.to_string())?;

    // Try development paths
    let dev_path = cwd.join("models").join("yunet.onnx");
    if dev_path.exists() {
        return Ok(dev_path.to_string_lossy().to_string());
    }

    // Try relative to executable
    if let Ok(exe) = std::env::current_exe() {
        if let Some(exe_dir) = exe.parent() {
            let exe_path = exe_dir.join("models").join("yunet.onnx");
            if exe_path.exists() {
                return Ok(exe_path.to_string_lossy().to_string());
            }
        }
    }

    Err("YuNet model not found. Run scripts/download-model.sh first.".to_string())
}
