import pytest

from bernard_ledit.pdf import apply_page_options

KEEP = "keep_only"
REMOVE = "remove"


def test_apply_page_options_valid_keep(test_data_dir):
    data = (test_data_dir / "file_types/pdf/multipage.pdf").read_bytes()
    doc, skipped = apply_page_options(data, KEEP, on_min_pages=1, page_indexes=[0, 2])

    assert doc is not None
    assert isinstance(skipped, list)

    assert doc.get_page(0) is not None


def test_apply_page_options_valid_remove(test_data_dir):
    data = (test_data_dir / "file_types/pdf/multipage.pdf").read_bytes()
    doc, skipped = apply_page_options(data, REMOVE, on_min_pages=1, page_indexes=[1])

    assert doc is not None
    assert doc.get_page(0) is not None


def test_apply_page_options_with_binary_io(test_data_dir):
    with open(test_data_dir / "file_types/pdf/multipage.pdf", "rb") as f:
        doc, skipped = apply_page_options(f, KEEP, on_min_pages=1, page_indexes=[0])

    assert doc is not None


def test_apply_page_options_min_pages_threshold(test_data_dir):
    data = (test_data_dir / "file_types/pdf/multipage.pdf").read_bytes()

    doc, skipped = apply_page_options(data, REMOVE, on_min_pages=50, page_indexes=[0])

    assert doc is None
    assert skipped == []


def test_apply_page_options_invalid_operation_raises(test_data_dir):
    data = (test_data_dir / "file_types/pdf/multipage.pdf").read_bytes()

    with pytest.raises(Exception, match="Invalid cut behavior specified: shred_it"):
        apply_page_options(data, "shred_it", on_min_pages=1, page_indexes=[0])


def test_apply_page_options_negative_indexing(test_data_dir):
    data = (test_data_dir / "file_types/pdf/multipage.pdf").read_bytes()
    doc, skipped = apply_page_options(data, REMOVE, on_min_pages=1, page_indexes=[-1])

    assert doc is not None
