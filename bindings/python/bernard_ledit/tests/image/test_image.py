import io
import os

import pytest

import bernard_ledit.image as bl_image
from bernard_ledit.image import Image, compress, guess_format

ImageError = getattr(
    bl_image, "ImageError", getattr(bl_image, "PyImageError", Exception)
)

PNG_MAGIC = b"\x89PNG\r\n\x1a\n"
JPEG_MAGIC = b"\xff\xd8\xff"


@pytest.fixture
def jpg_bytes(test_data_dir):
    return (test_data_dir / "file_types/receipt.jpg").read_bytes()


@pytest.fixture
def png_bytes(test_data_dir):
    return (test_data_dir / "file_types/receipt.png").read_bytes()


@pytest.fixture
def tiff_bytes(test_data_dir):
    return (test_data_dir / "file_types/receipt.tiff").read_bytes()


def test_compress_png(png_bytes):
    compressed = compress(png_bytes, quality=85, optimize=True)
    assert isinstance(compressed, bytes)
    assert 0 < len(compressed) < len(png_bytes)


def test_compress_jpg(jpg_bytes):
    compressed = compress(jpg_bytes, quality=85, optimize=True)
    assert isinstance(compressed, bytes)
    assert 0 < len(compressed) < len(jpg_bytes)


def test_compress_png_shrinks(png_bytes):
    out = compress(png_bytes, quality=85)
    assert len(out) <= len(png_bytes)
    assert Image(out).format == "JPEG"


def test_compress_with_bounds_always_resizes(jpg_bytes):
    out = compress(jpg_bytes, max_width=100, max_height=100)
    width, height = Image(out).size
    assert width <= 100 and height <= 100


def test_compress_accepts_file_like(jpg_bytes):
    out = compress(io.BytesIO(jpg_bytes), quality=85)
    assert isinstance(out, bytes)


def test_compress_garbage_errors():
    with pytest.raises(ImageError):
        compress(b"not an image")


@pytest.mark.parametrize(
    "fixture, expected",
    [("jpg_bytes", "JPEG"), ("png_bytes", "PNG"), ("tiff_bytes", "TIFF")],
)
def test_decode_records_format(request, fixture, expected):
    img = Image(request.getfixturevalue(fixture))
    assert img.format == expected
    width, height = img.size
    assert width > 0 and height > 0


def test_constructor_accepts_file_like(jpg_bytes):
    assert Image(io.BytesIO(jpg_bytes)).size == Image(jpg_bytes).size


def test_constructor_garbage_errors():
    with pytest.raises(ImageError):
        Image(b"\xde\xad\xbe\xef")


def test_constructor_empty_errors():
    with pytest.raises(ImageError):
        Image(b"")


def test_repr(jpg_bytes):
    img = Image(jpg_bytes)
    width, height = img.size
    assert repr(img) == f"Image({width}x{height}, format:JPEG)"


# --- crop ------------------------------------------------------------------------


