#!/usr/bin/env bash
# Round three (design/08-ICONS.md 2.9): plate the emboss renders of one model and mode, and build
# the contact sheet (256 tile, 16/32/48 strip under each).
# usage: tools/icons/round3.sh <sheet-name> <raw-dir> <plated-dir> <ground|tile> <sheet-dir>
set -euo pipefail
name=$1 raw=$2 plated=$3 mode=$4 sheets=$5
bin=${ICONS_BIN:-target/release/icons}
rows=${ROWS:-mail,files,terminal,notes,photos}
cols=${COLS:-s11,s22,s33,s44}
for f in "$raw"/*.png; do
  "$bin" face --input "$f" --mode "$mode" --out-dir "$plated" --name "$(basename "$f" .png)"
done
up=$(echo "$name" | tr a-z- A-Z\ )
"$bin" sheet --dir "$plated" --rows "$rows" --cols "$cols" --kind plated \
  --title "$up - 16 32 48 ON LIGHT AND DARK UNDER EACH" --out "$sheets/$name.png"
