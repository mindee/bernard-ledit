use crate::common::{Format, OutputFormat, guess_format};
use crate::image::Image;
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
    match guess_format(data)? {
        Format::Pdf => return Ok(Document::decode(data)?),
        Format::Image(ImageFormat::Jpeg) => {
            return Ok(Document::from_jpeg_autosize(data)?);
        }
        _ => {}
    }
    let pdf = Image::decode(data)?.encode(OutputFormat::Pdf, 85, true)?;
    Ok(Document::decode(&pdf)?)
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn extracts_heic_as_a_single_page_pdf() {
        let bytes = include_bytes!("../../tests/fixtures/rotated.heic");
        let documents = extract_from_page_ranges(bytes, &[vec![0]]).unwrap();
        assert_eq!(documents.len(), 1);
        assert_eq!(documents[0].page_count().unwrap(), 1);
        let encoded = documents[0].to_bytes().unwrap();
        assert_eq!(guess_format(&encoded).unwrap(), Format::Pdf);
    }
}
