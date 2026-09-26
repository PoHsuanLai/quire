"""Decode an animated WebP into composited frames with their durations, find the rest pose,
and resample the loop to a fixed step.
"""

from __future__ import annotations

import struct
from dataclasses import dataclass
from pathlib import Path

from PIL import Image, ImageChops, ImageStat


@dataclass(frozen=True)
class Frame:
    image: Image.Image
    ms: int


def durations(path: Path) -> list[int]:
    """Each ANMF chunk's duration in ms (Pillow does not report per-frame WebP durations)."""
    data = path.read_bytes()
    out: list[int] = []
    at = 12
    while at + 8 <= len(data):
        tag = data[at : at + 4]
        (size,) = struct.unpack("<I", data[at + 4 : at + 8])
        if tag == b"ANMF":
            payload = data[at + 8 : at + 8 + 16]
            out.append(int.from_bytes(payload[12:15], "little"))
        at += 8 + size + (size & 1)
    return out


def decode(path: Path) -> list[Frame]:
    """Every frame, fully composited, as RGBA at the source size."""
    times = durations(path)
    with Image.open(path) as image:
        count = getattr(image, "n_frames", 1)
        if len(times) != count:
            raise ValueError(f"{path}: {count} frames but {len(times)} durations")
        frames = []
        for index in range(count):
            image.seek(index)
            frames.append(Frame(image.convert("RGBA").copy(), max(times[index], 10)))
    return frames


def _normalised(image: Image.Image) -> Image.Image:
    """The drawing cropped to its own bounds, on white, at 64 px: the still and the animation
    frames are drawn at different scales, so they are compared by shape, not by position."""
    image = image.convert("RGBA")
    box = image.getchannel("A").point(lambda a: 255 if a > 32 else 0).getbbox()
    ground = Image.new("RGBA", image.size, (255, 255, 255, 255))
    ground.alpha_composite(image)
    return ground.crop(box).convert("RGB").resize((64, 64), Image.BILINEAR)


def _distance(a: Image.Image, b: Image.Image) -> float:
    return sum(ImageStat.Stat(ImageChops.difference(_normalised(a), _normalised(b))).mean)


def rest_index(frames: list[Frame], still: Image.Image) -> int:
    """The frame closest to the still emoji: the pose the animation rests in. Ties go to the
    longest-held frame, then the earliest."""
    scored = [(_distance(f.image, still), -f.ms, i) for i, f in enumerate(frames)]
    return min(scored)[2]


def rotated(frames: list[Frame], start: int) -> list[Frame]:
    """The loop begun at `start` (the loops are seamless, so this is the same animation)."""
    return frames[start:] + frames[:start]


def resampled(frames: list[Frame], step_ms: int) -> list[Frame]:
    """The loop sampled every `step_ms`, a run of identical picks merged into one held frame.
    Frame 0 stays frame 0, and the total length is kept to within one step."""
    total = sum(f.ms for f in frames)
    starts = []
    at = 0
    for f in frames:
        starts.append(at)
        at += f.ms
    picks: list[int] = []
    t = 0
    while t < total:
        index = max(i for i, s in enumerate(starts) if s <= t)
        picks.append(index)
        t += step_ms
    out: list[Frame] = []
    for index in picks:
        if out and out[-1].image is frames[index].image:
            out[-1] = Frame(out[-1].image, out[-1].ms + step_ms)
        else:
            out.append(Frame(frames[index].image, step_ms))
    return out
