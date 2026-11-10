//! Image module.

/// Image error module.
pub mod error;
/// Extraction module.
pub mod extract;
/// Image module.
pub mod image_data;

pub use crate::common::parse_output_format;
pub use ::image::imageops::FilterType;
pub use error::ImageError;
pub use image_data::{Image, compress_image, format_name, guess_format, parse_format};
