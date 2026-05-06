use pdfium_render::prelude::*;

/// Main struct for handling PDF operations.
pub struct PdfProcessor {
    pdfium: Pdfium,
}

impl PdfProcessor {
    /// Initializes `PDFium` using the statically linked library from our build.rs
    /// # Errors
    /// * `PdfiumError` - If `PDFium` library binding fails.
    pub fn new(library_path: &str) -> Result<Self, PdfiumError> {
        let bindings = Pdfium::bind_to_library(library_path)?;
        let pdfium = Pdfium::new(bindings);
        Ok(Self { pdfium })
    }

    /// Open PDF and get page count
    /// # Errors
    /// * `PdfiumError` - If PDF loading fails.
    pub fn get_page_count(&self, file_path: &str) -> Result<u16, PdfiumError> {
        let document = self.pdfium.load_pdf_from_file(file_path, None)?;
        let count = document.pages().len().try_into().unwrap_or(0);

        Ok(count)
    }

    // Future methods will go here:
    // pub fn extract_text(&self, file_path: &str) -> ...
    // pub fn rasterize_page(&self, file_path: &str, page_index: u16) -> ...
}

#[cfg(test)]
mod tests {
    use super::*;
    use std::path::PathBuf;

    #[test]
    fn test_initialization() {
        let out_dir = env!("OUT_DIR");

        #[cfg(target_os = "windows")]
        let lib_name = "pdfium.dll";
        #[cfg(target_os = "macos")]
        let lib_name = "libpdfium.dylib";
        #[cfg(target_os = "linux")]
        let lib_name = "libpdfium.so";

        let lib_path = PathBuf::from(out_dir).join(lib_name);

        assert!(
            lib_path.exists(),
            "PDFium dynamic library not found at {}",
            lib_path.display()
        );

        let lib_path_str = lib_path.to_str().expect("Invalid UTF-8 in library path");
        let processor = PdfProcessor::new(lib_path_str);

        assert!(
            processor.is_ok(),
            "Failed to initialize dynamically linked PDFium: {:?}",
            processor.err()
        );
    }
}
