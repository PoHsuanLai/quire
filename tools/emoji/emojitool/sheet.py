"""Pack frames into one sprite sheet PNG: a grid `COLUMNS` wide, read left to right, top to
bottom. A grid rather than one long strip keeps the image inside the renderer's 4096 px
texture limit at 256 px frames."""

from __future__ import annotations

import io
import math

from PIL import Image

from .frames import Frame

COLUMNS = 8


def grid(count: int) -> tuple[int, int]:
    columns = min(COLUMNS, count)
    return columns, math.ceil(count / columns)


def packed(frames: list[Frame], size: int) -> Image.Image:
    columns, rows = grid(len(frames))
    sheet = Image.new("RGBA", (columns * size, rows * size), (0, 0, 0, 0))
    for index, frame in enumerate(frames):
        cell = frame.image.resize((size, size), Image.LANCZOS)
        sheet.paste(cell, ((index % columns) * size, (index // columns) * size))
    return sheet


def encoded(sheet: Image.Image, colours: int) -> bytes:
    """The sheet as an optimised PNG; with `colours`, quantised to one shared palette first
    (alpha included), which is what keeps the set small."""
    image = sheet
    if colours:
        image = sheet.quantize(colours, method=Image.Quantize.FASTOCTREE, dither=Image.Dither.NONE)
    out = io.BytesIO()
    image.save(out, "PNG", optimize=True)
    return out.getvalue()
