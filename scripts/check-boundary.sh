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

# ds-core is plain data and maths, so sill's pure crates can use it: not even dioxus. ds-style and
# ds-motion name dioxus only through their `dioxus` feature (ds-style: scope, scale, task, busy,
# `Glyph`; ds-motion: the hooks, timers and event conversions), which neither enables by default,
# so a compositor or a crate that only reads settings links no Dioxus; ds-settings has the same
# feature for the environment that provides them. ds-behaviour holds the input machines a
# compositor runs too, so like ds-core it reaches no Dioxus and no renderer. ds is renderer-free
# and effect-free; ds-settings does I/O but never renders, and takes a Spawner instead of naming a
# runtime. ds-helpers (missing helpers: probe PATH, ask PackageKit) is I/O too: zbus only, no
# renderer, no runtime, no other quire crate.
# ds-blitz reaches D-Bus only through its opt-in `print` and `menus` features, so an app that
# never prints or exports a menu builds no D-Bus client for it, and pdfrum only through `pdf`; ds-harness inherits both rules
# (it turns on `pdf` only for its own `pdf` feature). pdfrum-anyrender (a git dependency
# from the pdfrum repo) is not an in-tree edge, so it is not listed there. wgpu is a renderer
# dependency like anyrender: only ds-blitz (the texture layer's device) and ds-harness (the
# hybrid painter) name it; every other crate stays device-free.
RULES=(
  "ds-core: zbus notify tokio winit dioxus blitz blitz-dom blitz-paint blitz-traits blitz-html blitz-net blitz-shell stylo_taffy dioxus-native dioxus-native-dom anyrender anyrender_vello anyrender_vello_cpu anyrender_vello_hybrid anyrender_skia anyrender_svg pdfrum-anyrender pdfrum-edit wgpu wgpu_context"
  "ds-behaviour: zbus notify tokio winit dioxus blitz blitz-dom blitz-paint blitz-traits blitz-html blitz-net blitz-shell stylo_taffy dioxus-native dioxus-native-dom anyrender anyrender_vello anyrender_vello_cpu anyrender_vello_hybrid anyrender_skia anyrender_svg pdfrum-anyrender pdfrum-edit wgpu wgpu_context"
  "ds-intents: zbus notify tokio winit dioxus blitz blitz-dom blitz-paint blitz-traits blitz-html blitz-net blitz-shell stylo_taffy dioxus-native dioxus-native-dom anyrender anyrender_vello anyrender_vello_cpu anyrender_vello_hybrid anyrender_skia anyrender_svg pdfrum-anyrender pdfrum-edit wgpu wgpu_context"
  "ds-style: dioxus zbus notify tokio winit blitz blitz-dom blitz-paint blitz-traits blitz-html blitz-net blitz-shell stylo_taffy dioxus-native dioxus-native-dom anyrender anyrender_vello anyrender_vello_cpu anyrender_vello_hybrid anyrender_skia anyrender_svg pdfrum-anyrender pdfrum-edit wgpu wgpu_context"
  "ds-motion: dioxus zbus notify tokio winit blitz blitz-dom blitz-paint blitz-traits blitz-html blitz-net blitz-shell stylo_taffy dioxus-native dioxus-native-dom anyrender anyrender_vello anyrender_vello_cpu anyrender_vello_hybrid anyrender_skia anyrender_svg pdfrum-anyrender pdfrum-edit wgpu wgpu_context"
  "ds-lint: zbus notify tokio winit dioxus blitz blitz-dom blitz-paint blitz-traits blitz-html blitz-net blitz-shell stylo_taffy dioxus-native dioxus-native-dom anyrender anyrender_vello anyrender_vello_cpu anyrender_vello_hybrid anyrender_skia anyrender_svg pdfrum-anyrender pdfrum-edit wgpu wgpu_context"
  "ds-shell: zbus notify tokio winit blitz blitz-dom blitz-paint blitz-traits blitz-html blitz-net blitz-shell stylo_taffy dioxus-native dioxus-native-dom anyrender anyrender_vello anyrender_vello_cpu anyrender_vello_hybrid anyrender_skia anyrender_svg pdfrum-anyrender pdfrum-edit wgpu wgpu_context"
  "ds: zbus notify tokio winit blitz blitz-dom blitz-paint blitz-traits blitz-html blitz-net blitz-shell stylo_taffy dioxus-native dioxus-native-dom anyrender anyrender_vello anyrender_vello_cpu anyrender_vello_hybrid anyrender_skia anyrender_svg pdfrum-anyrender pdfrum-edit wgpu wgpu_context"
  "ds-settings: dioxus tokio blitz blitz-dom blitz-paint blitz-traits blitz-html blitz-net blitz-shell stylo_taffy dioxus-native dioxus-native-dom anyrender anyrender_vello anyrender_vello_cpu anyrender_vello_hybrid anyrender_skia anyrender_svg pdfrum-anyrender pdfrum-edit wgpu wgpu_context"
  "ds-helpers: dioxus tokio blitz blitz-dom blitz-paint blitz-traits blitz-html blitz-net blitz-shell stylo_taffy dioxus-native dioxus-native-dom anyrender anyrender_vello anyrender_vello_cpu anyrender_vello_hybrid anyrender_skia anyrender_svg pdfrum-anyrender pdfrum-edit wgpu wgpu_context"
  "ds-blitz: zbus memfd pdfrum-anyrender pdfrum pdfrum-edit"
  "ds-harness: zbus memfd pdfrum-anyrender pdfrum pdfrum-edit"
  "ds-core-derive: dioxus zbus tokio"
  "ds-settings-derive: dioxus zbus tokio"
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
  "ds-intents: ds-core"
  "ds-behaviour: ds-core"
  "ds: ds-core ds-intents ds-motion ds-style"
  "ds-shell: ds ds-core ds-motion ds-style"
  "ds-settings: ds-behaviour ds-core ds-style ds-settings-derive"
  "ds-helpers:"
  "ds-blitz: blitz-kit ds"
  "ds-harness: blitz-kit ds ds-blitz ds-core"
  "ds-gallery: ds ds-core ds-harness ds-lint ds-blitz ds-settings ds-shell"
  "ds-conformance:"
  "icons: ds ds-style ds-settings"
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

# ds-conformance is test-only: no normal or build edge at all (the EDGES row above), and its
# dev-dependencies are the crates it tests through, never the lower crates directly.
DEV_EDGES=(
  "ds-conformance: ds ds-shell ds-lint ds-settings ds-blitz ds-harness"
)
for edge in "${DEV_EDGES[@]}"; do
  crate="${edge%%:*}"
  read -r -a allowed <<<"${edge#*:}"
  found=$(cargo tree -p "$crate" --depth 1 -e dev --prefix none --all-features 2>/dev/null \
    | grep '(/' | awk '{print $1}' | grep -vx "$crate" | sort -u)
  outside=$(comm -23 <(printf '%s\n' "$found") <(printf '%s\n' "${allowed[@]}" | sort -u))
  if [ -n "$outside" ]; then
    echo "DEV EDGE: $crate dev-depends on [$(echo $outside)], the table allows [${allowed[*]}]"
    fail=1
  else
    echo "dev edges hold: $crate dev-depends on [$(echo $found)]"
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
  "host: focus edit file_drop machine spell window stack root components assembly"
  "focus edit file_drop machine spell window: stack root components assembly"
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
COMPONENT_GROUPS=(content controls overlays lists fields menus editor chrome companion app)
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

# The crate roots of ds and ds-shell hold declarations, `prelude` and the curated roots (the
# stylesheet assembly) only: every other name has one path, its home module or the prelude. A new
# `pub use` at either root introduces a name outside the allowed set below and fails here.
# The three facades (`pub use ds_core as base;`, `ds_style as style`, `ds_motion as motion`) are the
# only crate re-exports.
ROOT_ALLOWED_DS="pub use crate assembly selectors kit KIT kits stylesheet component_sheets ds_core base ds_style style ds_motion motion as"
ROOT_ALLOWED_DS_SHELL="pub use crate kit KIT kits stylesheet component_sheets"
for root in "ds:$ROOT_ALLOWED_DS" "ds-shell:$ROOT_ALLOWED_DS_SHELL"; do
  crate="${root%%:*}"
  read -r -a allowed <<<"${root#*:}"
  lib="crates/$crate/src/lib.rs"
  # Every `pub use` statement, comments removed, reduced to its identifier tokens.
  tokens=$(sed 's://.*$::' "$lib" | tr '\n' ' ' | grep -o 'pub use[^;]*;' | grep -o '[A-Za-z_][A-Za-z_0-9]*\|\*' | sort -u)
  outside=$(comm -23 <(printf '%s\n' "$tokens") <(printf '%s\n' "${allowed[@]}" | sort -u))
  if [ -n "$outside" ]; then
    echo "ROOT: $lib re-exports [$(echo $outside)] at the crate root; put the name in the prelude or reach it by its home path"
    fail=1
  else
    echo "root holds: $lib re-exports only the curated roots"
  fi
done

exit "$fail"
