import pytest

from bernard_ledit.image import Image

FULL = [(0.0, 0.0), (1.0, 0.0), (1.0, 1.0), (0.0, 1.0)]
TOP_LEFT_QUARTER = [(0.0, 0.0), (0.5, 0.0), (0.5, 0.5), (0.0, 0.5)]


@pytest.fixture
def receipt(test_data_dir):
    return Image((test_data_dir / "file_types/receipt.jpg").read_bytes())


def test_crop_full_image_polygon_keeps_size(receipt):
    img = receipt.crop_polygon(FULL)
    assert img.size == receipt.size
    assert img.format == "JPEG"


def test_crop_quarter_polygon_is_half_size(receipt):
    width, height = receipt.size
    img = receipt.crop_polygon(TOP_LEFT_QUARTER)
    cw, ch = img.size
    assert abs(cw - width / 2) <= 1
    assert abs(ch - height / 2) <= 1


def test_crop_multiple_polygons_independently(receipt):
    crops = [receipt.crop_polygon(polygon) for polygon in [FULL, TOP_LEFT_QUARTER]]
    assert len(crops) == 2


def test_crop_empty_polygon_errors(receipt):
    with pytest.raises(ValueError):
        receipt.crop_polygon([])


def test_crop_polygon_outside_image_errors(receipt):
    with pytest.raises(ValueError):
        receipt.crop_polygon([(2.0, 2.0), (3.0, 3.0)])
