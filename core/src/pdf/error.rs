use pdfium_render::prelude::PdfiumError;

/// Combines PDF and image errors for builder operations like [`Document::from_jpeg`].
#[derive(Debug, thiserror::Error)]
pub enum PdfBuilderError {
    /// A PDF-level error.
    #[error("PDF processing error: {0}")]
    Pdf(#[from] PdfError),
    /// An image-level error.
    #[error("Image processing error: {0}")]
    Image(#[from] ImageError),
}

/// An error that can occur when working with PDF documents.
#[derive(Debug, thiserror::Error)]
pub enum PdfError {
    /// An error that occurred when interacting with the `PDFium` library.
    #[error("PDFium error: {0}")]
    Pdfium(#[from] PdfiumError),
    /// An error that occurred when reading or writing to a file.
    #[error("I/O error: {0}")]
    Io(#[from] std::io::Error),
    /// An error that occurred during PDF document processing.
    #[error("{0}")]
    Other(String),
    /// An error that occurred when the page count is out of bounds.
    #[error("Page count out of bounds")]
    PageCountOutOfBounds,
    /// A page index supplied to `import_pages` exceeds the source document's page count.
    #[error("Page index {index} out of bounds (document has {page_count} pages)")]
    PageIndexOutOfBounds {
        /// The index of the page that was out of bounds.
        index: u16,
        /// The total number of pages in the source document.
        page_count: u16,
    },
    /// An error that occurred when creating a new PDF document.
    #[error("Document creation failed")]
    DocumentCreationFailed,
}

/// An error that can occur when working with images.
#[derive(Debug, thiserror::Error)]
pub enum ImageError {
    /// JPEG data is malformed or too short to parse.
    #[error("Invalid or truncated JPEG data")]
    InvalidJpeg,
    /// JPEG component count is not 1 (gray), 3 (RGB), or 4 (CMYK).
    #[error("Unsupported JPEG color space ({0} components)")]
    UnsupportedColorSpace(u8),
    /// An error that occurred when adding an image object to a page.
    #[error("Error adding image object to page")]
    AddObject,
}
