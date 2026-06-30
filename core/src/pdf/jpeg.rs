//! JPEG-to-PDF embedding helpers.
//!
//! # DISCLAIMER
//! This module was mostly LLM-generated, as we do not have sufficient JPEG encoding knowledge to
//! do so ourselves.
//!
//! # Overview
//! Embeds a JPEG byte stream into a single-page PDF **without decoding pixels**
//! by using the PDF `/Filter /DCTDecode` mechanism.
//!
//! # Limitations
//!
//! Exif metadata (`APP1` segment) is intentionally **not** parsed. Images that
//! rely on Exif `Orientation` for display rotation (e.g. smartphone portrait
//! photos) will be embedded in their stored pixel orientation, which may appear
//! sideways or upside-down. Callers that need orientation correction must
//! pre-rotate the pixel data before calling these helpers.

use crate::pdf::error::{ImageError, PdfBuilderError};

/// Parse the first JPEG SOF (Start Of Frame) marker to extract image dimensions
/// and component count. The pixel data is never decoded.
pub(super) fn jpeg_info(data: &[u8]) -> Result<(u32, u32, u8), PdfBuilderError> {
    let err = || PdfBuilderError::Image(ImageError::InvalidJpeg);

    if data.len() < 4 || data[0] != 0xFF || data[1] != 0xD8 {
        return Err(err());
    }

    let mut pos = 2usize;
    while pos + 1 < data.len() {
        if data[pos] != 0xFF {
            return Err(err());
        }
        let marker = data[pos + 1];
        pos += 2;

        // SOF0-SOF3, SOF5-SOF7, SOF9-SOF11, SOF13-SOF15 carry frame dimensions.
        // (Excludes DHT=C4, JPG=C8, DAC=CC which share the 0xCx range.)
        if matches!(marker, 0xC0..=0xC3 | 0xC5..=0xC7 | 0xC9..=0xCB | 0xCD..=0xCF) {
            // SOF payload layout: length(2), precision(1), height(2), width(2), components(1)
            if pos + 8 > data.len() {
                return Err(err());
            }
            let height = u32::from(u16::from_be_bytes([data[pos + 3], data[pos + 4]]));
            let width = u32::from(u16::from_be_bytes([data[pos + 5], data[pos + 6]]));
            let components = data[pos + 7];
            return Ok((width, height, components));
        }

        // RST0-RST7 (D0-D7), SOI (D8), EOI (D9) have no length field.
        if matches!(marker, 0xD0..=0xD9) {
            continue;
        }

        // SOS (Start of Scan, 0xDA) is followed by entropy-coded image data
        // that cannot be skipped using the segment length field. A valid JPEG
        // always places SOF before SOS, so encountering SOS first means the
        // stream is malformed for our purposes.
        if marker == 0xDA {
            return Err(err());
        }

        // All other markers carry a 2-byte segment length (length includes itself).
        if pos + 2 > data.len() {
            return Err(err());
        }
        let seg_len = u16::from_be_bytes([data[pos], data[pos + 1]]) as usize;
        if seg_len < 2 {
            return Err(err());
        }
        pos += seg_len;
    }

    Err(err())
}

