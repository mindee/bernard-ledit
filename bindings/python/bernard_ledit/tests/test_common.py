import pytest

from bernard_ledit.common import format_from_extension


@pytest.mark.parametrize(
    ("extension", "expected"),
    [
        ("jpg", "JPEG"),
        (".jpeg", "JPEG"),
        ("tif", "TIFF"),
        ("exr", "OPENEXR"),
        ("pdf", "PDF"),
        ("", "JPEG"),
        ("custom", "CUSTOM"),
    ],
)
def test_format_from_extension(extension, expected):
    assert format_from_extension(extension) == expected
