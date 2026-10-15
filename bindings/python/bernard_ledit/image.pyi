import os
from typing import BinaryIO

class ImageError(Exception): ...
class ImageExtractError(Exception): ...

class Image:
    def __init__(self, data: bytes | BinaryIO) -> None: ...
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
def extract_polygons(
    data: bytes | BinaryIO,
    page_id: int,
    polygons: list[list[tuple[float, float]]],
    dpi: float = 72.0,
) -> list[Image]: ...
