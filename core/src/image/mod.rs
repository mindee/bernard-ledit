//! Image module.

/// Image error module.
pub mod error;
/// Image module.
pub mod image_data;

pub use error::ImageError;
pub use image_data::{Filter, Image, compress, format_name, guess_format, parse_format};
