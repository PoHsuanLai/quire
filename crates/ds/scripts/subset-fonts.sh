#!/usr/bin/env bash
# Turn the vendored WOFF2 faces into the subset TTFs `ds::FACES` ships (design/02-TYPE.md
# section 2; FINDINGS F14): fontique, which Blitz registers faces through, reads sfnt, not WOFF2.
#
# Each `<stem>-<subset>.woff2` becomes `<stem>-<subset>.ttf` with the same unicode range
# (Google Fonts' own `latin` / `latin-ext` split, the ranges build.rs writes for the webview),
# every layout feature, every name record and the variable axes kept. The WOFF2 files stay:
# they are this script's input and the webview's `@font-face` source.
#
# Run from anywhere; needs `uv` (fonttools and brotli are fetched into a throwaway tool env):
#   crates/ds/scripts/subset-fonts.sh
set -euo pipefail
cd "$(dirname "$0")/../assets/fonts"

LATIN="U+0000-00FF,U+0131,U+0152-0153,U+02BB-02BC,U+02C6,U+02DA,U+02DC,U+0304,U+0308,U+0329,U+2000-206F,U+20AC,U+2122,U+2191,U+2193,U+2212,U+2215,U+FEFF,U+FFFD"
LATIN_EXT="U+0100-02BA,U+02BD-02C5,U+02C7-02CC,U+02CE-02D7,U+02DD-02FF,U+0304,U+0308,U+0329,U+1D00-1DBF,U+1E00-1E9F,U+1EF2-1EFF,U+2020,U+20A0-20AB,U+20AD-20C0,U+2113,U+2C60-2C7F,U+A720-A7FF"

for woff2 in *.woff2; do
  stem="${woff2%.woff2}"
  case "$stem" in
    *-latin-ext) range="$LATIN_EXT" ;;
    *-latin) range="$LATIN" ;;
    *) echo "unknown subset: $woff2" >&2; exit 1 ;;
  esac
  uv tool run --from fonttools --with brotli pyftsubset "$woff2" \
    --unicodes="$range" \
    --layout-features='*' \
    --name-IDs='*' \
    --name-languages='*' \
    --notdef-outline \
    --glyph-names \
    --output-file="$stem.ttf"
  case "$stem" in
    bricolage-grotesque-*) family="Bricolage Grotesque" ;;
    karla-*) family="Karla" ;;
    space-mono-*) family="Space Mono" ;;
    *) echo "unknown family: $stem" >&2; exit 1 ;;
  esac
  # Google's Bricolage names its family after the default instance ("Bricolage Grotesque 96pt
  # ExtraBold"). The renderer registers a face under its name table, and `--font-display`
  # asks for "Bricolage Grotesque", so the family and typographic family are set to the name
  # the stylesheet uses.
  uv tool run --from fonttools python - "$stem.ttf" "$family" <<'PY'
import sys
from fontTools.ttLib import TTFont
path, family = sys.argv[1], sys.argv[2]
font = TTFont(path)
names = font["name"]
for record in list(names.names):
    if record.nameID in (1, 16):
        names.setName(family, record.nameID, record.platformID, record.platEncID, record.langID)
names.setName(family, 16, 3, 1, 0x409)
font.save(path)
PY
  echo "$stem.ttf ($family)"
done
