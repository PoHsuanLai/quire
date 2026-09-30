# Findings

This file holds two kinds of entry about the renderer stack (Blitz, stylo, taffy, parley, vello,
dioxus-native, winit) and the design system built on it: **open items**, things still
unresolved, and **standing facts**, things true at the current pin that code and tests rely on,
with the workaround quire uses. It is a reference, not a log: how each fact was found, and what
changed when, lives in git history. Code cites sections by name (FINDINGS "Edit surface"), so a
renamed heading needs its citations updated.

The pinned Blitz is the fork rev `bf588142` (github.com/PoHsuanLai/blitz, branch `quire-pin`):
upstream `e99fbdbd` plus one patch that restyles an element whose animation was cancelled
mid-way. "At the pinned rev" below means that rev, with dioxus 0.7.x, stylo 0.21, taffy at
blitz's lock, parley 0.11, anyrender_vello_cpu 0.17 / vello_cpu 0.1, anyrender_vello_hybrid 0.10 /
vello_hybrid 0.1, glifo 0.2 and winit 0.31.0-beta.3. File and line references into Blitz are at
that rev.

## Open items

- **Lower crates' CSS names variables the shell declares.** `ds-motion`'s keyframes read
  `--osd-dy`, `--banner-dx` and `--banner-dy`, and `ds-style`'s shapes read `--dock-floor`, all
  declared by `ds-shell`'s kit, so the stylesheet of `ds` alone (`ds::stylesheet()`) lints with
  `UndeclaredVar` and only `ds_shell::stylesheet()` lints clean (`ds-shell/tests/self_lint.rs`).
  Nothing under `ds` alone draws the OSD, the banners or the dock floor, so the reads are inert
  there. Ends when the keyframes and the floor shape move to `ds-shell`'s own sheets.
- **The picker's Space dot is styled above `ds`.** `ds::AppearancePicker` draws `.ds-space-dot`,
  whose rules are in `ds-shell`'s `space_editor` sheet, so under `ds::stylesheet()` alone the
  dot is unstyled (`STYLED_ABOVE` in `ds/tests/components_lists.rs`). Ends when those rules move
  to a sheet `ds` owns, which reorders the stylesheet golden.
- **Motion and interaction primitives of step 4a.3 have consumers still to move.** Two
  primitives have no caller in quire yet: `use_collapse` waits for the `Disclosure` of step 4a.5
  (`TreeItem` is a native `details`), and `LongPress` waits for the dock and titlebar menus in
  sill. `rubber::resist` is read by the host's scroll; `Swipe` keeps its own `swipe_damping`
  setting (a quarter, not `.55`) until design/22 decides the key. The tooltip's Fly label is
  CSS, so its delay is 2.5 x `--t-big` (1 s, the Tip profile's) until the Fly is driven by
  `HoverProfile::Tip`. `SubmenuOpen` and `TriangleTimeout` are read by no menu tracker yet
  (`MenuTiming` carries its own). The pressed appearance (`data-pressed`) is written by `Button`
  and `IconButton` only; the other controls lose the deleted press squish until step 4a.4
  rebuilds them. `--focus-ring` (2.5 px) is read only by the preview pane; the global ring is
  `--ring` with `--focus-gap`, and the ring's fade over `--t-quick` waits for Blitz to
  transition `outline`.
- **The Mac Look applies part of design/30 section 3.2.** `Look::Mac` carries the neutral
  colours (paper, surface, raise, ink, lines) and the radii the table names; the ink levels
  are firmer than macOS's 85, 55 and 25 % because the legibility gates hold them (design/03
  section 6). The accent (system blue #007AFF / #0A84FF), the materials' tints and edges, the
  shadows, the card's flap corner on the icon plate and the accent band's grounds
  (`accent_band::grounds`, `css::ground_css`, which read `Look::default()`) are not yet a
  Look's values; the second Look (step 4a.8) threads `TokenScope::look` to them and adds
  `Look::Arc` from the Post values recorded in design/30 section 3.2.
Upstream (pinned around; re-check at every toolchain bump):

- **Blitz fork patch.** The restyle-on-cancelled-animation patch (`bf588142`) is not upstream.
  Move the pin back to DioxusLabs/blitz once upstream has it or an equivalent.
- **dioxus-native-dom writes every attribute in the HTML namespace** (`qual_name(name, None)`).
  Unprefixed attribute selectors never match (S2), and a `details { open }` written by dioxus
  never opens ("Selectors and cascade"). Worth an upstream report; quire writes `[*|…]` and
  sets `open` itself.
- **`get_client_rect` borrows the document mutably** when a shared borrow would do, which is why
  rect reads can collide with the renderer ("Bar gaps"). Upstream report material.
- **Hit test in an inline formatting context** returns the parent, not an atomic inline
  (`inline-flex`) Button ("Polish pass"). `crates/ds-native/tests/click.rs` keeps the failing
  shape as an ignored reproduction for an upstream report.
- **Paint and hit order of `z-index: auto` positioned boxes** is ranked per parent, not across
  the stacking context as CSS 2.1 Appendix E step 8 says ("Hit testing"). The fix (hoist them
  to the stacking-context root as a zero-z layer in tree order) is not written; sill's own CSS
  was not audited box by box for the trap.
  `blitz_orders_auto_positioned_boxes_among_siblings_only` flips when Blitz is fixed.
- **Programmatic focus dispatches no focus or blur event** (dioxus-native-dom's `set_focus`
  says "TODO: queue focus events"). quire tells the newly focused field itself; the element that
  *loses* the caret to a host focus write still hears no `blur` on Blitz.
- **blitz-shell panics on a session with no clipboard** (it unwraps `arboard::Clipboard::new()`).
  quire's own calls catch it; a field's own Ctrl+C in such a session still panics.
- **anyrender's `draw_glyphs` carries no text.** PDF output walks the parley layouts to recover
  each glyph's text ("PDF output"); the clean fix is a text-and-clusters argument upstream.
- **vello_hybrid's glyph atlas cache is off** (`atlas_cache_enabled: false`, never turned on by
  anyrender_vello_hybrid), so every visible COLRv1 glyph is re-flattened every frame. Turning it
  on needs an anyrender_vello_hybrid change or a fork of that crate.
- **CJK line breaking**: parley at this rev prints `ICU4X data error: No segmentation model for
  complex script` and wraps CJK without dictionary word boundaries.
- **Closing a second window's vello_hybrid renderer crashes the first window on NVIDIA/Wayland**
  (`vkAcquireNextImageKHR` through a null function pointer). quire keeps closed renderers alive
  and reuses them. Not reported upstream; other drivers untested.
- **Blitz reads `Instant` itself** for the double-click interval and the scrollbar fade, in
  `pub(crate)` fields, so the virtual clock cannot reach them ("Timing tests").
- **Blitz's text editor ignores most of the input's style** (family, weight, letter-spacing,
  text-align). Bare fields draw their value in the default face; masked fields draw their own
  caret to compensate ("Text, fonts and editing").
- **Taking `display: none` away does not restart a CSS animation** in Stylo; quire relies on
  the `X--b` alias swap wherever a restart is needed.

Not built, or limited, in quire:

- **Other Blitz hosts get less than `launch` and the harness.** shell-host and sill call
  `ds_native::provide_host()`, which installs every part of the document host, but do not
  forward IME events or pointer capture to an `EditSurface`, have no click-focus fallback or
  hand-back, find no element by selector, and provide no spell checker
  (`ds_native::spell::provide()`) or resting-pointer hover replay; they must call
  `ds_native::snap_to_device` after each resolve themselves. Each needs a ds-native entry point
  the host calls.
- **shell-host does not register quire's faces.** `SharedFonts::system()` never calls
  `SharedFonts::register`, and Inter is not installed system-wide, so sill draws system-ui
  wherever it asks for Inter, Inter Display, Bricolage, Karla, Space Mono or Noto Serif (its
  headless `Fonts::Bundled` registers only the first `.ttf` in quire's font folder). The fix is
  shell-host's: register `ds::FACES` (`ds_native::register_fonts`). Until then, never put the
  emoji face in a stack that carries ordinary text ("Colour emoji").
- **`ds_native::launch` is not pixel-snapped and gets no hover replay**: blitz-shell resolves and
  paints in one redraw with no hook between.
- **No tooltips for `title=` on the launch path.** Blitz draws none. Closing it means a host
  overlay that shows a quire `Tooltip` for a hovered element's `title`; meanwhile
  `IconButton { tooltip }` and `HoverTarget` cover the places that matter.
- **Scrollbars on the launch path.** blitz-paint's `scrollbars` feature is off in the pinned
  block and quire's scroll containers set `scrollbar-width: none`; whether a window should draw
  a thumb (with `--scroll-thumb`) is a design decision not taken.
- **`Slider` carries `data-wheel="capture"` but has no `onwheel` handler**, and whether a host
  reads `data-wheel`/`data-overscroll` correctly is shell-host's test to write.
- **The window's IME route is proven by reading, not by a test** ("Edit surface"). Try it by
  hand with fcitx5 or IBus (`cargo run -p ds-native --example edit` prints every input).
- **Edit surface limits**: offsets are UTF-8 bytes, not graphemes; generated text that begins
  with the same character as the text after it (`1. ` before `1 apple`) can shift the
  DOM-to-layout map by one character; Tab moves the focus and cannot be kept for indentation; a
  composition interrupted by a focus change ends empty and a commit after the blur is lost;
  geometry ignores CSS transforms on the surface or its ancestors; surrounding-text requests
  (`Ime::DeleteSurrounding`) are ignored.
- **Masked fields**: a range selected in an Inline (unboxed) masked field shows Blitz's own
  highlight beside the drawn one; the mask has one dot per `char` while Blitz moves by grapheme
  (dots per grapheme would need `unicode-segmentation` in ds); a secret longer than the field
  clips the drawn caret with the dots (the mask does not scroll).
- **A field keeps the keyboard through a prevented kept click** (a host write cannot blur it
  without an event), so a rename field is not told it lost the keyboard in that case.
- **The window frame**: no veto hook for close (an app with unsaved work cannot intercept the red
  light); Left half, Right half and Centre are unavailable on Wayland (a compositor gap on
  cosmic-comp and KWin, "Window frame"; KWin's scripting D-Bus interface was not examined); the
  X11 placement path has not been run on a live X server; there is no un-minimize.
- **Second windows**: a second window's `use_window_event` never hears its own
  `CloseRequested` (apps act in `use_drop`); a `Signal` cannot cross windows (data goes by props
  or a shared `Arc`). The compositor's close of a second window and `handle.focus()` raising it
  are queued as manual checks.
- **File drops from a real file manager** are unverified (`cargo run -p ds-native --example
  file_drop`, queued in `docs/manual-checks.md` for Wayland and X11).
- **Frame link hover and click text in a window** are compiled but not driven by any test.
- **The print dialog** is unparented on Wayland (neither winit nor `launch` exports an
  xdg-foreign handle); xdg-desktop-portal-gtk issue #562 (a second PreparePrint + Print in one
  backend process can lose the fd) is not detected; COSMIC has no Print backend of its own, so a
  system without xdg-desktop-portal-gtk falls back to the viewer.
- **pdfrum asks**: sweep (conic) gradients, gradient extend Repeat/Reflect, gradient paint on
  strokes and text, blur and filters, compositing operators beyond source-over. pdfrum is pinned
  by git rev until pdfrum 0.4 is on crates.io. Right-to-left runs in PDF output are untested.
- **Pagination** is a heuristic, not CSS fragmentation ("PDF output").
- **Spelling**: marks follow reads, not layout (a resize or font load that reflows the text
  without input leaves them until the next read); whether the surface should stay focused while
  the spelling menu is open is undecided (the reference keeps the caret shown); a language list
  works through the seam but `appearance.spelling_languages` is not in design/22; grammar,
  autocorrect and "Show Spelling and Grammar" are not built; Thai, Lao and Khmer are checked as
  whole runs against a Latin dictionary, untested.
- **Pane switcher**: the height snaps to the arriving pane (no measured height transition on
  Blitz); a reversal restarts both animations from their first frame.
- **Level control and swipe have no pointer capture**: leaving the control's hit zone (the rail
  and 16 px beside it) while dragging lets go; a swipe that leaves the card releases there.
