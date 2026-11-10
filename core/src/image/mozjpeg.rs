use std::sync::Mutex;

static MOZJPEG_LOCK: Mutex<()> = Mutex::new(());

/// Encodes raw RGB8 pixels as a JPEG using mozjpeg.
///
/// More or less matches pillow's way of compressing JPEGs.
///
/// # Errors
/// Returns an `io::Error` if the mozjpeg encoder fails.
pub(crate) fn encode_mozjpeg(
    data: &[u8],
    width: u32,
    height: u32,
    quality: u8,
    optimize: bool,
) -> std::io::Result<Vec<u8>> {
    let _lock = MOZJPEG_LOCK
        .lock()
        .unwrap_or_else(std::sync::PoisonError::into_inner);
    let mut comp = mozjpeg::Compress::new(mozjpeg::ColorSpace::JCS_RGB);
    comp.set_size(width as usize, height as usize);
    comp.set_quality(f32::from(quality));
    comp.set_optimize_coding(optimize);
    let mut started = comp.start_compress(Vec::new())?;
    started.write_scanlines(data)?;
    started.finish()
}
