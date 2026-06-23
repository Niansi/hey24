use image::RgbaImage;
use std::path::{Path, PathBuf};
use std::process::Command;
use std::mem;
use tempfile::TempDir;

use crate::error::{AppError, AppResult};

pub struct RenderParams {
    pub out_width: u32,
    pub out_height: u32,
    pub fps_in: u32,
    pub fps_out: u32,
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

pub fn render_video(
    frames: &[RgbaImage],
    output_path: &Path,
    params: &RenderParams,
) -> AppResult<PathBuf> {
    let tmp_dir = TempDir::new().map_err(|e| AppError::Io(e))?;

    // Write each frame as a PNG image
    for (i, frame) in frames.iter().enumerate() {
        let filename = format!("frame_{:04}.png", i);
        let frame_path = tmp_dir.path().join(&filename);
        frame
            .save(&frame_path)
            .map_err(|e| AppError::Render(format!("Failed to save frame {}: {}", i, e)))?;
    }

    // Build FFmpeg command
    let frame_pattern = tmp_dir.path().join("frame_%04d.png");
    let frame_pattern_str = frame_pattern.to_string_lossy().to_string();

    let mut cmd = Command::new("ffmpeg");
    cmd.arg("-y");
    cmd.arg("-framerate");
    cmd.arg(params.fps_in.to_string());
    cmd.arg("-i");
    cmd.arg(&frame_pattern_str);

    // Add BGM if provided
    if let Some(bgm_path) = &params.bgm_path {
        cmd.arg("-i");
        cmd.arg(bgm_path);
        cmd.arg("-shortest");
    }

    cmd.arg("-c:v");
    cmd.arg(&params.codec);
    cmd.arg("-b:v");
    cmd.arg(&params.bitrate);

    // Build the filter complex string
    let vf = format!(
        "fps={},format=yuv420p,scale={}:{}:force_original_aspect_ratio=decrease,pad={}:{}:(ow-iw)/2:(oh-ih)/2",
        params.fps_out, params.out_width, params.out_height, params.out_width, params.out_height
    );
    cmd.arg("-vf");
    cmd.arg(&vf);

    cmd.arg("-pix_fmt");
    cmd.arg("yuv420p");
    cmd.arg(output_path);

    // Run FFmpeg
    let output = cmd
        .output()
        .map_err(|e| AppError::Render(format!("Failed to run ffmpeg: {}. Make sure FFmpeg is installed (`brew install ffmpeg`)", e)))?;

    if !output.status.success() {
        let stderr = String::from_utf8_lossy(&output.stderr).to_string();
        return Err(AppError::Render(format!(
            "FFmpeg failed: {}",
            stderr
        )));
    }

    // Keep temp directory alive (temp workaround for MVP)
    mem::forget(tmp_dir);

    Ok(output_path.to_path_buf())
}

pub fn build_frames(aligned_photos: &[RgbaImage]) -> Vec<RgbaImage> {
    aligned_photos.to_vec()
}

pub fn check_ffmpeg() -> AppResult<String> {
    let output = Command::new("ffmpeg")
        .arg("-version")
        .output()
        .map_err(|_| {
            AppError::Render(
                "FFmpeg not found. Please install it with: brew install ffmpeg".into(),
            )
        })?;

    if !output.status.success() {
        return Err(AppError::Render(
            "FFmpeg returned a non-zero exit code on -version".into(),
        ));
    }

    let stdout = String::from_utf8_lossy(&output.stdout);
    let first_line = stdout.lines().next().unwrap_or("").to_string();

    Ok(first_line)
}

#[cfg(test)]
mod tests {
    use super::*;

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

    #[test]
    fn test_render_params_default() {
        let params = RenderParams::default();
        assert_eq!(params.out_width, 1080);
        assert_eq!(params.out_height, 1920);
        assert_eq!(params.fps_in, 5);
        assert_eq!(params.fps_out, 30);
    }
}
