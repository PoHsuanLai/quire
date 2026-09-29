# Consuming quire

This is the guide for a program that draws with quire instead of hand-rolling its own CSS and
markup: today that is `examples/consumer` (a minimal, fully worked example — read its
`Cargo.toml`, `src/lib.rs` and `tests/coherence.rs` alongside this doc), sill and mailo. It assumes you have read
`design/README.md`'s reading order at least through `04-COMPONENTS.md`; this doc does not repeat
what a component looks like, only how to reach it from your own crate.

Citations below follow `design/README.md#3-citation-convention`:
`design/<file>.md#<heading-anchor>`.

## 1. Adding quire

**Rust version.** quire's minimum is `rust-version = "1.92"`: pdfrum's crates (PDF output,
`ds-native`) declare 1.92, above the 1.91 blitz needs at the pinned rev. The toolchain quire
builds and tests on stays pinned at 1.98.1 (`rust-toolchain.toml`); a consumer on an older
compiler than 1.92 cannot build `ds-native`.

**zbus and your executor.** `ds-settings` builds zbus with its default `async-io` backend, which
works under any executor, tokio included. Do not enable `zbus/tokio` in an app: Cargo unifies
features, and with `tokio` on, zbus's blocking API panics on a thread that drives a tokio
runtime ("Cannot start a runtime from within a runtime"; mailo's keyring and notifications hit
this). A shell that runs zbus on tokio everywhere enables `ds-settings = { features = ["tokio"] }`
deliberately.

**Today: a path dependency.** quire has no published version yet, so every consumer depends on
it by path, the way `examples/consumer/Cargo.toml` does:

```toml
[dependencies]
ds          = { path = "../quire/crates/ds", features = ["lint"] }
ds-settings = { path = "../quire/crates/ds-settings" }
ds-native   = { path = "../quire/crates/ds-native" }   # only if you run on Blitz
```

Enable `ds`'s `lint` feature wherever your own tests call `ds::lint` (coherence rule 1 and 2,
section 5 below) — it is off by default so a consumer that never lints does not pull in
`cssparser`.

**Later: a git dependency**, once quire is tagged, the same shape shell-host and sill will use:

```toml
ds = { git = "https://github.com/PoHsuanLai/quire", tag = "v0.1.0", features = ["lint"] }
```