- **`TreeItem` has no keyboard toggle of its own**; no harness test presses Enter on a summary.
- **The virtual clock is opt-in** (`Clock::Virtual`): 24 ds-native tests assume the wall clock
  (they time with `Instant::now()`, sleep the thread, or click one spot twice with a long
  `advance` between). Off-thread work (the settings watch, D-Bus replies, an `AppNet` answering
  from another thread) is never on the virtual clock.
- **`assert_settles_to_zero_frames` on the wall clock** passes while a timer longer than its
  500 ms quiet window is pending (the check mark's 900 ms `SettleHold`); use `Clock::Virtual`
  for a strict check.
- **Hover replay, covered case**: when the element under a resting pointer is covered at every
  probed point, hover is cleared rather than restored, losing its `pointerleave` and re-entering
  shared ancestors (`a_block_covering_the_whole_field_is_still_entered` pins it).
- **Harness: an absolutely positioned child inside `Ds` is not hit by `element_from_point`**, so
  `Harness::wheel` over it scrolls nothing; the same container in flow scrolls. Not investigated.
- **One Blitz document with about sixty tinted cards loses layers** (cards vanish or paint at
  partial opacity) while each row alone is right. Not chased; the level sheet renders row by row.
- **Details**: the battery ring's fill (`use_level_run`, `--t-fill` 800 ms) predates
  `use_sweep` (`--t-sweep` 700 ms) and should move onto it, as should the Batteries widget
  (whose `WakeStamp` contract sill's widgets use); until then `--t-fill` is outside the grammar's
  durations. `Touch::Contact` carries no velocity yet (design/27). `VolumeGlyph`'s slash fades
  rather than draws on; drawing it would change `LevelControl` and the OSD too, left for the user
  to ask.
- **Widgets**: the month title on a see-through card defaults to `--ink-soft` (3.90:1 worst
  light, 5.19 dark) over the other option (the light card's tint at .96, 4.53:1); the user picks.
  The D-Bus transport and view templates for out-of-process widgets (design/23 section 9.5) are
  documented only; the Large calendar's events wait for the Calendar app.
- **Plate tint on light paper** stays white at the top-left stop under any tint; another
  lightness for tinted paper needs a design decision, not a flag.
- **`app_icon_path` probes the disk per call** (cache the answer with the decoded icon), and a
  partly installed set is not merged with a later lookup step.
- **Reduced motion and holds** (unsure whether settled since): `--t-flash` (the
  mentioned-person ring, 1200 ms) is shortened to `--t-quick` under Reduced like every moving
  duration token, while `--t-send-ring` and `--t-spin-step` are exempt as holds. Whether every
  hold should be exempt is an open design question.
- **Modal focus** (design/04 O-15, unsure; check design/04 before acting): focus on mount
  exists; Peek and Sheet neither move nor trap focus.
- **design/08 section 1.4's size table** lacks `IconSize::Tile48`, `Tile96` and `Px(IconPx)`.
- **Harness key caps**: the glyphs for Home, End, Delete, PageUp, PageDown and Insert are pending
  design/04 O-2 sign-off, as the arrows are.
- **The overlay bounds are the `.ds` box**, which is as tall as its content, so a menu under a
  button in a content-high root flips and clamps to the top. `RootExtent::Viewport` addresses
  roots that should fill the viewport; unsure whether anything else is still affected.
- **The gallery's own wallpaper, stage and grid layout** have no quire token or component (unsure
  whether this still matters).

## Blitz spike results (S1-S16)

Each case was rendered headlessly at 400 x 300 on anyrender_vello_cpu (S15 and S16 also on
anyrender_vello_hybrid) as a `DioxusDocument` driven with `resolve(t)`, `poll` and
`handle_ui_event`. The gallery's Gaps page restates the table.

| id | question | result |
|----|----------|--------|
| S1 | a `<style>` in the body applies | YES |
| S2 | the `.ds[data-*]` custom-property cascade, nested | PARTIAL: `[data-x=v]` never matches; `[*\|data-x=v]` and classes do |
| S3 | `@keyframes`, with `var()` inside | YES |
| S4 | `transition` on a `var()`-driven value | YES |
| S5 | swapping `animation-name` restarts it | YES |
| S6 | SVG `stroke="currentColor"` follows CSS `color` | YES, as an attribute only |
| S7 | `mask-image: url(data:image/svg+xml,…)` | YES, only with a `data:` NetProvider |
| S8 | `background-image: url(data:image/png,…)` tiles | YES, only with a `data:` NetProvider |
| S9 | `onmounted` + `get_client_rect()` | PARTIAL: 0 x 0 inside `onmounted`, correct after the first layout |
| S10 | a `futures-timer` sleep wakes the document | YES, from the component body and from a handler |
| S11 | a TTF registered in `DocumentConfig.font_ctx` is picked by `font-family` | YES |
| S12 | `:focus-visible` after a keyboard focus | NO |
| S13 | `text-overflow: ellipsis`; a `mask-image` end fade | NO for ellipsis, YES for the fade |
| S14 | `color-mix()` in a background | YES |
| S15 | `backdrop-filter: blur()` | NO on cpu and on hybrid |
| S16 | `filter: saturate()` / `blur()` | NO on stock anyrender/vello; YES on cpu and hybrid with the local forks (see "CSS `filter`") |

- **S1.** A `style` element under `<main>` applies; `Ds { stylesheet: Inject::Inline }` works.
- **S2.** Custom properties, `var()`, inheritance and nested scopes all work; attribute selectors
  are what is broken. dioxus-native-dom's `mutation_writer.rs` creates every attribute through
  `qual_name(name, None)`, in the HTML namespace, and Stylo's `attr_matches` compares
  namespaces, so `[data-theme=dark]`, `[aria-selected=true]` and `[data-variant=…]` never match
  an attribute dioxus sets. `[*|data-theme=dark]` (valid CSS that browsers also match) and
  classes do. Every attribute selector in quire's CSS is written with `*|`, and the lint flags
  an unprefixed one under the Blitz profile. This is the most far-reaching finding.
- **S3.** Keyframes with literal values, `var()` colours, `var()` distances and a `var()`
  duration all interpolate (a 1 s, 300 px linear move is at x = 90 at 0.3 s). motion.css and
  `var(--t-*)` durations work as designed.
- **S4.** A class flip through a vdom re-render transitions `width`, `background-color` and
  `transform` driven by custom properties.
- **S5.** Switching to an identical keyframe under another name (`X` to `X--b`) restarts it;
  removing `animation-name` for one resolve and restoring it also restarts. `use_pulse`'s A/B
  alias relies on this.
- **S6.** `stroke="currentColor"` as an SVG attribute follows the element's CSS `color`. A
  stylesheet rule on `path`/`circle` inside the SVG does nothing: SVG children are rendered from
  their own attributes. `Glyph` writes every stroke property as an attribute, and the lint bans
  `stroke`/`fill` styling of icon internals.
- **S7.** With the default `DocumentConfig` (`DummyNetProvider`) a `data:` mask image never
  loads and the element is fully masked. With a `data:`-only `NetProvider` (the logic of
  blitz-shell's `DataUriNetProvider`, behind a feature not in the pinned set) it paints. The
  image arrives one resolve late, so the first frame is blank. Every ds-native document installs
  a `data:` provider and a `NetWaker` that requests a redraw.
- **S8.** The same condition as S7; with the provider an 8 x 8 PNG tiles correctly. The grain
  tile and the gallery wallpaper rely on it.
- **S9.** Mounted events are flushed in `initial_build`/`poll` before any layout, so a rect read
  inside `onmounted` is `0 x 0 at (0,0)`; the same `MountedData` read after the first resolve is
  right. `use_rect` keeps the handle and reads it after the next frame, never inside `onmounted`.
- **S10.** Timers may start in render or in handlers; each wakes the waker passed to `poll` once.
  A host bridges that waker to its own loop.
- **S11.** `FontContext::new().collection.register_fonts(blob, None)` registers a face under the
  family name in the file, and `font-family` picks it. Faces are registered once in a shared
  `FontContext` cloned into each document.
- **S12.** blitz-dom's `stylo.rs` hard-codes `NonTSPseudoClass::FocusVisible => false` and
  `FocusWithin => false`, and pointer-down focuses only text inputs and checkboxes. ds-native
  tracks input modality and stamps `data-modality=keyboard|pointer` on `.ds`; focus rings are
  `.ds[*|data-modality=keyboard] :focus`, and the lint bans `:focus-visible` and
  `:focus-within` in the Blitz profile. Anything that must show while focus is inside a row (the
  hover strip, a tree row's trailing button) is revealed by the caller through a prop.
- **S13.** `text-overflow: ellipsis` hard-clips with no "…". `.ds-truncate` uses a
  `mask-image: linear-gradient(to right, black 70%, transparent)` fade; a real ellipsis is
  computed in Rust (`text/clip.rs`).
- **S14.** `color-mix(in srgb | in oklab, …)` works, `var()` inside it too. The precomputed
  washes stay for determinism, not because Blitz forces them.
- **S15.** Both anyrender backends take `backdrop-filter` as `_backdrop_filter` and ignore it.
  Behind-surface blur comes from the compositor (ext-background-effect-v1); a material uses
  `data-blur=on` with `--m-tint` or `data-blur=off` with `--m-tint-solid`.
- **S16.** Stock anyrender_vello_cpu 0.17 drops every filter when `multithreading` is on (the
  pinned feature), and stock anyrender_vello_hybrid 0.10 applies `blur()` and `drop-shadow()` only
  and one filter node at most. Quire builds against forks of vello and anyrender that paint every
  `filter` function and filter list on both (see "CSS `filter`"); `filter` is no longer linted.
  The vibrancy saturation boost stays precomputed into the tint colours.
- **Offscreen GPU rendering works**: `wgpu_context::BufferRenderer`, `vello_hybrid::Renderer::
  render` to its texture view, then `copy_texture_to_buffer` ("Hybrid harness backend").

## CSS `filter`

Measured by `ds-native/tests/css_filter.rs`: one swatch per function, and per filter list, beside an
unfiltered control, read back from the pixels of each backend (the hybrid cases skip where no GPU
adapter opens).

| function | vello_cpu (`multithreading`) | vello_hybrid |
|----------|------------------------------|--------------|
| `blur()`, `drop-shadow()` | paints | paints |
| `brightness()`, `contrast()`, `invert()`, `opacity()` | paints | paints |
| `grayscale()`, `hue-rotate()`, `saturate()`, `sepia()` | paints | paints |
| lists: `brightness() invert()`, `brightness() blur()`, `blur() contrast()` | paints, first function first | paints, first function first |

- **How.** Stock anyrender and vello cannot do this, so the workspace patches them
  (`[patch.crates-io]` in the root `Cargo.toml`) with the `quire-filters` branches of our forks,
  github.com/PoHsuanLai/vello and github.com/PoHsuanLai/anyrender, and enables anyrender_vello_cpu's `filters` feature
  in the pinned block. vello: a colour matrix filter (`vello_common`, the CPU pass in `vello_cpu`,
  a `PASS_COLOR_MATRIX` in `vello_hybrid`'s `filter.wesl` with the GPU filter struct grown from 48
  to 96 bytes), the CSS functions' matrices (`vello_common::filter_effects::matrices`), and filter
  layers in the multi-threaded CPU dispatcher (recorded on the main thread through a
  `ViewportState`, so `multithreading` and filters coexist). anyrender: `FilterEffect::
  as_color_matrix` (a colour matrix, or an affine component transfer, which is what CSS
  `brightness()`, `contrast()`, `invert()` and `opacity()` are), and a filter list as one nested
  filter layer per function. A function vello cannot draw (a blend or a lighting effect) is skipped,
  never a panic.
- **Cost.** None measurable. `multithreading` stays on and filters run on the threaded dispatcher
  (a scene with a filter layer syncs the workers once when the layer opens). The whole workspace
  suite, debug profile, sums to 259.7 s of test time across 219 binaries with the forks against
  271.5 s before (1489 tests against 1485; the heaviest snapshot binaries, `components_overlays`
  12.4 s and `segmented_thumb` 14.8 s, are unchanged). Turning `multithreading` off, the old way to
  get a filter on the CPU, cost 2.7 times the wall time of the pixel-heavy binaries.
- **The lint.** There is none: `Rule::FilterNotPainted` is gone because nothing is left unpainted,
  and `Rule::BlitzUnsupported` never covered `filter`. `backdrop-filter` and `mix-blend-mode` stay
  banned.
- **Colour maths.** A colour function is a 4x5 matrix on the non-premultiplied colour, clamped after
  (un-premultiply, apply, clamp, re-premultiply), per the Filter Effects spec; the swatches'
  browser values (`hue-rotate(180deg)` of red is (0, 108, 108)) agree within the harness tolerance.

## Conic gradients, masks and registered properties

- `conic-gradient(from calc(var(--a) * 2) at 25% 70%, ...)` with transparent stops paints and
  turns with the variable; `mask-image: radial-gradient(...)` masks; several layered conics under
  `border-radius` clip to the circle (`ds-native/tests/voice_orb.rs`).
- `@property --angle` with `@keyframes { to { --angle: 360deg } }` animates a conic gradient on the
  headless harness exactly as a `rotate()` keyframe does (`registered_property_animation.rs`); not
  measured in a window. The voice orb drives its turn from Rust regardless (design/30 `VoiceOrb`).

## Selectors and cascade

- **Attribute selectors need `*|`** (S2), in stylesheets and in harness queries alike:
  `[*|data-light=zoom]` matches, `[data-light=zoom]` matches nothing.
- **`details { open }` from dioxus stays closed.** blitz-dom's user-agent rule
  `details:not([open]) > :not(summary:first-of-type) { display:none !important }` matches only a
  null-namespace `open` (the one its own summary toggle writes), and an author rule cannot beat
  a user-agent `!important`. dioxus also writes `open: false` as the text "false", which is still
  an open attribute. `TreeItem` writes `open` itself as an `Attribute` with an empty namespace
  and value "true", or removes it (`AttributeValue::None`) when closed; "true" rather than empty
  because dioxus-ssr writes `open` only when truthy.
- **`disabled="false"` is disabled on Blitz.** A boolean attribute is written only when true
  (`Availability::Disabled` writes `disabled="true"` beside `aria-disabled`).
- **`data-*` names are case-sensitive on Blitz** (HTML folds them to lowercase), so
  `DataName::parse` takes lowercase letters, digits and `-` only, and refuses `ds-` names and the
  names quire writes or reads (`variant`, `size`, `theme`, `accent`, `motion`, `material`,
  `slot`). dioxus names attributes by `&'static str`, so each distinct runtime `data-*` name is
  interned once (a leaked string per name for the program's life); a spread goes after the named
  attributes.
- **`:where` works.** The reset is written `:where(.ds) button, :where(.ds) input, …`
  (specificity 0,0,1), so a component rule of one class outranks it. Written `.ds button`
  (0,1,1), the reset takes a lone class's `color` and `font` on every `button` and `input`.
  `self_lint` counts `:where(.ds)` as quire's own ground.
- **Blitz's user-agent sheet** gives `button { justify-content: center }` (quire's items set
  `flex-start`) and gives an unsized `input` the 300 x 150 replaced-element default (quire sizes
  every input).
- **Cascade order matters at equal specificity**: components come after motion, so a component
  rule's `animation` outranks a pulse class unless it excludes it
  (`.ds-chip[*|data-variant=person]:not(.a-chip-flash)`); `icon_button.css` comes after
  `account_tile.css`, so the add tile's rule names both classes; the shared drop rules
  (`drop_place.css`) sort last because they tie with the items' hover and current rules.
- **`DsInternals`** forbids a consumer selector on quire's classes or root attributes
  (`data-theme`, `data-hover`, `data-extent`, …); `data-slot` is a consumer seam
  (`CONSUMER_SEAMS` in `lint/selector.rs`). A type selector on a quire element trips the
  consumer-only exemption, which keys on a selector starting `.ds`, so a `div`/`li` hover target
  carries `data-as`.
- **A custom property on a `ds-*` element is how quire hands its sheet a per-instance colour**
  (`--dot-c1..3`, `--plate-*-l/-d`, `--av-bg`, `--f-*`, `--ic-size`): the markup lint allows
  custom properties inline on quire's elements and flags a literal colour in any other property
  on any element. An `<svg>` is quire's when it is `.ds-ic` or carries `data-ds-svg`.
- **`scroll-behavior: smooth` is in `Rule::BlitzUnsupported`'s table** (as `position: sticky`
  is); `scroll-behavior: auto` lints clean.
- **`Rule::UnknownAnimation` parses the `animation` shorthand**: each animation's name is its
  first identifier that is not a shorthand keyword; a name behind `var()` or in a string is not
  judged. A consumer sheet that defines `@keyframes` under one of quire's names shadows quire's.
- **Other lint rules**: `Rule::RawHairline` (Strict) flags a literal `1px`/`.5px` border or
  outline width and a box whose whole width or height is `1px`, naming the pixel token;
  `Rule::RawSpacing` (Strict) flags a literal `px` in margin, padding and gaps (`0`, `auto`, `%`,
  `em` and `var()` pass); `Rule::InfiniteLoop` is an error in every profile. `assert_clean`
  fails on an exception that suppresses nothing.

## SVG and icons

- **SVG paint is attribute-only** (S6). An SVG's own CSS does not animate on Blitz, so a glyph
  that changes (the level glyph's waves, the status glyphs' parts) is stacked `svg` parts in an
  HTML wrapper the stylesheet cross-fades by opacity; a dash offset is tweened from Rust.
- **A CSS `width`/`height` overrides an `svg`'s presentation size and scales the drawing**, so a
  glyph can be sized from the sheet (`--bar-status-glyph`, `--ic-size`). An inline `width` cannot
  be overridden by any sheet rule, so sized parts write `--ic-size` instead.
- **Masks.** A PNG works as `mask-image` as well as an SVG; its alpha is what masks, and
  `mask-size`/`background-size: 100% 100%` stretch it. Blitz composites mask layers with `add`,
  and `min()`/`calc()` with percentages resolve in `mask-size`. A mask clips the element's own
  `box-shadow`.
- **External icons.** `IconUrl` accepts `data:` and `file:` only, percent-encodes `"`, `\` and
  control characters so the URL cannot leave its CSS string, and draws a symbolic icon as a
  `mask-image` over `background-color: currentColor` and an image as `background-image`, each at
  its own resolved size. The symbolic test (`ds::icon::classify`) is OKLab by Ottosson's
  matrices, opaque meaning alpha >= 128, with the limit from `icons.symbolic_chroma_max`
  (default 0.04, range 0..0.2).
- **Glyph strokes at a fractional scale** are an even number of device pixels (`icon/stroke.rs`,
  design/08 section 1.4.1): a 16 px glyph's stroke is 2 device pixels at 1.25, 1.5 and 1.75.
- **Lucide transcriptions** come from lucide-static 1.47.0 (ISC). A `<line>` is written as the
  equivalent path. `Icon::Switches` and `Icon::MoonFilled` are quire's own compositions from
  Lucide parts (`icon/geometry_own.rs` explains each number); a filled glyph is
  `Shape::Solid(d)`, which fills and still strokes, so its silhouette equals the outline glyph's.
  At 16 px a ring knob would touch its track, so a knob is a stroked point, as Lucide draws dots.
- **App icons.** The shipped set is in design/08 section 2.11 (`assets/icons/apps/`): thirteen
  sizes, three styles (Colour, Muted, Monochrome). Monochrome ships neutral and is tinted at run
  time by `ds::icon::retint`, so one set serves every Space. The lookup
  (`ds_settings::app_icon_path`) tries `$QUIRE_ICON_ASSETS` (the apps directory itself),
  `$XDG_DATA_HOME/quire/icons/apps`, each `$XDG_DATA_DIRS/quire/icons/apps`, then the
  repository's `assets/icons/apps` in development; the first existing directory wins whole; the
  exact size, else the nearest larger, else the largest smaller. It lives in `ds-settings`
  because it reads the disk and `ds` stays effect-free.
- **Plate tint.** `IconView { plate_tint }` sends the plate's two stops and its ink through the
  same `retint::recolour` as the raster, per scheme, written as `--plate-{base,deep,ink}-{l,d}`;
  a white stop has no room to take a tint (as a white icon does not).
- **The Space tint for Monochrome** is the accent `ds::derive` returns, already at a muted
  chroma (0.045 + 0.035 x dot chroma), so it matches the frame.

## Text, fonts and editing

- **fontique reads sfnt, not WOFF2.** Faces ship as TTF for Blitz (WOFF2 only for a webview). A
  current pyftsubset keeps the input's flavour, so `scripts/subset-fonts.sh` drops the WOFF2
  flavour itself. fontique registers a face by the family name in the file: Bricolage
  Grotesque's file says "Bricolage Grotesque 96pt ExtraBold", so the subset script rewrites name
  IDs 1 and 16 to the family the stylesheet uses. fontique has no `unicode-range`.
- **Font fallback is fontique's, per cluster, first family that maps it.** With only `Inter,
  sans-serif`, emoji come from Symbola (`fc-match` agrees). Stack order: "Colour emoji".
- **Inter is the system typeface** (`Typeface::System`; `Editorial` is Bricolage, Karla and Space
  Mono). Blitz sets no optical size from the font size (no `font-optical-sizing`), so Inter's
  `opsz` is pinned into two families: Inter (opsz 14, wght 400..700, italic 400) and Inter
  Display (opsz 32, wght 500..800), cut from Inter 4.1 (`crates/ds-style/scripts/cut-inter.sh`
  records the release hash). parley applies `font-variant-numeric`, so `tnum` works; the layout
  features kept are `kern mark mkmk ccmp locl calt case tnum pnum zero`. Inter's `tnum` also
  makes `-` and `:` tabular, so a date under `.ds-mono` reads a little open; accepted, it keeps
  times aligned. A nested `Ds` takes its enclosing root's typeface. `.ds-mono` lines are 20.0 px
  (.86em).
- **Noto Serif** (`--font-serif`) is cut from Fedora's variable files (400-700, upright and
  italic) into the latin and latin-ext subsets; the stack falls back to Georgia and Times New
  Roman.
- **Space Mono's latin subset** is re-cut with U+2190-2193 added (Google Fonts' `latin` subset
  keeps `↑↓` but not `←→`, which fell back to a system face and drew as a dash). A Small key cap's
  arrow is drawn at `--fs-control` with `line-height: 1.13`, so the cap keeps a letter cap's
  height (its width grows 18 to 20 px).
- **Inline boxes.** Blitz lays out an inline box's text but not its padding or border, and a
  plain inline span's horizontal margin as nothing, so key caps, `.ds-menu-when` and
  `.ds-menu-keys` are `inline-block`. Blitz drops whitespace at the end of an inline element's
  box, so a toned run's leading and trailing spaces are written as text nodes outside its
  element.
- **Decorations.** blitz-paint draws `text-decoration` underline and line-through; it draws every
  outline style but `none` as solid (it does dash a border, so a dashed look is a border); it
  draws `border-bottom: dotted` as round dots.
- **No `line-clamp`.** A clamped body is a `max-height` of whole lines with a `max-height`
  transition, and whether its fade shows is decided by measuring its text against one line's
  height after layout.
- **Blitz's text editor** (`create_text_editor`, `blitz-dom/src/layout/construct.rs`) clears the
  editor's styles and sets only font size, line height and brush. An input's text never takes
  `text-align`, `font-family`, `font-weight` or `letter-spacing`: `<div style="text-align:center">
  <input value="abc">` draws "abc" at the left (a browser centres it). `.ds-input-wrap` sets
  `text-align: start` so input, mask and placeholder agree everywhere; a Bare field's value draws
  in the default face at the default weight (its placeholder, a span, takes the face).
- **Password and Secret fields.** blitz-dom paints a `type=password` field's characters as typed
  (no masking in blitz-dom or blitz-paint). The field paints its own text transparent and lays
  `.ds-input-mask` (one `•` per character) over it. The hidden text is measured untracked in the
  editor's default face while the dots are Inter tracked .1em (.14em in the lock pill), so
  Blitz's caret drifts from the dots; with the `CaretHost::selection` seam the input carries
  `data-caret=drawn` (`caret-color: transparent`, which Blitz honours) and the mask draws the
  caret and the selection itself, re-read in a task after each key, input, press, focus and blur
  (Blitz moves its caret after the handlers run). Blitz's caret is `cursor_geometry(1.5)`: 1.5
  px wide, the line box's height, never blinking. The drawn caret matches it, centred in the gap
  before the next dot (`calc(-.05em - .75px)`; `-.07em` in the lock pill, `-.75px` on Bare),
  hung from the line top (`top: -1.139em`) in a zero-size inline box so the baseline does not
  move. Blitz paints its own selection in `SELECTION_COLOR` (rgb 180 213 255, a blitz-paint
  constant no style reaches); a Boxed mask covers it with `--surface`. A `Secret` never writes a
  `value` attribute (the text lives only in blitz-dom's editor); to clear it, remount under a new
  key.
- **No `change` event.** blitz-dom's text input dispatches only `input`, selection and an
  implicit submit; the field makes `onchange` itself (Enter in a one-line field, or blur).
- **No file picker.** blitz-dom's `file-input` feature (off) only draws a button; a file field is
  a read-only span plus a Choose button that asks the host (`on_pick`).
- **`textarea`** is a multiline editor whose text comes from the `value` attribute; its intrinsic
  height is `rows` x line height (2 rows when absent), its width `cols` x 0.6em or 300 px; Enter
  inserts a newline. `Grow::ToContent` raises `rows` to the hard line count; soft wraps cannot
  be counted before layout, so a long wrapped line scrolls.
- **The caret starts at byte 0** when a field's value is set, and a focus write does not move it;
  `FocusRequest::with_caret(InitialCaret::End)` asks the host's `CaretHost::place_caret` (parley's
  `move_to_text_end`).
- **A field focused on mount has no editor yet**: Blitz builds it with the document's first
  layout, so select-all and caret writes answer `Busy` and retry a frame later.
- **Tab is Blitz's `focus_next_node`, run on keydown only**; a handler's `prevent_default`
  cancels it (dioxus copies the event's flags to Blitz's `EventState` after the handlers run).

## Edit surface

`ds::EditSurface` is a `div.ds-edit` (`role=textbox`, `tabindex=0`, `white-space: pre-wrap`)
that delivers input and reports geometry for an app that owns its own editor core, drawing its
own caret and selection. The route needs no Blitz fork.

- **IME never reaches a dioxus handler.** blitz-shell converts winit `Ime` to `UiEvent::Ime`;
  blitz-dom's default action edits only a focused `input`/`textarea`; dioxus-native-dom maps
  `DomEventData::Ime(_)` to `None` and its composition converter is `unimplemented!()`.
- **ds-native sees window events first.** dioxus-native runs every `use_window_event` handler
  before passing the winit event to blitz-shell (`dioxus_application.rs`), so ds-native's `Host`
  takes `WindowEvent::Ime` there and routes it to the surface registered at the focused node or
  its nearest registered ancestor (`EditListeners`); blitz-dom's own IME handling then no-ops.
  The harness calls the same router. If a Blitz bump reorders `use_window_event` after the
  document, IME silently stops reaching the surface: re-check
  `dioxus_application.rs::window_event` at every bump.
- **IME is switched on by the surface.** blitz-dom enables it (`set_ime_enabled`,
  `set_ime_cursor_area`) only when a text field takes the focus, and winit sends no `Ime`
  otherwise; the surface does the same through the shell provider as it takes the keyboard, and
  turns it off at blur.
- **Hit testing and geometry are reachable without Blitz changes.**
  `BaseDocument::find_text_position(x, y)` answers the inline root and a byte offset into its
  laid-out text; `Node::inline_root_ancestor`, `absolute_position`, `final_layout` and
  `inline_layout_data` are public; parley's `Cursor::from_byte_index(..).geometry(..)` and
  `Selection::geometry(..)` give caret and selection boxes. Geometry is in the coordinates
  `get_client_bounding_rect` uses (unrounded layout to a 64th of a pixel, less the viewport
  scroll). Hit testing uses `doc.hit` (an atom answers its nearer side), then
  `Cursor::from_point` in the vertically nearest stretch of text, so a point in padding, between
  paragraphs or past a line's end still lands.
- **The layout does not name text nodes.** A glyph run's brush is the element of its style span,
  and the layout text is every text node concatenated after white-space collapsing (plus case
  transforms, list markers, `br` as `\n`). ds-native aligns DOM text nodes to the layout text
  character by character, so a layout offset maps to (text node, byte) and back. A caret between
  two text nodes that an inline atom separates resolves to the later node's start.
- **Focus.** A press would start Blitz's own document text selection and the click after it would
  clear the focus, so the surface prevents both defaults, focuses itself through `FocusHost::focus`, and
  treats the write's success as its focus-in (a host write dispatches no `focus` event). It keeps
  its own click count. `EditHandle::focus()`/`blur()` run the same focus-in/focus-out as a press.
- **Pointer capture** is quire's: a press registers the surface's sink (`EditHost::capture`), and
  until the primary release every move and the release go to it wherever the pointer is. The
  window's hook hears winit's pointer events before the document (logical = physical / scale
  factor); while captured the surface ignores its own `pointermove`/`pointerup`.
- **Clipboard HTML.** `ShellProvider` has text only; the window reads `text/html` through arboard
  directly (`ds_native::clipboard::read_html()`). On Wayland arboard goes through XWayland, as
  blitz-shell's text clipboard does (`wayland-data-control` is off in both).
- **Stacking with an app's layers follows CSS 2.1 Appendix E** here: a positioned (z-index auto)
  block paints over an earlier absolute layer, a static block under it, and a `z-index: -1` layer
  goes under its container's background when the container makes no stacking context. For a
  selection layer drawn before the surface, the surface (or its class) must be positioned, with
  no z-index on either, or a positive one on the surface. `EditSurface` sets no `position` itself.
- **`--caret-w`** (`PixelToken::CaretW`) is 1 px floored to whole device pixels like `--hair`;
  `caret_rect` is that wide from the insertion point rightwards (0.667 px at 1.5).

## Native focus

- **Blitz's click default clears the focus** (`handle_click`) unless something on the way up is
  its own (a text field, any `input`, a submit button, a `summary`, a `label`, a link, a
  `disabled` element). A `button type="button"` is none of these. The click default also
  dispatches `dblclick` and the blur of a field the click leaves.
- **Programmatic focus dispatches no event** (Open items). Every host focus write that lands
  calls the target field's `onfocus` itself (`focus_soon_told`); `FieldHandle::blur()` calls
  `onblur` itself; a node found by selector is matched to a mounted `TextInput` through
  `GeometryHost::same` so it is told too.
- **No `MountedData` can be built for a node found by selector**: `NodeHandle` has crate-private
  fields and no constructor. ds-native wraps a found node in its own `RenderedElementBacking`
  (`FoundNode`), which answers focus, blur and select only. `focus_by_selector` waits up to
  twenty frames for the element to be drawn.
- **Keep-focus.** Under `FocusFallback::Ancestor` (the default), `Ds`'s root click handler asks
  `ClickFocusHost` what the click will do: when Blitz would clear the focus and the target or an
  ancestor is focusable (`tabindex >= 0` or natively focusable), that element takes the keyboard
  after the click if the click left it nowhere. The default is not prevented; when the ancestor
  already has the focus it is cleared first without events and refocused by a task, so a handler
  that moved the focus during the click wins. `BlitzDefault` switches the fallback off.
- **The kept-click rule.** dioxus 0.7 has no capture phase and Blitz dispatches bottom-up only
  (`events/driver.rs`), so a control that stops a click keeps it from the root. Every quire click
  handler that stops a click or prevents its default calls `focus::click::kept_click` last: with
  the default left to run it goes through `ClickFocusHost`; with the default prevented,
  `ClickFocusHost::press` gives the nearest focusable element from the pressed one up the keyboard at
  once, as a pressed button has it in a browser. `crates/ds/tests/kept_click_rule.rs` fails on a
  component `onclick` that stops a click without it.
- **Removal sends the focus nowhere.** `process_removed_subtree` blurs a removed focused node and
  sets the focus to `None` (hover and active retarget to a surviving ancestor; focus does not).
  A removed node's ancestors cannot be walked afterwards (`remove_node` takes the parent after
  processing, and the id may be reused in the same batch), so `FocusKeeper` remembers the
  focusable ancestors (id and tag) while an element has the keyboard, and when that element is
  gone and the focus is nowhere it focuses the first live candidate among: the opener a surface
  registered (`FocusHost::hand_back`, which a floating `Menu` anchored to an element uses), the removed
  element's focusable ancestors, the element focused before it, and the element under the
  pointer when the focus moved. A focus that went nowhere while its element stayed is left
  alone. It runs after each settled harness frame and before each window event reaches the
  document. A click's restore likewise re-checks liveness along a chain remembered at the click.
- **A Stay menu refocuses itself** a frame after a pointer pick, because Blitz's click on a row
  (no default action) clears the focus.

## Tasks, borrows and host seams

### Launcher gaps: task ownership and focus from a task

- **`Runtime::spawn(scope, future)` does not register the task in the scope**; only the scope's
  own `spawn` does, and dioxus-core drops exactly the registered tasks when a scope is removed. A
  task spawned the first way outlives its component, and once dioxus reuses the freed scope slot
  the stale task writes a dropped signal (`ValueDroppedError`). `ds::task::spawn_in` enters the
  owner's scope and spawns through its own `spawn`, and task bodies write through
  `try_set`/`try_get`.
- **`NodeHandle::set_focus` borrows the document when called, not when polled.** dioxus polls a
  task woken in the same turn as a dirty scope inside `render_immediate`, while the mutation
  writer holds the document, so a focus from such a task panics "RefCell already borrowed". Every
  focus change goes through `FocusHost` (ds-native's probes with `NodeHandle::try_doc` first and
  answers `Busy`); `Busy` retries. Without a host the call is guarded and a panic reads as busy.
- **An `EventHandler` made inside `queue_effect` has no scope** and panics when called; timer
  handlers are made in render and started from an effect.

### Bar gaps: rect reads and stacking

- **Rect reads can collide with the renderer** the same way: dioxus-native-dom's
  `get_client_rect` borrows the document mutably on its first poll. Every rect read goes through
  `geometry::measure::client_rect`, which asks the host's `GeometryHost::measure` (ds-native's answers
  `Measured::Busy` and the reader waits a frame); without a measurer the poll is guarded.
- **Busy retries land in the same frame.** The first four `Busy` retries of a focus write, a rect
  read and a list scroll wait for `ds::busy::after_render` (an effect, which dioxus runs outside
  the render) rather than `FRAME_SLACK`.
- **A root with negative z-index layers must be a stacking context** (`position: relative;
  z-index: var(--z-raise)`), or the layers paint under whatever is behind the root. Tinted chrome
  roots and opaque window roots both stamp this; `.ds-layer` has `pointer-events: none` so the
  layers inside the root's context do not take hits before the content, and the tinted root's
  inset edge is redrawn by `.ds-frame::after`.

### Other seams

- **Every host seam is a part of `ds::DocumentHost`, `ds::HostSignals` or a trait**, provided by
  `ds_native::launch`, the harness and `ds_native::provide_host()`: the host's `FocusHost`,
  `CaretHost`, `GeometryHost`, `ClickFocusHost`, `EditHost` (with `ImeHost`) and `FileDropHost`;
  the `FileDropBoard` and `Rc<dyn Clipboard>` beside it; `HostSignals` (modality, scale,
  activity); `WindowHost`. `ds` names none of Blitz's types.
- **`use_environment` runs its watches on a `Spawner`** (`ds::Spawner`; the portal watch and
  the file watch are tasks it hands over). `ds_native::TokioSpawner::current` is the one
  implementor; `ds_native::launch` and `Harness` enter a process-wide two-worker runtime for it.
  `ds` and `ds-settings` may not depend on a renderer, and `scripts/check-boundary.sh` forbids
  `tokio` in both. The file watch's debounce is a `futures_timer` wait on the spawner's own thread:
  `ds::sleep` follows the clock installed on its caller's thread and is not `Send`.
- **A `Signal` read guard lives through an `if let` body** (edition 2024): copy a peeked value out
  before writing the same signal.
- **Contexts on the window path** (`AppConfig::with_context`) must be `Clone + Send + Sync`:
  dioxus-native builds the tree on the event loop's thread and each document gets its own clone.

## Frames and links

- **A frame's document is built through the parent's HTML parser** with a config that inherits
  the parent's net provider (`blitz-dom/src/iframe.rs`). ds-native wraps the parser, so every
  frame document (a `srcdoc`, a `src` load, a nested frame) is born with the frame's own net and
  navigation providers. The app renders one frame after the host installs its providers, so no
  frame is parsed with dioxus-native's.
- **Network policy.** `data:` is always served; `about:` never fetches. The app's document gets
  `file:` under `NetPolicy::Local` and `Custom`; a frame never gets `file:` from ds-native (under
  `Local` and `Sealed` a frame gets `data:` only; under `Custom` everything else is put to
  `AppNet::decide`). An `<iframe src>` in the app's own markup is fetched by the app's document
  (`start_iframe_load` uses the parent's id and provider).
- **A frame's first requests are made before it has an element**: `attach_iframe_document`
  parses the sub-document (where its images are requested) and attaches it afterwards, with the
  parent mutably borrowed throughout. ds-native holds a frame's requests in a per-document
  `FrameBook` until the next walk of the document's `iframe`s reads its `data-frame-tag`, then
  hands them to `AppNet::decide`. The lookups (`tag_of`, `frame_by_tag`) answer from a
  thread-local list on the UI thread.
- **Parser.** blitz-html pulls html5ever, markup5ever and xml5ever 0.39.0 (pinned to match
  stylo's `web_atoms`), not `markup5ever_rcdom`. No JS engine is in ds-native's graph. The
  security boundary for mail stays ammonia upstream; the worst case of a parser differential is
  markup resurrected inside the frame's own document, where the network is sealed and navigation
  frozen.
- **Links.** Blitz's `IframeNavigationProvider` reloads a frame with a clicked link's target;
  ds-native replaces it, so a frame never navigates: `FrameLinks::Inert` does nothing, and
  `Intercept` hands the app a `FrameLink { frame, tag, href, text, title }` through a channel,
  with the document free. The navigation provider hears only a URL, so the text is read at
  delivery (the anchor under the frame's hover node, else the focused one, else the first
  `a[href]` resolving to the same URL). Links in the app's own document open http(s) and mailto
  in the browser.
- **Hover over frame links is hit-tested by ds-native**, not read from Blitz: Blitz forwards the
  move into the frame and keeps the hover state there, and the window hook hears a move before
  the document. `frame_hit.rs` goes into the frame's document as Blitz forwards events (minus the
  element's position, plus the frame's scroll); only a change of link is reported.

## Windows

### Window frame

winit at the pinned rev is 0.31.0-beta.3. What the client-drawn frame relies on, read in
`winit-core` and `winit-wayland` (not observed on a live session):

- `drag_window`/`drag_resize_window` send `xdg_toplevel.move`/`resize` with the pointer's latest
  button serial, which is the press's while the button is held, so a move asked on the first
  motion past the threshold is valid (asking on the press would steal a double click); after the
  release it does nothing.
- `outer_position` is `NotSupported` on a Wayland toplevel and `set_outer_position` is "Not
  possible.", so a client cannot place itself: Left half, Right half and Centre answer
  `Support::No` there. On X11 they are placed from `current_monitor()`'s position and video-mode
  size; winit reports no work area, so a half covers the output's panels too.
- No client protocol offers a placement. xdg-shell's `xdg_toplevel` requests (wayland-protocols
  0.32.13) include `set_maximized` (Fill) but no position, and none of the staging xdg protocols
  vendored there sets one; cosmic-comp's `zcosmic_toplevel_manager_v1` and KWin's
  `org_kde_plasma_window` (both privileged) have no geometry request.
- `set_minimized(false)` is ignored on Wayland and `is_minimized` is always `None`.
  `is_maximized`/`fullscreen` read the last configure, so `launch` refreshes the state on every
  `SurfaceResized` and `Focused`. `show_window_menu` is implemented for Wayland toplevels despite
  its doc.
- Close is blitz-shell's `ShellProvider::request_window_close`, handled as `CloseRequested`: the
  window drops and the loop exits, with no veto hook.
- `with_decorations(false)` asks for no second frame; winit-wayland is built with `sctk-adwaita`,
  so server decorations on a compositor without xdg-decoration draw adwaita's frame.
- `app_id` goes to winit as the Wayland or X11 platform attribute; Wayland is chosen when
  `WAYLAND_DISPLAY` or `WAYLAND_SOCKET` is set.
- `dblclick` arrives on the titlebar (the click fallback leaves it alone); a light stops its own
  `dblclick` and `pointerdown`. A long press (500 ms) or a hover-hold (800 ms: the 450 ms hover
  intent and 350 ms more) on the green light opens the tiling menu, so passing over it does not;
  inactive windows' lights are grey until the pointer is over the group.

### Second windows

- **dioxus-native has no public way for a running app to open a window**: `add_window` pushes
  onto `pending_windows`, drained only on resume, and such a window gets none of dioxus-native's
  contexts (the `use_window_event` registry's type is crate-private). ds-native runs the event
  loop itself, one `DioxusNativeApplication` per window under its own `ApplicationHandler`
  (`window_shell`), with its own copies of the two private providers (the `dioxus:` asset net
  provider and the link opener).
- **blitz-shell exits the loop when an application's last window closes**, and every second
  window is its own application's last, so a second window's close is relayed and drops that
  application; the first window's close drops the others first, then ends the loop.
- **A closed window's renderer is suspended and kept** for the next window (Open items).

### File drops

- winit 0.31's drag events carry no paths: `DragEntered`, `DragPosition`, `DragDropped`,
  `DragLeft` over a data-transfer API. A drag is refused until the app calls
  `set_valid_dnd_actions`, and a refused drag let go arrives as `DragLeft`.
- Wayland finishes the offer inside `DragDropped`, so the URI list is fetched on enter, and a
  release that arrives before the paths is held until they come. X11 gives no position on enter,
  proposes only `Copy`, and answers each position with the state set before it (the cursor
  trails by one move).
- blitz-shell ignores the drag events and the document has no drag events; ds-native reads them
  in the same window hook as IME. A URI list with any non-`file:` entry is `Offer::Other`,
  refused. A target writes the existing `DropState` values (`target` over it, `accepts`
  elsewhere during a file drag).

### Clipboard

- dioxus-native's `clipboard` feature is on (blitz-shell over arboard 3.6), so Ctrl+C/X/V work in
  every field of a `launch` window. The app's calls answer `NoHost` outside a ds-native document,
  or `Unavailable`. The harness keeps an in-memory clipboard with an HTML slot.

## Layout, hit testing and hover

### Pixel snapping

- **Blitz lays out in logical pixels and rounds to whole logical pixels.** The device takes the
  viewport as `window_size / scale`, and `resolve_layout` ends with `taffy::round_layout`
  (cumulative). Nothing in taffy, blitz-dom, blitz-paint, anyrender or vello_cpu rounds geometry
  to device pixels; paint multiplies by the scale and antialiases.
- **stylo snaps border and outline widths to device pixels** (floor, never below one device
  pixel: `border: 1px` at 1.5 is 0.667 logical px), and taffy then rounds that back up to 1
  logical pixel (1.5 device pixels). `box-shadow` offsets and spreads are painted unrounded.
- **So no CSS value alone makes a crisp line at a fractional scale.** A black `height: 1px` line
  on white inks its device rows 255, 128 at 1.5 (y 10); 128, 191 at 1.25; 128, 255, 64 at 1.75;
  two full rows at 2.
- **`ds_native::snap_to_device`** runs after every resolve in the harness and `snapshot`: it
  redoes taffy's cumulative rounding from each node's unrounded layout on the device grid,
  rounds border widths to whole device pixels (at least one), re-derives transforms and
  scrollable overflow, and rounds pure translations, through blitz-dom's public layout
  accessors. Hit testing and rect reads use the same final layout. At a whole scale it does
  nothing.
- **The pixel tokens** (`--hair` 1 px, `--hairline` .5 px, `--px`, `--ring`, `--focus-ring`,
  `--caret-w`, `--dpr`) are declared on `.ds` from `--scale-*` inputs the root writes at a
  fractional scale, so nested scopes keep the root's value. With snapping, rules, `--hair` edges
  and menu separators are one full device row at 1.25, 1.5 and 1.75.
- **Not snapped**: the `launch` window; a literal `1px` box (it lands on one or two rows by
  phase); transforms that are not pure translations while they run; text baselines (parley
  positions runs inside the snapped box); glyph diagonals and off-grid lines; 1.5 px shadows and
  inset rings; fractional scroll offsets.

### Root height

- **`#main` is `height: auto`**, so no `height: 100%` above a root resolves, and a `.ds` root is a
  block as tall as its content. A root holding only positioned content (a centred sheet, an OSD
  card, a catcher, a wallpaper frame) is 0 px tall, and so is every box from `html` down. Taffy
  places a `position: fixed; inset: 0` box against its parent, not the viewport.
- **`vh` and `vw` do resolve against the viewport.** `Ds { extent: RootExtent::Viewport }` writes
  `min-height: 100vh; min-width: 100vw`; a consumer frame can use `display: grid; width: 100vw;
  min-height: 100vh`. A panel scope that must fill its root is a classed scope
  (`.ds-panel-scope`); inside a plain `Surface` an absolutely placed panel was 24 px tall.
- **Trust rects over clicks when debugging heights**: the harness's hit test still lands a press
  inside a 0 px ancestor where a shell's input path may not.

### Hit testing

- **Blitz's hit test follows its paint order.** `Node::hit_inner` tries, at each stacking-context
  root, the positive-z hoisted children first, then `paint_children` in reverse, then negative-z,
  then the node; blitz-paint paints the same lists forwards. A child with a non-zero `z-index`
  that is positioned (or a flex/grid item) is hoisted to the nearest stacking-context root;
  others are sorted by `node_to_paint_order`, positioned above in-flow and floated, tree order
  within a level, **per parent**: a static box sorts below its positioned siblings, so an
  `absolute` card with no z-index inside a static wrapper after `relative` rows goes under the
  rows (a browser puts it on top). quire's floating surfaces render in the overlay host with a
  z-index, so they are hoisted and safe.
- **Hoisted boxes' hit offsets trail a layout change by one resolve** (`flush_styles_to_layout`
  runs before `resolve_layout`). The overlay host covers the root, so its offsets never move.
- **Transforms**: Blitz applies a transform when painting and when hit-testing (`hit_inner`
  inverts it), but `get_client_bounding_rect` (what `GeometryHost::measure` reads) leaves transforms out.
  Anything whose rect anchors something is centred without a transform (the hover strip is
  `top: 0; bottom: 0; margin: auto 0; height: max-content`; the centred sheet uses a flex stage).
- **`pointer-events: none` is honoured and inherited.**
- **`overflow: hidden` clips paint, not the hit test**: hidden lines of a clamped body take
  presses unless they have `pointer-events: none`.
- **A hidden node keeps its last layout box**; read a following element to measure a collapse.
- **An element's own client rect is offset by its own scroll offset** (`absolute_position`
  subtracts it), so a scrolled list's rect moves with its content.
- **`scroll_into_view` scrolls the document viewport only.** ds-native's `GeometryHost::reveal` sets a
  nested list's own offset (instant, clamped) by CSSOM's `block: nearest`
  (`ds::nearest_scroll`).

### Polish pass: the inline-context click

- A Button is `inline-flex`, an atomic inline. When its parent holds only inline content (the
  Button alone, beside a placeholder, or beside inline text) the parent is an inline formatting
  context, and blitz-dom's `Node::hit` returns the parent, so the click goes to the parent. A
  block sibling or a flex parent gives the Button its own box. Every quire container is a flex
  row; a consumer puts a lone button in one. `crates/ds-native/tests/click.rs` pins both working
  shapes and keeps the failing one ignored.

### Hover under a resting pointer

- **Blitz does not synthesise enter/leave after a layout change.** `BaseDocument::resolve` ends
  with `refresh_hover`, which re-hit-tests the last pointer position and stores the hovered node
  without dispatching (a TODO says so); the next move's diff is then empty, so an element that
  slid under the pointer never hears `pointerover`/`pointerenter`, and the one left never hears
  `pointerout`/`pointerleave`.
- **ds-native replays** (`hover_sync.rs` decides, `hover_replay.rs` acts, from
  `Headless::resolve`): when the hovered node changed across a resolve and a pointer event has
  arrived, put Blitz's hover back on the old element without events (probe its rect's centre,
  then its corners inset 1 px, through `hit()` and `nearest_non_anonymous_ancestor`, then
  `set_hover_to`; `clear_hover()` if none hits), then replay the last pointer event as a
  `PointerMove`. The driver then dispatches the real leave/enter and one `pointermove`, as a
  browser's fake mouse move does. At most one replay per round, bounded by `MAX_ROUNDS`.

