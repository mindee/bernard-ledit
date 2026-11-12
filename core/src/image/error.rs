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

    /// Successive JPEG quality values applied to compression operation are too low.
    #[error(
        "successive JPEG quality values applied to compression operation are too low: {jpeg_quality}"
    )]
    ProgressiveQualityTooLow {
        /// JPEG quality applied to compression.
        jpeg_quality: u8,
    },

    /// Compression somehow returned a bigger object than the input.
    #[error(
        "compression returned a bigger object than the input: {compressed_size} > {initial_size}"
    )]
    SizeIncreased {
        /// Initial byte sequence size.
        initial_size: usize,
        /// Length of bytes returned by compression.
        compressed_size: usize,
    },
}

/// Errors related to image extraction.
#[derive(Debug, thiserror::Error)]
pub enum ImageExtractError {
    /// Errors related to image extraction in images.
    #[error(transparent)]
    Image(#[from] ImageError),
    /// Polygon is empty or falls outside the image.
    #[error("polygon is empty or falls outside the image")]
    EmptyPolygon(),
}
