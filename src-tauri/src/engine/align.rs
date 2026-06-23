use image::{DynamicImage, Rgba, RgbaImage};

use crate::error::AppError;
use crate::error::AppResult;
use crate::engine::face::FaceBox;

// ---------------------------------------------------------------------------
// Parameters
// ---------------------------------------------------------------------------

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

// ---------------------------------------------------------------------------
// Similarity transform (2 × 3)
// ---------------------------------------------------------------------------

/// Compute a 2×3 similarity transform (rotation + uniform scale +
/// translation) that aligns the two eyes.
///
/// `landmarks` are in normalized [0, 1] coordinate space.
/// The returned matrix maps **source → destination** in pixel coordinates:
///
/// ```text
/// ⎡ a  b  tx ⎤
/// ⎣ d  e  ty ⎦
/// ```
pub fn compute_similarity_transform(
    landmarks: &[[f32; 2]; 5],
    img_w: f32,
    img_h: f32,
    params: &AlignParams,
) -> [[f32; 3]; 2] {
    // Convert landmarks from normalized [0, 1] to pixel coordinates.
    let le = [
        landmarks[0][0] * img_w,
        landmarks[0][1] * img_h,
    ];
    let re = [
        landmarks[1][0] * img_w,
        landmarks[1][1] * img_h,
    ];

    // Source eye vector and distance.
    let dx = re[0] - le[0];
    let dy = re[1] - le[1];
    let source_dist = (dx * dx + dy * dy).sqrt();

    // Rotation angle: negate so the transform de-rotates the eye line
    // back to horizontal.
    let angle = -dy.atan2(dx);
    let (sin_a, cos_a) = angle.sin_cos();

    // Uniform scale.
    let scale = params.target_eye_distance / source_dist;

    // Target position for the left eye: centered horizontally at
    // target_eye_y fraction of the output height.
    let target_cx = params.out_width as f32 * 0.5;
    let target_cy = params.target_eye_y * params.out_height as f32;

    // Translation that sends the source left eye to the target position.
    let tx = target_cx - scale * cos_a * le[0] + scale * sin_a * le[1];
    let ty = target_cy - scale * sin_a * le[0] - scale * cos_a * le[1];

    [
        [scale * cos_a, -scale * sin_a, tx],
        [scale * sin_a, scale * cos_a, ty],
    ]
}

// ---------------------------------------------------------------------------
// Bilinear interpolation helper
// ---------------------------------------------------------------------------

fn bilinear(c00: u8, c10: u8, c01: u8, c11: u8, fx: f32, fy: f32) -> u8 {
    let top = c00 as f32 + (c10 as f32 - c00 as f32) * fx;
    let bottom = c01 as f32 + (c11 as f32 - c01 as f32) * fx;
    (top + (bottom - top) * fy).round() as u8
}

// ---------------------------------------------------------------------------
// Blur fill for transparent edge pixels
// ---------------------------------------------------------------------------

/// 3-pass blur fill: for each transparent pixel, fill with the average of
/// non-transparent neighbours within a 2 px radius.
fn fill_transparent_with_blur(img: &mut RgbaImage) {
    let (w, h) = img.dimensions();

    for _pass in 0..3 {
        // Collect fills to apply after the scan so we don't pollute the
        // current pass's neighbourhood.
        let mut fills: Vec<(u32, u32, [u8; 3])> = Vec::new();

        for y in 0..h {
            for x in 0..w {
                let px = img.get_pixel(x, y);
                if px[3] != 0 {
                    continue;
                }

                let mut r_sum: u32 = 0;
                let mut g_sum: u32 = 0;
                let mut b_sum: u32 = 0;
                let mut count: u32 = 0;

                // 5 × 5 neighbourhood (2 px radius), clamped to image bounds.
                let x0 = if x >= 2 { x - 2 } else { 0 };
                let x1 = (x + 2).min(w - 1);
                let y0 = if y >= 2 { y - 2 } else { 0 };
                let y1 = (y + 2).min(h - 1);

                for ny in y0..=y1 {
                    for nx in x0..=x1 {
                        if nx == x && ny == y {
                            continue;
                        }
                        let npx = img.get_pixel(nx, ny);
                        if npx[3] > 0 {
                            r_sum += npx[0] as u32;
                            g_sum += npx[1] as u32;
                            b_sum += npx[2] as u32;
                            count += 1;
                        }
                    }
                }

                if count > 0 {
                    fills.push((
                        x,
                        y,
                        [
                            (r_sum / count) as u8,
                            (g_sum / count) as u8,
                            (b_sum / count) as u8,
                        ],
                    ));
                }
            }
        }

        if fills.is_empty() {
            break; // No more transparent pixels — done.
        }

        for (x, y, rgb) in fills {
            img.put_pixel(x, y, Rgba([rgb[0], rgb[1], rgb[2], 255]));
        }
    }
}

