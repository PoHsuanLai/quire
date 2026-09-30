#!/usr/bin/env bash
# The consumer example is its own cargo workspace (examples/consumer), so `cargo test
# --workspace` at the root never builds it. This gate step builds, lints and tests it against
# today's public API, so CONSUMING.md's coherence rules cannot rot. It builds in
# examples/consumer/target.
set -euo pipefail
cd "$(dirname "$0")/.."

manifest=examples/consumer/Cargo.toml
cargo clippy --manifest-path "$manifest" --all-targets -- -D warnings
cargo test --manifest-path "$manifest"
