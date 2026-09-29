#!/usr/bin/env bash
# Crate boundaries, mechanically enforced.
#
# `cargo tree -i <dep>` exits 101 when the dependency is absent, which is precisely the state
# we want. Checking the exit status would therefore fail whenever the boundary holds, so we
# check for OUTPUT instead: any line naming the dependency is a leak.
#
# Each rule is "<crate>: <forbidden deps...>". Forbidden names are exact package names; a
# family such as blitz-* is spelled out member by member.
set -uo pipefail
cd "$(dirname "$0")/.."

# ds is renderer-free and effect-free; ds-settings does I/O but never renders.
# ds-native reaches D-Bus only through its opt-in `print` feature, so an app that never prints
# builds no D-Bus client for it; anyrender_pdfrum stays Blitz-free so it can go upstream.
RULES=(
  "ds: zbus notify tokio winit blitz blitz-dom blitz-paint blitz-traits blitz-html blitz-net blitz-shell stylo_taffy dioxus-native dioxus-native-dom anyrender anyrender_vello anyrender_vello_cpu anyrender_vello_hybrid anyrender_skia anyrender_svg anyrender_pdfrum pdfrum-edit"
  "ds-settings: blitz blitz-dom blitz-paint blitz-traits blitz-html blitz-net blitz-shell stylo_taffy dioxus-native dioxus-native-dom anyrender anyrender_vello anyrender_vello_cpu anyrender_vello_hybrid anyrender_skia anyrender_svg anyrender_pdfrum pdfrum-edit"
  "ds-native: zbus memfd"
  "anyrender_pdfrum: blitz blitz-dom blitz-paint blitz-traits blitz-html parley stylo_taffy dioxus dioxus-native"
)
fail=0

for rule in "${RULES[@]}"; do
  crate="${rule%%:*}"
  read -r -a forbidden <<<"${rule#*:}"
  # A crate that cargo cannot find would make every check below pass vacuously.
  if ! cargo tree -p "$crate" --depth 0 >/dev/null 2>&1; then
    echo "ERROR: cargo tree cannot resolve $crate; the boundary was not checked"
    fail=1
    continue
  fi
  leaked=0
  for dep in "${forbidden[@]}"; do
    if cargo tree -p "$crate" -i "$dep" -e normal,build 2>/dev/null | grep -q .; then
      echo "LEAK: $crate depends on $dep"
      cargo tree -p "$crate" -i "$dep" -e normal,build 2>/dev/null | head -20
      leaked=1
      fail=1
    fi
  done
  if [ "$leaked" -eq 0 ]; then
    echo "boundary holds: $crate reaches none of ${forbidden[*]}"
  fi
done

# Layers inside ds, the crates it will split into (core, style, motion, lint, ds, shell), with
# assembly on top. A file in a layer may name (`crate::<module>`) only its own layer and the
# ones below it; mail's own components (components::app) are named by nothing but themselves
# and the layers above the components. Doc links count: they would break at the split too.
DS=crates/ds/src
LAYERS=(
  "core: style motion lint host focus edit file_drop spell window overlay root components shell assembly"
  "style: motion lint host focus edit file_drop spell window overlay root components shell assembly"
  "motion: lint host focus edit file_drop spell window overlay root components shell assembly"
  "lint: host focus edit file_drop spell window overlay root components shell assembly"
  "host focus edit file_drop spell window overlay root components: lint shell assembly components::app"
  "shell: lint assembly"
)
layered=0
for rule in "${LAYERS[@]}"; do
  read -r -a dirs <<<"${rule%%:*}"
  read -r -a above <<<"${rule#*:}"
  pattern="crate::($(IFS='|'; echo "${above[*]}"))\\b"
  for dir in "${dirs[@]}"; do
    [ -d "$DS/$dir" ] || { echo "ERROR: $DS/$dir is missing; the layer was not checked"; fail=1; continue; }
    hits=$(grep -rnE "$pattern" "$DS/$dir" | grep -v "^$DS/components/app/" || true)
    if [ "$dir" = components ]; then
      hits=$(grep -rnE "$pattern" "$DS/$dir" --exclude-dir=app || true)
    fi
    if [ -n "$hits" ]; then
      echo "LAYER: $dir names a layer above it:"
      echo "$hits" | head -20
      layered=1
      fail=1
    fi
  done
done
if [ "$layered" -eq 0 ]; then
  echo "layers hold: core < style < motion < lint, ds < shell < assembly"
fi

exit "$fail"
