"""Bernard l'Édit - PDF, image, and geometry utilities."""

import os as _os
import sys as _sys
from pathlib import Path as _Path
from typing import Any as _Any

_pkg_dir = _Path(__file__).parent
_lib_name = (
    "pdfium.dll"
    if _os.name == "nt"
    else ("libpdfium.dylib" if _sys.platform == "darwin" else "libpdfium.so")
)
_bundled = _pkg_dir / _lib_name
if _bundled.exists():
    _os.environ.setdefault("PDFIUM_PATH", str(_bundled))

from . import _bernard_ledit as _native  # noqa: E402
from ._bernard_ledit import *  # noqa: E402, F401, F403

_native_any: _Any = _native
geometry = _native_any.geometry
pdf = _native_any.pdf
image = _native_any.image
_sys.modules[__name__ + ".geometry"] = geometry
_sys.modules[__name__ + ".pdf"] = pdf
_sys.modules[__name__ + ".image"] = image

__all__ = ["geometry", "pdf", "image"]