def test_crop_valid_box(png_bytes):
    img = Image(png_bytes)
    width, height = img.size
    cropped = img.crop(0, 0, width // 2, height // 2)
    assert cropped.size == (width // 2, height // 2)


def test_crop_preserves_format(png_bytes):
    assert Image(png_bytes).crop(0, 0, 10, 10).format == "PNG"


def test_crop_does_not_modify_original(png_bytes):
    img = Image(png_bytes)
    size = img.size
    img.crop(0, 0, 10, 10)
    assert img.size == size


def test_crop_out_of_bounds_errors(png_bytes):
    img = Image(png_bytes)
    width, height = img.size
    with pytest.raises(ImageError):
        img.crop(0, 0, width + 1, height)


def test_crop_inverted_box_errors(png_bytes):
    with pytest.raises(ImageError):
        Image(png_bytes).crop(20, 5, 10, 25)


def test_crop_zero_size_box_errors(png_bytes):
    with pytest.raises(ImageError):
        Image(png_bytes).crop(5, 5, 5, 20)


def test_crop_negative_value_raises(png_bytes):
    # u32 parameters: pyo3 rejects negatives before the core is reached.
    with pytest.raises(OverflowError):
        Image(png_bytes).crop(-1, 0, 10, 10)


# --- resize ----------------------------------------------------------------------


def test_resize_exact_dimensions(jpg_bytes):
    assert Image(jpg_bytes).resize(80, 20).size == (80, 20)


def test_resize_can_upscale(jpg_bytes):
    width, height = Image(jpg_bytes).size
    assert Image(jpg_bytes).resize(width * 2, height * 2).size == (
        width * 2,
        height * 2,
    )


def test_resize_preserves_format(png_bytes):
    assert Image(png_bytes).resize(10, 10).format == "PNG"


@pytest.mark.parametrize("width, height", [(0, 20), (20, 0), (0, 0)])
def test_resize_zero_dimension_errors(jpg_bytes, width, height):
    with pytest.raises(ImageError):
        Image(jpg_bytes).resize(width, height)


def test_encode_jpeg_magic_bytes(png_bytes):
    out = Image(png_bytes).encode("JPEG", quality=85)
    assert isinstance(out, bytes)
    assert out.startswith(JPEG_MAGIC)


def test_encode_png_magic_bytes(jpg_bytes):
    assert Image(jpg_bytes).encode("PNG").startswith(PNG_MAGIC)


def test_encode_format_is_case_insensitive(jpg_bytes):
    img = Image(jpg_bytes)
    assert img.encode("png").startswith(PNG_MAGIC)
    assert img.encode("Jpeg").startswith(JPEG_MAGIC)


def test_encode_pdf_produces_single_page_pdf(jpg_bytes):
    out = Image(jpg_bytes).encode("PDF")
    assert out.startswith(b"%PDF-")
    assert b"/DCTDecode" in out


def test_encode_jpeg_roundtrip_keeps_size(png_bytes):
    img = Image(png_bytes)
    reloaded = Image(img.encode("JPEG", quality=90))
    assert reloaded.size == img.size
    assert reloaded.format == "JPEG"


def test_encode_lower_quality_is_not_larger(jpg_bytes):
    img = Image(jpg_bytes)
    high = img.encode("JPEG", quality=95, optimize=False)
    low = img.encode("JPEG", quality=20, optimize=False)
    assert len(low) <= len(high)


def test_encode_optimize_is_not_larger(jpg_bytes):
    img = Image(jpg_bytes)
    plain = img.encode("JPEG", quality=85, optimize=False)
    optimized = img.encode("JPEG", quality=85, optimize=True)
    assert len(optimized) <= len(plain)


def test_encode_unknown_format_errors(jpg_bytes):
    with pytest.raises(ImageError):
        Image(jpg_bytes).encode("not-a-format")


@pytest.mark.parametrize("quality", [0, 101])
def test_encode_jpeg_invalid_quality_errors(jpg_bytes, quality):
    with pytest.raises(ImageError):
        Image(jpg_bytes).encode("JPEG", quality=quality)


def test_encode_pdf_invalid_quality_errors(jpg_bytes):
    with pytest.raises(ImageError):
        Image(jpg_bytes).encode("PDF", quality=0)


@pytest.mark.parametrize(
    "name, magic",
    [
        ("out.png", PNG_MAGIC),
        ("out.jpg", JPEG_MAGIC),
        ("out.jpeg", JPEG_MAGIC),
        ("out.pdf", b"%PDF-"),
    ],
)
def test_save_to_path_infers_format_from_extension(jpg_bytes, tmp_path, name, magic):
    dest = tmp_path / name
    Image(jpg_bytes).save(str(dest))
    assert dest.read_bytes().startswith(magic)


def test_save_tif_extension_maps_to_tiff(jpg_bytes, tmp_path):
    dest = tmp_path / "out.tif"
    Image(jpg_bytes).save(dest)
    assert Image(dest.read_bytes()).format == "TIFF"


def test_save_accepts_pathlike(jpg_bytes, tmp_path):
    dest = tmp_path / "out.png"
    Image(jpg_bytes).save(dest)  # pathlib.Path implements os.PathLike
    assert os.path.getsize(dest) > 0


def test_save_explicit_format_overrides_extension(jpg_bytes, tmp_path):
    dest = tmp_path / "out.jpg"
    Image(jpg_bytes).save(dest, format="PNG")
    assert dest.read_bytes().startswith(PNG_MAGIC)


def test_save_to_buffer_uses_image_format(png_bytes):
    buf = io.BytesIO()
    Image(png_bytes).save(buf)
    assert buf.getvalue().startswith(PNG_MAGIC)


def test_save_to_buffer_with_explicit_format(png_bytes):
    buf = io.BytesIO()
    Image(png_bytes).save(buf, format="PDF")
    assert buf.getvalue().startswith(b"%PDF-")


def test_save_roundtrips(png_bytes, tmp_path):
    img = Image(png_bytes)
    dest = tmp_path / "out.png"
    img.save(dest)
    assert Image(dest.read_bytes()).size == img.size


def test_save_unknown_extension_errors(jpg_bytes, tmp_path):
    with pytest.raises(ImageError):
        Image(jpg_bytes).save(tmp_path / "out.notaformat")


def test_save_unknown_format_errors(jpg_bytes):
    with pytest.raises(ImageError):
        Image(jpg_bytes).save(io.BytesIO(), format="nope")


def test_save_to_missing_directory_raises_ioerror(jpg_bytes, tmp_path):
    with pytest.raises(OSError):
        Image(jpg_bytes).save(tmp_path / "missing" / "out.jpg")


@pytest.mark.parametrize(
    "fixture, expected",
    [("jpg_bytes", "JPEG"), ("png_bytes", "PNG"), ("tiff_bytes", "TIFF")],
)
def test_guess_format_from_bytes(request, fixture, expected):
    assert guess_format(request.getfixturevalue(fixture)) == expected


def test_guess_format_from_file_like(png_bytes):
    assert guess_format(io.BytesIO(png_bytes)) == "PNG"


def test_guess_format_garbage_errors():
    with pytest.raises(ImageError):
        guess_format(b"\x00\x01\x02\x03")


def test_guess_format_empty_errors():
    with pytest.raises(ImageError):
        guess_format(b"")
