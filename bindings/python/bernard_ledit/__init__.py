"""Bernard l'Édit - PDF, image, and geometry utilities."""

import sys as _sys

from . import _bernard_ledit as _native
from ._bernard_ledit import *  # noqa: F401, F403

geometry = _native.geometry
_sys.modules[__name__ + ".geometry"] = geometry

__all__ = ["geometry"]
