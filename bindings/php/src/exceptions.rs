//! The binding's exception classes.
//!
//! They're registered without the Rust object `#[php_class]` gives a class, so they keep
//! `\Exception`'s constructor and `new ImageException('message')` works. They're declared in
//! `tools/stubs/exceptions.stubs.php`, since the stub generator only sees `#[php_class]` classes.

use ext_php_rs::zend::ClassEntry;
use ext_php_rs::{builders::ClassBuilder, error::Result, exception::PhpException, zend::ce};

/// Input that isn't an identifiable or decodable image, and encoding failures.
pub const IMAGE_EXCEPTION: &str = "BernardLedit\\Image\\ImageException";
/// `PDFium` can't be loaded, or fails to read, render or write a document.
pub const PDFIUM_EXCEPTION: &str = "BernardLedit\\Pdf\\PdfiumException";

/// Registers the classes; called from MINIT.
///
/// # Errors
/// Returns an error if a class can't be registered.
pub fn register() -> Result<()> {
    for name in [IMAGE_EXCEPTION, PDFIUM_EXCEPTION] {
        ClassBuilder::new(name)
            .extends((ce::exception, "\\Exception"))
            // No methods, so there are no argument tables to keep alive.
            .registration(|_, _| {})
            .register()?;
    }
    Ok(())
}

/// A `BernardLedit\Image\ImageException` with `message`.
#[must_use]
pub fn image_exception(message: String) -> PhpException {
    exception(IMAGE_EXCEPTION, message)
}

/// A `BernardLedit\Pdf\PdfiumException` with `message`.
#[must_use]
pub fn pdfium_exception(message: String) -> PhpException {
    exception(PDFIUM_EXCEPTION, message)
}

/// An exception of class `name`, or a plain `\Exception` if the class isn't registered.
fn exception(name: &str, message: String) -> PhpException {
    match ClassEntry::try_find(name) {
        Some(class) => PhpException::new(message, 0, class),
        None => PhpException::from_message(message),
    }
}
