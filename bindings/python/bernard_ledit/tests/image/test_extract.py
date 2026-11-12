import pytest

from bernard_ledit.image import Image, extract_from_polygons

FULL = [(0.0, 0.0), (1.0, 0.0), (1.0, 1.0), (0.0, 1.0)]
TOP_LEFT_QUARTER = [(0.0, 0.0), (0.5, 0.0), (0.5, 0.5), (0.0, 0.5)]


@pytest.fixture
def receipt(test_data_dir):
    return Image((test_data_dir / "file_types/receipt.jpg").read_bytes())


def test_extract_full_page_polygon_keeps_size(receipt):
    (img,) = extract_from_polygons(receipt, [FULL])
    assert img.size == receipt.size
    assert img.format == "JPEG"


def test_extract_quarter_polygon_is_half_size(receipt):
    width, height = receipt.size
    (img,) = extract_from_polygons(receipt, [TOP_LEFT_QUARTER])
    cw, ch = img.size
    assert abs(cw - width / 2) <= 1
    assert abs(ch - height / 2) <= 1


def test_extract_returns_one_image_per_polygon(receipt):
    images = extract_from_polygons(receipt, [FULL, TOP_LEFT_QUARTER])
    assert len(images) == 2


def test_extract_requires_an_image(test_data_dir):
    data = (test_data_dir / "file_types/receipt.jpg").read_bytes()
    with pytest.raises(TypeError):
        extract_from_polygons(data, [FULL])


def test_extract_empty_polygon_errors(receipt):
    with pytest.raises(ValueError):
        extract_from_polygons(receipt, [[]])


def test_extract_polygon_outside_image_errors(receipt):
    with pytest.raises(ValueError):
        extract_from_polygons(receipt, [[(2.0, 2.0), (3.0, 3.0)]])
