#!/usr/bin/env bash
# Cut the Inter faces `ds::FACES` ships (design/02-TYPE.md section 2, the System typeface) from
# the official release, into the same WOFF2 inputs the other faces have; `subset-fonts.sh` then
# turns them into the TTFs Blitz registers, exactly as it does for every other face.
#
# Source: Inter 4.1, https://github.com/rsms/inter/releases/download/v4.1/Inter-4.1.zip
#   SHA-256 9883fdd4a49d4fb66bd8177ba6625ef9a64aa45899767dde3d36aa425756b11e
#   (the release publishes no digest of its own; this is the file as downloaded 2026-09-26).
# Licence: SIL OFL 1.1, the release's LICENSE.txt, copied to assets/fonts/OFL-inter.txt.
#
# The release's variable fonts carry two axes, `opsz` 14..32 and `wght` 100..900. The renderer
# does not set `opsz` from the size (no `font-optical-sizing`), so each optical size is pinned
# into a family of its own, the way the release's static fonts name them:
#   Inter          opsz 14 (text), wght 400..700 upright, 400 italic: the UI and data faces;
#   Inter Display  opsz 32 (display), wght 500..800 upright: headings, names, big numbers
#                  (the same weight range Bricolage Grotesque, the Editorial display face, has).
# The layout features kept are the ones the desktop draws with: kerning and marks (`kern`,
# `mark`, `mkmk`), composition (`ccmp`, `locl`, `calt`), case-sensitive forms (`case`), figures
# (`tnum`, `pnum`, `zero`). The stylistic sets and character variants (`ss01`-`ss08`,
# `cv01`-`cv14`), `aalt`/`salt`, `dlig`, fractions and super/subscripts are dropped: nothing
# asks for them, and the alternates they pull in were 44% of each file (165 KB against 92 KB
# for the latin text face). To ship one, add its tag to FEATURES and run this again.
#
# Run from anywhere; needs `uv` and `curl`. With no argument the zip is downloaded to a
# temporary directory; pass a path to use a copy already on disk.
set -euo pipefail
here="$(cd "$(dirname "$0")" && pwd)"
fonts="$here/../assets/fonts"
sha=9883fdd4a49d4fb66bd8177ba6625ef9a64aa45899767dde3d36aa425756b11e
work="$(mktemp -d)"
trap 'rm -rf "$work"' EXIT

zip="${1:-}"
if [ -z "$zip" ]; then
  zip="$work/Inter-4.1.zip"
  curl -sSL -o "$zip" https://github.com/rsms/inter/releases/download/v4.1/Inter-4.1.zip
fi
echo "$sha  $zip" | sha256sum -c -
unzip -q -o "$zip" InterVariable.ttf InterVariable-Italic.ttf LICENSE.txt -d "$work"
cp "$work/LICENSE.txt" "$fonts/OFL-inter.txt"

LATIN="U+0000-00FF,U+0131,U+0152-0153,U+02BB-02BC,U+02C6,U+02DA,U+02DC,U+0304,U+0308,U+0329,U+2000-206F,U+20AC,U+2122,U+2191,U+2193,U+2212,U+2215,U+FEFF,U+FFFD"
LATIN_EXT="U+0100-02BA,U+02BD-02C5,U+02C7-02CC,U+02CE-02D7,U+02DD-02FF,U+0304,U+0308,U+0329,U+1D00-1DBF,U+1E00-1E9F,U+1EF2-1EFF,U+2020,U+20A0-20AB,U+20AD-20C0,U+2113,U+2C60-2C7F,U+A720-A7FF"

FEATURES="kern,mark,mkmk,ccmp,locl,calt,case,tnum,pnum,zero"

# stem, source file, axis limits
cut() {
  local stem="$1" source="$2"
  shift 2
  uv tool run --from fonttools fonttools varLib.instancer "$work/$source" "$@" \
    --output "$work/$stem.ttf" --quiet
  for subset in latin latin-ext; do
    local range="$LATIN"
    if [ "$subset" = latin-ext ]; then range="$LATIN_EXT"; fi
    uv tool run --from fonttools --with brotli pyftsubset "$work/$stem.ttf" \
      --unicodes="$range" \
      --layout-features="$FEATURES" \
      --name-IDs='*' \
      --name-languages='*' \
      --notdef-outline \
      --glyph-names \
      --flavor=woff2 \
      --output-file="$fonts/$stem-$subset.woff2"
  done
}

cut inter-normal-400-700 InterVariable.ttf opsz=14 wght=400:700
cut inter-italic-400 InterVariable-Italic.ttf opsz=14 wght=400
cut inter-display-normal-500-800 InterVariable.ttf opsz=32 wght=500:800

"$here/subset-fonts.sh" \
  inter-normal-400-700-latin.woff2 inter-normal-400-700-latin-ext.woff2 \
  inter-italic-400-latin.woff2 inter-italic-400-latin-ext.woff2 \
  inter-display-normal-500-800-latin.woff2 inter-display-normal-500-800-latin-ext.woff2
