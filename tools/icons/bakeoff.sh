#!/usr/bin/env bash
# Plate every raw bake-off render of one model and build its two contact sheets.
# usage: tools/icons/bakeoff.sh <model> <raw-dir> <plated-dir> <sheet-dir>
# Subject -> plate family is the bake-off's proposal (docs/icons-bakeoff.md).
set -euo pipefail
model=$1 raw=$2 plated=$3 sheets=$4
declare -A FAMILY=([mail]=blue [files]=amber [terminal]=violet [notes]=green [photos]=red)
rows=mail,files,terminal,notes,photos
cols=${COLS:-s11,s22,s33,s44}
bin=${ICONS_BIN:-target/release/icons}
for f in "$raw"/*.png; do
  name=$(basename "$f" .png)
  subject=${name%%-*}
  "$bin" plate --input "$f" --family "${FAMILY[$subject]}" --out-dir "$plated" --name "$name"
done
up=$(echo "$model" | tr a-z A-Z)
"$bin" sheet --dir "$raw" --rows $rows --cols "$cols" --kind raw --title "$up RAW 1024 - SEEDS ${cols//,/ }" --out "$sheets/$model-raw.png"
"$bin" sheet --dir "$plated" --rows $rows --cols "$cols" --kind plated --title "$up PLATED - 16 32 48 ON LIGHT AND DARK UNDER EACH" --out "$sheets/$model-plated.png"
