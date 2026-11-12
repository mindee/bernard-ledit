//! Image module.

mod crop;
/// Image error module.
pub mod error;
/// Image module.
pub mod image_data;

pub use crate::common::parse_output_format;
pub use ::image::imageops::FilterType;
pub use error::ImageError;
pub use image_data::{Image, format_name, guess_format, parse_format};
