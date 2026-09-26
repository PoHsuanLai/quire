"""emojitool build: fetch the curated set and write the sheets and manifest quire ships.

    cd tools/emoji && uv run emojitool build --out ../../crates/ds/assets/emoji
"""

from __future__ import annotations

import argparse
import json
from pathlib import Path

from PIL import Image

from . import curated, fetch, frames, sheet

HERE = Path(__file__).resolve().parent.parent


def one(args: argparse.Namespace, pick: curated.Pick) -> dict:
    webp = fetch.cached(args.cache, pick.codepoint, "animation")
    still = fetch.cached(args.cache, pick.codepoint, "still")
    source = frames.decode(webp)
    with Image.open(still) as image:
        start = pick.rest if pick.rest is not None else frames.rest_index(source, image)
    loop = frames.resampled(frames.rotated(source, start), args.step)
    columns, rows = sheet.grid(len(loop))
    sizes = {}
    for size in args.sizes:
        png = sheet.encoded(sheet.packed(loop, size), args.colours)
        name = f"{pick.slug}-{size}.png"
        (args.out / name).write_bytes(png)
        sizes[str(size)] = {"file": name, "bytes": len(png)}
    print(f"{pick.slug}: {len(source)} -> {len(loop)} frames, rest {start}, "
          + ", ".join(f"{s}px {v['bytes']} B" for s, v in sizes.items()), flush=True)
    return {
        "slug": pick.slug,
        "codepoint": pick.codepoint,
        "name": pick.name,
        "columns": columns,
        "rows": rows,
        "durations_ms": [f.ms for f in loop],
        "sizes": sizes,
        "source": {
            "url": fetch.url(pick.codepoint, "animation"),
            "sha256": fetch.sha256(webp),
            "frames": len(source),
            "rest_frame": start,
        },
    }


def cmd_build(args: argparse.Namespace) -> None:
    args.out.mkdir(parents=True, exist_ok=True)
    picks = [p for p in curated.SET if not args.only or p.slug in args.only]
    entries = [one(args, pick) for pick in picks]
    total = sum(v["bytes"] for e in entries for v in e["sizes"].values())
    manifest = {
        "source": "Noto Animated Emoji, https://googlefonts.github.io/noto-emoji-animation/",
        "licence": "CC-BY-4.0",
        "attribution": "Animated emoji from Noto Animated Emoji by Google, CC BY 4.0 "
                       "(https://creativecommons.org/licenses/by/4.0/). Resampled, cropped to "
                       "frames and packed into sprite sheets by quire's tools/emoji.",
        "step_ms": args.step,
        "colours": args.colours,
        "sizes": args.sizes,
        "emoji": entries,
    }
    if not args.only:
        (args.out / "manifest.json").write_text(json.dumps(manifest, indent=1) + "\n")
    print(f"{len(entries)} emoji, {total} bytes of sheets")


def main() -> None:
    parser = argparse.ArgumentParser(prog="emojitool")
    sub = parser.add_subparsers(required=True)
    build = sub.add_parser("build", help="fetch, resample, pack; write sheets and manifest.json")
    build.add_argument("--out", type=Path, required=True)
    build.add_argument("--cache", type=Path, default=HERE / ".cache")
    build.add_argument("--step", type=int, default=80, help="ms between sampled frames (80: 12.5 fps keeps the set under 8 MB)")
    build.add_argument("--sizes", type=int, nargs="+", default=[128, 256])
    build.add_argument("--colours", type=int, default=256, help="palette size; 0 keeps RGBA")
    build.add_argument("--only", nargs="*", default=[], help="a subset, for trials (no manifest)")
    build.set_defaults(run=cmd_build)
    args = parser.parse_args()
    args.run(args)
