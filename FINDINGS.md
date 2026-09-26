# Findings

## Blitz spike S1-S16 (W0 `spike`, blitz @ e99fbdbd, dioxus 0.7.10)

How to reproduce: `cd spike/blitz-probe && cargo run -- all` renders every probe headlessly at
400x300 with anyrender_vello_cpu (the S15/S16 probes also go through anyrender_vello_hybrid,
offscreen on the first wgpu adapter) into `spike/out/*.png` and prints the table below. PNGs are
gitignored. `cargo run --features cpu-filters -- s15 s16` repeats S15/S16 with
`anyrender_vello_cpu/filters` on. The probe is its own Cargo workspace, excluded from quire's,
and uses the pinned lines from `docs/workspace-deps.toml` unchanged. Documents are
`DioxusDocument`s driven with `resolve(t)`, `poll`, `handle_ui_event`.

| id | result | evidence |
|----|--------|----------|
| S1 | YES | s01-body-style.png |
| S2 | PARTIAL: `[data-x=v]` never matches, `[*\|data-x=v]` and classes do | s02-ds-cascade-{attr,nswild,class}.png |
| S3 | YES | s03-keyframes-t{000,010,030}.png |
| S4 | YES | s04-transition-{before,t000,t010,t030}.png |
| S5 | YES | s05-restart-t{050,060}.png |
| S6 | YES (attribute only) | s06-svg-currentcolor.png |
| S7 | YES, but only with a data: NetProvider | s07-mask-svg-{dummynet,datauri}.png |
| S8 | YES, but only with a data: NetProvider | s08-bg-png-{dummynet,datauri}.png |
| S9 | PARTIAL: zero rect inside onmounted, correct afterwards | s09-onmounted-rect.png |
| S10 | YES from both component body and handler | s10-timer-{body,handler}.png |
| S11 | YES | s11-font-{unregistered,registered}.png |
| S12 | NO | s12-focus-{none,tab,click}.png |
| S13 | NO for ellipsis, YES for the mask fade | s13-ellipsis-and-fade.png |
| S14 | YES | s14-color-mix.png |
| S15 | NO on cpu and on hybrid | s15-backdrop-{cpu,cpu-filters,hybrid}.png |
| S16 | NO on cpu as pinned; hybrid does blur() but not saturate() | s16-filter-{cpu,cpu-filters,hybrid}.png |

### S1. Does a `<style>` element in the body apply?

Probed: `rsx! { style { ".s1 { width: 200px; height: 100px; background-color: rgb(0,160,0) }" } div { class: "s1" } }`
under `<main>` inside `<body>`.

Result: **YES**. `spike/out/s01-body-style.png` shows a green 200x100 box.

Fallback: none needed. `Ds{stylesheet: Inject::Inline}` works on Blitz; `Inject::Host` stays
optional.

### S2. Does the `.ds[data-*]` custom-property cascade work, including a nested `.ds`?

Probed:
```css
.ds { --paper: rgb(240,240,240); --ink: rgb(20,20,20); background-color: var(--paper); color: var(--ink); padding: 10px }
.ds[data-theme=dark] { --paper: rgb(20,20,20); --ink: rgb(240,240,240) }
.ds[data-accent=blue] { --accent: rgb(0,0,220) }   .ds[data-accent=red] { --accent: rgb(220,0,0) }
.chip { background-color: var(--accent) }  .frame { background-color: var(--f-x) }
```
Markup: `div.ds[data-theme=light][data-accent=blue][style="--f-x: rgb(220,0,220)"] > (.chip, .frame,
div.ds[data-theme=dark][data-accent=red] > (.chip, .frame))`. The same CSS was also run with
`[*|data-…]` selectors and with classes (`.theme-dark`, `.accent-red`).

Result: **PARTIAL**.
- `spike/out/s02-ds-cascade-attr.png`: **no** `[data-…=…]` rule matches. The nested scope stays
  light and both chips are unpainted. Only the inline `--f-x` custom property and its inheritance
  into the nested scope work (magenta bars).
- `spike/out/s02-ds-cascade-nswild.png` (`[*|data-theme=dark]`) and
  `spike/out/s02-ds-cascade-class.png`: all six checks pass. The outer scope is light with a blue
  chip, the nested scope is dark with a red chip, and `--f-x` is inherited.

Root cause: dioxus-native-dom's `mutation_writer.rs` creates every attribute through
`qual_name(name, None)`, which puts it in the **HTML namespace** rather than the null namespace.
Stylo's `attr_matches` compares namespaces, so a selector without a namespace
(`[data-theme=dark]`, `[aria-selected=true]`, `[data-variant=…]`) never matches an attribute
Dioxus sets. Custom-property cascade, `var()` and nesting all work; attribute selectors are what
is broken. This also explains the first S4/S5 runs, which failed only because they used
`[data-on]` and `[data-pulse]`.

Fallback: this is a new branch not in the risk table. The CSS emitter must write attribute
selectors as `[*|data-theme=dark]` (valid CSS that browsers also match, so the mailo webview
phase is unaffected). Every §11 `data-variant`/`data-size`/`aria-*` selector in component CSS
needs the same prefix, or the state becomes a class. Add a `lint::Rule` that flags an unprefixed
attribute selector under the Blitz profile. Worth reporting upstream as a dioxus-native-dom bug.

### S3. Do `@keyframes` work, including `var()` inside keyframes?

Probed (1 s, linear, `both`; resolved at t = 0, 0.1 and 0.3 s):
```css
@keyframes s3lit  { from { transform: translateX(0px) } to { transform: translateX(300px) } }
@keyframes s3var  { from { background-color: var(--from) } to { background-color: var(--to) } }
@keyframes s3dist { from { transform: translateX(0px) } to { transform: translateX(var(--dist)) } }
.a1 { animation: s3lit 1s linear both }
.a2 { --from: rgb(255,0,0); --to: rgb(0,0,255); animation: s3var 1s linear both }
.a3 { --dist: 300px; animation: s3dist 1s linear both }
.a4 { --t: 1s; animation: s3lit var(--t) linear both }
```
Result: **YES**. In `spike/out/s03-keyframes-t030.png` the literal, `var()`-distance and
`var()`-duration boxes are all at x = 90 (0.3 x 300). In the `var()`-colour box the colour moves
from rgb(255,0,0) at t=0 to rgb(179,0,77) at t=0.3. The t000 and t010 PNGs show the intermediate
positions.

Fallback: none. The canonical motion.css keyframes and `var(--t-*)` durations can be used as
designed.

### S4. Does `transition` work on a value driven by a custom property?

Probed:
```css
.t { width: var(--w); background-color: var(--c); transition: width 1s linear, background-color 1s linear }
.t.on-a { --w: 40px; --c: rgb(255,0,0) }   .t.on-b { --w: 340px; --c: rgb(0,0,255) }
.u { transform: translateX(var(--x)); transition: transform 1s linear }
.u.on-a { --x: 0px }   .u.on-b { --x: 300px }
```
The class flips from `on-a` to `on-b` through a vdom re-render at t = 1.0 s, and the document is
resolved at 1.0, 1.1 and 1.3 s.

Result: **YES**. Width goes 40, 70, 130 px, the colour is rgb(179,0,77) at +0.3 s, and the
translate box is at x = 90 (`spike/out/s04-transition-t030.png`). Note: `[data-on=…]` selectors
fail for the S2 reason, so the probe uses classes.

Fallback: none. Tokens can drive transitioned properties through `var()`.

### S5. Does swapping `animation-name` (X to X--b) restart the animation?

Probed: `@keyframes grow` and an identical `@keyframes grow--b` (width 20 to 380 px, 1 s linear).
Three rows:
1. switches `.pulse-a` to `.pulse-b` (name swap) at t = 0.5;
2. a control that keeps running;
3. gets `.off { animation-name: none }` for one resolve, and the animation is then restored.

Result: **YES**. In `spike/out/s05-restart-t060.png`, 0.1 s after the change, the name-swap row
is 56 px wide (restarted), the control is 236 px (continued), and the one-frame-removal row is
20 px (restarted on the resolve where it came back). At t = 0.5 all three were 200 px
(`s05-restart-t050.png`).

Fallback: A/B aliases (`use_pulse`) work as designed. The one-frame class removal also works if it
is ever needed.

### S6. Does inline SVG `stroke="currentColor"` follow the CSS `color`?

Probed: `div.red { color: rgb(220,0,0) } > svg[viewBox="0 0 24 24"][fill=none][stroke=currentColor][stroke-width=4] > path[d="M2 12h20M12 2v20"]`,
the same under `div.blue`, and a third copy with no `stroke` attribute but the CSS rule
`.css-stroke path { stroke: currentColor }`.

Result: **YES** for the attribute. `spike/out/s06-svg-currentcolor.png` shows a red cross and a
blue cross. The third copy, stroked only through a CSS rule, is **invisible**: stylesheet rules do
not reach SVG children, which are rendered from their own attributes.

Fallback: `Glyph` renders `svg.ds-ic` with `stroke="currentColor"` and the other stroke attributes
inline, as planned. The mask-image fallback is not needed. Component CSS must never try to style
`path`/`circle` inside an icon; add that to the lint's Blitz profile.

### S7. Does `mask-image: url(data:image/svg+xml,…)` with `background-color: currentColor` work?

Probed:
```css
.m { width: 100px; height: 100px; background-color: currentColor; color: rgb(0,160,0);
     mask-image: url("data:image/svg+xml,%3Csvg xmlns='http://www.w3.org/2000/svg' viewBox='0 0 24 24'%3E%3Ccircle cx='12' cy='12' r='6' fill='black'/%3E%3C/svg%3E");
     mask-size: 100% 100%; mask-repeat: no-repeat }
```
Result: **YES, conditionally**.
- With the default `DocumentConfig` (`DummyNetProvider`), nothing is painted at all: the element
  is fully masked (`spike/out/s07-mask-svg-dummynet.png`, blank).
- With a 20-line data:-only `NetProvider` (the same logic as blitz-shell's `DataUriNetProvider`,
  which sits behind blitz-shell's `data-uri` feature and is not in our pinned set),
  `spike/out/s07-mask-svg-datauri.png` shows a green disc on white.
- The image arrives one resolve late: the first frame is blank (`s07-mask-svg-datauri-first.png`).

Fallback: none for masks, but ds-native must install a data: `NetProvider` in every
`DocumentConfig` and pass a `NetWaker` that requests a redraw, so the second frame gets painted.

### S8. Does `background-image: url(data:image/png;base64,…)` tile?

Probed: an 8x8 red/blue checker PNG, base64-encoded at runtime, as
`.bg { width: 200px; height: 100px; background-image: url(data:image/png;base64,…) }`.

Result: **YES, conditionally**. The conditions are the same as S7: blank with the default net
provider (`spike/out/s08-bg-png-dummynet.png`), and correctly tiled at 8 px with the data:
provider (`spike/out/s08-bg-png-datauri.png`). The pixels at (2,2), (6,2), (10,2), (14,6) and
(190,90) are red, blue, red, red, blue. The first frame is blank here too.

Fallback: none. The grain PNG from build.rs works, given the data: `NetProvider` from S7.

### S9. Does dioxus `onmounted` plus `get_client_rect()` return a rect?

Probed: `div.target { position: absolute; left: 50px; top: 60px; width: 123px; height: 45px }`
with `onmounted: move |e| async move { e.get_client_rect().await; Delay(20ms).await; e.get_client_rect().await }`.
The host calls `initial_build`, `resolve(0)`, `poll`, sleeps 60 ms, then calls `poll` again.

Result: **PARTIAL**. Inside the handler the call returns `Rect(0x0 at (0,0))`: mounted events are
flushed in `initial_build`/`poll` before any layout has run. The same `MountedData` queried after
the first resolve returns `Rect(123x45 at (50,60))`. `spike/out/s09-onmounted-rect.png` shows both
values in the readout.

Fallback: a partial version of the S9 fallback. `use_rect` keeps the `MountedData` and reads it
after the next frame, or on request (a timer tick or the host's post-resolve hook), never inside
`onmounted` itself. `LayoutProbe` is not needed, since the data is correct once layout exists.

### S10. Does a `futures-timer` sleep wake the document, spawned from the component body and from an event handler?

Probed: `use_hook(|| spawn(async { Delay::new(50ms).await; body.set(true) }))`, and a button whose
`onclick` spawns the same sleep and then `handler.set(true)`. The host polls with a counting
`Waker`, sleeps 120 ms, then polls again. The click is synthesised as PointerMove, PointerDown,
PointerUp.

Result: **YES for both**. Body timer: the waker is woken once and the next `poll` returns true.
Handler timer: the waker is woken once and the next `poll` returns true. Both flags turn green in
`spike/out/s10-timer-handler.png`, and `s10-timer-body.png` shows the body flag alone.

Fallback: none. Timers may start in render or in handlers. The host still has to bridge the waker
it passes to `poll` to its calloop ping.

### S11. Is a bundled TTF registered through `DocumentConfig.font_ctx` picked by `font-family`?

Probed: FiraMono-Medium.ttf (OFL, not installed system-wide) registered with
`FontContext::new().collection.register_fonts(Blob, None)`, which reports family `"Fira Mono"`.
The context is passed as `DocumentConfig.font_ctx`, with the CSS
`.f { font-family: 'Fira Mono', serif; font-size: 28px }` over the lines `iiiiiiiiii` and
`WWWWWWWWWW`.

Result: **YES**.
- Unregistered (`spike/out/s11-font-unregistered.png`): the serif fallback is used and the lines
  are 87 px and 292 px wide.
- Registered (`spike/out/s11-font-registered.png`): Fira Mono is used and the lines are 162 px and
  167 px wide, i.e. monospace.

Fallback: none. Register the faces once in a shared `FontContext` and clone it into each document.

### S12. Does `:focus-visible` match after a synthetic keyboard focus?

Probed:
```css
button:focus { background-color: rgb(255,220,0) }
button:focus-visible { box-shadow: 0 0 0 6px rgb(220,0,220) }
```
The probe sends a Tab key (KeyDown and KeyUp), then clicks the second button.

Result: **NO**.
- After Tab (`spike/out/s12-focus-tab.png`) the first button is yellow, so `:focus` matches, but
  it has no magenta ring. blitz-dom's `stylo.rs` hard-codes `NonTSPseudoClass::FocusVisible => false`
  (and `FocusWithin => false`).
- Clicking the second button (`s12-focus-click.png`) blurs the first button but does **not** focus
  the second: pointer-down focuses only text inputs and checkboxes.

Fallback: a new branch. ds-native tracks input modality (the last input was a key or a pointer)
and stamps `data-modality=keyboard|pointer` on `.ds`. Focus rings are written as
`.ds[*|data-modality=keyboard] :focus`, and the lint bans `:focus-visible` and `:focus-within` in
the Blitz profile. A component that must take focus on click calls `set_focus` from its handler
(or the host does it).

### S13. Does `text-overflow: ellipsis` work (expected NO), and does a `mask-image` end-fade work as the fallback?

Probed (150 px wide, 20 px text "The quick brown fox jumps over"):
```css
.t { white-space: nowrap; overflow: hidden; text-overflow: ellipsis }
.f { white-space: nowrap; overflow: hidden; mask-image: linear-gradient(to right, black 70%, transparent) }
.c { white-space: nowrap; overflow: hidden }
```
Result: **NO for ellipsis, YES for the fade**. In `spike/out/s13-ellipsis-and-fade.png`:
- the ellipsis row is hard-clipped at "The quick brow" with no "…", identical to the plain clip row;
- the fade row fades "brow" out to transparent (ink in its last 12 px: 1659, against 8697 for the
  clip).

Fallback: as planned, `.ds-truncate` uses the mask fade, and `text/clip.rs` handles cases that need
a real ellipsis.

### S14. Does `color-mix()` work in a background?

Probed:
```css
.a { background-color: color-mix(in srgb, rgb(255,0,0) 50%, rgb(0,0,255)) }
.b { --accent: rgb(0,0,255); background-color: color-mix(in srgb, var(--accent) 20%, transparent) }
.c { background-color: color-mix(in oklab, rgb(255,0,0), rgb(0,0,255)) }
```
Result: **YES**. In `spike/out/s14-color-mix.png` the three boxes are:
- `.a`: rgb(128,0,128);
- `.b` over white: rgb(204,204,255), so `var()` inside works;
- `.c`: rgb(140,83,162), a plausible oklab midpoint.

Fallback: not required. The precomputed washes (`--ok-wash` and the rest) may stay for
determinism, but `color-mix` is not something Blitz forces us to ban.

### S15. Does `backdrop-filter: blur()` work (expected NO on vello backends)?

Probed: a full-page pattern of 10 px black and white stripes, covered by
`.glass { position: absolute; left: 100px; top: 75px; width: 200px; height: 150px; backdrop-filter: blur(10px); background-color: rgba(255,255,255,0.1) }`.

Result: **NO**, on every backend tried: anyrender_vello_cpu as pinned, vello_cpu with `filters`,
and anyrender_vello_hybrid on an RTX 5070 Ti. `spike/out/s15-backdrop-{cpu,cpu-filters,hybrid}.png`
all show sharp stripes under the glass; a black stripe reads rgb(26,26,26), i.e. only the 10 %
white tint. Both anyrender backends take the backdrop filter as `_backdrop_filter` and ignore it.

Fallback: as planned, behind-surface blur comes from the compositor (ext-background-effect-v1).
The material uses `data-blur=on` with `--m-tint`, or `data-blur=off` with `--m-tint-solid`.

### S16. Does `filter: saturate()` work on anyrender_vello_cpu and on the hybrid backend?

Probed: `.s { filter: saturate(0) }` on a red box and `.bl { filter: blur(4px) }` on another.

Result: **NO as pinned**. In detail:
- **anyrender_vello_cpu as pinned** (`features = ["multithreading"]`,
  `spike/out/s16-filter-cpu.png`): neither filter is applied.
- **With `filters` added** (`s16-filter-cpu-filters.png`): still nothing, because
  `anyrender_vello_cpu` 0.17 `scene.rs` drops every filter when `multithreading` is on
  (`.filter(|_| cfg!(not(feature = "multithreading")))`). Per its `filters.rs`, single-threaded
  vello_cpu with `filters` would convert ColorMatrix (saturate) and blur. That configuration was
  not rendered, because it means dropping a pinned feature.
- **anyrender_vello_hybrid** (`s16-filter-hybrid.png`): `blur(4px)` is applied (soft edges), but
  `saturate(0)` is **not**. The box stays rgb(220,0,0), because `anyrender_vello_hybrid` 0.10
  `filters.rs` has `FilterEffect::ColorMatrix(_) => return None` (the conversion is commented
  out). That also covers grayscale, sepia, hue-rotate, brightness, contrast and invert. Only one
  filter node is supported in each case.

Fallback: `filter` stays banned by the lint. Material recipes must not rely on saturate or any
colour-matrix filter: the "vibrancy" saturation boost has to be precomputed into the tint colours.
`vello_hybrid` stays the default backend, where `filter: blur()` does work for in-surface effects,
but the design must not depend on it, because the CPU fallback as pinned does not blur.

### Cross-cutting notes for ds-native

- Every `DocumentConfig` needs a data: `NetProvider` (S7/S8) plus a `NetWaker` that requests a
  frame.
- Attribute selectors need the `*|` namespace prefix (S2); this is the most far-reaching finding.
- Headless offscreen rendering through vello_hybrid works: `wgpu_context::BufferRenderer`,
  `vello_hybrid::Renderer::render` to its texture view, then `copy_texture_to_buffer`. See
  `harness::shot_hybrid`. That gives `ds_native::snapshot` a GPU path as well as the CPU one.

## Wave 0 freeze: where the plan and the design docs disagreed (2026-09-24)

The design doc won each time. Recorded by the freeze agent; kept here so wave 1 fillers do
not re-litigate them.

- F1 Dependencies: `toml`, `dioxus-ssr` (dev) and, from the shell-host spike, `peniko` and
  blitz-paint's `custom-widget` feature were missing from the pinned block; promoted into
  `docs/workspace-deps.toml` and every workspace copy.
- F2 Settings file: `quire/appearance.toml` (design/22-SETTINGS.md §2), not the plan's
  appearance.json; mailo's json is imported once and left in place.
- F3 Lenient read: `ds_settings::lenient(text)` lays each key over the struct's default and
  keeps it only if it deserialises; the doc's per-field `deserialize_with` fallback would turn
  a bad key into the type's zero, not the key's default.
- F4 Motion types: `Motion` is the preference (five values, default System); `MotionLevel` is
  the resolved four (Calm, Standard, Extra, Reduced).
- F5 Accents: the docs never name six accents; frozen as Postmark plus the five Candy hues;
  "exactly four" means four properties per accent.
- F6 CardAccent: `{Postmark, SpaceHue}` per design/22-SETTINGS.md, default Postmark.
- F7 `blur_region()` dropped from ds; the host owns region geometry (design/03-COLOR.md §17.1).
- F8 `Avatar` the component vs the data type: the data type is `AvatarFace`.
- F9 `HoverTarget(key, …)`: dioxus reserves `key`; the prop is `hover_key`.
- F10 Peek: `PeekMode{Center, Full}` per design/04-COMPONENTS.md; mailo keeps `Side`.
- F11 Placement is a struct `{side, align, flip}`.
- F12 Chip variants follow design/04-COMPONENTS.md §10 (no Clip variant; O-6 stays open).
- F13 Token names the docs only bracket were filled in and marked proposed: intermediate z
  layers and type steps, eight radii, eight shadows, durations park/nudge/shake-long/sail/
  boat-return/spin/send-ring/curl-heavy, `--e-in-out`, `--danger-ink`, `--mark-ground`.
- F14 Fonts: mailo ships WOFF2; the wave 1 subset script must produce TTFs for fontique;
  `every_face_is_woff2` pins today's state and flips then.
- F15 Material keys for banners, control center and launcher live in sill's settings, not
  `AppearanceSettings` (design/22-SETTINGS.md §4.3).
- F16 Lint keeps both `SvgPaintInCss` and `CurrentColourOutsideStrokeFill`.
- F17 Two `Px` types: `ds::Px(f32)` for layout, `ds_settings::Px(u16)` for stored keys.
- F18 One unclamped `Fraction`; components clamp.
- F19 `Roster::leave` takes `(key, exit, emphasis)` so an unread row can exit heavier.
- F20 Material tint alpha is a settings key, so `recipe()` takes it as a parameter and the
  static stylesheet does not bake it in.
- F21 Components in no wave's file list (command_pill, account_tile, provider_mark,
  link_pill, selection_bubble, send_pill, space_editor, edge_strip, drag_ghost, sync_halo):
  assigned in wave 2.
- Signatures the freeze believes wrong, reported not changed: `SystemPrefs{scheme: Scheme}`
  cannot express the portal's "no preference" (mapped to Light for now); the plan's lint
  rule list lacked the three Blitz rules the spike forced (now added).

## W1 tokens (2026-09-24)

- Four translucent material tints fail the 4.5:1 ink legibility floor over an extreme ground:
  Bar dark over white 3.63, Dock dark over white 2.81, Widget light over black 4.04, Widget dark
  over white 2.43. `tests/legibility.rs::TRANSLUCENT_SHORT` pins them; retune the alphas in the
  gallery pass (03-COLOR open decision 11). The solid fallbacks all pass.
- Google's Bricolage Grotesque file names its family "Bricolage Grotesque 96pt ExtraBold";
  fontique registers by that name, so `--font-display` never matched. `scripts/subset-fonts.sh`
  rewrites name IDs 1 and 16 to the family the stylesheet uses. Why the tests missed it: no test
  rendered text with the face until the spike.
- Contracts consumers must follow: the root or `Surface` always writes `data-theme` (a nested
  `.ds` without one resets to light); `--m-tint-alpha` is written inline by the root from the
  settings key (default .8); `--d-heal` is the heal step; `--swatch-<accent>` feeds the picker.

## W1 integration (2026-09-24)

The five wave-1 branches merged; the amendments the fillers reported are applied. Recorded here:
what changed beyond the brief's list, and what the tests had missed.

- Material legibility: the four short translucent tints are raised in .02 steps to the first
  value that holds the card's ink at 4.5:1 over pure black and white: dark Bar .58 to .66, dark
  Dock .50 to .66, light Widget .50 to .54, dark Widget .45 to .65 (design/03-COLOR.md §17.2,
  still proposed). `tests/legibility.rs` is now a gate over every material, not a pin of the
  shortfall.
- `--t-flash` (1200 ms) follows the Reduced rule like every duration token (05-MOTION §3.2:
  60 ms), so under Reduced the mentioned-person ring shows for 60 ms. A hold is not motion;
  whether holds should be exempt is an open design question. `DelayToken::FlashHold` stays for a
  consumer that times the flash in Rust.
- The Person chip's `animation:chip-in` rule would have outranked the pulse class (equal
  specificity, components come after motion in the cascade), so the flash would never have
  played. The entry is now `.ds-chip[*|data-variant=person]:not(.a-chip-flash)`.
- `Count` keeps its last value in a plain cell rather than a `use_memo`/`use_pulse` signal pair:
  the bump is decided during render, and writing a signal there schedules a second render. The
  O-8 suppression across a Space switch is now "key the count by Space" (design/04 §14, O-8);
  the doc's `Motion::Quiet` hint had no type behind it.
- `Offence` gained `selector` (what an `Exception` matches); `lint::stylesheet` and
  `lint::markup` return only what no exception covers; `assert_clean` prints per-exception
  counts. `data-hover` joined the root attributes the lint calls `DsInternals`.
- The lint registry was a hand list and had drifted: it listed `--d-heal-step` (the table emits
  `--d-heal`) and lacked `--t-crumple-heavy`, every `--swatch-*`, `--m-tint-alpha`,
  `--handle-ring` and `--t-flash`; `ANIM_NAMES` was complete but would not have learnt `chip-flash`. Both are now
  derived from the tables, with a test that every custom property a `.ds` block declares is
  registered. Why the tests missed it: `self_lint` was ignored and no test compared the list with
  the stylesheet.
- Lint false positives the self-lint found once it ran, each fixed in the lint with a case in
  `tests/lint_rules.rs`: `font-family: var(--font-ui)` fired `FontFamily` (the passing case only
  tested an absent font-family); `transparent` fired `NamedColour`; a custom property named
  `--m-radius` fired `RawRadius` (its name ends in `-radius`) and so did a square corner `0`.
- Materials paint `box-shadow:var(--m-edge),var(--m-shadow)` (only the layers that are not
  `none`) instead of repeating the literals, so the colours live only in the `--m-*` tokens.
- Two stale golden checks: `tests/tokens.rs` did not know the component-inline `--f`, `--av-bg`,
  `--av-fg` (it ran before any component CSS existed), and the stylesheet golden predated the
  component sheets. Its re-bless differs from master in the component sections and in exactly
  the amendments above (tokens, materials, `chip-flash`, `.ds-ic`).
- `lint_rules` and `self_lint` now declare `required-features = ["lint"]`: ORCHESTRATION's plain
  `cargo test --workspace` could not compile them. The `ds::lint` doc example runs instead of
  being ignored.
- New gate: every control golden's markup lints clean against `ds::stylesheet()`
  (`tests/self_lint.rs`), with one exception: the avatar's computed colours and its `#fff`
  letter are inline hexes (O-7, O-3). A consumer rendering `Avatar` hits the same offence and
  needs the same exception until the letter gets a token and the hue a non-hex channel.
- Contract gap, not fixed: `Ds` does not yet write `--m-tint-alpha` inline (the W1 tokens note
  says the root does); the stylesheet's `var(--m-tint-alpha,.8)` fallback keeps the default
  correct, but `appearance.material_tint_alpha` has no effect until a later wave passes it.

## W2 integration (2026-09-24)

The four wave-2 branches merged; the amendments the fillers reported are applied. What changed,
and what the tests had missed.

- Merged tree: the stylesheet golden already matched `ds::stylesheet()` (the schema branch
  re-blessed last), so its re-bless on the merged tree was empty. Eleven list and overlay goldens
  differed, every one only by `--av-fg:#fff` becoming `--av-fg:var(--on-hue)` (the schema
  branch's `--on-hue`, rendered by components the other two branches blessed before it existed).
- Spinner: `.ds-spinner[data-kind=spin]` set `transform:scale(1)`, and `spin` is
  `to{ transform:rotate(360deg) }`. The two lists differ in function, so they interpolate as
  matrices, and both are the identity: the ring never turned. The spinning ring now has
  `transform:none`, which interpolates as `rotate(0)` to `rotate(360deg)`.
  `ds-native/tests/snapshot.rs::the_spinner_turns` renders it 275 ms apart and fails on the old
  rule. Why the tests missed it: the only moving snapshot showed the *breathing* ring, whose
  motion is opacity.
- Four `Anim`s for section 5 rows that play a catalogue keyframe at their own recipe:
  `PaletteFade` (`fade --t-quick --e-out`, row 7), `LinkPillIn` (`hc-in --t-quick --e-out`, row
  26), `BubblePop` (`menu-pop --t-quick --e-spring`, row 37), `PeekFullIn` (`peek-in --t-move
  --e-out`, row 64). The palette wrap, link pill, bubble and Full peek had borrowed a
  neighbour's recipe to satisfy `motion_drift`; they now play their own, the bubble and Full
  peek settle on theirs, and the settle table pins all four. No duration token was missing. The
  recipe table moved to `motion/recipe.rs` so `anim.rs` stays under 400 lines.
- Heavy exits: `exit_anim` gave only `Fold` a heavy variant, so an unread snooze or trash
  settled light while `Anim::{CurlHeavy, CrumpleHeavy}` went unused, and `list_row.css`
  followed the roster. Both now play the heavy variant of every row exit. `motion_machines`
  pinned the old mapping (a test that encoded the gap); it now pins the heavy one, and
  `components_lists::the_row_stylesheet_plays_what_the_roster_settles` reads the rule each
  leaving row matches and compares it with the recipe `RosterState::leave` returns, so the two
  cannot drift again. 05-MOTION open decision 3 is implemented; its 644 ms stays proposed.
- `Exit::TabOut`: a Today entry leaves as `Presence::Leaving(Exit::TabOut)`, and the roster
  settles `Anim::TabOut` for it (no heavy variant). `Exit::slug` is public and replaced the row's
  private copy.
- `TextInput{focus: Focus::OnMount}` (default `Manual`) calls `set_focus` from `onmounted` and
  writes `autofocus` for a webview; `SearchField` passes it through. The palette input and the
  bubble's link field use it (06-INTERACTIONS section 17). The harness case types one key with
  no click and sees it land only in the `OnMount` field.
- `DropState{Idle, Target, Source}` on `ListRow` and `SidebarItem` writes `data-drop=target` or
  `data-drag=source`. `drag_ghost.css` described a `.35` source rule that no stylesheet had;
  it is in `list_row.css` and `sidebar_item.css` now, with the section 34 drop-target look on
  rows as well as sidebar items. Why it was missed: no golden rendered a dragged source.
- `BubbleAction` is an enum, `Button(BubbleButton)` or `Separator`, so `ds-bubble-sep` renders.
  `AccountFace::One` gained `address: Option<String>`, which is the tile's `aria-label` when
  given (section 27); the face is no longer `Copy`. `SpaceEditor` gained `name` (the title reads
  "{name} Space") and `on_active_dot: Option<EventHandler<ActiveDot>>` (`ActiveDot` is
  `DotIndex`), reported by every pick, add, remove and preset through one `Picker` rather than a
  signal and prop threaded to each part; a harness case clicks the second stop and sees it
  reported. `SideState::slug` is public, tested against the edge strip's selectors. These are
  breaking for any caller built against the wave-2 signatures (`BubbleAction`, `AccountFace`).
- `Ds{tint_alpha: Option<Alpha>}` replaces the root `Signal<Alpha>` context the overlays branch
  introduced; `ds_settings::Environment::tint_alpha` turns `appearance.material_tint_alpha`
  (percent) into thousandths, with a compiled example of the wiring on that method. This closes
  the W1 integration "contract gap": the settings key now reaches the root.
- Rect reads could panic on dioxus-native with "RefCell already borrowed". dioxus-core polls a
  task inside `render_immediate` when it woke in the same turn as a dirty scope, and
  dioxus-native-dom's writer holds the document mutably there; its `get_client_rect` borrows
  the document mutably too. The hover-card harness case hit it on both runs before the fix (the
  backtrace is the card's `RectProbe` read, polled from `render_immediate`). The same `poll` path drives a real
  window (blitz-shell `window.rs`), so this was not a harness artefact. ds now reads every rect
  through `geometry::measure::client_rect`, which asks the host's `HostMeasure` first; ds-native
  provides one (in `Host` and in the headless document) that reads through
  `NodeHandle::try_doc` and answers `Measured::Busy`, and the reader waits a frame. Worth
  reporting upstream: `get_client_rect` needs only a shared borrow.
- The four plan harness cases run un-ignored. Their selectors were already the real class
  names; the row case advanced 900 ms, past the heal as well as the fold (454 + 284 ms at
  Standard), and now looks at 500 ms. The only `#[ignore]` left is the live portal test.
- `FrameVars::names` is used only by the lint registry, so `cargo test -p ds-native` (which
  builds `ds` without `lint`) warned; it is compiled only with the feature.
- 04-COMPONENTS: the prop signatures of the amended components are updated in place. No open
  decision is settled outright; O-15 is marked partly settled (focus on mount exists; Peek and
  Sheet still neither move nor trap focus).

## Fix wave: the two gaps CONSUMING.md §9 named, closed (2026-09-24)

Both gaps were `examples/consumer` findings the W3 adoption wave reported but did not fix
(CONSUMING.md §9 documented one as an open bug with a reviewed `Exception`, the other as a
workaround in §3). This wave fixes both, with tests, and removes the workaround text.

- **S2, frame layers**: `FrameLayers::render` (`crates/ds/src/root/ds.rs`) wrote the hidden
  frame layer's class as `"ds-layer back"`, a second CSS class; `crates/ds/src/css/
  utilities.css`'s `.ds-layer[*|data-layer=back]{opacity:0}` is an attribute selector (the `*|`
  namespace prefix S2 forces on every attribute selector in a Blitz consumer's CSS, this one
  included). The class was never `back`-only-in-name matched by that rule, so the hidden layer
  never got `opacity:0` and a Space switch's cross-fade (design/21-SPACES.md section 5) had
  nothing to fade from. Fixed by writing `data-layer="back"` on the hidden layer and no
  `data-layer` at all on the front one, matching how every other root attribute
  (`data-theme`, …) is written and how the lint's `DsInternals` rule already expects a
  `.ds`-scoped attribute rather than a bespoke class. Tests: `crates/ds/tests/root_ssr.rs::
  the_hidden_frame_layer_carries_the_back_attribute` (the attribute lands, the old class form is
  gone) and `::a_window_roots_markup_lints_clean_of_unstyled_classes` (`ds::lint::markup` over a
  `Material::Window` root reports no `Rule::UnstyledClass`, run under the `lint` feature).
  `examples/consumer/tests/coherence.rs`'s `KNOWN_GAPS` `Exception` is removed, and
  CONSUMING.md §9 now records the gap as fixed rather than open.
- **Runtime, `use_environment` under Blitz**: `ds_settings::use_environment` spawns two tasks
  with `tokio::spawn` (the portal watch, over zbus's `tokio` feature, and the file-watch
  debounce in `ds_settings::watch`), which panics ("there is no reactor running") unless a
  runtime is entered on the calling thread. `ds_native::launch` and `ds_native::Harness` never
  entered one, so CONSUMING.md §3 documented this as a real gap with a one-shot,
  synchronous-load workaround. Neither `ds` nor `ds-settings` may depend on a renderer or
  windowing stack (`scripts/check-boundary.sh`), so neither can own a host thread to enter a
  runtime on; `ds-native` is quire's one host crate, so it now owns a process-wide, lazily built
  Tokio runtime (multi-thread, two worker threads — `crates/ds-native/src/runtime.rs`).
  `ds_native::launch` enters it and holds the guard for the rest of its call (which blocks until
  the window closes, i.e. for the process's life); `ds_native::Harness` enters it in
  `Harness::new`, before the app's first render, and holds the guard as a struct field, for the
  harness's own life. `scripts/check-boundary.sh` still forbids `tokio` to `ds`; it has no rule
  for `ds-native`, which is exactly the "ds-native may reach tokio" shape this fix needed.
  Test: `crates/ds-native/tests/harness.rs::use_environment_does_not_panic_under_the_harness`
  renders a `Ds` root whose component calls `use_environment(AppName("consumer-test"))` and
  asserts it renders past its first frame instead of panicking (the portal may be absent or
  answer; both are fine, matching CONSUMING.md's own wording for this case).
  `examples/consumer::App` was switched from a one-shot `ds_settings::load` to
  `use_environment`, and CONSUMING.md §3 no longer tells a Blitz consumer to enter their own
  runtime or fall back to a synchronous load.
## Gallery fixes B (2026-09-24)

The API gaps the gallery and `examples/consumer` exposed, closed on branch `fix-gallery-b`.

- Markup lint: a `style` attribute is now checked declaration by declaration
  (`lint/inline_style.rs`). A custom property (`--*`) written inline on an element carrying
  `ds` or a `ds-*` class is quire's computed value handed to its own stylesheet (the `--f-*`
  frame and `--f-grad`, `--av-bg`, `--pc`) and is not an offence; a literal colour in a
  non-custom property stays one on any element, and a custom property on a consumer's element
  stays one. An `<svg>` is quire's when it is `.ds-ic` or carries `data-ds-svg`; the SendPill's
  ring now writes `data-ds-svg="ring"` (a one-attribute change to `send_pill.rs`, whose three
  goldens re-blessed by exactly that attribute).
- Exceptions removed: seven of the gallery's thirteen (`div.ds` HexColour and ColourFunction,
  `div.ds-layer` and `div.ds-layer.back` HexColour, `span.ds-avatar`, `span.ds-provider`,
  `svg.ds-send-ring`) and `self_lint`'s avatar markup exception. Six remain, each still
  suppressing something (the gallery gained `every_exception_still_suppresses_something`): the
  back layer's unstyled class (the `root/ds.rs` bug, fixed on the sibling branch, which will make
  that exception stale and fail the new test until it is removed), and five Space-editor/SpaceDot
  elements that paint a literal gradient or colour into `background` rather than a custom
  property. The consumer's one exception is the same back-layer class; nothing to remove there.
- `Surface` takes `accent: Option<Accent>` and `blur: Option<BlurState>` beside `theme`; each
  `None` inherits. The `Env` its children read carries the overrides (`tests/surface.rs`, four
  goldens under `snapshots/root/surface/`).
- Spacing tokens: `SpacingToken` (`--s-1` … `--s-36`, the eighteen common steps of 01-LAYOUT §2,
  each named by its pixel value), emitted on `.ds` only and registered in the lint registry from
  the table. `Rule::RawSpacing` (Strict profile) flags a literal `px` in `margin`, `padding`,
  their sides and logical forms, and the three gaps; `0`, `auto`, `%`, `em` and `var()` pass.
  quire's own component sheets still use the literal pixels 04-COMPONENTS quotes from `S`, so
  `self_lint` sets `RawSpacing` aside; converting them to `--s-*` is a follow-up. The gallery's
  `gallery.css` has 43 raw spacings, three of them (20, 28, 40) off the scale; its strict lint
  test sets the rule aside too, also a follow-up. `examples/consumer/src/style.css` had three,
  all on the scale, and now reads `--s-12`, `--s-16`, `--s-8`, so the example passes Strict in
  full as CONSUMING.md tells a consumer to.
- `HoverCard{parts: Vec<HoverCardPart>}`: `Title`, `Sub`, `Person{initial, tone, title, sub}`,
  `Stats(Vec<HoverStat>)`, `Flag{tone: FlagTone, icon, text}`, `Messages(Vec<HoverMessage>)`,
  `Foot{text, keys: Option<KeyHint>}`, `Actions(Vec<Element>)`, drawn in order before children.
  The flag takes its glyph because the icon catalogue has no info glyph. A golden per part, and
  a sender card from parts is byte-identical to the hand-written `sender-open` golden (tested).
- `ToastHub::push_undoable(text, token, on_undo: EventHandler<UndoToken>)`: the undo calls the
  handler of the push on screen; a later push replaces it. `last_undo()` stays. The handler
  belongs to the scope that created it, so it must outlive the toast (documented).
- Bug found by the new toast unit test, fixed: `ToastHub::stop_hold` ran
  `if let Some(t) = *self.hold.peek() { … self.hold.set(None) }`; the peek's read guard lives
  through the `if let` body, so the `set` panicked "already borrowed" whenever a hold was
  running: every undo or hide, and every second push, within 5.2 s of a push. Why no test caught
  it: no test pushed and then undid or re-pushed inside a render context that surfaced the panic.
- `Button` and `IconButton` take `mounted: Option<EventHandler<MountedEvent>>` (no attribute;
  goldens unchanged). `crates/ds-native/tests/anchor.rs` anchors a Slim menu to a button's
  handle through `Anchor::Mounted` and finds it 6 below and 8 left of the button. Two harness
  observations while writing it, neither fixed here: the overlay bounds are the `.ds` box, which
  is as tall as its content, so in a content-high root a menu under a button flips and clamps to
  the top (the test adds a spacer); and with nothing but the menu's `if` placeholder after the
  Button in its component, the harness's click never reached the button, while any element
  after it makes the click land (the test has a caption after the button). The second looks like
  a hit-testing issue in the harness or dioxus-native; worth a minimal reproduction.
- Not changed, for whoever merges: the gallery's Gaps page (`pages/gaps.rs`) still lists the
  spacing, Surface, HoverCard, toast, avatar and Button gaps this branch closes;
  `examples/consumer` still anchors its menu through a wrapper span and could pass `mounted`;
  CONSUMING.md §4 and §5 describe `Surface` and `RawMarkup` as they were.
## Gallery fixes A (2026-09-24)

The component bugs the gallery's contact sheets exposed, each with its cause, its fix and its
proof. Headless proofs are in `crates/ds-native/tests/gallery_fixes.rs`; each fails on the
pre-fix tree (checked by restoring the old sheets) and passes now.

- Two root causes explain most of the list:
  - **A class collision.** TextInput's placeholder wrapper was `.ds-field`, which is also the
    Space editor's colour field. The field's `height:176px`, border and radius landed on every
    TextInput wrapper, and TextInput's `display:inline-flex` landed on the colour field. That
    collapsed the field to its 1 px border (plane and handles clipped to nothing). The wrapper is
    now `.ds-input-wrap`; `.ds-field` is the editor's alone, as design/04 section 32 names it.
  - **The reset outranks a lone class.** `.ds button,.ds input{font:inherit;color:inherit}` is
    (0,1,1), so a component rule written as one class (0,1,0) loses its `color` and `font-*` on a
    `button` or `input`. The toast's tab and the send pill's Undo took the pill's `--paper` on their
    own `--paper` ground, which is why their labels were invisible. The sidebar item lost its 13.5 / 600 and
    `--f-ink-soft`; the header action, the stop's remove button and the row's star lost their
    colours; the input lost 13.5 px. Every rule in this branch's files that styles a button or input now
    outranks the reset (a parent class, `button.` or `.ds-input-wrap input`). **Not fixed at the
    source:** `css/reset.css` is outside this branch; writing it as `:where(.ds) button, …` (specificity 0,0,1)
    would fix every component at once, including ones this branch does not own (for example
    `selection_bubble`, `menu_entry` and `tabs`, if they style a button with one class). Recommended.
- 1. TextInput was 176 px tall: the collision above, and Blitz gives an `input` the
  300 x 150 replaced-element default because nothing sized it. The input is `width:100%` of its
  wrapper, and its height is `calc(1.55em + 16px)` boxed and `calc(1.55em + 8px)` inline (the
  13.5 px line at the inherited 1.55, plus padding and border). The height is in `em`, so SearchField's 16 px (and
  the bubble's 12.5) stay one line. SearchField's rules now name the variant, so they outrank
  TextInput's (its padding 0 had been losing). Proof: 36.9, 28.9 and 24.8 px, each within 2 px.
- 2. The hidden toast was laid out in every root. `ToastHost` now draws nothing while the
  hub is empty. A push mounts it `data-shown=hidden` for one frame (`FRAME_SLACK`), so the
  spring rises from `translateY(160%)` (the harness's 5200 ms toast case now sees `hidden` on the
  click's frame, `shown` 100 ms later, and nothing once it has sunk). A hide slides it back and drops it after `--t-big` plus
  a frame. The stage machine is a pure `Stage::next` with a table test. Proofs: an SSR test
  (no `.ds-toast` in an empty root; hidden on the first frame, shown after, gone after the hold
  and the sink) and a pixel probe (the bottom of an empty root is one flat colour).
  Many overlay goldens lost the trailing empty toast.
- 3. The toast tab and SendPill Undo labels: the reset specificity above. Proof: the goldens
  carry "Undo", and a pixel probe finds the label's ink on the tab and on Undo (0 pixels before).
- 4. LinkPill: the stylesheet already played `hc-in --t-quick --e-out` (W2 integration); the
  gallery's gap line is stale. The pill now reports `use_entrance(Anim::LinkPillIn)` as
  `data-presence`. Unit tests pin its rule to the recipe, check that it differs from the hover
  card's, and pin the doc's colours (ink/paper, lying `--danger`/`--danger-ink`).
- 5. SectionHeader: the eyebrow is `.ds-section-header-text` (nowrap). The rule is already a real
  span (O-22). The header is `width:100%; grid-column:1 / -1`, so in a grid it gets a row of its own.
  Proof: four headers in a 4-column grid are each 400 px wide and stacked.
- 6. SidebarItem: Blitz's user-agent `button{justify-content:center}` centred an item without a
  count. The item now sets `justify-content:flex-start`, and the label is the flexible part
  (`flex:1 1 0`), so a truncating label's fade falls on its empty end. Proof: the label starts
  33 px in, with and without a count.
- 7. ListRow: `NameFit::of(name, NAME_BUDGET)` uses `clip_chars`'s count. `.ds-truncate` goes on
  the name only when it is `Overflowing`. design/02 gives no column budget. `NAME_BUDGET` = 26 is
  derived and documented in the code: 980 px minimum window, about 196 px of name column, about
  7.4 px per character at ui 13.5 / 700. A name that fits but is unusually wide is hard-clipped
  (`overflow:hidden`) rather than faded. Proof: a table test and the `name-overflowing` golden
  beside `bare`.
- 8. SpaceEditor: the field was blank because of the collision (not a late image: the
  harness's frame loop already goes round while a fetch lands, spike S7). `snapshot_at` did
  jump straight to CSS time without letting wall-clock timers run, so once the toast rose on a
  one-frame wait the gallery's posed toast never showed. It now lets 120 ms pass first
  (`MOUNT_SETTLE`: late fetches and the `FRAME_SLACK` mount waits land); each moment's CSS time
  is unchanged. Presets are 22 px discs (`repeat(8, 22px)`); `aspect-ratio:1` in a wide
  panel made them 140 px. Proof: the field is 176 px tall, its dots cover more than 5 % of
  it, its centre is not `--paper`, a handle sits on it, and a preset is 22 x 22.
- Found, not fixed (another branch's file): `ToastHub::stop_hold` writes `hold` inside
  `if let Some(running) = *self.hold.peek()`. In edition 2024 the peek guard lives through the
  block, so `hide()` or `undo()` while the hold runs panics with `AlreadyBorrowed`. Any undo
  from the pull tab hits it. The SSR sink test waits out the hold instead. Fix: copy the value
  out first (`let running = *self.hold.peek(); if let Some(running) = running { … }`).
- The gallery's "What quire does not draw yet" list (`ds-gallery/src/pages/gaps.rs`) still
  names the TextInput, toast, tab, SidebarItem, Space editor and LinkPill items; it is outside
  this branch and should drop them.

## Sill gaps (2026-09-24)

sill's interface freeze reported five gaps against quire (sill FINDINGS Q1-Q5). All five are
closed on branch `sill-gaps`; sill adopts the APIs in a later wave and can then delete its
copies (`bar/menu_track/*`, `sill_services::curve`, most of `sill-settings/src/{file,watch}.rs`).

- **Q1 Menu tracking.** `ds::MenuTrack<K>` (`crates/ds/src/overlay/menu_track.rs` and
  `menu_track/`) is sill's `bar/menu_track` machine with sill's tests. Changes in the port:
  - It is generic over the caller's menu key `K`, not a `MenuId(u32)`.
  - The timings live in the machine (`MenuTiming`, built from `menus.*`), so the signature is
    `step(self, event, now)`, the same shape as `HoverIntent`.
  - Time is `std::time::Instant`, not sill's `Stamp`, and points are `ds::Point` (f32).
  - The session is one struct (`Session<K>`), not seven loose fields passed to every helper.
  - Names that would collide at the crate root were renamed: `Anim` became `MenuAnim`
    (`Pop`/`Fade`), `Side` became `MenuDirection`, `Pick` became `Pickable`, `Sub` became
    `Submenu`, `Guard` became `SafeTriangle`.

  One deviation from the brief: `step` returns `Vec<MenuTrackEffect<K>>`, not a single effect.
  One event can need several effects in order: `Close` then `Pick`, or `Highlight` then
  `CloseSub` then `RequestTick`. A single effect would need a combined variant for every
  pair. The safe triangle's hysteresis is design/13 §13.5's rule: the corners are inflated
  4 px and the apex follows the pointer.
- **Q2 Curves.** `CubicBezier::at(Fraction) -> Fraction`, `Easing::curve()` and `Easing::at`
  (`crates/ds/src/motion/curve.rs`). It is integer only (i128). The curve parameter is found by
  bisection on x in 1/2^24 steps, then y is read there and rounded to the thousandth. Plain
  bisection replaced Newton: with exact integers it is deterministic, and it needs no guard
  against a zero slope. The test table was computed with exact rational bisection
  (Python `fractions`).
- **Q3 Spaces store.** `ds::SpaceStore { version, by_id, by_index: Vec<Option<SpaceLook>>,
  extra }` with `look_for`, `look_for_workspace` and `with_look`
  (`crates/ds/src/space/store.rs`). It follows design/21 §10's schema, which has both `by_id`
  and `by_index`. `SpaceDefaults` carries the two settings keys the preset fallback needs.
  A malformed `by_index` entry costs only its own position. The I/O is `ds_settings::SPACES`
  (`crates/ds-settings/src/spaces.rs`).
- **Q4 Generic settings I/O.** `ds_settings::file::{FileName, Format, SettingsFile,
  Settings<T>, load, save}` and `watch::{FileWatch<T>, watch_file}`. `appearance.toml` is
  `APPEARANCE` and `spaces.json` is `SPACES`. JSON gets the same key-by-key lenient reader
  (`lenient_json`). The old names still mean `appearance.toml`: `ds_settings::{load, save,
  watch, load_or_import, FILE_NAME, AppearanceWatch}`, and `AppearanceWatch` is now
  `FileWatch<AppearanceFile>`. The appearance-specific code moved to `appearance_file.rs`, and
  `file::load_or_import` is re-exported from there. `SettingsError` gained `EncodeJson`.
- **Q5 Schema derive.** Three changes:
  - Text is recognised by type (`String`, `PathBuf`, `Cow<str>`, any path) or by
    `#[settings(text)]`. `text` together with `range` is an error.
  - A one-variant enum is `KeyKind::Fixed { variant }`, drawn as the new `Widget::Readout`.
    Zero variants is still an error. sill can now drop `PanelMaterial::Popover` and
    `SpaceLookLookup::ByIndexOnly` if it wants.
  - A known numeric type (the primitives and ds-settings' unit newtypes) without `range` is a
    compile error that starts `MissingRange { field: <name> }`. A caller's own numeric newtype
    cannot be recognised from tokens alone. For that case, `SchemaVariants` now carries a
    `#[diagnostic::on_unimplemented]` message that names the fix (range, text, or the derive),
    so the error no longer just names the trait.

  There is no `trybuild` in the lockfile. The refusals are unit-tested on the expansion itself
  (`gen_struct.rs` and `gen_enum.rs` tests), and the accepted shapes are tested through the
  real derive in `crates/ds-settings/tests/derive.rs`.
## Polish pass (2026-09-24)

What the two gallery fix branches left behind, closed on branch `polish`. Headless proofs are in
`crates/ds-native/tests/polish.rs` and `tests/click.rs`; each probe in `polish.rs` except the
Danger one was checked to fail with its fix reverted (the Danger probe pins behaviour the doc
already asked for).

- **`:where` on Blitz.** `reset.css`'s element rules now read `:where(.ds) h1`, `:where(.ds) p`,
  `:where(.ds) button, :where(.ds) input, …` (specificity 0,0,1). Stylo at the pinned rev
  honours `:where`: a plain `button` under `.ds` still inherits the column's colour and 30 px
  font through the reset (so the rule is parsed and matched, not dropped), and a
  `.ds-where-probe{color;font-size}` rule with one class wins over it on a `button` (red and
  24 px, none of the inherited blue). Written as `.ds button` the same probe loses its colour
  (checked). Three per-component workarounds from Gallery fixes A were removed because S's own
  CSS writes a lone class there: `.ds-row .ds-star` is `.ds-star` again, `button.ds-sidebar-item`
  is gone, and `.ds-input-wrap input`'s font and colour moved back onto `.ds-input`. The ones
  whose parent class S's CSS itself writes stay, with the S selector now cited as the reason
  instead of the reset: `.ds-toast .ds-toast-tab` (`.toast .tab`), `.ds-send-pill
  .ds-send-pill-undo` (`.sendpill button`), `.ds-section-header .ds-section-header-action`
  (`.s-h button`), `.ds-sidebar-item .ds-sidebar-item-close` (`.item .x`) and `.ds-stop
  .ds-stop-remove` (`.stop .rm`). `self_lint` recognises `:where(.ds)` as quire's own ground for
  `DsInternals`. Only the stylesheet golden changed for this.
- **Spacing.** 218 literal pixels in `margin`/`padding`/`gap` became `var(--s-N)`: 175 in 29
  component sheets (utilities.css had none) and 43 in `gallery.css`. Negative margins (the
  Space handle, the slider thumb, the avatar stack) are `calc(var(--s-N) * -1)`. Four component
  values were off the scale and quoted exactly by 04-COMPONENTS, so they became steps:
  `--s-1-5` (the chip's `1.5px 7px`, section 10, and the image provider mark's `1.5px`, section
  28), `--s-13` (the hover card's `12px 13px`, section 22) and `--s-15` (the toast's
  `5px 5px 5px 15px`, section 23); `SpacingToken` gained `S1Half`, `S13`, `S15` and its `px()`
  became `tenths()`, and design/01 §2 records the addition. The gallery's three off-scale
  lengths took the step below: 20 to 18 (`.g-card` top), 28 to 26 (`.g-card` bottom), 40 to 36
  (`.g-app` bottom), so the gallery card is 2 px shorter at the top and bottom and the page
  4 px shorter at its foot; nothing else moves. `Rule::RawSpacing` now runs in `self_lint` and
  in the gallery's lint test, with no set-aside and no exception.
- **Space editor dots.** The field was one 540 x 352 PNG at `background-size:100% 100%`, so a
  panel wider than S's stretched every dot into an ellipse. It is now two layers: a 60 x 39 hue x chroma colour plane stretched to the field (a
  smooth gradient, so stretching is harmless) and, over it, one 18 x 18 RGBA cell of the ground
  with a round hole of radius 5.2 at its centre, tiled at `background-size:9px 9px` (S's 18 px
  canvas cell at the half scale its 176 px field draws it). Every dot is round at any width and
  shows the colour of its own place. `png.rs` writes RGBA too. Proof: a dot's width equals its
  height within 1 device px at three places in a 280 px and a 560 px panel (with the tile
  stretched to 16 x 9 px it measures 18 x 8 and fails); SSR tests decode both images. The five Space editor goldens gained the `.ds-field-dots` element.
- **Three defects the regenerated sheets showed.** (a) Button Danger looking like Mini at rest
  is the design: 04-COMPONENTS section 1 says Danger is "at rest as Mini; red only on hover".
  Its hover rule is in place and wins; the new probe checks rest fills are equal and the hovered
  Danger fill is red and differs from hovered Mini. Not changed. (b) Quiet had no layout for an
  icon (S's link-button has none), so the glyph touched the label: it is now
  `inline-flex; align-items:center; gap:var(--s-6)` like every other variant (probe: 6 px).
  (c) A section header's action sat against the eyebrow when the kind draws no rule (Menu):
  the action now has `margin-left:auto`, so it ends the row in all four kinds (probe: its right
  edge is the header's content edge for Frame, Group, Field and Menu).
- **The click observation (Gallery fixes B).** Reproduced and explained, not fixed: it is not the
  `if` placeholder. A Button is `inline-flex`, an atomic inline; when its parent holds only
  inline content (the Button alone, the Button and a placeholder, or the Button beside inline
  text) the parent is an inline formatting context, and blitz-dom's `Node::hit` returns the
  parent rather than the Button, so the click goes to the parent. A block after the Button (the
  anchor test's caption) or a flex parent gives the Button a box of its own and the click lands.
  `crates/ds-native/tests/click.rs` pins both working shapes and keeps the failing one as an
  `#[ignore]`d reproduction. The hit test is blitz-dom's (the same one a real window uses);
  ds-native neither hit-tests nor routes clicks, so there is nothing in quire to fix. Every
  quire container is a flex row, and CONSUMING.md §8 tells a consumer to put a lone button in
  one. Upstream report material: the reproduction test.
- Gaps page: every item the fix branches closed is gone; three remain, each naming its owner
  here or in Gallery fixes B: the gallery's own wallpaper/stage/grid layout (no quire token or
  component for them; open, owned here), the overlay bounds being the content-high `.ds` box
  (Gallery fixes B), and the inline-context click above.
- `examples/consumer` anchors its menu through `Button { mounted }` and `Anchor::Mounted`; the
  wrapper span and `use_rect` are gone. CONSUMING.md §2 (idle ToastHost), §4 (Surface's accent
  and blur), §5 (RawSpacing under Strict, the inline-style rules, `data-ds-svg`), §6 (the
  Overlays example and the Gallery fixes B props), §8 (the click caveat), §9 and §10 (the gallery
  exists) were brought up to date, and `docs/mailo-migration.md` gained "The quire APIs this
  brief assumes" with the anchors the mailo session should read.
## Tray gaps (2026-09-24)

sill's tray work (sill FINDINGS F29-F34) reported three gaps against quire, Q6-Q8. All three are
closed on branch `tray-gaps`; sill adopts the APIs in its next wave. Headless proofs are in
`crates/ds-native/tests/tray_gaps.rs`.

- **Q6 External icons.** `ds::IconSource::{Glyph(Icon), Symbolic(ExternalIcon), Image(ExternalIcon)}`
  with `ExternalIcon { url: IconUrl, size: IconSize }` (`crates/ds/src/icon/external.rs`) and
  `ds::IconView { source, size }` (`components/icon_view.rs`). `IconUrl` is parsed once: `data:`
  and `file:` only (anything else is `DsError::IconScheme`, the crate's first error enum,
  `crates/ds/src/error.rs`), with `"`, `\` and control characters percent-encoded so the URL
  cannot leave its quoted CSS string; `IconUrl::png(&bytes)` does the base64 (sill can drop its
  hand-rolled encoder), `IconUrl::svg` percent-encodes as spike S7 did, `IconUrl::file` takes an
  absolute path. A symbolic icon is `span.ds-ext-icon[data-kind=symbolic]` with an inline
  `mask-image:url(...)` over `background-color:currentColor`; an image is `[data-kind=image]`
  with the URL as `background-image`. The size is the inline `--ic-size`. An external icon is
  drawn at its own `size` (the size its caller resolved it for); a glyph still takes the slot's
  size. Blitz facts probed here: a PNG (not only an SVG) works as `mask-image`, its alpha is what
  masks, and `mask-size`/`background-size:100% 100%` stretch it; the pixel proof renders a 16 x
  16 one-bit PNG mask at 18 px inside an element coloured `--ink` (its opaque middle matches an
  `--ink` swatch, its transparent ring matches the ground) and an opaque red RGB PNG in the same
  host (it stays red). `self_lint` carries one exception for `currentColor` in the symbolic rule
  (the rule allows it only in `stroke`/`fill`; here it is the Glyph's stroke by another path), and
  `--ic-size` joins the inline variables. `IconButton { icon }` is `#[props(into)] IconSource`, so
  `icon: Icon::Star` compiles; `Button { icon }` is `Option<IconSource>`, and two `SuperFrom`
  impls with a local marker keep `icon: Icon::Send` and `icon: Some(Icon::Archive)` compiling
  (plain `From` is not possible: `Option` is foreign).
- **Q7 Submenus and disabled items.** `MenuEntry::Item` gained `availability: Availability`, and
  `MenuEntry::Submenu { title, tile, availability, children }` is a new variant (a parent row
  picks nothing, so it carries no value or check). A disabled row has `aria-disabled="true"`,
  opacity .35 (design/13 §13.3.3; the brief said "faint colour", the doc says .35 opacity, and
  the doc won), no click, and Up/Down skip it (`menu_lines::moved_live`; the palette clamps the
  same way). A parent row shows the 12 px chevron and `aria-haspopup`/`aria-expanded`. Every
  panel (the menu and each submenu) owns one `MenuTrack<()>` for its own children, so submenus
  nest to any depth with the one machine: a pointer move over a row is `Move(at, Item {..})` (the
  200 ms rest, the safe triangle and its 300 ms timeout are the machine's), a move inside the
  submenu is reported to the parent as `Move(at, Menu)` so the parent's highlight and triangle
  hold, the landed submenu's near edge is `SubPlaced`, and Left/Escape are `Key(Left)`. The
  machine gained two events for the keyboard, which it had no way to express: `Select(path)`
  (Up/Down moved the highlight: another item's submenu closes, nothing opens) and
  `Expand(path)` (Right, Enter or a click: open now), and a constructor `MenuTrack::open(timing,
  key)` for a menu that is already open when its tracker exists (click mode). A submenu is placed
  through `place()` against the parent panel's width at the parent row's top less the panel
  padding (5, or 6 for Dropdown), `Side::Right`, gap 2 (the doc says gap 2, not an overlap), so
  it flips left at the edge. It floats on the menu layer without joining the layer stack (the
  menu takes Escape and the outside click; the submenu closes with it), opens with no entrance
  (`[data-depth]`), and takes the focus only when the keyboard opened it. `Menu` gained
  `timing: MenuTiming` (the caller reads `menus.submenu_delay_ms` into it) and `expanded:
  Option<usize>` (open a choice's submenu on mount; the posed gallery uses it). One overlay-host
  change was needed: `Overlays::show` now replaces an entry in place, so a menu that re-renders
  keeps its place under its submenu (it used to move to the end of its layer and cover the
  submenu with its outside-click catcher). `menu.rs` split into `menu_lines` (pure lists and
  navigation), `menu_keys` (the keyboard as one table), `menu_rows` (drawing), `menu_tracker`
  (the machine's effects) and `menu_panel` (what the menu and `SubMenu` share); the fuzzy matcher
  moved to `menu_match`. SSR goldens cover the closed states; the open submenu needs layout, so
  it is proved headless: a rest opens it after the delay and not at 120 ms, beside the menu 2 px
  off and 5 px above the row; Left closes it; Right opens it at once and focuses it, where Down
  and Enter pick a child; Escape in it closes one level; Down skips a disabled row and a click on
  it picks nothing.
- **Q8 Pointer buttons and ids.** `Button` and `IconButton` take `id: Option<String>` (written as
  `id`; absent, the markup is unchanged) and their `onclick` is `EventHandler<ds::Press>`,
  `Press { button: PointerButton::{Primary, Secondary, Middle}, modifiers }`. Blitz sends a
  right-click as `contextmenu` and never as `click`, and the middle button as `mouseup` only
  (blitz-dom `handle_pointerup`), so the controls listen to all three: `click` (a keyboard
  activation has no trigger button and reports `Primary`), `contextmenu` (default prevented,
  `Secondary`) and `mouseup` with the auxiliary button (`Middle`). A closure `move |_| …`
  compiles as before; an `EventHandler<()>` value converts through a `SuperFrom` impl (sill's
  `TrayButtonView` passes its own `onclick: EventHandler<()>` straight through, which keeps
  working); a closure written `move |()| …` does not, and cannot be made to (a blanket impl over
  closures would break coherence), so quire's one such site (Peek's close) became `|_|`. The
  harness gained `Harness::press(at, PointerButton)`. Not included: the pointer position, which
  the brief's `Press` does not carry (SNI's `x, y` are screen coordinates a client cannot know
  anyway).
- Left for sill: choosing `Symbolic` or `Image` per item (design/08 §1.5 steps 2-3, the chroma
  test and `--warn` for `NeedsAttention`), and sizing its tray popup to leave room for a submenu,
  which is drawn in the same document as its menu.
## Bar gaps (2026-09-24)

sill's bar (sill FINDINGS "M1 bar-ui", F44-F53) reported Q9-Q13, G7 and the Q12 status-item
gap against quire. Closed on branch `bar-gaps`; headless proofs in
`crates/ds-native/tests/bar_frame.rs` and `bar_menu.rs`, SSR goldens under
`crates/ds/tests/snapshots/root/chrome/`, `controls/icon_button/status*.html` and
`overlays/menu/bar-status-lines.html`.

- **Q10 HostMeasure.** `ds-native`'s measurer is public: `ds_native::measure::MEASURE` and
  `ds_native::measure::provide()` (a hook that provides it). Without a measurer, `ds`'s read
  no longer panics: dioxus-native-dom's `get_client_rect` borrows its document on the first
  poll, not on the call, so the guard wraps the poll (`geometry/measure.rs`'s `Guarded`): a
  panic there ends the read as `Measured::Busy` and the caller waits a frame. `ds` cannot name
  Blitz's `NodeHandle`, so `try_borrow` is not reachable from it; the guard is the closest
  equivalent, and the default panic hook still prints the collision, which is why hosts should
  provide the measurer. Unit test with a held `RefCell`; harness test of `provide()`.
- **Q9 Frame layers on chrome.** `root/chrome.rs` holds three rules, each with an override prop
  on `Ds`: `RootChrome::of(material)`, `FrameTint::of(material, chrome)`, `Ground::of(material)`.
  A Bar, Dock, painted Popover, Osd or Widget root stamps `data-frame="tinted"` and draws the
  window's two layers and grain inside one `.ds-frame` whose opacity is `--m-frame-alpha` (each
  material's own tint alpha scaled by the settings key, the same arithmetic as `--m-tint`) with
  blur and .94 without. The group's own background is the current gradient, so inside it stays
  opaque through a cross-fade and only the group's alpha lets the blur through. Sheet and Toast
  keep their flat tint (design/21 §3's "no" rows); OSD takes the gradient as the brief asked,
  where design/21 §3 had proposed "no" (recorded there, the user's call). Two Blitz facts: the
  tinted root must be a stacking context (`position:relative; z-index:var(--z-raise)`), or the
  frame's negative z-index paints under whatever is behind the root (the gallery's wallpaper
  hid it entirely on the first sheet); and the root's inset edge (the bar's hairline) is then
  under the frame, so `.ds-frame::after` draws `--m-edge` again. Proofs: the bar's corner at
  rest is the first stop at .70 over white (blur on) and at .94 (off), within 3 of 255, and not
  the old surface tint; a look change (preset 1 to 4) sampled when `--e-out` reaches .25 is
  between the two Spaces per channel and 5+ from each. The symmetric layer fade over the new
  gradient weighs the old Space (1-p)^2, so it is not a linear mix: at half the easing it is
  already 75 % new (the window's frame does the same).
- **Legibility of the tinted chrome.** Without blur (.94) the ink each ground draws in holds 4.5
  on every stop of every preset over black and white (`tests/legibility.rs`). Over blur, at
  the flat tints' alphas, 50 pairs fall short over a pure black or white backdrop, worst the
  light widget (paper ink, 3.91 over black), the light dock (4.11) and the dark bar (4.36 over
  white). The smallest .02 raises that clear them all: Bar dark .66 to .68, Dock light .55 to
  .59 and dark .66 to .68, Osd dark .66 to .68, Widget light .54 to .60 and dark .65 to .67
  (Bar light, Popover and Osd light already clear). Not applied: the alphas are design/03
  §17.2's proposed values and over-blur legibility is its open decision 11; a test pins today's
  shortfall so a retune has to update this note. `--f-ink` on `--f-pill-hover` and on
  `--f-pill` over every stop clears 4.5 for all presets in both schemes.
- **Q13 Popup roots.** A Popover, Sheet or Toast root stamps `data-chrome="transparent"`: no
  background, no box-shadow; its `.ds-popover` (every menu) and `.ds-sheet` cards paint
  `--m-tint-solid` (`--m-tint` with blur), a transparent border and `--m-box` (the material's
  edge and drop, a new variable every material block declares). Proof over the harness's new
  `Backdrop::Clear`: inside a Popover root's rounded corner and far from the card, alpha 0; the
  card's middle alpha > 200; the same root with `chrome: Painted` covers its corner.
- **Q11 Menu control.** `on_hover`, `on_release`, `entrance: MenuEntrance`, the `MenuOut` exit
  fade on Escape and outside click, `onpick` before `onclose`. Blitz sends a `click` to the
  release target even when the press began elsewhere, so a press-drag-release onto an item would
  pick twice; the menu picks once (`Closing::Picked`). A drag counts as one when the pointer came
  in and no press began inside the menu; a release over a disabled item, a header or padding
  then closes through the fade. `Anim::MenuOut` is quire's own keyframe (`menu-out`, 170 ms
  `--e-exit` at Standard; the catalogue is now 47 variants and 40 keyframes). Proofs: hover
  reports `0, 2, none`; a drag from an opener onto Quit logs `hover:2,release:Primary,pick:3,
  close`; a disabled release logs no pick and closes after the fade; Escape and an outside
  click leave the menu `leaving` with no `close` until `settle(MenuOut)`; an instant menu is
  `present` on its first frame, an animated one `entering`. Submenu panels do not take a
  press-drag-release (bar menus have none); the root panel does.
- **Q12 Status items and the frame ground.** `IconButtonVariant::Status` (box
  `--bar-status-box`, glyph `--bar-status-glyph`, defaults 22 and 16, `--f-ink-soft`, hover
  `--f-pill-hover`, pressed/open `--f-pill`) and `ds::StatusMetrics` to write the properties.
  The glyph is sized in the sheet, not by its attributes: Blitz lets a CSS width override an
  `svg`'s presentation width and scales the drawing (proof: the wifi glyph inks 20/24 of 24 and
  of 16 px). `data-ground="frame"` (`css/ground_css.rs`) redirects the paper inks and fills to
  the frame's and gives `.ds-overlay` the paper values back per scheme; proof: a `var(--ink)`
  swatch on the bar is `--f-ink`, one inside a popover opened from it is the paper ink.
- **Small items.** `Icon::Ethernet` (Lucide `ethernet-port`, lucide-static 1.47.0); `WifiOff`
  and `BatteryCharging` already existed. `MenuEntry::Info { title, detail }`. `Press.at`
  (`Point`, client coordinates; `Press` lost `Eq`). `ds::icon::classify` / `classify_with`
  (`IconKind`, `ChromaLimit`, `DsError::IconDecode`), OKLab by Ottosson's published matrices,
  opaque meaning alpha >= 128; `ds` now depends on the pinned `image` (png) for decoding. No
  settings key names the 0.04 threshold yet: design/22 should gain one (for example
  `icons.symbolic_chroma_max`), then sill passes it to `classify_with`.
- Gallery: Controls has "Status items on the frame" (both metrics policies, blur on and off,
  rest, open, pressed, disabled, with a Quiet app name, a Count and the clock on the frame);
  Materials' Bar specimen uses status items and every tinted material shows the gradient; the
  Overlays sheet poses a bar status menu with two status lines. The gallery root and its
  `Scope`s pass `chrome: Painted` (a specimen root is the panel) and the Tokens, Gaps and Matrix
  scopes `frame: Some(FrameTint::None)` so their Popover backdrops stay flat.
- Left: G7 (`PopupSize::FitContent`) is shell-host's, not quire's; the materials page's floor
  chips still measure the flat tint recipe, not the gradient over blur (the numbers above are
  that measurement); disabled status items have no visual (04-COMPONENTS O-1).

## Tune wave (2026-09-24)

Three sensible defaults the user asked for now, all configurable, to be tuned later. Branch
`tune`; `crates/ds/tests/legibility.rs`, `crates/ds/src/material/recipe.rs`,
`crates/ds/src/css/materials_css.rs`, `crates/ds/src/icon/classify.rs`,
`crates/ds/src/error.rs`, `design/03-COLOR.md` section 17.2, `design/21-SPACES.md` sections 3,
`design/22-SETTINGS.md` section 3.3, `design/08-ICONS.md` section 1.5.

- **Over-blur legibility raises.** "Bar gaps" measured six material/scheme pairs short of 4.5:1
  over a pure black or white backdrop at the wave 1 tint alphas (worst: the light widget, 3.91
  over black). The smallest further `.02` raises that clear every pair: Bar dark `.66` to
  `.68`, Dock light `.55` to `.59`, Dock dark `.66` to `.68`, Osd dark `.66` to `.68`, Widget
  light `.54` to `.60`, Widget dark `.65` to `.67` (the light bar, Popover and light OSD already
  cleared it and are unchanged). After the raise the worst margin over the full sweep — 8
  presets x 2 schemes x 5 tinted materials x {black, white} — is the dark Widget, preset 2, over
  white, at 4.551:1 (measured with a temporary probe test, then discarded); everything else
  clears with more room. `crates/ds/src/material/recipe.rs`'s `tint()` carries the new values
  and the wave-1/wave-2 history in comments; the stylesheet golden
  (`crates/ds/tests/snapshots/stylesheet.css`) and `crates/ds/src/css/materials_css.rs`'s unit
  test were re-blessed/updated for the dark-widget literal it still hardcodes.
  `crates/ds/tests/legibility.rs`'s `the_tinted_chrome_over_blur_shortfall_is_the_recorded_one`
  (which pinned the count of failures, 50, so a retune would be noticed) is now
  `the_tinted_chrome_holds_its_ink_over_blur`, asserting the failure list is empty — the same
  shape as the existing blur-off test, so a future retune that drops any material/scheme pair
  below 4.5 over blur (with or without compositor blur) fails a test instead of only a doc note.
  Values marked settled with today's date in design/03-COLOR.md section 17.2 (table and prose)
  and design/03's open decision 11 (the over-blur half of that question is now specified); the
  intro sentence there is careful to say only these six alphas are settled — the rest of section
  17.2 stays proposed, unaffected.
- **OSD takes the Space gradient.** Confirmed rather than built: bar gaps' `FrameTint::of`
  already returns `Tinted` for `Material::Osd` (coherence across chrome), and
  `crates/ds/tests/snapshots/root/chrome/osd.html`'s golden and the Materials gallery sheet
  already show the gradient on the OSD specimen — no code change needed. design/21-SPACES.md
  section 3's row is now split: OSD reads "yes (settled 2026-09-24)" on its own row; the
  remaining "no (proposed)" row is notifications, power menu, lock and polkit only.
  design/22-SETTINGS.md section 3.14's `spaces.overlay_tint` description is updated to match: it
  no longer covers OSD, since the OSD now always tints rather than following that toggle.
- **`icons.symbolic_chroma_max` settings key.** Added to design/22-SETTINGS.md section 3.3:
  `Fraction`, default `40` (0.04), range `0..200` (0.0..0.2), Advanced (file only), citing
  design/08-ICONS.md section 1.5 and `ds::icon::ChromaLimit`. `ChromaLimit` gains
  `impl Default` (0.04, the same as `ChromaLimit::PROPOSED`) and
  `impl TryFrom<f32> for ChromaLimit`, which takes a plain OKLCH chroma value (as the settings
  key stores it, not thousandths) and refuses anything outside `0.0..=0.2` with the new
  `DsError::ChromaLimitRange` (a `String`-formatted value, not a bare `f32`, since `DsError`
  derives `Eq` and floats cannot — `CONVENTIONS.md` section 2). `design/08-ICONS.md` section 1.5
  and `classify.rs`'s module doc point at the key instead of saying "no key names it yet".
  **Left for later, deliberately:** `crates/ds-settings` already has an `IconsSettings` domain
  struct with five sibling keys in this exact shape (`crates/ds-settings/src/settings.rs`), and
  registering a sixth field there would be the mechanical next step — but that crate is not in
  this wave's ownership (crates/ds, crates/ds-gallery, design/03, 21, 22, 08, FINDINGS, CONSUMING
  only), so it is untouched here. Consequence: `cargo test --workspace --all-features` fails one
  test outside this wave's files, `ds-settings`'s
  `crates/ds-settings/tests/schema.rs::every_key_in_catalogue_has_a_spec`, because the new doc
  row in section 3.3 has no matching `KeySpec` yet (it is not on either allow-list the test
  already carries for exactly this kind of gap, `SILL_OWNED`/`DOC_ALIASES`). Registering
  `icons.symbolic_chroma_max` on `IconsSettings` (bumping `icons_keys_are_all_advanced`'s count
  from 5 to 6) and then having `sill`'s dock/tray code read the key and pass it into
  `classify_with` are both still open, for whoever owns `ds-settings` and `sill` next.
- Gallery: `materials-{light,dark}-{postmark,green}.png` regenerated with the six new alphas;
  looked at with Read (light and dark, Postmark) — the OSD, Bar, Dock and Widget specimens read
  slightly more opaque than before, as expected of a `.02` alpha raise, with no other visible
  change to layout, radius or shadow.

## Launcher gaps (2026-09-24)

sill's launcher (sill FINDINGS "M4 launcher-ui", F140-F149, Q40-Q45) and its dock (Q15-Q17)
reported gaps against quire; two were crashes that took the shell daemon down. All are closed on
branch `launcher-gaps`. Headless proofs: `crates/ds-native/tests/launcher_crash.rs`,
`launcher_gaps.rs`, `dock_gaps.rs`; SSR goldens under `crates/ds/tests/snapshots/`
(`overlays/command_palette/{surface-cmdk,app-icon}.html`, `overlays/tooltip/fly-{shown,hidden}.html`,
`controls/icon_view/{glyph-tile96,image-px}.html`, `root/surface/radius.html`,
`root/chrome/dock-radius.html`).

- **Q45, crash: a motion task outlived its component.** The cause was not the timer's body but
  how it was spawned. `ds::time::spawn_in(scope, …)` called `Runtime::spawn(scope, future)`,
  which runs a task *for* a scope without registering it in that scope's `spawned_tasks`; only
  a scope's own `spawn` registers, and dioxus-core drops exactly the registered tasks when a
  scope is removed (`Runtime::remove_scope`). So a palette unmounted before `peek-in` settled
  left its settle task alive. The runtime skips a task whose scope is gone, which is why an
  early unmount alone rarely crashed; but dioxus reuses freed scope slots, and once a new scope
  (the next palette, a result row) sat in the old slot the stale task was polled again and its
  `phase.set(Settled)` wrote the dropped signal: `ValueDroppedError` at `timer.rs:50`, the
  daemon's panic on a toggle pair. Fix (`crates/ds/src/task.rs`): `spawn_in` enters the owner's
  scope and spawns through its own `spawn`, so the task is registered and dropped with it; and
  every task body writes through `try_set`/`try_get`, which end the task's work on a dropped
  signal instead of panicking (a signal can belong to another scope than the task). Covered:
  `MotionTimer` (`on_settled` never runs for a component that is gone), `Roster::leave` and its
  rest timer, `HoverHub`'s open, close, leave and warm timers (the `HoverIntent` machine itself
  spawns nothing), `ToastHub`'s hold, and `Pulse::fire` (no task; it writes through `try_set`).
  Every other task in `ds` is a plain `spawn` from a handler or a hook of the component that
  owns the signals it writes (rect probes, the menu tracker, the send pill, the toast stage), so
  it was already registered and dropped with its scope. Proof: `launcher_crash.rs` mounts a
  `CommandPalette` and a `Menu` and unmounts them 100 ms later, toggles a palette open and shut
  four times 500 ms apart at Extra motion (each close lands mid `peek-in`), and mounts a palette
  fifty times with varied lifetimes, while small components are remounted every 16 ms beside it
  (they keep dioxus reusing freed slots, as the launcher's rows do). On master the fifty-mount
  test panics on every run (ValueDroppedError, or Q43's RefCell first) and the toggle test on
  some; the early-unmount pair passes there too, since nothing reuses the slot in time. On this
  branch all four pass (run three times).
- **Q43, crash: focus from a task polled inside the render.** dioxus-native-dom's
  `NodeHandle::set_focus` calls `doc_mut()` when it is *called*, not when its future is polled;
  `TextInput`'s `Focus::OnMount` spawned a task that called it, and dioxus polls a task woken in
  the same turn as a dirty scope inside `render_immediate`, while the mutation writer holds the
  document: "RefCell already borrowed" (`events.rs:161`). The menu's own focus on mount, the
  submenu's, the tracker's return to its panel and the slider's did the same. Fix: a `HostFocus`
  seam beside `HostMeasure` (`crates/ds/src/focus/host.rs`). Every focus change is
  `focus_soon(element)` (a task of the calling scope) or `focus_element(&element).await`, which
  asks the host's `HostFocus` and answers `Focused::{Done, Busy, Unknown}`; `Busy` waits
  `FRAME_SLACK` and tries again, up to eight frames. ds-native's `FOCUS`
  (`crates/ds-native/src/focus.rs`) probes the document with `NodeHandle::try_doc` first and only
  then moves the focus, so a held document is `Busy` and nothing panics or prints. With no host
  seam the call and its future are guarded (`crates/ds/src/guarded.rs`, `Guarded` moved there
  from `geometry/measure.rs`, plus `guarded_call` for a call that borrows when it is made): the
  panic is caught and read as busy, which changes nothing (the borrow failed first) but the
  default panic hook still prints it. `ds_native::launch`, the harness and
  `ds_native::focus::provide()` provide `FOCUS`. Proof: the fifty-mount test (Q43 on master,
  every run) and a unit test of `guarded_call` against a held `RefCell`.
- **Q44 Giving a field the keyboard back.** `ds::use_focus_request() -> FocusRequest`,
  `FocusRequest::request(&self)`, and `Focus::Controlled(FocusRequest)`: the field focuses on
  mount and again for every request made since the last one it served (a `FocusTicket`
  counter); `CommandPalette { focus: Some(request) }` passes it to its field. `Focus` lost `Eq`
  and `Hash` (a signal handle has neither). Proof: a palette in a surface opens an actions
  `Menu` on Tab, the menu takes the keyboard, Escape fades it out, its `onclose` calls
  `request()` and the field has the focus again with the card still `data-presence="present"`
  (not remounted); the same page without the request leaves the field unfocused. The consumer
  example's subject field takes the keyboard back from its More menu the same way.
- **Q40 Embedded palette.** `CommandPalette { host: CommandPaletteHost::Surface }` renders the
  card in place, with no wrap, scrim or overlay slot: `width:100%; height:100%`, a flex column
  whose list takes the height under the field (`max-height:none`), painting the enclosing
  material (`--m-tint-solid`, `--m-tint` under `data-blur=on`, `--m-box`, border transparent)
  at `--r-panel`. It still joins the layer stack (Escape in its field closes it only when
  nothing is above it). `id: Option<String>` goes on the card in both hosts;
  `entrance: PaletteEntrance::{PeekIn, CmdkIn}` picks the keyframe (`data-entrance`) and the
  settle timer's `Anim`. Proof: the card's rect equals its 600 x 400 container, no
  `.ds-palette-wrap` and no palette overlay exist.
- **Q41 Selection and row rects.** `selected: Option<usize>` (controlled: Up, Down and the pointer
  only ask through `on_select`), `on_select: Option<EventHandler<usize>>` (uncontrolled, every
  change of the effective selection, reported from an effect after the render that made it,
  the first choice on mount and after a new query included), `on_select_rect:
  Option<EventHandler<Rect>>` and `onkey: Option<EventHandler<KeyboardData>>` (every key the
  field gets, after the palette's own reading). The rect comes from the row's element: the
  palette keeps each row's `MountedRef` by the *line* it is drawn on (the choice-to-line table
  of the render that mounted it, `palette_lines::choice_lines`), because a row element persists
  at its line while the choice drawn there changes; it reports when the selection or the element
  under it changes, reading through the measurer a frame after layout
  (`palette_rows.rs`; the selection itself is `palette_select.rs`). The pointer moving over a row already moved the selection (O-13); now
  the caller hears it. Proof: the log reads `select:0,rect:<row 1>`, then after Down
  `select:1,rect:<row 2>`, then after a pointer move `select:2,rect:<row 3>`, each rect equal to
  the harness's own measurement of that row; a controlled palette whose caller ignores the
  request keeps row 1 selected; an actions menu anchored with `Anchor::Rect(rect)` hangs 6 px
  under the selected row.
- **Q42 App icons in rows.** `Tile::Source(IconSource)`: `Image` draws the icon file filling the
  tile (`[data-tile=image] > .ds-ext-icon{width:100%;height:100%}`, no border or ground), a
  `Symbolic` sits on the plate, a `Glyph` is `Tile::Icon`'s. Proof: a red PNG's icon rect equals
  its 34 px tile and its centre pixel is red.
- **Dock Q15 Radius.** `Surface { radius: Option<Corner> }` and `Ds { radius: Option<Corner> }`
  write `--m-radius` inline, which the material block's `border-radius:var(--m-radius)` then
  reads. `Corner::{Token(Radius), Px(Px)}` (`tokens/shape.rs`), `From<Radius>`. The dock's pill is
  its `Ds` root now (sill F125), so the root takes it too. Proof: a dark Dock surface with
  `Corner::Px(Px(0))` paints the pixel 2 px inside its corner (luma < 80) where the material's
  22 px leaves the light ground (> 180).
- **Dock Q16 Icon sizes.** `IconSize::Tile48` (48), `IconSize::Tile96` (96) and
  `IconSize::Px(IconPx(u8))`. Proof: glyphs lay out at 96, 48 and 71 px. design/08-ICONS.md
  §1.4's size table should gain the three rows (not in this wave's files).
- **Dock Q17 Controlled tooltip.** `Tooltip { shown: Option<Shown> }`, `Shown::{Visible, Hidden}`:
  `[data-shown=visible]` shows the Fly's label at once with no pointer, `[data-shown=hidden]`
  keeps it down under the pointer; a Card follows `shown` instead of the hover hub. Proof by
  pixels: a shown label's padding is ink-dark at rest, a hidden one stays the ground under the
  pointer after 700 ms, and an uncontrolled one still shows there.
- `ds_native::Harness::is_focused(selector)` (the focus proofs above).
- Gallery: Overlays has "Palette in a surface" (two 540 x 330 launcher panels: `cmdk-in` with
  app icons over a query, `peek-in` over the empty query's actions and recent apps); Controls
  has "Dock tiles" (glyphs and app icons at `Tile48`, `Px(71)`, `Tile96`, and a Dock pill at
  `Corner::Px(Px(12))` whose middle label is `Shown::Visible`). Overlays grew to 1640 px and
  Controls to 2400. Sheets regenerated and looked at in both schemes.

What sill changes to drop each workaround (sill FINDINGS F141-F148):

- **900 ms keep-mounted (`ENTRANCE_GUARD`, `mounting.rs`, F145).** Delete it: unmount or remount
  the palette and the actions menu whenever the state says so. A quick reopening may still
  reuse the mounted palette if the entrance should not replay, but that is a design choice now,
  not a crash guard.
- **60 ms delayed mount (`MOUNT_WAIT`, `events.rs::opened`).** Delete it and mount the palette on
  the opening. Also give every root the focus seam beside the measurer:
  `Root::context(ds_native::focus::FOCUS)` next to `Root::context(MEASURE)` (or re-export it
  from `sill-surfaces` as `MEASURE` is), and `ds_native::focus::provide()` beside
  `ds_native::measure::provide()` in the popups. Without it nothing crashes, but a collision
  prints the caught panic.
- **Remount to regain focus (`actions_closed`/`menu_gone`).** Keep one
  `let field = use_focus_request();`, pass `focus: field` to the `CommandPalette`, and call
  `field.request()` from the actions menu's `onclose`; the palette's `key` no longer changes.
- **Harness retry-on-panic (`tests/launcher_harness::retrying_the_focus_race`, F148).** Delete it;
  the tests run once.
- **Fixed actions anchor (`ACTIONS_AT`) and the mirrored selection.** Pass `on_select_rect` and
  anchor the menu at `Anchor::Rect(rect)`; take the selected row from `on_select` (or own it
  with `selected`) instead of mirroring Up and Down; read Tab and Ctrl+K from `onkey` instead
  of the root's `onkeydown`.
- **Scrim and card size (F141).** `host: CommandPaletteHost::Surface`, `id: Some("launcher-card")`
  for the blur region at radius 14, `entrance: PaletteEntrance::CmdkIn` if wanted; give the
  palette's container the panel's size.
- **Provider glyphs (Q42).** `tile: Some(Tile::Source(icon))` with the app's resolved
  `IconSource::Image(ExternalIcon { url: IconUrl::file(&path)?, size: IconSize::Tile48 })`.
- **Dock (F94, F125, F126).** `Ds { radius: Some(Corner::Px(Px(f32::from(dock.pill_radius_px.0)))) }`
  on the pill root; `IconSize::Px(IconPx(side))` for the tile's resolved icon side instead of
  22 and `transform:scale`; `Tooltip { shown: Some(if label_gate open { Shown::Visible } else
  { Shown::Hidden }) }` driven by the machine's `ShowLabel`/`HideLabel` instead of leaving the
  label out of the document.

## Palette follow-ups (2026-09-24)

sill's adoption of the launcher gaps (sill FINDINGS "Adoption of the launcher gaps", F164,
Q60-Q63) left three gaps against `CommandPalette`, closed on branch `palette-followups`.
Headless proofs: `crates/ds-native/tests/palette_followups.rs` and `palette_actions_key.rs`;
SSR goldens under `crates/ds/tests/snapshots/overlays/command_palette/` gained `data-shown` and
`data-pulse`, and `stylesheet.css` gained the palette's three new rules.

- **Q60 The selected row's rect waits for layout.** A palette on a surface that is mapped
  again reads its selected row before the surface's first layout, got 0 x 0 at the origin,
  reported that, and never measured again, because neither the selection nor the row element
  changed afterwards. Now `palette_rows` reads through `geometry::measure::laid_out_rect`: a
  frame after the row mounts, then again while the read has no area, every frame for 30 tries
  (about half a second), then every 100 ms for 30 more, then it gives up (`layout_retry`, a
  pure schedule with a table test). An empty rect is never reported. The row is measured again
  when the selection, the element under it, or the results change (a `Revision` bumped when the
  `groups` or the field's `tokens` differ from the last render's, since a token row moves the
  list without moving the selection). A read still waiting is cancelled when a newer one
  starts, and an identical rect is not reported twice. To reproduce the fresh surface,
  `ds_native::Harness::unmapped(app, viewport)` builds a document whose renders and tasks run
  but which is never styled or laid out until `Harness::map()`. Proof: unmapped for 150 ms the
  log is `select:0` with no rect (on master it reads `select:0,rect:0x0@0,0`, which is F164);
  mapped, `rect:588x44@6,81` follows within 100 ms, Down reports row 2's rect, and a token row
  appearing re-reports row 1 lower down. With the revision keyed on `groups` alone that last
  step fails, so the test covers the tokens.
- **Q61 `onkey` hands on the event.** `CommandPalette { onkey: Option<EventHandler<KeyboardEvent>> }`,
  and `TextInput`'s and `SearchField`'s `onkey: EventHandler<KeyboardEvent>`. The owned
  `KeySnapshot` copy (`owned_key`) is gone: a dioxus `Event` is a cheap `Rc` clone whose
  `prevent_default`/`stop_propagation` flags are shared, and dioxus-native-dom copies them to
  Blitz's `EventState` after the handlers run, so a caller's `prevent_default` cancels Blitz's
  default action. For Tab that action is `focus_next_node`, and it runs only on keydown. Proof: a
  caller that prevents Tab keeps the field focused, and one that only counts it loses the focus
  to the button after the palette (both assert the caller heard the key).
- **Q63 A palette kept mounted.** `CommandPalette { shown: Option<Shown>, retain: Retain }`,
  `Retain::{Nothing, Query}` (default `Nothing`), `Shown` being the tooltip's `{Visible, Hidden}`
  (`components/palette_shown.rs`). `Hidden` puts `data-shown="hidden"` on the card (and on the
  overlay host's wrap), `display:none`: the rows and field stay in the document but nothing is
  laid out or painted, and the palette leaves the layer stack (`Float::withdraw`, so a hidden
  palette takes no Escape and blocks no other layer; `Float::rejoin` on show). On each
  `Hidden -> Visible` change (`palette_shown::change`, table-tested), an effect after that render
  rejoins the stack, flips `data-pulse` between `a` and `b`, restarts the settle timer
  (`data-presence` entering, then present), forgets the last row report so the row is
  reported afresh, and requests the field's focus. Unless `Retain::Query` is given it also
  resets the selection (own: the first choice, reported again; controlled: `on_select(0)`) and
  calls `oninput("")`. A palette mounted hidden does not take the focus until it is first
  shown: its field is `Focus::Manual` until then and `Focus::Controlled(request)` after, where
  `request` is the caller's `focus` or the palette's own. **The alias is load-bearing.** Taking
  `display:none` away does not restart a CSS animation in Stylo. With the two `data-pulse=b`
  rules removed, even the first show is not mid-entrance 40 ms in (8 384 of 201 600 pixels
  differ, under the 10 % the test asks), because the entrance already ran out while
  hidden. The rules name `peek-in--b`/`cmdk-in--b`, which `motion_css` emits and the lint
  already knows (`lint::registry::is_known_anim`). Proof: hidden, `#card` has no area and the frame
  is the frame without it; two shows 300 ms apart each render a frame 40 ms in that differs
  from the settled card in over 10 % of its pixels (they are scaled and faded, looked at); each
  show focuses the field and the second empties the query `f` typed during the first; hidden
  again, the card's area is pixel-identical to the first hidden frame. With `Retain::Query`
  the query survives a hide and show.
- **Q62 hint (sill's to act on).** While the actions `Menu` is open it has the keyboard, so the
  second Ctrl+K never reaches the palette's `onkey`. The menu stops only the keys it acts on,
  and a chord is not one of them, so the keydown bubbles out of the overlay host to the
  caller's root. The first Ctrl+K has to be stopped as well as prevented in `onkey`, or it
  bubbles on to that same root handler, which sees the menu already open (the signal write is
  synchronous) and closes it in the same dispatch. That was the first failure of the proof.
  `Harness::chord(&[Key::Ctrl], Key::Char('k'))` presses a key with modifiers held. Proof
  (`palette_actions_key.rs`): Ctrl+K opens the menu and it takes the keyboard; the second Ctrl+K
  closes it and the field has the keyboard back. CONSUMING.md has the pattern.
- Gallery: Overlays' "Palette in a surface" has a third specimen, "Warm: kept mounted, shown by
  the button": a `Launcher` button toggling a `cmdk-in` palette that lists actions and recent
  apps, and apps as you type, hidden on pick or Escape. Posed (the sheets) it is shown; live it
  starts hidden. Overlays grew to 2080 px.

What sill changes (read-only here; `crates/sill-surfaces/src/surfaces/launcher/panel.rs`,
`events.rs`):

- **F164 fallback.** Delete `FIRST_ROW_FOOT` and `measured`, pass
  `on_select_rect: move |rect| row.set(Some(rect))`, and anchor with `Anchor::Rect` alone (the
  menu waits for the first report, which now always has an area).
- **Q61.** `onkey: move |event: KeyboardEvent| key.field_key(&event)`, with `field_key` taking
  `&KeyboardEvent` and calling `event.prevent_default()` on Tab, Shift+Tab, Ctrl+K and Alt+K. It
  also calls `stop_propagation()` if the root keeps a handler for the menu's second key. Drop
  the root `div#panel`'s `onkeydown` that only prevented those keys (`PanelCtx::root_key`).
- **Q62.** Give that root handler the one job it still needs: while `ActionsMenu::Open`, a
  Ctrl+K or Alt+K there is `prevent_default`, closes the menu and calls `panel.field.request()`.
- **Q63 (if the warm opening is wanted).** Render the palette always, not per opening: drop
  `key: "{session.serial.0}"` and the `if let Some(shown)`, and pass
  `shown: if opened { Shown::Visible } else { Shown::Hidden }` with `retain: Retain::Nothing`.
  The session's reset on opening then comes from the palette's `oninput("")` and
  `on_select(0)`, or stays the session's own. Keep the query and selection with
  `Retain::Query`.
## Icon bake-off (2026-09-24)

design/08-ICONS.md 3.4 run once on the local GPU, branch `icon-bakeoff`. Full write-up and
the agent's assessment: `docs/icons-bakeoff.md`; sheets: `tools/progress/shots/icons/`;
weight licences: `docs/licensing-references.md`. The user picks the model.

- **Both candidates run on the 16 GB card at 1024 px.** FLUX.2 Klein 4B (Q4_K_M GGUF): 3-4 s
  per warm render, peak 12.0 GB on the card. Qwen-Image-2512 (Q3_K_M GGUF, NVFP4 text
  encoder): 70-90 s warm, peak 14.7 GB, so it cannot share the card with any other job.
  Qwen-Image-Edit-2511 Q3_K_M (reference edits): ~120 s warm, 489 s cold.
- **FLUX.2 Klein 4B is Apache-2.0, not a separate "Klein licence".** 08 3.2 and 5 expect
  special terms; the 4B repo's card says Apache-2.0. The 9B Klein and FLUX.2 dev are
  `flux-non-commercial-license` and must stay out.
- **Terminal is the weak subject for both models**: neither drew a prompt caret under "no
  letters"; both drew a checkmark or a mouse pointer on a screen. The subject phrasing (or 08
  2.7's glyph route) needs another round whichever model wins.
- **Reference-conditioned edits over-copy the hero's shape** (08 3.6). Klein's reference latent
  turned "files" into a blue envelope; Qwen-Image-Edit at the settings used turned "notes"
  and "terminal" into envelopes and patterned every background. Both passes keep the hero's
  material and light. The plain brief already gives a consistent set, so 08 3.6's premise
  (the rest of the set must be edits of the hero) is worth re-deciding.
- **08 3.7's key (ΔE_OK 0.03 from one corner colour) does not survive real renders**: the
  ground has a vignette and a lit floor band, and white objects sit within 0.1 of it. The
  post-process grows the ground from the border under a chroma gate and a per-step ΔE limit,
  keeps the floor shadow as a matte, and drops specks (`tools/icons/src/key.rs`). White-rimmed
  objects still lose slivers; a segmentation model may be needed after all.
- **08 2.5's shadow does not fit 08 2.2's margin at 1024.** Scaled by plate/48 it is offset
  103 px with 275 px blur against a 100 px margin, so the master's shadow is clipped at the
  canvas edge. 08 2.5 needs a cap or a different scale.
- **Every ComfyUI run in this session was killed twice by harness restarts**, not by memory:
  a server started as a background task of the agent dies with it. Start it with `setsid` so
  it survives, and stop it by its `agent-limit16-*` scope as the README says.

## Icon round two: abstract (2026-09-24)

The user found round one "too realistic" and asked for abstract icons in the design language.
Branch `icon-round2`; write-up `docs/icons-bakeoff.md` "Round two"; the language is 08-ICONS 2.8.

- **The models go flat when asked, but do not keep our rules.** With a flat paper-cut-out brief
  on a mid-grey ground, Klein and Qwen both dropped the clay look, but Klein outlines everything
  in black at its own weight and Qwen drifts back to bevels and drop shadows; neither holds the
  2-unit round-capped stroke or the four-colour rule. Qwen drew the first real terminal
  (a `>` and a bar) and failed notes on every seed (a green sheet on the green plate).
- **A mid-grey ground keys cleanly.** Round one's light-grey ground ate white rims; with mid grey
  every round-two render keyed without loss, paper white and ink both.
- **"Prompt caret" and "chevron" are ambiguous to the models.** Klein drew down-pointing double
  chevrons until the prompt said "right-pointing, like a greater-than sign".
- **Procedural icons from the tokens** (`tools/icons abstract`, TOML specs, six shape kinds as
  signed distances) are coherent by construction, render every export size natively in about
  3 s for the set, and stay crisp at 16 px. The cost is designing each arrangement by hand.
- **A circle over a hill in a rounded frame reads as a person at 16 px** (Klein photos). Keep
  the sun off-centre, as the procedural photos spec does.

## Icon round three: pressed into the plate (2026-09-24)

Branch `icon-round2`; write-up `docs/icons-bakeoff.md` "Round three"; rules 08-ICONS 2.9.

- **Klein 4B does soft embossing well when it draws the whole surface.** Asked for a symbol
  pressed into a full-bleed two-hue surface, all 20 renders came out as one consistent clay-matte
  family; used whole as the plate face (centre crop, our squircle and bevel) they need no keying.
  Its gradients are pastel and patchy, though, and the pale symbols are weak at 16/32 px.
- **A model-drawn tile keys badly when its face is pale.** The region-grow key stops at edges;
  a soft bevel on a pale tile against mid grey has none, so holes appear. Face mode avoids it.
- **"A circle above a hill in a rounded frame" is a person**, now in both rounds. Put the sun
  small and off to the upper right.
- **Two distant hues need OKLCh, not OKLab.** Violet to amber mixed in OKLab passes through a
  muddy mauve; mixing hue along the shorter arc passes through rose.
- **The emboss is noise below 48 px.** The procedural renderer drops it at 16 and 32 and
  strengthens recessed fills so the details survive.

## Icon round six: the shipped set (2026-09-25)

Branch `icon-ship`; design/08-ICONS.md 2.11; assets in `assets/icons/apps/`.

- **One finish makes model faces and drawn faces one set.** Resampling the Klein face onto each
  size's plate and drawing the plate, grain, bevel, rim and shadow at that size with the same code
  as the procedural icons gives matching silhouettes, bevels and shadows; the exported 512s agree
  within 2 px of shadow reach (`tools/icons/tests/shipped.rs`).
- **A near-white plate cannot show the bevel arc.** The Paper plate (L 0.98) has no headroom for
  a white highlight; its bevel reads through the bottom shade and the rim only.
- **Exporting Monochrome neutral and tinting at run time holds up.** The neutral set retinted by
  `ds::icon::retint` with the Work or Home tint reads like round four's Monochrome rendered in
  that hue (compare `round6-shipped.png` with `round4-monochrome-space.png`), and one set serves
  every Space.
- **The grain dominates PNG size**: a 1024 icon is about 1.2 MB, a 512 about 300 KB; stopping
  `@2` at 256 (design/08 2.6) keeps the whole three-style set at 6.1 MB.

## Icon round five: colourways (2026-09-25)

Branch `icon-round5`; write-up `docs/icons-bakeoff.md` "Round five"; palette in 08-ICONS 2.10.

- **Eight hues 45 degrees apart all read at C 0.07**, with clay/rose and teal/slate the only pairs
  that come close at 16 px.
- **The dock pill never decides a colourway.** Every hue at every cap is 3.0-4.1 against the pill
  on the Work and Home frames in both schemes, because the plate's L 0.62 sits between the light
  and the dark frame; hue only decides harmony with the Space, not legibility.
- **Near-complements of a Space's hue clash only when bold.** At C 0.07 ochre on Work or plum on
  Home is fine; at 0.15 they vibrate against the tinted frame.
- **C 0.15 at L 0.62 is outside sRGB for ochre, jade and teal**; of the three caps, 0.11 is the
  highest at which every hue's plate colour stays in gamut, which makes it the natural "bolder"
  step.

## Icon round four: dialects (2026-09-25)

Branch `icon-round4`; write-up `docs/icons-bakeoff.md` "Round four"; rules 08-ICONS 2.10; keys
`icons.style` and `icons.monochrome_tint` in design/22 3.3.

- **A chroma cap of 0.07 in OKLCh is enough to make a mixed set read as one palette.** Round
  three ran to 0.15-0.20; at 0.07 the procedural plates and the retinted Klein clay sit in the
  same family of tones.
- **A model's face can be brought into a dialect without re-rendering.** Fitting a lightness
  plane to the border as the ground and keeping only the relief above it keeps Klein's emboss and
  drops its colour and its uneven gradient; the gain has to be adaptive (a percentile of the
  relief mapped to the symbol lightness), since each render's symbol sits a different amount
  above its ground.
- **Retinting amplifies ground mottling.** Where the model's ground is blotchy (Klein files s22)
  the gain that lifts a pale symbol lifts the blotches too; a small dead zone helps, a cleaner
  render helps more.
- **The Space tint comes free from `ds`.** The accent `ds::space::derive` returns for a Space is
  already at a muted chroma (0.045 + 0.035 x dot chroma), so its hue and chroma drop straight into
  the Monochrome dialect and match the frame.
- **A paper plate nearly vanishes at 16 px on a light ground**; the ink symbol has to carry it.

## macOS polish (2026-09-24)

The user compared the shell with macOS and found it flat; the brief was to add the layers macOS
stacks without touching the Arc/Post identity. Branch `macos-polish`. Proofs:
`crates/ds-native/tests/macos_polish.rs` (pixels), `crates/ds/tests/polish_ssr.rs` (goldens
under `tests/snapshots/polish/`), unit tests beside each new module; the gallery's new Polish
page puts each piece beside the macOS number it targets.

- **Material stack v2** (design/03 §17.4). Every card material gains `--m-hairline` (0.5 px
  black .14 light / .60 dark), `--m-shadow-contact` (`0 1px 2px`, .10 / .30), `--m-shadow-ambient`
  (menus `0 12px 40px -12px` .28 / .55; sheet, dock, widget their own) and `--m-highlight` (1 px
  white .30 / .12); the bar a bottom hairline. `--m-box` lists them outside in; a tinted root's
  frame redraws the inner pair (`--m-inner`). The tint carries a vibrancy boost computed in OKLab
  (chroma x1.4, light lightness +.012), since Blitz cannot saturate behind (S15, S16), mixed in
  CSS by `color-mix` on `--m-vibrancy` so a key can switch it off. Each alpha reads an input
  (`MaterialStack`, `Ds { stack }`). Blitz facts: five `box-shadow` layers paint; an outset
  shadow is drawn from the averaged `border-radius`; `calc()` inside an `rgba()` alpha and a
  `color-mix` percentage from `calc(var()*100%)` both paint. Legibility gates re-measured on the
  boosted tints: all green, no alpha moved. Proof: a dark card over grey has a brighter top row,
  a darker pixel just outside its side, a contact shadow 1 px under it and the ambient 20 px under.
- **Squircle corners.** `Corner::Squircle(Px(r))`: one quadrant of the `n = 5` superellipse
  reaching `2 r` along each edge (capped at half the box), drawn as a six-layer `mask-image` (four
  quadrant SVGs and two rectangles; Blitz composites mask layers with `add`, and `min()`/`calc()`
  with percentages resolve in `mask-size`). A mask clips the element's own `box-shadow`, so the
  tint moves to a masked `::before` (a tinted root masks its `.ds-frame`) and the shadow stays on
  the unmasked box at `--r-squircle`, the circle of radius .884 r that touches the squircle at 45
  degrees and is never more than .03 r away. Proof: at radius 40 the pixel whose centre is 11.5 px
  in on the diagonal is dark for the squircle (inside by 1.6 px) and the white ground for
  `Px(40)`; well inside and outside agree. A squircle dock root over a clear backdrop keeps its
  outermost corner empty, paints its 45 degree point and still casts its shadow. CSS radius stays
  the default everywhere.
- **Plates.** `IconView { plate: Some(PlateFamily) }` (Red, Amber, Green, Blue, Violet, Neutral;
  design/08 §2.3): the whole superellipse (a single stretched SVG mask), the 135 degree gradient,
  the inner highlight .35 and rim .08 on the masked face, the drop on the box, the glyph at 56 %
  in the family's colour (`--plate-glyph`, an image at `--plate-inset` 72 %). Proof: a 96 px blue
  plate's corner shows the ground, its top-left is the light stop, its bottom-right darker, its
  glyph 53.8 px.
- **Shell type scale.** Tuned tokens (`tokens/tuned.rs`): each is declared on `.ds` from an input
  with the key's default behind it (`--fs-shell-menu:var(--shell-menu-font,13px)`), so one inline
  write (`ShellMetrics::style_attr()`) reaches every nested scope, which re-declaring the token
  itself would not. Bar 13/500, pill 24, radius 4; text menus 22 px rows, 13/400, highlight radius
  6, separators 5 px margins; launcher field 22/500 with a 20 px glyph, rows 14 / 12; tooltips 12.
  Applied inside `IconButton{Status}` (radius 4), `Menu` (Slim, Context, Dropdown), `CommandPalette`
  in a surface and `Tooltip`. Proof: Slim rows are 22 px and the separator sits 5 px under the row.
- **Bar.** `MenuBarItem { open, emphasis }` (the hover and open pill; proof: the open item's
  padding differs from a closed one's) and `WorkspacePills`/`WorkspacePill` (one segmented group
  on the frame, `aria-current`, `Press` from every button).
- **Dock.** `DockMetrics` (tile 48, gap 8, pad 6, dot 4, dot centre 3 px under the tile, floor
  off), `RunningDot`, `DockFloor` (a light band, `--dock-floor` 0 or 1). The floor is subtle on a
  light dock; it reads on a dark one.
- **Launcher.** The card in a surface is `height:auto; max-height:100%` with the list
  `flex:0 1 auto`: as tall as its content. `CommandPalette { corner }` takes a squircle. Proof:
  one result in a 600 x 400 panel is a card under 160 px tall, the glyph 20 px. The launcher-gaps
  proof that the card equals its container now asserts origin and width, and a height below it.
- **Windows and popovers.** `--shadow-window` is `0 1px 3px .12, 0 24px 64px -16px .40` (dark
  .40 / .66). `Popover` fades out on Escape or an outside click (`menu-out`, `--t-quick`) before
  `onclose`. Proof: after an outside click the popover is `leaving` and the caller still open; 400
  ms later it is closed and gone.
- **Settings keys.** Every new number is an input a key writes; design/22 is not in this wave's
  files, so the keys are named here and in the design docs for the owner of design/22 and
  ds-settings to add (existing keys' defaults change where marked): `appearance.material_highlight_light`
  (30), `_dark` (12), `appearance.material_hairline_light` (14), `_dark` (60),
  `appearance.material_shadow_strength` (100), `appearance.material_vibrancy` (100);
  `bar.item_font_px` (13), `bar.item_font_weight` (500), `bar.item_radius_px` (4),
  `bar.open_title_pill_height_px` (24, existing), `bar.title_padding_px` (10, existing);
  `menus.font_px` (13), `menus.item_height_px` (existing, 24 to 22), `menus.separator_margin_px`
  (existing, 4 to 5), `menus.highlight_radius_px` (6), `menus.tooltip_font_px` (12);
  `launcher.field_font_px` (22), `launcher.field_font_weight` (500), `launcher.field_glyph_px`
  (20), `launcher.row_title_px` (14), `launcher.row_detail_px` (12); `dock.tile_size_px` (48,
  existing), `dock.tile_gap_px` (existing, 4 to 8), `dock.pill_padding_px` (6),
  `dock.running_dot_diameter_px` (4, existing), `dock.running_dot_gap_px` (3), `dock.floor`
  (`DockFloor::{Off,On}`, Off), `dock.pill_radius_px` (existing, 22; 18 recommended as a squircle);
  `icons.symbolic_fallback_glyph_percent` (56) and `icons.plate_inset_percent` (72) already exist
  and write `--icons-glyph-share` / `--icons-inset-share`.
- Goldens re-blessed: `tests/snapshots/stylesheet.css` (the materials, shapes, tuned tokens and
  component sheets). Control and overlay goldens did not change (the markup is the same; the
  new props default to absent).

## mailo gaps (2026-09-24)

mailo's first migration wave (Phase A) reported seven gaps against quire v0.1.1. Branch
`mailo-gaps`. Proofs: `crates/ds-native/tests/window_frame.rs` (pixels),
`crates/ds/tests/legibility.rs`, `tokens.rs`, `lint_rules.rs`, the `icon_view/printer` and
`icon_view/folder-input` goldens, `ds-settings`'s `environment` tests.

1. **The window root painted its frame layers beneath itself (visible bug).** Cause: a
   `Material::Window` root was not a stacking context, so `.ds-layer` (`--z-scene`, -2) and
   `.ds-grain` (`--z-grain`, -1) joined the document's stacking context and painted *under* the
   root's own `background: var(--m-tint-solid)`, which for a window is the current `--f-grad`.
   The window showed only that background: a Space switch was an instant swap and the grain was
   invisible. The tinted chrome roots never had this (bar gaps gave them `position:relative`
   and `--z-raise`). Fix: `FrameTint::Opaque` now stamps `data-frame="opaque"` and
   `.ds[data-material][data-frame=opaque]{position:relative;z-index:var(--z-raise)}` makes the
   root its own stacking context. Its background stays the current gradient, under both layers,
   as `.ds-frame`'s does on shell chrome, so the window stays opaque through the fade (a
   transparent background would show the desktop through the half-faded pair). `.ds-layer`
   gains `pointer-events:none`: once the layers sit inside the root's context Blitz hit them
   before the content (the harness's hover-card test caught it). Pixel numbers (320 x 200,
   light, presets 0 and 3): with a test override painting the layers red, the corner is
   `[255,0,0]` against `[226,234,254]` plain (before the fix: `[226,234,254]` both ways);
   grain 100 against grain 0 moves the luminance of 1270 of 1600 pixels in the top-left 40 x
   40 by 2 or more, at most 47 (before: 0 of 1600); a switch from preset 0 to 3 sampled when the
   `--e-out` curve reaches a quarter is `[238,230,241]`, between `[225,234,254]` before and
   `[254,226,225]` after (before the fix the mid sample was already `[254,226,225]`). The
   tinted chrome's tests (`bar_frame.rs`, `macos_polish.rs`) are unchanged and green.
   `gallery_fixes.rs`'s empty-root test now gives its look `Grain(0)`: it asks for a flat
   ground, and the default look's grain 35 is now visible. Every gallery sheet was regenerated
   (window roots show their grain; the tokens page has the new tokens).
2. **Glyphs.** `Icon::Printer` and `Icon::FolderInput`, Lucide 1.47.0 `printer` and
   `folder-input` as published, stroke by attribute, in `icon/geometry_actions.rs` and a third
   list `Icon::ACTIONS` (`Icon::ALL` is the mailo set, the shell set, then the actions; the
   mailo set stays locked to `icons.js`). design/08-ICONS.md is not in this wave's files: its
   owner may list them.
3. **`use_environment(AppName::MAILO)` did not import `appearance.json`.** `load_initial`
   imported mailo's JSON only for non-mailo apps and loaded mailo's own TOML directly. Decided:
   every app, mailo included, imports mailo's legacy `appearance.json` once, the first time its
   own `appearance.toml` does not exist (for mailo the two are in the same directory). One code
   path, `environment::initial_from(dir, mailo_dir)`; test
   `every_app_mailo_included_imports_mailos_json_once`. docs/mailo-migration.md section 4 and
   CONSUMING.md section 3 say so.
4. **Contrast.** Dark `--danger-ink` was white on `#E0705A`, 3.17:1; it is now `#1A0B08`
   (6.06:1; light stays white, 6.03:1). The legibility test checks every `X` / `X-ink` pair in
   both schemes, which needed the pairs to exist: `--ok-ink` (white / `#0B1A12`, 5.21 / 7.15)
   and `--warn-ink` (`#140D03` both, 4.78 / 8.27; white on light `--warn` is 4.03) are new
   `ColourToken`s. design/03 section 3 has the table, settled.
5. **Person colours outside `Avatar`.** `ds::person_hue(name) -> PersonHue` (the existing
   `PersonHue::of`, design/03 section 13's hash), with `PersonHue::hex()` now public and
   `PersonHue::colour()`; `ds::PersonSwatch`, the eight stored-colour swatches as
   `--c-person-1..8` in mailo's `AVATAR` order, declared once (the same in both schemes) and
   known to the lint. design/03 open decision 15 is settled. CONSUMING.md "Person colours".
6. **`SpaceLook` has no motion: not a gap.** design/21 keeps motion global (the
   `Appearance`), and now says so in section 1. No API change: a consumer that wants motion per
   Space passes its Space's motion into the root's `appearance.motion`. docs/mailo-migration.md
   section 3 says this instead of calling it a gap.
7. **`KEEP_FOCUS`.** The brief's section 5.1 said to delete its injection while 5.3 and 5.4
   expected it to work. Fixed: keep the constant and its injection through Phase A; Phase B
   removes it, where `ds_native::focus` (provided by `launch`) takes over (sections 5.1, 5.3,
   6.1, 6.3).

What mailo changes:

- Nothing for item 1; a mailo pixel test that assumed a flat window ground sets `Grain(0)`.
- Delete `space::AVATAR` and `ui/compose/items.rs::hue`: stored account colours come from
  `ds::PersonSwatch::nth(i)` (`.colour()` for `AvatarTone::Account`, or its `var()` in CSS), a
  person with no stored colour from `ds::person_hue(address)` (`AvatarTone::Person`). Note the
  compose recipients currently index `AVATAR` by a byte hash; switching to `person_hue` changes
  their colours to the design's hash.
- Use `Icon::Printer` / `Icon::FolderInput` where mailo kept its own print and move glyphs.
- Text on `--danger`/`--ok`/`--warn` uses `--danger-ink`/`--ok-ink`/`--warn-ink`, not white.
- `appearance.rs`'s startup `load_or_import` is redundant once the window calls
  `use_environment(AppName::MAILO)`; it may stay (it is idempotent) or go.
- Keep `Space::motion` and feed it into the `Ds` root's `appearance.motion`; keep `KEEP_FOCUS`
  until Phase B.

## Pixel snapping (2026-09-25)

The user's 27 inch 4K panel runs at 1.5 (Apple's 109 points per inch); other panels land at 1.25
or 1.75. macOS renders a fractional scale at 2x and downsamples; Blitz renders at the true scale,
so the design system snaps. Branch `pixel-snap`. Proofs: `crates/ds-native/tests/pixel_snap.rs`
(rows read from the PNG at 125, 150, 175 and 200), `crates/ds/tests/pixel_root.rs` (the root's
writes), unit tests in `tokens/pixel.rs`, `icon/stroke.rs`, `geometry/scale.rs` and
`ds-native/src/snap.rs`, lint cases in `tests/lint_rules.rs`.

**What Blitz does (blitz e99fbdbd, taffy 4863877, stylo 0.21), read in the source and measured:**

- Layout runs in logical (CSS) pixels: the stylist's device takes the viewport as
  `window_size / scale` (`resolve_layout` reads `au_viewport_size()`), and `resolve_layout` ends
  with `taffy::round_layout`, which rounds every box's location, size, border and padding to
  whole **logical** pixels (cumulative, `(v + .5).floor()`). There is no option to round
  geometry to device pixels in taffy, blitz-dom, blitz-paint, anyrender or vello_cpu 0.1
  (blitz-paint snaps only a text decoration's thickness, and hints glyphs); paint multiplies
  the rounded layout by the scale (`blitz-paint/src/render.rs`, `layout.border * scale`) and
  rasterises with antialiasing.
- stylo does snap a border (and outline) width to device pixels (`snap_as_border_width`: floor,
  never below one device pixel): `border: 1px` at 1.5 computes to 40 app units, 0.667 logical px,
  one device pixel. taffy then rounds that back up to 1 logical pixel, 1.5 device pixels.
- `box-shadow` spreads and offsets are painted from the computed style, unrounded, relative to
  the (logically rounded) box.
- Measured before the fix, a black line on white, ink per device row (255 is full):

  | Scale | `height:1px` at y 10 | at y 11 | `border:1px` top at y 20 | at y 21 | `height:.6667px` |
  | --- | --- | --- | --- | --- | --- |
  | 1.25 | 128, 191 | 64, 255 | 255, 64 | 191, 128 | 128, 191 (rounded up to 1 px) |
  | 1.5 | 255, 128 | 128, 255 | 255, 128 | 128, 255 | 255, 128 (the same) |
  | 1.75 | 128, 255, 64 | 191, 255 | 255, 191 | 64, 255, 128 | 128, 255, 64 |
  | 2 | 255, 255 | 255, 255 | 255, 255 | 255, 255 | 255, 255 (2 device rows) |

  So no CSS value alone can make a crisp line at a fractional scale: widths below a logical
  pixel are rounded up to one, and positions sit on half device pixels at 1.5 whenever the
  logical coordinate is odd.

**What is snapped, and how:**

- **Positions and widths: `ds_native::snap_to_device(&mut BaseDocument)`** (`src/snap.rs`),
  after every resolve. It redoes taffy's cumulative rounding from each node's unrounded layout on
  the device grid (so neighbours still abut exactly), rounds border widths on their own to whole
  device pixels (never below one, whatever phase the edge lands on), then re-derives what Blitz
  computed from the old rounding: each node's transform (percent translations read the size) and
  scrollable overflow, and rounds a pure translation to whole device pixels. It reaches the
  layout through blitz-dom's public `Node::{unrounded_layout, final_layout_mut, set_transform,
  transform_mut, scrollable_overflow_mut, layout_children}`; no new dependency. Hit testing and
  rect reads use the same final layout, so they agree with the picture. At a whole scale it does
  nothing, so every 1x and 2x picture is unchanged. `Harness` and `snapshot` run it each frame.
- **Widths: the pixel tokens** (`ds::PixelToken`, `tokens/pixel.rs`): `--hair` (1 px lines),
  `--hairline` (.5 px), `--px` (one device pixel), `--ring` (3 px), `--focus-ring` (2.5 px),
  `--dpr`. Tuned tokens: `.ds` declares `--hair:var(--scale-hair,1px)` and the root writes
  `--scale-hair` inline, so nested `.ds` scopes keep the root's value. `Ds { scale: Option<Scale> }`
  (120ths, shell-host's unit) sets them; ds-native provides `ds::HostScale` from the viewport
  (headless) or the window's scale factor (`launch`). At 1x the root writes nothing and the
  generated sheet's defaults are the old values, so 1x markup and pixels are unchanged.
- Converted to the tokens: every 1 px border in quire's component sheets (button, chip, palette,
  drag ghost, hover card foot, hover strip, icon button, kbd, list row, menu tile, popover,
  search field, segmented, space editor, tabs, text input, toggle), the menu separator, the
  section header rule, the selection bubble separator, the provider mark's ring, the text input's
  focus ring and the `dest`/`chip-flash` rings (`--ring`), the keyboard focus outline
  (`--focus-ring`), the material stack's outer hairline, inner edge and top highlight and the
  bar's hairlines (`--hairline`, `--hair`), the plate's rim and highlight, and the `--shadow-1`,
  `-2`, `-current`, `-pill-inset`, `-card`, `-handle`, `-mark` hairlines. The gallery's own sheet
  too. Golden `tests/snapshots/stylesheet.css` re-blessed; the diff is exactly those values.
- **Glyphs**: at a fractional scale `Glyph` writes a stroke width that is an even number of
  device pixels (`icon/stroke.rs`, design/08 §1.4.1); a 16 px glyph is 2 device pixels at
  1.25, 1.5 and 1.75.
- **Measured after**: the rule, a `--hair` card's top and left edges, a literal `border:1px`
  card's (stylo's floor plus the snap), and a Slim menu's separators are one full device row at
  1.25, 1.5 and 1.75 with the ground on both sides, and two full rows at 2; a 16 px plus glyph's
  bar is two full rows at the three fractional scales. With the snap switched off the same test
  fails at 1.25 (rule `64, 255`, separator `191, 128`, glyph bar `191, 255, 64`).
- **Lint**: `Rule::RawHairline` (Strict) flags a literal `1px`/`.5px` `border*`/`outline*` width
  and a box whose whole `width`/`height` is `1px`, naming the token. Known downstream offence:
  sill's `.sill-dock-separator{width:1px}` (it becomes `var(--hair)`); sill's Strict dock lint
  will fail on it until then.

**What is not snapped:**

- `ds_native::launch`'s window (the portable winit path): blitz-shell calls `resolve` and paints
  in the same redraw with no hook between, so positions stay Blitz's logical rounding there; the
  tokens still apply. shell-host resolves its own documents and must call
  `ds_native::snap_to_device` after each `resolve` (shell-host is not changed on this branch).
- A literal `height:1px`/`width:1px` box in a consumer sheet: snapped to whole device pixels, but
  1.5 device pixels rounds to one or two rows by phase; the lint points at `--hair`.
- Transforms that are not a pure translation (a menu's `menu-pop` scale part-way through, a
  hover's `translateY(-1px) scale(1.015)`) keep their fractional positions while they run.
- Text: parley positions glyph runs inside the snapped content box; baselines are not moved.
- A glyph's diagonals and curves, and at 1.25/1.75 its grid lines that do not fall on a device
  boundary (only every sixth grid line does at 16 px); a 24 px glyph's grid lines at odd
  coordinates at 1.5. Symbolic external icons (a PNG/SVG mask) are resampled as before.
- `--shadow-mark-tile` (1.5 px) and the sidebar's 1.5 px inset ring, box-shadow offsets and
  blurs: not lines, left as designed.
- Scroll offsets are fractional and applied at paint; a scrolled list's rows move by whatever
  the offset is.

## mailo gaps 2 (controls and tiles) (2026-09-25)

mailo's component migration reported the props and variants it still hand-rolls controls for.
Branch `mailo-gaps-2b`. Every change is additive unless an item says the markup changed. Proofs:
`crates/ds/tests/mailo_gaps_ssr.rs` (goldens beside each component's existing ones, so the
controls' and lists' class scans cover them) and one Harness test file per behaviour in
`crates/ds-native/tests/`. CONSUMING.md "The mailo gaps 2" has one row per new prop.

1. **Button and TextInput.** `Button` takes `title`, `aria_label` and `expanded:
   Option<Expanded>` (`Expanded::{Open, Closed}`, new in `vocab`: a trigger's open state is a
   different fact from a toggle's pressed state, so it is not a `Switch`). `IconButton` already
   had all three (`tooltip`, `label`, `expanded: Option<Switch>`) and is unchanged; its
   `expanded` stays a `Switch` because changing it would break every caller. `TextInput` takes
   `onfocus`/`onblur: EventHandler<()>` and `kind: TextInputKind::{Text, Password}`. A range is
   the existing `Slider` (thousandths, keys, drag), so there is no range kind.
   - **Blitz dispatches no focus event for a programmatic focus.** A click or Tab goes through
     blitz-dom's `generate_focus_events` (blur on the old node, focus on the new); the focus
     seam's `set_focus_to` (ds-native's `HostFocus`, and dioxus-native-dom's own `set_focus`,
     whose source says "TODO: queue focus events somehow") changes the focused node silently.
     So a `Focus::OnMount` or `Focus::Controlled` field on Blitz never heard its own focus.
     Fixed for the field itself: `focus::host::focus_soon_told` calls the field's `onfocus` once a
     *host's* write succeeds; without a host (the webview) the renderer's real event fires and
     `told` is not called, so the caller hears it once either way. Not fixable here: the field
     that *lost* the caret to a seam focus gets no `blur` on Blitz. A caller that tracks "typing"
     with one flag is unaffected (the new field's focus sets it); one that tracks each field
     must treat a focus elsewhere as the other's blur. Proof: `field_focus.rs`
     (`a_click_and_the_seam_both_report_focus_and_a_click_reports_blur`,
     `a_controlled_request_still_focuses_and_reports_it`); with `told` disabled both fail.
   - **Blitz draws a password in clear.** blitz-dom lays `type="password"` out as a text editor
     and paints its characters as typed (no masking anywhere in blitz-dom or blitz-paint at the
     pinned rev). The field therefore paints its own text transparent (`caret-color` keeps the
     caret in `--ink`) and lays `.ds-input-mask`, one `•` per character, over it where the
     placeholder would sit. The caret follows the hidden text's advance, so it can sit a little
     off the last dot; the secret never paints. A browser masks natively, and its (transparent)
     dots sit under the same overlay.
2. **AccountTile.** `mark: MarkStyle` (default `Letter`, so every current tile is as it was)
   is handed to the tile's `ProviderMark`: mailo's provider-marks setting reaches the tiles.
   The Add account face is its own component, `AddAccountTile { label, title, onclick }`,
   rather than an `AccountFace::Add` variant: an add tile has no `pressed` and no `unread`, and a
   variant would have made both required for it and broken every `match` on `AccountFace`.
   design/04 section 27 does not draw it; mailo's `.acct-add` does (a dashed `--f-ink-faint`
   ring around a plus, `--f-ink` on hover), and quire takes that, on a Pin with no ground at
   rest. That rule has to name `.ds-icon-button.ds-account-tile`: `icon_button.css` comes after
   `account_tile.css` in the cascade, so an equal-specificity rule lost to the Pin's ground.
   Proof: `account_tiles.rs::the_add_tile_has_no_plate_at_rest` (the add tile's top edge is the
   window ground, the account tile's is its plate; with the one-class selector it fails) and
   `the_add_tile_presses_and_is_never_pressed`.
3. **SendPill.** `mood: SendMood::{Calm, Nudge, Shake, Fatal}` (named `SendMood`, not `Mood`,
   in the flat `ds::` namespace), `action: PillAction::{Undo, Cancel, Nothing}` (`Nothing`
   because mailo's failed and in-flight faces offer no button), `ring: SendRing::{Drain, Spin}`
   and `refusal: Option<String>`; each defaults to the old markup. A failed mood fires the pulse
   machinery's A/B alias (`a-nudge`, `a-shake`; `Fatal` shakes as `Shake` does and turns
   `--danger`) only when the mood *changes*, never on mount (the pill is still springing up, and
   a keyframe on `transform` would fight that transition), and is put back at rest at
   `settle(anim)` (554 / 594 ms Standard). The last mood lives in a plain value, as `Count`'s
   last value does; only the settle timer writes a signal (the finished firing's round, only
   ever forward, so a slower shake from an older firing cannot reopen a newer nudge). mailo's
   own CSS looped each twice at its own durations; quire plays each once at design/05's, per
   principle 7. The spin ring turns the whole `svg` at `--t-spin` (CSS cannot reach inside it on
   Blitz, S6), with a 20/37 dash written as attributes. Proof: `send_pill_moods.rs`
   (`a_nudge_plays_once_settles_and_the_pill_stays`: playing 100 ms before `settle(Nudge)`, at
   rest 50 ms after, still `data-shown=shown`; `a_mood_that_returns_plays_again` on alias `b`;
   `a_spinning_ring_turns_and_a_draining_one_holds`: the ring's pixels 250 ms apart differ
   for Spin and are identical for Drain).
4. **SidebarItem.** A Today item's close button is named `Close {label}` (it was `Close` on
   every row, so a screen reader heard a column of identical buttons). This is the first of the
   two changes to existing markup in this wave (the brief asked for both): the `today-entering`, `today-present` and `today-leaving`
   goldens were re-blessed and differ only in that attribute. `trailing: Option<TodayTrailing>`
   (`time`, `cancel` as the button's accessible name, `on_cancel`) draws a scheduled row's time
   and its cancel after the label; the cancel stops propagation, as the close does, so it does
   not also open the row. A typed struct rather than an `Element` slot: the row owns the markup
   (a slot would let a consumer put a raw button inside a `role=button` div). Proof:
   `today_trailing.rs` (cancel then label logs `cancel,open`; the close is `Close Q3 notes`) and
   the `today-scheduled` golden.
5. **SpaceEditor.** `on_rename: Option<EventHandler<String>>` turns the title into an inline
   `TextInput` holding `name` (the title's face and size, not the field's body size);
   `motion: Option<MotionChoice { level, on_motion }>` adds a Motion row after Appearance;
   `measured: MeasuredIn::{ThisScheme, EachScheme}` measures each scheme the Space's theme can
   show, each under a small-caps heading (`schemes_of`: System both, Light or Dark its own).
   Each defaults to the old markup. The Theme row stays the editor's own
   `SegmentedControl<Theme>`, not `AppearancePicker`. `Preset` gains `name` (Dusk, Orchard,
   Harbour, Ember, Lagoon, Heather, Moss, Stone: mailo's names in design/21's order, now in its
   table), and each preset button is named by it: the second change to existing markup in this
   wave (the five `space_editor` goldens differ only in the preset buttons' `aria-label`, now
   the name, and a new `title`). The file was split to stay short: the field and its handles
   are `space_editor/handles.rs`, `SpaceDot` is `space_editor/dot.rs`, the new rows
   `space_editor/rows.rs`, and `Checks` became a header over `CheckRows` so each scheme reuses
   the rows.
   - **The Motion row is over `Motion`, not `MotionLevel`, as the brief named it.** `MotionLevel`
     is the resolved level (`Calm`, `Standard`, `Extra`, `Reduced`: no `System`, no label); what
     a person picks, and what a root takes (`Appearance { motion: Motion }`, and what mailo's
     `Space::motion` feeds per the first mailo gaps item 6), is `Motion`, which has `System` and a
     `label()`. A row over `MotionLevel` could not offer "follow the desktop".
   - Not done: mailo's six extra presets (the retired accents) are not added to quire's
     `PRESETS: [Preset; 8]`; design/21 names eight, and changing the array's length would break
     every consumer that indexes it by workspace.
   Proof: `space_editor_rows.rs` (typing `s` in the title makes the Space "Works"; picking
   Reduced reports `reduced`; a System Space shows Light and Dark headings over eight checks),
   the `rows-system`, `rows-dark` and `rename-unnamed` goldens, and
   `presets::the_presets_are_named_in_the_design_order`.

What mailo changes (docs/mailo-migration.md section 2 has the row for each):

- Raw buttons with a title, an assistive name or an open state become `Button { title,
  aria_label, expanded }`; `Field` becomes `TextInput { kind, onfocus, onblur }` (the typing
  guard hangs off the two handlers) and its `Range` kind becomes `Slider`.
- `AccountTiles` passes its provider-marks setting as `mark`, and its "+" is `AddAccountTile`.
- `compose/pill.rs` maps `Mood`, `Offer` and `Ring::Spin` onto `SendMood`, `PillAction` and
  `SendRing`, and `.sp-why` onto `refusal`; its own nudge and shake CSS goes.
- `compose/later.rs`'s scheduled rows are `SidebarItem { trailing }`; a test that looked for a
  Today close named "Close" looks for "Close {label}".
- The Space editor's name, Motion and per-scheme readout are `on_rename`, `motion` and
  `measured`; a test that found presets by "Preset n" finds them by name.

## mailo gaps 2 (lists and overlays) (2026-09-25)

mailo's second migration wave moves its rows, strip, command panel, menus and hover cards onto
quire and reported what they could not express. Branch `mailo-gaps-2a`. Every change is
additive: a prop defaulting to the old behaviour, a new variant or a new type; no existing
component golden changed (the stylesheet golden gained the new rules and lost two comment
lines). Proofs: `crates/ds-native/tests/mailo_lists.rs`, `mailo_palette.rs`, `mailo_menu.rs`,
`mailo_hover.rs` (Harness), the goldens added under `tests/snapshots/lists/` and
`tests/snapshots/overlays/` (`crates/ds/tests/lists/mailo.rs`, `overlays/mailo.rs`), unit
tests beside each new module. Each proof was checked by removing the change it proves and
watching it fail (the strip's and the row action's stops).

1. **Rows and the strip** (`ListRow`, `HoverStrip`).
   - `subject` and `snippet` are `ds::Text` (`text_runs.rs`): `Plain(String)` or `Runs(Vec<Run>)`,
     `Run { text, tone: RunTone::{Plain, Mark, Strong, Faint} }`, drawn as `mark.ds-mark` (the
     soft accent ground mailo's `.row mark.hit` had) and `span.ds-run[data-tone]`; a plain run
     is a bare text node, so a plain `Text` renders the markup a `String` did (every existing
     golden is byte-identical). `subject` is `#[props(into)]` and `Text: From<String>, From<&str>,
     From<&String>`; `format!` in `rsx!` is a `String`. `snippet: Option<Text>` takes a string,
     `None` or a `Text` through two `SuperFrom` impls under quire's own marker (as `Press` does
     for `EventHandler<()>`). **Not** `Some(String)`: accepting both `Option<String>` and
     `Option<Text>` would leave a bare `None` uninferable, and `None` is the commoner call
     (every in-repo caller writes either a `String` or `None`). The one doc example that wrote
     `Some("…".to_owned())` now writes the string. A `Display` value that is not a string no
     longer converts for `subject` (none exists in this repo or mailo).
   - Blitz drops the whitespace at the end of an inline element's box: a faint `Re: ` span
     before a mark drew as `Re:UIDL` in the gallery (a bare text node keeps its space). A toned
     or marked run's leading and trailing spaces are therefore drawn as text nodes outside its
     element (`edges`, table-tested); the SSR shows `<span …>Re:</span> <mark …>`.
   - `list_row.rs` passed 300 lines with the new props, so the star (`row_star.rs`) and the
     click snapshot (`row_click.rs`) moved out, unchanged; the hooks are `row_hooks.rs`.
   - `PartHooks { onpointerenter, onpointerleave }` (`EventHandler<PointerEvent>`, the event
     itself, so the caller reads the point as mailo's `corner(&event)` does) on the name
     (`on_sender`) and the time (`on_time`); `onpointerenter`, `onpointerleave` and
     `onpointerdown` on the row. The brief named only the row's enter and press; the thread
     card closes on the row's leave, so the leave is there too. Listeners are always attached
     and call nothing without a handler: SSR writes no listener, so the markup is unchanged.
     Proof: the pointer over the name, the subject, the time and away logs
     `row-enter,sender-enter,sender-leave,time-enter,time-leave`; a press logs `row-down`. The
     strip, when revealed, covers the time (absolute, right 8): the test's row has none.
   - `aria_label: Option<String>` on the `li`.
   - `HoverStrip { shown: Option<Shown> }` (the tooltip's `Shown`): `data-shown=visible` shows
     the strip and pops its buttons exactly as `.ds-row:hover` does; `hidden` holds it down under
     the pointer. This closes design/04 O-24 for Blitz, which never matches `:focus-within`
     (S12): the caller reveals the strip on its keyboard row. Proof: a revealed strip takes a
     press with the pointer never over the row; without the CSS the press lands on the row.
   - A strip button's click calls `stop_propagation` (the star already did). This is the one
     behaviour change for a current caller: a strip click used to open the row too. Proof: a
     click on the revealed strip's button logs `archive` and not `open`; with the
     `stop_propagation` removed both tests fail (`open` is logged). Only the click stops: a press
     still reaches the row's `onpointerdown` (mailo dismisses its card there).
   - `title` from the label: behind `titles: Titles::{Omitted, FromLabel}` (default `Omitted`),
     not always on, because every existing row golden carries a strip and the brief keeps
     current markup; mailo passes `FromLabel`. On the webview a `title` is a native tooltip beside
     the Fly; on Blitz it draws nothing.
   - `expanded` is a strip prop, `Vec<(ActionId, Expanded)>`, not a field of `StripAction`: a
     new public field would break every `StripAction { .. }` literal. `Expanded::{Open, Closed}`
     is the controls wave's (`Button { expanded }`), reused rather than a second type for the
     same attribute; a listed button also gets `aria-haspopup="menu"`.
   - Gallery: Lists page, "Search hits and a keyboard-shown strip".

2. **The command panel** (`CommandPalette`, `MenuEntry`).
   - Neither existing entrance gives an opaque first frame: `peek-in` and `cmdk-in` both start
     at `opacity:0`, and over a window the wrap's `fade` starts at 0 too (mailo's test fails on
     either). Reworking `cmdk-in` would change every current caller's entrance, so it is a new
     variant, `PaletteEntrance::Opaque`, playing a new keyframe `cmdk-rise` (`Anim::CmdkRise`,
     `--t-big --e-spring` like `cmdk-in`): `cmdk-in`'s three stops with the `opacity`
     declarations removed, so only the scale and lift spring. With it, an overlay palette's wrap
     carries `data-entrance=cmdk-rise` and does not fade (the wrap's opacity multiplies the
     card's); the scrim appears at once. `Anim::ALL` grows to 48. Proof
     (`mailo_palette.rs`): on the first frame the middle of the card differs from the bare page
     in over half its pixels with `Opaque`, and in under 1 % with `CmdkIn`. The palette's
     module passed 300 lines, so its host and entrance moved to `palette_host.rs`
     (`CommandPaletteHost` and `PaletteEntrance` are still re-exported from `command_palette`).
   - Runs in a row: `MenuEntry::Row(MenuRow<T>)`, a new variant rather than a new field of
     `Item` (a field would break every `MenuEntry::Item { .. }` literal; a `String` title
     cannot become `Text` in a literal). It is the one compile change of this wave: a `match`
     over `MenuEntry` with no wildcard needs an arm (sill has two, `dock/menu_geometry.rs` and
     `bar/content.rs`'s test; quire's gallery had one). `Submenu` and `Info` were added the same
     way. A `Row` is a choice exactly as an `Item` is (picked, navigated, filtered on its plain
     text); a plain title takes the query's marks, runs keep the caller's. `menu_entry.rs`
     now holds only the data: an item's drawing moved to `menu_item.rs`, unchanged but for the
     words (`Words::{Str, Text}`) and the action.
   - `RowAction { icon, label, on_press: EventHandler<Press> }` on `MenuRow::trailing`: a Strip
     `IconButton` in `span.ds-menu-action`, whose row gets `data-trailing=action` (a fourth grid
     column; 22 px buttons in the text menus). The span stops the click (no pick), the
     press and release (no press-drag-release pick, focus stays), and the pointer moves (the
     selection does not follow the pointer onto the row). Proof: the second row's × logs
     `remove:2` after `select:0` and nothing else; with the click and move stops removed the log
     is `select:0,select:1,remove:2,close,pick:2`.
   - Gallery: Overlays page, "Command panel: opaque entrance, runs, trailing actions".

3. **Menus** (`Menu`).
   - `active: Cursor::{Auto, Controlled(Option<usize>)}` (`menu_cursor.rs`, pure, with its
     table tests). The menu machine calls its highlight the tracker's selection; `Cursor` is the
     prop's name for whose it is. Under `Controlled` the drawn highlight is the caller's
     (clamped; `None` draws none, a new state: `Drawn::selected` became an `Option`), Up and Down
     become `on_active(Some(next))` requests (from an end when nothing is highlighted), the
     pointer over another choice asks the same, and Enter picks the caller's choice. It also
     does not take the keyboard as it opens: a field that drives the cursor must keep it, which
     is the whole case (the composer's `/` and `@`). A submenu's panel keeps its own cursor.
     Under `Auto`, `on_active` hears each change after the render that made it. Proof
     (`mailo_menu.rs`): Down, Down, Up in the field move the highlight Dana, Priya, Sam while the
     field keeps the focus; the pointer over the last row logs `active:Some(3)` and the highlight
     stays where the page left it.
   - `onquery: Option<EventHandler<String>>`: the typed filter's text on every change. Proof: an
     own-cursor menu logs `active:Some(0),active:Some(1),query:m,active:Some(0),query:me,query:m`
     for Down, `m`, `e`, Backspace (the filter resets the highlight to the first match).
   - The trailing action is the palette's `MenuRow::trailing`. Proof: the second person's ×
     logs `remove:1`; the menu is still open and nothing was picked.
   - `menu.rs` passed 300 lines: `MenuKind` and `MenuEntrance` moved to `menu_kind.rs`
     (re-exported from `menu`, unchanged).
   - Gallery: Overlays page, "Menu driven by a field".

4. **Hover cards** (`HoverTarget`, `HoverKind`).
   - `HoverTarget { as_: TargetElement::{Span, Div, Li} }` rather than hooks the caller
     spreads: the handlers (`mouseover` with the innermost-wins stop, `mouseleave`,
     `pointerdown`, the mounted element the anchor book measures) stay quire's, in one `Hooks`
     value shared by the three `rsx!` branches, so the intent machine cannot be half-wired by a
     caller that forgets one. `Contents` is left out: a `display:contents` wrapper has no box,
     so `Anchors::record` would read an empty rect and the card would open at the origin. A
     `div` or `li` target carries `data-as` and is `display:block` (a type selector would trip
     the consumer-only `DsInternals` exemption, which keys on a selector starting `.ds`). The
     span's markup is unchanged. `HoverTarget` moved to `hover_card/target.rs`. Proof
     (`mailo_hover.rs`): three `li` targets in a `ul`; the second opens its side card after the
     wait (none at 400 ms), 10 right of the item and 6 above it, and moving to the third while
     warm switches the card within 60 ms.
   - `HoverKind::Tip`: design/06 section 3's `time` kind. It uses the hub's intent timing (450
     ms, 0 warm, 150 ms close, 400 ms warm window), the same as every card and as the Card
     tooltip (which already goes through the hub), not a timing of its own: the tooltip has no
     other. Placed like `Sender` (below, 6); drawn by `.ds-hovercard[data-kind=tip]` at the
     Card tooltip's size (auto width to 260, `--s-6`/`--s-10` padding, `--fs-shell-tip`,
     `--r-item`) on one line (`nowrap`). Adding a variant breaks a `match` over `HoverKind` with
     no wildcard (none in this repo, sill or mailo). The `Tooltip { Card }` markup is unchanged
     (it still files its target as `Sender`). Proof: a time's tip opens under its left edge,
     at most 260 wide and one line tall.
   - Gallery: Overlays page, "Hover cards and tooltips" gains a time tip and two `li` targets.

What mailo changes:

- Rows: `ListRow { subject: Text::Runs(..), snippet, on_sender, on_time, onpointerenter,
  onpointerleave, onpointerdown, aria_label, strip: HoverStrip { shown, titles:
  Titles::FromLabel, expanded } }`; drop the strip's own `stop_propagation`.
- The command panel: `entrance: PaletteEntrance::Opaque`; recent searches as
  `MenuEntry::Row(MenuRow { trailing: Some(RowAction { .. }), .. })`.
- The composer's menus: `active: Cursor::Controlled(..)`, `on_active`; the label menu's create
  row from `onquery`.
- Hover: sidebar entries as `HoverTarget { as_: TargetElement::Li }`, the time's tip as
  `HoverKind::Tip`.


## mailo gaps 3 (motion, tokens, lint) (2026-09-25)

mailo reported seven more gaps against quire. Branch `mailo-gaps-3`, one commit per item.
Everything is additive or a bug fix; the exceptions are named below. CONSUMING.md "The mailo
gaps 3" has one row per change, docs/mailo-migration.md section 2 the row for each mailo site.

1. **An exit could not be taken back.** `RosterState::stay(key)` turns a `Leaving` row back to
   `Present` in place and answers `Result<Stayed, StayError>`: `Stayed::Restored`, or
   `Stayed::Unchanged` for a row that is entering, present or healing (a healing row keeps
   healing: it is not leaving); `StayError::UnknownKey` for a key the roster no longer holds (its
   exit settled; the consumer lists it again and it enters), never a panic. `Roster::stay` does
   the same through the hook and cancels the row's settle timer: the heal of the rows below only
   ever starts at that settle, so nothing below heals, and a row folded again after a stay
   settles on its own new timer (the hook now keeps one `ExitTimer { key, task }` per leaving row,
   and a second `leave` of the same key replaces the first timer). Proofs:
   `motion_machines.rs::stay_takes_an_exit_back_in_place` (nine cases, including a late settle
   after a stay changing nothing), `ds-native/tests/roster_stay.rs` (a fold stayed 200 ms in: the
   row is `present` with no `data-exit`, and 1 s later all three rows are present, carry no
   `--dy`, and sit where they sat; and a row folded, stayed and folded again drops only at its
   second fold's settle, which fails with the cancellation removed).
2. **The rest timer was spawned from the component body.** `use_roster` reconciled and spawned
   its rest timer in the render. Reconciling stays in the render (the rows to draw must be right
   in the render that lists them) and is pure; the timer is now queued with `queue_effect`, which
   dioxus runs after the render on every renderer, and the effect spawns it as a task of the
   roster's owner (`task::spawn_in`, sill FINDINGS Q45). A `RestQueue` flag queues the effect once
   however many reconciles happen before it runs, and a pending rest due at or after the new one
   is kept rather than joined by a second task (an earlier one is cancelled and replaced), so
   there is never more than one rest task. The settle timer's own reschedule runs in a task and
   spawns directly. Proofs: `motion/roster_rest/tests.rs` (after the first render nothing is
   spawned, after its effect one timer; two reconciles in two renders before the effect spawn one
   timer, which fails with the flag removed); the Harness list and launcher-crash tests, and the
   roster hook test in `motion_machines.rs`, are unchanged and green.
3. **Four keyframes.** `Anim::PillUp` (`pill-up`, `translate(-50%,160%)` to `translate(-50%,0)`,
   `--t-big --e-spring`), `RingDrain` (`ring-drain`, `stroke-dashoffset` 0 to 57, the SendPill's
   dash, `--t-send-ring` linear, forwards), `FadeIn` (`fade-in`, to `--veil`, `--t-move --e-out`)
   and `Busy` (`busy`, 1 to .45 and back, `--t-ambient --e-in-out`, infinite), in `motion.css`
   with their `--b` aliases, their recipes (the quire-added recipes now live in
   `motion/recipe_own.rs`), their pulse classes and settle entries in the drift table. `Anim::ALL`
   is 52. `CmdkRise` is mailo-gaps-2a's (`PaletteEntrance::Opaque`) and is reused, not added
   again; note its shape is `cmdk-in`'s (.93, -12 px, overshoot), not mailo's gentler 8 px / .98.
   - **`--t-send-ring` is now a hold.** The ring is the undo window, time a person has to act,
     so Reduced no longer shortens it to 60 ms (the Reduced block loses `--t-send-ring:60ms`;
     `settle(RingDrain, Reduced)` is 5034 ms, like `ChipFlash`'s hold).
   - **`fade-in` stops at `--veil`, not at `--scrim`'s alpha.** `--scrim` is a colour, black at
     .22, painted at full opacity and faded in by `fade`; C's veil (mailo's `.scrim`) is `--ink`
     at opacity .16. The .16 is a new token, `OpacityToken::Veil` (`--veil`), declared on `.ds`
     and known to the lint, so the keyframe and the element that rests there agree.
   - **PillUp is not applied to `SendPill` and `Toast`.** Neither plays a translate keyframe: each
     slides by a `transform` transition on `data-shown`, which also carries it back down when it
     hides (`SendPill` at `SentHold`, a toast on dismissal). A keyframe entrance would replace only
     the rise and lose nothing only if the exit were rewritten too; their goldens and Harness
     tests are unchanged. PillUp is for a pill a consumer draws itself (mailo's list toast).
4. **The contact keyframes kept their shape under Calm.** `gulp`, `bump` and `seal-pop` now scale
   each departure from rest by `min(1, (var(--overshoot) - 1) * 25)`: 1 at Standard (1.04) and at
   Extra (1.14, capped: Extra's `--e-spring` already overshoots harder, and an uncapped bump
   would reach 1.875), 0 at Calm and Reduced. Standard's frames are the catalogue's exactly.
   Blitz (stylo) resolves `calc()` with `min()` and `var()` inside a keyframe's `scale()` and
   `translateY()`. Proofs: `ds-native/tests/contact_motion.rs` measures a 100 px ink square on the
   painted frame at each keyframe's offset: Standard gulp 107 x 84, bump 125 x 125, seal-pop
   150 x 150; Calm 100 x 100 for all three (before the change Calm painted 106 x 84, 124 x 124 and
   150 x 150); Extra bump 125 x 125. `motion_css.rs::every_contact_keyframe_follows_the_motion_level`
   asserts every spring that answers a contact (pop-in, row-in, compose-rise, chip-in, cmdk-in,
   gulp, bump, seal-pop) reads `--overshoot`.
5. **SpaceDot wrote a literal colour inline.** `SpaceDot` wrote `background:linear-gradient(…#hex…)`
   in its `style`, which `lint::markup` flags as `HexColour` on any element; so did the Space
   editor's presets, title swatch, handles and stop discs (the gallery excepted all five). Each now
   writes its colours as custom properties, `--dot-c1`, `--dot-c2`, `--dot-c3`, with `data-stops`
   counting a gradient's stops, and `space_editor.css` paints the same 135deg gradient
   `space::gradient` writes (flat for one stop, 0/100 % for two, 0/50/100 % for three); a handle
   and a disc are `background:var(--dot-c1)`. `FrameVars` gains `stops: Vec<String>` so the dot has
   its stops (a new public field: a struct literal of `FrameVars` would need it; nothing
   downstream builds one). Proofs: the gallery's `the_space_page_is_clean_under_strict` (the
   whole Space page under `Profile::Strict`, no exception; the gallery's exception list is now
   empty), `ds-native/tests/space_dot_paint.rs` (a two-stop and a three-stop dot, pixel for pixel
   against a box painted with the old inline gradient: every pixel within 2), the re-blessed
   `lists/space_editor/*` goldens (the diff is exactly the `style` attributes and `data-stops`).
6. **A serif face and a muted avatar.** quire shipped no serif. `--font-serif` (`Family::Serif`;
   `Family::ALL` replaces the three-element lists) is Noto Serif 2.015, OFL 1.1
   (`assets/fonts/OFL-notoserif.txt`), cut from the variable files Fedora ships (weights 400-700,
   upright and italic) into the same latin and latin-ext subsets as the others, as WOFF2 for the
   webview and TTF for Blitz: 91 KB, 315 KB, 100 KB and 336 KB of TTF (latin-ext carries most of
   the weight; the other faces' TTFs total 480 KB). The stack falls back to Georgia and Times New
   Roman. `scripts/subset-fonts.sh` takes file names now, names the Noto family, and drops the
   WOFF2 flavour itself (a current pyftsubset keeps the input's flavour, which wrote WOFF2 bytes
   into a `.ttf`). Proofs: `fonts::tests` (both subsets per face, every face an sfnt),
   ds-native's `every_family_resolves_after_registration` (Noto Serif registers), the lint cases
   for `var(--font-serif)`.
   - The muted avatar is a prop, `Avatar { muting: AvatarMuting::{Plain, Muted} }`, not an
     `--avatar-muted` token: one grey token would paint every account not in view alike and drop
     the one fact the colour states (which account), while the muted colour keeps the hue at .55
     of its chroma, lightness kept (S's `saturate(.55)`, computed because Blitz paints no
     `filter`; the arithmetic moved out of `AccountTile` into `components/muted.rs`, which
     `AccountTile` still uses for an unpressed tile, unchanged). Greyscale tones are untouched.
     Proof: `legibility.rs::a_muted_avatar_is_as_legible_as_a_plain_one`, over the eight swatches
     and a person hue every 5 degrees, both schemes: the `--on-hue` letter on a muted disc is at
     least 3.0 (lowest 3.22) and within .3 of the plain one, and the faintest muted disc stands off
     `--paper`, `--surface` and `--raise` at least as far as the faintest plain one (light 2.70 /
     3.05 / 3.22 against 2.68 / 3.02 / 3.19; dark 2.13 / 1.95 / 1.63 against 2.08 / 1.90 / 1.59).
     Goldens `controls/avatar/{account-28,person-18,ink-28}-muted.html`.
7. **Docs against source.**
   - CONSUMING section 5 said `assert_clean` panics on a stale exception; it only printed the
     counts. It now panics, naming each exception that suppressed nothing, unless the new
     `LintConfig.stale` is `Stale::Report` (default `Stale::Fail`), which prints and passes. This
     is the one change that can break a consumer's build: a `LintConfig` literal naming every
     field needs `stale` (or `..LintConfig::default()`). `stylesheet` and `markup` are unchanged:
     they return offences and never judge exceptions.
   - `Rule::UnknownAnimation` read only `animation-name`. It now also parses the `animation`
     shorthand (`lint/animation.rs`): split at top-level commas, each animation's name is its
     first identifier that is not a shorthand keyword, with times, counts and functions skipped;
     a name in a string or behind a `var()` is not judged. Cases in `lint_rules.rs` (a made-up
     name first, after keywords, second in a list; quire's names, an alias, raw easing words,
     `none`, a `var()`).
   - **What these newly flag downstream, checked read-only before the change.** sill
     (`/home/pohsuanlai/sill` at c1b8f6d): its four `assert_clean` calls
     (`sill-surfaces/tests/{bar,dock,launcher,wallpaper}_lint.rs`) and its tray's `markup` test
     pass no exceptions, so none can go stale; its six stylesheets (`sill-surfaces/src/style/
     {bar,bar_popup,dock,dock_popup,launcher,wallpaper}.css`) contain no `animation` at all, so the
     shorthand parse flags nothing. Every sill `LintConfig` is built with `..LintConfig::default()`
     and compiles as it is. `examples/consumer`: no exceptions, no `animation` in `style.css`,
     `..LintConfig::default()` throughout; nothing newly flagged. quire itself: no `assert_clean`
     caller had a stale exception, and every shorthand in quire's and the gallery's sheets names a
     known keyframe. mailo (not asked, read for the migration doc): `ui/style/mod.rs:815` builds a
     `LintConfig` with all three fields and will not compile until it adds `stale` or
     `..LintConfig::default()`; every name its `animation:` shorthands use is one quire plays after
     this branch (the five it defined itself, `pill-up`, `ring-drain`, `fade-in`, `busy` and
     `cmdk-rise`, are now quire's names), so the shorthand parse flags nothing there either; its
     `button.ds-space-dot` markup exception becomes stale with item 5. Its own `@keyframes` of
     those five names should be deleted when it adopts: under the same name its sheet's frames
     would shadow quire's, and its `cmdk-rise` is not quire's shape.

What mailo changes (docs/mailo-migration.md section 2 has each row):

- Delete its `@keyframes pill-up`, `ring-drain`, `fade-in`, `busy` and `cmdk-rise` and their
  `Keyframes` exceptions; the ring's `5s` becomes `var(--t-send-ring)`, the scrim's `.16`
  `var(--veil)`.
- `.bubble .serif` is `font-family: var(--font-serif)`; the muted account avatar is
  `AccountTile`'s own or `Avatar { muting: AvatarMuting::Muted }`, and the `filter` exception goes.
- An undo during a row's exit calls `roster.stay(key)` and lists the key again.
- The `LintConfig` at `ui/style/mod.rs:815` gains `..LintConfig::default()`; the SpaceDot markup
  exception goes.

## mailo gaps 4 (controls) (2026-09-25)

mailo, on v0.1.5 with 31 raw controls left, reported what still keeps them its own. Branch
`mailo-gaps-4b`. Every change is additive: a prop that defaults to the old markup, or a new
variant (a `match` over `Provider`, `ButtonVariant`, `InputVariant` or `TextInputKind` needs the
new arm). No existing golden changed except `stylesheet.css`, which grew by the new rules.
Proofs: `crates/ds/tests/mailo_gaps4_ssr.rs` (goldens beside each component's, so the controls'
and lists' class scans cover them), `crates/ds-native/tests/mailo_fields.rs` and
`motion_levels.rs`, and `crates/ds/tests/snippet_forms.rs`. CONSUMING.md "The mailo gaps 4 (2026-09-25): controls" has
one row per prop.

1. **A local-folders account.** `Provider::Local` rather than a new `AccountFace` variant or an
   `Option<Provider>`: every `AccountFace::One { initial, colour, provider, address }` literal
   keeps compiling, and a local account is drawn wherever a provider is (tiles, rows' via,
   inline). Its mark is the `folder` glyph (10, 8 or 9 px by the mark's size) stroked in IMAP's
   neutral `#5D6660` on the same white chip, `data-kind="local"`, titled "Local folders"; not a
   letter, since there is no provider to abbreviate. It has no favicon, so `MarkStyle::Image` is
   ignored for it. Unnamed tiles read "L, Local folders account". Goldens `one-local`,
   `local-row`, `local-image-ignored`.
2. **Button.** `ButtonVariant::Frame` is the sidebar item's chrome for a word (6px 8px,
   `--r-item`, 13.5 / 600, `--f-ink-soft`; `--f-pill-hover` hover; `--f-pill` held, and with
   `--shadow-current` when `aria-pressed`). `trailing: Option<Trailing::{Caret, Glyph(Icon)}>`
   puts a glyph after the label in `span.ds-button-trail` (the caret is a 12 px chevron at .7).
   `face: ButtonFace::{Label, Bold, Italic, Underline, Strike}` draws the bubble's marks as
   `span.ds-button-face[data-face]` holding `B`, `i`, `U`, `S`, aria-hidden, and the button is
   then named by `label` through `aria-label` (an explicit `aria_label` wins). The italic is
   `--font-serif` (S's Georgia, which design/02 left open and mailo gaps 3 shipped). An enum
   rather than a `Mark(Element)` slot: a slot would let the consumer put raw `b`/`i` markup back.
   Because `BubbleButton`'s `label` is already an `Element` (and a new field would break its
   literals), the same face is exported alone as `FaceMark { face, label }` for the bubble.
   Blitz paints `text-decoration` underline and line-through (blitz-paint's `text.rs` draws both
   from the computed style), so the U and S faces need no drawn rule. Goldens `frame`,
   `frame-pressed`, `quiet-caret`, `mini-trailing-glyph`, `face-*`.
3. **TextInput.** The component was split to stay short: `text_input_focus.rs` (`Focus` and the
   request bookkeeping), `text_input_kind.rs` (`TextInputKind`, `Rows`, `Grow`, the mask) and
   `text_input_parts.rs` (the line, the textarea and the file shapes).
   - **Secret.** `TextInputKind::Secret`, not a change to `Password`: `Password` writes its
     `value` (the wave-2 goldens `password-empty`/`password-filled` and mailo gaps 2's docs say
     so) and changing it would have broken both. A secret keeps what is typed in a signal of its
     own, draws one dot per character from it, and never writes a `value` attribute: on Blitz the
     text lives only in blitz-dom's editor (an absent `value` is never set, so the editor is never
     reset), on a webview in the DOM's own password field. `oninput` hears each change and
     `onchange` the commit. Its `value` prop is ignored; to clear it, remount under a new `key`.
     Proof: the `secret-value-unwritten` golden (given `hunter2`, the markup has no `value`), and
     `a_secret_is_typed_and_heard_but_never_written_into_the_markup` (typing `abc` is heard as
     `a,ab,abc`, the input has no `value` attribute before or after, the mask is three dots, the
     field's markup never contains `abc`, and Enter commits `abc`).
   - **`onchange`.** blitz-dom dispatches no `change` event at all (its text input generates only
     `input`, selection and an implicit submit), so the field makes its own: Enter in a one-line
     field, or blur of any field, calls `onchange` with the value (a secret's own). It is the
     same on the webview, since the native `change` is not listened to.
   - **File.** Blitz has no file picker. blitz-dom has a `file-input` feature (off in quire's
     pinned features); even on, it only draws a "Browse" button and a label, and the files come
     from the shell's drop events. So the field is a read-only `span.ds-input[data-kind=file]`
     (`role=textbox`, `aria-readonly`) holding the caller's `value`, and a Tool `IconButton`
     (folder, titled "Choose…", named "{label}: Choose…" so two file fields are told apart); a
     click on either calls `on_pick: EventHandler<()>`, and the host opens its own chooser (an
     XDG portal, a sheet) and passes the name back. Proof:
     `the_choose_button_and_the_name_both_ask_the_host_to_pick` (two clicks, two picks, and the
     answered name replaces the placeholder).
   - **Multiline.** What Blitz does with a `textarea`, read from blitz-dom at the pinned rev:
     it is a multiline text editor (`create_text_editor(.., true)`); its text comes from the
     `value` attribute, not its children (the same path as an input, so the controlled field
     works unchanged); its intrinsic height is `rows` x line height (2 rows when absent) and
     its width `cols` x 0.6 em or 300 px; Enter inserts a newline (a one-line field submits);
     accessibility role `MultilineTextInput`. So the field writes `rows` and `height:auto` and
     lets Blitz size it. `Grow::ToContent` raises `rows` to the value's hard line count; a soft
     wrap cannot be counted before layout, so a long wrapped line scrolls inside the field instead
     of growing it. Proof: `a_growing_field_gains_a_row_per_line_and_a_fixed_one_does_not`
     (typing four lines into two rows: `rows="4"` and 2 x 13.5 x 1.55 px taller; the fixed one
     keeps `rows="2"` and its height), the `multiline-*` goldens, and the `Grow::rows` table.
   - **Bare.** `InputVariant::Bare`, with `FieldFace` as a type alias for `InputVariant`, not a
     second `face` prop beside `variant`: two props would let `variant: Boxed, face: Bare`
     disagree. It inherits font, size, weight, tracking, line height and colour, has no box, and
     styles only the caret (`--accent`) and the selection (`--accent-soft`); its placeholder is
     the parent's colour at .45. Proof: `a_bare_field_is_one_line_of_its_parents_text` (in a
     24 px / 1.25 title the field is exactly 30 px tall; a boxed one beside it keeps its own
     13.5 x 1.55 + 16).
     **Blitz limit, seen in a render:** blitz-dom's text editor takes only the font size, line
     height and colour from the field's computed style (`create_text_editor` inserts exactly
     those three into the parley styles), so on Blitz a bare field's value is drawn in the
     default family at the default weight, not the title's display face at 700; the placeholder,
     a plain span, does take the face. Size, line and colour match. This is blitz-dom's, not
     fixable in quire's CSS; a webview inherits the whole face.
   - **Seen in a render, fixed:** the file name, a span with `flex:1 1 0`, collapsed to its
     padding on Blitz (an input has a 300 px intrinsic width, a span none); the wrap now takes
     its container's width. The frame button stretched in a column was centred by Blitz's
     user-agent `button{justify-content:center}`; it now starts at its left edge, as a sidebar
     item does.
4. **SpaceEditor.** `motion_levels: MotionLevels::{All, Contact}` as a prop of the editor, not
   a field of `MotionChoice`: `MotionChoice` is a struct literal in mailo, and a new field would
   have broken it. `Contact` offers Calm, Standard and Extra; a `level` outside them (System,
   Reduced) presses no segment. Proof: the `motion-contact` golden and
   `the_contact_row_offers_three_levels_and_reports_a_pick`.
5. **CONSUMING's `snippet` row** said `Some(string)` no longer infers but not what does. It now
   lists every accepted form (a `&str`, a formatted literal, a `String`, a `Text`, an
   `Option<Text>`/`Some(text)`/`None`) and the rejected ones (`Some(string)`, `Some("literal")`,
   a `&String`, an `Option<String>`), each checked: `snippet_forms.rs` compiles and renders every
   accepted form, and each rejected one was confirmed not to compile.

Not done: no gallery specimen for `motion_levels` (the brief named none; the Space page's editor
keeps the default). The frame button's hover and pressed grounds are not pixel-tested, only
golden-tested, as the sidebar item's same rules already are.

What mailo changes (docs/mailo-migration.md section 2 has each row):

- A local account's tile passes `Provider::Local`.
- Frame words become `Button { variant: ButtonVariant::Frame }`; `… ▾` triggers
  `Button { trailing: Trailing::Caret, expanded }`; the bubble's faces `FaceMark` (or
  `Button { face }`).
- `Field { kind: Secret }` becomes `TextInput { kind: TextInputKind::Secret }`, a file field
  `TextInputKind::File` with `on_pick`, a `textarea` `TextInputKind::Multiline`, and an input
  styled as its row `FieldFace::Bare`.
- The Space editor's Motion row passes `motion_levels: MotionLevels::Contact`.


## mailo gaps 4 (overlays and lists) (2026-09-25)

mailo (on v0.1.5, 31 raw controls left) reported four more places where quire's overlays and
lists kept pieces its own. Branch `mailo-gaps-4a`, one commit per item. Every change is
additive: a new prop defaulting to the old behaviour, a new type or a new function. No existing
golden changed (the stylesheet golden gained two rules). CONSUMING.md "The mailo gaps 4 (2026-09-25): overlays and lists" has
one row per change, docs/mailo-migration.md section 2 the row for each mailo site. Proofs:
`crates/ds-native/tests/mailo4_{strip,hover,menu,sidebar}.rs` (Harness), the goldens
`lists/hover_strip/on-press`, `lists/sidebar_item/{place-named,place-drop-target,pinned-named}`,
`overlays/hover_card/hook-keyed-{inline,unplaced,rect}` and
`overlays/menu/{stay-checklist,inline-rich,inline-checklist}` (`crates/ds/tests/lists/mailo4.rs`,
`overlays/mailo4.rs`), and the gallery's Lists and Overlays pages.

1. **A strip press waited on a measurement.** A strip button's `onclick: EventHandler<Rect>`
   only ran once `client_rect` resolved, so with no layout (a server render, a host whose
   measurer answers nothing) a press did nothing, and an archive always waited a frame for a
   read it does not need. `HoverStrip { on_press: Option<EventHandler<ActionId>> }` fires
   synchronously inside the click, before the read; the measured `onclick` still follows when a
   rect arrives. A new strip prop was chosen over changing `onclick`'s type (a
   `Pressed { action, at: Placement }` payload) and over a new `StripAction` field: either would
   break every `StripAction { .. }` literal (quire's gallery, its tests and mailo's rows), while
   a defaulted prop breaks nothing, and the strip already carries its per-button state as props
   (`expanded`, mailo gaps 2) for the same reason. Proof (`mailo4_strip.rs`): under a measurer
   that answers `Unknown` for every element the click logs `press:archive` and nothing else;
   with the harness's own measurer it logs `press:archive,rect`, press first.
2. **Hover cards could only be keyed by quire's own target.** The hub was public but the anchor
   book was not, so a caller feeding `HoverEvent::Over` from its own hooks got a card at the
   overlay's corner with no way to place it, and mailo kept its own timer.
   `use_hover_intent() -> HoverDriver` hands out the hub with the anchor book:
   `over(key, kind, HoverAnchor)` files the anchor and feeds the machine (suppressed while a
   peek, the palette or a menu is open, as a target is), `out()`, `press()`, `hub()`.
   `HoverAnchor::{Rect(Rect), Element(MountedRef), Unplaced}`: `Unplaced` drops any stale rect
   for the key, so the card opens at the overlay's top-left corner, which is where an
   unanchored card already opened (the existing `tip-open` golden is unchanged). `HoverTarget`
   now goes through the same driver, so there is one path into the machine. The anchor enum is
   `HoverAnchor`, not `Anchor`: `ds::Anchor` (`Point`, `Rect`, `Mounted`) is the popover's and
   has no unplaced case. `HoverCard { flow: Flow::Inline }` draws the card in the caller's
   container (`position:static`, no `left`/`top`, `data-flow="inline"`) with the same markup,
   entrance and card hooks, for a test with no layout or a page that shows a card as part of
   itself. Proofs (`mailo4_hover.rs`): hook-keyed `li`s with a measured element anchor open a
   side card 10 right of the item and 6 above it after the wait (none at 400 ms) and switch
   within 60 ms while warm; a rect anchor from the pointer event opens a sender card at its x,
   6 below it; under a measurer that answers nothing, with `Unplaced` and `Flow::Inline`, the
   card is absent at 400 ms, present in its slot at 500 ms with no `style`, not yet leaving 100
   ms after the pointer left, leaving at 200 ms, gone at 450 ms, reopened at once within the
   warm window, and removed at once by a press. The SSR goldens assert the same card with no
   renderer at all.
3. **Every pick closed a menu, and every menu floated.**
   - `Menu { dismiss: PickDismiss::{Close, Stay} }` (default `Close`): under `Stay` a pick calls
     `onpick` only; the caller flips the row's `check` and the menu re-renders with the cursor
     where it was. design/06 section 5 already names the behaviour (C's properties and label
     picker "toggle and stay open"). Blitz ends a click on an element with no default action by
     clearing the focus (`blitz-dom` `handle_click`), which left a Stay menu deaf to Up, Down
     and Enter after a pointer pick; a Stay menu that floats and holds the keyboard
     (`Cursor::Auto`) therefore takes the focus back a frame after a pick
     (`Tracker::refocus`, the path a closed submenu already used). Preventing the click's
     default on every row would also do it, but would change focus for every closing menu too,
     so it was not taken. Proof (`mailo4_menu.rs`): a click on Travel logs `pick:1`, the menu
     is still open, Travel is checked and highlighted and the menu has the focus; a second
     click unchecks it; Enter checks it again; Escape then closes (`close` after the fade); an
     outside press closes a Stay menu; a `Close` pick still logs `pick:1,close`.
   - `Menu { flow: Flow::Inline }`: the rows drawn where the caller renders the menu,
     `div.ds-menu[data-flow=inline]` without `.ds-popover` (no surface, no entrance, no
     padding, width from the card), no overlay host and no outside-press catcher, never on the
     layer stack (`Stacking::Passive`, so Escape and an outside press are the caller's), no
     focus taken, and no press-drag-release. `Flow` is one type shared with `HoverCard`, since
     it is the same fact about either. The flow is read when the menu mounts (its stack
     membership is decided then); switching needs a new key. `MenuKind::Inline` was not used:
     the kind is the row shape (Rich, Slim, Dropdown, Context), and an inline menu of any shape
     is wanted (a Dropdown checklist inline). Proof: the inline menu stands inside its card, no
     overlay menu and no catcher exist, the focus stays put, Escape closes nothing, and a click
     picks and closes as usual.
   - `menu.rs` passed 300 lines: the pick and gesture states moved to `menu_pick.rs`, the
     cursor reports to `menu_active.rs`, the surface choice to `menu_surface.rs`, unchanged in
     behaviour.
4. **A sidebar place could not be a drop target.** `SidebarItem` had `drop: DropState`
   (`Target` is the `--accent-soft`, 1.045 highlight of design/06 section 6.1) but no pointer
   hooks and no name for the place under the pointer. It gains `place: Option<PlaceId>`
   (`data-place`) and `onpointerenter`, `onpointerleave`, `onpointermove`, `onpointerup`
   (`Option<EventHandler<PointerEvent>>`, the event itself as `ListRow`'s hooks are), on every
   kind. The listeners are always attached and call nothing without a handler, so a server
   render's markup is unchanged. `DropState` is not new: the brief's `DropState::{Idle, Target}`
   already existed (with `Source` for the dragged row). Proof (`mailo4_sidebar.rs`): three
   places carry their `data-place`; the pointer over Archive lights it `data-drop=target` (and
   no other), moving on to Invoices and releasing logs
   `enter:archive,leave:archive,enter:label:7,drop:label:7`, and the moves are heard.

Not done: nothing the brief asked for was left out. The strip's measured `onclick` still does
nothing without layout, by design: a snooze or label menu needs the rect to anchor, and
`on_press` is how the caller acts without it.

## mailo gaps 5 (2026-09-25)

mailo, on v0.1.6 with 5 raw controls left, reported seven small gaps. Branch `mailo-gaps-5`, one
commit per item. Every change is additive; no existing golden changed (the stylesheet golden
grew by the new rules). CONSUMING.md "The mailo gaps 5 (2026-09-25)" has one row per change,
docs/mailo-migration.md section 2 the row for each mailo site. Proofs: the Harness tests
`crates/ds-native/tests/mailo5_{propagation,scrim,menu_field,drop_accepts}.rs`, the goldens in
`crates/ds/tests/mailo_gaps5_ssr.rs` (controls and lists) and `overlays/mailo5.rs`, and the
gallery's Controls and Overlays pages.

1. **A press that must not bubble.** `Button` and `IconButton` take `propagation:
   Propagation::{Bubble, Stop}` (default `Bubble`), not a bool. The caller only receives a
   `Press`, so it could not stop the event itself. Under `Stop` each of the three press
   listeners (click, contextmenu, middle mouseup) stops propagation and prevents the default
   before reporting. **Found: stopping propagation is not enough on Blitz.** blitz-dom's click
   default action (`events/pointer.rs`, `handle_click`) walks up from the target to the first
   element with an action, and a `summary` inside a `details` is one, so a click on a button in
   the summary toggles the details however the event propagates. Preventing the click's default
   skips that walk; a `type=button` has no default action of its own, so nothing is lost. With
   only `stop_propagation` the Harness test fails on the first assertion (the details opens).
   `Press` gained nothing: it already carries the button, the modifiers and the client point,
   and the event itself is renderer-bound. Proof (`mailo5_propagation.rs`): a Stop `Button` and
   a Stop `IconButton` inside summaries log their presses and leave their details closed, with
   the summary's own handler not called; a Bubble button logs its press, then the summary's, and
   its details opens; a click on the summary's text still toggles a section whose button stops.
2. **A scrim under a peeked reader.** `Scrim { flow: Flow::Inline }` reuses the shared `Flow`.
   It is drawn where the caller renders it, `button.ds-scrim[data-flow=inline]`, with the
   scrim's own `position:absolute; inset:0`, so it covers the nearest positioned ancestor. It
   joins no overlay and no layer (`Stacking::Passive`, as the inline menu), sets no z-index, and
   calls `onclose` on a press at once (there is no stack for it to be under). The layering rule,
   in the component's docs: it dims its container's earlier content and lies under its later
   content and every floating surface. The brief named the handler `on_press`; the scrim's
   existing `onclose` is kept, as renaming it would have broken every caller. Proof
   (`mailo5_scrim.rs`): the scrim is the pane's child, absent from the overlay, and its rect is
   the pane's; a press on the reader drawn after it reaches the reader and not the scrim; a
   press on the uncovered veil dismisses. Golden `overlays/scrim/inline`.
3. **A label of runs.** `Button`'s `label` is `#[props(into)] Text`, the wave-2 run model, drawn
   through the same run renderer as `ListRow`'s subject inside the label's span; a `String`,
   `&str` or formatted literal renders the same markup as before (every existing button golden
   is unchanged, and the whole workspace compiled without a call-site change). A label of runs
   is named by `Text::plain_text()` through `aria-label`, so the name is one string whatever
   spans the tones need; a plain label is still named by its own text. `FaceMark`'s `label`
   takes a `Text` too. Goldens `controls/button/label-runs{,-named}`; a table test of the
   spoken name.
4. **A visible filter line.** `Filter::Field { placeholder }` beside `Typing` and `None`, in a new
   `menu_filter.rs` (the enum moved there from `menu_lines.rs`, re-exported as before). Keys are
   identical to `Typing` (`Filter::types()`); the row is `div.ds-menu-filter[role=searchbox]`,
   drawn and not a real input: the menu holds the keyboard, and an input would take the focus
   from the cursor it drives. It shows the placeholder, or the typed text, with a drawn
   `--accent` caret, in the inline `TextInput`'s measures over a hairline. **`Filter` is no
   longer `Copy`** (the variant holds a `String`); the panel's key reading takes it by reference.
   The controls' CSS scan banned the substring `filter` (for the unpainted `filter` property),
   which the class `ds-menu-filter` tripped; it now bans a `filter`/`backdrop-filter`
   declaration only. Proof (`mailo5_menu_field.rs`): the placeholder shows in a row above the
   first choice; typing `r`, `e` shows `re` and leaves Receipts and Travel (the fuzzy ranker's
   subsequence match), Backspace and `c` leave only Receipts, the highlighted row is always a
   choice, and Enter picks it; the log reads `query:r,query:re,query:r,query:rc,pick:Receipts`.
   Goldens `overlays/menu/filter-field{,-inline}`.
5. **Places that accept a drag.** `DropState::Accepts`, `data-drop="accepts"`. On a
   `SidebarItem` it is a dashed `--accent` hairline, with the hairline taken out of the padding so
   the item keeps its size and its label stays put. Not an `outline`: blitz-paint draws every
   outline style but `none` as solid (`render/border.rs`, `draw_outline`), while it dashes a
   border. It is weaker than `Target` (no fill, no scale, no shadow). A `ListRow` writes the same
   attribute and has no rule for it (rows are not drop targets). Proof (`mailo5_drop_accepts.rs`:
   same box and label position as an idle place) and golden
   `lists/sidebar_item/place-drop-accepts`.
6. **A leading mark.** `Button { leading: Option<Leading::{Glyph(Icon), Mark(Element)}> }`
   beside `trailing`, drawn first in `span.ds-button-lead`. The slot takes an `Element` rather
   than a `Provider` so any quire mark fits; keeping it to quire components is the caller's, as
   for any children. Goldens `controls/button/leading-{mark,glyph}`.
7. **A flag of runs.** A struct variant's field cannot take `impl Into<Text>`, and changing
   `Flag`'s `text` to `Text` would have broken every `Flag { text: String }` literal. So
   `HoverCardPart::FlagText { tone, icon, text: Text }` is added with the constructor
   `HoverCardPart::flag(tone, icon, impl Into<Text>)`, both tones (the "Info if symmetrical" of
   the brief is `FlagTone::Info`, which it covers). `Flag` is drawn through the same function as
   a plain `Text`, so its goldens are unchanged, and a plain `FlagText` renders byte for byte as
   `Flag`. Goldens `overlays/hover_card/part-flag-runs-{danger,info}`, `part-flag-text-plain`.

Not done: no gallery specimen for `Propagation`, `DropState::Accepts` or the flag of runs (the
brief named four; they have goldens and Harness tests). The accepts outline and the filter row
are golden- and geometry-tested, not pixel-tested.

What mailo changes (docs/mailo-migration.md section 2 has each row): its summary buttons pass
`propagation: Propagation::Stop` and drop their own `stop_propagation`; the reader pane's scrim
becomes `Scrim { flow: Flow::Inline }` before the reader; the quoted-message head a `Button` with
runs; its "Filter…" menus `Filter::Field`; the drag's outline `DropState::Accepts`; the From
value `leading: Leading::Mark(ProviderMark)`; the spoof flag `HoverCardPart::flag` with runs.

## mailo gaps 6 (2026-09-25)

mailo, on v0.1.8 with one raw control left, reported four gaps. Branch `mailo-gaps-6`, one commit
per item. Every prop is additive; the one markup change is `SidebarItem`'s class list
(`ds-sidebar-item ds-drop-place`), which its goldens show. CONSUMING.md "The mailo gaps 6
(2026-09-25)" has one row per change, docs/mailo-migration.md section 2 the row for each mailo
site. Proofs: the Harness tests `crates/ds-native/tests/mailo6_{scrim_layer,tree_item}.rs`, the
goldens in `crates/ds/tests/mailo_gaps6_ssr.rs` and `overlays/mailo6.rs`, the lint test
`crates/ds/tests/pass_through_lint.rs`, and the gallery's Controls and Lists pages.

1. **A "more" glyph.** `Icon::Ellipsis` and `Icon::EllipsisVertical`, Lucide `ellipsis` and
   `ellipsis-vertical` from lucide-static 1.47.0 (three unit circles each, centre first, as
   published), in `geometry_actions` and `Icon::ACTIONS`. They sit at the end of the enum and of
   each table under a comment, so the control set another branch adds to the same enum merges
   without touching these lines. A test locks the circles. The Controls page's glyph grid
   (from the control-set branch, merged) draws them with the actions, and a "Glyphs: more"
   section draws both at the row and header sizes and on Strip and Tool buttons. After the merge
   they follow the control set at the end of the enum, and `Icon::ALL` lists them with the
   actions, before the control set.
2. **The consumer's data and class on a quire button.** `Button` and `IconButton` take `data:
   Vec<DataAttr>` and `extra_class: Option<ExtraClass>`. `DataAttr { name: DataName, value:
   String }` renders `data-<name>`; `DataName::parse` takes lowercase ASCII letters, digits and
   `-` starting with a letter (HTML folds a `data-*` name to lowercase; Blitz does not, so an
   uppercase one would mean two things), and refuses a `ds-` name and the names quire writes or
   reads (`variant`, `size`, `theme`, `accent`, `motion`, `material`), so no consumer attribute
   can restyle a quire element. `ExtraClass::parse` takes one class or a space-separated list,
   each token letters, digits, `-` and `_` starting with a letter or `_`, and refuses any `ds-`
   token. **Found: dioxus attributes are named by `&'static str`.** An attribute named at run
   time has no static name, so each distinct `data-*` name is interned once (a `Mutex`ed set of
   leaked strings, one per name, for the program's life) when it is parsed; names are a
   consumer's small vocabulary (`folder`), and the value carries the data. The attributes are
   spread after the named ones (dioxus requires a spread last). Proof: goldens
   `controls/button/{data-attr,extra-class}`, `controls/icon_button/data-attr-extra-class`;
   `pass_through_lint.rs` renders a button with a consumer class and lints it clean against
   quire's sheet plus the consumer's rule, shows the same markup against quire's sheet alone
   names the class unstyled (so the clean result is not vacuous), shows `ds-` classes and
   reserved names refused with `PassThroughError::Reserved`, and that a consumer rule reaching
   for `.ds-button` is the stylesheet lint's `DsInternals`. `self_lint`'s control scan now reads
   a three-class consumer sheet for those goldens. On Blitz, `mailo6_tree_item.rs` reads
   `data-folder` off a trailing `IconButton`.
3. **A layer for the inline scrim.** `Scrim { layer: Option<ZLayer> }` (the token enum is
   `ZLayer`); an inline scrim with one writes `style="z-index:var(--z-…)"` on itself, from the
   token table. Without one it keeps z-index auto, and a positioned sibling drawn after it
   paints over it: that is CSS (positioned boxes at z-index auto paint in tree order), not a
   Blitz quirk. The component documents the rule: the caller picks a layer above its rows and
   below its floating surfaces, which then need a layer above it. A floating scrim ignores the
   prop (it is on `--z-scrim` already). Proof (`mailo6_scrim_layer.rs`): with no layer, a press
   at a positioned row's centre hits the row (`Harness::hits`) and a click logs the row; with
   `ZLayer::Raise` the same spot hits the scrim and the click dismisses. Golden
   `overlays/scrim/inline-layer-raise`.
4. **A tree row.** `TreeItem` is `details.ds-tree-item > summary.ds-tree-item-row` in the
   sidebar item's chrome, `open: Disclosure::{Open, Closed}` controlled through `on_toggle`,
   `TreeShape::{Branch, Leaf}` (a leaf is a row with no `details`; mailo draws its leaves as
   rows), `label: impl Into<Text>`, `glyph`, `count`, `here`, `onselect` (the label becomes a
   button that selects and keeps its press), `trailing: Option<Element>`, `drop`, `place` and
   the four pointer hooks, children in `div.ds-tree-item-children[role=group]` one `--s-12`
   step in (mailo's own indent). The summary's click prevents its default (the details' own
   toggle) and reports `open.flip()`; the app's state is the only state. The trailing slot
   stops and prevents every click that reaches it, so a ⋯ that forgot `Propagation::Stop` still
   does not toggle; its button hears the press first. The chevron is `chevron-right` at 12,
   turned a quarter by `aria-expanded=true` over `--t-quick`; the ⋯ shows on the row's hover,
   while its menu is open and under keyboard focus (`:focus-within` is false on Blitz, S12), and
   lends its height into the row's padding so a row with a 26 px Strip button is as tall as one
   without. **Found: a `details` that dioxus opens stays closed on Blitz.** dioxus-native writes
   every attribute in the HTML namespace (`qual_name` defaults to `ns!(html)`), and blitz-dom's
   user-agent rule `details:not([open]) > :not(summary:first-of-type) { display:none
   !important }` matches only a null-namespace `open` (the one its own summary toggle writes),
   so the children of `details { open: true }` never show, and an author rule cannot override a
   user-agent `!important`. dioxus-native also writes `open: false` as the text "false", which
   is an open attribute all the same. `TreeItem` writes `open` itself as an `Attribute` with an
   empty namespace (the null one) and value "true", or `AttributeValue::None` (removed) when
   closed; "true" rather than empty because dioxus-ssr writes `open` only when truthy. mailo's
   own folder tree (`details { class: "fold", open: true }`) hides its subfolders on Blitz for
   this reason. **The shared drop rules:** the three drop rules moved from `sidebar_item.css` to
   `drop_place.css` as `.ds-drop-place[data-drop=target|accepts]`, `[data-drag=source]`, last in
   the component order (they tie on specificity with the items' hover and current rules), and
   both items carry the class; no rule is written twice. Proof (`mailo6_tree_item.rs`): a Stop ⋯
   and a Bubble ⋯ each log their press and toggle nothing; the selectable label logs a select
   and toggles nothing; the chevron and the plain label call `on_toggle` with the other state
   and the next row moves up or down by the child's height (a hidden node keeps its last layout
   box on Blitz, so the test reads the following row, not the child); `data-drop` is `accepts`
   and `target`; the Accepts row keeps an idle row's box and label position; and the Target row
   paints a different colour from an idle row at the same spot. Goldens
   `lists/tree_item/{open,closed}-{idle,accepts,target,source}` and `leaf-current`; the existing
   `SidebarItem` goldens gained `ds-drop-place` in their class list.

Not done: no keyboard toggle of its own on `TreeItem`. Blitz's summary activation arrives as the
summary's click, which the component handles; no Harness test presses Enter on a summary.

What mailo changes (docs/mailo-migration.md section 2 has each row): its `span` wrappers around
buttons for `data-folder` and the reveal class become `data`/`extra_class`; the `⋯` text becomes
`Icon::Ellipsis`; the inline scrim takes `layer: ZLayer::Raise` instead of mailo's layered box
around it; the folder tree's `details`, chevron, `.fold-kids` indent and `can-drop` /
`is-drop-target` rules become `TreeItem`.

## Native phase B (2026-09-25)

mailo's Phase B plan (moving its window onto `ds_native::launch`) named the host gaps below,
G1 and G3-G7 (G2, a rich-text editing surface, is out of this branch's scope). Branch
`native-phase-b`, one commit per gap. The blitz rev is unchanged (`e99fbdbd`). Proofs are
harness tests under `crates/ds-native/tests/native_*.rs`; the window path (`launch`, which no
test drives) was run on Wayland with the `window` example: it opens, paints the app and closes.

**Breaking:** `AppConfig` is a builder now, `AppConfig::new(title, width, height)`, not a struct
literal (its new fields are private). The in-repo callers (the gallery, the `window` example,
`examples/consumer`) and `docs/mailo-migration.md` are updated.

1. **G1, contexts.** `AppConfig::with_context(value)`/`with_contexts(RootContexts)` go to
   dioxus-native's context list, which `launch` used to pass as `Vec::new()`.
   `HarnessConfig::with_context` carries the same values into `Headless`, through
   `Harness::with_config`, `Harness::with_contexts` and `snapshot_with`. A value is `Clone +
   Send + Sync`: dioxus-native wants `Send + Sync` factories (it builds the tree on the event
   loop's thread), and each document gets its own clone. A later value of a type shadows an
   earlier one. Proof: `native_contexts.rs`.
2. **G3, network policy.** `NetPolicy::{Local (default, what launch did), Custom(Arc<dyn
   AppNet>), Sealed}` on `AppConfig::with_net`/`HarnessConfig::with_net`. Where a request goes
   is one pure table (`crates/ds-native/src/route.rs`, table-tested) of the asking document, the
   scheme and the policy. `data:` is always served; `about:` never fetches. The app's own
   document gets `file:` under `Local` and `Custom`. A frame never gets `file:` from ds-native:
   under `Local` and `Sealed` a frame gets `data:` only. Under `Custom` every other request, a
   frame's `file:` included, is put to `AppNet::decide(&NetRequest) -> NetDecision::{Allow,
   Deny}`. A `NetRequest` carries `origin() -> RequestOrigin::{Top, Frame(FrameId)}`. An
   admitted request goes to `AppNet::fetch(request, NetReply)`, which may answer from any thread.
   - **How a frame is told apart.** Blitz builds a frame's document through the parent's HTML
     parser, with a config that inherits the parent's net provider (`blitz-dom/src/iframe.rs`).
     ds-native wraps the parser (`frames.rs`), so every frame document is born with the frame
     provider. That covers a `srcdoc`, a `src` load, and a frame nested in a frame.
   - **Found: the window's providers used to arrive too late.** The app's first render used to
     run before `Host`'s hidden element mounted and installed the providers. A frame in that
     first render would have been parsed with dioxus-native's providers. The app now renders one
     frame after the host, once `install.rs` has run.
   - **Found: an `<iframe src>` in the app's own markup is fetched by the app's document.**
     blitz's `start_iframe_load` uses the parent's id and provider. So an app-authored `src=file:`
     frame would load. This is the app's markup, not a sender's; mail uses `srcdoc`, and a frame
     nested inside a mail body is sealed.
   - Proof: `native_net.rs`. A frame's `file:` image is denied while the app's succeeds. A custom
     handler sees the app document's request as `Top` and the frame's as `Frame(id)`, with the
     id `Harness::frame` reports. `Allow` lands the image; `Deny` never reaches the frame.
3. **G4, the HTML parser; html5ever.** ds-native enables dioxus-native's `html` feature for the
   window and installs `blitz_html::HtmlProvider` (behind the frame wrapper) in `Headless`.
   `blitz-html` joins the pinned block at the same rev. That is a new line, so shell-host and
   sill copy it; neither uses it.
   - **Versions.** At this rev, blitz-html pulls **html5ever 0.39.0**, **markup5ever 0.39.0**
     and **xml5ever 0.39.0** (blitz's workspace pins all three to match stylo's `web_atoms`).
     It does not pull `markup5ever_rcdom`.
   - **quire's `deny.toml` needs no change.** It has no `[bans]` section, only licences, and
     `cargo deny check licenses` passes.
   - **mailo's `deny.toml` still holds.** Its ban is on `markup5ever_rcdom`, which is not pulled.
     But its reasoning ("a second parser generation reading ammonia's 0.40 output is where
     mutation-XSS lives") now applies to blitz-html's html5ever 0.39 itself. That needs a
     narrowly scoped note in mailo's `deny.toml` and the `mail-mime` owner's sign-off, as mailo's
     plan §4 says.
   - **The security boundary stays ammonia**, upstream in `mail-mime`. Blitz only parses HTML
     that is already sanitised, and it executes no scripts: there is no JS engine in ds-native's
     graph (no `boa`, `v8`, `rquickjs`, `mozjs` or `blitz-vibey-script`).
   - **The worst case of a parser differential** is markup resurrected inside the frame's own
     document. There, G3 seals the network and G7 freezes navigation.
   - **Harness access.** `Harness::frame(selector) -> Option<FrameView>` reads the frame's
     document: `id`, `text`, `html`, `count`, `text_of`, `attr`, `width`, and `centre` (in the
     app document's coordinates, for `click`).
   - Proof: `native_frames.rs`. The frame's text renders and paints in its own colour, neither
     document's queries see the other's nodes, and a `<script>` rewriting the body does nothing.
4. **G5, clipboard.** dioxus-native's `clipboard` feature is on (blitz-shell over `arboard`
   3.6.1), so Ctrl+C/X/V work in every text field of the window.
   - **The app's API.** `ds_native::clipboard::{write_text, read_text}` reach the document's
     shell through a `HostClipboard` context. They answer `ClipboardError::NoHost` (outside a
     ds-native document) or `Unavailable`.
   - **Found: blitz-shell panics on a session with no clipboard.** It unwraps
     `arboard::Clipboard::new()`. The app's calls catch that as `Unavailable`. A field's own
     Ctrl+C goes through blitz and would still panic there; that is upstream.
   - **The harness keeps its clipboard in memory:** `Harness::clipboard_text`,
     `set_clipboard_text`, and `selected_text(selector)` to read a field's selection.
   - **Not done: other Blitz hosts.** shell-host's surfaces and sill do not use `launch`, so they
     provide no `HostClipboard`, and the app calls answer `NoHost` there. A `provide` for them
     would take their shell provider.
   - Proof: `native_clipboard.rs`. Ctrl+A and Ctrl+C in one field, then Ctrl+V into a second;
     the app's write and read reach the same clipboard.
5. **G6, focus.** `ds::focus_soon` is public, beside `focus_soon_selecting(element, Select)`.
   - **Select-all.** `ds::Select::{None, All}`. `FocusRequest::with_select_all()` makes
     `TextInput { focus: Focus::Controlled(request) }` select the whole value each time the
     request lands, including on mount. This touched TextInput's focus wiring
     (`text_input_focus.rs`, two call sites) after mailo-gaps-4b merged.
   - **The host seam.** The select is a host write, `ds::HostSelect`: `ds_native::focus::SELECT`,
     provided by `launch`, `Headless` and `focus::provide()`. It runs after the focus write
     lands.
   - **Found: a field focused on mount has no editor yet.** Blitz builds a field's editor with
     the document's first layout, so the seam answers `Busy` until then and retries a frame
     later.
   - **Without a host** (a webview), `Select::All` does nothing.
   - Proof: `native_select.rs`. Select-all holds on mount and on a later request, and typing
     replaces the value. There is no selection without select-all. `focus_soon` focuses an
     app's own element.
6. **G7, frame links.** Blitz's `IframeNavigationProvider` reloaded the frame with a clicked
   link's target, which would fetch it through the app's document. Every frame document now gets
   ds-native's navigation provider instead, through the same parser wrapper.
   - **The API.** `FrameLinks::{Inert (default), Intercept(handler)}` on
     `AppConfig::with_frame_links`/`HarnessConfig::with_frame_links`. With `Intercept`, the app
     gets a `FrameLink { frame: FrameId, href }` with the resolved URL.
   - **Delivery.** A click goes through a channel, so the handler runs with the document free:
     the window's `Host` serves it from a task, and `Headless` drains it every frame. The frame
     never navigates. Links in the app's own document are unchanged: dioxus-native opens
     http(s) and mailto links in the browser.
   - Proof: `native_frame_links.rs`. An intercepted click reaches the app with the frame's id,
     and an inert one does nothing. In both cases the frame keeps its document and no request is
     made. With Blitz's provider in place both tests fail.
7. **`app_id`, done.** `AppConfig::with_app_id(AppId)` goes to winit as Wayland or X11 platform
   attributes. The backend is chosen as winit's own event loop chooses it: Wayland when
   `WAYLAND_DISPLAY` or `WAYLAND_SOCKET` is set. Off Linux it is ignored.

The notes mailo listed, and what would close each:

- **No tooltips for `title=` on the launch path.** Blitz draws no native tooltip.
  - Closing it: a ds-native host that shows a quire `Tooltip` for a hovered element's `title`
    after the hover delay. That is new overlay plumbing in the host, not a flag.
  - Meanwhile, `IconButton { tooltip }` and `HoverTarget` show one where it matters.
- **No scrollbars.** blitz-paint's `scrollbars` feature is off in the pinned block. Wheel
  scrolling works; no bar is drawn.
  - Closing it: turn on dioxus-native's `scrollbars` (and blitz-paint's for `Headless`) in
    ds-native. That is one feature line, but the bar blitz draws is unstyled and quire has no
    scrollbar token, so it wants a design decision first. Not done.
- **No `app_id`.** Done (item 7).
- **OS file drops.** blitz-shell ignores `WindowEvent::DragDropped`. That is upstream; mailo has
  no drop site.

## Edit surface (2026-09-25)

mailo's composer keeps its own renderer-free editor core (`editor::InputEvent { input_type, data,
ranges, composing, html }`) and draws its own caret and selection; on Blitz it needs a surface
that delivers input and reports geometry (mailo Phase B plan §3, option A). Branch
`edit-surface`, blitz rev unchanged (`e99fbdbd`).

### Spike: how IME reaches a component, and what layout exposes (read at the pinned rev)

- **The drop.** blitz-shell converts winit `Ime` to `BlitzImeEvent` (`convert_events.rs:31-46`)
  and hands it to the document as `UiEvent::Ime`; blitz-dom's driver targets the focused node
  and its default action edits only a focused `input`/`textarea` (`events/ime.rs`);
  dioxus-native-dom maps `DomEventData::Ime(_)` to `None` (`dioxus_document.rs:332`), so no
  Dioxus handler ever sees it, and its composition converter is `unimplemented!()`.
- **(b) works: ds-native sees the event first, no fork.** dioxus-native's application runs
  every `use_window_event` handler *before* it passes the winit event to blitz-shell's view
  (`dioxus_application.rs:197-199`), inside the registering scope's runtime
  (`hooks.rs:23`). ds-native's `Host` already listens there (modality, scale); it now also
  takes `WindowEvent::Ime`, reads the document's focused node through its `NodeHandle`, and
  routes the event to the edit surface registered at that node or an ancestor. blitz-dom's
  own IME handling then no-ops (the surface is not a text field). The harness owns its
  document, so its IME driver calls the same router directly. Route (a), a dioxus-native-dom
  patch on a fork, is not needed; route (c), a blitz `Widget`, would paint and hit-test
  outside the DOM and lose mailo's own markup, so it was not pursued.
- **IME has to be switched on by the surface.** blitz-dom enables IME (`ShellProvider::
  set_ime_enabled`, `set_ime_cursor_area`) only when a text field takes the focus
  (`node.rs:675-712`); winit delivers no `Ime` events otherwise. The surface's host seam does
  the same for a focused surface through the document's shell provider, and disables it on
  blur.
- **Hit-testing is reachable.** `BaseDocument::find_text_position(x, y)` (public) answers the
  inline root under a point and a byte offset into that root's laid-out text
  (`inline_layout_data.text`, the parley layout's string), using the same hit test and
  transforms as the pointer. `Node::inline_root_ancestor`, `Node::absolute_position`,
  `final_layout` and `element_data().inline_layout_data` are public, and parley 0.11's
  `Cursor::from_byte_index(..).geometry(..)` and `Selection::geometry(..)` give caret and
  selection boxes in the layout's (device-scaled) coordinates. So caret and selection rects
  for any element's inline content are computable in ds-native with no blitz change.
- **What the layout does not give: the text node.** A glyph run's brush is the *element* whose
  style span it is in (`TextBrush { id }`), not the text node, and the layout text is the
  concatenation of every text node after white-space collapsing (and case transforms, list
  markers, `br` as `\n`). ds-native aligns the DOM text nodes of an inline root to its layout
  text character by character (collapsed whitespace and generated text are skipped), so a
  layout offset maps to (text node, byte) and back.
- **Clipboard HTML.** `ShellProvider` has text only. The window reads `text/html` through
  arboard directly (`arboard::Clipboard::get().html()`, 3.6.1, already in the graph through
  blitz-shell's `clipboard` feature); the harness keeps an HTML slot in its memory clipboard.
- **Focus.** A click's default action on a non-field element clears the focus
  (`events/pointer.rs:816`) and a press starts Blitz's own document text selection
  (`pointer.rs:499`); a host focus write dispatches no `focus` event. The surface prevents the
  press's and the click's default actions, focuses itself through `HostFocus`, and treats the
  write's success as its focus-in; Tab still dispatches real focus events.

**Chosen route: (b).** No fork, no `[patch]`, no pinned-block change for blitz.

### Built

- **`ds::EditSurface`** (`components/edit_surface.rs`, state in `edit_surface_state.rs`): a
  `div.ds-edit` (`role=textbox`, `aria-multiline`, `tabindex=0`, `white-space: pre-wrap`, no
  focus ring) around the app's children. Its vocabulary is `ds::edit`: `EditInput`,
  `Composition`, `Pasted`, `KeyInput`, `EditPointer`, `TextPosition`/`TextRange` over
  `data-edit-node` keys, `EditKind::{Text, Atom}`, `EditHandle`, and the seam `HostEdit` (fn
  pointers like `HostFocus`, answering `Probe::{Found, Busy, Unknown}`). The IME's steps go
  through a pure machine (`edit/composition.rs`), keys through a pure table (`edit/keys.rs`),
  multi-clicks through `edit/clicks.rs`; each is table-tested.
- **The surface keeps two Blitz defaults from running.** A press would start Blitz's own document
  text selection (painted) and the click after it would clear the focus, so the surface prevents
  both and focuses itself through `HostFocus`, treating the write's success as its focus-in
  (a host write dispatches no `focus` event). It keeps its own click count for the same reason.
- **`ds_native::edit`** (`edit.rs`, `edit_tree.rs`, `edit_align.rs`, `edit_locate.rs`,
  `edit_geometry.rs`, `edit_hit.rs`, `edit_ime.rs`): resolves a `TextPosition` to an inline
  root and a layout byte through the DOM-to-layout alignment, and back; caret boxes from
  `parley::Cursor::geometry`, selection boxes from `parley::Selection::geometry` per inline root
  in reading order plus every whole atom between the ends; hit-testing through `doc.hit` (an
  atom answers its nearer side), then `Cursor::from_point` in the vertically nearest stretch of
  text, so a point in padding, between paragraphs or past a line's end still lands. Geometry is
  in the coordinates `get_client_bounding_rect` (and so `HostMeasure`) uses: the unrounded
  layout to a 64th of a pixel, less the viewport scroll, so an app subtracting
  `EditHandle::bounds` gets exact offsets.
- **IME routing**: `EditListeners` (root context) maps a surface's node to its sink; the window's
  `Host` and the harness hand an event to the listener registered at the focused node or its
  nearest registered ancestor. The surface turns the IME on (`set_ime_enabled`) and places its
  candidate window (`set_ime_cursor_area`, from `ime_area`) as it takes the keyboard, and turns
  it off at blur. The harness's memory shell records both.
- **Clipboard HTML**: `ds_native::clipboard::read_html()`; `arboard` joins the pinned block
  (`version = "3.6", default-features = false`, blitz-shell's own line), a new line shell-host
  and sill copy; neither uses it. The lockfile gains only the edge.
- **Proofs**: `crates/ds-native/tests/native_edit.rs`: a click focuses the surface and switches
  the IME on; typed keys arrive as `Text`, `Text`, `Key(Enter)`, `Key(Ctrl+b)`; a composition
  (start, two updates, commit) arrives as `Start`, `Update(ㄓ)`, `Update(ㄓㄨ)`, `Update("")`,
  `End(注)`, and typing after it is text again; keys during a preedit are withheld; a paste
  carries `Html { html, text }`, a plain one `Text`, and Ctrl+C/X are `Copy`/`Cut`; hit-testing a
  paragraph with a hard break across two text nodes answers offsets 11 (second line's start), 10
  (past the first line's end), 22 (past the second's) and 14 (right of a caret rect), and either
  half of a chip answers its side; caret rects at a line's start, middle, end and the next line's
  start, at 100 and 150 percent; a selection across the break is two boxes (from the anchor's
  caret to the line's end, from the line's start to the focus's caret), the same either way
  round; a selection over a chip includes its box; the IME area follows the app's `ime_area`
  and is not set before the surface has the keyboard. SSR goldens under
  `controls/edit_surface/`. Gallery: the Edit page, with the page's own caret placed from the
  host's rect. The window path (`cargo run -p ds-native --example edit`) opens on Wayland and
  closes cleanly.

### Limits

- **The window's IME is proven by reading, not by a test.** dioxus-native's hook order is what
  makes route (b) work; the harness drives the same router, but no test drives winit. Try it by
  hand with fcitx5 or IBus: `cargo run -p ds-native --example edit` prints every input. If a
  blitz bump reorders `use_window_event` after the document, IME silently stops reaching the
  surface: re-check `dioxus_application.rs::window_event` in the toolchain-bump wave.
- **Other Blitz hosts** (shell-host's surfaces, sill) get the seam from
  `ds_native::edit::provide()`, but nothing forwards their IME events yet: a host that owns its
  winit or Wayland text-input would need a ds-native entry point to hand them over. Not built.
- **Offsets are UTF-8 bytes**, not graphemes; mailo converts. The alignment tolerates collapsed
  whitespace, `br`, list markers and case transforms, but generated text that begins with the
  same character as the text after it (`1. ` before `1 apple`) could shift the map by that
  character; the surface's `pre-wrap` avoids collapsing altogether.
- **A caret between two text nodes of one paragraph that an inline atom separates** resolves to
  the later node's start, after the atom; the atom's own positions (0/1) name either side
  exactly.
- **No pointer capture**: a drag hears moves only over the surface, and a release outside is
  noticed at the next move without the button. (Closed in "Edit surface 2": a press captures
  the pointer until the release.)
- **Tab moves the focus** (Blitz's default) and is also delivered as `Key(Tab)`; the surface
  cannot keep Tab for list indentation yet.
- **Focus lost mid-composition** ends the composition empty; a commit the IME sends after the
  blur is not delivered (the surface no longer has the keyboard). Surrounding-text requests
  (`Ime::DeleteSurrounding`) are ignored: the surface never offers surrounding text.
- **Transforms**: geometry ignores a CSS transform on the surface or its ancestors (Blitz's hit
  test applies them, the rects do not). Scroll offsets are applied.
- **CJK line breaking**: laying out CJK text prints `ICU4X data error: No segmentation model for
  complex script: Chinese/Japanese` (parley's line breaker at this rev); lines still wrap, but not
  at dictionary word boundaries.
- **Clipboard HTML on Wayland** goes through arboard's X11 backend (XWayland), as blitz-shell's
  text clipboard already does at this rev (`wayland-data-control` is off in both).

## Icon plates and the assets path (2026-09-25)

Two gaps sill reported after wiring `icons.style` (sill FINDINGS Q71 and Q72).

### Q72: a tinted plate

- **What was wrong.** `IconView { plate: Some(PlateFamily::Neutral) }` draws the plate in the
  stylesheet from the family's stops, so under Monochrome the third-party raster was re-coloured
  by `retint` and its plate was not: in the dark, a `#2A2E28` paper under a blue icon. sill could
  not fix it itself: styling `.ds-plate` trips `DsInternals`, an inline colour `HexColour`.
- **What quire does now.** `IconView { plate_tint: Option<PlateTint> }`, defaulted, so every
  `plate: Some(..)` call compiles unchanged. `PlateTint::of(style, tint)` is `None` for Colour,
  `Muted`, or `Monochrome(tint)`. The plate's two stops and its ink go through
  `retint::recolour`, the one function `retint` now calls per pixel, for the light and the dark
  scheme (`PlateStops::of(family, scheme).tinted(tint)`), and are written on the plate as
  `--plate-base-l`/`--plate-deep-l`/`--plate-ink-l` and the `-d` three, with
  `data-icon-style="muted|monochrome"`. Two sheet rules after the family rules, at their
  specificity, point `--plate-base`/`-deep`/`-ink` at the `-l` set, or the `-d` set under
  `.ds[data-theme=dark]`. The Space dot's `--dot-c*` pattern (mailo gaps 3, item 5): a custom
  property on a `ds-*` element is how quire hands its sheet a per-instance colour, so the
  gallery's markup lint stays clean with no exception; the six names join `--dot-c*` in the
  self-lint's `INLINE_VARS` and the token test's `PER_ELEMENT`.
- **Proof.** `ds-native/tests/plate_tint.rs` renders a Neutral plate in the Work and the Home
  Space's Monochrome tint and an untinted one, in both schemes: the tinted centre and deep corner
  read the tint's hue (dark within 8 degrees at chroma above .03, measured within 2; light within
  12 at chroma above .003, measured within 8, since the light paper sits at lightness .96-1
  where the tint eases off); the untinted plate's centre is the mean of `#2A2E28`/`#1D211B`
  (dark) and `#FFFFFF`/`#F1F3EE` (light) within 2 per channel. SSR goldens
  `polish/plate-neutral-monochrome-{light,dark}.html` and `polish/plate-blue-muted.html`; unit
  tests on the stops. Gallery: Controls, "Tinted plates", Colour / Work / Home / Muted in light
  and dark, each with a retinted raster and the glyph fallback.
- **Limits.** The light paper stays white at its top-left stop under any tint (retint's rule has
  no room at white, the same as a white icon); the tint shows at the deep stop and in the ink.
  A consumer that needs another lightness for tinted paper needs a design decision, not a flag.

### Q71: where the app icons are

- **What was wrong.** quire shipped `assets/icons/apps` and no way to find it; sill searched
  `$SILL_ICON_ASSETS`, the XDG data dirs and a sibling checkout on its own, so each consumer
  would have written its own order.
- **What quire does now.** `ds_settings::icon_assets` (re-exported as `ds_settings::apps_dir` and
  `ds_settings::app_icon_path`): `$QUIRE_ICON_ASSETS`, `$XDG_DATA_HOME/quire/icons/apps`, each
  `$XDG_DATA_DIRS/quire/icons/apps`, then the repository's `assets/icons/apps` from
  `CARGO_MANIFEST_DIR` (`AssetsOrigin::DevAssets`, development only). First existing directory
  wins, whole. `app_icon_path(app, Px, IconStyle)` picks `<app>/[muted|monochrome/]<px>.png`,
  the exact size from design/08 2.11's thirteen (`APP_ICON_PX`), else the nearest larger, else
  the largest smaller. It lives in `ds-settings`, not `ds`: it reads the environment and the
  disk, and `ds` stays effect-free (`check-boundary.sh`'s note); every step is a pure function
  of an `AssetsEnv` and a probe, table-tested. `icons install [--from] [--to]` copies the set to
  `$XDG_DATA_HOME/quire/icons/apps`; `tools/icons/tests/install.rs` runs the binary into a
  scratch directory and finds all 195 files (5 apps x 3 styles x 13 sizes) through the lookup,
  and checks the tool's `SHIP_PX` against `APP_ICON_PX`.
- **What sill changes.** Drop its own search and `$SILL_ICON_ASSETS` (use `QUIRE_ICON_ASSETS`),
  call `ds_settings::app_icon_path(app, Px(tile * scale), style)`; for Monochrome files keep
  calling `retint` after loading. For third-party icons on a plate, pass
  `plate_tint: PlateTint::of(style, tint)` (Q72) with the same pair it hands `retint`.
- **Limits.** `app_icon_path` probes the disk per call; cache its answer with the decoded icon.
  A partly installed set is not merged with a later step. `$QUIRE_ICON_ASSETS` names the apps
  directory itself, not a data directory.

## OSD parts (2026-09-25)

sill reported three gaps building its OSD (sill FINDINGS Q74, Q75, Q76). Branch `osd-parts`. Mid-way
the user moved the OSD to the top right under the bar (design/20 §1.7 as updated on master) and
asked for a richer level control than a read-only slider ("Level control" below). A plain
`SliderMode::Level` was committed and then reverted in favour of it; `Slider` is the form slider
again, unchanged.

1. **Q75: the OSD's own motion.** `Anim::OsdIn` (`osd-in`, `--t-quick --e-out`: from `--osd-dy` at a
   .96 scale and transparent to rest, `pop-in`'s entrance with no overshoot) and `Anim::OsdOut`
   (`osd-out`, `--t-move --e-exit`, forwards: to half of `--osd-dy` and transparent). One pair whose
   keyframes read `--osd-dy`, a signed offset the card declares per position (as `--dy` drives
   `heal`): -8 px at the top right, so the card drops in from above and lifts back out; 8 px at the
   bottom centre, so it rises and drops. The drift test expresses it unchanged (the recipe's name,
   duration and easing are what it checks). Recipes in `recipe_own.rs`, `X--b` aliases and pulse
   classes generated, settle rows in `motion_drift.rs` (204 and 284 ms at Standard), `Anim::ALL`
   59 with `LevelTick` and master's four pane anims; the keyframe count assertion in `motion_css.rs`
   is 50; design/05 §4.7.
2. **Q76: an `Osd` component.** One card, in one transparent Osd root: the root gives the fade its
   motion tokens and the card its `--f-*`, `--m-*`, stack and tint; the card paints what a tinted
   root paints (`.ds-frame` at the frame alpha, the solid floor without blur, the inner pair
   redrawn), so the nested painted root sill used is gone. Presence is a pure machine
   (`osd_phase.rs`, table-tested): Hidden, Entering, Present, Leaving; a show while leaving is
   Present at once and cancels the exit timer (`MotionTimer::cancel`, new), a late settle changes
   nothing. `on_hidden` runs from `settle(OsdOut)`'s timer. The phase steps in render (so the render
   draws it), the timers start in an effect after it. Proof: `ds-native/tests/osd_presence.rs`
   (leaving at once, `on_hidden` not at settle - 40 ms and once at settle + 40 ms, hidden after;
   shown during the fade: present, no `on_hidden` 500 ms later); goldens `level/osd-*`.
3. **Q74: the level bar.** `LevelControl` ("Level control" below), `mode: LevelMode::ReadOnly` in
   the card.

What sill switches to: `Ds { material: Osd, chrome: Transparent, .. }` holding `Osd { shown, label,
level: Level { value, glyph }, position, id: "osd", on_hidden }` (its view's nested painted root,
`osd.css`'s fade rules and the `DurationToken::Move + FRAME_SLACK` leave time go); its machine's
hold stays its own and drives `shown`; `on_hidden` unmaps the surface; `OsdMetrics { margin }`
carries `osd.margin_px`, with the layer margin 0 and the surface sized to the card plus its margins
and shadow room; `sill_settings::OsdPosition` maps onto `ds::OsdPosition`.

## Level control (2026-09-25)

The user found a plain read-only slider too simple and asked for macOS-grade looks, variants to
choose from. `LevelControl` (new; `Slider` stays for settings rows). References, described (no
asset copied): the current macOS volume and display modules and OSD, a thick capsule whose fill is
white on the vibrant material with the glyph inside at its left end; the Big Sur control center
slider, a capsule with a separate round white knob at the fill's end; the classic pre-Big Sur OSD,
sixteen small squares under the glyph, one per volume key step.

- **Looks** (`LevelLook`): `Capsule` (recommended), `CapsuleKnob`, `Segments`. Capsule 26 px, full
  radius. Inks are new material tokens per scheme (`material/level.rs`): `--m-level-fill` (a
  near-white .97 on light, white .94 on dark), `--m-level-well` (black .10 / white .14) with
  `--m-level-shade` (`inset 0 1px 2px`), the glyph's two inks, the knob's hairline and drop, the
  tick mark.
- **Two-tone glyph without blend modes.** The glyph is drawn twice at the same place: on the well
  in the well's ink, and inside the fill (which clips it) in the dark ink. Where the fill covers
  it, it reads knocked out.
- **Glyph follows the level** (`glyph.rs`): stacked `svg` parts that cross-fade by opacity over
  `--t-quick` (an SVG's own CSS does not animate on Blitz, so no dash draw): Lucide's speaker body,
  three waves (arcs on one centre at radii 5, 8.25, 11.5, spaced for a 2-unit stroke; Lucide has
  two) shown by thirds, Lucide `volume-off`'s slash when muted; Lucide's sun, its rays scaled by
  `.6 + .4 x level`.
- **Machine** (`machine.rs`, table-tested): press (holds at once), measured (the track is read
  after layout in a task; a click let go before it lands still sets the level; a move before it is
  kept), drag, release, rubber band `6 x d / (d + 12)` px past an end, off under Reduced, keys to
  the next point of the 16 or 64 grid.
- **Motion.** Set from outside: `width` over `--t-quick --e-out`; under the pointer
  (`data-drag=live`): no transition. Press: `scaleY(1.08)` over `--t-quick --e-spring` (contact).
  Release from a stretch: `left`/`right` back over `--t-move --e-spring`. Segments: each changed
  square after `i x --stagger`. `Tick::Quiet`: the fill edge's mark, `Anim::LevelTick` at `--t-tap`.
- **Proofs.** `ds-native/tests/level_control.rs`: a set from 20 to 80 % is at the `--e-out` curve's
  width a quarter and half of `--t-quick` in (131.2 vs 129.8, 153.9 vs 153.9) and painted between,
  then at 80 %; a press jumps and a move 16 ms later is exactly under the pointer; the press
  swells the painted fill 27 to 29 px; 12 px past the end stretches the track 3.0 px and it is back
  600 ms after release; Reduced does not stretch; keys 500 to 563, 438, Shift 453. Goldens
  `tests/snapshots/level/` per look, glyph state, mode and the OSD card; unit tables for the
  machine, the glyph's parts, the segments and the inks.
- **Sheets for the pick.** `ds-gallery --level-sheet DIR` writes `level-variants.png` (each look in
  light and dark over the Work tint and over a light ground (the Home Space), volume 0, 40, 100 %,
  muted, brightness 30 %, 1x then 2x) and `level-motion.png` (Harness frames through a set, a press,
  a drag past the end and the release); copies are in `tools/progress/shots/gallery/`. The gallery's
  new Level page has them live, and the Polish page an OSD specimen.
- **Found: one Blitz document with sixty tinted cards loses layers.** Drawn as one page, every
  card vanished (and with twenty, one row's cards painted at partial opacity), while each row alone
  is right; the sheet therefore renders each row on its own. Not chased further; a page that
  needs that many materials at once is not a real surface.
- **Limits.** Leaving the control's hit zone (the rail and 16 px beside it) while dragging lets go
  (Blitz has no pointer capture); the swell is 1 px each side at 26 px; the waves are cross-fades,
  not drawn strokes.
- **Recommendation: `Capsule`.** It is the current macOS form, needs no separate glyph column, and
  carries the most information in the least width (glyph, level and mute in one shape); `Segments`
  reads as dated and steps visibly; the knob adds a target that means nothing on a read-only OSD.

## Native focus (2026-09-25)

Three gaps mailo's window reported once it came up on `ds_native::launch` (mailo pins quire by
tag, v0.1.7/8, behind its `native` feature). Branch `native-focus`, one commit per item; blitz
rev unchanged (`e99fbdbd`). Proofs are harness tests: `native_focus_field.rs`,
`native_keep_focus.rs`, `native_strip.rs`.

1. **G8, a field by handle, any element by selector.**
   - **Found: there is no way to make a `MountedData` for a node found by selector.**
     dioxus-native-dom's `NodeHandle` has crate-private fields and no constructor, so mailo
     re-did ds-native's document writes itself (`query_selector`, `set_focus_to`,
     `with_text_input(select_all)`, retried per frame), and a focus written that way fired no
     `focus` event. ds-native now wraps a found node in its own `RenderedElementBacking`
     (`FoundNode`, `node_ref.rs`); the focus, blur and select writes take either handle. A
     found node answers no other mounted call (rects, scrolling).
   - **`TextInput { handle: Option<FieldHandle> }`**, from `use_field_handle()`, as
     `use_edit_handle` is: `focus(Select)`, `blur()`, `element()`. The focus task runs in the
     handle owner's scope. Blitz's blur is `clear_focus` when the field has the focus
     (`ds::HostBlur`, `ds_native::focus::BLUR`), again with no event, so the handle calls the
     field's `onblur` (and the commit it makes on blur) itself.
   - **`ds::focus_by_selector(selector, Select) -> Result<(), FocusError>`**, async, through
     `ds::HostFind { find, same }` (provided by `launch` over the host's document handle and by
     the harness over its document; not by `focus::provide()`, which has no document). It waits
     up to twenty frames for the element to be drawn (mailo's webview scripts waited twenty).
     `FocusError::{NoHost, BadSelector, NoSuchElement, Busy, Refused}`; an unknown selector is
     `NoSuchElement`, never a panic.
   - **The told-path for a selector.** Every mounted `TextInput` enters a root-context list
     (`focus/targets.rs`) with its element and its `onfocus`/`onblur`; a node found by selector
     is matched to an entry through `HostFind::same` (node identity) and its `onfocus` is called
     once the host's write lands, as `focus_soon_told` does for `Focus::OnMount`.
   - Unchanged: the element that *loses* the caret to a host focus still hears no blur on Blitz.
   - Proof: `native_focus_field.rs`. By handle and by selector the field is focused, its log is
     `focus` exactly once and `selected_text` is the whole value; the handle's blur logs `blur`
     once and leaves the field; `#nothing-here` is `NoSuchElement` after the wait, `[[` is
     `BadSelector` at once.
2. **Keep-focus.** Blitz's `handle_click` default walks up from the target and, when nothing on
   the way is one of its own (a text field, any `input`, a submit button, a `summary`, a
   `label`, a link, a `disabled` element), clears the focus. A `button type="button"` is none
   of these, so a click on a quire `Button` cleared it too. mailo's keys are handled on
   `.app[tabindex]`, so it re-focused `.app` a frame after every pointer release.
   - **Now:** `FocusFallback::{Ancestor (default), BlitzDefault}` on `AppConfig` and
     `HarnessConfig`. Under `Ancestor`, `Ds`'s root click handler (the last handler a click
     reaches) asks `ds::HostClickFocus` (`ds_native::CLICK_FOCUS`, `click_focus.rs`) what the
     click will do: when Blitz would clear the focus and the target or an ancestor is focusable
     (`is_focussable`: a `tabindex` >= 0 or a natively focusable element), that element takes
     the keyboard once the click is done, if the click left the focus nowhere.
   - **The default is not prevented.** The EditSurface prevents it, but Blitz's click default
     also dispatches `dblclick` and the blur of a field the click leaves, and both matter here.
     So when the ancestor already has the focus it is cleared first without events (Blitz's
     clear then has nothing to blur), and the ancestor is focused again by a task. A handler
     that moved the focus during the click wins, because the task only acts on a focus that is
     nowhere.
   - **The EditSurface's own focus wins inside it:** it prevents its click's default, and the
     root skips a click whose default is taken.
   - **Limits.** The refocus is a task: at once when the document is free, a frame later when the
     click's own re-render holds it. On the window a key would have to arrive between the click
     and the next poll to miss it. The ancestor hears no `focus` event (a host write), as with
     `HostFocus`. Another Blitz host (sill, shell-host) gets the fallback only by providing
     `CLICK_FOCUS` itself.
   - Proof: `native_keep_focus.rs`. A click on plain text in `div.app[tabindex=0]` leaves `.app`
     focused and its keydown hears `j`. With `BlitzDefault` the focus is cleared and the key is
     not heard. A second click logs no `app-blur`. A double click still logs `dblclick`. A field
     the click leaves logs `field-blur` and `.app` takes the keyboard. A click inside an
     `EditSurface` focuses the surface.
3. **The hover strip on Blitz, measured** (`native_strip.rs`, a 74 px row in a 560 px list, four
   buttons). Design/04 section 17 says: right 8, vertically centred, padding 3, buttons 26 with
   gap 3, a 1 px `--line` border. So in a 74 px row the strip is 34 tall and 20 from the row's
   top. Its right edge is 9 in from the row's border box (8 plus the row's hairline). It is
   `26n + 3(n - 1) + 8` wide: 121 for four buttons.
   - **Painted and hit box: already the design's.** Blitz applies `translateY(-50%)` when it
     paints and when it hit-tests (`Node::hit_inner` inverts the transform). A point 1 px inside
     each corner of that box hits the strip, 2 px above or below misses it.
   - **The hidden strip takes no hits.** Blitz honours `pointer-events:none`, and the buttons
     inherit it. So a click at the centre of a row whose strip is hidden reaches the row.
   - **Found: the layout rect was wrong.** `get_client_bounding_rect`, which `HostMeasure`
     reads, leaves transforms out. So the strip read from the row's vertical middle down
     (top 37, where the painted box's top is 20). That is the "right half from its vertical
     middle down" mailo reported. A strip button's `onclick: Rect`, which anchors the snooze and
     label menus, was 17 px low on Blitz.
   - **Fixed in CSS.** The strip is centred with `top:0; bottom:0; margin:auto 0;
     height:max-content`: the same box in a browser, with no transform to leave out. The layout
     rect now reads top 20, height 34, width 121, right inset 9.
     `mailo_lists.rs`/`mailo4_strip.rs` no longer lift their click by half the strip's height.
   - **A click at the row's centre** reaches the row with the strip hidden, shown
     (`Shown::Visible`) and hover-revealed. A shown strip legitimately covers the centre when it
     is wider than half the row less 9: five buttons (150 px) in a 300 px row put its left edge
     159 from the right. There a centre click presses the first button (`press:archive`), in the
     webview too. The same row with the strip hidden opens. So mailo's centre click landed on
     Archive because its strip was shown (hovered, or its row held the focus) and wide enough
     for its row, not because Blitz's box is wrong.
4. **A click a quire control keeps to itself (mailo, against v0.1.10; branch
   `stop-click-focus`).** mailo measured the focus on `html` right after a `HoverStrip` Archive
   click, against §6.6 of `docs/mailo-migration.md` ("a click on a quire strip button focuses
   that button").
   - **Found: two ways a quire click never reached the root's fallback.** A control that stops
     the click's propagation (the strip's buttons, a `TreeItem`'s select button and trailing
     slot, `Propagation::Stop`, the row star, a sidebar item's close and cancel, a menu row's
     action, a settings row's switch slot, the space editor's remove, an inert tile chevron, a
     rich-text link) keeps it from `Ds`'s click handler. Blitz's default still ran and cleared
     the focus (a `button type="button"` is none of Blitz's own), and with the pressed button
     still in the document the removal keeper took the clear for a deliberate one
     (`Seen::Released`) and left it: the keyboard was on `html`. And a control that prevents the
     default (every `Propagation::Stop`, the tree row's `summary`, which prevents its own toggle)
     made the root pass the click by (`after_click` skips a click whose default is taken), while
     Blitz, its default prevented, left the focus wherever it was: a click on a tree row from
     nowhere left it nowhere.
   - **Checked: no capture phase.** dioxus 0.7's events have no capture listeners, and Blitz's
     driver dispatches along the target's chain bottom-up only (`events/driver.rs`), so the root
     cannot hear a click before a control stops it.
   - **The rule now.** Every quire click handler that stops a click, or prevents its default,
     calls `focus::click::kept_click` last, after its own work (the root is the last to hear a
     click, and a handler's own later focus must still win). `Ds` provides its seams and its
     element as context (`ClickRoot`) for that. With the default left to run, the click goes
     through `HostClickFocus` exactly as the root's does (fallback now, restore a frame later
     if the focus is nowhere). With the default prevented, Blitz will not clear the focus, so
     the new `ds::HostPressFocus` (`ds_native::PRESS_FOCUS`, provided beside `CLICK_FOCUS`
     under `FocusFallback::Ancestor`) gives the nearest focusable element from the pressed one
     up the keyboard at once, inside the click. The pressed control, or its focusable ancestor
     (the `summary` for a tree row's plain label), has the keyboard afterwards, as a pressed
     button does in a browser.
   - **Limit: a field keeps the keyboard through a prevented kept click.** A host write
     dispatches no event, so moving the focus off a text field (or an editable surface,
     `role=textbox`) without Blitz's clear would leave it unblurred: a rename field would never
     hear that it lost the keyboard. `PRESS_FOCUS` leaves the focus there, as before this fix.
     A kept click whose default runs (a strip button) is Blitz's clear, with its `blur`, so there
     the button takes the keyboard from the field as any click does.
   - **Held in the source.** `crates/ds/tests/kept_click_rule.rs` reads every component's
     `onclick` handlers (a named one, `onclick: fence`, from its function) and fails on one that
     stops the click without `kept_click`. A handler that only prevents the default is not
     scanned (the `summary` is the one today).
   - **The editing slot's pointer-up.** `TreeItem`'s editing slot stopped `pointerup` as well as
     the click, so an ancestor's `onpointerup` (mailo's `.app`, which ends a drag there, and the
     row's own drop relay) never heard a press that ended in the rename field. Nothing depended
     on it (Blitz's pointer-up default, the click, is unaffected), so the fence is gone; the
     click and double click stay fenced.
   - Proof: `crates/ds-native/tests/kept_click_focus.rs`. After a click on a strip button, a
     tree row's select button, the ⋯ in its trailing slot and a plain tree label, the pressed
     control (the label's `summary`) is focused, not `html`, the press stayed the control's and
     `j` reaches `.app`; a kept click takes the keyboard from `.app`; under
     `FocusFallback::BlitzDefault` none of them gets it (the negative control); a rename field
     keeps the keyboard through a select click elsewhere; a press in the field reaches `.app`'s
     `onpointerup`. The four focus cases, the take from `.app` and the pointer-up failed on master;
     the negative control and the field's case pass on both.

## Control center parts (2026-09-25)

Four additive pieces sill asked for to build the control center with macOS Control Center's
polish (sill FINDINGS Q78-Q81). No existing golden changed; the stylesheet golden grew by the new
rules and two keyframes. CONSUMING "Control center parts (2026-09-25)" has the API.

### Q78: a module tile

- **What was wrong.** quire had no control-center tile: sill drew one from a `Button { Mini }`
  and a local disc, and the chevron that opens a module's detail was a second button beside it
  whose click also reached the tile, toggling Wi-Fi when the person only wanted its networks.
- **What quire does now.** `ModuleTile` in a `ModuleGrid` (two columns, gap 8): a
  `div[role=button]` at `--r-tile` with a 28 px disc, the title at the shell menu's 13/600 and a
  faint status line. `ModuleState::Off` is the Mini's plate with a paper disc in ink, `On` the
  plate in `--accent-soft` and the disc in `--accent` with `--accent-ink`, `Busy` the Off disc
  with the Spinner's breathe around it (`aria-pressed="mixed"`, `aria-busy`). The hover is the
  Mini's (`--raise`, `--shadow-1`) plus the list row's `--lift`; the press squishes. The tile is
  not a `button` because the chevron inside it is one. The chevron (`Chevron::Detail`) is 18 x 28,
  named "{title} details", `aria-expanded` from `expanded`, and keeps its press
  (`Propagation::Stop`, mailo gaps 5): its click never reaches the tile. Keys are the tile's own on
  both renderers: Enter or Space toggles, Enter or Right on the chevron opens the detail and stops
  there (Blitz synthesises no click from a key; the chevron prevents the key's default, so a
  browser's synthesised click cannot run the press twice). `TileSpan::Full` spans both columns.
  A half tile's title clips instead of fading: `.ds-truncate`'s fade covers the column's last
  1.5em whether the words reach it or not, and the column (about 72 of 144 px) is barely wider
  than "Bluetooth".
- **Proof.** `ds/tests/control_center_ssr.rs`: goldens `control_center/tile-{off,on,busy}-
  {half,full}-{light,dark}.html` in a Popover root, and the chevron absent, inert and open; every
  golden lints clean and every class is styled; the state, span, pressed and chevron attributes
  are asserted per case. `ds-native/tests/cc_module_tile.rs` on a Blitz document: a chevron press
  logs `detail` and the tile stays off; a press on the title toggles it; Tab reaches the tile,
  Enter and Space toggle, Tab reaches the chevron, Enter and Right open the detail and neither
  toggles.
- **What sill changes.** Its tile becomes `ModuleTile`, its grid `ModuleGrid`, inside the panel's
  own 12 px padding.

### Q79: a settings row outside a menu

- **What was wrong.** A network or device list in the control center was `ListRow` (mail's
  two-line card, far too heavy) or `MenuEntry::Row` (which needs a `Menu`: an overlay, a layer and
  the keyboard).
- **What quire does now.** `SettingsRow`: 44 px, a hairline (`--hair`, `--line`) above every row
  after the first, `MenuEntry::Row`'s type in a Slim menu (title `--fs-shell-menu` at 400, detail
  `--fs-help` faint), a 16 px glyph in a 22 px column, the Dropdown's `--surface-2` under the
  pointer. `RowTrailing` is the end: `Check(Switch)` in `--accent` (the row writes
  `aria-pressed`), `Toggle { value, on_toggle }`, `Chevron`, `Text`, `Glyph`, or nothing. The
  toggle sits in a span that stops its click and its keys, so flipping headphones on never runs
  the row. Enter or Space runs `onclick`.
- **Proof.** Goldens `control_center/row-*.html` (each mark, a bare row, a disabled row) and
  `rows-networks-{light,dark}.html`; `a_row_writes_its_trailing_mark`.
  `ds-native/tests/cc_settings_row.rs`: the switch logs `toggle:On` and not the row, the words log
  the row only, Enter on the focused row runs it, and a disabled row and its switch hear nothing.

### Q80: a pane and its detail

- **What was wrong.** design/13 13.3.7 plays the detail pane in with `slide-r`/`slide-l` at
  `--t-move --e-spring`, but the catalogue's `Anim::SlideR`/`SlideL` are `--t-big` (their Space
  switch rows), so no `Anim` matched and the drift test forbids an animation that is not one.
  There was no exit for the outgoing pane, and `use_entrance` (the one-shot entrance timer every
  overlay uses) was private to `popover.rs`.
- **What quire does now.** Four `Anim`s (56 in all): `PaneInR`/`PaneInL` play `slide-r`/`slide-l`
  at `--t-move --e-spring`; `PaneOutL`/`PaneOutR` play two new keyframes `pane-out-l`/`pane-out-r`
  (to `-26px`/`+26px` and transparent, the mirror of the slides) at `--t-move --e-exit`, forwards.
  `PaneSwitcher { shown, root, detail, on_settled }` runs a pure `PaneSlide` machine: asking for
  the other pane starts a round in which both panes are drawn, the arriving one `entering` and the
  outgoing one `leaving`, out of the flow at the top so the switcher is the arriving pane's height
  from the first frame; one `MotionTimer` settles the pair (all four recipes are `--t-move`, a unit
  test holds it), and `on_settled` hears the pane. Asking again mid-slide starts a new round: each
  pane takes the other animation from its start (the names differ, so Stylo restarts them), the
  timer restarts (the reversed round's task is cancelled) and a settle naming an old round changes
  nothing. The timer is started from an effect after the render, with its handler made in the
  component's scope (an `EventHandler::new` inside the effect has no scope and panicked in the
  first run of the Harness test). `use_entrance` is `ds::motion::use_entrance` and `ds::use_entrance`,
  unchanged; the menu, sheet, peek, bubble, hover card and link pill import it from there, and
  their goldens are as they were.
- **Proof.** `pane_slide` unit tests (a switch, a reversal and a stale settle, the arrival and
  departure of each pane, one duration). Goldens `control_center/panes-{root,detail}.html`.
  `ds-native/tests/cc_pane_switcher.rs`: after a switch both panes are drawn, entering and leaving,
  and the switcher's height is the short detail's, not the tall root's; 350 ms later only the
  detail is drawn, present, and `on_settled` logged `detail`. Reversed 150 ms into a switch, the
  root turns back to `entering` and the detail to `leaving`; past the first round's settle both are
  still drawn and nothing is logged; then the root rests and the log is `root` alone. The drift
  test's settle table has the four at 284 ms (Standard).
- **Limits.** The height snaps to the arriving pane rather than animating (no measured height
  transition on Blitz); the leaving pane is clipped where it overhangs. A reversal restarts both
  animations from their first frame instead of from where they were.

### Q81: control glyphs

- **What quire does now.** `Icon::Play`, `Pause`, `SkipBack`, `SkipForward`, `LogOut`, `Restart`
  (Lucide `rotate-ccw`), `Headphones`, `Speaker`, `Mouse`, `Gamepad` and `Phone` (Lucide
  `smartphone`), transcribed from `lucide-static` 1.47.0 into `icon/geometry_control.rs` (ISC,
  the licence line in `icon/mod.rs`; a `<line>` as the equivalent path, as the shell set does).
  `Power` was already in the shell set. `Icon::CONTROL` lists them and `Icon::ALL` ends with them;
  the named sets moved to `icon/sets.rs` so `icon/mod.rs` is under 300 lines. The Controls page
  draws every glyph by set at 22 px.
- **Proof.** `every_control_glyph_renders_and_lints_clean` (one child per shape, `ds-ic`, clean
  under the Strict profile), `the_control_set_is_lucides`, and the set test counting `ALL`.

## Edit surface 2 (2026-09-25)

mailo's composer runs on `EditSurface`; its six follow-ups, branch `edit-surface-2`. Proofs:
`crates/ds-native/tests/native_edit2.rs`, `native_edit_stacking.rs`, the SSR golden
`controls/edit_surface/app-class-and-data.html`, `tokens/pixel.rs`'s tables.

1. **Class and data.** `EditSurface { extra_class: Option<ExtraClass>, data: Vec<DataAttr> }`,
   the types mailo gaps 6 gave `Button` (`components/pass_through.rs`, brought over byte for byte
   before that branch landed; the merge of master took it without a conflict).
2. **A programmatic focus is a press's focus.** `EditHandle::focus()` used to call
   `focus_soon`: a host focus write dispatches no `focus` event, so `on_focus` never heard `In`
   and the IME stayed off. The surface now hands its handle its own focus and blur
   (`SurfaceHooks`): `focus()` goes through the same `focus_soon_told` a press uses, and on
   success runs the surface's focus-in (`on_focus(In)`, the IME on and at `ime_area`); the
   surface is the IME's target because routing follows the document's focus. `blur()` is the
   `ds::HostBlur` write plus the surface's focus-out (`on_focus(Out)`, an open composition
   ended empty, the IME off). Proof: focus, a composition that reaches the surface, blur, and a
   commit after it that does not.
3. **Harness input.** `Key::{Home, End, Delete, PageUp, PageDown, Insert}` (with key-cap glyphs,
   pending O-2 sign-off like the arrows); `HeldButtons` (the harness's mouse buttons down, carried
   by every move between a press and its release, `Harness::held_buttons`); `Modifiers` on
   `pointer_move_with`, `button_down_with`, `button_up_with`, `click_with`; `drag(from, to,
   steps)`. The other keyboard-types keys (function keys, media keys) are not added: nothing
   tests them. Proofs: Shift+click reports `Extend::FromAnchor` at `p1:0`; a drag from `p0` to
   `p1` reports Press at `p0`, then Drag and Release in `p1`.
4. **Pointer capture, done without Blitz changes.** A press on the surface registers its sink
   with the document's `EditListeners` (`HostEdit::capture`); until the primary release every
   pointer move and the release go to it, wherever the pointer is. The window's hook hears
   winit's `PointerMoved`/`PointerButton` before the document (the same ordering the IME route
   uses; logical position = physical / scale factor; modifiers from `ModifiersChanged`), and the
   harness routes its own events before handing them to the document. While captured the
   surface ignores its own `pointermove`/`pointerup`, so nothing is heard twice. Proof: a drag
   that ends over a button below the surface still reports its drags and the release (the
   position resolves to the nearest text), and a move after the release reports nothing; with
   the routing switched off the test fails.
5. **Stacking: Blitz agrees with CSS 2.1 Appendix E here; the surface is not the cause.**
   Pixel tests (`native_edit_stacking.rs`): a red `position:absolute` layer, then a blue block over
   the same box. Blocks with `position:relative` (z-index auto) paint over the earlier layer (step 8,
   tree order); a static block paints under it (steps 3/7 before 8); `z-index:1` is over it; a
   `z-index:-1` layer goes under its container's own background when the container makes no
   stacking context (the root's step 2). With mailo's shape, a red layer then an `EditSurface`
   with `extra_class: c-body`: when `.c-body{position:relative}` the text's ink is over the layer;
   without it the layer covers the text, as a browser would draw it. So the rule for mailo:
   **the selection layer before the surface, `.c-body` positioned, no `z-index` on either** (or a
   positive `z-index` on `.c-body`). mailo's layer painted over its text means its `.c-body` was
   not positioned in the end (the rule not applying, or the positioned box being a wrapper that
   no longer exists now the surface carries the class): the opacity workaround can go.
   `EditSurface`'s own sheet sets no `position`, which is right: the app decides.
6. **`--caret-w`.** `PixelToken::CaretW` (`--caret-w: var(--scale-caret-w, 1px)`), 1 px floored
   to whole device pixels like `--hair`, and `PixelToken::logical(scale)`. `caret_rect` is that
   wide at the document's scale (1 px at 1x and 2x, 0.667 px at 1.5), from the insertion point
   rightwards. The stylesheet golden gains the one declaration; the root's inline writes gain
   `--scale-caret-w` at a fractional scale. The gallery's caret draws `width: var(--caret-w)`.

Still open: a composition interrupted by a focus change ends empty; capture in hosts other than
`launch` and the harness (shell-host, sill) is not routed, as their IME is not.
## Timing tests (2026-09-25)

Two flakes under a loaded parallel `cargo test --workspace` (`tray_gaps.rs::a_rest_opens_the_submenu_after_the_delay_and_left_closes_it`, `bar_menu.rs::escape_fades_the_menu_out_before_it_closes`) led to a sweep of every `ds-native` integration test and `examples/consumer/tests/coherence.rs` for the same fault: `ds_native::Harness::advance` lets real wall-clock time pass (quire's settle timers are `futures-timer` sleeps a harness cannot fake, its own module doc says so), and it only ever guarantees *at least* the time asked for — a busy machine can stretch an `advance(ms(N))` past a boundary the test meant to stop short of. A "not yet" assertion taken more than half-way through a timer's window, or any check of an exact state at a fixed instant within tens of ms of a settle/hover-intent/submenu/toast boundary, is fragile; a "not yet" kept at or under half the window, or an "already true" check with a comfortable cushion past the boundary, is safe (real time only ever overshoots forward, so it can only make an "already true" check *more* true, never less).

- **The rule (now CONVENTIONS.md §11).** Never assert a state at one fixed instant near a timer boundary. Poll with `ds_native::harness::settle_until(&mut harness, |h| ..)` (new: 10 ms steps, 3 s bound, returns the wall-clock `Instant` the condition first held, panics with the document's HTML on timeout) up to its bound, and assert order — A landed before B, or landed only once at least the window's duration had passed since it was asked for (`landed.duration_since(asked) >= window`, always safe since real time only overshoots) — rather than a state at an instant. A "not yet" check may still assert a fixed elapsed time, but only at or under half the window.
- **The helper.** `crates/ds-native/src/harness_settle.rs`, `pub fn settle_until` and `pub const SETTLE_BOUND`, re-exported at `ds_native::harness::settle_until`. `cc_pane_switcher.rs` and `coherence.rs` each already carried their own copy of the same pattern (from the `cc-test-fix` fix that landed just before this sweep, 7955e20); both were switched onto the shared one and their local copies deleted.
- **Marking the clock.** The `Instant` a test times a settle from must be taken *before* the action that starts the timer, not after: `show()`/`click()` themselves call `Harness::advance`, and the real time that one call takes (never exactly what was asked) is lost if the clock starts after it returns, undermining the `>= window` lower bound. (Found while verifying `osd_presence.rs`: capturing the instant after `show()` returned measured 276 ms against an 284 ms `settle(OsdOut)` bound and failed once under load; moving it to before `show()` fixed it for good.)
- **Tests rewritten** (12, all now green across three `cargo test --workspace --no-fail-fast` runs plus one run under a concurrent `cargo build -p ds-gallery --release`):
  - `tray_gaps.rs::a_rest_opens_the_submenu_after_the_delay_and_left_closes_it` — "not open" was checked at 120 of 200 ms (60 %); now 80 ms (40 %) plus `settle_until` and `opened.duration_since(rested) >= 200ms`.
  - `bar_menu.rs::escape_fades_the_menu_out_before_it_closes` — "still fading" at `fade() - 40ms` (76 % of 170 ms); now `fade() / 2`.
  - `crates/ds-native/tests/harness.rs::the_toast_hub_hides_after_its_hold_and_not_before` and `::a_toast_hides_after_5200_ms` — "still shown" at 5000 of 5200 ms (96 %, only 200 ms of cushion); now ≤2600 ms.
  - `crates/ds-native/tests/harness.rs::the_hover_hub_opens_after_450_ms_and_not_before` and `::a_hover_card_appears_after_450_ms_and_not_before` — "not open" at 400 of 450 ms (89 %); now 200 ms.
  - `mailo4_hover.rs::a_card_keyed_on_the_callers_hooks_opens_beside_its_measured_element`, `mailo4_hover.rs::with_no_layout_an_unplaced_card_opens_in_place_on_the_hubs_timing` (both the 450 ms open check and the 150 ms close check) and `mailo_hover.rs::a_hover_target_on_an_li_opens_its_card_beside_the_item` — the same 400-of-450 (and 100-of-150) pattern as the hub tests above.
  - `osd_presence.rs::hidden_it_fades_and_on_hidden_runs_at_settle_and_not_before` — "not before settle(OsdOut)" at `out - 40ms` (a `MARGIN` constant, 84 % of ~250 ms); the `MARGIN` const is gone.
  - `roster_stay.rs::a_row_folded_again_after_a_stay_settles_on_its_own_clock` — "still playing" at 250 of `settle(Fold)` ≈ 454 ms (55 %, just over the line).
  - `send_pill_moods.rs::a_nudge_plays_once_settles_and_the_pill_stays` — "still playing" at `settles - 100ms` (81 % of 520 ms).
- **Left as-is, with why** (full sweep of `crates/ds-native/tests/**`, `examples/consumer/tests/coherence.rs`; no `#[cfg(test)]` in `ds-native` src calls `advance`): the large majority are either synchronous checks right after an event with no timer in play, or "already happened" checks with 200+ ms of cushion past their boundary (safe, since overshoot only helps them). Cases worth naming because they look closer to the line than they are:
  - `roster_stay.rs`'s first test checks an undo precondition at 200 of `settle(Fold)` ≈ 454 ms (44 %) — under the half-window line, kept.
  - `level_control.rs::a_level_set_from_outside_slides_over_t_quick` samples the eased fill at 25 %/50 % of 170 ms with a ±3 px tolerance on the harness's own animation clock (`resolve(t)`, not the wall clock `advance` reads for timers) — a continuous, monotonic value, not a boundary flip, so jitter in *when* the sample lands cannot flip it from true to false.
  - `window_frame.rs::a_space_switch_cross_fades_the_window` and `bar_frame.rs`'s `half_of_the_fade` sample a mid-fade pixel and assert it lies between the before/after values (or early in an ease-out curve) — again monotonic, immune to sampling jitter.
  - `contact_motion.rs` samples keyframe offsets against the harness's own animation clock with a static `data-pulse`, never a wall-clock timer.
  - `coherence.rs::the_sent_badge_times_out_on_ds_motions_own_clock` already asserted only the "after settle" half (its own comment already called out why the "not before" half was dropped); nothing to change.
  - `launcher_crash.rs::run_script` already polls rather than asserting at an instant.

## Frame tags and link text (2026-09-25)

mailo's sealed Original view (working on v0.1.7/8; isolation, no-script, no-file and
no-navigation hold under its hostile-body suite) reported two additive gaps in ds-native's frame
support. Branch `frame-tags`, one commit per item; the blitz rev is unchanged (`e99fbdbd`).

1. **Which frame is which.** A `FrameId` named a Blitz document and had no public accessor, and
   `FrameView` exists only in the Harness, so mailo inferred which message a frame showed from the
   URLs it requested and keyed frames by hashing the id.
   - **What quire does now.** The app writes `data-frame-tag="<text>"` on its `iframe`. The tag is
     `FrameTag(String)` on `NetRequest::frame_tag()`, on `FrameLink::tag`, and through
     `ds_native::frames::{tag_of, frame_by_tag}`; `FrameId::index()` is a stable key.
   - **Found: a frame's first requests are made before it has an element.** Blitz's
     `attach_iframe_document` parses the sub-document through the parser provider (which is
     handed only the HTML and a `DocumentConfig`, nothing naming the `iframe`) and attaches it to
     the element afterwards; the parse is where the body's images are asked for. The parent is
     mutably borrowed throughout, so nothing can read the element's attribute then. ds-native
     therefore holds each frame's requests for the app in a per-document `FrameBook`
     (`frame_book.rs`) until the next walk of the document's `iframe`s (`frame_tree.rs`, run after
     each render in the harness and on each redraw in the window; an attach asks for a redraw)
     finds the frame's document under its element and reads the tag; then the held requests go
     to `AppNet::decide` with it. `data:` is served at once as before (it never reaches the app),
     and `Local`/`Sealed` are untouched. A frame gone before it is found drops what it held.
   - **The lookups need no context.** Each document's book registers with a thread-local list,
     so `tag_of`/`frame_by_tag` answer from any handler on the UI thread (and a test's thread),
     and a book is forgotten with its document.
   - Proof: `native_frame_tags.rs`. Two srcdoc frames tagged `msg-1` and `msg-2`: each frame's
     image request reaches the app as `Frame(id)` with its own tag; a click in each carries its
     tag; `frame_by_tag` finds each frame's id (as `Harness::frame` reports it) and `tag_of` the
     reverse; after the harness is dropped neither lookup finds anything. With the walk removed
     the requests never reach the app (checked by hand).
2. **What a link says.** `FrameLink` had only `href`; mailo's link-honesty check compares a
   link's visible text with its destination, and its link pill needs to know when the pointer is
   on a link inside the frame, which Blitz never tells the app (it forwards the move into the
   frame's document and keeps the hover state there).
   - **What quire does now.** `FrameLink` has `text` (the anchor's text content, whitespace
     runs collapsed and trimmed) and `title`. `FrameLinks::intercept(..).with_hover(handler)`
     delivers `FrameLinkHover { frame, tag, href, text, title, at, phase: Enter | Leave }`.
   - **Found: the navigation provider hears only a URL.** Blitz's click default calls
     `navigate_to` with the resolved URL and the source document id, while the document is
     held. The text is read when the click is delivered, with the document free: the frame's
     document is found again by id (`frame_tree::in_frame`) and the anchor is the one under its
     hover node (a click), else the focused one (a key), else the first `a[href]` resolving to
     the same URL (`frame_anchor.rs`); a vanished anchor gives empty text.
   - **Hover is hit-tested by ds-native, not read from Blitz.** The window's event hook hears a
     move before the document does, so reading the frame's hover node there would lag one move.
     `frame_hit.rs` hit-tests the app's document at the point, and on an `iframe` goes into its
     document the way Blitz forwards events (minus the element's position, plus the frame's
     scroll); the nearest `a[href]` ancestor is the link. `HoverTracker` keeps the last link
     (frame and node) and reports only a change: a leave before an enter, nothing while on the
     same link or on none. The window converts winit's physical position by the scale factor
     and treats `PointerLeft` as off every link; the harness reports from `pointer_move`. With
     `FrameHover::Ignore` (the default, and always on `Inert`) nothing is hit-tested.
   - **Breaking, narrowly.** `FrameLinks::Intercept` became `Intercept { click, hover }`;
     `FrameLinks::intercept` and `Inert` are unchanged, and mailo builds its links with them.
     `FrameLink`'s new fields change a struct literal of it (the frame-links test's).
   - Proof: `native_frame_link_text.rs`. A frame placed 20 px in and 40 px down with two links
     and a paragraph: clicks carry `"Your bank"` with title `Sign in` (from `Your\n   bank`) and
     `"Offer"` with none. Moving onto the first link reports one `Enter` with its href, text,
     title, tag and the move's point; two more moves on it report nothing; moving onto the
     paragraph reports `Leave` at the paragraph. Link to link is `Leave` then `Enter` at the same
     point, and onto the app's own text is `Leave`. `Inert.with_hover(..)` reports nothing.
   - **Not run in a window here.** The window path (`window_hover.rs`, the hook in `host.rs`,
     reading a click's text through the document handle in the link task) is compiled and
     shares every function the harness tests call, but no test drives winit.

## Window frame (2026-09-25)

Our client-decorated windows need a movable, resizable, zoomable frame: mailo on Blitz through
`ds_native::launch`, and later the shell's apps on shell-host's xdg toplevels. Branch
`window-frame`, one commit per item; blitz rev unchanged (`e99fbdbd`). Proofs:
`crates/ds-native/tests/window_frame_controls.rs` (harness, stub host),
`crates/ds/tests/window_frame_ssr.rs` (goldens under `controls/window_frame/`), the unit tests in
`ds/src/window/{grab,hold}.rs` and `ds-native/src/window_place.rs`, and the Polish page's
"Window frame" section. Nothing was run on a live session (the user was away; nothing touched
`wayland-0`), so every platform claim below is read from the pinned sources, not observed.

1. **The seam.** `ds::HostWindow` is a trait (each host holds its own window handle, and a test's
   stub records what it was asked), provided as `ds::WindowHost`, which also carries the last
   `WindowState` as a signal for `use_window_state()`. The brief's `Tile` is `WindowTile`:
   `ds::Tile` is already a menu row's tile, re-exported at the crate root. `Availability` has no
   `Unavailable`; the menu's rows use `Availability::Disabled`. shell-host's side landed on its
   master (`cb9078c`) while this branch was built: `SurfaceHandle::begin_move()` and
   `begin_resize(ResizeEdge) -> Result<(), GrabRefused>`, `set_maximized(Maximize)`,
   `set_minimized()`, `set_fullscreen(Fullscreen)` and `use_toplevel_state()`. Its `ResizeEdge`
   has the same eight variants as ours, so sill's `HostWindow` is a variant-for-variant map
   (CONSUMING "Window frame"). Its F50 reaches the conclusion of item 5 below independently.
2. **winit is 0.31.0-beta.3, not 0.30** (dioxus-native at the pinned rev). What the frame relies
   on, read in `winit-core` and `winit-wayland` 0.31.0-beta.3:
   - `drag_window` and `drag_resize_window` send `xdg_toplevel.move`/`resize` with the pointer's
     `latest_button_serial`, which every press and release updates. While the button is held
     it is the press's serial, so asking for the move on the first motion past the threshold
     (not on the press, which would steal a double-click) is valid. After the release it is not,
     so a move asked late does nothing.
   - `outer_position` answers `NotSupported` for a Wayland toplevel ("the compositor does not
     report absolute positions"), and `set_outer_position`'s Wayland body is the comment "Not
     possible.". So a client cannot place itself on Wayland: `WinitWindow::supports` answers
     `Support::No` for Left half, Right half and Centre whenever `outer_position` fails or no
     monitor position is known, and `tile` returns `TileError::Unsupported`. On X11 (and
     wherever winit can place a window) they are placed from `current_monitor()`'s position and
     video-mode size. winit reports no work area, so a half there covers the output's panels
     too. Not tried on a live X11 server.
   - `set_minimized(false)` is ignored on Wayland and `is_minimized` is always `None`; the seam
     has no un-minimize. `is_maximized` and `fullscreen` read the last configure, `has_focus`
     the keyboard focus, so `launch` refreshes the state on every `SurfaceResized` and `Focused`.
   - `show_window_menu` is implemented for Wayland toplevels in this beta although its doc says
     unsupported. The frame does not use it; a right-click on the titlebar could.
   - Close has no winit request: blitz-shell's `ShellProvider::request_window_close` (the
     document's shell provider, a root context) sends `BlitzShellEvent::CloseWindow`, handled as
     `CloseRequested`: the window is dropped and the loop exits. There is no veto hook, so an app
     with unsaved work has nowhere to intercept the red light yet.
   - `with_decorations(false)` (`Decorations::Client`) is how the app asks for no second frame;
     winit-wayland is built with `sctk-adwaita`, so `Server` on a compositor with no
     xdg-decoration server mode draws winit's adwaita frame.
3. **Blitz double-click.** `dblclick` arrives on the titlebar (the keep-focus work found the
   root's click fallback leaves it alone; `a_double_click_on_the_titlebar_zooms`). A light stops
   its own `dblclick` and `pointerdown`; with that removed the harness logs `close,close,zoom:Toggle`
   and a move, so the tests prove the stop.
4. **Selectors in the harness** need the attribute namespace (`[*|data-light=zoom]`), as the
   stylesheet does (spike S2); `[data-light=zoom]` matches nothing.
5. **Compositor gaps: placements no client protocol offers.** Only Fill has a request a client
   may send: `xdg_toplevel.set_maximized`. Left half, Right half and Centre need a way to set a
   toplevel's position (and, for the halves, a work-area size), and none exists:
   - xdg-shell (stable, as vendored by wayland-protocols 0.32.13): `xdg_toplevel`'s requests
     are `destroy`, `set_parent`, `set_title`, `set_app_id`, `show_window_menu`, `move`,
     `resize`, `set_max_size`, `set_min_size`, `set_maximized`, `unset_maximized`,
     `set_fullscreen`, `unset_fullscreen`, `set_minimized`. The `tiled_left/right/top/bottom`
     states go the other way, compositor to client. None of the staging protocols vendored there
     (`xdg-toplevel-drag`, `xdg-toplevel-tag`, `xdg-toplevel-icon`, `xdg-dialog`,
     `xdg-session-management`) sets a position.
   - cosmic-comp: `zcosmic_toplevel_manager_v1` (v4, the shell's privileged protocol, not an
     app's) has `close`, `activate`, `set/unset_maximized`, `set/unset_minimized`,
     `set/unset_fullscreen`, `set_rectangle` (a minimize animation's target, in surface
     coordinates), `set/unset_sticky` and workspace moves. No geometry.
   - KWin: `org_kde_plasma_window` (plasma-window-management, also privileged) has `set_state`,
     `set_virtual_desktop`, `set/unset_minimized_geometry`, `highlight`, `close`,
     `request_move`, `request_resize` (both interactive), `get_icon`, virtual-desktop and
     activity requests and `send_to_output`. No geometry.
   So Left half, Right half and Centre are compositor gaps on both cosmic-comp and KWin, for our
   apps and for sill alike: until a compositor offers a placement request, the menu shows them
   unavailable on Wayland. (KWin's scripting D-Bus interface can set a window's geometry; that is
   not a Wayland protocol and was not examined.)
6. **Choices the brief left open.** The frame is a `Ds` prop, `window: WindowFrame` (the prop
   `frame` is `FrameTint`'s), default `None`, so every existing golden is unchanged (only the
   stylesheet golden grew). The lights are the frame's own `button.ds-light`s, not
   `IconButton`s: a 12 px disc with a mark is not one of IconButton's variants, and a new
   variant would have edited `icon_button.rs`, which a sibling branch owns; they use the same
   press path (a click, the keyboard) and their marks are quire's own vectors
   (`data-ds-svg="light"`). Both a long press (500 ms) and a hover-hold (800 ms: the 450 ms hover
   intent and 350 ms more) open the tiling menu, so passing over the green light on the way to
   close does not; right-click and ArrowDown open it too. The grey rule is macOS's: inactive
   windows are grey until the pointer is over the group; the active window keeps its colours.
## Control center parts 2 (2026-09-25)

Six follow-ups from sill's finished control center (sill FINDINGS Q100-Q105). Additive except
two visible changes: `ModuleGrid` now pads itself by 12 (the key's default), and the light
level well is darker. CONSUMING "Control center parts 2 (2026-09-25)" has the API.

### Q100: a module that holds content

- **What was wrong.** `ModuleTile` is a toggle with a glyph, a title and a status and holds no
  children, so sill drew the Sound, Display, Now Playing, Appearance and Battery modules on a
  plate of its own (`.sill-cc-module`: `--r-tile`, `--surface-2`, a hairline, 10/12 padding),
  and a second rule to drop that plate in a bar item's dropdown.
- **What quire does now.** `ModulePanel { glyph, title, trailing, span, plate, children }`: the
  Off tile's plate at `--r-tile` with the module padding of 10/12; an optional header row (a 16
  px glyph in `--ink-soft`, the title in the tile's 13/600, the trailing slot at the far end in
  the status type with tabular figures, for a level's percentage) over the children. Not
  pressable (no role, no handler, no hover lift): its content takes every press. `span` is
  `Full` by default (`grid-column:1 / -1`); `PanelPlate::Bare` drops plate, border and padding
  where a popover card already is the module's.
- **Proof.** Goldens `control_center/panel-level-{light,dark}.html` and `panel-bare.html`, linted,
  every class styled. `ds-native/tests/cc_module_panel.rs`: on a 320 px grid the panel is at x 12,
  296 wide; a press at 30 % of the level's rail sets 30 %, a drag to 80 % follows, and the
  header's trailing slot follows the level.

### Q101: the picker at the control center's width

- **What was wrong.** In a 296 px module (270 px of content) `AppearancePicker`'s Motion row is
  331 px (five regular segments), so "Reduced" was clipped.
- **What quire does now.** `layout: PickerLayout::{Full, Compact}` (Full by default, unchanged).
  Compact sets the segmented rows at the small size, each row filling the width it has, with
  segments of 4 px sides that grow from their words' width (`flex:1 1 auto`, `min-width` left at
  auto), so no label is squeezed and every row ends inside the content box. The markup writes
  `data-layout`.
- **Proof.** `ds-native/tests/cc_picker_compact.rs` at 296: Full's Motion row ends past the
  content box (the bug is real); in Compact every row (Theme, the swatches, Motion) ends at
  x 295, the content box's edge, and the last segment is "Reduced", whole. Golden
  `control_center/picker-compact.html`; the three existing picker goldens gained
  `data-layout="full"` only.

### Q102: the grid's columns, gap and padding

- **What was wrong.** `ModuleGrid` was two columns at gap 8, fixed, and left its padding to the
  panel: `control_center.grid_columns`, `grid_gap_px` and `grid_padding_px` sized only sill's
  popup estimate.
- **What quire does now.** `ModuleGrid { columns: GridColumns, gap: Px, padding: Px }`, defaults
  2/8/12 (design/22 section 3.13), written inline as `--grid-columns`, `--grid-gap`,
  `--grid-padding` (`GridMetrics::style_attr`: lengths rounded, never negative, never fewer than
  one column) and read by `repeat(var(--grid-columns),minmax(0,1fr))`. `TileSpan::Full` and a
  Full `ModulePanel` span every column. **Visible change:** the grid now pads itself by 12, so a
  consumer that pads its panel too either passes `padding: Px(0.0)` or drops its own.
- **Proof.** `ds-native/tests/cc_module_grid.rs`: three columns, gap 6, padding 10 at 320 put
  three 96 px tiles in one row at x 10, 112, 214 and a Full tile at x 10, 300 wide. Golden
  `control_center/grid-3-columns.html`; `panes-root.html` gained the inline metrics only.

### Q103: a control center glyph

- **What was wrong.** sill's control center bar item used `Settings` (a gear), which reads as
  "open Settings", not as a panel of switches.
- **What quire does now.** `Icon::Switches` (the end of the enum and of `Icon::SHELL`): two pill
  tracks 20 x 8 (`rx` 4) at y 2 and y 14 on Lucide's 24 grid and 2 px round stroke, 2 px of clear
  space between their strokes, each with a dot knob of radius 1 centred in an end cap: left on the
  top track, right on the bottom. Composed from Lucide `toggle-left`/`toggle-right`'s parts and
  stacked as `sliders-horizontal` stacks its rails; macOS draws its control center as two
  switches too, but nothing is traced from its symbol. The geometry and why each number is what
  it is are in `icon/geometry_own.rs`. At 16 px a ring knob would touch its track, so the knob is
  a stroked point, as Lucide draws its dots.
- **Proof.** `icon::tests::switches_is_two_tracks_with_knobs_at_opposite_ends`; the control glyph
  lint in `control_center_ssr.rs` covers it through the shell set's rendering; the gallery's
  glyph page shows it beside `Settings` and `Wifi` (it iterates `Icon::SHELL`). Checked by eye at
  22 and 16 px at 3x.

### Q104: a document's frame must have a height

- **What was wrong.** Three sill surfaces met the same bug: the wallpaper (F172), every bar popup
  (F225) and, earlier, the launcher's catcher. A frame with `position:absolute; inset:0` (or
  `fixed`, or `height:100%`) resolves against `#main` and `.ds`, which Blitz lays out at
  `height:auto`; with its content out of flow the frame is 0 px, and so is every box from `html`
  down. Nothing said so in CONSUMING.
- **What quire does now.** CONSUMING section 2 "A document's frame must have a height": the rule,
  the reason, `RootExtent::Viewport` (the sheet-parts branch, not on master at this writing) or
  until then the floor (`display:grid; width:100vw; min-height:100vh` on the frame, which
  stretches the root to fill its one cell), and a debug tip (the Harness's `rect` of `html`,
  `.ds` and the frame; `count(".ds")`; `hits`).
- **Proof.** `ds-native/tests/root_frame.rs`, with a card placed absolutely in the root as a popup
  card is: the absolutely placed frame with no floor is 0 px, and so are `.ds` and `html`; the
  floor inside an absolutely placed frame and an in-flow frame both give the frame and the root
  160 px (the viewport) and take a press on the card's button.
- **Found on the way.** In the harness a press on the button inside the 0 px frame still lands,
  where sill's live popup (F225) took none: Blitz's hit test and the shell's input path differ on
  0 px ancestors. So the test holds the heights, which are what the rule is about, and the debug
  tip says to trust the rects over a click.

### Q105: the capsule's fill against its well, light scheme

- **What was wrong.** On light the capsule's fill (near-white, `rgba(253,253,251,.97)`) sat over a
  black .10 well, which on the control center's `--surface-2` plate is itself nearly white.
- **Measured** (`ds-native/tests/level_contrast.rs`: the painted fill at 40 % of the track and
  the well at 80 %, middle row, level at 50 %, WCAG contrast ratio):

  | Ground | Before (well .10) | After (well .18) | Dark (unchanged) |
  | --- | --- | --- | --- |
  | The paper (a Window root) | 1.55:1 | 1.85:1 | 10.21:1 |
  | The module plate (`ModulePanel` in a painted Popover) | 1.37:1 | 1.66:1 | 8.79:1 |
  | The OSD card over the Work tint (the level sheet's rows) | 1.64:1 | 1.93:1 | 9.95:1 |

- **What quire does now.** The light well is black .18 (`material/level.rs`); the fill, the glyph
  inks and every dark ink are unchanged. .17 held the module at 1.61, too close to the floor. The
  test holds light at 1.6:1 or more on all three grounds. `ds-gallery --level-sheet` re-rendered
  `tools/progress/shots/gallery/level-variants.png` and `level-motion.png` under the same names.
## Sheet and modal parts (2026-09-25)

sill's power menu (a centred sheet, design/20 §1.8) found six gaps, Q90-Q95, and a scrim too
light for a modal. Branch `sheet-parts`. Everything is additive; the goldens that moved are named
below.

- **Q90: the sheet had no exit.** `Anim::SheetOut` (`sheet-out`: fade, a .98 scale and 8 px back
  down, `--t-move --e-exit`, forwards) joins the motion table with its recipe, `X--b` alias, pulse
  class, settle row (284 ms Standard and Calm) and drift entries; `Anim::ALL` is 60 after the
  merge. `Sheet { shown: Option<Shown>, on_hidden }` runs a small machine (`sheet_presence.rs`,
  `Stage::{Up, Leaving, Gone}`, a pure `step`): hidden, the sheet goes `leaving`, withdraws from
  the layer stack, its scrim plays `menu-out`, and `on_hidden` runs from the exit timer's settle;
  shown again while leaving, the stage is `Up` again, the entrance timer restarts and the stale
  exit settle finds the stage is no longer `Leaving` and does nothing. Both timers' handlers are
  made in render (one made inside `queue_effect` has no scope and panicked, as the pane switcher
  found). Proof: `ds-native/tests/sheet_exit.rs` (60 ms before `settle(SheetOut)` nothing is
  logged and the sheet is drawn; 20 ms after it `hidden` is logged and the sheet is gone; a
  re-show 100 ms into the exit enters, logs nothing past the old settle, and a later hide logs
  once); goldens `overlays/sheet/{shown,mounted-hidden}.html`; `sheet/default.html` unchanged.
- **Q91: centred.** `SheetPlacement::Centre` wraps the sheet in `div.ds-sheet-stage` (absolute,
  inset 0, flex-centred, the 36 px inset as padding, no pointer) and makes the sheet `position:
  relative`. No transform: the entrance and exit keyframes own it, and Blitz's layout rect leaves
  a transform out (as with the hover strip). Proof: in a 640 x 400 viewport root the sheet's
  middle is within 1 px of 200 and of 320; golden `overlays/sheet/centre.html`.
- **Q92: Danger's size.** `Button { size: Option<ButtonSize> }`; `None` writes nothing and keeps
  the variant's size (Danger stays Mini-sized, as its goldens encode), so no golden moved.
  `Regular` is Primary's box in the variant's colours, `Mini` is Mini's. Proof: a Danger Regular
  is exactly the Secondary's height beside it, a sizeless Danger more than 4 px shorter; goldens
  `controls/button/{danger-regular,primary-mini}.html`.
- **Q93: disabled.** `Availability::Disabled` (the existing vocabulary: `Enabled`/`Disabled`, not
  `Available`/`Unavailable`) now writes `disabled="true"` beside `aria-disabled`, and both sheets
  draw it at .35, the settled disabled menu item of design/13 §13.3.3 (design/04's O-1 only has an
  unsigned .45 candidate), with the default cursor and `pointer-events:none`, so no hover lift,
  wash or squish can match. The attribute is written only when disabled: passing a `bool` wrote
  `disabled="false"` on every enabled button, which Blitz treats as disabled, and a Bubble press
  stopped toggling its `<details>` (`mailo5_propagation` caught it). Proof:
  `ds-native/tests/sheet_buttons.rs` (a press on a disabled Button and IconButton logs nothing,
  the enabled neighbour's logs; the disabled label's darkest ink is far lighter, and with the
  opacity rule removed the test fails at 271 against 271). Goldens: `controls/button/disabled`
  and `controls/icon_button/disabled` gained the attribute; `danger-regular-disabled` is new.
- **Q94: the root's height.** A `.ds` root is a block as tall as its content; a root holding only
  positioned content (a centred sheet, an OSD card, a catcher) is 0 px tall, and so is its
  overlay and the sheet's `max-height:calc(100% - 72px)`, so the sheet drew no body at all. This
  is sill F172's trap again: dioxus-native-dom's `#main` is `height:auto`, so no `height:100%`
  above the root resolves, and a `position:fixed; inset:0` box is placed by Taffy against its
  parent (the 0 px root), not the viewport. `Ds { extent: RootExtent::Viewport }` writes
  `data-extent="viewport"` and `min-height:100vh; min-width:100vw` (vh and vw do resolve against
  the viewport on Blitz, which is how sill fixed its wallpaper). Proof:
  `ds-native/tests/root_extent.rs` (a `Content` root and its sheet measure 0 px tall; a `Viewport`
  root measures 640 x 400 and its sheet has its content's height). `data-extent` joins the root
  attributes the lint forbids a consumer to select on.
- **Q95: Kbd arrows.** The cause was the font, not the stroke: `space-mono-normal-*-latin.ttf` is
  Google Fonts' `latin` subset, whose `unicode-range` keeps `↑` (U+2191) and `↓` (U+2193) but
  not `←` (U+2190) or `→` (U+2192). Those two fell back to a system face, whose arrows at 9.5 px
  are a thin shaft with a head about a pixel tall: a dash. Fixed at the source: both latin files
  are re-subset from the Space Mono 1.003 release (`google/fonts` `ofl/spacemono`, the version the
  old files carry, with the same hinting programs) with the old files' own code points plus
  U+2190-2193 (fontTools `Subsetter`, all layout features, glyph names kept); every shared glyph's
  outline and advance compare equal. The woff2 copies are regenerated and the webview's `latin`
  range names U+2190-2193. A second Blitz fact surfaced: Blitz lays out an inline box's text but
  not its padding or border, so every cap was its glyph on a patch of `--surface-2` with the caps
  run together; `.ds-kbd` is now `inline-block`. Proof: `ds-native/tests/kbd_arrows.rs` (a Small
  cap's `←` and `→` ink is at least 5 px long, at least 3 px and a third of its length tall, and
  as long as the `↑` is tall, within 2 px); with the old font files it fails, `←` 4 px long.
- **Scrim strength.** `--scrim-modal`, black .40 light and .55 dark, as a `ColourToken`
  (`ScrimModal`) rather than the `OpacityToken` the brief named: `--scrim` is itself a colour and
  `OpacityToken` has one value for both schemes. `Scrim { strength: ScrimStrength::Modal }` and
  `Sheet { scrim }` write `data-strength="modal"`. The legibility gate is in design/03 §17.3.1:
  3.05:1 sheet-to-paper in light (1.89 under `--scrim`), 1.09 against 1.04 in dark, ink 15.6 and
  15.0:1. Goldens `overlays/scrim/modal.html`, `overlays/sheet/modal-scrim.html`.
- **Gallery.** The Overlays page's Power menu section (the page is 5700 tall): the centred sheet
  over the modal scrim in its own Sheet root, light and dark, with Cancel, a disabled Suspend, a
  Danger Restart and Shut Down, and Small caps for the arrows, Enter and Escape.

## Button heights and small key caps (2026-09-25)

Two Sheet and modal parts follow-ups from the power menu review, sill Q110 and Q111. Branch
`button-kbd-fixes`. Both additive: no markup a caller wrote changes shape, only a border colour
and a `data-glyph` attribute that was always absent before.

- **Q110: a Primary stood shorter than its neighbours.** `ButtonVariant::Primary` drew with
  `border:0` while Secondary (which shares Primary's rule) and Danger both carry
  `border:var(--hair) solid var(--line)`. At `ButtonSize::Regular` that is a `--hair` top and
  bottom Primary never paid for: 36 px against Secondary and Danger's 38 px, so a Shut Down
  (Primary) sat visibly shorter than the Cancel and Restart beside it. Fixed in the box model,
  not the paint: Primary (and Secondary, before Secondary's own rule repaints it) now carries
  `border:var(--hair) solid transparent`. `background-clip`'s default is `border-box`, so a
  transparent border does not create a gap — the background already painted there shows through
  the border area exactly as it did at `border:0` — but the border now costs the same box-model
  height Secondary and Danger's `--hair` already costs, so all three match at both sizes the
  power menu draws them at. Proof: `ds-native/tests/button_heights.rs`
  (`primary_secondary_and_danger_share_one_height_at_regular_and_at_mini`) lays out Primary,
  Secondary and Danger side by side at `ButtonSize::Regular`, then again at `ButtonSize::Mini`
  (Danger with no `size` prop, its own Mini height), and asserts all three heights are equal at
  each size and that Mini is shorter than Regular. No golden's markup changed (the CSS is the
  only diff), so none moved.
- **Q111: a Small cap's arrow was still a dash.** Q95 fixed the font subset so `←` and `→` are
  drawn by Space Mono rather than falling back to a system face, but at `KbdSize::Small`'s
  9.5 px (`--fs-micro`) the arrow's own stroke is only about 4-6 px of ink: readable as an arrow
  under a loupe, still close to a dash at a glance. `Key::glyph_kind` (vocab.rs) marks Up, Down,
  Left and Right; `Kbd` writes `data-glyph="arrow"` on their cap only (every other key writes no
  `data-glyph` at all, so their markup is unchanged). `.ds-kbd[data-size=small][data-glyph=arrow]`
  draws that cap's glyph at `--fs-control` (13, up from 9.5) with `line-height:1.13` rather than
  the browser's own metric for that face: the line-box the bigger glyph asks for
  (`font-size * line-height`) comes out within a device pixel of the line-box `--fs-micro` at its
  own default line-height already gave the cap, so the cap's padding-and-border box (its height
  in a shortcut hint row) does not move, only the glyph inside it grows. The cap's *width* does
  grow a little (18 px to 20 px in the harness, an inline-block sized by its own content) — no
  existing shortcut hint aligns key caps to a fixed width, only to a common height, so this is
  not a regression the way a height change would be. Proof:
  `ds-native/tests/kbd_arrows.rs`: `a_small_caps_left_and_right_arrows_have_heads`'s width floor
  moved from 5 to 7 px (an arrow's ink is now 8 px wide in the harness, comfortably over);
  `a_small_arrow_caps_box_is_the_same_height_as_a_plain_small_cap` (new) lays out a plain Small
  letter cap beside all four Small arrow caps and asserts every arrow cap's height is within
  1 px of the letter cap's. No golden moved: none of `controls/kbd`'s existing cases (`Ctrl T`,
  `Ctrl K`, the five-modifier chord) hit an arrow key, so `data-glyph` never appears in their
  markup.

## Notification parts (2026-09-25)

sill M6's notifications (design/20 §1.6, design/13 §13.3.6) asked for six pieces, Q120-Q125, at
macOS polish. Branch `notification-parts`. Everything is additive; only the stylesheet golden
moved. Four `Anim`s join the table (`BannerOut`, `BannerIn`, `PanelIn`, `PanelOut`; `Anim::ALL`
is 64, `motion.css` holds 55 keyframes), each with its recipe, `X--b` alias, pulse class, settle
rows (284 ms at Standard) and drift entries; design/05 §4.9 records them.

- **Q120: the card.** `NotificationCard` paints its material from a transparent scope of its own
  (`Surface { chrome: Some(RootChrome::Transparent) }`; the transparent-root card rule now names
  `.ds-notification-plate` and `.ds-notification-layer`), so a Toast card sits in a Popover
  center unchanged. The close button is a sibling of the plate, not inside it, and every action
  and link keeps its press. Proof: `ds-native/tests/notification_card.rs` (after hovering, a
  press on the close button, an action and a link logs each and never `open`; a press on the
  summary or the body's text logs `open`). Goldens `notifications/card-{light,dark,group,popover-root}.html`.
  - **Line clamp.** Blitz has no `line-clamp` (the risk table), so the body is a `max-height` of
    whole `1.35em` lines (two, six on hover) with a `max-height` transition, and the fade that
    marks cut text is decided by measuring: the body reads its text's height and one line's
    (`span.ds-notification-body-line`, invisible) after layout and writes `data-clip` `rest`
    (three to six lines) or `always` (more), so a two-line body never fades. Proof: a nine-line
    body measures `always`; under the pointer its height reaches three times its resting height
    (two lines to six) and returns when the pointer leaves (`on_hover` hears `Over` and `Away`).
  - **Blitz hit-tests what a clamp hides.** `overflow:hidden` clips the paint, not the hit test:
    the clamped body's hidden lines ran under the actions row and took its presses (the action
    press opened the card), and a link on a hidden line was pressable. The body's text takes no
    pointer (`pointer-events:none`, links excepted) and the actions row is raised
    (`z-index:var(--z-raise)`); the test failed with `["close","open","link"]` before and passes
    after.
- **Q121: the stack.** `BannerStack` runs the roster with one new rule, `use_leaving_roster`: a
  key the caller stops listing leaves by an `Exit` (here `Exit::BannerOut`) instead of dropping,
  a key listed again while leaving stays (`Roster::stay`), and the rows after it heal by the
  height the leaving row measured as it started to leave (`RosterState::settled_by`), since
  banners differ in height. `on_hidden` hears the key at the exit's settle. Proof:
  `ds-native/tests/notification_stack.rs`: of three, the middle one removed is `leaving` and
  still drawn at half the exit, gone only after `settle(BannerOut)`, the one after it `healing`
  with `--dy` equal to the removed row's height within half a pixel, the one before never moves,
  and `on_hidden` hears `2`; an emptied stack reports both keys; a key listed again stays;
  goldens `notifications/stack-{top-right,bottom-right-dark}.html`.
  - **Blitz keeps the last animated value when an animation is taken off.** The entrance was
    first a rule on `data-presence=entering`. In the gallery's snapshot the roster's rest (a
    Rust timer) turned the rows `present` before any frame had been resolved past `banner-in`'s
    end, the rule stopped matching, and the rows stayed at the keyframe's first frame, off to
    the right, at 300 ms, 2 s and 10 s alike (removing the rule put them in place). The entrance
    now plays on `div.ds-banner-card`, which mounts with it and never loses it. The OSD, the
    sheet and the panel still switch `data-presence` on a settle timer; a window resolves frames
    far more often than the gap between the animation's end and its settle, so only a clock
    that does not advance with the timers (a snapshot) shows it.
  - **Context reaches a caller's element.** A card built by the caller and handed over as
    `Banner.card` is mounted under the stack's row, so it sees the stack's context (`Carried`)
    and a swiped card reports at the release while its row carries it out. Proof: the stack test
    drags a banner 100 px and finds its row `leaving` at the release and the card holding
    `--swipe-dx:100px`.
- **Q122: swipe.** `motion::swipe` is pure and table-tested (`swipe_tests.rs`: 1:1 right, a
  quarter left, springs back under 80 px and 600 px/s, flies out past either, the release speed
  from the last two moves within 100 ms, mostly-vertical scroll ignored, a drag swallowing its
  click). Proof on Blitz: `ds-native/tests/notification_swipe.rs`.
  - **Wheel.** Blitz forwards winit's `MouseWheel` delta unchanged (positive x is content moving
    right, the opposite of the web's `deltaX`), multiplies a line by 20 px only for its own
    scrolling, targets the hover node, which only a pointer move sets, and carries no scroll
    phase, so a touchpad gesture's end is a quiet spell: `DelayToken::SwipeQuiet` (120 ms).
    `Harness::wheel` moves the pointer to its point first, as a window has.
  - **No pointer capture** (as before): a drag that leaves the card is released there
    (`onpointerleave`), and a move that arrives with no button down is the release Blitz never
    delivered.
- **Q123: the center.** `Panel` rather than a `Sheet` placement: design/20 §1.6 gives the center
  the Popover material and its own surface; a sheet is a modal Sheet-material dialog on the
  layer stack. It shares the OSD's presence machine (`shown_phase::use_shown_phase`). Its scope
  fills the root (`ClassedScope`, `.ds-panel-scope`): Taffy places an absolutely positioned box
  against its parent (Q94's finding), and inside a plain `Surface` the panel was 24 px tall.
  Proof: `ds-native/tests/notification_center.rs` (at rest 384 wide, 8 in from the right edge,
  the root's height less 16; `on_hidden` only after `settle(PanelOut)`; a re-show 60 ms into the
  exit takes it back); goldens `notifications/center-{light,dark-scrim}.html`.
- **Q124: rich runs.** `RunTone::{Italic, Underline}` and `Rich`/`RichRun::Link`; a link's
  press stops at the link and prevents the document's own navigation. Existing `Text` callers
  compile unchanged (a `match` over `RunTone` needs the two arms). Golden
  `notifications/rich-runs.html`; the card test above proves the link keeps its press.
- **Q125: the group header.** `GroupHeader`; goldens `notifications/group-header-{closed,open-dark}.html`.
- **Gallery.** The Overlays page's Notifications section, light and dark over the Work tint (the
  page is 7100 tall).

## mailo gaps 7 (2026-09-25)

mailo, on v0.1.9 in its native window, reported four gaps. Branch `mailo-gaps-7`, one commit per
item; blitz rev unchanged (`e99fbdbd`). CONSUMING.md "The mailo gaps 7 (2026-09-25)" has one row
per change, docs/mailo-migration.md §2 and §6.6 what mailo deletes. Proofs: the Harness tests
`crates/ds-native/tests/mailo7_{removed_focus,restore,tree_rename}.rs`, the goldens in
`crates/ds/tests/mailo_gaps7_ssr.rs`, the lint cases in `lint_rules.rs` and the unit test in
`ds::delays`.

1. **The keyboard after its element is removed.**
   - **Found: Blitz sends the focus nowhere on removal.** `process_removed_subtree` calls
     `clear_interaction_state_for_removed_node` for every removed node, which blurs a focused one
     and sets the focus to `None` (hover and active retarget to the nearest surviving ancestor;
     focus does not). A quire `Menu` focuses its panel as it opens, so Escape (or a pick) that
     removed it left every later key going to the root, and mailo's `.app[tabindex]` heard
     nothing until its own re-focus effect ran. `FocusFallback::Ancestor` covered only a click.
   - **Found: the ancestors of a removed node cannot be walked.** `remove_node` takes the node's
     `parent` after processing the subtree, and a dropped node's id may be reused by the next node
     the same mutation batch creates. So the host remembers, while an element has the keyboard,
     its focusable ancestors (`focus_chain.rs`: each one's id and tag, so a reused id of another
     kind is not taken for it), and checks each for life (in the document, same tag,
     `is_focussable`) only when it focuses.
   - **Now:** `FocusKeeper` (`focus_keep.rs`) looks at the focus after each settled frame in the
     harness (after the renders and tasks, so a component's own focus move goes first) and before
     each window event reaches the document in the window (so a key after the removal is
     delivered to the new focus). When the element it last saw focused is gone and the focus is
     nowhere, it focuses the first live candidate among: the opener a surface registered, the
     removed element's own focusable ancestors, the element focused before it, and the element
     under the pointer when the focus moved to it. A focus that went nowhere while its element
     stayed (a blur, a click on nothing) is left alone, even if the element leaves later.
   - **The menu's panel has no focusable ancestor** (it sits in the overlay layer, outside the
     app), so the element focused before it is what makes Escape land in the app: `.app` for a
     menu opened by a key, the button for one opened by a press. **Found: the press's own focus
     is often overtaken.** The click fallback's restore and the menu's `focus_soon` are both
     tasks; when the menu's runs first the button never has the keyboard, and the keeper, which
     looks once per settled frame, would not have seen it anyway. The element under the pointer
     at the moment the focus moved is the button in that case, so it is the last candidate.
   - **A menu anchored to an element hands back to it.** A floating `Menu` with
     `Cursor::Auto` and `Anchor::Mounted(el)` registers `(panel, el)` through the new
     `ds::HostHandBack` seam in its panel's `onmounted`; the keeper puts `el` (or its nearest
     focusable ancestor) first. **Found: a hand-back task from the menu's drop loses the race.**
     The first try (a root task spawned in `use_drop`) ran while the renderer held the document
     (`Busy`), waited a frame, and by then the keeper had focused `.app`; the registration makes
     the keeper the only writer, after the removal, with nothing to race.
   - Proof (`mailo7_removed_focus.rs`): a menu opened from a quire `Button` inside `.app` and
     closed with Escape leaves the button or `.app` focused and the next `j` reaches `.app`'s
     handler; opened by `m` on `.app`, Escape gives `.app` the keyboard; anchored to the button
     and opened by `m` (the button never focused), Escape gives the button the keyboard (only the
     registration can); a field whose Enter removes it leaves `.pane[tabindex]` (its nearest
     focusable ancestor) focused and `j` reaches `.app`. Under `BlitzDefault` both stay nowhere
     and `j` is heard by no one. `launcher_gaps.rs`'s Q44 check that a field without a focus
     request stays unfocused after the actions menu closes now runs under `BlitzDefault`; under
     the default the field is focused again (it had the keyboard before the menu), which the same
     test now asserts.
2. **A click's restore focused an element the click removed.** mailo's "Show images" button
   removes itself on press. The click fallback had found the button itself (a `button` is
   focusable) and its restore focused that node a frame later, after it had left the document.
   **Now** the fallback hands `restore` a `ChainNode`: every focusable element from the found one
   up, remembered at the click; `restore` checks liveness at the moment it focuses and takes the
   first live one (`.app`). A handle `restore` is given directly (`NodeHandle`, `FoundNode`) is
   walked from when it runs, if it is still in the document. Proof (`mailo7_restore.rs`): the
   self-removing button leaves `.app` focused and `j` reaches it (it fails on master: the focus
   stays nowhere); a button that stays keeps the keyboard.
3. **A tree row renamed in place.** `TreeItem { editing: Option<Element> }` draws the caller's
   field in the label's place, in `span.ds-tree-item-label.ds-tree-item-edit[data-slot=editing]`:
   the label's class keeps its flex share, so the slot starts where the label did and is as wide,
   and a Bare `TextInput` takes the row's face. The slot stops and prevents its clicks, as the
   trailing slot does. **Preventing the click is safe for a field on Blitz:** the caret is placed
   and the field focused on pointer-down (`handle_pointerdown`); the click's default only
   "matches" the text input and does nothing more, and without the fence the summary's own
   handler toggled the folder (`toggle:Closed`, checked by removing it). Keys are left alone:
   they start at the field, so its `onkey` hears Enter and Escape first. **The trailing seam:**
   `data-slot="trailing"` on the trailing slot; `CONSUMER_SEAMS` in `lint/selector.rs` names
   `data-slot` as a consumer-styleable attribute (it was never internal, the constant keeps it so),
   and `DataName::parse` now refuses `slot`, so a consumer's button cannot pose as the slot.
   Proof: goldens `lists/tree_item/editing-{branch,leaf}` (the gaps 6 tree goldens gain the
   attribute); lint cases (`[*|data-slot=trailing] .fold-more` clean, `.ds-tree-item-trail
   .fold-more` `DsInternals`); `mailo7_tree_rename.rs`: the slot's left edge and width equal the
   label's, the row's height, the count and the next row are unmoved; the field has the keyboard
   with `Projects` selected; a press and typing in it log no toggle and no select, the folder
   stays open and the value is typed; Enter and Escape reach the field's handler and end the
   rename.
4. **Hover delays for tests.** `ds::delays::{HOVER_OPEN, HOVER_CLOSE, HOVER_WARM}` are the
   token values (450, 150, 400 ms) as `const Duration`s; a unit test holds each equal to its
   `DelayToken` at every `MotionLevel`.

## PDF output (2026-09-25)

mailo is going native-only, and printing was its last webview use. Branch `native-pdf`; blitz rev
unchanged (`e99fbdbd`). The route is the one mailo's research prototyped: Blitz lays the document
out, an anyrender backend writes the pages, and the result is a vector PDF. The prototype wrote
through krilla. At the user's direction the writer is now **pdfrum** (the user's own pure-Rust
PDF library), and what pdfrum lacked was added to pdfrum (branch `blitz-print`, "pdfrum
additions" below). krilla and pdf-writer are out of the tree. The subsetter stays: it is
pdfrum-edit's own.

### What was built

- **`crates/anyrender_pdfrum`**, a sibling crate rather than a module of ds-native. It names
  only anyrender, peniko, pdfrum and skrifa; the boundary script enforces "no Blitz, no parley",
  so it can be offered to DioxusLabs/anyrender as it is.
  - **Each page is painted into anyrender's recording `Scene` first.** `write(&[Page], &Sources)`
    then embeds every face and image the scenes use, and replays each scene onto a pdfrum
    `Canvas`.
  - The recording is what makes pdfrum's canvas fit. anyrender's layers are a push/pop stack,
    while `Canvas::saved` takes a closure, so an unbalanced `q`/`Q` cannot be written. A pushed
    layer's commands, up to its pop, replay inside one saved state.
  - It covers fills, strokes (in the shape's space, so widths scale), clip/blend/opacity layers,
    linear and radial gradients (with stop alpha), images, and glyph runs.
- **`ds_native::{pdf, pdf_app}`, `Harness::pdf`**, `PageSpec { size: PageSize::{A4, Letter,
  Custom { width, height }}, margins: Margins }` in `Pt`, `PdfError`. Code: `crates/ds-native/
  src/pdf/` (`html`, `flow`, `paginate`, `run_texts`, `images`, `pages`, `spec`).
  - `pdf(html)` builds an `HtmlDocument` as a snapshot's document is built: quire's shared font
    context, the HTML parser, sequential styling. Its `NetPolicy` is `Sealed` (only `data:` loads)
    and its `MediaType` is `print()`.
  - **Blitz honours `@media print`**: the fixture's screen-only paragraph is absent.
  - The document is laid out once, at the content box's width, at scale 1 (a CSS px is a layout
    pixel; the research found that re-laying out at 300 dpi reflowed the text).
  - `Harness::pdf` switches the live document to the page viewport and print media, prints, then
    restores both.
  - `pdf_app` is a harness at the page viewport, advanced by the snapshot's mount settle, then
    `Harness::pdf`.
- **Pages.** The document is paginated once. Each page is `paint_scene` into a `Scene` with the
  viewport scrolled to the page's top. Its placement maps CSS px, y down, to PDF points, y up,
  with the margins, and its clip is the page's band.
  - **Found: the clip ends half a CSS pixel above the cut.** Poppler at 96 dpi snaps a clip outward
    to whole device pixels. It showed a full-width grey row at page 1's foot: the top border of
    the block moved to page 2, which begins exactly at the cut.
- **`print_dialog(pdf, title)`**, feature `print` (zbus and memfd, Linux only; the boundary
  script checks that ds-native's default build reaches neither).
  - The flow: `PreparePrint`, then `Print` with a sealed, rewound memfd and the dialog's `token`.
    Each `Response` is subscribed on the request path from `handle_token` before the call is made.
  - Checked against the live portal's introspection: `PreparePrint ssa{sv}a{sv}a{sv}`,
    `Print ssha{sv}`, version 4.
  - Falls back to a temp file plus `xdg-open` / `open` / `start` when there is no session bus, no
    portal or no Print backend.
  - It blocks until the dialog is answered.
  - The dialog is unparented. The portal wants `wayland:<xdg-foreign handle>`, and neither winit
    nor `launch` exports one.
  - xdg-desktop-portal-gtk issue #562 (a second PreparePrint + Print in one backend process can
    lose the fd) is not detected.
  - Not run by a test, because it would open a real dialog. `--example print` runs it by hand.

### pdfrum additions (branch `blitz-print`, for pdfrum's CHANGELOG [Unreleased] "Added")

- `pdfrum_edit::blank_document(&[Size])`: a new file of blank pages, to draw on and save. Before
  this, `EditDoc` could only edit a parsed file.
- `EditDoc::embed_glyph_font(program: ByteSpan, face, FontInstance)` returns a `GlyphFont`, drawn
  with `Canvas::glyphs(&GlyphRun { font, size, transform, glyphs: &[RunGlyph { id, x, y, text }],
  text, paint })`.
  - Glyphs are drawn by id, and a CID is given at first use.
  - The content is `TJ` with exact adjustments: each glyph's advance is the one `/W` carries.
  - Text goes into `/ToUnicode` per glyph, taken from its cluster's text, so a ligature maps to
    all its letters. A cluster of several glyphs, or a glyph already mapped to other text, gets
    `/ActualText`.
  - Collections (`.ttc`) are picked by face index.
  - When a drawing call returns, the face is subset to the drawn glyphs in CID order, so new GID
    equals CID and no `/CIDToGIDMap` is needed. It is written with `/W` and `/ToUnicode`.
- `FontInstance::{Default, Normalized(Vec<i16>), User(Vec<AxisValue>)}`. Normalized coordinates
  are mapped back through `avar` and `fvar` to user values for the subsetter.
- Feature `variable-fonts` on pdfrum-edit (`subsetter/variable-fonts`) instances variable faces
  and CFF2 (converted to TrueType outlines). It is off by default, because it pulls the
  subsetter's own skrifa and write-fonts. Without it, such a face is `Error::VariableFontsDisabled`.
- `Canvas::fill_gradient(shape, rule, &Gradient { kind: GradientKind::{Linear, Radial}, stops },
  transform)`: axial and radial shadings, stitched functions.
  - Stops of one alpha use a constant opacity.
  - Varying alpha uses a luminosity soft mask of the same geometry in grey, so a fade goes to the
    page, not to black.
- `Canvas::blend(BlendMode)` writes `/BM`.
- `EditDoc::embed_png(bytes)` passes through an opaque, non-interlaced greyscale, RGB or palette
  PNG's `IDAT` as-is (`/FlateDecode` with the PNG predictors). A PNG with alpha, `tRNS`, interlace
  or 16-bit is `Error::PngNeedsDecoding`; decode it and use `embed_image`.
- `ByteSpan::from_owner(impl AsRef<[u8]>)` (pdfrum-object) shares any owner's buffer. A 32 MB CJK
  collection held by the renderer is named without a copy.
- New errors: `BadPageSize`, `BlankDocument`, `TooManyGlyphs`, `VariableFontsDisabled`,
  `ForeignGlyphFont`, `PngNeedsDecoding` (`Error` is `#[non_exhaustive]`).
- Tests (pdfrum-edit `tests/glyph_runs.rs`, `tests/canvas_paint.rs`), each read back with pdfrum's
  reader and rasterised with `pdfrum-raster-vello-cpu`:
  - text, a ligature, a glyph shared by two texts, a multi-glyph cluster;
  - exact origins, a tagged subset, ink where the run is, a collection face;
  - with the feature: 400 and 700 instances differ, and normalized coordinates equal user values;
  - linear, fading and radial gradients, clipping, multiply;
  - PNG passthrough (the `IDAT` bytes in the file once), and alpha refused.
  - Fixture: Karla (OFL-1.1, with its licence file).
- **pdfrum asks** (still missing; the painter simplifies them, listed in the crate docs):
  - sweep (conic) gradients, which PDF can only spell as a type 4 function or a mesh;
  - gradient extend `Repeat`/`Reflect` (shadings only pad);
  - gradient paint on strokes and on text (only fills take a shading);
  - blur (box shadows) and filters;
  - compositing operators beyond source-over.

### Page-break rules

stylo is built for servo here, and marks `break-before`, `break-inside` and `page-break-*` as
gecko-only; `@page` is ignored. So pagination reads markers. It is one pure function,
`paginate(&Flow, page_height)`, table-tested.

- A **forced break** at `data-break-before="page"` ends the page at the element's border-box top.
  A marker at the document's top makes no blank page.
- Otherwise a page ends where it is full, **moved up past every keep-together span** the cut
  would split, until it splits none. The spans are:
  - each line box (parley `LineMetrics::block_min/max_coord`);
  - `img`, `svg`, `canvas`, `video`, `iframe` and `tr` border boxes;
  - `data-break-inside="avoid"` boxes;
  - each text block's first two and last two lines: **orphans and widows of 2**, CSS's initial
    values, as spans; a 3-line block therefore moves whole.
- A span **taller than a page** is not kept. It is cut where the page is full, and its own lines
  still move whole.
- A span starting at the page top is never moved, so every page makes progress.
- **Margins** come from `PageSpec` only. `Margins::default()` is 18 mm / 16 mm, mailo's `@page`.
- **Glyphs outside the page's band are dropped, never left under the clip.** `GlyphArea::
  Within(rect)`: a glyph is written only if the centre of its box (its advance across, ascent to
  descent down) is inside, so each glyph belongs to exactly one of two pages sharing an edge.
  - The test drives this: with `GlyphArea::Anywhere`, two tests fail. The next message's heading
    turns up in page 2's text layer, and glyphs sit outside the content box.

### The text behind each glyph (task 4): reachable, and used

- **blitz-paint hands anyrender glyph ids and positions only.** `stroke_text` calls `draw_glyphs`
  with `glyph_run.positioned_glyphs()`.
- **But the layouts are public, and parley 0.11 keeps each cluster's text.** `inline_layout_data
  { text, layout }`, a list item's outside marker layout (its text is the marker) and a text
  input's `editor.raw_text()` / `try_layout()` are all reachable. A parley `GlyphRun`'s glyphs
  are its run's `visual_clusters()` glyphs, in order, and each `Cluster` has `text_range()`.
- **So before painting, ds-native walks every layout** (`pdf/run_texts.rs`).
  - It pairs each glyph run's glyphs with their clusters' text, and keys the run as the scene
    records it: `RunKey` = the face's blob id and index, the size's bits, and each glyph's id and
    x/y bits.
  - **A ligature's continuation clusters** (zero glyphs, `is_ligature_continuation`) join the
    glyph's cluster, so `fi` drawn as one glyph carries `fi`.
  - pdfrum writes the cluster text into `/ToUnicode`, or `/ActualText` where a glyph's mapping
    would conflict (above).
- **Proofs** (`crates/ds-native/src/pdf/text_tests.rs`, read back with pdfrum):
  - Noto Serif `field office flat` extracts exactly. Every glyph's text came from the layout
    (`GlyphTally::cmap_text == 0`).
  - `一⼀一` in Noto Sans CJK TC extracts exactly, though 一 and ⼀ share one glyph (the second run
    carries `/ActualText`).
- **The fallback** is for runs with no entry: SVG text (usvg outlines it), a custom widget's
  scene, anything painted outside those three layout kinds. It is the face's cmap read backwards.
  - Where code points share a glyph, it **prefers the canonical code point**: not a CJK radical
    (U+2E80-2FDF), a compatibility ideograph, a presentation form (U+FB00-FDFF, FE30-FE4F,
    FE70-FEFF, FF00-FFEF) or private use; lowest code point otherwise.
  - `the_cmap_fallback_cannot_spell_a_ligature` shows its limit: the same ligatures printed with
    no run texts read back as `"\u{1}eld o\u{7}ce at"`. The `fi`, `ffi` and `fl` glyphs have no
    cmap entry, so they carry no text, and pdfrum shows their raw CIDs.
  - `the_cmap_fallback_prefers_the_ideograph_to_the_radical` reads `一⼀一` back as `一一一`.
  - Arabic and Indic contextual forms would be wrong the same way.
- **Found and fixed: a textless glyph swallowed the next glyph's text.** When a page's glyphs were
  selected, clusters were identified by their start byte. A textless glyph's empty range starts
  where the next glyph's does, so the two merged, and the `c` after `ffi` lost its text too.
  Clusters are now identified by equal ranges (`a_glyph_without_text_does_not_swallow_the_next`).
- **Limits of the layout path:**
  - Two runs with the same key but different text take the first recorded. Only glyph-sharing
    code points can cause that, with everything else identical.
  - Right-to-left runs are drawn in visual order, with their clusters' logical text per glyph.
    This is not tested.
  - The key includes the blob id, so the renderer must paint the very `FontData` the layout
    holds. Blitz does.
- **Upstream**, the clean fix is still a text-and-clusters argument on anyrender's `draw_glyphs`.
  The pre-walk replicates `GlyphRunIter`'s split of a run into glyph runs, so a parley change
  there would show as runs falling back to the cmap (`GlyphTally::cmap_text` rising). Re-check it
  in the toolchain-bump wave.

### Variable fonts (task 5): the layout's instance is embedded

- The painter passes the run's normalized coordinates to `embed_glyph_font` as
  `FontInstance::Normalized`. pdfrum maps them to user values and the subsetter instances at
  them, so outlines and the `/W` advances are the layout's.
- Karla at 400 and at 700 embeds two different subsets (`native_pdf_app.rs`).
- **Found: the prototype's "thin, grey" CJK was the default instance, not a fallback choice.**
  Noto Sans CJK on this machine is one variable CFF2 collection (`NotoSansCJK-VF.ttc`, 32 MB)
  whose default instance is Thin. The prototype ignored the coordinates and embedded Thin. With
  the instance honoured, the CJK prints at 400, as on screen.
  - It is shared into pdfrum without a copy (`ByteSpan::from_owner`) and subset to about 20 KB.
- **The embedded subset is named after its instance** (pdfrum `02a8dcb6`). A variable face's
  subset used to keep the default instance's PostScript name, which named the default instance:
  `pdffonts` listed `NotoSansCJKtc-Thin` for glyphs printed at 400, and mailo's viewers showed
  it.
  - `/BaseFont` and `/FontName` now take the named `fvar` instance's own name
    (`NotoSansCJKtc-Regular`, `Karla-Bold`).
  - An unnamed instance follows the OpenType recommendation (`Karla_550wght`).
  - Proofs: `native_pdf_app.rs::each_instance_is_named_after_its_weight` (the Karla subsets are
    exactly `Karla-Bold` and `Karla-Regular`) and the fixture's CJK subset ending in
    `NotoSansCJKtc-Regular` (`native_pdf.rs`). Both fail on the previous rev
    (`NotoSansCJKtc-Thin`).

### Images

- Blitz keeps only decoded RGBA. ds-native re-decodes each `<img>`'s `src`, and each background
  image's URL, when it is a `data:` URL, and records the encoded bytes against the decoded
  blob's id.
- A **JPEG is written as-is** (`embed_jpeg`, DCTDecode). The fixture's 10,391-byte JPEG is in the
  PDF byte for byte, once.
- An **opaque PNG is written as-is** (`embed_png`).
- A PNG with alpha is drawn from its decoded pixels (`embed_image`, Flate, alpha as SMask). The
  fixture's 120 x 40 alpha PNG costs 335 bytes.
- Images an app's `AppNet` fetched are also drawn from their decoded pixels.
- An image brush draws once, clipped to its shape; Blitz tiles backgrounds itself.

### Simplified

- Box shadows are dropped.
- Filters and backdrop filters are ignored.
- Compositing operators other than source-over paint as source-over. The blend (mix) modes map
  one to one.
- A sweep gradient, and a gradient on a stroke or on text, paints as its stops' average colour.
- Gradients pad at their ends and interpolate in sRGB.
- Synthetic bold is text render mode 2 (fill then stroke), so it stays text. Synthetic oblique is
  one skew about the baseline.

### Measured (this machine)

The fixture (`tests/support/print_fixture.rs`) has an A4 heading in Karla and Latin and CJK
paragraphs. It also has a 320 x 200 JPEG and a 120 x 40 alpha PNG as `data:` URLs, a
keep-together block that must move, and a forced break.

- **Pages:** 3. The block starts page 2; the forced break starts page 3.
- **Size: 42,118 bytes** (the krilla build was 47,338). Four subset faces: Karla, Noto Serif,
  Noto Sans CJK TC and DroidSansFallback.
- **Time, release** (`cargo run --release -p ds-native --example pdf`):
  - 21-43 ms for the first print in a process, which builds the shared font context and scans
    the system fonts;
  - 5.8-6.9 ms for a second print.
- **Time, debug test build:** 67 ms for a warm print (`the_fixture_is_small_and_quick`, bound
  300 KB and 500 ms).
- **Visual check** (`native_pdf_raster.rs`): the printed page, rasterised by pdfrum at 96 dpi,
  against the headless snapshot of the same app. The mean channel difference is 0.52/255 and
  0.13% of pixels differ by more than 64 (bounds 3.0 and 2%). Anti-aliasing is the only
  difference; the layout is identical.

### Limits

- **Pagination is quire's heuristic, not CSS fragmentation.**
  - A box's background and border split at a cut, as a browser's do.
  - A single line or image taller than a page is cut.
  - Table headers do not repeat.
  - Floats and absolutely positioned boxes are not kept.
  - Orphans and widows are fixed at 2; the CSS properties are not read.
  - `break-after` is not supported.
  - There are no running headers, footers or page numbers.
- **Font fallback is fontique's.** The fixture's Karla heading got its CJK from
  DroidSansFallback. Print CSS should name the CJK families.
- **CJK line breaking** still prints `ICU4X data error: No segmentation model` at this rev.
- **The dialog**: unparented on Wayland; issue #562 undetected; COSMIC has no Print backend of its
  own, so a system without xdg-desktop-portal-gtk falls back to the viewer.
- **Deviation from the brief:** `Harness::pdf` returns `Result<Vec<u8>, PdfError>`, not
  `Vec<u8>`. A face pdfrum cannot read fails the write, and CONVENTIONS §5 forbids a panic on
  input.
- **Pinned block** (`docs/workspace-deps.toml`, copied into `Cargo.toml`): `pdfrum-edit` (with
  `variable-fonts`), `pdfrum-object`, `pdfrum-common`, `pdfrum` (tests, `vello-cpu`), `skrifa =
  0.44` (parley's own) and `memfd = 0.6`.
  - They are a git rev of pdfrum `main` (`02a8dcb6`) until pdfrum 0.4 is on crates.io.
  - pdfrum's crates declare `rust-version = "1.92"`, so quire's workspace `rust-version` is
    1.92 now (was 1.91, blitz's minimum). The pinned toolchain stays 1.98.1.
  - Resolving pdfrum `main` moved `smallvec` back up to 1.16.1 (pdfrum pins it exactly).
  - `cargo deny check licenses`: ok.

## Hover under a resting pointer (2026-09-26)

sill Q170: a notification banner that arrived under a resting pointer never counted as hovered
(no hold pause, no close button) until the pointer left it and came back.

- **Cause (Blitz @ e99fbdbd, not patched).** `BaseDocument::resolve` ends with `refresh_hover`
  (`blitz-dom/src/resolve.rs:135`, `document.rs:1925`). It hit-tests the last pointer position
  against the fresh layout and stores the result as the hovered node, dispatching nothing; a TODO
  there says the enter/leave synthesis is missing. The event driver
  (`events/driver.rs::handle_pointer_move`) diffs the next move against that stored node. The
  diff comes out empty, so the element that slid in never hears `pointerover`/`pointerenter`,
  and the one the pointer left never hears `pointerout`/`pointerleave`.
- **Recipe (host level, `crates/ds-native/src/hover_sync.rs` decides,
  `hover_replay.rs` acts, called from `Headless::resolve`).**
  1. Read `get_hover_node_id()` before and after each `resolve`. If they are equal, or no
     pointer event has arrived yet, do nothing.
  2. Otherwise put Blitz's hover back on the old element without events. Probe
     `hit()` at the old element's current rect (centre, then the corners inset 1 px), mapping
     each hit through `nearest_non_anonymous_ancestor` (the mapping Blitz applies before it stores
     a hovered node). The first probe that hits the old element goes to `set_hover_to`. If none
     does (the element left the tree or is covered), call `clear_hover()`.
  3. Replay the last pointer event as a `UiEvent::PointerMove` (same position, buttons and
     modifiers) through `handle_ui_event`. The driver now sees the real diff and dispatches
     leave/out for the old chain, over/enter for the new one, and one `pointermove` (a browser's
     "fake mouse move" after a layout shift does the same).
  4. The frame loop goes round again. The replay left Blitz's hover where the next
     `refresh_hover` finds it, so that pass changes nothing. At most one replay per round, and
     `MAX_ROUNDS` bounds the rest.
- **Proof.** `crates/ds-native/tests/hover_sync.rs`. A block slides under a resting pointer: it
  is entered once, the field it covers is left once, and the page they share is not entered
  again. It is entered before the pointer moves at all. With no pointer yet, nothing is heard. A
  `BannerStack` banner that lands under the resting pointer has its `NotificationCard` at
  `data-hover="on"` after a 1 px move. Three of these fail with the replay switched off.
- **Limitation: the covered case.** When the old element is covered everywhere probed, hover is
  cleared rather than restored, because Blitz can only be told to hover a point. The replay then
  loses the old element's `pointerleave` and re-enters every shared ancestor.
  `a_block_covering_the_whole_field_is_still_entered` pins this.
- **Limitation: coverage.** Only the `Harness` (and `snapshot`) is covered, plus any host that
  calls `resolve` itself. `ds_native::launch` is not: it opens a blitz-shell window whose frame
  loop Blitz owns, and ds-native never sees its resolve. shell-host gets the same recipe on its
  own branch.

## Scroll physics wave 1: quire's row (2026-09-26)

design/11-BEHAVIOUR-scroll.md section 11.7's row for quire: `scrollbar-width: none`,
`scroll-behavior: smooth` banned, `--scroll-thumb`, `data-wheel="capture"`,
`data-overscroll="band"`. shell-host's own engine is a separate branch; nothing here runs it.

- **`scrollbar-width: none` is honoured by Blitz, and cheaply.** `blitz-dom`
  (`e99fbdbd`, `packages/blitz-dom/src/node/scrollbar.rs`) grew its own overlay-scrollbar
  painting (`Node::wants_scrollbar`, `Node::scrollbar_thumb`) since the plan's own spike table
  was written — a synthetic thumb Blitz paints itself, Chromium-style geometry (10 px thick, 32 px
  minimum thumb length, a 500 ms fade delay and a 200 ms fade). `wants_scrollbar` reads the
  computed `scrollbar-width` first and returns `false` immediately when it is `None`, before it
  even asks whether the axis overflows — so `scrollbar-width: none` suppresses Blitz's thumb
  outright, not merely its width, and does so independently of `overflow: auto` vs `scroll`. Added
  to every ds scroll container's own rule (`.ds-menu`, `.ds-panel`, `.ds-sheet`,
  `.ds-input[*|data-kind=multiline]`) rather than as a blanket reset rule, since `reset.css`
  declares no scroll container of its own.
- **`scroll-behavior: smooth` joins `Rule::BlitzUnsupported`'s table**
  (`crates/ds/src/lint/blitz.rs`), rather than a new `Rule` variant: the existing rule is already
  a per-property/value table (`position: sticky` is the precedent — one property, banned only for
  one value), so a new row reuses the offence path, the exception mechanism and the doc comment
  wholesale. `scroll-behavior: auto` (the default) lints clean.
- **`--scroll-thumb`** (`ds::tokens::ColourToken::ScrollThumb`): `--ink` at .5 alpha over the
  layer's own .8 opacity (effective .4, `alpha(0x000000, 400)`) in light; `--paper`'s equivalent,
  white at the same effective alpha (`alpha(0xFFFFFF, 400)`), in dark — the same shape as
  `HandleRing`'s black-in-light/white-in-dark pair, one alpha rather than two literal RGB pairs.
  Registered the same way every other colour token is (`lint::registry::declared_vars` derives
  its known-variable list from `ColourToken::ALL`, so nothing else needed touching). No
  `--scroll-thumb-hover`: design/11 section 11.3.12 only changes the *track*'s fill on hover
  (`--line-soft`, already a token) and the thumb's thickness, never its colour.
- **`data-wheel="capture"`: only `Slider` gets it today.** design/11 section 11.3.1 item 2 names
  "sliders, zoomable canvases" as the exception; of quire's own components, `EditSurface`
  (`.ds-edit`) sets no `overflow` at all (it grows with its content, per FINDINGS "Edit surface"),
  and `LevelControl`'s capsule (`.ds-level`) has no `onwheel` handler — neither "scrolls
  internally" or "takes wheel" today, so the brief's own conditions (`only if it scrolls
  internally`, `if it takes wheel`) are unmet and both are left unmarked. Only `Slider` carries the
  attribute; `every_slider_carries_the_wheel_capture_marker`
  (`crates/ds/tests/components_controls.rs`) pins it across every slider case, not just the
  default one.
- **`data-overscroll="band"`, scope.** Of quire's own ds components with a real scroll container
  (`overflow-y: auto`/`overflow: auto`), only two are not already excluded by design/11's own
  R14 list (`Menu` is a menu; multiline `TextInput` is a text field): `Panel` (the notification
  center's edge panel) and `Sheet` (its own doc comment: "the general modal panel for settings and
  dialogs"). Both now carry the attribute. `Peek`'s own box (`.ds-peek`) is `overflow: hidden` and
  never scrolls itself — its markup comment literally says `…reader…` where the consumer's content
  goes (design/04-COMPONENTS.md section 24), confirming "the reader" in the brief is mailo's
  `.reader-body` (design/01-LAYOUT.md section 7), not a ds component; the same is true of "list
  rows' scroller" (mailo's `.list`, design/01-LAYOUT.md section 5). Both are documented in
  design/11 section 11.3.7's new table and in `CONSUMING.md` section 12 as guidance for the
  consumer that owns them, since there is no ds element to mark.
- **sill has no `scroll-behavior` use.** `grep -rn "scroll-behavior" ~/sill/crates` returns
  nothing, so the new `Rule::BlitzUnsupported` row cannot regress sill's Strict lint (it reads
  quire by path — CONVENTIONS "Warn before lint rules land").
- **Unproven.** Everything above is quire's markup/CSS/lint/token row only: no `onwheel` handler
  exists on `Slider` yet (the attribute is a marker for a host that is not built in this branch),
  and whether `data-overscroll`/`data-wheel` are read correctly is shell-host's test to write, not
  this one's.

## Inter, the system typeface (2026-09-26)

The user's decision: "make this desktop use mostly inter". `appearance.typeface`
(`Typeface::{System, Editorial}`, default System) on `Ds` and in `AppearanceSettings`; design/02
section 2.

- **Faces.** Inter 4.1 from the official release (zip SHA-256
  `9883fdd4a49d4fb66bd8177ba6625ef9a64aa45899767dde3d36aa425756b11e`, recorded in
  `crates/ds/scripts/cut-inter.sh`), OFL 1.1 (`assets/fonts/OFL-inter.txt`). Blitz sets no
  optical size from the font size (no `font-optical-sizing`; `font-variation-settings` would also
  have reached Bricolage's own `opsz`), so the variable font's `opsz` is pinned into two families
  by `fonttools varLib.instancer`: Inter (opsz 14, wght 400..700, italic 400 static) and Inter
  Display (opsz 32, wght 500..800), then subset by the same ranges and `subset-fonts.sh` as the
  others. parley applies `font-variant-numeric` (blitz `stylo_to_parley::font_variant_numeric`),
  so `tnum` works: the layout features kept are `kern mark mkmk ccmp locl calt case tnum pnum
  zero`; the `ss`/`cv` sets and fraction/super forms were 44% of each file and nothing asks for
  them. Growth: +608,140 bytes of TTF in the binary (1,346,348 before), +243,040 bytes of WOFF2
  and licence in the repository only (the `webview-fonts` path).
- **Tabular Inter changes more than digits.** Inter's `tnum` also gives the hyphen and colon
  tabular widths, so `2026-09-24` under `.ds-mono` reads a little open. Accepted: it keeps times
  and dates aligned, which is the data face's job.
- **Editorial is exact.** Rendered at 2x, the Controls, Lock and Widget reference pages under
  Editorial are pixel-identical to master below the toolbar (the gallery gained a Type tool).
  A nested `Ds` takes its enclosing root's typeface (`typeface: None`); the lock page's own roots
  first rendered in Inter under Editorial until that was so.
- **Layout.** No quire test pinning a pixel moved (month grid compact still 126 x 132 in its
  140 x 140 box). Text widths: UI text +6 to +8% (Karla to Inter), data text -14 to -16% (Space
  Mono to Inter tabular), the eyebrow -25% wide and -14% tall (11 px mono to 9.5 px caps), the
  widget clock digits +11%. Line boxes do not move where a `line-height` is set (almost
  everywhere); `.ds-mono` grows 18.1 to 20.0 px tall (.78em to .86em).

## Colour emoji (2026-09-26)

Probe for sill's M9 emoji grid: does Blitz at the pinned rev (parley, skrifa, glifo 0.2,
vello_cpu 0.1, vello_hybrid 0.1) paint colour emoji? `examples/emoji_probe.rs` lays five emoji
(😀 🎉 👍🏽 🇹🇼 👨‍👩‍👧) out in rows through Blitz on the headless path and counts, per 88 x 72 cell,
inked pixels and coloured ones (channel spread over 60); `examples/emoji_probe_hybrid.rs` draws
😀 and 🎉 as glyph runs through anyrender (what Blitz's text painting calls) once on vello_cpu
and once on vello_hybrid on the GPU (RTX 5070 Ti), read back, measured the same way.
`tests/colour_emoji.rs` keeps the COLRv1 result as a regression test (skipped where the font is
not installed).

| Face (48 px) | vello_cpu via Blitz | vello_cpu glyph run | vello_hybrid glyph run |
| --- | --- | --- | --- |
| Noto Color Emoji **COLRv1** (installed, `Noto-COLRv1.ttf`), named in the stack | colour: 😀 2016 inked / 1731 coloured, 🎉 1439/1322, 👍🏽 1417/1216 (skin tone applied), 🇹🇼 2143/2088 (one flag glyph), 👨‍👩‍👧 2776/0 (one ZWJ glyph; Noto's family is drawn in greys) | colour, 2010/1733, 1438/1323 | colour, 2010/1734, 1435/1323 (same as the CPU to within anti-aliasing) |
| Noto Color Emoji **CBDT** (googlefonts/noto-emoji `2D/fonts/NotoColorEmoji.ttf`, OFL, fetched for the probe, not committed) | nothing: 0 inked in every cell | nothing | nothing |
| CBDT with glifo's `png` feature turned on (tried, reverted) | colour, as COLRv1 | colour | **panics**: `pixmap image sources are not supported by Vello Hybrid` (vello_hybrid 0.1 `render/wgpu.rs:694`) |
| No emoji family named (system fallback) | monochrome, and wrong: fontique falls back to Symbola (`fc-match` agrees), so 👍🏽 is a thumb plus a tofu box, 🇹🇼 is two boxed letters, the family three separate faces | | |
| Noto Emoji (monochrome), named | monochrome outlines, 0 coloured | | |

- **COLRv1 paints in colour on both backends.** glifo interprets COLRv1 paint graphs (layers,
  gradients, clips) on either renderer; parley shapes skin-tone modifiers, flags and ZWJ
  sequences to single glyphs. Nothing in quire needs changing.
- **The fallback is the catch.** With only `Inter, sans-serif` the emoji come from Symbola. The
  stack must name `'Noto Color Emoji'`, and **after** the text face: with the emoji face first
  (`'Noto Color Emoji', 'Inter'`), "Ab 2#" drew "Ab" and nothing for "2#", because Noto Color
  Emoji maps the digits, `#` and `*` to the empty bases of keycap sequences and the first face
  that maps a character wins. quire now carries it: `--font-emoji` (`"Inter","Noto Color
  Emoji",system-ui,sans-serif`) and the utility `.ds-emoji-text` apply it
  (`tests/colour_emoji.rs` asserts the emoji paints colour and the text is Inter's, pixel for
  pixel), and the System display, UI and data stacks and Editorial's display and UI stacks name
  the emoji face after their text faces. Before that change and after it, the gallery's contact
  sheet (76 pictures per typeface) was compared pixel for pixel: the only differences were the
  widget pages' battery rings and one 4 x 7 px spot on the dark Editorial lists page, and a second
  run of the unchanged build differs from the first in exactly those places, so no glyph moved.
  **Reverted 2026-09-27 (sill Q342):** under sill on vello the stacks with the emoji face drew
  digits and spaces from Noto Color Emoji (wide keycap bases): "m 9 -extra- 1 .pdf", "0 0 : 2 2",
  huge word gaps in the launcher, the bar clock and every sill capture. The gallery comparison
  did not catch it. The text stacks are back to Inter / system-ui; `--font-emoji` and
  `.ds-emoji-text` stay for emoji-only content (the emoji grid, the preview glyph).
  **Root cause (2026-09-27): Inter is not a family in sill's documents at all — a shell-host
  bug.** shell-host gives every surface a clone of `SharedFonts::system()` (`dom/fonts.rs`):
  `FontContext::new()` (system fonts) plus Blitz's bullet face. `SharedFonts::register`, which
  would add quire's faces, has no caller anywhere in shell-host or sill, and Inter is not
  installed on the system (`fc-list` has no Inter), so `family_by_name("Inter")` is `None` there
  while quire's `ds_native::font_context()` (gallery, harness, `launch`) registers `ds::FACES`.
  The stack's first family that exists under shell-host is therefore Noto Color Emoji, and
  parley takes each cluster from the first family in the stack that maps it: the emoji face's
  cmap holds U+0020, 0-9, `#` and `*` (the keycap bases), so spaces and digits came from it and
  every other character from system-ui. It is not a fallback-order quirk: with Inter registered
  the same stack lays out exactly as `"Inter"` alone. `tests/text_stack_fonts.rs` lays the stack
  out through Blitz under both contexts (16 px): "00:22" is 45 px under quire's (= Inter's) and
  84 under shell-host's against 41 for system-ui; "m9-extra-1.pdf" 114 / 132 / 108; "a b c" 37 /
  65 / 32; and a stack naming Inter under shell-host is system-ui's to the pixel. **So sill has
  been drawing system-ui (Noto Sans here) wherever it asked for Inter, Inter Display, Bricolage
  Grotesque, Karla, Space Mono or Noto Serif**, and every sill capture's type is not quire's
  (shell-host's headless `Fonts::Bundled` registers one file, the first `.ttf` in quire's font
  folder, Bricolage's latin-ext, so it has no Inter either). The fix is shell-host's:
  `SharedFonts::system()` registers `ds::FACES` (`ds_native::register_fonts(&mut ctx)` does
  exactly that). The safe way to colour emoji in ordinary text afterwards is the same stack,
  `"Inter","Noto Color Emoji",system-ui,sans-serif`, *only once the text face is registered in
  every host*: the text face must precede the emoji face (so it claims the space and the digits)
  and the emoji face must precede `system-ui` (which maps emoji itself, from Symbola, so a face
  named after it is never reached; `tests/text_stack_fonts.rs` checks that too). fontique has no
  `unicode-range`, so a range-limited registration would mean shipping a copy of the emoji font
  with its ASCII cmap entries removed under another family name; with the host fixed it is not
  needed. Until shell-host registers the faces, never put the emoji face in a stack that carries
  ordinary text.
- **CBDT does not paint, and must not be made to.** glifo decodes CBDT's PNG strikes only with
  its `png` feature, which nothing in our tree enables (vello_cpu's default does, but
  anyrender_vello_cpu turns defaults off). Turning it on fixes vello_cpu and crashes the window
  renderer on the first bitmap glyph, so a CBDT font installed on a user's machine is harmless
  today only because it draws nothing.
- **Not measured.** vello_hybrid's `glyph_run` has the glyph atlas cache off
  (`atlas_cache_enabled: false`), so a COLRv1 glyph's paint graph is re-rasterised every frame it
  is drawn; a full picker grid (a few hundred glyphs) scrolling at 60 Hz is the case to time
  before M9 ships; the probe drew ten glyphs and timed nothing. **Measured 2026-09-27: "Hybrid
  harness backend" below.**

**Recommendation for the emoji grid**, in order:

1. **COLRv1 by name** for the whole set: `'Noto Color Emoji'` in the grid's font stack, the
   system's own font (Fedora installs the COLRv1 build by default), full Unicode coverage, skin
   tones, flags and ZWJ sequences, crisp at any size and scale. Time a scrolling grid on
   vello_hybrid first (above).
2. **AnimatedEmoji's static first frames** (`playback: EmojiPlayback::Still`) for the 42 we ship,
   wherever the grid picks the user's picture: those cells must look exactly like the picture
   they become, and they already exist, so the picture picker uses them and the general emoji grid
   uses COLRv1.
3. **Pre-rendered sprite sheets of the whole set**: only if the scrolling measurement fails. It
   costs several MB of PNG per size, a build step, and loses the system font's updates.
4. **CBDT**: no. It paints nothing, and the one-feature fix panics the window renderer.

## File drops and a second window (2026-09-27)

Two gaps mailo's native window needed (mailo items 21 and 8). Branch `drop-and-windows`; blitz
rev unchanged (`e99fbdbd`), no Blitz fork. APIs in CONSUMING.md "File drops and a second
window"; what mailo writes in `docs/mailo-migration.md` §6.8.

1. **File drops.**
   - **winit 0.31 (beta.3) does not hand over paths in its drag events.** The brief expected
     `DragEntered/DragMoved/DragDropped/DragLeft` with paths; at this version they are
     `DragEntered { id, position: Option<_> }`, `DragPosition { id, position, proposed_action }`,
     `DragDropped { id, proposed_action }` and `DragLeft { id }`, over a data-transfer API: the
     app reads what the drag offers (`ActiveEventLoop::data_transfer(id)`), asks for a type
     (`fetch_data_transfer(id, &TypeHint::UriList)`), and the bytes arrive later as
     `DataTransferReceived`. A drag is **refused until the app accepts it**
     (`set_valid_dnd_actions(id, &[DndAction::Copy])`), and a refused drag let go arrives as
     `DragLeft`, not `DragDropped`.
   - **Fetch on enter, not on drop.** Wayland finishes the offer inside the `DragDropped` it
     sends, so a fetch after the release is `Ignored`; X11 keeps it until the data comes. The
     host fetches the URI list as the drag enters, and a release that comes before the paths is
     held (`Released`) and lands when they arrive.
   - **Wayland and X11 differ in the details, not the shape.** Wayland gives the position on
     enter (`Some`, physical pixels converted from the surface's logical ones); X11 gives `None`
     and the first position with the first `XdndPosition`, and only proposes `Copy`. X11 answers
     each position with the accept state the app set *before* it (winit sends `XdndStatus` before
     the event reaches the app), so the cursor there trails the pointer by one move. Both deliver
     the same five events otherwise, so the host has one code path.
   - **Blitz ignores every one of them**: blitz-shell's `handle_winit_event` matches the three
     drag events with empty arms, and the document has no drag or drop events at all. The
     dioxus-native window hook ds-native already uses for IME (it hears each winit event before
     the document) is where the host reads them, and it has the `ActiveEventLoop` the
     data-transfer calls need.
   - **What was built.** `ds` (pure): `FileDrag`, `FileDragInput`, `FileDrop`, `Offer`,
     `DropAcceptance`, `DropHit`, a tracker, `HostFileDrop` and `use_file_drop`. ds-native:
     `crate::window_drop` (winit to inputs, the fetch, the answer), `crate::drop_hit` (Blitz's
     `element_from_point`, then the innermost registered target on the ancestor chain), and
     `Harness::file_drag`. A target writes the existing `DropState` values (`data-drop="target"`
     over it, `"accepts"` elsewhere during a file drag), so no new attribute value or class.
   - **Paths only.** A URI list is read with winit's `try_as_file_paths`, which fails the whole
     list if any entry is not a `file:` URI; such a drag, and one that offers no URI list, is
     `Offer::Other`, refused and never lit.
   - Proof: `ds` unit tests (the tracker's table, the view and acceptance table) and
     `tests/file_drop.rs` on a real Blitz document: files entering over the composer light it
     `target` and its inner chip `accepts`; moving off lights both `accepts` and refuses; leaving
     clears both and drops nothing; a release over the composer hands it exactly the two paths and
     it shows `Dropped` until the next drag; a release on the chip reaches only the chip; a
     release before the paths lands when they come; a dragged URL lights and drops nothing.
   - **Not verified: a real drop from a file manager.** The winit half (`crate::window_drop`) is
     not driven by any test (a test cannot make an `ActiveEventLoop`), and no drag was performed
     by a person. `cargo run -p ds-native --example file_drop` is the check; it is queued in
     `docs/manual-checks.md` (quire) for Wayland and X11.
2. **A second window.**
   - **dioxus-native at this rev has no public way for a running app to open one.**
     `DioxusNativeApplication::add_window` pushes onto blitz-shell's `pending_windows`, which only
     `can_create_surfaces` drains (a resume, which a desktop never repeats), and a window made
     that way gets none of dioxus-native's contexts: its own `Document`, the window, the shell,
     history, the renderer, and `use_window_event`'s registry, whose type is crate-private, so no
     outside code can provide it either. `launch_cfg_with_props` runs the event loop itself and
     returns no application.
   - **The route through public types.** ds-native now runs the loop itself: it builds each
     window's document as `launch_cfg_with_props` does (with quire's features: no `net`, no hot
     reload) and runs one `DioxusNativeApplication` per window under its own `ApplicationHandler`
     (`crate::window_shell`), which routes each window event to the application whose window it is
     and forwards the lifecycle calls to all. Each application sets its one window up with every
     dioxus-native context, exactly as for a first window. Two dioxus-native providers are private
     (the `dioxus:` asset net provider and the link opener); ds-native has its own of each, doing
     what those do at quire's features (`crate::native_providers`), with `dioxus-asset-resolver`
     and `webbrowser` at the versions already in the tree. `blitz-shell` is named directly for
     `BlitzShellProxy`, `WindowConfig` and the event loop. Three workspace dependencies, no new
     crate in the lock.
   - **Closing a second window must not end the loop.** blitz-shell exits the loop when an
     application's last window closes, and every second window is its own application's last. So
     each application's shell events go through a relay only the handler reads: a window's
     `CloseWindow` (its frame's close) and a compositor's `CloseRequested` for a second window drop
     that application instead of reaching it. The first window's close drops the others first, then
     reaches blitz-shell and ends the loop as before (winit wants windows gone before the loop
     exits).
   - **A closed window's renderer must outlive it (NVIDIA, Wayland).** Each vello-hybrid renderer
     owns a wgpu instance. Dropping a second window's renderer made the first window's next frame
     crash inside `libnvidia-eglcore` (`vkAcquireNextImageKHR` through a null function pointer;
     backtrace through `wgpu_hal::vulkan::Surface::acquire_texture`). A closing window's renderer is
     now suspended (its surface released, as blitz-shell does) and kept, and the next window opened
     draws with it; with that, closing and reopening is clean. Not reported upstream yet; whether
     any other driver cares is unknown.
   - **What cannot be done cleanly:** a second window's `use_window_event` never hears its own
     `CloseRequested` (handing it on would end the loop through blitz-shell's exit), so an app acts
     on close in a `use_drop`; the VirtualDom's drop runs them. A `Signal` cannot cross between
     windows (each VirtualDom has its own runtime); data goes by props or a shared `Arc`.
   - Proof: unit tests for the request queue (every request wakes the loop once and is answered
     in order; `Opening` until the loop opens it) and the close rule, and a live run of
     `cargo run -p ds-native --example second_window` with `QUIRE_AUTOPILOT=1`, no wrapper, on the
     desktop's Wayland session (KWin, NVIDIA) and under X11 (XWayland): the first window and every
     message window print the same eight seams present (`HostModality`, `HostScale`, `HostCaret`,
     `HostClickFocus`, `HostFind`, `HostFileDrop`, `WindowHost`, and the app's own
     `with_context` value); a window closed by its handle prints its `use_drop` and reads
     `Closed`; one that closes itself through its frame's `WindowHost::close` does the same; closing
     the first window with a third open drops the third, and `launch` returns, exit 0. No harness
     test: the harness has no event loop, and `open_window` there is `OpenWindowError::NoHost`.
   - Manual (queued): the compositor's close of a second window (its title-bar button, Alt+F4)
     and `handle.focus()` raising it; the runs above closed windows only from the app's side.

## Palette fixes: selection in view, caret at the end, trail gap (2026-09-27)

- **Blitz's `scroll_into_view` scrolls the document viewport only** (`BaseDocument::
  scroll_into_view` aligns against `viewport_scroll` and calls `scroll_to` on the root), so
  `MountedData::scroll_to_with_options` never moves a nested scroller such as the palette's
  360 px list. ds-native's `HostReveal` (`ds_native::reveal::REVEAL`, provided by `launch`, the
  harness and `ds_native::focus::provide`) reads the item's offset in the list from the layout
  tree and sets the list's own offset with `scroll_to` (instant, clamped); the rule is
  `ds::nearest_scroll`, CSSOM's `block: nearest`. A webview keeps the renderer's own call (sill
  Q340). A group's "Show More" stop is drawn in its header, above its rows, so reaching it with
  Down scrolls the list *up* to the header.
- **Blitz offsets an element's own client rect by its own scroll offset**
  (`Node::absolute_position` subtracts the node's `scroll_offset` before recursing), so a
  scrolled list's `getBoundingClientRect` moves up with its content. A test that wants the
  list's visible rect reads it before anything scrolls (`tests/palette_fixes.rs`).
- **Blitz lays a plain inline span's horizontal margin out as nothing**: a shaped row's
  `.ds-menu-when{ margin-right }` gave "00:33↵" with no gap (sill Q343). `.ds-menu-when` and the
  new `.ds-menu-keys` are `inline-block`, and the gap is 6 px (`--s-6`) as intended.
- **A Blitz field's caret starts at byte 0** when its value is set, and a focus write does not
  move it; the palette now asks for `InitialCaret::End` through `FocusRequest::with_caret` and
  the host's `HostPlaceCaret` (parley's `move_to_text_end`), so a palette opened on a query reads
  Right as `Caret::AtEnd` at once (sill Q341).

## Spelling (2026-09-27)

mailo's composer wants the reference platform's spellchecking on `EditSurface`. Branch
`spellcheck`. design/04-COMPONENTS.md section 50 has the behaviour; CONSUMING.md "Spelling" the
API; `docs/mailo-migration.md` §6.9 what mailo adds.

### Decisions

- **Checker: `spellbook` 0.4, unmodified.** Pure Rust, reads Hunspell `.aff`/`.dic` as the
  system ships them, no C library. MPL-2.0: file-level copyleft, already allowed by
  `deny.toml` (stylo, cssparser and `option-ext` arrive under it); depending on it obliges
  nothing of ours, and no source of it is copied (CONVENTIONS "Borrowing from other
  projects"). Its deps (`hashbrown` 0.17, `foldhash` 0.2) are MIT or Apache-2.0 and Zlib.
  `cargo deny check licenses` passes with and without `--all-features`.
- **Dictionaries come from the system** (`/usr/share/hunspell`, then Debian's `myspell`
  directories, then `$XDG_DATA_HOME/hunspell`); nothing is bundled. Fedora 44 here has the
  `en_*` set. A language with no dictionary marks nothing rather than everything.
- **ds stays pure.** Tokenising, the skip rules, the CJK test, how marks follow an edit and which
  word is being typed are pure functions in `ds::spell`, table-tested. The seam is a trait,
  `SpellService`, because two implementations are swapped there (the worker and a test's fake);
  its answers are boxed futures, so `ds` needs no executor or channel crate. `ds_native::spell`
  (feature `spellcheck`) owns every file read and write and the worker thread.
- **Marks are decoration, not document.** The app's markup is never touched: a layer, last in
  the surface and absolutely positioned, holds one box per line of each marked word, from
  `HostEdit::selection_rects` less the layer's own measured corner. Out of the flow because
  mailo's `.c-body > * { margin: 0 0 .55em }` would give an in-flow layer a margin and move the
  text; last because mailo's placeholder rule is `.c-body.ph > p:first-child`. Blitz draws
  `border-bottom: dotted` as round dots (`blitz-paint` `draw_dotted_border_edge`), which is the
  reference's look; `text-decoration-style: dotted` would need the app's markup.
- **Tasks belong to the surface.** A pick runs in the menu's handler, and a task spawned there is
  the menu's: it was cancelled the moment the menu closed, so the recheck after a replacement
  never ran, and an undo straight after brought back a word whose paragraph read as already
  checked. Every checker task now spawns in the surface's scope (`task::spawn_in`), and a
  paragraph edited since the last read is dropped from the checked set, so an undo to the text a
  check once saw is checked again.
- **The word being typed.** When a paragraph changes, the word touching the caret (`start <=
  caret <= end`) is held unmarked until the caret stops touching it. The surface does not own
  the caret, so the app passes it (`caret`); mailo already computes that `TextPosition`.
- **The replacement is the app's edit.** `EditInput` is matched exhaustively by mailo's adapter,
  so a new variant would have broken it; `on_replace: EventHandler<SpellReplace>` is additive,
  and mailo's core already takes `insertReplacementText` as one undoable step.

### Proofs

`ds/src/spell/*_tests.rs` and `lang.rs`, `ds-native/src/spell/choose.rs`,
`ds-native/tests/spell_worker.rs` (a temporary dictionary; the system's `en_US` only where
installed: `recieve`, `definately` and `colour` wrong, `receive` right),
`ds-native/tests/spell_edit.rs` (the harness: marked at least 300 ms after the last key, under
the second word, dots under the glyphs; `teh` unmarked while the caret is on it, marked once a
space moves it off; right-click, pick "the", then Ctrl+Z restores "teh" and its mark; Ignore;
the context-menu key and Learn writing `en_US.dic`; a right-click off a mark opens nothing), SSR
goldens `controls/edit_surface/spell-on.html` and `spell-marked.html`. Looked at: a harness
picture of four marks under `Teh`, `brwn`, `teh`, `helo` in Inter at 16 px (round red dots just
under the descenders).

### Limits

- **Geometry follows reads, not layout.** Marks are re-measured after each input, caret move and
  check; a resize or a font load that reflows the text without either leaves them where they were
  until the next one.
- **Busy dictionaries.** The first check in a language loads its dictionary on the worker (the
  whole `spell_worker` binary, system `en_US` included, runs in 0.07 s here); the UI never
  waits for it, the marks just land later.
- **The menu takes the keyboard**, so `on_focus` hears `Out` and `In` around it; the reference
  keeps the caret shown. Undecided whether to keep the surface focused instead.
- **Languages.** The default is the locale's single language; a list of languages (checked with
  every one, a word right in any is right) works through the seam but has no setting yet: the
  proposed `appearance.spelling_languages` row is not in design/22.
- **Grammar, autocorrect and "Show Spelling and Grammar"** are not built.
- **CJK is skipped, not checked.** Thai, Lao and Khmer (no spaces either) are not in the CJK
  blocks and would be checked as whole runs against a Latin dictionary; nothing tests them.
- **Other Blitz hosts** (shell-host, sill) get the checker only by calling
  `ds_native::spell::provide()`; the harness provides none (tests call `provide_with`).

## A part's leave is not the row's enter (2026-09-27)

mailo: with the sender card open over the message list, moving the pointer onto the card opened
a different card (the row's thread card, then the next rows' sender cards), so the card's
actions (Pin to sidebar, Their mail, Copy address) could not be pressed. The guess was that
Blitz's hit test ignores the card's stacking layer. It does not; the cause was a hook. Branch
`overlay-hit-test`; blitz rev unchanged (`e99fbdbd`), no Blitz fork.

- **Blitz's hit test follows its paint order (e99fbdbd).** `Node::hit_inner`
  (`blitz-dom/src/node/node.rs:1322`) tries, at each stacking-context root, the positive-z
  hoisted children first (`:1421`), then `paint_children` in reverse (`:1437`), then the
  negative-z ones (`:1444`), then the node itself. blitz-paint (`render.rs:993-1016`) paints the
  same three lists forwards. `flush_styles_to_layout_impl` (`layout/damage.rs:622`) builds them:
  a child with a non-zero `z-index` that is positioned (or a flex/grid item) is hoisted to the
  nearest stacking-context root, sorted by z; the rest are sorted by `node_to_paint_order`
  (`:740`), which puts positioned (`z-index: auto`) children above in-flow and floated ones,
  tree order within a level. So what is painted on top is what is hit.
- **Proof (`crates/ds-native/tests/overlay_hit.rs`).** Eight `position: relative` rows, a card
  over rows 1-4: `position: fixed` after the rows, `position: absolute; z-index: 5` before or
  after them, and through the overlay host (`.ds-overlay`, the card layer's z-index, last in
  `.ds`) each take the pointer and the click, and no row hears either. quire's `HoverCard` over
  `ListRow`s, wired as mailo wires them, keeps the sender card while the pointer jumps onto it or
  crosses the row to it, and its button takes the click.
- **The cause: mailo's name hook.** `ui/row.rs` gives `ListRow`'s `on_sender` (and `on_time`) an
  `onpointerleave` that calls `over(Hook::Thread(id))`: "leaving the name is being back on the
  row". The sender card is placed 6 px below the name, over the rows. When the pointer leaves the
  name for the card, the leave still says "back on the row", the hub is warm (a card is open),
  so the row's thread card replaces the sender card at once, beside the list. The pointer is
  now over bare rows, so the next row's name opens its sender card, and so on: "a different
  row's card". mailo's own harness shows it: the card's centre hits the card before the move
  (`hits(".ds-hovercard")`), and the card is gone right after it. With the name's leave
  calling only `out()`, the same run keeps the sender card to its last action.
- **The fix (quire, additive): `ListRow`'s `onpointerback`.** A part's leave cannot know where
  the pointer went. `ListRow` owns the row's and the parts' listeners, so it answers: it counts
  the row's enters and leaves and the parts' crossings (`components/row_hooks.rs` `Back`), and a
  part's leave calls `onpointerback` after the close grace (`HoverClose`, 150 ms) only if
  nothing was crossed meanwhile and the pointer is still on the row. The wait matters twice:
  the trip from a name to its card crosses 6 px of row, and the row's leave and the part's
  arrive in one burst in an order engines disagree on (next bullet). A pointer that rests on the
  row gets the thread card when the sender card's own grace ends, which is when the hub would
  have closed it anyway. mailo: `docs/mailo-migration.md` §6.10.
- **Blitz sends leaves outermost first.** `events/driver.rs::handle_pointer_move` reverses both
  chains (`:67`) and sends `pointerleave` from the first differing ancestor down (`:93`): the
  row hears its leave before the name. UI Events sends `mouseleave` innermost first. Anything
  that orders a row's and a part's leave (as mailo's hook implicitly did) breaks on one engine
  or the other; `onpointerback` does not depend on it.
- **A real Blitz deviation, not mailo's bug.** CSS 2.1 Appendix E step 8 paints every
  `z-index: auto` positioned box of a stacking context in tree order, across subtrees.
  `node_to_paint_order` ranks them per parent only: a static box sorts below its positioned
  siblings, so an `absolute` card with no z-index inside a static wrapper after `relative` rows
  goes under the rows, for paint and hit alike (in a browser it is on top). Conversely a fixed
  card in a static wrapper before a static list of relative rows stays on top in Blitz and not
  in a browser. `blitz_orders_auto_positioned_boxes_among_siblings_only` pins the first case
  and flips when Blitz follows step 8. It cannot reach quire's surfaces: every floating one
  renders in the overlay host with a z-index (hoisted, `damage.rs:675`). An upstream fix would
  hoist `z-index: auto` positioned descendants to the stacking-context root as a zero-z layer
  in tree order (the `HoistedPaintChildren` list, with a tree-order key) rather than sorting
  them among siblings; not written, since nothing here needs it.
- **Also seen, harmless here.** `resolve` runs `flush_styles_to_layout` (which records hoisted
  offsets and each root's `content_area` from `final_layout`) before `resolve_layout`
  (`resolve.rs:111`, `:115`), so a hoisted box's hit offsets trail a layout change by one
  resolve. The overlay host's layer covers the root (`inset: 0`), so its offsets never move.
- **sill and shell-host.** shell-host's `dom/document.rs` hit-tests with the same Blitz
  (`element_from_point` for first-mouse, `doc.hit` for `element_at`; `dom/scroll.rs` too), at
  the same rev, so its hits follow the same paint order: a sill popup drawn with a z-index, or
  last among positioned siblings, is hit where it is painted. The one trap is the step-8
  deviation above (an `absolute`/`fixed` popup with no z-index inside a static wrapper that
  sorts below positioned siblings). sill's popups that use quire's overlay host are safe;
  sill passes no `PartHooks`, so the hook that broke mailo is not in it. sill's own CSS was not
  audited box by box for the deviation. Nothing here was changed in shell-host or sill.

## Hybrid harness backend (2026-09-27)

sill Q330: time the emoji picker's grid on vello_hybrid, the renderer shell surfaces use, not
only on vello_cpu. Branch `harness-hybrid`. API in CONSUMING.md "Hybrid harness backend".

- **What was built.** `HarnessConfig::with_backend(Backend::Hybrid)` paints the same document
  through anyrender_vello_hybrid into an offscreen `Rgba8Unorm` texture on one wgpu device
  opened when the harness is built (`crate::gpu_paint`); the renderer, its `Resources` and the
  uploaded images live for the harness's life, as in a window. Pictures are read back, so
  `render()` / `render_over()` and every pixel assertion work on both backends
  (`tests/hybrid_backend.rs`: under 1% of pixels differ by more than 24 levels from vello_cpu,
  the anti-aliased edges; flat fills are identical). The adapter is chosen as shell-host's
  `GpuContext::offscreen` chooses it (`AdapterPref`, `WGPU_ADAPTER_NAME`), reimplemented in
  `crate::gpu_adapter` because quire cannot depend on shell-host. No adapter:
  `Harness::try_with_config` returns `NativeError::Renderer`, no panic.
- **Timing is honest.** `Harness::paint_timed()` paints without a readback and, on
  vello_hybrid, submits and polls the device with `wait_indefinitely` before stopping the clock,
  so the frame's time includes the GPU finishing it. `PaintTime::scene` is the part spent
  building the scene (Blitz's paint walk plus vello_hybrid flattening paths and glyphs into
  strips, all on the CPU); the rest is recording, submission and the GPU. On vello_cpu the
  harness still builds a fresh renderer per picture (unchanged), so its numbers include that.
- **The benchmark** (`emoji_grid_scrolls_on_both_backends`, `#[ignore]`, run in release):
  a 400 x 480 viewport at 100%, one `overflow-y:scroll` grid of 300 `.ds-emoji-text` cells
  (50 x 48 px, 32 px glyphs, U+1F300 onwards, all distinct COLRv1 glyphs; about 90 visible at
  once), scrolled 7 px per frame for 180 frames by `Harness::wheel`, each frame painted with
  `paint_timed`. Two controls: the same 300 cells holding "Ab" in Inter, and a 104-cell grid
  (13 rows) to price the cells scrolled out of view. Three runs on this machine (Ryzen 9 9950X,
  RTX 5070 Ti on NVIDIA 615.71 Vulkan; the 9950X's iGPU, RADV RAPHAEL_MENDOCINO, selectable with
  `AdapterPref::Integrated` or `WGPU_ADAPTER_NAME=AMD`). Median of the three runs' figures, ms:

| Grid | Backend | p50 | p95 | max | scene p50 |
| --- | --- | --- | --- | --- | --- |
| 300 emoji | vello_cpu | 11.33 | 17.03 | 21.88 | 10.22 |
| 300 emoji | vello_hybrid, RTX 5070 Ti | 4.33 | 5.05 | 6.40 | 3.15 |
| 300 emoji | vello_hybrid, AMD iGPU | 4.91 | 6.65 | 10.07 | 3.07 |
| 300 "Ab" cells | vello_cpu | 0.81 | 0.97 | 1.12 | 0.53 |
| 300 "Ab" cells | vello_hybrid, RTX 5070 Ti | 2.16 | 2.24 | 2.59 | 0.47 |
| 300 "Ab" cells | vello_hybrid, AMD iGPU | 1.59 | 2.22 | 2.28 | 0.46 |
| 104 emoji | vello_cpu | 7.04 | 10.09 | 11.32 | 6.12 |
| 104 emoji | vello_hybrid, RTX 5070 Ti | 4.35 | 4.40 | 5.27 | 2.30 |
| 104 emoji | vello_hybrid, AMD iGPU | 3.72 | 3.91 | 5.09 | 2.22 |

  The wheel event plus style and layout is 0.03 to 0.1 ms p50 (0.47 ms worst) on every run: a
  scroll does not relayout. The first frame is 6 to 7 ms on vello_hybrid (once 13 ms on the iGPU).
  The runs are noisy on vello_cpu (p50 9.55 to 12.37 ms for the main grid over four runs) and steady
  on vello_hybrid (4.32 to 4.34 on the 5070 Ti).
- **The grid fits a 60 Hz frame on vello_hybrid with room to spare**: p95 5 to 7 ms on either
  GPU against a 16.7 ms budget, the worst frame 10 ms (the iGPU). On vello_cpu the p95 is 17
  ms, over budget, so the CPU backend is the one that would miss frames.
- **What sill should know about vello_hybrid's glyph cost.**
  - **It is CPU time, not GPU time.** vello_hybrid rasterises coarsely on the CPU (paths and
    glyph outlines into sparse strips) and only fills them on the GPU. With the atlas cache off
    (`Scene::glyph_run` hard-codes `atlas_cache_enabled: false`, and anyrender_vello_hybrid's
    `draw_glyphs` never turns it on), every visible COLRv1 glyph's paint graph (layers,
    gradients, clips) is walked and flattened again every frame: the scene is 3.1 ms (p50) for the
    emoji grid against 0.5 ms for the same cells as text, so about 2.6 ms for ~90 visible
    emoji, **roughly 30 µs per visible emoji per frame** at 32 px, on a 9950X core. A slower CPU
    scales it; a faster GPU does not help (the 5070 Ti and the iGPU build the same 3.1 ms
    scene).
  - **Cells scrolled out of view are cheap but not free.** 300 cells against 104 (about the
    same number visible) is 3.1 against 2.2 ms of scene: ~4.5 µs per off-screen cell, Blitz's walk
    and clipped glyph runs, not a full re-rasterisation. A virtualised grid saves under 1 ms.
  - **The rest of the frame is 1.1 to 1.8 ms whatever the content** (`total - scene`: 1.7 ms on
    the 5070 Ti for the text grid, 1.2 for the emoji grid; 1.1 and 1.8 on the iGPU): recording,
    submission and waiting for the GPU. `paint_timed` serialises it after the scene; a window
    can overlap it with the next frame's CPU work, so a window's frame is probably nearer the
    scene time than `total` (not measured).
  - **Size and scale were not varied.** The cost is flattening outlines into strips, which
    should grow with the glyph's pixel size; time the picker's real cell size at 125% and 150%
    (`Viewport::scale_percent`) before relying on these numbers there.
  - Turning the atlas cache on should make a scrolling grid far cheaper after the first frame
    (glifo would keep each rasterised glyph and draw it as an image; not tried), but it is not
    reachable from anyrender at this pin: it needs anyrender_vello_hybrid to call `.atlas_cache(true)` on its
    glyph run (an upstream change, or a fork of the one crate).
- **A harness gotcha found on the way**: inside `Ds`, a `position:absolute` child is not hit by
  Blitz's `element_from_point` (`Harness::hits` is false over it), so `Harness::wheel` over it
  scrolls nothing; the same scroll container in normal flow scrolls. The benchmark's grid is in
  flow. Not investigated further.

## Details D0 (2026-09-27)

design/26-DETAILS.md wave D0 (D0a and D0b together): the grammar of small state details as
types, its primitives, the bounded Spinner, the idle-frame assertion, two lint rules and the
gallery's Details page. Branch `details-d0b`. API in CONSUMING.md "Details".

- **Moved onto the rewritten history.** The first attempt (`details-d0`) was based on history
  from before the identity rewrite; its eight commits were cherry-picked one by one onto master
  and re-authored. Master had moved under them: the persona is gone (its `Drift` token,
  keyframes, `Anim` rows and `--pa-*` vars were dropped from the details commits, and the
  details' motion section is design/05 §4.13, after the battery fill's §4.12), `LockPrompt`
  gained a `Field` struct and the user's picture (the Checking operation rides in `Field`), and
  `PromptState::Accepted` is new (its moment is Success; first frame Rest).
- **What the types enforce.** A primitive takes a `Cue`, which only `use_detail` makes from a
  state's own `Detailed` table; a spring needs a `Contact`, which only a handler's event gives;
  a pending loop needs a `PendingToken` whose deadline cannot pass `PendingCap`. `Moment` is a
  plain enum (a table has to name moments), so "only via `use_detail`" holds for the `Cue` that
  carries one, not for the enum.
- **Idle frames are checked from both sides.** `is_animating()` sees only CSS; a Rust timer
  (a tween, a pending step) wakes the document through its waker, so `Harness::wakes()` counts
  those and `assert_settles_to_zero_frames` requires a 500 ms window with neither. Every
  primitive and both `Detailed` components (`ModuleState`, `PromptState`) end their tests with
  it; `details_layers.rs` covers the ones the first attempt had left out (`LayerGlyph` with a
  Fill, `CheckMark`, LockIn on contact and remote, OffUp and CrossFade).
- **The gallery page snapshots at rest.** `snapshot_at` runs no Rust timer, so an Appear sweep
  would be caught at its first step; the page's specimens mount `FirstShow::Still` and each
  replay mounts them `Animate`. `--detail-frames DIR` renders eight-frame strips of the sweep,
  the pending loop, the check, the shake and the slash through a live harness. The check's
  draw-on is faster than a frame strip's real render interval, so its strip shows it drawn by
  the second frame; `details_layers.rs` checks the partial dash offsets instead.
- **The lint on sill (2026-09-27, sill master b454262 against this branch).** All 22 of sill's
  stylesheets under `Standard`, `Strict` and `Details`: 0 `InfiniteLoop`, 0 `OffGrammarTiming`
  (sill times its motion with `--t-big`, `--t-move`, `--t-scene`, `--e-out` and `--e-exit`
  only). sill's 14 `*_lint` tests (stylesheets and SSR markup) pass. So `InfiniteLoop` lands as
  an error in every profile, as designed; sill has nothing to change.
- **mailo** (pinned at v0.1.11). It uses none of `Spinner`, `ModuleTile`, `LockPrompt` or
  `SyncHalo`, so the Spinner break does not reach it, and `SendPill` and `SidebarItem` keep
  their loops. One thing breaks on the bump: `accounts.css`
  `.files-look.acct-busy{ animation: busy var(--t-ambient) var(--e-in-out) infinite }` fails
  `InfiniteLoop` in its Strict `our_stylesheet_lints_clean`. mailo either adds an exception
  (`Rule::InfiniteLoop`, `.files-look.acct-busy`, with its reason) to `exceptions::STYLE`, or
  drives the busy word from a `PendingToken` started with the account's sign-in and
  `use_pending` (design/26 R4).
- **Open.** The battery ring's fill (`use_level_run`, `--t-fill` 800 ms) predates `use_sweep`
  (`--t-sweep` 700 ms) and should move onto it; until then `--t-fill` is outside the grammar's
  durations. `Touch::Contact` carries no velocity yet (H1, design/27).

## A secret's dots in a centred card; the compact month's today disc (2026-09-27)

sill Q360 and Q361. Branch `q360-q361`.

- **Q360, root cause: our CSS, exposed by a Blitz gap.** `.ds-polkit` centres its text
  (`text-align:center`), and `.ds-input-mask` (the row of dots laid over a `Secret` or
  `Password` field, `left:0; right:0` across it) inherited that, so the dots were centred in
  the field. Blitz lays a text input's own text and caret out from the start whatever
  `text-align` says, so the (transparent) typed text and its caret stayed at the leading
  edge: eight characters in the 296 px polkit field drew the dots from 114 px in and the caret
  at 69 px, before them. The placeholder was unaffected only because it has no `right:0` and
  shrinks to its text. Fonts played no part, so bundled faces on or off looked the same. Fix:
  `.ds-input-wrap{ text-align:start }`, so the input, the mask and the placeholder all read from
  the leading edge in any container (a browser, which does centre an input's text in a centred
  block, agrees). No class or API change. Regression: `ds-native/tests/secret_mask_align.rs`
  types into a real `PolkitPrompt` and reads the painted ink: first ink at the padding, all of it
  in the field's leading third (before the fix: first ink 69 px in, the dots out to 180).
- **Blitz facts (rev e99fbdbd), not fixed upstream here.** `create_text_editor`
  (`blitz-dom/src/layout/construct.rs`) clears the editor's styles and sets only the font size,
  line height and brush: an input's text never takes `text-align`, `font-family`, `font-weight`
  or `letter-spacing`. Minimal repro: `<div style="text-align:center"><input value="abc"
  style="width:300px"></div>` draws "abc" at the left on Blitz and centred in a browser. The
  second half is why a masked field's caret can still sit a little off its last dot: the hidden
  text is measured in the editor's default family, untracked, while the dots are Inter tracked
  .1em, so the caret lands within a dot or two of the end for ordinary input. Closing that
  needs the editor to take the element's font (an upstream change) or a field-drawn caret over
  the mask; neither is done.
- **Q361: the compact MonthGrid's today disc.** The 16 px disc round `--fs-caption` 10 at 700
  left two tabular Inter digits (about 12 px) under 2 px a side, and "27" touched the rim. The
  regular grid puts 11.5 px digits on a 24 px disc (about 2.1x), so the compact disc takes the
  same proportion from the token: `calc(2 * var(--fs-caption))`, 20 px, still a circle. To hold
  it the columns go 18 to 20 (seven fill the small frame's 140 px content box, which is 164 less
  12 a side; the 132 in the old comments predates the 12 px inset) and the rows 18 to 19; a disc
  overhangs its row by 1 px into the next row's clear top, and the busy dot sits inside the
  disc's foot, `--accent-ink` on today's disc. Six weeks: 14 + 10 + 6 x 19 = 138 (139 with the
  last disc), inside 140. `month_grid_density.rs` measures it and checks today's disc is 20 x 20
  inside its column. No class or API change.

## A harness on virtual time (sill Q380, 2026-09-27)

- **The flake.** sill's gate failed a different `Harness` test each run (notification center
  render, launcher actions, launcher card) at load average 50-120; each passed alone. The cause
  is the split clock the harness module doc already admitted: CSS resolved at the harness's own
  time (the sum of `advance`s), while every ds timer (`futures-timer` sleeps) and every
  `Instant::now()` (hover intent, roster rest, menu tracker, pending tokens, detail tweens) ran on
  the wall clock. Under load the two drift, so an entrance timer can end before the frame clock
  played the entrance (G295's shape) or a check lands on the other side of a boundary.
- **The fix.** `ds::time` owns time: `now()`, `since(instant)` and `sleep(d)` read a thread-local
  clock, the wall clock by default. `ds::VirtualClock` is a timeline (origin, elapsed, a queue of
  sleeps keyed by due instant then start order); `install()` makes it the thread's clock until its
  `ClockGuard` drops. Every `Instant::now()`/`.elapsed()` in ds non-test code now goes through
  `ds::time` (edit surface, lock mood, menu tracker, motor, pending token, use_settle,
  use_level_run, use_swipe, roster rest, hover hub); every ds sleep already went through
  `ds::sleep`. `tests/clock_rule.rs` scans ds's sources so a direct `Instant::now()` or
  `futures_timer` cannot come back. ds stays free of blitz and tokio.
- **The harness.** `HarnessConfig::with_clock(Clock::Virtual)` installs a `VirtualClock` before
  the first render; `advance(d)` then stops at each timer due before the end in order (moving the
  clock, running the woken tasks and renders, resolving the CSS at that same instant), then at the
  end, and never waits on the wall clock. `Harness::now()` and `settle_until` /
  `assert_settles_to_zero_frames` use the harness's clock (identical on the wall clock).
- **Proofs.** `ds-native/tests/virtual_clock.rs`: a panel entrance, a hover card and a toast run
  in 20 ms steps idle and twice with every core spun (2x cores busy threads) and give identical
  samples (presence, rects, hover cards, toast state, `is_animating`, painted-frame hashes), with
  the panel present exactly at the first step past `settle(PanelIn)`, the card at 450 ms, the toast
  hidden at 5200 ms; the G295 shape (a wall-clock stall longer than the entrance before the first
  advance) leaves the panel entering until exactly `settle(PanelIn)`, twice alike. The same
  scenario on `Clock::Wall` under the same load differs at the first sample (80 ms: present vs
  entering).
- **Default stays Wall.** With `Virtual` as the default 24 of ds-native's tests fail: they time
  windows with `Instant::now()` against `settle_until`'s instant, sleep the thread to simulate a
  stall, or click the same spot twice with a long `advance` between (see Limits). Each is a test
  rewrite, not a clock bug (checked on `cc_pane_switcher`), so the switch is per test.
- **Limits.** Off-thread work (Tokio tasks such as `ds_settings`' watch, D-Bus replies, a custom
  `AppNet` answering from another thread) is not on the virtual clock. Blitz (rev e99fbdbd) reads
  `Instant` itself for the double-click count (`last_mousedown_time`, 500 ms) and scrollbar fade,
  in `pub(crate)` fields, so they cannot take an injected clock without a Blitz patch: on the
  virtual clock two clicks at one spot are always a double click.

