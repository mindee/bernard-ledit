"""Bernard l'Édit - PDF and image utilities."""

import os as _os
import sys as _sys
from typing import Any as _Any

# assign DLL directory handle for Windows to module level variable to keep it alive
# and avoid it being garbage collected.
_dll_directory_handle: object | None = None
if _sys.platform == "win32":
    _libheif_dll_dir = _os.environ.get("LIBHEIF_DLL_DIR")
    if _libheif_dll_dir:
        # since Python 3.8 the add_dll_directory function is necessary on windows
        # to load DLLs from custom directories for security reasons.
        _dll_directory_handle = _os.add_dll_directory(  # type: ignore[attr-defined]
            _libheif_dll_dir
        )

from . import _bernard_ledit as _native  # noqa: E402
from ._bernard_ledit import *  # noqa: E402, F401, F403

_native_any: _Any = _native
common = _native_any.common
pdf = _native_any.pdf
image = _native_any.image
_sys.modules[__name__ + ".common"] = common
_sys.modules[__name__ + ".pdf"] = pdf
_sys.modules[__name__ + ".image"] = image

__all__ = ["common", "pdf", "image"]
