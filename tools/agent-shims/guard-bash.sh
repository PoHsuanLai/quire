#!/usr/bin/env bash
# PreToolUse hook for Bash (this session and every agent it spawns).
#
# 1. Refuses commands that bypass the cargo shim or its memory cap (exit 2 = blocked).
# 2. For commands that build, rewrites the command so the shim directory is first on
#    PATH, which routes cargo through the build lock and the 4 GB cgroup cap.
#
# Reads the tool-call JSON on stdin. Prints a JSON `updatedInput` when it rewrites.
set -u
input="$(cat)"
cmd="$(printf '%s' "$input" | python3 -c 'import json,sys; print(json.load(sys.stdin).get("tool_input",{}).get("command",""), end="")' 2>/dev/null)"
[[ -z "$cmd" ]] && exit 0

shims="$HOME/.local/bin/agent-shims"

deny() {
  echo "blocked: $1. Use plain 'cargo' (the shim serialises builds and caps memory at 4 GB); never set CARGO_BUILD_JOBS or RUSTFLAGS inline." >&2
  exit 2
}

# Regexes live in single-quoted variables: unquoted `;&` or backticks inside [[ =~ ]]
# are parsed by bash itself and break the script.
sep='(^|[[:space:];&|(`$])'
abs_cargo="${sep}[^[:space:]]*(\\.cargo/bin/cargo|/usr/bin/cargo|/usr/local/bin/cargo)([[:space:]]|\$)"
rustc_direct="${sep}rustc[[:space:]]"
toolchain="${sep}cargo[[:space:]]+\\+"
override="${sep}(CARGO_BUILD_JOBS|RUSTFLAGS|CARGO_ENCODED_RUSTFLAGS|AGENT_MEM_MAX|AGENT_SHIM_REAL_CARGO)="
routed="${sep}(cargo|limit4g|limit16g|rustup)([[:space:]]|\$)"

[[ "$cmd" =~ $abs_cargo ]]    && deny "cargo called by absolute path"
[[ "$cmd" =~ $rustc_direct ]] && deny "rustc called directly"
[[ "$cmd" =~ $toolchain ]]    && deny "toolchain override on cargo"
[[ "$cmd" =~ $override ]]     && deny "inline override of a build-cap variable"

# Already rewritten, or nothing to route: leave the command alone.
[[ "$cmd" == *"$shims"* ]] && exit 0
[[ "$cmd" =~ $routed ]] || exit 0

prefix="export PATH=\"$shims:\$PATH\"; "
python3 - "$prefix" "$cmd" <<'EOF'
import json, sys
prefix, cmd = sys.argv[1], sys.argv[2]
print(json.dumps({
    "hookSpecificOutput": {
        "hookEventName": "PreToolUse",
        "updatedInput": {"command": prefix + cmd},
    }
}))
EOF
exit 0
