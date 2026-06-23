from bernard_ledit.pdf import PdfDocument


def test_bitmap_to_bytes_matches_dimensions(test_data_dir):
    data = (test_data_dir / "file_types/pdf/blank_1.pdf").read_bytes()
    doc = PdfDocument(data)
    bitmap = doc.get_page(0).render()
    raw = bitmap.to_bytes()
    assert len(raw) == bitmap.width * bitmap.height * 4


def test_bitmap_to_png_bytes(test_data_dir):
    data = (test_data_dir / "file_types/pdf/blank_1.pdf").read_bytes()
    doc = PdfDocument(data)
    png = doc.get_page(0).render().to_png_bytes()
    assert png.startswith(b"\x89PNG\r\n\x1a\n")


def test_bitmap_repr(test_data_dir):
    data = (test_data_dir / "file_types/pdf/blank_1.pdf").read_bytes()
    doc = PdfDocument(data)
    bitmap = doc.get_page(0).render()
    assert repr(bitmap) == f"PdfBitmap({bitmap.width}x{bitmap.height})"
