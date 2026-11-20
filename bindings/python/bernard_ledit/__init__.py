"""Bernard l'Édit - PDF and image utilities."""

import sys as _sys
from pathlib import Path as _Path
from typing import Any as _Any

if _sys.platform == "win32":
    import os as _os

    _native_dir = _Path(__file__).parent / "native"
    if _native_dir.is_dir():
        _native_dll_directory = _os.add_dll_directory(str(_native_dir))

from . import _bernard_ledit as _native
from ._bernard_ledit import *  # noqa: F401, F403

_native_any: _Any = _native
common = _native_any.common
pdf = _native_any.pdf
image = _native_any.image
_sys.modules[__name__ + ".common"] = common
_sys.modules[__name__ + ".pdf"] = pdf
_sys.modules[__name__ + ".image"] = image

__all__ = ["common", "pdf", "image"]