### Pointer events

- **Leaves are sent outermost first** (`handle_pointer_move` reverses both chains): a row hears
  its leave before its name part; UI Events sends innermost first. `ListRow`'s `onpointerback`
  (a part's leave followed, after the 150 ms close grace, by the pointer still on the row with no
  crossing meanwhile) does not depend on the order.
- **Blitz sends a `click` to the release target even when the press began elsewhere**, so a
  press-drag-release onto a menu item would pick twice; the menu picks once.
- **Right-click is `contextmenu`, never `click`; the middle button is `mouseup` only**
  (`handle_pointerup`). `Button` and `IconButton` listen to all three and report a `Press`.
- **Blitz's click default walks up to the first element with an action**, and a `summary` is
  one, so a button inside a summary toggles its `details` however propagation is stopped;
  `Propagation::Stop` also prevents the default (a `type=button` has none of its own). Pressing
  in a field places the caret on pointer-down, so a field's click may be prevented safely.
- **Blitz synthesises no click from a key**; key handling is the control's own.
- **The wheel.** Blitz forwards winit's `MouseWheel` delta unchanged (positive x is content
  moving right, the opposite of the web's `deltaX`), multiplies a line by 20 px only for its own
  scrolling, targets the hover node (set only by a pointer move), and carries no scroll phase, so
  a touchpad gesture's end is a quiet spell (`DelayToken::SwipeQuiet`, 120 ms).
  `Harness::wheel` moves the pointer first.