// ---------------------------------------------------------------------------
// Apply the affine transform
// ---------------------------------------------------------------------------

/// Apply the 2×3 similarity transform to `img` via inverse mapping and
/// bilinear interpolation.
pub fn apply_transform(
    img: &DynamicImage,
    transform: &[[f32; 3]; 2],
    params: &AlignParams,
) -> AppResult<RgbaImage> {
    let out_w = params.out_width;
    let out_h = params.out_height;

    let src = img.to_rgba8();
    let (src_w, src_h) = src.dimensions();

    if src_w < 2 || src_h < 2 {
        return Err(AppError::FaceAlignment(
            "Source image is too small to sample from".into(),
        ));
    }

    // Forward matrix coefficients.
    let a = transform[0][0];
    let b = transform[0][1];
    let c = transform[0][2];
    let d = transform[1][0];
    let e = transform[1][1];
    let f = transform[1][2];

    // Determinant of the linear part (should be scale², always > 0).
    let det = a * e - b * d;
    if det.abs() < 1e-8 {
        return Err(AppError::FaceAlignment(
            "Similarity transform has a zero determinant".into(),
        ));
    }

    // Inverse of the linear part: (1/det) * [[e, -b], [-d, a]].
    let inv_a = e / det;
    let inv_b = -b / det;
    let inv_d = -d / det;
    let inv_e = a / det;

    let mut out = RgbaImage::new(out_w, out_h);

    for oy in 0..out_h {
        for ox in 0..out_w {
            // Inverse map: source pixel from destination pixel.
            let sx = inv_a * (ox as f32 - c) + inv_b * (oy as f32 - f);
            let sy = inv_d * (ox as f32 - c) + inv_e * (oy as f32 - f);

            // Bilinear interpolation (4 nearest neighbours).
            if sx >= 0.0 && sx < (src_w - 1) as f32 && sy >= 0.0 && sy < (src_h - 1) as f32 {
                let x0 = sx as u32;
                let y0 = sy as u32;
                let fx = sx - x0 as f32;
                let fy = sy - y0 as f32;

                let c00 = src.get_pixel(x0, y0);
                let c10 = src.get_pixel(x0 + 1, y0);
                let c01 = src.get_pixel(x0, y0 + 1);
                let c11 = src.get_pixel(x0 + 1, y0 + 1);

                out.put_pixel(
                    ox,
                    oy,
                    Rgba([
                        bilinear(c00[0], c10[0], c01[0], c11[0], fx, fy),
                        bilinear(c00[1], c10[1], c01[1], c11[1], fx, fy),
                        bilinear(c00[2], c10[2], c01[2], c11[2], fx, fy),
                        bilinear(c00[3], c10[3], c01[3], c11[3], fx, fy),
                    ]),
                );
            }
            // Otherwise pixel stays transparent (alpha = 0) — correct by
            // default for a freshly allocated RgbaImage.
        }
    }

    // Edge fill.
    if params.fill_mode == "blur_expand" {
        fill_transparent_with_blur(&mut out);
    }

    Ok(out)
}

// ---------------------------------------------------------------------------
// Fallback: centre-crop + cover scale
// ---------------------------------------------------------------------------

