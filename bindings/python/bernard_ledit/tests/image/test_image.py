import io

import pytest

from bernard_ledit.image import Image, ImageError, compress, decode, guess_format


@pytest.fixture
def make_png(test_data_dir):
    """Factory returning a PNG byte stream of exact dimensions.

    Derives from the real receipt.png fixture (resize + re-encode) so tests get
    deterministic width/height without depending on Pillow or hand-built bytes.
    """
    base = decode((test_data_dir / "file_types/receipt.png").read_bytes())

    def _make(width: int, height: int) -> bytes:
        return base.resize(width, height).encode("PNG")

    return _make


def test_decode_jpeg_fixture(test_data_dir):
    data = (test_data_dir / "file_types/receipt.jpg").read_bytes()
    img = decode(data)
    assert isinstance(img, Image)
    w, h = img.size
    assert w > 0 and h > 0
    assert img.format == "JPEG"


def test_decode_png_fixture(test_data_dir):
    data = (test_data_dir / "file_types/receipt.png").read_bytes()
    img = decode(data)
    assert img.format == "PNG"


def test_decode_garbage_raises():
    with pytest.raises(ImageError):
        decode(b"\xde\xad\xbe\xef")



def test_guess_format_jpeg(test_data_dir):
    data = (test_data_dir / "file_types/receipt.jpg").read_bytes()
    assert guess_format(data) == "JPEG"


def test_guess_format_png(test_data_dir):
    data = (test_data_dir / "file_types/receipt.png").read_bytes()
    assert guess_format(data) == "PNG"


def test_guess_format_tiff(test_data_dir):
    data = (test_data_dir / "file_types/receipt.tiff").read_bytes()
    assert guess_format(data) == "TIFF"


def test_guess_format_garbage_raises():
    with pytest.raises(ValueError):
        guess_format(b"\x00\x01\x02\x03")


def test_size_property(make_png):
    img = decode(make_png(30, 20))
    assert img.size == (30, 20)


def test_format_property(make_png):
    img = decode(make_png(10, 10))
    assert img.format == "PNG"


def test_crop_valid_box(make_png):
    img = decode(make_png(100, 80))
    cropped = img.crop(10, 20, 60, 70)
    assert cropped.size == (50, 50)


def test_crop_out_of_bounds_raises(make_png):
    img = decode(make_png(30, 30))
    with pytest.raises(ValueError):
        img.crop(0, 0, 31, 10)


def test_crop_inverted_box_raises(make_png):
    img = decode(make_png(30, 30))
    with pytest.raises(ValueError):
        img.crop(20, 5, 10, 25)


def test_resize_exact(make_png):
    img = decode(make_png(40, 40))
    resized = img.resize(80, 20)
    assert resized.size == (80, 20)


def test_resize_default_filter_is_lanczos(make_png):
    img = decode(make_png(40, 40))
    # Default filter argument must be accepted without error.
    assert img.resize(10, 10).size == (10, 10)


def test_resize_named_filters(make_png):
    img = decode(make_png(40, 40))
    for name in ("nearest", "triangle", "catmullrom", "bicubic", "gaussian", "lanczos"):
        assert img.resize(20, 20, name).size == (20, 20)


def test_resize_unknown_filter_raises(make_png):
    img = decode(make_png(40, 40))
    with pytest.raises(ValueError):
        img.resize(20, 20, "sinc")


def test_resize_zero_dimension_raises(make_png):
    img = decode(make_png(40, 40))
    with pytest.raises(ValueError):
        img.resize(0, 20)

def test_encode_jpeg_magic(make_png):
    img = decode(make_png(16, 16))
    out = img.encode("JPEG")
    assert out[:3] == b"\xff\xd8\xff"


def test_encode_png_magic(make_png):
    img = decode(make_png(16, 16))
    out = img.encode("PNG")
    assert out[:8] == b"\x89PNG\r\n\x1a\n"


def test_encode_quality_argument(make_png):
    img = decode(make_png(64, 64))
    out = img.encode("JPEG", quality=50)
    assert out[:3] == b"\xff\xd8\xff"


def test_encode_optimize_is_noop(make_png):
    img = decode(make_png(32, 32))
    assert img.encode("JPEG", 85, True) == img.encode("JPEG", 85, False)


def test_encode_unknown_format_raises(make_png):
    img = decode(make_png(16, 16))
    with pytest.raises(ValueError):
        img.encode("NOPE")


def test_encode_roundtrip_readable(make_png):
    img = decode(make_png(24, 12))
    jpeg = img.encode("JPEG", quality=90)
    reloaded = decode(jpeg)
    assert reloaded.size == (24, 12)
    assert reloaded.format == "JPEG"

def test_repr_contains_dimensions_and_format(make_png):
    img = decode(make_png(24, 18))
    text = repr(img)
    assert "24x18" in text
    assert "PNG" in text


def test_compress_returns_jpeg_and_dimensions(make_png):
    data = make_png(200, 100)
    jpeg, w, h = compress(data, max_width=100, max_height=100)
    assert jpeg[:3] == b"\xff\xd8\xff"
    assert (w, h) == (100, 50)


def test_compress_does_not_upscale(make_png):
    data = make_png(30, 30)
    _, w, h = compress(data, max_width=500, max_height=500)
    assert (w, h) == (30, 30)


def test_compress_no_bounds_keeps_dimensions(make_png):
    data = make_png(64, 48)
    _, w, h = compress(data)
    assert (w, h) == (64, 48)


def test_compress_default_quality(make_png):
    data = make_png(50, 50)
    jpeg, _, _ = compress(data)
    assert jpeg[:3] == b"\xff\xd8\xff"


def test_compress_fixture_jpeg(test_data_dir):
    data = (test_data_dir / "file_types/receipt.jpg").read_bytes()
    jpeg, w, h = compress(data, quality=80, max_width=512, max_height=512)
    assert jpeg[:3] == b"\xff\xd8\xff"
    assert w <= 512 and h <= 512


def test_compress_output_is_bytes(make_png):
    data = make_png(40, 40)
    jpeg, _, _ = compress(data)
    assert isinstance(jpeg, bytes)
    # Sanity: the returned bytes are a valid stream a BytesIO can wrap.
    assert io.BytesIO(jpeg).read(3) == b"\xff\xd8\xff"


def test_compress_garbage_raises():
    with pytest.raises(ImageError):
        compress(b"\x00\x01")
