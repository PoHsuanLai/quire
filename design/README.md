# quire design docs

These documents are the canonical UX specification for quire (design system), shell-host
and sill (the shell), mailo and every bundled app. Agents build from them; reviewers check
against them.

## 1. Index

| File | One line |
| --- | --- |
| `00-PRINCIPLES.md` | The stance in the prototypes' words; which parts are Arc, which are Mac; one design system for every surface. |
| `01-LAYOUT.md` | Window grid, pane widths, row heights, spacing scale, z-order. |
| `02-TYPE.md` | Three faces, every size/weight/tracking pair, truncation. |
| `03-COLOR.md` | Light/dark tokens, usage map, Space palette, grain, Candy hues, materials. |
| `04-COMPONENTS.md` | Every component: markup skeleton, sizes, every state. |
| `05-MOTION.md` | Every keyframe, transition, timing token and level; the motion rules. |
| `06-INTERACTIONS.md` | Every behaviour as a state machine: keys, hover intent, menus, drag, Escape. |
| `07-LOOKS.md` | Post, Riso, Tide, Candy and warmth; which look Spaces is. |
| `08-ICONS.md` | Lucide/Tabler glyphs; our app icon template and generation pipeline; third-party plates. |
| `10-BEHAVIOUR-dock.md` | macOS dock behaviour with numbers, mapped to our dock, with acceptance tests. |
| `11-BEHAVIOUR-scroll.md` | Momentum, rubber band, acceleration, scrollbars; host-side physics. |
| `12-BEHAVIOUR-gestures.md` | Magic Mouse 2 gestures, thresholds, palm rejection, configurable mapping. |
| `13-BEHAVIOUR-menus-windows.md` | Menu bar, menus, Cmd+Tab, notifications, control center, focus, launcher, sounds. |
| `20-SURFACES.md` | Per shell surface and per app: layer, Material, components, motion, behaviours, milestone. |
| `21-SPACES.md` | Spaces on the desktop: SpaceLook per workspace, frame tokens on shell chrome, storage. |
| `22-SETTINGS.md` | Every proposed value as a settings key with its default; storage, Rust shape, UI mapping. |
| `23-WIDGETS.md` | Desktop widgets' depth language, the battery and clock candidate looks, the calendar widget brief. |
| `CHECKLIST.md` | The "design port means the whole look" review list, run at every wave gate. |

## 2. Reading order

1. `00-PRINCIPLES.md` (why).
2. `03-COLOR.md`, `02-TYPE.md`, `01-LAYOUT.md` (tokens).
3. `04-COMPONENTS.md`, `05-MOTION.md`, `06-INTERACTIONS.md` (the building blocks).
4. `07-LOOKS.md`, `08-ICONS.md`, `21-SPACES.md` (the look around them).
5. The BEHAVIOUR doc for what you build (`10`-`13`).
6. `20-SURFACES.md` for your surface's row.
7. `CHECKLIST.md` before asking for review.

An agent brief names the sections to read; it does not replace this order for new readers.

## 3. Citation convention

`design/<file>.md#<heading-anchor>`, where the anchor is the GitHub-style slug of the
heading: lower case, punctuation removed, spaces to hyphens. Headings are numbered and the
numbers are stable, so `## 6. Acceptance` in 08 is `design/08-ICONS.md#6-acceptance`, and
`### 2.1 Plate shape` is `design/08-ICONS.md#21-plate-shape`.

- A heading's number never changes once published. A new section takes the next free
  number; a removed one leaves a gap and a one-line "moved to" note.
- Prototype lines are cited as `S:<line>` (`~/mailo-design/mailo-spaces.html`) and
  `C:<line>` (`mailo-charm.html`); code as `path:line`.

## 4. Canonical source

- These Markdown docs are canonical. Where code and docs disagree, the code is wrong until
  the doc is changed.
- The HTML prototypes in `~/mailo-design/` are the docs' source: values were extracted
  from them with line citations. `mailo-spaces.html` wins over `mailo-charm.html` on any
  clash. A value found in a prototype but missing here is added to the doc first, then
  built.
- Every value is marked settled (decided with the user, or copied from a cited source) or
  proposed. Decision of 2026-09-24: **every proposed value is the shipped default and is a
  settings key** (see `22-SETTINGS.md`), so the user tunes it once things are in place instead
  of guessing. An unmarked number is a doc bug.

## 5. Proposing a change

1. Edit the doc section in the same change as (or before) the code.
2. Mark the new value "proposed" and add an entry under the doc's "Open decisions" with the
   reason and the alternative.
3. Add a `FINDINGS.md` entry in the repo where the need showed up: what was seen, which test
   missed it.
4. The user settles it; the orchestrating session changes "proposed" to "settled" and
   removes the open-decision entry.

Agents never change a settled value on their own; they stop and report (coherence rule 3).

## 6. Claude Doc mirror

A Claude Doc mirror (one doc, one tab per file) is published for review. The user comments
there; the orchestrating session folds the comments back into these files. The Markdown here
stays canonical.

Mirror link: https://claude.ai/code/artifact/f0d97aa6-8617-42c8-9b8c-302492146490

## 7. Open decisions

1. None at the moment.

## 8. Sources

- PLAN `~/.claude/plans/vast-toasting-peach.md`: "Phase 0", "UX decisions settled with the
  user (2026-09-23, second round)", "Phase 0 execution", "Execution model".
