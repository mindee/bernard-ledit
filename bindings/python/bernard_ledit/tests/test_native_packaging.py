"""Checks for native dependency packaging."""

import subprocess
import sys
from pathlib import Path


def test_packaging_rejects_wrong_pdfium_release(tmp_path):
    root = Path(__file__).resolve().parents[4]
    pdfium = tmp_path / "pdfium"
    pdfium.mkdir()
    (pdfium / "VERSION").write_text("BUILD=0\n", encoding="utf-8")
    extension = tmp_path / "extension"
    extension.write_bytes(b"unchanged")

    result = subprocess.run(
        [
            sys.executable,
            str(root / "tools" / "bundle_native.py"),
            str(extension),
            str(tmp_path / "libheif"),
            str(pdfium),
        ],
        capture_output=True,
        text=True,
        check=False,
    )

    assert result.returncode != 0
    assert "expected PDFium" in result.stderr
    assert extension.read_bytes() == b"unchanged"
    assert not (tmp_path / "native").exists()
