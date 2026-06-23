use image::DynamicImage;
use ort::session::Session;
use ort::value::Tensor;
use serde::{Deserialize, Serialize};

use crate::error::AppError;
use crate::error::AppResult;


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
            .map_err(|e| AppError::FaceDetection(format!("Failed to create session builder: {e}")))?
            .commit_from_file(model_path)
            .map_err(|e| AppError::FaceDetection(format!("Failed to load model from {model_path}: {e}")))?;
        Ok(Self { session })
    }

    /// Detect faces in an image.
    pub fn detect(&mut self, img: &DynamicImage) -> AppResult<Vec<FaceBox>> {
        // 1. Resize to 320x320 using Lanczos3 filter
        let resized = img.resize_exact(INPUT_SIZE, INPUT_SIZE, image::imageops::Lanczos3);
        let rgb = resized.to_rgb8();
        let (width, height) = rgb.dimensions();

        // 2. Normalize to CHW float tensor: (pixel - 127.5) / 128.0
        let mut data = Vec::with_capacity((INPUT_SIZE * INPUT_SIZE * 3) as usize);
        for c in 0..3 {
            for y in 0..height {
                for x in 0..width {
                    let pixel = rgb.get_pixel(x, y)[c];
                    data.push((pixel as f32 - 127.5) / 128.0);
                }
            }
        }

        // 3. Create input tensor with shape [1, 3, 320, 320]
        // Tensor::from_array accepts a tuple of (shape, data) where shape is [usize; N]
        // and data is Vec<T> or Box<[T]>
        let shape = [1usize, 3, INPUT_SIZE as usize, INPUT_SIZE as usize];
        let input_tensor = Tensor::from_array((shape, data))
            .map_err(|e| AppError::FaceDetection(format!("Failed to create input tensor: {e}")))?;

        // 4. Run inference
        let outputs = self
            .session
            .run(ort::inputs! { "input" => input_tensor })
            .map_err(|e| AppError::FaceDetection(format!("Inference failed: {e}")))?;

        // 5. Parse outputs
        // scores: [1, N, 1], bboxes: [1, N, 4] as [cx, cy, w, h], landmarks: [1, N, 10]
        // try_extract_array returns ndarray::ArrayViewD which supports direct indexing
        let scores = outputs["score"]
            .try_extract_array::<f32>()
            .map_err(|e| AppError::FaceDetection(format!("Failed to extract scores: {e}")))?;
        let bboxes = outputs["bbox"]
            .try_extract_array::<f32>()
            .map_err(|e| AppError::FaceDetection(format!("Failed to extract bboxes: {e}")))?;
        let landmarks = outputs["landmarks"]
            .try_extract_array::<f32>()
            .map_err(|e| AppError::FaceDetection(format!("Failed to extract landmarks: {e}")))?;

        // Sanity check: verify expected shapes
        let num_detections = scores.shape().get(1).copied().unwrap_or(0);

        let mut faces: Vec<FaceBox> = Vec::new();

        for i in 0..num_detections {
            let confidence = scores[[0, i, 0]];
            if confidence < CONFIDENCE_THRESHOLD {
                continue;
            }

            // YuNet bbox format: [center_x, center_y, width, height]
            let cx = bboxes[[0, i, 0]];
            let cy = bboxes[[0, i, 1]];
            let w = bboxes[[0, i, 2]];
            let h = bboxes[[0, i, 3]];

            // Convert to [x, y, w, h]
            let x = cx - w / 2.0;
            let y = cy - h / 2.0;

            // Extract 5 landmarks [x, y] pairs
            let mut face_landmarks = [[0.0f32; 2]; 5];
            for j in 0..5 {
                face_landmarks[j] = [
                    landmarks[[0, i, j * 2]],
                    landmarks[[0, i, j * 2 + 1]],
                ];
            }

            faces.push(FaceBox {
                bbox: [x, y, w, h],
                confidence,
                landmarks: face_landmarks,
            });
        }

        // 6. Sort by confidence descending
        faces.sort_by(|a, b| b.confidence.partial_cmp(&a.confidence).unwrap_or(std::cmp::Ordering::Equal));

        // 7. Apply NMS
        let faces = nms(faces, NMS_THRESHOLD);

        Ok(faces)
    }
}

/// Non-maximum suppression: greedily select the highest-confidence face,
/// remove all others that overlap above the threshold, repeat.
fn nms(mut faces: Vec<FaceBox>, threshold: f32) -> Vec<FaceBox> {
    let mut result = Vec::with_capacity(faces.len());
    while !faces.is_empty() {
        let best = faces.remove(0);
        faces.retain(|f| iou(&best.bbox, &f.bbox) <= threshold);
        result.push(best);
    }
    result
}

/// Intersection over Union for two bounding boxes [x, y, w, h].
fn iou(a: &[f32; 4], b: &[f32; 4]) -> f32 {
    let x1 = a[0].max(b[0]);
    let y1 = a[1].max(b[1]);
    let x2 = (a[0] + a[2]).min(b[0] + b[2]);
    let y2 = (a[1] + a[3]).min(b[1] + b[3]);

    let intersection = (x2 - x1).max(0.0) * (y2 - y1).max(0.0);
    if intersection <= 0.0 {
        return 0.0;
    }

    let area_a = a[2] * a[3];
    let area_b = b[2] * b[3];
    let union = area_a + area_b - intersection;

    if union <= 0.0 {
        0.0
    } else {
        intersection / union
    }
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
        assert!((iou(&b, &a) - 0.0).abs() < 0.001);
    }

    #[test]
    fn test_iou_partial_overlap() {
        let a = [0.0, 0.0, 1.0, 1.0];
        let b = [0.5, 0.0, 1.0, 1.0];
        let i = iou(&a, &b);
        // area_a = 1, area_b = 1
        // intersection: x1=0.5, y1=0, x2=1, y2=1 => 0.5*1 = 0.5
        // union = 1 + 1 - 0.5 = 1.5
        // iou = 0.5 / 1.5 = 0.333...
        assert!((i - 0.333).abs() < 0.01);
    }

    #[test]
    fn test_nms_removes_overlapping() {
        let faces = vec![
            FaceBox {
                bbox: [0.0, 0.0, 1.0, 1.0],
                confidence: 0.9,
                landmarks: [[0.0; 2]; 5],
            },
            FaceBox {
                bbox: [0.05, 0.05, 0.9, 0.9],
                confidence: 0.8,
                landmarks: [[0.0; 2]; 5],
            },
            FaceBox {
                bbox: [0.8, 0.8, 0.1, 0.1],
                confidence: 0.7,
                landmarks: [[0.0; 2]; 5],
            },
        ];
        let result = nms(faces, 0.45);
        // First two overlap significantly (>0.45), so the second should be removed
        // Third box is far away from the first, so it should remain
        assert_eq!(result.len(), 2);
        assert!((result[0].confidence - 0.9).abs() < 0.001);
        assert!((result[1].confidence - 0.7).abs() < 0.001);
    }
}
