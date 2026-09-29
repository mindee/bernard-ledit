//! PDF, image, and geometry utilities for the Mindee SDK ecosystem.

/// Include bytes from a file under the workspace-level `tests/data/` directory.
///
/// The path is resolved relative to `<workspace>/tests/data/`, regardless of the
/// location of the source file invoking the macro. This avoids brittle
/// `../../../tests/data/...` paths in unit tests.
///
/// # Example
/// ```ignore
/// let bytes = test_data_bytes!("file_types/receipt.jpg");
/// ```
#[cfg(test)]
#[allow(unused_macros)]
macro_rules! test_data_bytes {
    ($path:literal) => {
        include_bytes!(concat!(
            env!("CARGO_MANIFEST_DIR"),
            "/../tests/data/",
            $path
        ))
    };
}

/// Geometry module.
pub mod geometry;
/// Image module.
#[cfg(feature = "image")]
pub mod image;
/// PDF module.
pub mod pdf;
