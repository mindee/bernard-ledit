use crate::image::Image;
use crate::image::ImageError;
use crate::image::error::ImageExtractError;

impl Image {
    /// PIL-style crop by pixel box.
    /// # Errors
    /// Returns `ImageError::InvalidCrop` if the crop is invalid.
    pub fn crop(&self, left: u32, top: u32, right: u32, bottom: u32) -> Result<Self, ImageError> {
        let (img_width, img_height) = self.size();
        if left >= right || top >= bottom || right > img_width || bottom > img_height {
            return Err(ImageError::InvalidCrop {
                left,
                top,
                right,
                bottom,
                width: img_width,
                height: img_height,
            });
        }
        let crop_width = right - left;
        let crop_height = bottom - top;
        let cropped_inner = self.inner.crop_imm(left, top, crop_width, crop_height);
        Ok(Self {
            inner: cropped_inner,
            format: self.format,
            source_len: self.source_len,
        })
    }

    /// Crops the bounding box of a polygon from this image.
    ///
    /// Polygon coordinates are normalized relative to the image dimensions.
    /// # Errors
    /// Returns an [`ImageExtractError`] if the polygon is empty or its bounding box is empty.
    #[allow(
        clippy::cast_precision_loss,
        clippy::cast_possible_truncation,
        clippy::cast_sign_loss
    )]
    pub fn crop_polygon(&self, poly: &[(f32, f32)]) -> Result<Self, ImageExtractError> {
        if poly.is_empty() {
            return Err(ImageExtractError::EmptyPolygon());
        }
        let (w, h) = self.size();
        let (min_x, max_x) = poly.iter().fold((f32::MAX, f32::MIN), |(lo, hi), p| {
            (lo.min(p.0), hi.max(p.0))
        });
        let (min_y, max_y) = poly.iter().fold((f32::MAX, f32::MIN), |(lo, hi), p| {
            (lo.min(p.1), hi.max(p.1))
        });

        let left = (min_x * w as f32).round().clamp(0.0, w as f32) as u32;
        let top = (min_y * h as f32).round().clamp(0.0, h as f32) as u32;
        let right = (max_x * w as f32).round().clamp(0.0, w as f32) as u32;
        let bottom = (max_y * h as f32).round().clamp(0.0, h as f32) as u32;
        if right <= left || bottom <= top {
            return Err(ImageExtractError::EmptyPolygon());
        }
        Ok(self.crop(left, top, right, bottom)?)
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use ::image::{DynamicImage, ImageFormat, RgbImage};

    fn test_image(width: u32, height: u32) -> Image {
        Image::from_dynamic(DynamicImage::ImageRgb8(RgbImage::new(width, height)))
    }

    #[test]
    fn crops_polygon_bounding_box() {
        let image = test_image(20, 10);
        let crop = image
            .crop_polygon(&[(0.0, 0.0), (0.5, 0.0), (0.5, 1.0), (0.0, 1.0)])
            .unwrap();

        assert_eq!(crop.size(), (10, 10));
    }

    #[test]
    fn rejects_empty_polygon() {
        let image = test_image(20, 10);
        assert!(matches!(
            image.crop_polygon(&[]),
            Err(ImageExtractError::EmptyPolygon())
        ));
    }

    #[test]
    fn crop_valid_box() {
        let image = test_image(100, 80);
        let cropped = image.crop(10, 20, 60, 70).unwrap();
        assert_eq!(cropped.size(), (50, 50));
    }

    #[test]
    fn crop_preserves_format() {
        let image = Image::decode(test_data_bytes!("file_types/receipt.jpg")).unwrap();
        let cropped = image.crop(0, 0, 10, 10).unwrap();
        assert_eq!(cropped.format(), Some(ImageFormat::Jpeg));
    }

    #[test]
    fn crop_out_of_bounds_errors() {
        let image = test_image(30, 30);
        assert!(matches!(
            image.crop(0, 0, 31, 10),
            Err(ImageError::InvalidCrop { .. })
        ));
    }

    #[test]
    fn crop_inverted_box_errors() {
        let image = test_image(30, 30);
        assert!(matches!(
            image.crop(20, 5, 10, 25),
            Err(ImageError::InvalidCrop { .. })
        ));
    }

    #[test]
    fn crop_zero_size_box_errors() {
        let image = test_image(30, 30);
        assert!(matches!(
            image.crop(5, 5, 5, 20),
            Err(ImageError::InvalidCrop { .. })
        ));
    }
}