- **No pointer capture in Blitz**: a drag that leaves an element is noticed at the next move with
  no button down (the edit surface captures itself).

### Scrolling

- **`scrollbar-width: none` suppresses Blitz's own overlay thumb outright**:
  `Node::wants_scrollbar` reads it before asking whether the axis overflows (the thumb is
  Chromium-like: 10 px, 32 px minimum, 500 ms fade delay, 200 ms fade). Every ds scroll container
  sets it.
- **A scroll does not relayout**: wheel plus style and layout is 0.03 to 0.1 ms p50.

## Motion and timing

- **Stylo and CSS motion.** `calc()` with `min()` and `var()` resolves inside a keyframe's
  `scale()` and `translateY()` (the contact keyframes scale their departure by
  `min(1, (var(--overshoot) - 1) * 25)`: full at Standard, flat at Reduced).
  Five `box-shadow` layers paint; `calc()` inside an `rgba()` alpha and a `color-mix` percentage
  from `calc(var() * 100%)` paint.
- **Transform lists of different functions interpolate as matrices**: `scale(1)` to
  `rotate(360deg)` is identity to identity and never turns. A spinning ring starts from
  `transform: none`.
- **Taking `display: none` away does not restart an animation**, so a kept-mounted surface
  replays its entrance by flipping to the `X--b` alias on each show.
