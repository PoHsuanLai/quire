#!/usr/bin/env bash
# The performance budgets: tests that assert how long this machine took. A wall-time budget fails
# on a shared, loaded machine for reasons that are not the code's, so each is `#[ignore = "perf:
# ..."]` and outside the gate; run this on a quiet one. Add a new budget's test binary to PERF.
#
#   scripts/perf.sh           the budgets (debug build, as the gate builds)
#   scripts/perf.sh --release the same, optimised
set -euo pipefail
cd "$(dirname "$0")/.."

# "<package> <test module>": every `#[ignore = "perf: ..."]` test in that module of the crate's `it` executable.
PERF=(
  "ds-blitz pdf_output"
)

profile=()
[ "${1:-}" = "--release" ] && profile=(--release)

load=$(cut -d' ' -f1 /proc/loadavg 2>/dev/null || echo 0)
if awk -v l="$load" 'BEGIN { exit !(l > 8) }'; then
  echo "perf.sh: load average is $load; a budget measured now says little. Run it on a quiet machine." >&2
fi

for entry in "${PERF[@]}"; do
  read -r package binary <<<"$entry"
  cargo test "${profile[@]}" -p "$package" --all-features --test it "${binary}::" -- --ignored --nocapture
done