/// Scale the image to cover the output canvas (preserving aspect ratio) and
/// centre-crop.
fn fallback_align(img: &DynamicImage, params: &AlignParams) -> AppResult<RgbaImage> {
    let (img_w, img_h) = (img.width(), img.height());

    if img_w == 0 || img_h == 0 {
        return Err(AppError::FaceAlignment("Source image is empty".into()));
    }

    let scale_x = params.out_width as f64 / img_w as f64;
    let scale_y = params.out_height as f64 / img_h as f64;
    let scale = scale_x.max(scale_y); // cover

    let new_w = (img_w as f64 * scale).round() as u32;
    let new_h = (img_h as f64 * scale).round() as u32;

    let scaled = img.resize_exact(new_w, new_h, image::imageops::Lanczos3);

    // Centre crop.
    let offset_x = ((new_w as i64 - params.out_width as i64) / 2).max(0) as u32;
    let offset_y = ((new_h as i64 - params.out_height as i64) / 2).max(0) as u32;

    let cropped = scaled.crop_imm(
        offset_x,
        offset_y,
        params.out_width.min(new_w - offset_x),
        params.out_height.min(new_h - offset_y),
    );

    // If the cropped region is smaller than the output, paste onto a black
    // canvas.
    let mut out = RgbaImage::new(params.out_width, params.out_height);
    image::imageops::overlay(&mut out, &cropped.to_rgba8(), 0, 0);

    Ok(out)
}

// ---------------------------------------------------------------------------
// Public entry point
// ---------------------------------------------------------------------------

/// Align a face photo.
///
/// * If at least one face is detected – use the first face's landmarks to
///   compute a similarity transform and warp the image.
/// * Otherwise – fall back to a cover-scale centre-crop.
pub fn align_photo(
    img: &DynamicImage,
    faces: &[FaceBox],
    params: &AlignParams,
) -> AppResult<RgbaImage> {
    if let Some(face) = faces.first() {
        let transform = compute_similarity_transform(
            &face.landmarks,
            img.width() as f32,
            img.height() as f32,
            params,
        );
        apply_transform(img, &transform, params)
    } else {
        fallback_align(img, params)
    }
}

