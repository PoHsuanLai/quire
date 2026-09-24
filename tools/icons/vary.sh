#!/usr/bin/env bash
# Plate a reference-conditioned pass (<subject>-ref.png plus mail-hero.png) and sheet it as one row.
# usage: tools/icons/vary.sh <model> <raw-dir> <plated-dir> <sheet-dir>
set -euo pipefail
model=$1 raw=$2 plated=$3 sheets=$4
declare -A FAMILY=([mail]=blue [files]=amber [terminal]=violet [notes]=green [photos]=red)
bin=${ICONS_BIN:-target/release/icons}
for f in "$raw"/*.png; do
  name=$(basename "$f" .png)
  "$bin" plate --input "$f" --family "${FAMILY[${name%%-*}]}" --out-dir "$plated" --name "$name"
done
# One row: the hero, then each conditioned subject.
for d in "$raw" "$plated"; do
  for f in "$d"/*-hero*.png "$d"/*-ref*.png; do
    b=$(basename "$f"); subj=${b%%-*}; rest=${b#*-}; rest=${rest#hero}; rest=${rest#ref}
    cp "$f" "$d/set-$subj$rest"
  done
done
cols=mail,files,terminal,notes,photos
up=$(echo "$model" | tr a-z A-Z)
"$bin" sheet --dir "$raw" --rows set --cols $cols --kind raw --title "$up REFERENCE PASS RAW - HERO MAIL, REST CONDITIONED ON IT" --out "$sheets/$model-reference-raw.png"
"$bin" sheet --dir "$plated" --rows set --cols $cols --kind plated --title "$up REFERENCE PASS PLATED" --out "$sheets/$model-reference-plated.png"