**The pinned dependency block.** `ds`'s own manifest resolves its dependencies (`dioxus`,
and for `ds-native`, the whole blitz/anyrender/wgpu stack) against *quire's own* workspace —
a path dependency does not inherit your workspace's `[workspace.dependencies]`, because
`crates/ds/Cargo.toml` has no `[workspace]` of its own and so joins whichever ancestor manifest
does (quire's root, not yours). You only need to pin the lines **you** name directly: at minimum
`dioxus` (to write `rsx!`), plus `dioxus-ssr` as a dev-dependency if you lint an SSR render
(section 5). Copy those lines **verbatim** from `quire/docs/workspace-deps.toml`
(`CONVENTIONS.md#10-change-discipline`: "the pinned block ... is copied verbatim into
every workspace"; `docs/workspace-deps.toml` is quire's own copy of the source of truth, not
owned by this doc). If you also run on Blitz (`ds-native`), your own crate that calls
`dioxus_native::*` directly (rare — most consumers only call `ds_native::launch`) needs the
`dioxus-native`/`blitz-*` lines too, exactly as pinned, never a different revision.

**If your own crate sits inside a Cargo workspace tree it does not own** (as `examples/consumer`
does, under quire's own directory), give it an empty `[workspace]` table so Cargo does not treat
it as an orphaned member of the workspace above it.

## 2. The `Ds` root

**A bare document, with no `Ds` root** (a never-painted input catcher, a 2 px hot-corner
surface) still carries Blitz's user-agent stylesheet, which sets `body { margin: 8px }`: its
content never reaches the true (0, 0) corner, and `position: absolute`/`fixed` with `inset: 0`
paints nothing on it (sill hot corners, 2026-09-26). shell-host injects `body { margin: 0 }`
into every document it hosts; a bare document elsewhere (`ds_native::launch`,
a test) must cancel the margin itself or draw inside a `Ds`.

Every quire component must be drawn inside one `Ds` (`root/ds.rs`; design/03-COLOR.md
section 17.1; `crates/ds/src/root/ds.rs`). It resolves your appearance to a scheme, an accent
and a motion level; stamps `data-theme`, `data-typeface`, `data-accent`, `data-motion`,
`data-material`, `data-blur`, `data-modality` and `data-hover` on its own `div.ds`; injects the stylesheet
(unless you ask it not to); and provides the `Scope`, `HoverHub`, `ToastHub`, `LayerStack` and
`Overlays` contexts every component reads. It also renders `OverlayHost` and `ToastHost` after
your children, so menus, popovers and toasts always have somewhere to mount. `ToastHost` lays
out nothing while the hub is empty: a pushed toast mounts hidden for one frame and rises from
there, and is dropped again once it has sunk, so an idle root has
no toast element in its markup or its picture.

```rust
use ds::{Appearance, Ds, Material};
use dioxus::prelude::*;

#[component]
fn App() -> Element {
    rsx! {
        Ds {
            appearance: Appearance::default(),   // theme, accent, motion — see section 3
            material: Material::Window,           // one of the eight Materials (03-COLOR §17)
            style { {ds::stylesheet()} }           // or Inject::Inline (the default) does this for you
            YourPage {}
        }
    }
}
```

`Ds`'s props, all named, none a `bool` (`CONVENTIONS.md#4-types`):

| Prop | Type | Default | What it does |
| --- | --- | --- | --- |
| `appearance` | `Appearance` | required | theme, accent, motion preference — section 3 |
| `system` | `SystemPrefs` | `SystemPrefs::default()` | the desktop's own scheme/motion/contrast, from `ds_settings::read_system_prefs` |
| `look` | `SpaceLook` | `SpaceLook::default()` | the frame's dots, grain, theme override and card accent (design/21-SPACES.md §1) |
| `material` | `Material` | required | which of the eight materials this root paints (design/03-COLOR.md §17.1) |
| `blur` | `BlurState` | `BlurState::default()` | whether the compositor blurs behind this surface |
| `stylesheet` | `Inject` | `Inject::Inline` | `Inline` puts a `<style>` inside `.ds` (spike S1); `Host` lets you inject `ds::stylesheet()` yourself |
| `chrome` | `Option<RootChrome>` | `None`: `RootChrome::of(material)` | `Painted` or `Transparent`: whether the root box paints its material. A Popover, Sheet or Toast root is transparent by default (it hosts floating cards, which paint the material themselves); pass `Painted` for a root that *is* the panel (the launcher) — section 6, "bar gaps" |
| `ground` | `Option<Ground>` | `None`: `Ground::of(material)` | `Paper` or `Frame`: which inks the content takes; Bar and Dock default to `Frame` |
| `frame` | `Option<FrameTint>` | `None`: `FrameTint::of(material, chrome)` | `Opaque` (the window's frame), `Tinted` (the Space gradient at the tint alpha: bar, dock, a painted popover, OSD, widget) or `None` (the material's flat tint) |
| `radius` | `Option<Corner>` | `None`: the material's own corner | `Corner::Token(Radius::…)` or `Corner::Px(Px(n))`: overrides `--m-radius` inline, for a root whose corner is a setting (the dock's `dock.pill_radius_px`) |
| `tint_alpha` | `Option<Alpha>` | `None` (the tint's default alpha) | the materials' tint alpha over compositor blur (design/22-SETTINGS.md §3.1 `appearance.material_tint_alpha`); pass `ds_settings::Environment::tint_alpha()` (thousandths: `Alpha(800)` is 80%) once you are reading a live `Environment` (section 3) rather than leaving it at the default |
| `stack` | `Option<MaterialStack>` | `None` (the keys' defaults) | the material stack's six alphas (highlight and hairline per scheme, shadow strength, vibrancy; design/22-SETTINGS.md §3.1 `appearance.material_*`); pass `ds_settings::Environment::material_stack()` once you read a live `Environment`, as with `tint_alpha` |
| `extent` | `RootExtent` | `RootExtent::Content` | `Content`: as tall as the root's content (a window, the bar, a card). `Viewport`: at least the viewport (`min-height:100vh; min-width:100vw`), **the option for an overlay surface** (an OSD, a sheet, a click catcher) whose content is all positioned and would otherwise leave the root, and everything it places, 0 px tall (FINDINGS "Root height") |
| `scale` | `Option<Scale>` | `None`: the host's `ds::HostScale`, else 1x | the device scale this root draws for, in 120ths (`Scale(180)` is 1.5x, the `wp_fractional_scale_v1` unit and shell-host's `Scale`); the root writes the pixel tokens for it (below). `ds_native::launch`, `Harness` and `snapshot` provide `HostScale` themselves; a host that is not `ds-native` passes `scale` |
| `window` | `WindowFrame` | `WindowFrame::None`: the root is what it was | `WindowFrame::Titlebar { title, lights: TrafficLights::{Shown, Hidden}, timing }` (or `WindowFrame::titlebar(title, lights)`): the client-decorated window's frame, a 28 px titlebar that moves and zooms the window, the traffic lights, the body and eight resize edges, all acting through the host's `ds::HostWindow` — section 6, "Window frame" |

### Pixel snapping: `scale`, the pixel tokens and `snap_to_device` (2026-09-25)

At 1.25, 1.5 or 1.75 a `1px` line covers a fractional number of device pixels and paints as one
full row and one half row. Three pieces keep every line whole device pixels (design/01-LAYOUT.md
§2.1; FINDINGS "Pixel snapping"):

1. **The root knows the scale.** `Ds { scale: Some(Scale(180)) }` (or the `ds::HostScale`
   ds-native provides) makes the root write the pixel tokens' inputs inline. With no scale, or
   at `Scale::ONE`, it writes nothing and every token is its 1x value, so nothing changes.
2. **Lines read the pixel tokens** (`ds::PixelToken`, on `.ds`):

   | Token | 1x | 1.25 / 1.5 / 1.75 | 2x | Use it for |
   | --- | --- | --- | --- | --- |
   | `--hair` | `1px` | one device pixel | `1px` | every 1 px border, separator, rule, `0 0 0 1px` ring, 1 px inset highlight |
   | `--hairline` | `.5px` | one device pixel | `.5px` | a half-pixel hairline (the material stack's) |
   | `--px` | `1px` | one device pixel | `.5px` | exactly one device pixel, whatever the scale |
   | `--ring` | `3px` | 3 px to whole device pixels | `3px` | a focus or flash ring's spread |
   | `--focus-ring` | `2.5px` | 2.5 px down to whole device pixels | `2.5px` | the keyboard focus outline |
   | `--dpr` | `1` | `1.25` / `1.5` / `1.75` | `2` | a `calc()` that needs the scale |

   Write `border: var(--hair) solid var(--line)`, `height: var(--hair)`, never `1px`.
   `Rule::RawHairline` (Strict, section 5) flags a literal `1px`/`.5px` border or outline width,
   and a box whose whole `width`/`height` is `1px`, and names the token to use.
3. **The layout is snapped to the device grid.** Blitz rounds every box to whole *logical*
   pixels, so at 1.5 a box at y 11 starts at device y 16.5 whatever its width.
   `ds_native::snap_to_device(&mut BaseDocument)` re-rounds the laid-out document on the device
   grid (and rounds a pure translation to whole device pixels). `Harness` and `snapshot` run it
   every frame. **A host that resolves its own documents (shell-host) calls it after every
   `resolve` and before painting**; it does nothing at a whole scale. `ds_native::launch`'s
   window cannot (blitz-shell resolves and paints in one call), so there the tokens apply but a
   line may still sit half a device pixel off.

Glyphs need nothing from you: under a root at a fractional scale `Glyph` writes a stroke width
that is an even number of device pixels (design/08-ICONS.md §1.4.1).

### A document's frame must have a height (2026-09-25)

One rule for every surface whose document is a frame around a `Ds` root: a popup's frame, an
overlay root, a wallpaper, the launcher's catcher. **The frame (the
document's first box, the one around the `Ds`) is in normal flow and has a height; it is never
absolutely or fixed positioned, and never left to be sized by content that is itself out of
flow.** Blitz lays out `#main` and `.ds` at `height:auto`: a frame with `position:absolute;
inset:0` or `position:fixed` resolves against that 0 px box (Taffy places a fixed box against
its parent, not the viewport), a `height:100%` resolves against it too, and a frame whose only
content is a card hung from its anchor is 0 px tall. Then every box from `html` down is 0 px,
nothing paints and in the shell no press lands.

- **The surface fills its surface** (popup frame, overlay, wallpaper, catcher): pass
  `Ds { extent: RootExtent::Viewport }`, or put the floor on your frame: `display:grid; width:100vw; min-height:100vh` in flow, the
  room around the card as the frame's padding; the grid's one cell stretches the `Ds` root to
  fill it, so the root has the viewport's height too. The same floor holds even if the frame is
  absolutely placed (`crates/ds-native/tests/root_frame.rs` measures all three).
- **The surface is sized by its content** (a bar, a dock, an OSD card's surface): leave the
  content in flow; a card that must be positioned is positioned inside a frame that has a height.
- **Debug it** with the Harness before looking at the compositor: `harness.rect("html")`,
  `harness.rect(".ds")` and your frame's rect; a height of 0 on any of them is this bug.
  `harness.count(".ds")` confirms the root mounted at all, and `harness.hits(point, selector)`
  tells you whether a press at `point` reaches your card. (Whether Blitz's hit test enters a 0 px
  box has differed between the shell and the harness, so trust the heights, not a click.)

You almost never write more than one `Ds` per window: it is the root, not a per-panel wrapper —
use `Surface` (section 4) for a nested material, scheme, accent or blur state.

**Only one `Ds` prop most consumers get wrong first:** `appearance: Appearance::default()` is
fine for a first cut, but it means "System theme, Postmark accent, System motion" every time,
ignoring whatever the person picked last session. Section 3 below is how you read the real
value.

## 3. Reading appearance

### `ds_settings::use_environment`

`ds_settings::use_environment(app: AppName) -> ReadSignal<Environment>` (`ds-settings/src/
environment.rs`) is the one hook that gives you a live `Environment { settings: AppearanceFile,
system: SystemPrefs }`: it loads `appearance.toml` (importing mailo's `appearance.json` once
when the app's own `appearance.toml` does not exist yet, for every app, mailo included;
design/22-SETTINGS.md section 2 "Mailo migration"), watches the directory for edits (30 ms
debounce), reads the freedesktop settings portal, and watches it for changes — merging both into
one signal.

```rust
use ds_settings::{AppName, use_environment};
use ds::{Ds, Material};
use dioxus::prelude::*;

#[component]
fn App() -> Element {
    let env = use_environment(AppName("your-app-id"));
    let environment = env();
    rsx! {
        Ds {
            appearance: environment.settings.appearance.appearance(),
            system: environment.system,
            material: Material::Window,
            YourPage {}
        }
    }
}
```

**`use_environment` needs an entered Tokio runtime, in both halves, not only the obviously
networked one** — and on Blitz, `ds_native::launch` and `ds_native::Harness` provide it, so
there is nothing for you to do:

- the desktop-portal half (`SystemPrefsWatch`, `ds-settings/src/portal.rs`) goes over D-Bus
  through `zbus`'s `tokio` feature and calls `tokio::spawn` directly (`portal.rs`'s
  `linux::spawn_watch`);
- the file-watch half is not exempt either: `ds_settings::watch` (`ds-settings/src/watch.rs`)
  calls `tokio::spawn` directly too, to debounce and coalesce `notify` events
  (`tokio::time::timeout(DEBOUNCE, ...)` inside that spawned task), so it needs a runtime just
  as much as the portal does — only the plain, one-shot `ds_settings::load`/`load_or_import`
  (a synchronous `std::fs::read`) needs none.

Neither `ds` nor `ds-settings` may depend on a renderer or a windowing stack
(`scripts/check-boundary.sh`), so neither can own a *host thread* to enter a runtime on — only a
host crate can, and `ds-native` is quire's one host crate. `ds-native` owns a process-wide,
lazily built Tokio runtime (multi-thread, two workers; `crates/ds-native/src/runtime.rs`):
`ds_native::launch` enters it and holds the guard for the rest of the call, i.e. for the
process's life, and `ds_native::Harness` enters it in `Harness::new` and holds the guard as a
field, for the harness's own life. Call `use_environment` (or `ds_settings::watch` on its own)
from anything launched with `ds_native::launch`, or rendered inside a `ds_native::Harness`, and
it works — `examples/consumer::App` does exactly this
(`examples/consumer/src/lib.rs::App`), and `crates/ds-native/tests/harness.rs::
use_environment_does_not_panic_under_the_harness` is the regression test.

### `use_scope` — reading the resolved scope

Inside any component under a `Ds`, `ds::use_scope() -> Scope` gives you what that scope resolved to:
`resolved: Resolved{scheme, accent, motion}`, `scheme`, `material`, `blur`, `modality`
(`style/scope.rs`). Components use this to size icons, choose overlay placement and read the motion
level for their own timers (section 6); you will rarely need it directly unless you are building
your own component.

## 4. `Surface` — a nested material, scheme, accent or blur state

A subtree in a different `Material`, or forced to a scheme, accent or blur state other than the
root's (a popover over a dark card, an always-light preview pane, a specimen in another accent)
is a `Surface`, not a second `Ds`: no stylesheet, no frame layers, just a nested `div.ds` that
re-stamps `data-theme`/`data-typeface`/`data-accent`/`data-motion`/`data-material`/`data-blur`
(the typeface is always the root's) and updates the
`Scope` every component under it reads.

```rust
use ds::{Accent, BlurState, Material, Scheme, Surface};

rsx! {
    Surface { material: Material::Popover, theme: Some(Scheme::Dark),
        YourPopoverContent {}
    }
    Surface { material: Material::Widget, accent: Some(Accent::Green), blur: Some(BlurState::Unavailable),
        YourSpecimen {}
    }
}
```

| Prop | Type | Default | What it does |
| --- | --- | --- | --- |
| `material` | `Material` | required | the material this subtree paints |
| `theme` | `Option<Scheme>` | `None` | force a scheme; `None` inherits the enclosing scope's |
| `accent` | `Option<Accent>` | `None` | force an accent; `None` inherits |
| `blur` | `Option<BlurState>` | `None` | force the blur state (`Unavailable` paints the solid tint); `None` inherits |
| `on` | `Option<Ground>` | `None` | the ground its content is drawn on; `None` is the material's (`Frame` for Bar and Dock, paper otherwise), so a paper panel inside a bar root is paper again |
| `radius` | `Option<Corner>` | `None` | the material's corner (`--m-radius`), overridden: `Corner::Px(Px(f32::from(dock.pill_radius_px.0)))` or `Corner::Token(Radius::Panel)` |

Each `None` inherits, so the common case is the material alone (`crates/ds/tests/surface.rs`
has a golden per override).

## 5. The four coherence rules

`ARCHITECTURE.md#repo-rules` states the coherence rules every downstream crate
enforces on itself. Each one below is the exact test a consumer adds — `examples/consumer/
tests/coherence.rs` is these four, verbatim, run against a real app.

### Rule 1 — no literal design values in your own CSS

```rust
use ds::lint::{assert_clean, Exception, LintConfig, Profile, Rule};

const OUR_CSS: &str = ".row { color: var(--ink); }\n.fade { mask-image: linear-gradient(#000, transparent); }";
const EXCEPTIONS: &[Exception] = &[Exception {
    rule: Rule::HexColour,
    selector: ".fade",
    reason: "a mask's alpha, never painted",
}];

#[test]
fn our_css_is_clean() {
    assert_clean(OUR_CSS, &LintConfig {
        profile: Profile::Strict,   // Standard also allows raw font-size/radius/z-index; use Strict
        exceptions: EXCEPTIONS,
        ..LintConfig::new(&ds::kits())
    });
}
```

Every exception needs a `reason`, and `assert_clean` panics listing which exceptions suppressed
zero offences, so a stale one cannot hide silently (mailo gaps 3: before it, it only printed
them). A consumer mid-migration that must keep an exception for code another branch is about to
land sets `stale: Stale::Report` (`ds::lint::Stale`): the stale exception is printed and the
test passes. `Stale::Fail` is the default. A `LintConfig` is made from the kits it lints against (`LintConfig::new(&ds::kits())`), which
give it the variables, keyframes and timing tokens it accepts; name only the fields you change
and end with `..LintConfig::new(&ds::kits())`. Prefer `Profile::Strict` even though
`Profile::Standard` is the default: quire's own `self_lint` test runs Strict, and a raw
`border-radius`/`font-size`/`z-index` your CSS writes is exactly the kind of drift the token
table exists to prevent.

Strict also runs `Rule::RawSpacing`: a literal `px` in `margin`, `padding` (their sides and
logical forms included) or `gap`/`row-gap`/`column-gap` is an offence; `0`, `auto`, a
percentage, an `em` and `var()` pass. The steps are `ds::SpacingToken`, emitted on `.ds` as
`--s-1`, `--s-1-5`, `--s-2` … `--s-12`, `--s-13`, `--s-14`, `--s-15`, `--s-16`, `--s-18`,
`--s-22`, `--s-26`, `--s-36`, each named by its pixel value (design/01-LAYOUT.md §2). A length
between steps takes the nearest one; quire's own sheets and the gallery's do (FINDINGS "Polish
pass"). `examples/consumer/src/style.css` is Strict-clean.

`Rule::UnknownAnimation` reads the `animation` shorthand as well as `animation-name` (mailo gaps
3): in each comma-separated animation the name is the first identifier that is not a keyword
(an easing, `infinite`, a direction, a fill mode, a play state, a CSS-wide keyword), with times,
counts and functions skipped. `animation: sparkle 1s` is an offence now; a name only a `var()`
holds, or one written as a string, is not judged.

Strict also runs `Rule::RawHairline` (2026-09-25): a literal hairline (`1px`, `.5px`) as a
`border*` or `outline*` width, or as a box's whole `width`/`height`, is an offence whose text
names the token (`border: 1px (use var(--hair))`, `.5px` points at `var(--hairline)`). A 2 px
border and a `border-radius: 1px` pass: stylo already floors a border width to whole device
pixels, and only a line of one pixel or less goes blurry. sill's `.sill-dock-separator`
(`width:1px`) is the one offence known downstream; it becomes `width: var(--hair)`.

### Rule 2 — no raw markup, only quire components

Two tests, both against an SSR render of a real page (never a hand-built HTML string — the
markup lint runs on what your app *actually renders*, not on what you believe it renders):

```rust
use dioxus::core::VirtualDom;
use ds::lint::{markup, LintConfig, Rule};

fn render_ssr() -> String {
    let mut dom = VirtualDom::new(YourApp);
    dom.rebuild_in_place();
    dioxus_ssr::render(&dom)
}

fn full_css() -> String {
    format!("{}\n{}", ds::stylesheet(), YOUR_CSS)   // both — markup does not add ds::stylesheet() itself
}

#[test]
fn markup_lint_is_clean() {
    let offences = markup(&render_ssr(), &full_css(), &LintConfig::new(&ds::kits()));
    assert!(offences.is_empty(), "{offences:#?}");
}

#[test]
fn no_raw_button_is_rendered() {
    let offences = markup(&render_ssr(), &full_css(), &LintConfig::new(&ds::kits()));
    assert!(offences.iter().all(|o| o.rule != Rule::RawMarkup), "{offences:#?}");
}
```

`markup` flags three different things and the tests above catch all of them, but name them
separately because they read differently in a failing CI log: `Rule::UnstyledClass` (a class
nothing in scope styles — usually a typo or a stylesheet you forgot to concatenate),
`Rule::RawMarkup` (a hand-written `<button>`/`<input>`/`<select>`/`<textarea>` with no `ds-`
class, or an `<svg>` that is neither `Glyph`'s `.ds-ic` nor marked `data-ds-svg`, the attribute
a quire component that draws its own vector writes, as the SendPill's ring does), and the
inline-style rules below.

Every `style` attribute is checked declaration by declaration (`ds::lint`'s `inline_style`): a
literal colour (`Rule::HexColour`, `Rule::ColourFunction`, `Rule::NamedColour`) or a raw
duration (`Rule::RawDuration`) is an offence. The one allowance is a custom property (`--*`) on
an element carrying `ds` or a `ds-*` class: that is quire handing a value it computes per
instance to its own stylesheet (the Space's `--f-*` frame and `--f-grad`, an avatar's `--av-bg`,
a provider mark's `--pc`), so quire's own markup lints clean with no exception. The same
literal in a non-custom property (`background:#fff`) is an offence on any element, and a custom
property on your own element is one too: put your values in your stylesheet as tokens.
`examples/consumer/tests/coherence.rs::no_raw_form_control_or_svg_is_rendered` filters to
`Rule::RawMarkup` alone, so its name says exactly what regressing it means: someone wrote a bare
`button`/`input`/`svg` where a quire component belongs. Filtering the same call two different
ways, rather than one `offences.is_empty()` assertion, is what keeps a future failure legible —
"unstyled classes" and "raw markup" are different mistakes with different fixes.

### Rule 3 — nothing missing is patched locally

Not a runtime test: if a page needs a component, token or motion value quire does not have, stop
and add it to quire first (or ask quire's maintainers to), never draw it by hand in your own
crate. `examples/consumer` never needed this escape hatch; note in your own report if you do.

### Rule 4 — motion only through quire's own timers

Never `std::thread::sleep`, `tokio::time::sleep` or a hand-rolled `setTimeout`-equivalent to
drive a class toggle. Use `ds::use_pulse` (restart a keyframe: `Pulse::fire()`) or
`ds::use_motion_timer` (`MotionTimer::start(on_settled)`, which runs for exactly
`ds::settle(anim, level, index)`); both read the enclosing `Ds`'s resolved motion level, so
`MotionLevel::Reduced` collapses them automatically. Prove it with `ds_native::Harness`, which
drives a real Blitz document on a real (if fast-forwarded) clock — the test below is
`examples/consumer/tests/coherence.rs::the_sent_badge_times_out_on_ds_motions_own_clock`,
shortened:

```rust
use ds::{resolve, settle, Anim, Appearance, SpaceLook, StaggerIndex, SystemPrefs};
use ds_native::{Harness, Viewport};
use std::time::Duration;

let resolved = resolve(Appearance::default(), SpaceLook::default().theme, SystemPrefs::default());
let hold = settle(Anim::Fade, resolved.motion, StaggerIndex::default()); // never a millisecond literal

let mut harness = Harness::new(YourApp, Viewport { width: 480, height: 360, scale_percent: 100 });
// ...trigger the state change...
harness.advance(hold - Duration::from_millis(10));
// assert it has NOT settled yet
harness.advance(Duration::from_millis(20));
// assert it HAS settled now
```

Computing `hold` through `ds::settle` rather than copying today's millisecond value is what makes
this a real test of rule 4: a `Duration::from_millis(1200)` literal would still pass if `Anim::
Fade`'s token were retuned tomorrow, which is exactly the drift `ds::motion` is supposed to make
impossible.

**Exact timing on a loaded machine: the virtual clock.** The harness above runs
quire's timers on the wall clock, so `advance(hold - 10ms)` can overshoot the boundary under
load. Build it on the virtual clock instead and `advance` moves one clock that drives both the
CSS animations and every ds timer (motion timers, presence and roster rests, hover intent, toast
holds, pending, detail tweens, menu submenu delays): it steps to each timer's due instant in
order, runs the renders that queued and resolves the CSS at that same instant, and returns at
once. The two assertions above then hold exactly, every run:

```rust
use ds_native::{Clock, Harness, HarnessConfig, Viewport};

let config = HarnessConfig::new(Viewport { width: 480, height: 360, scale_percent: 100 })
    .with_clock(Clock::Virtual);
let mut harness = Harness::with_config(YourApp, config);
```

| Want | Call | Notes |
| --- | --- | --- |
| Timers on the harness's clock | `HarnessConfig::with_clock(Clock::Virtual)` | Default `Clock::Wall` (unchanged). `Harness::clock() -> Clock` |
| "Now" in a test | `Harness::now() -> Instant` | The virtual clock's now (or the wall clock's); `settle_until` returns instants on the same clock, and its 3 s bound is the harness's time. On the virtual clock, time a window from `harness.now()`, never `Instant::now()` |
| Read the time in your own component | `ds::time::now()`, `ds::time::since(instant)`, `ds::sleep(d)` | Whatever clock the thread has installed: the wall clock in a window, the harness's in a test. A component that calls `Instant::now()` or `futures_timer` itself stays on the wall clock and drifts from the harness |
| Install a virtual clock yourself (another harness) | `ds::VirtualClock::new()`, `.install() -> ClockGuard`, `.advance_to(d)`, `.next_due()`, `.now()`, `.elapsed()`, `.waiting()`, `.due_times()` | Thread-local, restored when the guard drops. Step through `next_due` and poll your executor between steps, as `Harness::advance` does |

Not on the virtual clock: work off the harness's thread (a Tokio task such as `ds_settings`'
file watch, a D-Bus reply, a resource fetched by a custom `AppNet` on another thread).
`advance` on the virtual clock never waits for the wall clock, so a test that needs such work
to land stays on `Clock::Wall`. Nor Blitz's own clock reads (rev e99fbdbd): its double-click
count (a press within 500 ms and 2 px of the last) and scrollbar fade read `Instant` in fields a
host cannot set, so on the virtual clock two clicks at one spot are always a double click, however
far apart the test advanced them; click a second spot in between, or keep that test on the wall
clock.

**`assert_settles_to_zero_frames` is only a strict check on `Clock::Virtual`.** It drains
`VirtualClock::next_due` — advancing straight to each pending sleep's due instant, earliest
first, until none remain — then asserts a quiet window right after, so "at rest" means exactly
design/26 R3: no CSS animation, no pending `ds` timer, nothing woke the document. On
`Clock::Wall` it cannot see what is pending, only what just happened, so it instead polls in
`QUIET` windows and returns as soon as *one* window is quiet; a timer that started later in the
same test (a 900 ms hold begun at the top of a window shorter than 900 ms) can still be pending
when it returns. Use `Clock::Virtual` for any test that needs the strict guarantee, not the poll.

## 6. The component catalogue

Every component is `ds::<Name>`, one `.rs`/`.css` pair per `design/04-COMPONENTS.md` section
(`DESIGN.md` "`ds`: components"). One worked example per family below; the full list is the
table after it.

### Controls

```rust
use ds::{Button, ButtonVariant};

rsx! {
    Button {
        variant: ButtonVariant::Primary,
        label: "Send".to_owned(),
        icon: Some(ds::Icon::Send),
        onclick: move |_| send(),
    }
}
```

### Lists

```rust
use ds::{ListRow, Selection, Emphasis, StaggerIndex, Presence, PulseKey};

rsx! {
    ListRow {
        selection: Selection::Unselected,
        emphasis: Emphasis::Plain,
        index: StaggerIndex::new(0),
        presence: Presence::Present,
        name: "Ada Lovelace".to_owned(),
        via: None,
        subject: "Re: the analytical engine".to_owned(),
        snippet: "I have translated the memoir...".to_owned(),
        time: "2:14 PM".to_owned(),
        tags: rsx! {},
        star: None,
        star_pulse: PulseKey::rest(ds::Anim::StarPop),
        strip: None,
        onclick: move |_| open_thread(),
    }
}
```

### Overlays

```rust
use ds::{Anchor, Button, ButtonVariant, Common, Menu, MenuKind, MountedRef, Shown, UndoToken, use_toasts};

let toasts = use_toasts();
toasts.push("Sent".to_owned(), None);   // ToastHost is already rendered by Ds — nothing else to mount
// With an undo: the handler of the toast on screen runs when the person undoes it.
toasts.push_undoable("Archived".to_owned(), UndoToken(7), EventHandler::new(move |token| restore(token)));

// A menu anchored to the button that opens it: the button hands over its own element.
let mut open = use_signal(|| Shown::Hidden);
let mut more = use_signal(|| None::<MountedRef>);
rsx! {
    Button {
        variant: ButtonVariant::Secondary,
        label: "More".to_owned(),
        onclick: move |_| open.set(Shown::Visible),
        common: Common {
            mounted: Some(EventHandler::new(move |event: MountedEvent| more.set(Some(MountedRef(event.data()))))),
            ..Common::default()
        },
    }
    if let (Shown::Visible, Some(button)) = (open(), more()) {
        Menu { kind: MenuKind::Rich, anchor: Anchor::Mounted(button), entries, onpick, onclose: move |()| open.set(Shown::Hidden) }
    }
}
```

`examples/consumer/src/lib.rs::Page` is this pattern, whole.

### Frame

```rust
use ds::{Appearance, AppearancePicker, SystemPrefs};

rsx! {
    AppearancePicker {
        value: current_appearance,
        system: SystemPrefs::default(),
        onchange: move |next: Appearance| save(next),
    }
}
```

Full catalogue (design doc section in parentheses):

| Family | Components |
| --- | --- |
| Controls | `Button` (§1), `IconButton` (§2), `SegmentedControl<T>` (§3), `Toggle` (§4), `Slider` (§5), `TextInput` (§6), `SearchField` (§7), `CommandPill` (§8), `Kbd` (§9), `Chip` (§10), `Avatar` (§11), `Tabs<T>` (§12), `SectionHeader` (§13), `Count` (§14), `Spinner` (§15) |
| Lists | `ListRow` (§16), `HoverStrip` (§17), `SidebarItem` (§19), `AnimatedList` (§16) |
| Overlays | `Tooltip`/`HoverTarget`/`HoverCard` (§18, §22), `Menu`/`MenuEntry` (§20), `Popover` (§21), `Toast`/`use_toasts` (§23), `Scrim`/`Sheet`/`Peek` (§24), `CommandPalette<T>` (§25) |
| Frame | `AppearancePicker` (§26), `AccountTile` (§27), `ProviderMark` (§28), `LinkPill` (§29), `SelectionBubble` (§30), `SendPill` (§31), `SpaceEditor` (§32), `EdgeStrip` (§33), `DragGhost` (§34), `SyncHalo` (§35) |

Every component's exact props are its own `#[component] pub fn` signature in
`crates/ds/src/components/<name>.rs` — read that, not this table, before wiring one up; this doc
only orients you to which family a component is in and where its design spec lives.

A few props worth knowing about before you read the signatures:

- `TextInput` takes `#[props(default)] focus: Focus` — `Focus::OnMount` focuses it as soon as it
  mounts (the command palette's input, a bubble's link field); the default, `Focus::Manual`, is
  what every other field wants; `Focus::Controlled(request)` focuses it on mount and again at
  every `request.request()` (the launcher gaps, below).
- `ListRow` and `SidebarItem` take `#[props(default)] drop: DropState` (`Idle`, `Target`,
  `Source`) — drag-and-drop visual state (design/04-COMPONENTS.md §34); leave it `Idle` unless
  you are wiring up drag and drop for that row.
- `SpaceEditor` takes two more optional props: `name: Option<String>` (the Space's own name
  field, drawn inside the editor rather than beside it) and
  `on_active_dot: Option<EventHandler<ActiveDot>>` (fires when the person's focus moves to a
  different dot, separately from `onchange`, which fires on an actual edit).
- `AccountFace::One` gained an `address: Option<String>` field — an `AccountTile`'s
  accessible name falls back to its letter and provider without one; pass the account's address
  when you have it.
- `SelectionBubble`'s `BubbleAction` is now `{Button(BubbleButton), Separator}` rather than a
  bare list of buttons — insert `BubbleAction::Separator` between groups instead of styling a
  gap yourself.

Anchors, hover-card parts and undo:

- `Button` and `IconButton` take `mounted: Option<EventHandler<MountedEvent>>`: the element
  itself, for `Anchor::Mounted` (the Overlays example above). It writes no attribute.
- `HoverCard` takes `parts: Vec<HoverCardPart>` (`Title`, `Sub`, `Person`, `Stats`, `Flag`,
  `Messages`, `Foot`, `Actions`), drawn in order before its children: no hand-written
  `ds-hovercard-*` markup.
- `ToastHub::push_undoable(text, token, on_undo)` calls `on_undo` with the token when the person
  undoes that toast; a later push replaces it. The handler belongs to the scope that made it,
  so that scope must outlive the toast. `last_undo()` still reports the last undo.
- `Surface` takes `accent` and `blur` beside `theme` (section 4).

External icons and a caller-driven tooltip (FINDINGS "Pointer events"):

- **External icons.** An icon slot takes `ds::IconSource`: `Glyph(Icon)`, `Symbolic(ExternalIcon)`
  or `Image(ExternalIcon)`. `ExternalIcon { url: IconUrl, size: IconSize }` is a `data:` or
  `file:` URL and the square size it is drawn at. Build the URL with `IconUrl::png(&bytes)` (a
  tray pixmap you have PNG-encoded; quire does the base64), `IconUrl::svg(&document)`,
  `IconUrl::file(&absolute_path)` (an icon theme lookup), or `IconUrl::parse(url)`, which refuses
  any other scheme (`DsError::IconScheme`). `Symbolic` paints the icon's alpha in the text colour
  (`mask-image` over `currentColor`), so it follows `--ink`, hover, pressed and `--f-ink*` exactly
  like a glyph; `Image` shows the bitmap as it is. Which to use is design/08-ICONS.md §1.5's rule
  (freedesktop `*-symbolic`, or a pixmap whose opaque pixels all have OKLCH chroma < 0.04, is
  symbolic; anything coloured is an image) and is the caller's decision. `IconButton { icon }`
  and `Button { icon }` take an `IconSource`, and an `Icon` (or `Option<Icon>` for `Button`)
  still converts, so existing call sites are unchanged. `ds::IconView { source, size }` draws one
  anywhere else. The URL loads through the document's net provider (ds-native and shell-host's
  `LocalNet` answer `data:` and `file:`), one frame late.

  ```rust
  let icon = ds::IconSource::Symbolic(ds::ExternalIcon {
      url: ds::IconUrl::file(&theme_path)?,
      size: ds::IconSize::Base,
  });
  rsx! { IconButton { variant: IconButtonVariant::Tool, icon, label: title, onclick } }
  ```
- **Pointer buttons and ids.** `Button` and `IconButton` take `id: Option<String>`, written as
  the element's `id` (a popup anchors to `tray-3` with no wrapper span). Their `onclick` is
  `EventHandler<ds::Press>`, `Press { button: PointerButton::{Primary, Secondary, Middle},
  modifiers }`: a right-click (which Blitz and browsers deliver as `contextmenu`, never as a
  click; its default is prevented) arrives as `Secondary`, the middle button as `Middle`, and
  Enter or Space on the focused control as `Primary`. A closure written `move |_| ...` compiles
  unchanged and ignores the button; so does an `EventHandler<()>` you already hold (passed
  straight through). A closure written `move |()| ...` does not: write `move |_|`.

  ```rust
  onclick: move |press: Press| match press.button {
      PointerButton::Secondary => open_menu(),
      PointerButton::Primary | PointerButton::Middle => activate(),
  },
  ```
- **Menu submenus and disabled items.** `MenuEntry::Item` gained `availability: Availability`;
  a disabled item is drawn at .35 opacity with `aria-disabled="true"`, skipped by Up and Down,
  and a click on it does nothing. `MenuEntry::Submenu { title, tile, availability, children }`
  is a row with a chevron that opens `children` beside the menu on a 200 ms rest, or at once on
  Right, Enter or a click; Left or Escape closes one level, and a picked child's value reaches
  the menu's `onpick` (the whole menu closes first). Submenus nest to any depth. The timing is
  the `timing: MenuTiming` prop (default 200 ms delay, 300 ms safe-triangle timeout; read
  `menus.submenu_delay_ms` into it), driven by the same `MenuTrack` machine as the bar's menus.
  `expanded: Option<usize>` opens a choice's submenu as the menu mounts (choices count items and
  parents, not headers or rules). A submenu is placed in the same document as its menu, so on a
  shell surface whose popup is sized to the menu, the popup must leave room for it.

For a bar (FINDINGS "Bar gaps"):

- **A Blitz host that is not `ds_native::launch`** (shell-host's surfaces, a popup's document)
  calls `ds_native::measure::provide()` at the top of its root component, before any quire
  component reads a rect; `ds_native::measure::MEASURE` is the same value for
  `use_context_provider`. Drop your own copy of the twelve-line measurer. Without one, a rect
  read no longer panics: `ds::use_rect` and every anchor treat a held document as busy and try
  again next frame (the default panic hook still prints the collision, so provide it).

  ```rust
  #[component]
  fn BarRoot() -> Element {
      ds_native::measure::provide();
      rsx! { Ds { appearance, material: Material::Bar, look, /* … */ } }
  }
  ```
- **Chrome materials draw the Space.** A `Ds` in `Bar`, `Dock`, `Osd`, `Widget` (and a
  `Popover` root passed `chrome: Some(RootChrome::Painted)`) draws the Space gradient, its A/B
  layers and grain as one `.ds-frame` group at the material's tint alpha
  (`--m-frame-alpha`, scaled by `appearance.material_tint_alpha`) with `data-blur=on`, and at
  the solid floor .94 without blur; a `look` change cross-fades it over `--t-scene`. Pass the
  workspace's `SpaceLook` as `look` and paint nothing yourself: delete any tint layer of your
  own. The root becomes a stacking context (`position:relative`, `z-index:var(--z-raise)`).
- **Popup roots are transparent.** A `Popover`, `Sheet` or `Toast` root paints no tint and no
  shadow on its own box (`data-chrome="transparent"`); the `.ds-popover`/`.ds-menu` and
  `.ds-sheet` cards inside paint the material's tint (`--m-tint` over blur, `--m-tint-solid`
  without), edge and drop (`--m-box`). A popup document keeps `Material::Popover` and its
  spare room is alpha 0 (`crates/ds-native/tests/bar_frame.rs` proves it over
  `Harness::render_over(Backdrop::Clear)`).
- **The frame ground.** Under `data-ground="frame"` (a Bar or Dock root, or `Surface { on:
  Some(Ground::Frame) }`) `--ink`, `--ink-soft`, `--ink-faint` are the Space's `--f-ink*`,
  `--surface` is `--f-pill-hover`, `--surface-2` and `--raise` are `--f-pill`, `--line*` is
  `--f-line`: `Button`, `IconButton`, `Chip`, `Count`, a menu's trigger and your text all draw in
  the frame inks with no variant of their own. Overlays opened from it (menus, popovers,
  tooltips) are paper again.
- **Status items.** `IconButton { variant: IconButtonVariant::Status, .. }` is a square of
  `--bar-status-box` holding its glyph (or external icon) at `--bar-status-glyph`,
  `--f-ink-soft` at rest, `--f-ink` on `--f-pill-hover` under the pointer, `--f-pill` when
  `pressed` or `expanded` is `On`. Write the two properties on any element around your items
  with `ds::StatusMetrics`, filled from your settings:

  ```rust
  let metrics = StatusMetrics {
      box_size: Px(f32::from(bar.status_icon_box_px.0)),
      glyph: Px(f32::from(match bar.glyph_size_policy {
          BarGlyphSize::StatusIcon16 => bar.status_glyph_px.0,
          BarGlyphSize::IconSizeBar22 => bar.status_icon_box_px.0, // the glyph fills the box
      })),
  };
  rsx! { div { class: "status", style: metrics.style_attr(), /* IconButton { Status } … */ } }
  ```
  (`style_attr()` is `--bar-status-box:22px;--bar-status-glyph:16px;` at the defaults; custom
  properties with lengths on your own element pass the markup lint.) An external icon in a
  status item is sized by the property too, whatever its `ExternalIcon::size`.
- **Menu control.** `Menu` gained `on_hover: Option<EventHandler<Option<usize>>>` (the choice
  under the pointer, numbered as `expanded` counts; `None` once it is over none),
  `on_release: Option<EventHandler<Press>>` (every button released over the menu),
  `entrance: MenuEntrance::{Animated, Instant}` (a bar menu and a hover switch pass `Instant`)
  and plays `Anim::MenuOut` (`--t-quick --e-exit`, `data-presence="leaving"`) before `onclose`
  when Escape or an outside click closes it. A pick calls `onpick`, then `onclose`, once. A
  release over an enabled item after a press that began outside the menu picks it
  (press-drag-release); over a disabled item, a header or the padding it closes picking
  nothing. The owner removing the menu (a hover switch) is immediate, no fade.
- **Status lines.** `MenuEntry::Info { title, detail: Option<String> }` is a row at an item's
  weight without the header's eyebrow, never a choice: keys, hover and picks pass it by. Use it
  for a network's address or a battery's time left, instead of headers.
- **`Press { button, modifiers, at }`**: `at` is the surface-local point (the event's client
  point); a keyboard activation reports the origin. `Press` is `PartialEq` only now. Hand
  `at.x`, `at.y` (converted to screen coordinates if you know the surface's origin) to SNI's
  `Activate` and `ContextMenu`.
- **Symbolic or image.** `ds::icon::classify(&png_bytes) -> Result<IconKind, DsError>`
  (`IconKind::{Symbolic, Image}`) is design/08-ICONS.md §1.5 step 2: symbolic when every pixel
  with at least half coverage has OKLCH chroma below 0.04; `classify_with(&png,
  ChromaLimit(n))` takes another threshold in thousandths. Pick `IconSource::Symbolic` or
  `Image` from it; `NeedsAttention` still recolours through the parent's colour (`--warn`). The
  0.04 default is `ChromaLimit::default()`; a settings value (design/22-SETTINGS.md §3.3
  `icons.symbolic_chroma_max`, plain chroma, not thousandths) goes through
  `ChromaLimit::try_from(value)`, which refuses anything outside `0.0..=0.2`
  (`DsError::ChromaLimitRange`). No caller reads the key yet — that is on the settings-owning
  side.
- **New glyph**: `Icon::Ethernet` (Lucide `ethernet-port`) for a wired network.

For a launcher and a dock (FINDINGS "Launcher gaps"):

- **Unmounting early is safe.** Every task quire spawns (a motion timer's settle, a roster's
  exit and rest, the hover hub's and the toast hub's timers, a focus retry) is a task of the
  scope that owns it and is dropped with that scope, and writes only through fallible handles.
  A palette, menu or popover may be unmounted at any moment, mid entrance included: drop any
  keep-mounted guard you added for it. `MotionTimer::start`'s `on_settled` never runs for a
  component that is gone.
- **Focus waits out a busy document.** `Focus::OnMount`, a menu taking the keyboard and every
  other focus change go through `ds::HostFocus` (`Focused::{Done, Busy, Unknown}`), tried again
  a frame later while the renderer holds the document; with no host seam the call is guarded
  and a collision is busy too, never a panic. A Blitz host that is not `ds_native::launch`
  provides it beside the measurer:

  ```rust
  #[component]
  fn LauncherRoot() -> Element {
      ds_native::measure::provide();
      ds_native::focus::provide();
      rsx! { Ds { appearance, material: Material::Sheet, look, /* … */ } }
  }
  ```
- **Giving a field the keyboard back.** `let field = ds::use_focus_request();` then
  `TextInput { focus: Focus::Controlled(field), .. }` (or `CommandPalette { focus: Some(field),
  .. }`), and `field.request()` from a handler (a menu's `onclose`) whenever the field should
  have the keyboard again. The field takes it as it mounts and at each request; nothing is
  remounted, so a palette does not replay its entrance.
- **An embedded palette.** `CommandPalette { host: CommandPaletteHost::Surface, .. }` draws no
  scrim and no overlay: the card fills its container (give the container the panel's size) and
  paints the enclosing material's tint, edge and radius-14 corner (`--r-panel`), its list taking
  the height under the field. `id: Some("…")` goes on the card, for a blur region.
  `entrance: PaletteEntrance::{PeekIn, CmdkIn}` picks the card's entrance. `Overlay` (the
  default) is unchanged. Escape in the field still closes it through the layer stack, so a
  menu opened over it takes Escape first.
- **The palette's selection.** `selected: Option<usize>` makes it the caller's (choices counted
  as a menu's `expanded` counts them: items and submenu parents, not headers); Up, Down and
  the pointer then only ask through `on_select`. Left to the palette, `on_select` hears every
  change of the selection, the reset to the first choice on a new query included.
  `on_select_rect: Option<EventHandler<Rect>>` hears the selected row's rect (client
  coordinates, read through the measurer a frame after layout) whenever the selection, the row
  under it or the results (groups, tokens) change: anchor an actions menu with
  `Anchor::Rect(rect)`. It is never an empty rect: a row read before its surface's first layout
  is read again each frame until it has an area (palette follow-ups, below). `onkey:
  Option<EventHandler<KeyboardEvent>>` hears every key the field gets after the palette has read
  it (Tab, Ctrl+K), as the event itself, so nothing needs to listen at your root; call
  `event.prevent_default()` on a key you take. The pointer moving over a row selects it.

  ```rust
  let mut row = use_signal(|| None::<Rect>);
  let field = use_focus_request();
  rsx! {
      CommandPalette::<Hit> {
          label, placeholder, query, tokens: Vec::new(), groups, empty, oninput, onpick, onclose,
          host: CommandPaletteHost::Surface,
          entrance: PaletteEntrance::CmdkIn,
          id: "launcher-card".to_string(),
          focus: field,
          on_select_rect: move |rect: Rect| row.set(Some(rect)),
          onkey: move |key: KeyboardEvent| if is_actions(&key) {
              key.prevent_default();
              actions.set(true);
          },
      }
      if let (true, Some(rect)) = (actions(), row()) {
          Menu { kind: MenuKind::Rich, anchor: Anchor::Rect(rect), entries, onpick,
                 onclose: move |()| { actions.set(false); field.request(); } }
      }
  }
  ```
- **App icons in rows.** `Tile::Source(IconSource)` puts any icon in a menu or palette row: an
  `Image` (an app's icon file) fills the tile with no plate under it, whatever size it was
  resolved at; a `Symbolic` sits on the plate in the text colour; a `Glyph` is `Tile::Icon`'s.
- **Icon sizes for tiles.** `IconSize::Tile48` (48), `IconSize::Tile96` (96) and
  `IconSize::Px(IconPx(n))` for any size a caller resolves (a magnified dock tile): an icon is
  drawn at that size, not scaled from 22.
- **A caller-driven tooltip.** `Tooltip { shown: Some(Shown::Visible | Shown::Hidden), .. }`
  shows or hides the label on the caller's say alone, at once, whatever the pointer does (the
  dock's label machine: hide on press, while a menu is open, while dragging). `None` is the
  hover behaviour as before. A Card tooltip follows `shown` the same way.

Palette behaviour:

- **The selected row's rect waits for layout.** A palette mounted on a surface that has not been
  laid out yet (a launcher surface mapped again) reads its selected row as 0 x 0; quire now
  reads it again every frame for about half a second, then every 100 ms for three seconds more,
  and reports it only once it has an area. It measures again when the selection, the row
  element or the results (groups, tokens) change, and drops a read still waiting when a newer
  one starts. Drop any "empty rect means unknown" fallback: `on_select_rect` never hands you
  one.
- **`onkey` hands on the event.** `CommandPalette { onkey: Option<EventHandler<KeyboardEvent>> }`
  (and `TextInput`'s and `SearchField`'s `onkey: EventHandler<KeyboardEvent>`): call
  `event.prevent_default()` on a key you take, and Blitz does not also act on it (Tab no longer
  moves the focus off the field). A closure typed `|key: KeyboardData|` becomes
  `|key: KeyboardEvent|`; `key.key()`, `key.modifiers()` read as before.
- **A palette kept mounted.** `CommandPalette { shown: Some(Shown::Visible | Shown::Hidden),
  retain: Retain::Nothing | Retain::Query }` (`Shown` is the tooltip's). Hidden, the card is
  `display:none` (nothing laid out or painted, no scrim) and leaves the layer stack, while its
  rows and field stay in the document. Each change to `Visible` replays the entrance
  (`cmdk-in`/`peek-in`, restarted through the keyframe's `X--b` alias), reports the selected
  row's rect afresh, gives the field the keyboard and, under `Retain::Nothing` (the default),
  asks for an empty query through `oninput("")` and goes back to the first choice (a controlled
  selection is asked for 0 through `on_select`); `Retain::Query` keeps both. A palette mounted
  hidden does not take the keyboard until it is first shown. `None` (the default) is the
  palette as before: shown, its entrance played as it mounts.

  ```rust
  let mut shown = use_signal(|| Shown::Hidden);
  // The shell's toggle: shown.set(Shown::Visible) on open, Shown::Hidden on hide.
  rsx! {
      CommandPalette::<Hit> {
          label, placeholder, query: query(), tokens: Vec::new(), groups, empty,
          oninput: move |text: String| query.set(text),
          onpick, onclose: move |()| shown.set(Shown::Hidden),
          host: CommandPaletteHost::Surface,
          entrance: PaletteEntrance::CmdkIn,
          shown: shown(),
          retain: Retain::Nothing,
      }
  }
  ```

  A hidden field keeps the focus it had (there is no blur seam): a shell surface that is
  unmapped while hidden gets no keys, but a palette hidden inside a window that stays up should
  move the focus somewhere else itself.
- **Ctrl+K toggles the actions menu.** design/06 §20.2: the second Ctrl+K (or Alt+K) closes the
  actions menu it opened. While the `Menu` is open it has the keyboard, so the palette's
  `onkey` never hears that key; listen on the menu instead. `Menu` has no `onkey`, but its
  keydown bubbles out of the overlay host to your root's `onkeydown` (the menu stops only
  Escape), so read the second Ctrl+K there while the menu is open, `prevent_default` it, close
  the menu and hand the field the keyboard back. The palette's `onkey` must stop the first
  Ctrl+K as well as prevent it: the key it opens the menu on bubbles on to the same root
  handler, which would see the menu open and close it at once
  (`crates/ds-native/tests/palette_actions_key.rs`):

  ```rust
  div {
      onkeydown: move |event: KeyboardEvent| {
          if actions() && is_actions(&event) {
              event.prevent_default();
              actions.set(false);
              field.request();
          }
      },
      CommandPalette::<Hit> { /* … */ focus: field,
          onkey: move |event: KeyboardEvent| if is_actions(&event) {
              event.prevent_default();
              // Or the same keydown bubbles on to the root, finds the menu open and closes it.
              event.stop_propagation();
              actions.set(true);
          },
      }
      if actions() { Menu { /* … */ onclose: move |()| { actions.set(false); field.request(); } } }
  }
  ```

### PDF and printing

A quire document as a vector PDF, without a webview: Blitz lays it out, quire paginates it, and
the `anyrender_pdfrum` painter writes it through pdfrum (text stays text in embedded, subsetted
faces at the layout's variable instance; a JPEG or an opaque PNG is embedded as it arrived). FINDINGS.md "PDF output" has the reasons, the limits and the
measured numbers.

| Need | API | Notes |
| --- | --- | --- |
| An HTML document as a PDF | `ds_native::pdf(&html, PageSpec::default()) -> Result<Vec<u8>, PdfError>` | A whole document (`<!DOCTYPE html>...`). quire's faces are registered; the network is sealed: only `data:` URLs load (no `file:`, no fetch). `@media print` applies. Laid out once at the page's content width, at scale 1. |
| A Dioxus tree as a PDF | `ds_native::pdf_app(app, HarnessConfig::new(viewport), spec)` | Built as `config` says (its contexts and `NetPolicy`; the viewport is replaced by the page's content box), rendered until its mount-time work and images have landed (as `snapshot`), then printed. |
| In a test | `Harness::pdf(spec) -> Result<Vec<u8>, PdfError>` | Prints the harness's document as it is now, at the page width with `@media print`; the harness's own viewport and media come back afterwards. Read the PDF back with `pdfrum` (dev-dependency) to assert on text. |
| The sheet | `PageSpec { size, margins }`; `PageSize::{A4, Letter, Custom { width: Pt, height: Pt }}`; `Margins { top, right, bottom, left }`, `Margins::uniform(Pt)`, `Margins::symmetric(vertical, horizontal)`; `Pt::from_mm`, `Pt::from_inches` | `PageSpec::default()` is A4 with 18 mm above and below, 16 mm at the sides. `@page` is not read: the spec's margins are the only ones. `PdfError::NoContentArea` when the margins meet. |
| Start a new page at an element | `"data-break-before": "page"` | CSS `break-before`/`page-break-before` do nothing on Blitz (stylo drops them). A marker at the document's top makes no blank page. |
| Keep a box on one page | `"data-break-inside": "avoid"` | Moved whole to the next page when it would straddle the cut, unless it is taller than a page (then it is cut between its lines). Lines, images, `svg`, `canvas`, `iframe` and table rows are kept whole on their own; a text block keeps its first two and last two lines together (orphans and widows of 2). |
| Screen-only or print-only styling | `@media print { .. }` | Applies in `pdf`, `pdf_app` and `Harness::pdf`. |
| CJK text | name the family: `font-family: "Noto Sans CJK TC", sans-serif` | Otherwise fontique's fallback picks (DroidSansFallback on this machine). A variable face prints at the instance the layout used. |
| Print it | `ds_native::print_dialog(&pdf, title) -> Result<PrintOutcome, PrintError>`, feature `print` | Linux: the desktop portal's print dialog (GTK or KDE backend), then the portal prints the PDF. No portal or no print backend, and other systems: the PDF is written to a temp file and opened in the viewer. `PrintOutcome::{Printed, Cancelled, Opened(PathBuf)}`, `PrintError::{Portal, Write, NoViewer}`. **Blocks** until the dialog is answered: call it off the UI thread. The dialog is not parented to the window. |
| Try it by hand | `cargo run --release -p ds-native --example pdf -- out.pdf`; `cargo run -p ds-native --features print --example print` | The first writes the fixture and prints its size and time; the second opens the real dialog. |
| The painter alone | `anyrender_pdfrum::write(&[Page { size, scene, placement, clip, area }], &sources)` | Pages recorded into anyrender's `Scene` by any anyrender renderer, written as one PDF; Blitz-free. `Sources { texts: RunTexts, images: ImageSources }` carry what anyrender does not: each run's text (`RunKey`, `RunText`) and each image's encoded bytes by decoded blob id. `GlyphArea::Within(rect)` drops glyphs whose box centre falls outside. |

## 7. Settings schema: `#[derive(SettingsSchema)]`

If your app has its own settings struct (not `AppearanceSettings`/`IconsSettings`, which quire
already derives), give it a schema the same way so the Settings app can render it
(design/22-SETTINGS.md section 9):

Both the struct and every enum one of its fields holds need the derive — the struct gets
`SettingsSchema` (a `KeySpec` per field, from `#[settings(...)]`), a fieldless enum gets
`SchemaVariants` (its variant words, so the struct's own derive can pick a widget by arity:
`ds_settings::schema::kind_from_variants`, section 9.1). The trait `.schema()` calls through is
`ds_settings::schema::SettingsSchema` — a different item from the `SettingsSchema` the derive
macro re-exports at the crate root (`crates/ds-settings/tests/schema.rs` is quire's own worked
example of both imports together):

```rust
use ds_settings::SettingsSchema;               // the derive macro (macro namespace)
use ds_settings::schema::{Page, SettingsSchema};   // the trait `.schema()` needs (type namespace)
use serde::{Deserialize, Serialize};

#[derive(Debug, Clone, Copy, PartialEq, Eq, Default, Serialize, Deserialize, SettingsSchema)]
#[serde(rename_all = "snake_case")]
pub enum OpenLinks {
    #[default]
    InApp,
    Externally,
}

#[derive(Debug, Clone, PartialEq, Serialize, Deserialize, SettingsSchema)]
#[serde(default)]
#[settings(file = "your-app/settings.toml", domain = "reader", page = Page::App("your-app".to_owned()))]
pub struct ReaderSettings {
    #[settings(label = "Open links", help = "Where a link in a message opens.", section = "Reader")]
    pub open_links: OpenLinks,   // a two-variant enum -> a Toggle widget
    #[serde(flatten)]
    #[settings(skip)]            // a catch-all field is never a key — every other field needs
    pub extra: toml::Table,      // its own #[settings(label = "...")] or the derive refuses to build
}

impl Default for ReaderSettings {
    fn default() -> Self {
        ReaderSettings { open_links: OpenLinks::default(), extra: toml::Table::new() }
    }
}
```

Wire `--write-schema <dir>` into your `main`'s argument parsing with
`ds_settings::schema::maybe_write_schema` (also in the `schema` module, not the crate root):

```rust
fn main() {
    let args: Vec<String> = std::env::args().collect();
    if ds_settings::schema::maybe_write_schema(&ReaderSettings::schema(), &args).unwrap_or(false) {
        return;
    }
    // ...normal startup...
}
```

`cargo run -p your-app -- --write-schema target/schema` writes `<app-id>.settings.toml`
(section 9.2); install it to `$XDG_DATA_DIRS/quire/settings/` for a local dev loop, or ship it
next to your `.desktop` file. `KeyKind` picks the widget for you — a two-variant enum is a
`Toggle`, three to five a `SegmentedControl`, more a `Menu`, a `#[settings(range = "..")]` newtype
a `Slider` — see `ds_settings::schema::KeyKind::widget` (design/22-SETTINGS.md section 9.1) for
the full table; you never choose a widget yourself.

## 8. What Blitz cannot do, and what to use instead

Everything below is from `FINDINGS.md` spikes S1-S16 (blitz @ `e99fbdbd`, dioxus 0.7.10) — the
authority; this table is a pointer. `ds::lint::Rule::BlitzUnsupported`
(`crates/ds/src/lint/blitz.rs`) catches most of the CSS-level ones for you under
`Profile::Strict` (well, under any profile — it is not one of the three raw-geometry rules
`Profile::Standard` skips) if you write the banned property in your own stylesheet.

| Blitz cannot | Finding | Use instead |
| --- | --- | --- |
| `[data-x=v]` attribute selectors (unprefixed) | S2 | write `[*|data-x=v]` — quire's own component CSS does this everywhere; `Rule::UnprefixedAttributeSelector` |
| `backdrop-filter: blur()` | S15 | ask the compositor to blur behind the surface (`Material`/`BlurState`), not CSS |
| CSS `stroke`/`fill` reaching `<svg>` children | S6 | `ds::Glyph` (renders `.ds-ic` with `stroke="currentColor"` as an attribute, not a rule); `Rule::SvgPaintInCss` |
| `text-overflow: ellipsis` | S13 | `.ds-truncate` (a mask-image fade) or `ds::clip_chars` for a real character-count ellipsis |
| `:focus-visible` / `:focus-within` (hard-coded `false`) | S12 | `.ds[*|data-modality=keyboard] :focus` — `Ds`/`ds_native::launch` track modality for you; `Rule::FocusPseudoClass` |
| `onmounted` + `get_client_rect()` inside the handler itself (returns 0×0) | S9 | `ds::use_rect()` — measures one frame later, never inside the handler; to anchor an overlay, `Anchor::Mounted` does this for you |
| a click on a `Button`/`IconButton` whose parent holds only inline content (the button alone, or beside text) | blitz-dom hit test | put the button in a flex row (every quire container is one) or a block; the parent of an atomic inline is hit instead (`crates/ds-native/tests/click.rs`, FINDINGS "Polish pass") |
| `mask-image:url(data:...)` / `background-image:url(data:...)` without a `data:` `NetProvider` | S7, S8 | `ds_native::launch`/`Harness` already install one; nothing to do if you use them |
| `mix-blend-mode`, `position: sticky`, `line-clamp`, `text-shadow` | risk table | avoid outright; `ds::clip_chars` covers the line-clamp case |
| `line-clamp` for a multi-line clamp that opens on hover | notification parts | a `max-height` in whole `em` lines with a transition, and a fade decided by measuring (`NotificationCard`'s body); the hidden lines are still hit-tested, so give them `pointer-events:none` |
| a wheel phase (a touchpad gesture's end) | notification parts | treat a quiet spell after the last delta as the end (`DelayToken::SwipeQuiet`, `use_swipe`) |
| a clean removal of a running animation | notification parts | Blitz keeps the last animated value when an animation is taken off an element before a frame resolved past its end: put an entrance on an element that mounts with it rather than on a presence attribute that changes (`BannerStack`) |
| `break-before`, `break-inside`, `page-break-*`, `@page` (printing) | FINDINGS "PDF output" | `data-break-before="page"`, `data-break-inside="avoid"`, and `PageSpec` margins, read by `ds_native::pdf` (section 6, "PDF and printing") |

What *does* work and needs no fallback: a `<style>` in the body (S1), the `.ds[data-*]` custom
property cascade once selectors carry `*|` (S2), `@keyframes` including `var()` inside them (S3),
`transition` on a `var()`-driven value (S4), restarting an animation by swapping its name (S5,
what `ds::use_pulse` does), `futures-timer` sleeps from render or a handler (S10, what `ds::sleep`
and every quire timer uses), registering a bundled font through a shared `FontContext` (S11), and
`color-mix()` (S14, though quire precomputes washes instead, for determinism).

## 9. Comparing your surface against the reference

`ds-gallery` (`crates/ds-gallery`) is quire's own contact sheet: every component across its
pages (tokens, type, controls, lists, overlays, materials, motion, space, gaps, matrix,
motion-lab), with a toolbar for theme, accent, motion, material, blur and Space preset. Its
"Gaps" page lists what quire does not draw yet, each with the FINDINGS section that owns it.

```bash
cargo run -p ds-gallery                              # opens the gallery interactively
cargo run -p ds-gallery -- --page controls            # opens directly on one page
cargo run -p ds-gallery --release -- --snapshot DIR   # renders every page x state to DIR as PNGs, then exits
                                                      # (only to DIR; --progress also refreshes the progress page's tracked shots)
```

Render your own surface at the same viewport and Appearance/Material combination and diff it
against the matching `--snapshot` PNG by eye (PNG snapshots are review artefacts, never a CI
gate — rasterisation drifts with Blitz revisions, per `DESIGN.md`'s "Moves verbatim" notes) —
that is the fastest way to catch a component you have subtly mis-wired (wrong `Material`, a
missing `Surface`, an icon at the wrong `IconSize`).

## 10. Menu tracking, curves, Spaces store

Three pure pieces a shell needs beside the components, and the generic settings file API they
sit on.

**Menu tracking** (`ds::MenuTrack<K>`, `crates/ds/src/stack/menu_track.rs`; design/13
§13.3.2-13.5). One machine drives bar menus and every ds `Menu`: open on press, click mode,
press-drag-release, hover switch between open menus, the submenu delay and the safe triangle.
`K` is your menu key (a bar title id). Feed it events with the time; perform what it returns,
in order.

```rust
use ds::{MenuTarget, MenuTiming, MenuTrack, MenuTrackEffect, MenuTrackEvent};
use std::time::Instant;

let track: MenuTrack<u32> = MenuTrack::new(MenuTiming::default()); // 200 ms delay, 300 ms triangle
let (track, effects) = track.step(MenuTrackEvent::PressTitle(1), Instant::now());
// effects == [MenuTrackEffect::Open(1, MenuAnim::Pop)]
// RequestTick(at) asks for MenuTrackEvent::Tick at `at`; SubPlaced{top, bottom} reports a
// submenu's near-edge corners so the safe triangle can arm.
```

Build `MenuTiming` from `menus.submenu_delay_ms` and `menus.submenu_triangle_timeout_ms`. Arrow
keys inside a menu, wrap, disabled skipping and filtering stay with the `Menu` component; the
machine takes `MenuKey::{Escape, Left, Right, Enter}` (map Space and Tab to `Enter`).

**Curves** (`crates/ds/src/motion/curve.rs`). For motion you drive from a frame clock rather
than CSS: `EasingToken::Out.easing(level).at(Fraction(t))` gives progress in thousandths at time
`t` (thousandths). `Easing::curve()` gives the `CubicBezier` (`linear` is `(0,0,1,1)`), and
`CubicBezier::at` evaluates it. Integer arithmetic throughout; a spring's overshoot reads above
1000.

**Spaces store** (`ds::SpaceStore`, `crates/ds/src/space/store.rs`; design/21 §4, §10). Which
look each workspace wears. `store.look_for_workspace(&Workspace { id, index }, defaults)` looks
up by compositor id, then by position, then falls back to `PRESETS[index % 8]`;
`store.look_for(WorkspaceIndex(i), defaults)` skips the id. `SpaceDefaults` carries
`spaces.default_grain` and `spaces.default_card_accent`. `store.with_look(&workspace, look)`
records a look under the id (when there is one) and always under the position.

**Settings files** (`ds_settings::file`, `ds_settings::watch`). Declare a file once and load,
save and watch it:

```rust
use ds::SpaceStore;
use ds_settings::{AppName, FileName, Format, Settings, config_dir};

const SPACES: Settings<SpaceStore> = Settings::new(FileName("spaces.json"), Format::Json);
let dir = config_dir(AppName::QUIRE).expect("a config dir");
let store = SPACES.load(&dir);            // lenient: a bad key costs only itself
SPACES.save(&dir, &store)?;               // atomic temp-and-rename
let mut watch = SPACES.watch(&dir)?;      // 30 ms debounce; needs a Tokio runtime
```

`ds_settings::{load, save, watch}` keep meaning `appearance.toml` (`APPEARANCE`); the generic
functions are `file::load(&SettingsFile)`, `file::save(&SettingsFile, &T)` and
`watch_file(SettingsFile)`.

**Settings derive, three more shapes.** Text is recognised by type (`String`, `PathBuf`,
`Cow<str>`) or by `#[settings(text)]` on a newtype. A one-variant enum is a key
(`KeyKind::Fixed`, drawn as a read-only `Widget::Readout`). A number without
`range = "min..=max"` is a compile error that starts `MissingRange { field: <name> }`; a type
the derive cannot place at all gets an error saying what to add.

## 11. Scrolling

shell-host owns scroll physics for every Blitz document (design/11-BEHAVIOUR-scroll.md section
11.3.1): it does not forward wheel or axis input to Blitz's default scroll action, it writes raw
offsets itself every frame, and it paints its own overlay scrollbar thumb. Everything below is
quire's side of that split (design/11 section 11.7); the engine itself is shell-host's.

**Every scroll container hides Blitz's own scrollbar.** Every ds component whose content scrolls
(`Menu`, `Panel`, `Sheet`, `TextInput`'s multiline kind) carries `scrollbar-width: none` in its own
CSS. Blitz honours this at the DOM level (`blitz-dom`'s `Node::wants_scrollbar` returns `false`
immediately when the computed `scrollbar-width` is `none`, before it even asks whether the content
overflows) — the host's thumb is the only one that ever paints. Give your own scroll containers
the same rule; `Rule::BlitzUnsupported`/self_lint do not check for its absence (there is nothing to
flag: an *absent* `scrollbar-width: none` is not, by itself, an offence), so this is a convention
to follow, not a lint you can lean on.

**`data-wheel="capture"` marks a component that consumes wheel input itself.** A ds component
that reads the wheel directly — today, only `Slider` (design/04-COMPONENTS.md section 5; a
zoomable canvas would be the other kind design/11 names) — carries this attribute so the host
hands it the raw `BlitzWheelEvent` through `handle_ui_event` instead of scrolling whatever
contains it; the component's own handler must call `prevent_default` once it exists (design/11
section 11.3.1 item 2). It is a marker only: quire does not itself drive a wheel-triggered value
change on `Slider` yet, so add the same attribute to your own wheel-consuming controls even before
you wire up the handler, and remove it from a component that turns out not to need it — an
un-marked wheel-consumer silently gets scrolled instead of heard.

**`data-overscroll="band"` marks a container that may rubber-band; its absence means clamp.**
design/11 section 11.3.7 has the full table. In short: `Panel` and `Sheet` carry it (ordinary ds
scroll containers, not on the exclusion list); `Menu`, `Popover`, the dock and the launcher result
list never do (excluded by name, R14); a text field never does (also excluded). Your own scroll
containers — a list's row scroller, a reader pane — should carry it too, unless they are one of
the excluded kinds; an un-marked container the host finds just clamps at its edge, which is the
safer failure than defaulting to elastic.

**`scroll-behavior: smooth` is banned**, in your own CSS as much as quire's:
`ds::lint::Rule::BlitzUnsupported` flags it (`crates/ds/src/lint/blitz.rs`) because Blitz's own
300 ms `ScrollTo` would fight the host's engine. Drive a programmatic scroll through the host's
`ScrollCmd` instead (design/11 sections 11.3.1 and 11.3.11); `scroll-behavior: auto` (the
default) lints clean.

**`--scroll-thumb`**: the colour the host paints its overlay scrollbar thumb in
(design/11 section 11.3.12) — `--ink` at .5 alpha over the layer's own .8 opacity (effective .4)
in light, `--paper`'s equivalent (white at the same effective alpha) in dark. It is a token like
any other (`ds::ColourToken::ScrollThumb`, registered in the lint's known-variable table the same
way every other colour is); quire's stylesheet declares it, the host reads it, and no component
CSS references it directly, since no ds component paints the thumb itself.
