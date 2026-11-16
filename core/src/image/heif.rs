use super::ImageError;
use ::image::{ColorType, DynamicImage, Limits, RgbImage};
use libheif_rs::{
    ColorSpace, CompressionFormat, DecodingOptions, HeifContext, LibHeif, Plane, RgbChroma,
};

fn decode_error(error: impl std::fmt::Display) -> ImageError {
    ImageError::Decode(format!("HEIC/HEIF: {error}"))
}

pub(super) fn decode(data: &[u8]) -> Result<DynamicImage, ImageError> {
    let lib = LibHeif::new_checked().map_err(decode_error)?;
    // currently we install only libde265 
    // but we can add ffmpeg as fallback decoder if needed (increase max_count to 2)
    let decoders = lib.decoder_descriptors(1, Some(CompressionFormat::Hevc));
    let descriptor = decoders.first().ok_or_else(|| {
        decode_error("no HEVC decoder is available; install libheif's libde265 decoder")
    })?;
    let context = HeifContext::read_from_bytes(data).map_err(decode_error)?;
    let handle = context.primary_image_handle().map_err(decode_error)?;
    check_size(handle.width(), handle.height())?;

    let mut options = DecodingOptions::new()
        .ok_or_else(|| decode_error("could not allocate decoding options"))?;
    options
        .set_decoder_id(Some(descriptor.id()))
        .map_err(decode_error)?;
    options.set_ignore_transformations(false);
    options.set_convert_hdr_to_8bit(true);
    options.set_strict_decoding(true);
    let decoded = lib
        .decode(&handle, ColorSpace::Rgb(RgbChroma::Rgba), Some(options))
        .map_err(decode_error)?;
    let plane = decoded
        .planes()
        .interleaved
        .ok_or_else(|| decode_error("decoder returned no interleaved pixels"))?;
    let rgb = rgb_on_white(&plane, handle.is_premultiplied_alpha())?;
    Ok(DynamicImage::ImageRgb8(rgb))
}

fn check_size(width: u32, height: u32) -> Result<(), ImageError> {
    if width == 0 || height == 0 {
        return Err(decode_error("image dimensions must be nonzero"));
    }
    // Account for both the temporary RGBA pixels and the final RGB buffer.
    let mut limits = Limits::default();
    limits
        .reserve_buffer(width, height, ColorType::Rgba8)
        .and_then(|()| limits.reserve_buffer(width, height, ColorType::Rgb8))
        .map_err(decode_error)
}

fn rgb_on_white(plane: &Plane<&[u8]>, premultiplied: bool) -> Result<RgbImage, ImageError> {
    check_size(plane.width, plane.height)?;
    if plane.storage_bits_per_pixel != 32 || plane.bits_per_pixel != 8 {
        return Err(decode_error("decoder did not return 8-bit RGBA pixels"));
    }
    let width = usize::try_from(plane.width).map_err(decode_error)?;
    let height = usize::try_from(plane.height).map_err(decode_error)?;
    let row_len = width
        .checked_mul(4)
        .ok_or_else(|| decode_error("row size overflow"))?;
    let plane_len = plane
        .stride
        .checked_mul(height)
        .ok_or_else(|| decode_error("plane size overflow"))?;
    if plane.stride < row_len || plane.data.len() < plane_len {
        return Err(decode_error("decoder returned an invalid pixel layout"));
    }
    let output_len = width
        .checked_mul(height)
        .and_then(|size| size.checked_mul(3))
        .ok_or_else(|| decode_error("RGB buffer size overflow"))?;
    let mut pixels = Vec::new();
    pixels.try_reserve_exact(output_len).map_err(decode_error)?;
    for row in plane.data[..plane_len].chunks_exact(plane.stride) {
        for rgba in row[..row_len].as_chunks::<4>().0 {
            let alpha = u16::from(rgba[3]);
            for &channel in &rgba[..3] {
                let channel = u16::from(channel);
                let blended = if premultiplied {
                    (channel + 255 - alpha).min(255)
                } else {
                    (channel * alpha + 255 * (255 - alpha) + 127) / 255
                };
                pixels.push(u8::try_from(blended).map_err(decode_error)?);
            }
        }
    }
    RgbImage::from_raw(plane.width, plane.height, pixels)
        .ok_or_else(|| decode_error("could not construct RGB image"))
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn composites_straight_and_premultiplied_alpha_and_skips_padding() {
        for (data, premultiplied) in [
            (&[0, 0, 0, 0, 200, 100, 0, 128, 9, 9, 9, 9][..], false),
            (&[0, 0, 0, 0, 100, 50, 0, 128, 9, 9, 9, 9][..], true),
        ] {
            let plane = Plane {
                data,
                width: 2,
                height: 1,
                stride: 12,
                bits_per_pixel: 8,
                storage_bits_per_pixel: 32,
            };
            let rgb = rgb_on_white(&plane, premultiplied).unwrap();
            assert_eq!(rgb.as_raw(), &[255, 255, 255, 227, 177, 127]);
        }
    }

    #[test]
    fn rejects_oversized_images_before_decoding() {
        assert!(check_size(65_536, 65_536).is_err());
    }
}
