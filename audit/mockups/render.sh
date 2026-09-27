#!/usr/bin/env bash
# Render every mockup page to PNG at 1x and 2x with headless Chrome, cropped to its content.
set -euo pipefail
here="$(cd "$(dirname "$0")" && pwd)"
profile="$here/.chrome-profile"
for page in "$here"/*.html; do
  name="$(basename "$page" .html)"
  for scale in 1 2; do
    out="$here/$name-${scale}x.png"
    google-chrome --headless=new --no-sandbox --disable-gpu --hide-scrollbars --user-data-dir="$profile" \
      --force-device-scale-factor="$scale" --window-size=1440,1400 --screenshot="$out" "file://$page" >/dev/null 2>&1
    python3 - "$out" <<'PY'
import sys
from PIL import Image, ImageChops
path = sys.argv[1]
im = Image.open(path).convert("RGB")
bg = Image.new("RGB", im.size, im.getpixel((2, 2)))
box = ImageChops.difference(im, bg).getbbox()
if box:
    pad = 16
    im.crop((max(0, box[0] - pad), max(0, box[1] - pad), min(im.width, box[2] + pad), min(im.height, box[3] + pad))).save(path)
PY
    echo "$out"
  done
done
rm -rf "$profile"