- **An animation taken off an element** left it at the last animated value upstream (the pinned
  fork's patch restyles such an element). Entrances play on an element that mounts with the
  animation and never loses it (the banner card) rather than on a presence attribute.
- **No measured height transition**: a height that follows content snaps.
- **Rust-driven motion** is used where CSS cannot retarget mid-flight (the idle dim, sweeps,
  tweens, the level fill); Reduced snaps to the target with no frames.
- **Holds are not motion**: `--t-send-ring` keeps its length under Reduced (it is the undo
  window).

### Timing tests

- **The split clock.** On `Clock::Wall`, CSS resolves at the harness's own time (the sum of
  `advance`s) while ds timers and `Instant::now()` run on the wall clock, and `advance` only
  guarantees *at least* the time asked. Under load a check near a boundary lands on either side.
- **The rule** (ARCHITECTURE.md "Repo rules"): never assert a state at one fixed instant near a
  timer boundary. Poll with `ds_native::harness::settle_until` (10 ms steps, `SETTLE_BOUND` 3 s,
  returns the instant the condition first held, panics with the document's HTML on timeout) and
  assert order, or that it landed at least the window after it was asked
  (`landed.duration_since(asked) >= window`, safe because real time only overshoots). A "not yet"
  check may use a fixed elapsed time only at or under half the window. Take the start instant
  *before* the action that starts the timer (`show()`/`click()` call `advance` themselves).
  Samples of a continuous, monotonic value on the animation clock (`resolve(t)`) are safe.
- **The virtual clock.** `ds::time` owns time (`now()`, `since()`, `sleep()` read a thread-local
  clock, the wall clock by default); `tests/clock_rule.rs` forbids a direct `Instant::now()` or
  `futures_timer` in ds. `HarnessConfig::with_clock(Clock::Virtual)` installs a `VirtualClock`;
  `advance(d)` resolves at the current instant first, then stops at each timer due before the
  end in order, running its tasks and renders and resolving CSS at that instant, and never waits
  on the wall clock. Results are identical idle and with every core busy.
- **On the virtual clock two clicks at one spot are always a double click** (Blitz's 500 ms
  `last_mousedown_time` reads the wall clock).
- **`snapshot_at` runs no Rust timer to its end**; posed specimens mount still
  (`FirstShow::Still`) so a sweep or count is not caught mid-way. It lets 120 ms of wall clock
  pass first (`MOUNT_SETTLE`) for late fetches and one-frame mount waits.
- **Idle frames are checked from both sides**: `is_animating()` sees CSS only, `Harness::wakes()`
  counts Rust wakes, and `assert_settles_to_zero_frames` requires a quiet window with neither (on
  the virtual clock, with nothing pending at all).

## Materials, blur and colour

- **No blur or saturation behind or inside a surface** (S15, S16). The vibrancy boost is computed
  in OKLab (chroma x1.4, light lightness +.012) and mixed by `color-mix` on `--m-vibrancy` so a
  key can switch it off; the muted avatar's `saturate(.55)` is computed in Rust.
- **Tint alphas** are design/03 section 17.2's, each the smallest `.02` step at which every
  material's ink holds 4.5:1 over pure black and pure white, with and without compositor blur
  (`tests/legibility.rs`); the worst margin is the dark Widget, preset 2, over white, 4.551:1.
- **Frame layers.** A tinted chrome root draws the window's two gradient layers and grain inside
  one `.ds-frame` at `--m-frame-alpha` (the material's tint alpha scaled by the settings key)
  with blur, .94 without; the group's background is the current gradient, so it stays opaque
  through a Space cross-fade. The symmetric layer fade weighs the old Space `(1 - p)^2`, so at
  half the easing it is already 75 % new.
- **An outset `box-shadow` is drawn from the averaged `border-radius`.** Squircle corners
  (`n = 5`) are a six-layer mask (four quadrant SVGs, two rectangles), with the tint on a masked
  `::before` and the shadow on the unmasked box at `--r-squircle` (0.884 r, within .03 r of the
  squircle).
- **Legibility pairs** (`X` / `X-ink`, both schemes): dark `--danger-ink` is `#1A0B08` (6.06:1;
  white was 3.17), `--ok-ink` white / `#0B1A12`, `--warn-ink` `#140D03` both. The modal scrim
  (`--scrim-modal`, black .40 light / .55 dark, a colour token since `--scrim` is a colour)
  gives 3.05:1 sheet-to-paper in light (1.89 under `--scrim`).
- **The level well on light** is black .18 so the near-white fill holds at least 1.6:1 on the
  paper, the module plate and the OSD over a tint (1.85 / 1.66 / 1.93:1; .10 gave 1.37-1.64).
- **OKLCh for distant hues**: mixing violet to amber in OKLab passes through a muddy mauve; the
  shorter hue arc passes through rose.
- **A muted avatar** keeps the hue at .55 of its chroma, lightness kept; its `--on-hue` letter is
  at least 3.0:1 and within .3 of the plain one. Greyscale tones are untouched.
- **A Space dot or editor colour** is written as `--dot-c1..3` with `data-stops`, and the sheet
  paints the same 135deg gradient `space::gradient` writes (flat for one stop, 0/100 % for two,
  0/50/100 % for three).

## PDF output

Blitz lays the document out, `crates/anyrender_pdfrum` replays anyrender's recording `Scene`
onto a pdfrum canvas, and the result is a vector PDF. `anyrender_pdfrum` names only anyrender,
peniko, pdfrum and skrifa (the boundary script enforces no Blitz and no parley).

- **Recording first**: anyrender's layers are a push/pop stack while pdfrum's `Canvas::saved`
  takes a closure, so a pushed layer's commands replay inside one saved state and no unbalanced
  `q`/`Q` can be written. Faces and images are embedded before replay.
- **The document** is an `HtmlDocument` built like a snapshot's (shared font context, HTML
  parser, `NetPolicy::Sealed`, `MediaType::print()`). **Blitz honours `@media print`.** It is
  laid out once at the content box's width at scale 1 (re-laying out at 300 dpi reflows text).
  `Harness::pdf` switches the live document to the page viewport and print media, prints, then
  restores both.
- **Pages**: each page is `paint_scene` with the viewport scrolled to the page's top, mapped from
  CSS px (y down) to points (y up). The clip ends half a CSS pixel above the cut, because Poppler
  at 96 dpi snaps a clip outward and showed the next block's top border. A glyph is written only
  if the centre of its box (advance across, ascent to descent) is inside the page's band, so each
  glyph belongs to exactly one page.
- **Page breaks**: stylo is built for servo and marks `break-before`, `break-inside` and
  `page-break-*` gecko-only; `@page` is ignored. Pagination reads markers in one pure function
  (`paginate`): `data-break-before="page"` forces a break (none at the document's top);
  otherwise a page ends where it is full, moved up past every keep-together span the cut would
  split (each line box; `img`, `svg`, `canvas`, `video`, `iframe`, `tr`;
  `data-break-inside="avoid"`; each text block's first two and last two lines). A span taller
  than a page is cut; a span starting at the page top never moves. Margins come from `PageSpec`
  (`Margins::default()` is 18 mm / 16 mm).
- **The text behind each glyph**: blitz-paint hands anyrender glyph ids and positions only, but
  the parley layouts are public (`inline_layout_data`, a list item's marker, a text input's
  editor). Before painting, ds-native pairs each glyph run's glyphs with their clusters' text,
  keyed by face blob id and index, size and each glyph's id and position bits
  (`pdf/run_texts.rs`); a ligature's continuation clusters join its glyph. pdfrum writes the text
  into `/ToUnicode`, or `/ActualText` where a mapping would conflict. The key includes the blob
  id, so the renderer must paint the layout's own `FontData` (Blitz does). The pre-walk
  replicates `GlyphRunIter`'s split, so a parley change there shows as `GlyphTally::cmap_text`
  rising; re-check it at every bump.
- **The cmap fallback** (SVG text, custom widgets) reads the face's cmap backwards, preferring the
  canonical code point over radicals, compatibility ideographs, presentation forms and private
  use. It cannot spell a ligature (no cmap entry) and would be wrong for Arabic and Indic
  contextual forms. Clusters are identified by equal ranges, not start bytes, so a textless glyph
  does not swallow the next glyph's text.
- **Variable fonts**: the run's normalized coordinates go to pdfrum as
  `FontInstance::Normalized`, so outlines and `/W` advances are the layout's instance, and the
  subset is named after its `fvar` instance (`NotoSansCJKtc-Regular`, `Karla-Bold`; an unnamed
  instance as `Karla_550wght`). Noto Sans CJK on this machine is one variable CFF2 collection
  (32 MB) whose default instance is Thin, so ignoring the coordinates prints Thin; it is shared
  into pdfrum without a copy and subset to about 20 KB.
- **Images**: Blitz keeps only decoded RGBA, so ds-native re-decodes `data:` URLs to recover the
  encoded bytes. JPEG and opaque PNG are written as-is; a PNG with alpha is drawn from pixels with
  an SMask; images an `AppNet` fetched are drawn from pixels. An image brush draws once, clipped;
  Blitz tiles backgrounds itself.
- **Simplified**: box shadows dropped; filters and backdrop filters ignored; non-source-over
  compositing paints as source-over (blend modes map one to one); sweep gradients and gradients
  on strokes or text paint as their stops' average; gradients pad and interpolate in sRGB;
  synthetic bold is text render mode 2, synthetic oblique one skew about the baseline.
- **Measured** (the fixture: A4, a Karla heading, Latin and CJK paragraphs, a JPEG and an alpha
  PNG as `data:` URLs, a keep-together block, a forced break): 3 pages, 42,118 bytes, four subset
  faces (Karla, Noto Serif, Noto Sans CJK TC, DroidSansFallback); release 21-43 ms for the first
  print in a process (font context and system scan), 5.8-6.9 ms after; rasterised at 96 dpi
  against the headless snapshot, mean channel difference 0.52/255 and 0.13 % of pixels off by
  more than 64 (anti-aliasing only; the layout is identical).
- **Limits**: pagination is quire's heuristic (backgrounds and borders split at a cut; a line or
  image taller than a page is cut; table headers do not repeat; floats and absolute boxes are
  not kept; orphans and widows fixed at 2; no `break-after`; no running headers, footers or page
  numbers). Font fallback is fontique's (the fixture's Karla heading got its CJK from
  DroidSansFallback), so print CSS should name CJK families. `Harness::pdf` returns
  `Result<Vec<u8>, PdfError>`: a face pdfrum cannot read fails the write.
- **Print dialog** (`print_dialog`, feature `print`, Linux): `PreparePrint` then `Print` with a
  sealed, rewound memfd, each `Response` subscribed before the call; falls back to a temp file and
  `xdg-open`/`open`/`start` with no bus, portal or Print backend; blocks until answered.
- **Pins**: pdfrum's crates declare `rust-version = "1.92"`, so the workspace's is 1.92; pdfrum
  pins `smallvec` exactly (1.16.1). `variable-fonts` on pdfrum-edit instances variable faces and
  CFF2.

## Colour emoji

What Blitz at the pinned rev paints (48 px, headless):

| Face | vello_cpu | vello_hybrid |
| --- | --- | --- |
| Noto Color Emoji COLRv1 (Fedora's default), named in the stack | colour, including skin tones, flags and ZWJ sequences shaped to single glyphs (Noto draws the family in greys) | colour, the same to within anti-aliasing |
| Noto Color Emoji CBDT | nothing (0 inked) | nothing |
| CBDT with glifo's `png` feature on | colour | **panics** (`pixmap image sources are not supported by Vello Hybrid`) |
| no emoji family named | monochrome from Symbola, with tofu for modifiers and flags | |
| Noto Emoji (monochrome), named | monochrome outlines | |

- **COLRv1 needs no change in quire**: glifo interprets the paint graphs (layers, gradients,
  clips) on either backend. `tests/colour_emoji.rs` keeps it as a regression test (skipped where
  the font is not installed).
- **CBDT must stay off.** glifo decodes CBDT only with its `png` feature (anyrender_vello_cpu
  turns vello_cpu's defaults off); turning it on crashes the window renderer on the first bitmap
  glyph. An installed CBDT font is harmless only because it draws nothing.
- **Stack order.** Noto Color Emoji's cmap maps U+0020, the digits, `#` and `*` (keycap bases),
  and parley takes each cluster from the first family in the stack that maps it. So the text
  face must come first *and be registered*, and the emoji face must come before `system-ui`
  (which maps emoji itself, from Symbola). With the emoji face first, "Ab 2#" drew "Ab" and
  nothing for "2#". `--font-emoji` (`"Inter","Noto Color Emoji",system-ui,sans-serif`) and
  `.ds-emoji-text` are for emoji content only (the emoji grid, the preview glyph); the text
  stacks name no emoji face while shell-host lacks quire's faces (Open items).
  `tests/text_stack_fonts.rs` lays the stack out under both font contexts: under shell-host's,
  "00:22" is 84 px against 45 for Inter and 41 for system-ui.
- **For an emoji grid**: COLRv1 by name for the whole set; AnimatedEmoji's still first frames
  (`EmojiPlayback::Still`) where a cell must match the picture it becomes; pre-rendered sprite
  sheets only if scrolling were too slow (it is not, "Hybrid harness backend"); never CBDT.

## Hybrid harness backend

- `HarnessConfig::with_backend(Backend::Hybrid)` paints through anyrender_vello_hybrid into an
  offscreen `Rgba8Unorm` texture on one wgpu device held for the harness's life, and reads it
  back, so pixel assertions work on both backends (under 1 % of pixels differ by more than 24
  levels from vello_cpu, all anti-aliased edges). The adapter is chosen as shell-host chooses it
  (`AdapterPref`, `WGPU_ADAPTER_NAME`, reimplemented because quire cannot depend on shell-host);
  with none, `try_with_config` returns `NativeError::Renderer`. `paint_timed()` waits for the GPU
  before stopping the clock; `PaintTime::scene` is the CPU scene build. On vello_cpu the harness
  builds a renderer per picture, which its numbers include.
- **Measured** (400 x 480 at 100 %, 300 COLRv1 cells of 32 px glyphs, about 90 visible, scrolled
  7 px a frame for 180 frames; Ryzen 9 9950X, RTX 5070 Ti, and the 9950X's iGPU; median of three
  runs, ms):

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

- **The grid fits a 60 Hz frame on vello_hybrid** (p95 5-7 ms, worst 10 ms); on vello_cpu p95 is
  over budget. The first frame on vello_hybrid is 6-7 ms. vello_cpu runs are noisy, vello_hybrid
  runs steady.
- **vello_hybrid's glyph cost is CPU time**: it flattens paths and glyph outlines into sparse
  strips on the CPU and fills them on the GPU. With the atlas cache off each visible COLRv1 glyph
  is re-flattened every frame, roughly 30 µs per visible emoji per frame at 32 px on a 9950X
  core; a faster GPU does not help. Off-screen cells cost about 4.5 µs each (virtualising saves
  under 1 ms). The rest of a frame is 1.1-1.8 ms whatever the content, and a window can overlap
  it with the next frame's CPU work. Glyph size and scale were not varied; time the real cell
  size at 125 % and 150 % before relying on these numbers there.

## Settings and schema

- **Settings files** are `quire/appearance.toml` (design/22) and `spaces.json`, each a
  `SettingsDoc` read, saved and watched through `ds_settings::Store`. The read is lenient key by
  key: each key is laid over the struct's default and kept only if it deserialises (a
  per-field `deserialize_with` would turn a bad key into the type's zero, not the key's default).
  A number that does not fit its type falls back to the field's default. A malformed `by_index`
  entry in the Spaces store costs only its own position. A key the file sets and the struct does
  not store back is reported in `Loaded::unknown` (cut at the first unknown table) and is gone
  after the next save; a value the struct refuses is in `Loaded::invalid`, and text that is not
  the format at all is one invalid key with an empty path. A watch keeps the last good value
  when the file is not valid text.
- **The portal's colour scheme** has three values (0 no preference, 1 dark, 2 light); no
  preference maps to Light.
- **Types.** `Motion` is the preference (Standard or Reduced, default Standard, with a `label()`; the
  desktop's reduce-motion preference makes Standard resolve to Reduced); `MotionLevel` is the
  resolved pair, so a picker offers `Motion`. `ds::Px(f32)` is for layout, `ds_settings::Px(u16)` for stored keys; `ds::Percent`
  and `ds_settings::Percent` agree by shape because `ds` may not depend on `ds-settings`. There
  is one unclamped `Fraction`; components clamp. `Secs(u16)` and `Mins(u16)` are unit newtypes
  like `Px`/`Ms`/`Count`, rendered as sliders from the field's own `#[settings(range, unit)]`.
- **The schema derive** recognises text by type (`String`, `PathBuf`, `Cow<str>`) or by
  `#[settings(text)]` (`text` with `range` is an error); a one-variant enum is `KeyKind::Fixed`
  (drawn as `Widget::Readout`) and zero variants an error; a known numeric type without `range`
  is a compile error starting `MissingRange { field: <name> }`. A caller's own numeric newtype
  cannot be recognised from tokens, so it is taken for a closed enum and `kind_of` asks it for a
  `Word`: the compile error says `Word` is not implemented for it. There is no `trybuild`;
  refusals are unit-tested on the expansion.
- **Material tint alpha is a settings key**, so `recipe()` takes it as a parameter and the root
  writes it (`Ds { tint_alpha }`, thousandths) rather than the stylesheet baking it in.
- **Material keys** for banners, the control center and the launcher live in sill's settings, not
  `AppearanceSettings` (design/22 section 4.3).
- **dioxus reserves the prop name `key`**, so `HoverTarget`'s is `hover_key`.
- **Pages**: `Page::Power` holds `idle.*` and `session.lock_grace_s`.

## Spelling

- **Checker**: `spellbook` 0.4, pure Rust, reading Hunspell `.aff`/`.dic` as the system ships
  them (`/usr/share/hunspell`, Debian's `myspell` directories, `$XDG_DATA_HOME/hunspell`);
  nothing is bundled. MPL-2.0, allowed by `deny.toml`. A language with no dictionary marks
  nothing rather than everything.
- **ds stays pure**: tokenising, skip rules, the CJK test, mark tracking and the word being typed
  are pure functions in `ds::spell`; the seam is the `SpellService` trait with boxed futures, so
  `ds` needs no executor; `ds_native::spell` (feature `spellcheck`) owns file access and the
  worker thread.
- **Marks are decoration**: a layer, last in the surface and absolutely positioned, holds a
  `border-bottom: dotted` box per line of each marked word from `EditHost::selection_rects`. It
  is out of flow and last so an app's child-margin and first-child rules do not move the text.
- **Tasks belong to the surface** (`task::spawn_in`): a task spawned in the menu's handler is
  cancelled when the menu closes. A paragraph edited since the last read is dropped from the
  checked set, so an undo to text a check once saw is checked again.
- **The word being typed** (touching the caret) is held unmarked until the caret leaves it; the
  app passes the caret. A replacement is the app's edit (`on_replace`), one undoable step.

## Design facts the code relies on

- **The hover strip** (design/04 section 17): right 8, vertically centred, padding 3, 26 px
  buttons with gap 3, a 1 px border; in a 74 px row it is 34 tall, 20 from the top, its right
  edge 9 in from the row's border box, `26n + 3(n - 1) + 8` wide. A shown strip wider than half
  the row less 9 legitimately covers the row's centre.
- **List row name budget**: `NAME_BUDGET` = 26 characters, from a 980 px minimum window, about
  196 px of name column and about 7.4 px per character at ui 13.5 / 700.
- **TextInput height** is `calc(1.55em + 16px)` boxed and `calc(1.55em + 8px)` inline (in `em`,
  so larger fields stay one line).
- **Button heights**: Primary carries a transparent `--hair` border so Primary, Secondary and
  Danger share one height at each size (the background shows through a transparent border).
  Danger at rest looks like Mini by design (design/04 section 1).
- **Compact month grid**: the today disc is `calc(2 * var(--fs-caption))` (20 px, the regular
  grid's digit-to-disc ratio); columns 20, rows 19; six weeks fit the small frame's 140 px
  content box.
- **Half-width module tiles clip their title** instead of fading: `.ds-truncate`'s fade covers
  the column's last 1.5em whether the words reach it or not, and the column (about 72 px) is
  barely wider than "Bluetooth".
- **Shell type scale** (tuned tokens declared on `.ds` from inputs with the key's default behind
  them, so one inline write reaches every nested scope): bar 13/500, pill 24, radius 4; text
  menus 22 px rows, 13/400, highlight radius 6, 5 px separator margins; launcher field 22/500
  with a 20 px glyph, rows 14/12; tooltips 12.
- **The level control** defaults to `LevelLook::Capsule` (the current macOS form: glyph, level
  and mute in one shape); `CapsuleKnob` and `Segments` remain. The two-tone glyph is drawn twice,
  once clipped by the fill. The rubber band past an end is `6 x d / (d + 12)` px, off under
  Reduced; keys step on a 16 or 64 grid.
- **The OSD's motion** reads a signed `--osd-dy` per position (-8 px top right, 8 px bottom
  centre), so one keyframe pair serves both.
- **Hover intent timing**: 450 ms open, 0 warm, 150 ms close, 400 ms warm window
  (`DelayToken`s), shared by every card, the Card tooltip and `HoverKind::Tip`.
- **Notification swipe** (`motion::swipe`): 1:1 right, a quarter left, springs back under 80 px
  and 600 px/s, flies out past either, release speed from the last two moves within 100 ms,
  mostly-vertical movement ignored, a drag swallows its click.
- **A battery glyph quantises** to 22 half-unit steps; any charge shows one step; the bolt and
  plug sit over a fill dimmed to .35 (an SVG mask would switch at once while the mark fades in).
- **The dock pill never decides an app icon's colourway**: every plate hue is 3.0-4.1:1 against
  the pill on the Work and Home frames in both schemes, because the plate's L 0.62 sits between
  the light and the dark frame.