// ---------------------------------------------------------------------------
// Tests
// ---------------------------------------------------------------------------

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_similarity_transform_identity() {
        let landmarks = [
            [0.45, 0.35], // left eye
            [0.55, 0.35], // right eye (horizontal, 0.1 apart)
            [0.50, 0.45], // nose
            [0.47, 0.55], // left mouth
            [0.53, 0.55], // right mouth
        ];

        let params = AlignParams {
            target_eye_distance: 108.0, // 0.1 * 1080
            out_width: 1080,
            out_height: 1920,
            ..Default::default()
        };

        let t = compute_similarity_transform(&landmarks, 1080.0, 1920.0, &params);

        // The transform should map left_eye to near target position
        let src_lx = 0.45 * 1080.0;
        let src_ly = 0.35 * 1920.0;
        let dst_x = t[0][0] * src_lx + t[0][1] * src_ly + t[0][2];
        let dst_y = t[1][0] * src_lx + t[1][1] * src_ly + t[1][2];

        // Eye should be near center X and near 38% Y
        assert!(
            (dst_x - 540.0).abs() < 15.0,
            "dst_x={} expected near 540",
            dst_x
        );
        assert!(
            (dst_y - 1920.0 * 0.38).abs() < 15.0,
            "dst_y={} expected near {}",
            dst_y,
            1920.0 * 0.38
        );
    }

    #[test]
    fn test_similarity_transform_rotated_eyes() {
        // Eyes on a diagonal: left eye lower, right eye higher.
        let landmarks = [
            [0.40, 0.40], // left eye
            [0.60, 0.30], // right eye
            [0.50, 0.45], // nose
            [0.47, 0.55], // left mouth
            [0.53, 0.55], // right mouth
        ];

        let params = AlignParams {
            target_eye_distance: 200.0,
            out_width: 800,
            out_height: 1200,
            ..Default::default()
        };

        let t = compute_similarity_transform(&landmarks, 800.0, 1200.0, &params);

        // After transform the left eye should be at the target position.
        let src_lx = 0.40 * 800.0;
        let src_ly = 0.40 * 1200.0;
        let dst_x = t[0][0] * src_lx + t[0][1] * src_ly + t[0][2];
        let dst_y = t[1][0] * src_lx + t[1][1] * src_ly + t[1][2];

        let target_cx = 800.0 * 0.5;
        let target_cy = 1200.0 * 0.38;

        assert!(
            (dst_x - target_cx).abs() < 5.0,
            "dst_x={} expected near {}",
            dst_x,
            target_cx
        );
        assert!(
            (dst_y - target_cy).abs() < 5.0,
            "dst_y={} expected near {}",
            dst_y,
            target_cy
        );

        // Eye distance should be preserved (scale is fixed).
        let src_rx = 0.60 * 800.0;
        let src_ry = 0.30 * 1200.0;
        let r_dst_x = t[0][0] * src_rx + t[0][1] * src_ry + t[0][2];
        let r_dst_y = t[1][0] * src_rx + t[1][1] * src_ry + t[1][2];

        let eye_dist =
            ((r_dst_x - dst_x).powi(2) + (r_dst_y - dst_y).powi(2)).sqrt();
        assert!(
            (eye_dist - params.target_eye_distance).abs() < 1.0,
            "eye_dist={} expected near {}",
            eye_dist,
            params.target_eye_distance
        );

        // The two eyes should be roughly horizontal.
        assert!(
            (r_dst_y - dst_y).abs() < 5.0,
            "eye y diff={} expected near 0",
            r_dst_y - dst_y
        );
    }

    #[test]
    fn test_bilinear_identity() {
        // All four pixels the same => output should match.
        let v = bilinear(128, 128, 128, 128, 0.5, 0.5);
        assert_eq!(v, 128);
    }

    #[test]
    fn test_bilinear_interpolation() {
        // Horizontal interpolation: 0 at left, 255 at right.
        // At fx = 0.5, fy = 0.0 (top row), result should be 128.
        let v = bilinear(0, 255, 0, 255, 0.5, 0.0);
        assert!((v as i32 - 128).abs() <= 1);
    }

    #[test]
    fn test_apply_transform_identity() {
        // Create a simple 200×200 image.
        let img = DynamicImage::new_rgb8(200, 200);

        let params = AlignParams {
            out_width: 200,
            out_height: 200,
            ..Default::default()
        };

        let transform = [[1.0, 0.0, 0.0], [0.0, 1.0, 0.0]];

        let result = apply_transform(&img, &transform, &params).unwrap();
        assert_eq!(result.dimensions(), (200, 200));
    }

    #[test]
    fn test_apply_transform_translation() {
        let mut img = RgbaImage::new(100, 100);
        // Draw a white pixel at (10, 10).
        img.put_pixel(10, 10, Rgba([255, 255, 255, 255]));
        let img = DynamicImage::ImageRgba8(img);

        let params = AlignParams {
            out_width: 100,
            out_height: 100,
            fill_mode: "crop".into(),
            ..Default::default()
        };

        // Shift right by 5, down by 10.
        let transform = [[1.0, 0.0, 5.0], [0.0, 1.0, 10.0]];
        let result = apply_transform(&img, &transform, &params).unwrap();

        let px = result.get_pixel(15, 20);
        assert_eq!(px[0], 255);
        assert_eq!(px[1], 255);
        assert_eq!(px[2], 255);
    }

    #[test]
    fn test_fallback_align_same_ratio() {
        let img = DynamicImage::new_rgb8(100, 200);
        let params = AlignParams {
            out_width: 200,
            out_height: 400,
            ..Default::default()
        };
        let result = fallback_align(&img, &params).unwrap();
        assert_eq!(result.dimensions(), (200, 400));
    }

    #[test]
    fn test_fallback_align_different_ratio() {
        // Landscape input → portrait output (cover + crop).
        let img = DynamicImage::new_rgb8(400, 200);
        let params = AlignParams {
            out_width: 200,
            out_height: 400,
            ..Default::default()
        };
        let result = fallback_align(&img, &params).unwrap();
        assert_eq!(result.dimensions(), (200, 400));
    }

    #[test]
    fn test_align_photo_no_faces() {
        let img = DynamicImage::new_rgb8(50, 80);
        let params = AlignParams {
            out_width: 100,
            out_height: 160,
            ..Default::default()
        };
        let result = align_photo(&img, &[], &params).unwrap();
        assert_eq!(result.dimensions(), (100, 160));
    }
}