/// Build a minimal valid PDF that embeds `jpeg_bytes` directly using
/// `/Filter /DCTDecode`. No pixel decode or re-encode takes place.
/// The result is approximately `jpeg_bytes.len() + 500 bytes`.
pub(crate) fn build_jpeg_pdf(
    jpeg_bytes: &[u8],
    page_width: f32,
    page_height: f32,
) -> Result<Vec<u8>, PdfBuilderError> {
    let (img_w, img_h, components) = jpeg_info(jpeg_bytes)?;

    let (color_space, decode_array) = match components {
        1 => ("/DeviceGray", ""),
        3 => ("/DeviceRGB", ""),
        // CMYK JPEGs (commonly produced by Photoshop / Adobe tools) store
        // channels inverted relative to the PDF convention (0 = no ink vs.
        // 0 = full ink). Without a /Decode array, viewers render these with
        // neon, inverted colours. The array below flips each channel back.
        4 => (
            "/DeviceCMYK",
            "\n   /Decode [1.0 0.0 1.0 0.0 1.0 0.0 1.0 0.0]",
        ),
        n => return Err(PdfBuilderError::Image(ImageError::UnsupportedColorSpace(n))),
    };

    // Content stream: push a CTM that maps the unit square to the full page,
    // then paint the image XObject.
    let content = format!("q {page_width} 0 0 {page_height} 0 0 cm /Im0 Do Q");

    let mut pdf: Vec<u8> = Vec::with_capacity(jpeg_bytes.len() + 1024);

    macro_rules! w {
        ($($arg:tt)*) => { pdf.extend_from_slice(format!($($arg)*).as_bytes()) };
    }

    // High-byte comment signals binary content to transfer tools.
    pdf.extend_from_slice(b"%PDF-1.4\n%\xe2\xe3\xcf\xd3\n");

    let off1 = pdf.len();
    w!("1 0 obj\n<< /Type /Catalog /Pages 2 0 R >>\nendobj\n");

    let off2 = pdf.len();
    w!("2 0 obj\n<< /Type /Pages /Kids [3 0 R] /Count 1 >>\nendobj\n");

    let off3 = pdf.len();
    w!("3 0 obj\n\
         << /Type /Page /Parent 2 0 R\n\
            /MediaBox [0 0 {page_width} {page_height}]\n\
            /Contents 4 0 R\n\
            /Resources << /XObject << /Im0 5 0 R >> >> >>\n\
         endobj\n");

    let off4 = pdf.len();
    w!(
        "4 0 obj\n<< /Length {} >>\nstream\n{content}\nendstream\nendobj\n",
        content.len()
    );

    let off5 = pdf.len();
    w!(
        "5 0 obj\n\
         << /Type /XObject /Subtype /Image\n\
            /Width {img_w} /Height {img_h}\n\
            /ColorSpace {color_space}{decode_array}\n\
            /BitsPerComponent 8\n\
            /Filter /DCTDecode\n\
            /Length {} >>\n\
         stream\n",
        jpeg_bytes.len()
    );
    pdf.extend_from_slice(jpeg_bytes);
    pdf.extend_from_slice(b"\nendstream\nendobj\n");

    // Cross-reference table — each entry is exactly 20 bytes (PDF spec §7.5.4).
    let xref_pos = pdf.len();
    w!("xref\n0 6\n");
    w!("0000000000 65535 f \n"); // free-list head
    for off in [off1, off2, off3, off4, off5] {
        w!("{off:010} 00000 n \n");
    }
    w!("trailer\n<< /Size 6 /Root 1 0 R >>\nstartxref\n{xref_pos}\n%%EOF\n");

    Ok(pdf)
}

/// An adapter that builds a PDF using the JPEG's intrinsic dimensions
/// for the page width and height (mapping 1 pixel to 1 PDF point).
pub(crate) fn build_jpeg_pdf_auto_size(
    jpeg_bytes: &[u8],
) -> Result<Vec<u8>, PdfBuilderError> {
    let (width, height, _components) = jpeg_info(jpeg_bytes)?;

    build_jpeg_pdf(jpeg_bytes, width as f32, height as f32)
}

#[cfg(test)]
mod tests {
    use super::*;

    // ── helpers ──────────────────────────────────────────────────────────────

    /// Build the smallest byte sequence that makes `jpeg_info` happy:
    /// SOI + SOF0 + 8-byte payload.
    fn make_minimal_jpeg(width: u16, height: u16, components: u8) -> Vec<u8> {
        let mut data = vec![
            0xFF, 0xD8, // SOI
            0xFF, 0xC0, // SOF0
            0x00, 0x08, // segment length = 8 (includes the 2 length bytes)
            0x08, // precision = 8 bpc
        ];
        data.extend_from_slice(&height.to_be_bytes());
        data.extend_from_slice(&width.to_be_bytes());
        data.push(components);
        data
    }

    /// Prepend a dummy APP0 segment (FF E0, length=6, four zero bytes) before
    /// the SOF, so we exercise the "skip unknown segment" path.
    fn make_jpeg_with_app0(width: u16, height: u16, components: u8) -> Vec<u8> {
        let mut data = vec![
            0xFF, 0xD8, // SOI
            0xFF, 0xE0, // APP0
            0x00, 0x06, // length = 6 (2 length bytes + 4 data bytes)
            0x00, 0x00, 0x00, 0x00, // dummy payload
            0xFF, 0xC0, // SOF0
            0x00, 0x08, // segment length
            0x08, // precision
        ];
        data.extend_from_slice(&height.to_be_bytes());
        data.extend_from_slice(&width.to_be_bytes());
        data.push(components);
        data
    }

