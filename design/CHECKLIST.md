# Checklist: design port means the whole look

Run at every wave gate, by the reviewer, for every surface or app the wave touched. Tokens
alone are not a port; type, layout, colour, components, motion, interaction and behaviour
all have to match (mailo memory rule, PLAN "Findings: what mailo's design layer gives us").
Derived from `30-CATALOGUE.md` (the source of truth): one quiet pre-Liquid-Glass Mac Look.

Copy this list into the wave's review note, tick each box, and cite the doc section for
anything that fails. A box that cannot be checked is a finding, not a skip.

## 1. Type

- [ ] Only the two faces: Inter (UI and display) and Space Mono (data); no second "Editorial"
      face. Cite `02-TYPE.md`, `30-CATALOGUE.md` section 3.2.
- [ ] Quiet: no caps-tracked or letter-spaced label anywhere; headings are sentence case, 11 px,
      secondary ink. Cite `30-CATALOGUE.md` section 3.4.
- [ ] Every size/weight pair is one from 02's pairs table; no raw `font-size` in
      consumer CSS (`lint` rule `RawFontSize`). Cite `02-TYPE.md`.
- [ ] Truncation follows 02 (single-line mask fade `.ds-truncate`, 2-line clamp only where
      listed). Cite `02-TYPE.md`.
- [ ] No UI text under 10 px under System: sizes are `--fs-*` steps at or above the floor
      (`lint` `MinFontSize`); a smaller drawing names its selector and reason in an exception.
      Cite `02-TYPE.md` section 12.

## 1b. Writing (02-TYPE.md section 13)

- [ ] Buttons, menu items, menu titles, segments, tabs and column headings in Title Case;
      alert and notification bodies, help and errors in sentence case with punctuation.
- [ ] "…" (never "...") on a command that opens a view asking for more (`lint` `ThreeDots`).
- [ ] Alerts: "Cancel" titles the cancel button; the default names its action; no "Yes"/"No";
      "OK" only where the alert only informs.
- [ ] Tooltips at most 75 characters, starting with a verb; errors say how to fix, no blame.
- [ ] Every control has an accessible name: text, or an `aria-label` on a glyph-only control
      (`lint` `UnnamedControl`).

## 2. Layout

- [ ] Grids, pane widths and row heights match 01. Cite `01-LAYOUT.md`.
- [ ] The window is flush: sidebar and content side by side, no inset card, no frame, no card
      shadow. Cite `30-CATALOGUE.md` section 3.4.
- [ ] Spacing values come from the spacing scale; floating surfaces clamp 8 px from edges.
      Cite `01-LAYOUT.md`.
- [ ] z-order uses `--z-*` tokens only (`lint` `RawZIndex`). Cite `01-LAYOUT.md`.

## 3. Colour

- [ ] No hex, rgb, hsl, named colour or `color-mix` outside quire (`lint` `HexColour`,
      `ColourFunction`, `NamedColour`). Cite `03-COLOR.md`.
- [ ] Each element uses the token the usage map assigns it. Cite `03-COLOR.md`.
- [ ] "Colour is a claim": chrome is greyscale; every hue states a fact. Cite
      `00-PRINCIPLES.md`.
- [ ] Contrast pairs pass in light and dark for all eight accents (`every-pair-legible`).
      Cite `03-COLOR.md`.
- [ ] Status colours are the Mac system green / orange / red (ok / warn / danger); no hue used for
      decoration.
- [ ] Shadows are the four tokens (window, popover, sheet, drag); no card shadow, no per-control
      shadow beyond the Mac push-button bezel. Cite `30-CATALOGUE.md` section 3.2.

## 4. Components

- [ ] Only `ds-` classes; no consumer rule styles `.ds-*` or `[data-theme|accent|motion|
      material]` (`lint` `DsInternals`). Cite `04-COMPONENTS.md`.
- [ ] No raw `button`, `input` or menu markup; every control is a quire component
      (`lint::markup`). Cite `04-COMPONENTS.md`.
- [ ] Every state (hover, active, focus-visible, selected, disabled) matches 04. Cite
      `04-COMPONENTS.md`.
- [ ] The surface uses the components and Material listed for it. Cite `20-SURFACES.md`.

## 4b. Short copy

- [ ] Copy is short Mac copy: a label is a word or two, a hint one short sentence; no marketing
      voice, no explanations a Mac would not show. Cite `30-CATALOGUE.md` section 3.4.
- [ ] Every default the surface introduces is a key in 22.

## 5. Motion

- [ ] Durations and easings are tokens only (`lint` `RawDuration`, `RawEasing`). Cite
      `05-MOTION.md`.
- [ ] Keyframes come from quire's `motion.css` only (`lint` `Keyframes`,
      `UnknownAnimation`), and only live ones. Cite `30-CATALOGUE.md` section 1.3.
- [ ] State after an animation is driven by quire timers (`settle`, `use_motion_timer`,
      `use_pulse`), never ad-hoc sleeps. Cite `05-MOTION.md`.
- [ ] No stagger, no first-show sweep, no overshoot, no hover lift or press squish. Cite
      `30-CATALOGUE.md` R2.
- [ ] Springs only on contact; exits accelerate; nothing loops but the spinner. Cite
      `00-PRINCIPLES.md`, `30-CATALOGUE.md` section 1.3.
- [ ] Reduced motion checked once (cross-fade at `--t-quick`, no springs). Cite
      `30-CATALOGUE.md` section 1.1.

