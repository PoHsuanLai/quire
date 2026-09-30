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

# ds-core is plain data and maths, so sill's pure crates can use it: not even dioxus. ds-style names
# dioxus only through its `dioxus` feature (scope, scale, task, busy, `Glyph`), which ds-settings
# leaves off, so a crate that only reads settings links no Dioxus; its `dioxus` feature is for the
# environment that provides them. ds is renderer-free and effect-free; ds-settings does I/O but
# never renders, and takes a Spawner instead of naming a runtime.
# ds-blitz reaches D-Bus only through its opt-in `print` feature, so an app that never prints
# builds no D-Bus client for it; anyrender_pdfrum stays Blitz-free so it can go upstream.
RULES=(
  "ds-core: zbus notify tokio winit dioxus blitz blitz-dom blitz-paint blitz-traits blitz-html blitz-net blitz-shell stylo_taffy dioxus-native dioxus-native-dom anyrender anyrender_vello anyrender_vello_cpu anyrender_vello_hybrid anyrender_skia anyrender_svg anyrender_pdfrum pdfrum-edit"
  "ds-style: zbus notify tokio winit blitz blitz-dom blitz-paint blitz-traits blitz-html blitz-net blitz-shell stylo_taffy dioxus-native dioxus-native-dom anyrender anyrender_vello anyrender_vello_cpu anyrender_vello_hybrid anyrender_skia anyrender_svg anyrender_pdfrum pdfrum-edit"
  "ds-motion: zbus notify tokio winit blitz blitz-dom blitz-paint blitz-traits blitz-html blitz-net blitz-shell stylo_taffy dioxus-native dioxus-native-dom anyrender anyrender_vello anyrender_vello_cpu anyrender_vello_hybrid anyrender_skia anyrender_svg anyrender_pdfrum pdfrum-edit"
  "ds-lint: zbus notify tokio winit dioxus blitz blitz-dom blitz-paint blitz-traits blitz-html blitz-net blitz-shell stylo_taffy dioxus-native dioxus-native-dom anyrender anyrender_vello anyrender_vello_cpu anyrender_vello_hybrid anyrender_skia anyrender_svg anyrender_pdfrum pdfrum-edit"
  "ds-shell: zbus notify tokio winit blitz blitz-dom blitz-paint blitz-traits blitz-html blitz-net blitz-shell stylo_taffy dioxus-native dioxus-native-dom anyrender anyrender_vello anyrender_vello_cpu anyrender_vello_hybrid anyrender_skia anyrender_svg anyrender_pdfrum pdfrum-edit"
  "ds: zbus notify tokio winit blitz blitz-dom blitz-paint blitz-traits blitz-html blitz-net blitz-shell stylo_taffy dioxus-native dioxus-native-dom anyrender anyrender_vello anyrender_vello_cpu anyrender_vello_hybrid anyrender_skia anyrender_svg anyrender_pdfrum pdfrum-edit"
  "ds-settings: dioxus tokio blitz blitz-dom blitz-paint blitz-traits blitz-html blitz-net blitz-shell stylo_taffy dioxus-native dioxus-native-dom anyrender anyrender_vello anyrender_vello_cpu anyrender_vello_hybrid anyrender_skia anyrender_svg anyrender_pdfrum pdfrum-edit"
  "ds-blitz: zbus memfd"
  "ds-core-derive: dioxus zbus tokio"
  "ds-settings-derive: dioxus zbus tokio"
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

# The allowed edges between our own crates (ARCHITECTURE.md section 1): each crate's direct normal
# and build dependencies that live in this workspace, and nothing else. A dependency on a crate
# not listed here is a leak; so is one the crate no longer has, so the table stays exact.
EDGES=(
  "ds-core-derive:"
  "ds-settings-derive:"
  "ds-core: ds-core-derive"
  "ds-style: ds-core ds-core-derive"
  "ds-motion: ds-core ds-style"
  "ds-lint: ds-core ds-style"
  "ds: ds-core ds-motion ds-style"
  "ds-shell: ds ds-core ds-motion ds-style"
  "ds-settings: ds-core ds-style ds-settings-derive"
  "ds-blitz: ds anyrender_pdfrum"
  "ds-gallery: ds ds-core ds-lint ds-blitz ds-settings ds-shell"
  "icons: ds ds-settings"
  "anyrender_pdfrum:"
)
for edge in "${EDGES[@]}"; do
  crate="${edge%%:*}"
  read -r -a allowed <<<"${edge#*:}"
  found=$(cargo tree -p "$crate" --depth 1 -e normal,build --prefix none --all-features 2>/dev/null \
    | grep '(/' | awk '{print $1}' | grep -vx "$crate" | sort -u | tr '\n' ' ')
  want=$(printf '%s\n' "${allowed[@]}" | grep . | sort -u | tr '\n' ' ')
  if [ "$found" != "$want" ]; then
    echo "EDGE: $crate depends on [${found% }], the table allows [${want% }]"
    fail=1
  else
    echo "edges hold: $crate depends on [${found% }]"
  fi
done

# The layers inside ds, the one crate that still has any: the host seams, the hooks over them, the
# overlay stack, the root, the components and assembly on top. A file in a layer may name
# (`crate::<module>`) only its own layer and the ones below it; mail's own components
# (components::app) are named by nothing but themselves and assembly. Doc links count. The layers
# that became crates (ds-core, ds-style, ds-motion, ds-lint, ds-shell) are kept by EDGES above and
# by cargo itself: a `pub(crate)` item cannot be named across a crate boundary.
DS=crates/ds/src
LAYERS=(
  "host: focus edit file_drop spell window stack root components assembly"
  "focus edit file_drop spell window: stack root components assembly"
  "stack: root components assembly"
  "root: components assembly"
  "components: assembly components::app"
)
layered=0
for rule in "${LAYERS[@]}"; do
  read -r -a dirs <<<"${rule%%:*}"
  read -r -a above <<<"${rule#*:}"
  pattern="crate::($(IFS='|'; echo "${above[*]}"))\\b"
  for dir in "${dirs[@]}"; do
    [ -d "$DS/$dir" ] || { echo "ERROR: $DS/$dir is missing; the layer was not checked"; fail=1; continue; }
    if [ "$dir" = components ]; then
      hits=$(grep -rnE "$pattern" "$DS/$dir" --exclude-dir=app || true)
    else
      hits=$(grep -rnE "$pattern" "$DS/$dir" | grep -v "^$DS/components/app/" || true)
    fi
    if [ -n "$hits" ]; then
      echo "LAYER: $dir names a layer above it:"
      echo "$hits" | head -20
      layered=1
      fail=1
    fi
  done
done

# The component groups, lowest first: a group names only itself and the groups below it.
COMPONENT_GROUPS=(content controls overlays lists fields menus editor chrome app)
for i in "${!COMPONENT_GROUPS[@]}"; do
  group="${COMPONENT_GROUPS[$i]}"
  above=("${COMPONENT_GROUPS[@]:$((i + 1))}")
  [ "${#above[@]}" -gt 0 ] || continue
  [ -d "$DS/components/$group" ] || { echo "ERROR: $DS/components/$group is missing; the group was not checked"; fail=1; continue; }
  pattern="crate::components::($(IFS='|'; echo "${above[*]}"))\\b"
  hits=$(grep -rnE "$pattern" "$DS/components/$group" || true)
  if [ -n "$hits" ]; then
    echo "GROUP: components::$group names a group above it:"
    echo "$hits" | head -20
    layered=1
    fail=1
  fi
done
if [ "$layered" -eq 0 ]; then
  echo "layers hold: host < hooks < stack < root < components < assembly; component groups in order"
fi

exit "$fail"