    // ── jpeg_info: happy paths ────────────────────────────────────────────────

    #[test]
    fn jpeg_info_rgb_sof0() {
        let jpeg = make_minimal_jpeg(160, 120, 3);
        assert_eq!(jpeg_info(&jpeg).unwrap(), (160, 120, 3));
    }

    #[test]
    fn jpeg_info_grayscale() {
        let jpeg = make_minimal_jpeg(64, 48, 1);
        assert_eq!(jpeg_info(&jpeg).unwrap(), (64, 48, 1));
    }

    #[test]
    fn jpeg_info_cmyk() {
        let jpeg = make_minimal_jpeg(200, 100, 4);
        assert_eq!(jpeg_info(&jpeg).unwrap(), (200, 100, 4));
    }

    /// SOF1 (extended sequential DCT) must also be recognised.
    #[test]
    fn jpeg_info_sof1_marker() {
        let mut jpeg = make_minimal_jpeg(32, 32, 3);
        jpeg[3] = 0xC1; // replace SOF0 with SOF1
        assert_eq!(jpeg_info(&jpeg).unwrap(), (32, 32, 3));
    }

    /// SOF2 (progressive DCT) must also be recognised.
    #[test]
    fn jpeg_info_sof2_marker() {
        let mut jpeg = make_minimal_jpeg(32, 32, 3);
        jpeg[3] = 0xC2;
        assert_eq!(jpeg_info(&jpeg).unwrap(), (32, 32, 3));
    }

    /// DHT (0xC4) is excluded from the SOF range; it must be skipped as a
    /// regular length-prefixed segment so the parser continues to the real SOF.
    #[test]
    fn jpeg_info_dht_skipped_before_sof() {
        let mut data = vec![
            0xFF, 0xD8, // SOI
            0xFF, 0xC4, // DHT — NOT a SOF, must be skipped
            0x00, 0x06, // length = 6
            0x00, 0x00, 0x00, 0x00, // dummy payload
        ];
        let sof = make_minimal_jpeg(10, 20, 1);
        data.extend_from_slice(&sof[2..]); // append from SOF0 onward
        assert_eq!(jpeg_info(&data).unwrap(), (10, 20, 1));
    }

    /// An RST marker (0xD0-D7) carries no length field and must be skipped.
    #[test]
    fn jpeg_info_rst_marker_skipped_before_sof() {
        let mut data = vec![
            0xFF, 0xD8, // SOI
            0xFF, 0xD0, // RST0 — no length field
        ];
        let sof = make_minimal_jpeg(8, 8, 3);
        data.extend_from_slice(&sof[2..]);
        assert_eq!(jpeg_info(&data).unwrap(), (8, 8, 3));
    }

    /// APP0 segment (unknown to the parser) must be skipped via its length.
    #[test]
    fn jpeg_info_app0_before_sof() {
        let jpeg = make_jpeg_with_app0(320, 240, 3);
        assert_eq!(jpeg_info(&jpeg).unwrap(), (320, 240, 3));
    }

    // ── jpeg_info: error paths ────────────────────────────────────────────────

    #[test]
    fn jpeg_info_empty_data() {
        assert!(jpeg_info(&[]).is_err());
    }

    #[test]
    fn jpeg_info_too_short() {
        assert!(jpeg_info(&[0xFF, 0xD8, 0xFF]).is_err());
    }

    #[test]
    fn jpeg_info_wrong_magic() {
        let mut jpeg = make_minimal_jpeg(1, 1, 3);
        jpeg[0] = 0x00; // corrupt SOI first byte
        assert!(jpeg_info(&jpeg).is_err());
    }

    #[test]
    fn jpeg_info_wrong_second_magic_byte() {
        let mut jpeg = make_minimal_jpeg(1, 1, 3);
        jpeg[1] = 0x00; // corrupt 0xD8
        assert!(jpeg_info(&jpeg).is_err());
    }

    #[test]
    fn jpeg_info_non_ff_marker_byte() {
        // Replace the FF before SOF0 with 0x00 — not a valid marker.
        let mut jpeg = make_minimal_jpeg(1, 1, 3);
        jpeg[2] = 0x00;
        assert!(jpeg_info(&jpeg).is_err());
    }