## 5b. State details (26-DETAILS.md)

- [ ] Every stateful component the change touches keeps its state as its own enum and implements
      `Detailed`: its moments (Appear, Pending, Progress, Success, Failure, Change, Select,
      Attention, Unavailable, Preview, Dismiss) are the ones its catalogue entry in 26 specifies.
      A stateful component is not done until its moments are specified in 26 and implemented
      with the primitives (`Sweep`, `CountUp`, `Reveal`, `Pending`, `Settle`, `Shake`,
      `MorphGlyph`, `Nudge`), never with a one-off keyframe or timer. Cite
      `26-DETAILS.md#5-the-catalogue`.
- [ ] Its moment table is tested as data (`moment_table`), including the transitions that must be
      `Rest` (a value that did not change as drawn, R2).
- [ ] Each moment's harness test ends at rest: `is_animating() == false` after settle (R3); no
      `infinite` animation anywhere.
- [ ] A pending loop runs only while a real operation is pending, after `PendingGrace`, and holds
      still after `PendingCap` (R4).
- [ ] Springs only on `Touch::Contact` (R5); a failure shakes once and never escalates (R6).
- [ ] Reduced shows every final state at once; every moment has a still state that carries its
      meaning without motion (R7, R8).

## 6. Interactions

- [ ] Full keyboard path: every action reachable, focus ring 2.5 px accent offset 2.
      Cite `06-INTERACTIONS.md`.
- [ ] Hover intent 450 / 150 / 400 ms where hover opens anything. Cite
      `06-INTERACTIONS.md`.
- [ ] Escape closes the innermost layer only (`LayerStack` order). Cite
      `06-INTERACTIONS.md`.
- [ ] Pointer: the arrow on every control; the pointing hand only on a link; `grab` on drag
      handles (`lint` `PointerCursor`). Cite `04-COMPONENTS.md` "Global rules",
      `27-HIG-PARITY.md` section 6.3.
- [ ] Shortcuts: standard ones bound through `Shortcut::standard(StandardAction)`, the app's own
      through `Shortcut::custom` (which refuses a reserved key); nothing repurposes a standard
      combination (no Cmd+S sidebar, no Cmd+T command menu); modifiers drawn `⌃⌥⇧⌘`; Space switching is ⌃1-9 and ⌘1-9 stay free for apps. Cite
      `06-INTERACTIONS.md` section 2.0.
- [ ] Tab reaches every control (the default `All`); a surface that traps Tab for its own use
      says so. Cite `06-INTERACTIONS.md` section 2.0.
- [ ] Menus: arrow keys wrap, Enter/Tab pick, typing filters, outside click closes. Cite
      `06-INTERACTIONS.md`, `13-BEHAVIOUR-menus-windows.md`.

## 7. Behaviour

- [ ] Dock acceptance tests pass (if touched): Mac bounce, magnification capped at 72 px. Cite
      `10-BEHAVIOUR-dock.md`.
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

- [ ] The Space tint is one flat colour on the sidebar ground only; no gradient, no frame tokens,
      no tint on the content or on a control. Cite `21-SPACES.md`.
- [ ] A Space switch cross-fades the tint over `--t-big`; no slide, no A/B layers. Cite
      `21-SPACES.md`.
- [ ] Grain defaults to none for new Spaces and every preset; the slider stays.
- [ ] Space contrast tests pass for every preset in both schemes (the readout is not shown in
      the editor). Cite `21-SPACES.md`.

## 10. Lint

- [ ] `ds::lint::assert_clean(css, Strict)` passes for every stylesheet the wave touched.
- [ ] `ds::lint::markup` passes on every SSR render (every class is one quire exports).
- [ ] The warnings (`ds::lint::warnings`, `ds::lint::markup_warnings` under Strict: the HIG
      guardrails, printed by `assert_clean`) are read, and each is fixed or listed for the
      consumer before the rule turns Strict (`27-HIG-PARITY.md` section 7).
- [ ] `cargo fmt --all --check`, `cargo clippy --workspace --all-targets -- -D warnings`,
      `cargo test --workspace`, `./scripts/check-boundary.sh`, `./scripts/check-consumer.sh`, `cargo deny check licenses`,
      in the gate worktree with its own `CARGO_TARGET_DIR`.

## 11. Review artefacts

- [ ] Gallery contact sheet (`cargo run -p ds-gallery --release -- --snapshot
      target/gallery`) reviewed: every touched component x theme x accent.
- [ ] Nested-shell screenshots (`dev/shot.sh`) reviewed for every touched shell surface.
- [ ] The screenshot is compared side by side with the Mac reference (a real Mac, or the
      archived HIG snapshots in `27-HIG-PARITY.md`) at the same size, not from memory.

## 12. Findings

- [ ] Every problem the review saw that no test caught has an entry in the repo's
      `FINDINGS.md` (what, where, why the tests missed it, the test that would catch it).
- [ ] Every design question raised went to the doc's "Open decisions", not into code.

## 13. Sources

- PLAN "Execution model" (coherence rules 1-4, gates), "Verification", "Cross-repo order"
  (coherence gate), "Design: `<ds>`" (lint rules, components, motion API).
- mailo memory rule "design port means the whole look".

## 14. Settings

- [ ] Every proposed value the change touches is read from a settings key listed in `22-SETTINGS.md`, with that doc's default; no proposed number is hard-coded.
