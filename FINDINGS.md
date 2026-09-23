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
