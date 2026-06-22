use image::{ImageBuffer, ImageFormat, Rgba};
use pdfium_render::prelude::PdfBitmap;
use std::io::Cursor;

/// A decoded RGBA8 bitmap produced by rendering a PDF page.
pub struct Bitmap {
    /// Width of the bitmap, in pixels.
    pub width: u32,
    /// Height of the bitmap, in pixels.
    pub height: u32,
    /// Raw RGBA8 pixels, row-major, 4 bytes per pixel.
    pub rgba: Vec<u8>,
}

impl Bitmap {
    /// Construct from a pdfium bitmap.
    /// # Panics
    /// Panics if the `PDFium` bitmap has a negative width or height.
    #[must_use = "this returns a newly constructed `Bitmap` without side effects; dropping it wastes allocation"]
    pub fn from_pdfium(pb: &PdfBitmap) -> Self {
        let width = pb
            .width()
            .try_into()
            .expect("PDFium returned a negative width");
        let height = pb
            .height()
            .try_into()
            .expect("PDFium returned a negative height");
        let rgba = pb.as_rgba_bytes();
        Self {
            width,
            height,
            rgba,
        }
    }

    /// Encode the bitmap as a PNG byte stream.
    ///
    /// # Errors
    ///
    /// Returns an [`image::ImageError`] if encoding fails.
    ///
    /// # Panics
    ///
    /// Panics if the `rgba` buffer size is less than `width * height * 4` (i.e., if the buffer
    /// is too small to contain the expected pixel data).
    pub fn to_png_bytes(&self) -> Result<Vec<u8>, image::ImageError> {
        let img: ImageBuffer<Rgba<u8>, &[u8]> =
            ImageBuffer::from_raw(self.width, self.height, self.rgba.as_slice())
                .expect("RGBA buffer size is inconsistent with width × height");
        let mut cursor = Cursor::new(Vec::new());
        img.write_to(&mut cursor, ImageFormat::Png)?;
        Ok(cursor.into_inner())
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::pdf::pdfium;
    use pdfium_render::prelude::PdfBitmapFormat;
    use test_case::test_case;

    #[test_case(1, 1 ; "1x1 square")]
    #[test_case(10, 10 ; "small 10x10 square")]
    #[test_case(1920, 1080 ; "full hd resolution")]
    #[test_case(3840, 2160 ; "4k resolution")]
    fn test_valid_nxm_bitmap(width: i32, height: i32) {
        pdfium();

        let total_pixels: usize = (width * height).try_into().unwrap();
        let buffer_size = total_pixels * 4;

        let mut buffer = [0, 0, 255, 255].repeat(total_pixels);

        let bitmap = PdfBitmap::from_bytes(width, height, PdfBitmapFormat::BGRA, &mut buffer)
            .expect("Failed to create bitmap from bytes");
        let local_bitmap = Bitmap::from_pdfium(&bitmap);
        assert_eq!(local_bitmap.width, width.try_into().unwrap());
        assert_eq!(local_bitmap.height, height.try_into().unwrap());
        assert_eq!(local_bitmap.rgba.len(), buffer_size);
        assert!(
            local_bitmap
                .rgba
                .chunks_exact(4)
                .all(|pixel| pixel == [255, 0, 0, 255]),
            "Bitmap data contained unexpected bytes (expected RGBA red: [255, 0, 0, 255])"
        );
    }

    /// Input: a single BGRA pixel (0, 128, 255, 200).
    /// pdfium-render's `as_rgba_bytes()` should swap B and R → (255, 128, 0, 200).
    #[test]
    fn test_from_pdfium_bgra_to_rgba_channel_order() {
        pdfium();
        let mut buffer: [u8; 4] = [0, 128, 255, 200];
        let bitmap = PdfBitmap::from_bytes(1, 1, PdfBitmapFormat::BGRA, &mut buffer)
            .expect("Failed to create bitmap from bytes");
        let local_bitmap = Bitmap::from_pdfium(&bitmap);
        assert_eq!(local_bitmap.rgba, vec![255, 128, 0, 200]);
    }

    #[test]
    #[should_panic(expected = "RGBA buffer size is inconsistent with width × height")]
    fn test_to_png_bytes_panics_on_small_buffer() {
        let bmp = Bitmap {
            width: 2,
            height: 2,
            rgba: vec![0u8; 4],
        };
        let _ = bmp.to_png_bytes();
    }

    #[test]
    fn test_to_png_bytes_works() {
        let bmp = Bitmap {
            width: 10,
            height: 10,
            rgba: vec![255u8; 400],
        };
        let png = bmp.to_png_bytes().expect("PNG encoding failed");
        assert!(
            png.starts_with(b"\x89PNG\r\n\x1a\n"),
            "output is not a valid PNG"
        );
    }
}
