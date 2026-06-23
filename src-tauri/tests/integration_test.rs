/// End-to-end integration tests for Hey24 pipeline.
/// ONNX face detection is tested via unit tests (not here) to avoid slow model loading.

use hey24_lib::engine::align::{self, AlignParams};
use hey24_lib::engine::render::RenderParams;
use hey24_lib::engine::template::TemplateConfig;

#[test]
fn test_e2e_load_template() {
    let config = TemplateConfig::load("templates/silent-grow.yaml")
        .expect("Failed to load silent-grow template");
    assert_eq!(config.id, "silent-grow");
    assert_eq!(config.name, "悄悄长大");
    assert_eq!(config.output.default.width, 1080);
    assert_eq!(config.output.default.height, 1920);
    assert_eq!(config.output.default.fps, 5);
}

#[test]
fn test_e2e_align_params_from_template() {
    let config = TemplateConfig::load("templates/silent-grow.yaml").unwrap();
    let params = config.align_params().unwrap();
    assert_eq!(params.target_eye_y, 0.38);
    assert_eq!(params.target_eye_distance, 320.0);
    assert_eq!(params.out_width, 1080);
    assert_eq!(params.out_height, 1920);
}

#[test]
fn test_e2e_fallback_align_no_face() {
    // Test that the fallback align (center-crop) works when no faces detected
    let img = image::DynamicImage::new_rgba8(800, 600);
    let params = AlignParams::default();
    let result = align::align_photo(&img, &[], &params)
        .expect("Fallback align should succeed on any image");
    assert_eq!(result.width(), 1080);
    assert_eq!(result.height(), 1920);
}

#[test]
fn test_e2e_fallback_align_wide_image() {
    // Wide image should be scaled and cropped correctly
    let img = image::DynamicImage::new_rgba8(4000, 2000);
    let params = AlignParams::default();
    let result = align::align_photo(&img, &[], &params)
        .expect("Wide image fallback should succeed");
    assert_eq!(result.width(), 1080);
    assert_eq!(result.height(), 1920);
}

#[test]
fn test_e2e_render_params_defaults() {
    let params = RenderParams::default();
    assert_eq!(params.out_width, 1080);
    assert_eq!(params.out_height, 1920);
    assert_eq!(params.fps_in, 5);
    assert_eq!(params.fps_out, 30);
    assert_eq!(params.codec, "h264_videotoolbox");
}

#[test]
fn test_e2e_compute_similarity_transform() {
    // Test that the similarity transform maps eye landmarks to target positions
    let landmarks: [[f32; 2]; 5] = [
        [0.45, 0.35],  // left eye
        [0.55, 0.35],  // right eye
        [0.50, 0.45],  // nose
        [0.47, 0.55],  // left mouth
        [0.53, 0.55],  // right mouth
    ];

    let params = AlignParams {
        target_eye_y: 0.38,
        target_eye_distance: 320.0,
        out_width: 1080,
        out_height: 1920,
        fill_mode: "blur_expand".to_string(),
    };

    let t = align::compute_similarity_transform(&landmarks, 1080.0, 1920.0, &params);

    // Verify transform maps left eye near target position
    let src_lx = 0.45 * 1080.0;
    let src_ly = 0.35 * 1920.0;
    let dst_x = t[0][0] * src_lx + t[0][1] * src_ly + t[0][2];
    let dst_y = t[1][0] * src_lx + t[1][1] * src_ly + t[1][2];

    // Should map near center X
    assert!((dst_x - 540.0).abs() < 15.0,
        "Expected dst_x near 540, got {}", dst_x);
    // Should map near 38% Y height
    let target_y = 1920.0 * 0.38;
    assert!((dst_y - target_y).abs() < 15.0,
        "Expected dst_y near {}, got {}", target_y, dst_y);
}
