#!/usr/bin/env bash
# Stress the GPU-using harness tests: many parallel runs of the hybrid-backend and texture-layer
# test binaries, counting runs that die on a signal (the Vulkan loader crash, SIGSEGV = 139)
# apart from ordinary test failures. Needs a real Vulkan GPU to mean anything; without an
# adapter the tests skip and the run reports zero crashes trivially.
#
# usage: scripts/stress-gpu-harness.sh [ROUNDS] [PROCS] [TEST_THREADS]
#   ROUNDS        rounds of PROCS simultaneous test processes (default 20)
#   PROCS         test processes per round, each running both binaries (default 4)
#   TEST_THREADS  libtest threads inside each process (default 16)
# Environment passes through, so `WGPU_DEBUG=1 WGPU_VALIDATION=1 scripts/stress-gpu-harness.sh`
# measures the debug-naming path.
set -uo pipefail
cd "$(dirname "$0")/.."
rounds=${1:-20}
procs=${2:-4}
threads=${3:-16}

bins=$(cargo test -p ds-harness --test hybrid_backend --test texture_layer --no-run \
  --message-format=json 2>/dev/null |
  sed -n 's/.*"executable":"\([^"]*\)".*/\1/p') || exit 1
[ -n "$bins" ] || { echo "no test binaries built" >&2; exit 1; }

work=$(mktemp -d)
trap 'rm -rf "$work"' EXIT

one() { # one <id>: run every binary once, record each exit code
  local n=0 bin
  for bin in $bins; do
    n=$((n + 1))
    "$bin" --test-threads="$threads" >"$work/$1.$n.log" 2>&1
    echo $? >>"$work/codes"
  done
}

for r in $(seq "$rounds"); do
  for p in $(seq "$procs"); do one "$r.$p" & done
  wait
done

total=$(wc -l <"$work/codes")
crashes=$(grep -c -E '^(139|134|132|133|135|136)$' "$work/codes")
failures=$(grep -c -v -E '^(0|139|134|132|133|135|136)$' "$work/codes")
echo "runs=$total crashed=$crashes failed=$failures (rounds=$rounds procs=$procs threads=$threads)"
[ "$crashes" -eq 0 ] && [ "$failures" -eq 0 ]