    #[test]
    fn jpeg_info_sof_payload_too_short() {
        // SOF0 at pos=4, needs pos+8 bytes available — only provide 11 (one short).
        let data = vec![
            0xFF, 0xD8, // SOI
            0xFF, 0xC0, // SOF0
            // only 7 bytes of payload instead of the required 8
            0x08, 0x00, 0x0A, 0x00, 0x14, 0x00, 0x14,
        ];
        assert!(jpeg_info(&data).is_err());
    }

    #[test]
    fn jpeg_info_segment_length_less_than_two() {
        let data = vec![
            0xFF, 0xD8, // SOI
            0xFF, 0xE0, // APP0
            0x00, 0x01, // length = 1 — illegal (must be >= 2)
        ];
        assert!(jpeg_info(&data).is_err());
    }

    #[test]
    fn jpeg_info_truncated_before_sof() {
        // Valid APP0 segment, but file ends before any SOF marker.
        let data = vec![
            0xFF, 0xD8, // SOI
            0xFF, 0xE0, // APP0
            0x00, 0x04, // length = 4
            0x00, 0x00, // two payload bytes
                  // EOF — no SOF
        ];
        assert!(jpeg_info(&data).is_err());
    }

    // ── build_jpeg_pdf: happy paths ───────────────────────────────────────────

    #[test]
    fn build_jpeg_pdf_rgb_returns_ok() {
        let jpeg = make_minimal_jpeg(100, 50, 3);
        assert!(build_jpeg_pdf(&jpeg, 100.0, 50.0).is_ok());
    }

    #[test]
    fn build_jpeg_pdf_starts_with_pdf_header() {
        let jpeg = make_minimal_jpeg(10, 10, 3);
        let pdf = build_jpeg_pdf(&jpeg, 10.0, 10.0).unwrap();
        assert!(pdf.starts_with(b"%PDF-1.4\n"), "PDF header missing");
    }

    #[test]
    fn build_jpeg_pdf_ends_with_eof() {
        let jpeg = make_minimal_jpeg(10, 10, 3);
        let pdf = build_jpeg_pdf(&jpeg, 10.0, 10.0).unwrap();
        assert!(pdf.ends_with(b"%%EOF\n"), "PDF %%EOF marker missing");
    }

    #[test]
    fn build_jpeg_pdf_contains_xref_and_startxref() {
        let jpeg = make_minimal_jpeg(10, 10, 3);
        let pdf = build_jpeg_pdf(&jpeg, 10.0, 10.0).unwrap();
        let text = String::from_utf8_lossy(&pdf);
        assert!(text.contains("xref\n"), "xref table missing");
        assert!(text.contains("startxref\n"), "startxref missing");
    }

    #[test]
    fn build_jpeg_pdf_contains_dct_filter() {
        let jpeg = make_minimal_jpeg(10, 10, 3);
        let pdf = build_jpeg_pdf(&jpeg, 10.0, 10.0).unwrap();
        assert!(
            String::from_utf8_lossy(&pdf).contains("/Filter /DCTDecode"),
            "DCTDecode filter missing"
        );
    }

    #[test]
    fn build_jpeg_pdf_embeds_jpeg_bytes_verbatim() {
        let jpeg = make_minimal_jpeg(10, 10, 3);
        let pdf = build_jpeg_pdf(&jpeg, 10.0, 10.0).unwrap();
        assert!(
            pdf.windows(jpeg.len()).any(|w| w == jpeg.as_slice()),
            "JPEG bytes not found verbatim in PDF output"
        );
    }

    #[test]
    fn build_jpeg_pdf_grayscale_uses_device_gray() {
        let jpeg = make_minimal_jpeg(10, 10, 1);
        let pdf = build_jpeg_pdf(&jpeg, 10.0, 10.0).unwrap();
        assert!(
            pdf.windows(11).any(|w| w == b"/DeviceGray"),
            "DeviceGray color space missing"
        );
    }

    #[test]
    fn build_jpeg_pdf_cmyk_uses_device_cmyk() {
        let jpeg = make_minimal_jpeg(10, 10, 4);
        let pdf = build_jpeg_pdf(&jpeg, 10.0, 10.0).unwrap();
        assert!(
            pdf.windows(11).any(|w| w == b"/DeviceCMYK"),
            "DeviceCMYK color space missing"
        );
    }

