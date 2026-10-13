from bernard_ledit.image import (
    Image,
    compress,
)


def test_compress_png(test_data_dir):
    data = (test_data_dir / "file_types/receipt.png").read_bytes()
    print(data[:10])
    compressed_bytes = compress(data, quality=85, optimize=True)

    assert isinstance(compressed_bytes, bytes)
    assert len(compressed_bytes) > 0
    assert len(compressed_bytes) < len(data)


def test_compress_jpg(test_data_dir):
    data = (test_data_dir / "file_types/receipt.jpg").read_bytes()
    compressed_bytes = compress(data, quality=85, optimize=True)

    assert isinstance(compressed_bytes, bytes)
    assert len(compressed_bytes) > 0
    assert len(compressed_bytes) < len(data)


def test_compress_png_shrinks(test_data_dir):
    data = (test_data_dir / "file_types/receipt.png").read_bytes()
    out = compress(data, quality=85)
    assert len(out) <= len(data)
    img = Image(out)  # needs the #[new] constructor
    assert img.format == "JPEG"


def test_compress_with_bounds_always_resizes(test_data_dir):
    data = (test_data_dir / "file_types/receipt.jpg").read_bytes()
    out = compress(data, max_width=100, max_height=100)
    w, h = Image(out).size
    assert w <= 100 and h <= 100
