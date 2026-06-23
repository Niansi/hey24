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

fn default_confidence() -> f32 {
    0.5
}

impl TemplateConfig {
    /// Load a template from a YAML file path.
    pub fn load(path: &str) -> AppResult<Self> {
        let content = std::fs::read_to_string(path)
            .map_err(|e| AppError::Template(format!("Cannot read template file '{}': {}", path, e)))?;
        let config: Self = serde_yaml::from_str(&content)
            .map_err(|e| AppError::Template(format!("Invalid template YAML: {}", e)))?;
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
        Err(AppError::Template(
            "No face_align stage found in pipeline".into(),
        ))
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
