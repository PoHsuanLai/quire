#!/usr/bin/env bash
# The portable-core gate (design/36-PORTABLE-CORE.md section 4), usable by any cross-platform repo.
#
#   scripts/check-portable.sh <workspace-dir>
#
# A consumer repo copies this file (or calls it by path from its lane gate) and passes its own
# workspace root. Two checks:
#   (a) `cargo check --workspace --no-default-features` builds: the core stands without the
#       `quire-desktop` feature, which every desktop-only dependency sits behind.
#   (b) No module outside a `desktop/` directory names `crate::desktop` or zbus: the core never
#       reaches the desktop modules or a D-Bus crate. (A service client crate's own D-Bus
#       transport is a `dbus` or `desktop` module of that crate, so it falls under the same rule.)
# A repo whose crates ARE the D-Bus layer (quire's ds-settings, ds-helpers, ds-blitz, ds-desktop) lists
# path regexes, one per line, in `<workspace-dir>/.portable-allow`; matching files are skipped by (b).
# An app repo leaves the file out. Exit status 0 when both hold.
set -uo pipefail
dir=${1:?usage: check-portable.sh <workspace-dir>}
cd "$dir" || exit 2
fail=0

if cargo check --workspace --no-default-features >/dev/null 2>"${TMPDIR:-/tmp}/check-portable.$$.err"; then
  echo "portable build holds: cargo check --workspace --no-default-features"
else
  echo "PORTABLE BUILD FAILS: cargo check --workspace --no-default-features"
  grep -E '^(error|warning: unused)' -A4 "${TMPDIR:-/tmp}/check-portable.$$.err" | head -400
  fail=1
fi
rm -f "${TMPDIR:-/tmp}/check-portable.$$.err"

# Every Rust source under a crate's src/ or tests/ outside target/, a `desktop/` directory, or a
# file named `desktop.rs`; feature-gated `#[cfg(feature = "quire-desktop")]` lines are the
# sanctioned seam, so a line is a leak only when it names the module or zbus.
leaks=$(find . -path ./target -prune -o -name '*.rs' -print \
  | grep -vE '(^|/)(target|desktop|\.git)/|(^|/)desktop\.rs$|/tests/|/benches/' \
  | { if [ -f .portable-allow ]; then grep -vEf <(grep -vE '^\s*(#|$)' .portable-allow); else cat; fi; } \
  | xargs -r grep -nE 'crate::desktop\b|\bzbus::|\buse zbus\b|\bextern crate zbus' 2>/dev/null \
  | grep -vE '^[^:]+:[0-9]+:\s*//' || true)
if [ -n "$leaks" ]; then
  echo "PORTABLE LEAK: core modules name crate::desktop or zbus:"
  echo "$leaks" | head -400
  fail=1
else
  echo "portable boundary holds: no core module names crate::desktop or zbus"
fi
exit "$fail"
