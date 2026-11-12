use crate::image::Image;
use crate::image::error::ImageExtractError;

/// Crops one `Image` per polygon from an image.
/// # Errors
/// Returns an [`ImageExtractError`] if the polygon is empty or if the polygon is out of bounds.
pub fn extract_from_polygons(
    image: &Image,
    polygons: &[Vec<(f32, f32)>],
) -> Result<Vec<Image>, ImageExtractError> {
    polygons
        .iter()
        .map(|poly| image.crop_polygon(poly))
        .collect()
}

impl Image {
    /// Crops a polygon from this image.
    /// # Errors
    /// Returns an [`ImageExtractError`] if the polygon is empty or if the polygon is out of bounds.
    #[allow(
        clippy::cast_precision_loss,
        clippy::cast_possible_truncation,
        clippy::cast_sign_loss
    )]
    fn crop_polygon(&self, poly: &[(f32, f32)]) -> Result<Self, ImageExtractError> {
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
    use ::image::{DynamicImage, RgbImage};

    fn test_image() -> Image {
        Image::from_dynamic(DynamicImage::ImageRgb8(RgbImage::new(20, 10)))
    }

    #[test]
    fn extracts_one_crop_per_polygon() {
        let image = test_image();
        let crops = extract_from_polygons(
            &image,
            &[vec![(0.0, 0.0), (0.5, 0.0), (0.5, 1.0), (0.0, 1.0)]],
        )
        .unwrap();

        assert_eq!(crops.len(), 1);
        assert_eq!(crops[0].size(), (10, 10));
    }

    #[test]
    fn rejects_empty_polygon() {
        let image = test_image();
        assert!(matches!(
            extract_from_polygons(&image, &[vec![]]),
            Err(ImageExtractError::EmptyPolygon())
        ));
    }
}
