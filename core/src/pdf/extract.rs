use crate::image::{Image, OutputFormat, guess_format};
use crate::pdf::Document;
use crate::pdf::error::PdfExtractError;
use ::image::ImageFormat;

/// Builds one new `Document` per group of 0-indexed page numbers.
/// # Errors
/// Returns a [`PdfExtractError`] if the data cannot be decoded as a PDF, if the groups are empty,
/// or if any group is empty.
pub fn extract_from_page_ranges(
    data: &[u8],
    groups: &[Vec<u16>],
) -> Result<Vec<Document>, PdfExtractError> {
    if groups.is_empty() {
        return Err(PdfExtractError::NoGroups);
    }
    if let Some(i) = groups.iter().position(Vec::is_empty) {
        return Err(PdfExtractError::EmptyGroup(i));
    }
    let source = load_as_pdf(data)?;
    groups
        .iter()
        .map(|group| {
            let mut out = Document::new()?;
            out.import_pages(&source, group)?;
            Ok(out)
        })
        .collect()
}

/// Loads the given data as a `Document`.
/// # Errors
/// Returns a [`PdfExtractError`] if the data cannot be decoded as a PDF.
fn load_as_pdf(data: &[u8]) -> Result<Document, PdfExtractError> {
    if data.starts_with(b"%PDF-") {
        return Ok(Document::from_bytes(data.to_vec())?);
    }
    if guess_format(data)? == ImageFormat::Jpeg {
        return Ok(Document::from_jpeg_autosize(data)?);
    }
    let pdf = Image::decode(data)?.encode(OutputFormat::Pdf, 85, true)?;
    Ok(Document::from_bytes(pdf)?)
}
