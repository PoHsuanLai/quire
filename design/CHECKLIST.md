# Checklist: design port means the whole look

Run at every wave gate, by the reviewer, for every surface or app the wave touched. Tokens
alone are not a port; type, layout, colour, components, motion, interaction and behaviour
all have to match (mailo memory rule, PLAN "Findings: what mailo's design layer gives us").

Copy this list into the wave's review note, tick each box, and cite the doc section for
anything that fails. A box that cannot be checked is a finding, not a skip.

## 1. Type

- [ ] Only the three faces: display Bricolage Grotesque, ui Karla, data Space Mono.
      Cite `02-TYPE.md`.
- [ ] Every size/weight/tracking pair is one from 02's pairs table; no raw `font-size` in
      consumer CSS (`lint` rule `RawFontSize`). Cite `02-TYPE.md`.
- [ ] Truncation follows 02 (single-line mask fade `.ds-truncate`, 2-line clamp only where
      listed). Cite `02-TYPE.md`.

## 2. Layout

- [ ] Grids, pane widths and row heights match 01. Cite `01-LAYOUT.md`.
- [ ] Spacing values come from the spacing scale; floating surfaces clamp 8 px from edges.
      Cite `01-LAYOUT.md`.
- [ ] z-order uses `--z-*` tokens only (`lint` `RawZIndex`). Cite `01-LAYOUT.md`.

## 3. Colour

- [ ] No hex, rgb, hsl, named colour or `color-mix` outside quire (`lint` `HexColour`,
      `ColourFunction`, `NamedColour`). Cite `03-COLOR.md`.
- [ ] Each element uses the token the usage map assigns it. Cite `03-COLOR.md`.
- [ ] "Colour is a claim": chrome is greyscale; every hue states a fact. Cite
      `00-PRINCIPLES.md`.
- [ ] Contrast pairs pass in light and dark for all 6 accents (`every-pair-legible`).
      Cite `03-COLOR.md`.

## 4. Components

- [ ] Only `ds-` classes; no consumer rule styles `.ds-*` or `[data-theme|accent|motion|
      material]` (`lint` `DsInternals`). Cite `04-COMPONENTS.md`.
- [ ] No raw `button`, `input` or menu markup; every control is a quire component
      (`lint::markup`). Cite `04-COMPONENTS.md`.
- [ ] Every state (hover, active, focus-visible, selected, disabled) matches 04. Cite
      `04-COMPONENTS.md`.
- [ ] The surface uses the components and Material listed for it. Cite `20-SURFACES.md`.

## 5. Motion

- [ ] Durations and easings are tokens only (`lint` `RawDuration`, `RawEasing`). Cite
      `05-MOTION.md`.
- [ ] Keyframes come from quire's `motion.css` only (`lint` `Keyframes`,
      `UnknownAnimation`). Cite `05-MOTION.md`.
- [ ] State after an animation is driven by quire timers (`settle`, `use_motion_timer`,
      `use_pulse`), never ad-hoc sleeps. Cite `05-MOTION.md`.
- [ ] Stagger only on first show, capped at 12 rows. Cite `05-MOTION.md`.
- [ ] Springs only on contact; exits accelerate; nothing loops. Cite `00-PRINCIPLES.md`,
      `05-MOTION.md`.
- [ ] Reduced motion level checked once (60 ms everywhere). Cite `05-MOTION.md`.

## 6. Interactions

- [ ] Full keyboard path: every action reachable, focus ring 2.5 px accent offset 2.
      Cite `06-INTERACTIONS.md`.
- [ ] Hover intent 450 / 150 / 400 ms where hover opens anything. Cite
      `06-INTERACTIONS.md`.
- [ ] Escape closes the innermost layer only (`LayerStack` order). Cite
      `06-INTERACTIONS.md`.
- [ ] Menus: arrow keys wrap, Enter/Tab pick, typing filters, outside click closes. Cite
      `06-INTERACTIONS.md`, `13-BEHAVIOUR-menus-windows.md`.

## 7. Behaviour

- [ ] Dock acceptance tests pass (if touched). Cite `10-BEHAVIOUR-dock.md`.
- [ ] Scroll acceptance tests pass (if touched). Cite `11-BEHAVIOUR-scroll.md`.
- [ ] Gesture acceptance tests pass (if touched). Cite `12-BEHAVIOUR-gestures.md`.
- [ ] Menu, window, notification and launcher acceptance tests pass (if touched). Cite
      `13-BEHAVIOUR-menus-windows.md`.
- [ ] Idle surface paints 0 frames (`sill debug stats`). Cite `20-SURFACES.md`.

## 8. Icons

- [ ] Glyphs come from the `Icon` enum through `Glyph`, at an `IconSize` variant; stroke is
      `currentColor`. Cite `08-ICONS.md#13-data-model-settled-moves-verbatim-from-mailo-then-made-pub`.
- [ ] Tray and third-party symbolic icons are recoloured to the ink token. Cite
      `08-ICONS.md#15-colour`.
- [ ] App icons pass the template acceptance (silhouette within 1 px, safe area,
      contrast ≥ 3.0, 16 px legible). Cite `08-ICONS.md#6-acceptance`.
- [ ] Third-party tiles use the plate-mask rule. Cite `08-ICONS.md#4-third-party-app-icons`.

## 9. Spaces

- [ ] Tinted chrome uses `--f-*` frame tokens only; apps stay on Post tokens. Cite
      `21-SPACES.md#3-where-the-tokens-apply`.
- [ ] Workspace switch cross-fades over `--t-scene` (380 ms) with the A/B layers. Cite
      `21-SPACES.md#5-workspace-switch-settled-model`.
- [ ] Space contrast tests pass for all 8 presets in both schemes. Cite
      `21-SPACES.md#7-contrast-guarantees-settled-floors-tests`.

## 10. Lint

- [ ] `ds::lint::assert_clean(css, Strict)` passes for every stylesheet the wave touched.
- [ ] `ds::lint::markup` passes on every SSR render (every class is one quire exports).
- [ ] `cargo fmt --all --check`, `cargo clippy --workspace --all-targets -- -D warnings`,
      `cargo test --workspace`, `./scripts/check-boundary.sh`, `cargo deny check licenses`,
      in the gate worktree with its own `CARGO_TARGET_DIR`.

## 11. Review artefacts

- [ ] Gallery contact sheet (`cargo run -p ds-gallery --release -- --snapshot
      target/gallery`) reviewed: every touched component x theme x accent.
- [ ] Nested-shell screenshots (`dev/shot.sh`) reviewed for every touched shell surface.
- [ ] The screenshot is compared side by side with the prototype (`~/mailo-design/`) at
      the same size, not from memory.

## 12. Findings

- [ ] Every problem the review saw that no test caught has an entry in the repo's
      `FINDINGS.md` (what, where, why the tests missed it, the test that would catch it).
- [ ] Every design question raised went to the doc's "Open decisions", not into code.

## 13. Sources

- PLAN "Execution model" (coherence rules 1-4, gates), "Verification", "Cross-repo order"
  (coherence gate), "Design: `<ds>`" (lint rules, components, motion API).
- mailo memory rule "design port means the whole look".
