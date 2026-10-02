# quire design docs

These documents are the canonical UX specification for quire (design system), shell-host
and sill (the shell), mailo and every bundled app. Agents build from them; reviewers check
against them.

## 0. Current direction

**One quiet Look: pre-Liquid-Glass macOS, with Dia as the reference browser for structure only.**
The window is flush (sidebar and content, no inset card); a Space tints the sidebar ground with one
flat colour; eight Mac accents, Blue default; one system face; sentence-case headings; four shadows;
Mac motion only; short copy. The dated decisions are `30-CATALOGUE.md` section 3.4. Post, Riso, Tide,
Candy, the Arc Look and the gradient frame are history in `archive/`. Where a doc describes code the
clean-up phase has not changed yet, it says "target (clean-up phase)".

**Where two docs disagree, the one lower in this list is wrong until fixed; only the user changes 30.**

1. `30-CATALOGUE.md`: rules, foundations, components, the one Look, decisions, drop list.
2. `27-HIG-PARITY.md`: the Mac anchor.
3. `00-PRINCIPLES.md`: the stance.
4. Token docs: `01-LAYOUT.md`, `02-TYPE.md`, `03-COLOR.md`, `29-SIZING.md`.
5. Behaviour: `05-MOTION.md` (what survives), `06-INTERACTIONS.md`, `10`-`13`, `26-DETAILS.md`.
6. Spaces and settings: `21-SPACES.md`, `22-SETTINGS.md`, `28-CUSTOMIZATION.md`.
7. Per surface: `20-SURFACES.md`, `23-WIDGETS.md`, `25-EMOJI.md`, `08-ICONS.md`, `31-ACCOUNTS.md`, `32-COMPANION.md`, `33-AGENT.md`.
8. `04-COMPONENTS.md`: reference for its KEEP entries only.
9. Workflow docs: `CHECKLIST.md`, this file, `ARCHITECTURE.md`, `DESIGN.md`, `CONSUMING.md`, `CONVENTIONS.md`, `FINDINGS.md`.
10. `archive/`: read-only history.

## 1. Index

