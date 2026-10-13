use crate::image::Image;
use crate::image::error::ExtractError;
use crate::pdf::Document;
use ::image::ImageFormat;

/// Crops one `Image` per polygon from page `page_id` of a PDF or image.
/// # Errors
/// Returns an [`ExtractError`] if the polygon is empty or if the polygon is out of bounds.
pub fn extract_polygons(
    data: &[u8],
    page_id: u16,
    polygons: &[Vec<(f32, f32)>],
    dpi: f32,
) -> Result<Vec<Image>, ExtractError> {
    let page_img = if data.starts_with(b"%PDF-") {
        let doc = Document::from_bytes(data.to_vec())?;
        doc.page(page_id)?
            .render_image(dpi / 72.0)?
            .with_format(ImageFormat::Jpeg)
    } else if page_id == 0 {
        Image::decode(data)?
    } else {
        return Err(ExtractError::PageOutOfRange(page_id));
    };

    polygons
        .iter()
        .enumerate()
        .map(|(i, poly)| crop_polygon(&page_img, poly, i))
        .collect()
}

/// Crops a polygon from an image.
/// # Errors
/// Returns an [`ExtractError`] if the polygon is empty or if the polygon is out of bounds.
#[allow(
    clippy::cast_precision_loss,
    clippy::cast_possible_truncation,
    clippy::cast_sign_loss
)]
fn crop_polygon(page: &Image, poly: &[(f32, f32)], idx: usize) -> Result<Image, ExtractError> {
    if poly.is_empty() {
        return Err(ExtractError::EmptyPolygon(idx));
    }
    let (w, h) = page.size();
    let (min_x, max_x) = poly.iter().fold((f32::MAX, f32::MIN), |(lo, hi), p| {
        (lo.min(p.0), hi.max(p.0))
    });
    let (min_y, max_y) = poly.iter().fold((f32::MAX, f32::MIN), |(lo, hi), p| {
        (lo.min(p.1), hi.max(p.1))
    });

    let left = (min_x * w as f32).floor().clamp(0.0, w as f32) as u32;
    let top = (min_y * h as f32).floor().clamp(0.0, h as f32) as u32;
    let right = (max_x * w as f32).ceil().clamp(0.0, w as f32) as u32;
    let bottom = (max_y * h as f32).ceil().clamp(0.0, h as f32) as u32;
    if right <= left || bottom <= top {
        return Err(ExtractError::EmptyPolygon(idx));
    }
    Ok(page.crop(left, top, right, bottom)?)
}
