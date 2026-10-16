#!/usr/bin/env bash
# A program built the way an app is (the consumer example, normal and build edges only) must not
# switch on the features that exist for tests and scenarios: `ds-blitz/debug-probe` writes every
# control's text to files, and `testing` opens seams that carry no stability promise. Cargo unifies
# features across a build, so one dependent that enables either would ship it in every release
# binary. Run from anywhere; exit status 1 names the offender.
set -euo pipefail
cd "$(dirname "$0")/.."

manifest=examples/consumer/Cargo.toml
fail=0
for crate in ds-blitz ds-core; do
  # `-i` inverts the tree to the crate, `-e features` lists the feature nodes that are on.
  on=$(cargo tree --manifest-path "$manifest" -e normal,build,features -i "$crate" --prefix none 2>/dev/null || true)
  for feature in debug-probe testing; do
    if grep -q "$crate feature \"$feature\"" <<<"$on"; then
      echo "LEAK: $crate feature \"$feature\" is on in a normal build of the consumer"
      fail=1
    fi
  done
done
if [ "$fail" -eq 0 ]; then
  echo "release features hold: debug-probe and testing are off for a normal build"
fi
exit "$fail"
