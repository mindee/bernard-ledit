"""Verify that the pinned runtime libheif has a usable HEVC decoder."""

import ctypes
import os
import sys

library = sys.argv[1] if len(sys.argv) > 1 else "libheif.so.1"
if os.name == "nt":
    dll_directory = os.add_dll_directory(os.path.dirname(os.path.abspath(library)))
heif = ctypes.CDLL(library)
heif.heif_get_version.restype = ctypes.c_char_p
version = heif.heif_get_version()
if version != b"1.21.1":
    raise SystemExit(f"libheif 1.21.1 required, found {version!r}")


class HeifError(ctypes.Structure):
    _fields_ = [
        ("code", ctypes.c_int),
        ("subcode", ctypes.c_int),
        ("message", ctypes.c_char_p),
    ]


heif.heif_init.argtypes = [ctypes.c_void_p]
heif.heif_init.restype = HeifError
error = heif.heif_init(None)
if error.code:
    raise SystemExit(f"libheif initialization failed: {error.message!r}")

heif.heif_have_decoder_for_format.argtypes = [ctypes.c_int]
heif.heif_have_decoder_for_format.restype = ctypes.c_int
if not heif.heif_have_decoder_for_format(1):  # heif_compression_HEVC
    raise SystemExit("libheif has no available HEVC decoder")
print("libheif 1.21.1: HEVC decoder available")
heif.heif_deinit()
