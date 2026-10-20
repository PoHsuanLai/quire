#!/usr/bin/env bash
# Fails when a generated key table of design/22-SETTINGS.md differs from the schemas. Fix it with
# scripts/gen-doc-keys.sh. Extra arguments (--schema <file>) are passed through.
set -euo pipefail
exec "$(dirname "$0")/gen-doc-keys.sh" --check "$@"
