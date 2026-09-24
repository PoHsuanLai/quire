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
