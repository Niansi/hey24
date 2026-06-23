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
