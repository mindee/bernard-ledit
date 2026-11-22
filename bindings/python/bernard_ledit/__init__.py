"""Bernard l'Édit - PDF and image utilities."""

import sys as _sys
from typing import Any as _Any

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
