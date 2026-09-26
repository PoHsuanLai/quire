"""Download the source files once into a cache; every later run reads the cache.

The source is Google's Noto Animated Emoji (CC BY 4.0), served from fonts.gstatic.com as the
project's own site links it. Nothing here runs at cargo build time.
"""

from __future__ import annotations

import hashlib
import urllib.request
from pathlib import Path

BASE = "https://fonts.gstatic.com/s/e/notoemoji/latest"

# What each file is for: the animation itself, and the still emoji used only to find which
# animation frame is its rest pose (never shipped).
FILES = {"animation": "512.webp", "still": "512.png"}


def url(codepoint: str, kind: str) -> str:
    return f"{BASE}/{codepoint}/{FILES[kind]}"


def cached(cache: Path, codepoint: str, kind: str) -> Path:
    """The cached file, downloading it first when absent."""
    path = cache / codepoint / FILES[kind]
    if not path.exists():
        path.parent.mkdir(parents=True, exist_ok=True)
        request = urllib.request.Request(url(codepoint, kind), headers={"User-Agent": "quire-emojitool"})
        with urllib.request.urlopen(request, timeout=60) as response:
            data = response.read()
        path.write_bytes(data)
    return path


def sha256(path: Path) -> str:
    return hashlib.sha256(path.read_bytes()).hexdigest()
