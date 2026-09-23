#!/usr/bin/env python3
"""Checks that every `design/<file>.md#<anchor>` (and `ORCHESTRATION.md#<anchor>`,
`CONVENTIONS.md#<anchor>`) citation in this wave's docs points at a heading that actually
exists, using the slugging rule `design/README.md#3-citation-convention` states: lower case,
punctuation removed, spaces to hyphens.

Run once, from the repo root: `python3 tools/check-doc-anchors.py`. Exits 1 and prints every
broken citation if any is found; prints nothing and exits 0 if all citations resolve.
"""

from __future__ import annotations

import re
import sys
from pathlib import Path

ROOT = Path(__file__).resolve().parent.parent

# The docs this wave (K, adoption) owns, plus README.md — the only files this check needs to
# scan for citations.
SOURCE_DOCS = [
    ROOT / "CONSUMING.md",
    ROOT / "docs" / "mailo-migration.md",
    ROOT / "README.md",
]

# A citation: `design/<file>.md#<anchor>`, or a bare `<FILE>.md#<anchor>` for a repo-root doc.
CITATION_RE = re.compile(
    r"(?P<target>(?:design/)?[A-Za-z0-9_-]+\.md)#(?P<anchor>[a-z0-9][a-z0-9-]*)"
)

HEADING_RE = re.compile(r"^(#{1,6})\s+(.*)$")


def slugify(heading: str) -> str:
    """GitHub-style slug: lower case, punctuation removed, spaces to hyphens."""
    heading = heading.strip()
    # Strip inline code backticks and markdown emphasis markers before slugging, the way
    # GitHub's own renderer does (it slugs the rendered text, not the markdown source).
    heading = re.sub(r"[`*_]", "", heading)
    heading = heading.lower()
    heading = re.sub(r"[^a-z0-9\s-]", "", heading)
    heading = re.sub(r"\s+", "-", heading.strip())
    return heading


def headings_of(path: Path) -> set[str]:
    slugs: set[str] = set()
    seen: dict[str, int] = {}
    for line in path.read_text().splitlines():
        m = HEADING_RE.match(line)
        if not m:
            continue
        slug = slugify(m.group(2))
        if not slug:
            continue
        # GitHub disambiguates a repeated heading with -1, -2, ...; count them the same way so
        # a second "Open decisions" doesn't collide with the first.
        if slug in seen:
            seen[slug] += 1
            slug = f"{slug}-{seen[slug]}"
        else:
            seen[slug] = 0
        slugs.add(slug)
    return slugs


def resolve_target(target: str) -> Path | None:
    candidate = ROOT / target
    if candidate.exists():
        return candidate
    return None


def main() -> int:
    heading_cache: dict[Path, set[str]] = {}
    broken: list[str] = []
    checked = 0

    for doc in SOURCE_DOCS:
        if not doc.exists():
            print(f"error: source doc missing: {doc}", file=sys.stderr)
            return 1
        text = doc.read_text()
        for match in CITATION_RE.finditer(text):
            target, anchor = match.group("target"), match.group("anchor")
            path = resolve_target(target)
            if path is None:
                broken.append(f"{doc.relative_to(ROOT)}: {target}#{anchor} (file not found)")
                continue
            if path not in heading_cache:
                heading_cache[path] = headings_of(path)
            checked += 1
            if anchor not in heading_cache[path]:
                broken.append(f"{doc.relative_to(ROOT)}: {target}#{anchor} (no such heading)")

    if broken:
        print(f"{len(broken)} broken citation(s) of {checked} checked:")
        for line in broken:
            print(f"  {line}")
        return 1

    print(f"all {checked} design-doc citations resolve")
    return 0


if __name__ == "__main__":
    raise SystemExit(main())
