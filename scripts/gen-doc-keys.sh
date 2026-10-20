#!/usr/bin/env bash
# Rewrites the generated key tables of design/22-SETTINGS.md from the schemas.
#
#   scripts/gen-doc-keys.sh [--check] [--schema <app>.settings.toml]...
#
# A table sits between `<!-- keys:begin <domain> -->` and `<!-- keys:end <domain> -->`; quire's own
# schema (appearance, icons, region) is always loaded, a program's is added with --schema (write it
# with its `--write-schema <dir>`). With --check nothing is written and the exit code is 1 when the
# doc is out of date; scripts/check-doc-keys.sh is that check.
set -euo pipefail
cd "$(dirname "$0")/.."
exec cargo run --quiet -p ds-settings --example doc_tables -- "$@" design/22-SETTINGS.md