| File | One line |
| --- | --- |
| `00-PRINCIPLES.md` | The stance: quiet Dia/Mac, the four rules, motion and interaction principles, one design system for every surface. |
| `01-LAYOUT.md` | Window grid, pane widths, row heights, spacing scale, z-order. |
| `02-TYPE.md` | One system face, every size/weight pair, sentence-case headings, truncation, writing. |
| `03-COLOR.md` | Light/dark tokens, usage map, eight accents, status colours, the flat Space tint, grain, materials, the four shadows. |
| `04-COMPONENTS.md` | Reference for KEEP entries only (markup skeleton, sizes, states); 30 owns the inventory. |
| `05-MOTION.md` | The surviving motion rules and timing tables; 30 section 1 wins; old keyframe and per-Look material is stamped. |
| `06-INTERACTIONS.md` | Every behaviour as a state machine: keys, hover intent, menus, drag, Escape. |
| `07-LOOKS.md` | Moved to `archive/07-LOOKS.md` (history: Post, Riso, Tide, Candy). |
| `08-ICONS.md` | Lucide/Tabler glyphs; our app icon template and generation pipeline; third-party plates. |
| `09-ARC-HEURISTICS.md` | Moved to `archive/09-ARC-HEURISTICS.md` (history: the Arc workflow brief; no longer a gate). |
| `10-BEHAVIOUR-dock.md` | macOS dock behaviour with numbers, mapped to our dock, with acceptance tests. |
| `11-BEHAVIOUR-scroll.md` | Momentum, rubber band, acceleration, scrollbars; host-side physics. |
| `12-BEHAVIOUR-gestures.md` | Magic Mouse 2 gestures, thresholds, palm rejection, configurable mapping. |
| `13-BEHAVIOUR-menus-windows.md` | Menu bar, menus, Cmd+Tab, notifications, control center, focus, launcher, sounds. |
| `20-SURFACES.md` | Per shell surface and per app: layer, Material, components, motion, behaviours, milestone. |
| `21-SPACES.md` | Spaces on the desktop: one flat tint per workspace on the sidebar ground, presets and a hue slider, grain, migration, storage. |
| `22-SETTINGS.md` | Every proposed value as a settings key with its default; storage, Rust shape, UI mapping. |
| `23-WIDGETS.md` | Desktop widgets: muted plates, the battery and clock looks, the calendar widget brief. |
| `25-EMOJI.md` | Animated emoji and the user's picture: source and CC BY 4.0 attribution, the curated set, the sheet pipeline, moods, frame budget, the idle rule; the picture's kinds, stored choice, accept beat and picker (section 7). |
| `26-DETAILS.md` | The grammar of small state details: the moments every stateful element passes through, the primitives that play them, and a per-element catalogue with the reference, today and the gap. |
| `27-HIG-PARITY.md` | Audit against the pre-2025 (macOS 14/15) HIG: archived snapshots, gaps ranked, a verdict and rule per HIG page, proposed waves H0-H7. |
| `28-CUSTOMIZATION.md` | What the person chooses and places: every surface where the reference lets people pick items, order and place (widgets, launcher categories, control center and bar, dock, toolbars, share, previewers, actions, sidebars, notifications), the shared Registry/Placement/Picker pattern, D-Bus registration, order of work. |
| `29-SIZING.md` | Settled and built: an audit of control heights, spacing and radii (quire and sill's bar and control center), the reference numbers with confidence, three principled systems with mockups, a recommendation, and the dark Monochrome/Muted plate fix. |
| `30-CATALOGUE.md` | The settled inventory: foundations (timing, motion and interaction primitives, state vocabulary, size ladder), every component with its AppKit counterpart, the one Look, the 2026-10-02 decisions, the drop list, the deferred items. The source of truth: wins over every other doc. |
| `31-ACCOUNTS.md` | Proposed: the account and capability layer: the capability vocabulary, providers as data files, accountd/syncd/inferd, consent and secrets, Photos on Storage, the local-first AI broker, the sync contract, phasing and open decisions. |
| `32-COMPANION.md` | Proposed: the companion: stance, the settled decisions, presence and the orb, the prompt and answer cards, runs, memory, confirmation, security, and where each piece lives. |
| `33-AGENT.md` | Proposed: the agent machinery in one page each (repos, shared types, daemons, flows), summarising `agent/SPEC.md`; `agent/` holds the locked spec and the area specs as reference. |
| `CHECKLIST.md` | The "design port means the whole look" review list, run at every wave gate. |
| `archive/` | Read-only history: the Arc-era principles, 07 Looks, 09 Arc heuristics, 10's old keyframes, 03's Arc/Post/Candy sections. |

## 2. Reading order

1. `30-CATALOGUE.md` sections 0 and 3 (the rules and the one Look).
2. `00-PRINCIPLES.md` (why).
3. `03-COLOR.md`, `02-TYPE.md`, `01-LAYOUT.md`, `29-SIZING.md` (tokens).
4. `06-INTERACTIONS.md`, `05-MOTION.md` (what survives), `26-DETAILS.md` (behaviour).
5. `21-SPACES.md`, `08-ICONS.md` (Spaces and icons).
6. The BEHAVIOUR doc for what you build (`10`-`13`).
7. `20-SURFACES.md` for your surface's row; `23-WIDGETS.md` for widgets.
8. `30-CATALOGUE.md` part 2 before building or changing any component or primitive; `04-COMPONENTS.md` only for a KEEP entry's markup.
9. `CHECKLIST.md` before asking for review.

An agent brief names the sections to read; it does not replace this order for new readers.

## 3. Citation convention

`design/<file>.md#<heading-anchor>`, where the anchor is the GitHub-style slug of the
heading: lower case, punctuation removed, spaces to hyphens. Headings are numbered and the
numbers are stable, so `## 6. Acceptance` in 08 is `design/08-ICONS.md#6-acceptance`, and
`### 2.1 Plate shape` is `design/08-ICONS.md#21-plate-shape`.

- A heading's number never changes once published. A new section takes the next free
  number; a removed one leaves a gap and a one-line "moved to" note.
- Prototype lines (`S:<line>`, `C:<line>`, in `~/mailo-design/`) are history: they cite the
  Arc/Post-era prototypes and are not a source of values; code is cited as `path:line`.
- A moved doc or section leaves a one-line "moved to `archive/<file>`" note where it was.

## 4. Canonical source

- These Markdown docs are canonical. Where code and docs disagree, the code is wrong until
  the doc is changed.
- The HTML prototypes in `~/mailo-design/` are the Arc/Post-era source and are history: a value
  found there is not a target unless `30-CATALOGUE.md` says so. The Mac anchor is
  `27-HIG-PARITY.md`.
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

A Claude Doc mirror (one doc, one tab per file) was published for review before the clean-up
phase; it is stale and is not refreshed. The Markdown here is canonical.

Mirror link: https://claude.ai/code/artifact/f0d97aa6-8617-42c8-9b8c-302492146490

## 7. Open decisions

1. Each doc that is still "proposed" (22, 28, 31) lists its own open decisions.
2. The clean-up phase's open code work is marked "target (clean-up phase)" in the docs it touches;
   the deferred items are `30-CATALOGUE.md` part 5.

## 8. Sources

- PLAN `~/.claude/plans/vast-toasting-peach.md`: "Phase 0", "UX decisions settled with the
  user (2026-09-23, second round)", "Phase 0 execution", "Execution model".
