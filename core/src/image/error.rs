/// Errors that can occur when decoding or encoding images.
#[derive(Debug, thiserror::Error)]
pub enum ImageError {
    /// Cannot decode image.
    #[error("failed to decode image: {0}")]
    Decode(String),

    /// Cannot encode image.
    #[error("failed to encode image: {0}")]
    Encode(String),

    /// Format is not supported.
    #[error("unsupported image format: {0}")]
    UnsupportedFormat(String),

    /// Format could not be determined.
    #[error("unknown image format")]
    UnknownFormat,

    /// Image is less than 4 bytes, making it too small to be a valid image.
    #[error("not enough data to determine image format")]
    NotEnoughData,

    /// Filter is not supported.
    #[error("unknown resize filter: {0}")]
    UnknownFilter(String),

    /// Image is too small to be resized.
    #[error(
        "invalid crop bounds: left={left}, top={top}, right={right}, bottom={bottom} for image {width}x{height}"
    )]
    InvalidCrop {
        /// Left pixel coordinate.
        left: u32,
        /// Top pixel coordinate.
        top: u32,
        /// Right pixel coordinate.
        right: u32,
        /// Bottom pixel coordinate.
        bottom: u32,
        /// Image width.
        width: u32,
        /// Image height.
        height: u32,
    },

    /// Error from the underlying I/O library.
    #[error("I/O error: {0}")]
    Io(#[from] std::io::Error),

    /// Invalid image dimensions.
    #[error("invalid image dimensions: {width}x{height}")]
    InvalidDimensions {
        /// Image width.
        width: u32,
        /// Image height.
        height: u32,
    },
}
