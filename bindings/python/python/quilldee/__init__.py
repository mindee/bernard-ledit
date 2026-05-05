"""Quilldee — PDF, image, and geometry utilities."""

import sys as _sys

from ._quilldee import *  # noqa: F401, F403
from . import _quilldee as _native

geometry = _native.geometry
_sys.modules[__name__ + ".geometry"] = geometry

__all__ = ["geometry"]
