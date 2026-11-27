//! Image module.

mod crop;
/// Image error module.
pub mod error;
mod heif;
/// Image module.
pub mod image_data;

pub use crate::common::parse_output_format;
pub use crate::common::{Format, format_name, guess_format};
pub use ::image::imageops::FilterType;
pub use error::ImageError;
pub use image_data::{Image, parse_format};