    #[test]
    fn build_jpeg_pdf_rgb_uses_device_rgb() {
        let jpeg = make_minimal_jpeg(10, 10, 3);
        let pdf = build_jpeg_pdf(&jpeg, 10.0, 10.0).unwrap();
        assert!(
            pdf.windows(10).any(|w| w == b"/DeviceRGB"),
            "DeviceRGB color space missing"
        );
    }

    #[test]
    fn build_jpeg_pdf_page_dimensions_in_media_box() {
        let jpeg = make_minimal_jpeg(10, 10, 3);
        let pdf = build_jpeg_pdf(&jpeg, 595.0, 842.0).unwrap();
        let text = String::from_utf8_lossy(&pdf);
        assert!(
            text.contains("/MediaBox [0 0 595 842]"),
            "MediaBox dimensions incorrect"
        );
    }

    // ── build_jpeg_pdf: error paths ───────────────────────────────────────────

    #[test]
    fn build_jpeg_pdf_invalid_jpeg_returns_err() {
        assert!(build_jpeg_pdf(&[0x00, 0x01, 0x02, 0x03], 10.0, 10.0).is_err());
    }

    #[test]
    fn build_jpeg_pdf_unsupported_components_returns_err() {
        let jpeg = make_minimal_jpeg(10, 10, 2); // 2-component is not valid
        let result = build_jpeg_pdf(&jpeg, 10.0, 10.0);
        assert!(matches!(
            result,
            Err(PdfBuilderError::Image(ImageError::UnsupportedColorSpace(2)))
        ));
    }

    // ── regression: SOS marker before SOF must be rejected ───────────────────

    #[test]
    fn jpeg_info_sos_before_sof_is_rejected() {
        // SOS (0xDA) appearing before any SOF must error out, not try to skip
        // by its length field (which would land in entropy-coded data).
        let data = vec![
            0xFF, 0xD8, // SOI
            0xFF, 0xDA, // SOS — illegal here, no SOF seen yet
            0x00, 0x0C, // length = 12 (would skip into garbage)
            0x03, 0x01, 0x00, 0x02, 0x11, 0x03, 0x11, 0x00, 0x3F,
            0x00,
            // (followed by what would be entropy data in a real file)
        ];
        assert!(jpeg_info(&data).is_err());
    }

    // ── regression: CMYK images get a /Decode array ──────────────────────────

    #[test]
    fn build_jpeg_pdf_cmyk_includes_decode_array() {
        let jpeg = make_minimal_jpeg(10, 10, 4);
        let pdf = build_jpeg_pdf(&jpeg, 10.0, 10.0).unwrap();
        assert!(
            String::from_utf8_lossy(&pdf).contains("/Decode [1.0 0.0 1.0 0.0 1.0 0.0 1.0 0.0]"),
            "CMYK /Decode array missing — image will render inverted"
        );
    }

    #[test]
    fn build_jpeg_pdf_rgb_has_no_decode_array() {
        let jpeg = make_minimal_jpeg(10, 10, 3);
        let pdf = build_jpeg_pdf(&jpeg, 10.0, 10.0).unwrap();
        assert!(
            !String::from_utf8_lossy(&pdf).contains("/Decode"),
            "RGB images should not have a /Decode array"
        );
    }

    #[test]
    fn build_jpeg_pdf_grayscale_has_no_decode_array() {
        let jpeg = make_minimal_jpeg(10, 10, 1);
        let pdf = build_jpeg_pdf(&jpeg, 10.0, 10.0).unwrap();
        assert!(
            !String::from_utf8_lossy(&pdf).contains("/Decode"),
            "Grayscale images should not have a /Decode array"
        );
    }

    #[test]
    fn build_jpeg_pdf_auto_size_uses_intrinsic_dimensions() {
        // Create a 300x150 RGB JPEG
        let jpeg = make_minimal_jpeg(300, 150, 3);

        // Build the PDF without passing explicit dimensions
        let pdf = build_jpeg_pdf_auto_size(&jpeg).unwrap();
        let text = String::from_utf8_lossy(&pdf);

        // Verify the MediaBox picked up the 300x150 dimensions
        assert!(
            text.contains("/MediaBox [0 0 300 150]"),
            "Auto-sized MediaBox dimensions incorrect"
        );
    }
}
