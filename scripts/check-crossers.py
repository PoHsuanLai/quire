#!/usr/bin/env python3
"""No `pub(crate)` module or item may be named from another layer of `ds`.

`ds` is layered as the crates it will split into: lint, ds (host, focus,
edit, file_drop, spell, window, stack, root, components), shell, assembly. `pub(crate)` stops
at a crate boundary, so anything another layer names is `pub` at its home module. This finds
every module-level `pub(crate)` item, and every `pub(crate) mod`, that a file in another layer
reaches by a `crate::` path (in a `use` or inline; doc links count). Methods and fields follow
their type, so they are not checked. Run by scripts/check-boundary.sh; exits 1 on any crosser.
"""
import collections
import os
import re
import sys

ROOT = os.path.join(os.path.dirname(os.path.abspath(__file__)), "..", "crates", "ds", "src")
DS_LAYER = ("host", "focus", "edit", "file_drop", "spell", "window", "stack", "root", "components")
OWN_LAYER = ("lint", "shell", "assembly")

ITEM = re.compile(
    r"^(?:#\[[^\]]*\]\s*)*pub\(crate\)\s+(?:(?:async|unsafe|const)\s+)*"
    r"(fn|struct|enum|const|static|type|trait|mod|union)\s+(\w+)",
    re.M,
)
USE = re.compile(r"^pub\(crate\)\s+use\s+([^;]+);", re.M)


def layer(path):
    if not path:
        return None
    if path[0] in OWN_LAYER:
        return path[0]
    return "ds" if path[0] in DS_LAYER else None


def module_path(root, file):
    parts = os.path.relpath(file, root)[:-3].split(os.sep)
    return tuple(parts[:-1] if parts[-1] == "mod" else parts)


def expand(tree):
    """`a::{b, c::{d}}` -> ['a::b', 'a::c::d']."""
    def rec(prefix, body):
        out, depth, cur, parts = [], 0, "", []
        for ch in body:
            depth += ch == "{"
            depth -= ch == "}"
            if ch == "," and depth == 0:
                parts.append(cur)
                cur = ""
            else:
                cur += ch
        if cur:
            parts.append(cur)
        for part in parts:
            i = part.find("{")
            if i >= 0 and part.endswith("}"):
                out += rec(prefix + part[:i], part[i + 1:-1])
            else:
                out.append(prefix + part)
        return out
    return rec("", re.sub(r"\s+", "", re.sub(r"\s+as\s+", "@", tree)))


def crossers(root=ROOT):
    """Return {(module path, name or None, kind, home layer): {(user file, user layer)}}."""
    files = [os.path.join(d, f) for d, _, fs in os.walk(root) for f in fs if f.endswith(".rs")]
    text = {f: open(f, encoding="utf-8").read() for f in files}
    modules = {module_path(root, f) for f in files}
    items = collections.defaultdict(dict)
    for f, t in text.items():
        here = module_path(root, f)
        for kind, name in ITEM.findall(t):
            items[here][name] = kind
        for tree in USE.findall(t):
            for path in expand(tree):
                items[here][path.rsplit("@", 1)[-1].rsplit("::", 1)[-1]] = "use"
    found = collections.defaultdict(set)
    for f, t in text.items():
        mine = module_path(root, f)
        L = layer(mine)
        if L is None:
            continue
        paths, alias = [], {}
        for m in re.finditer(r"\buse\s+(crate[^;]*);", t, re.S):
            for path in expand(m.group(1)):
                paths.append(path)
                full, _, local = path.partition("@")
                segs = [x for x in full.split("::") if x not in ("", "self")]
                if tuple(segs[1:]) in modules:
                    alias[local or segs[-1]] = segs[1:]
        paths += ["crate::" + m for m in re.findall(r"\bcrate::(\w+(?:::\w+)*)", t)]
        for name, target in alias.items():
            paths += ["crate::" + "::".join(target + [m]) for m in re.findall(r"(?<![\w:])" + name + r"::(\w+)", t)]
        for path in paths:
            segs = [s for s in path.partition("@")[0].split("::") if s not in ("", "self")]
            if not segs or segs[0] != "crate":
                continue
            segs = segs[1:]
            depth = 0
            while depth < len(segs) and tuple(segs[:depth + 1]) in modules:
                depth += 1
            for j in range(depth):
                parent, child = tuple(segs[:j]), tuple(segs[:j + 1])
                if items.get(parent, {}).get(segs[j]) == "mod" and layer(child) != L:
                    found[(child, None, "mod", layer(child))].add((os.path.relpath(f, root), L))
            home = tuple(segs[:depth])
            if depth < len(segs):
                name = segs[depth]
                kind = items.get(home, {}).get(name)
                if kind and kind != "mod" and layer(home) != L:
                    found[(home, name, kind, layer(home))].add((os.path.relpath(f, root), L))
    return found


def main():
    found = crossers()
    for (home, name, kind, home_layer), users in sorted(found.items(), key=lambda kv: str(kv[0])):
        who = sorted({layer_ for _, layer_ in users})
        where = "::".join(home) + (f"::{name}" if name else "")
        print(f"CROSSER: pub(crate) {kind} {where} ({home_layer}) is named from {', '.join(who)}")
    if found:
        print(f"{len(found)} pub(crate) crossers: make each `pub` at its home module, or move it")
    return 1 if found else 0


if __name__ == "__main__":
    sys.exit(main())
