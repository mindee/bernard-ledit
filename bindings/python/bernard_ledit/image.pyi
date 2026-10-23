import os
from typing import BinaryIO, final

__all__ = [
    "Image",
    "ImageError",
    "ImageExtractError",
    "compress",
    "extract_from_polygons",
    "guess_format",
]

class ImageError(Exception): ...
class ImageExtractError(Exception): ...

@final
class Image:
    def __new__(cls, data: bytes | BinaryIO) -> Image: ...
    @property
    def size(self) -> tuple[int, int]: ...
    @property
    def format(self) -> str | None: ...
    def crop(self, left: int, top: int, right: int, bottom: int) -> Image: ...
    def encode(
        self, format: str, quality: int = 85, optimize: bool = True
    ) -> bytes: ...
    def resize(self, width: int, height: int) -> Image: ...
    def save(
        self,
        dest: str | os.PathLike[str] | BinaryIO,
        format: str | None = None,
        quality: int = 85,
        optimize: bool = True,
    ) -> None: ...
    def as_bytes(self) -> bytes: ...
    def compress(
        self,
        quality: int = 85,
        max_width: int | None = None,
        max_height: int | None = None,
        optimize: bool = True,
    ) -> bytes | None: ...

def compress(
    data: bytes | BinaryIO,
    quality: int = 85,
    max_width: int | None = None,
    max_height: int | None = None,
    optimize: bool = True,
) -> bytes: ...
def extract_from_polygons(
    data: bytes | BinaryIO,
    page_id: int,
    polygons: list[list[tuple[float, float]]],
    dpi: float = 72.0,
) -> list[Image]: ...
def guess_format(
    data: bytes | BinaryIO,
) -> str: ...
