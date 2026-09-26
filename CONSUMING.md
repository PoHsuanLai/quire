# Consuming quire

This is the guide for a program that draws with quire instead of hand-rolling its own CSS and
markup: today that is `examples/consumer` (a minimal, fully worked example — read its
`Cargo.toml`, `src/lib.rs` and `tests/coherence.rs` alongside this doc) and, from
`docs/mailo-migration.md` onward, mailo. It assumes you have read
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
`cssparser`. If your consumer is still on a Dioxus desktop webview rather than Blitz (mailo
Phase A, `docs/mailo-migration.md`), also enable `ds`'s `webview-fonts` feature and use
`ds::font_face_css` (section 4).

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
(`CONVENTIONS.md#11-quire-addenda-2026-09-24`: "the pinned block... is copied verbatim into
every workspace"; `docs/workspace-deps.toml` is quire's own copy of the source of truth, not
owned by this doc). If you also run on Blitz (`ds-native`), your own crate that calls
`dioxus_native::*` directly (rare — most consumers only call `ds_native::launch`) needs the
`dioxus-native`/`blitz-*` lines too, exactly as pinned, never a different revision.

**If your own crate sits inside a Cargo workspace tree it does not own** (as `examples/consumer`
does, under quire's own directory), give it an empty `[workspace]` table so Cargo does not treat
it as an orphaned member of the workspace above it (`spike/blitz-probe/Cargo.toml` is the other
example of this in this repo).

## 2. The `Ds` root

**A bare document, with no `Ds` root** (a never-painted input catcher, a 2 px hot-corner
surface) still carries Blitz's user-agent stylesheet, which sets `body { margin: 8px }`: its
content never reaches the true (0, 0) corner, and `position: absolute`/`fixed` with `inset: 0`
paints nothing on it (sill hot corners, 2026-09-26). shell-host injects `body { margin: 0 }`
into every document it hosts (shell-host F62); a bare document elsewhere (`ds_native::launch`,
a test) must cancel the margin itself or draw inside a `Ds`.

Every quire component must be drawn inside one `Ds` (`root/ds.rs`; design/03-COLOR.md
section 17.1; `crates/ds/src/root/ds.rs`). It resolves your appearance to a scheme, an accent
and a motion level; stamps `data-theme`, `data-typeface`, `data-accent`, `data-motion`,
`data-material`, `data-blur`, `data-modality` and `data-hover` on its own `div.ds`; injects the stylesheet
(unless you ask it not to); and provides the `Env`, `HoverHub`, `ToastHub`, `LayerStack` and
`Overlays` contexts every component reads. It also renders `OverlayHost` and `ToastHost` after
your children, so menus, popovers and toasts always have somewhere to mount. `ToastHost` lays
out nothing while the hub is empty: a pushed toast mounts hidden for one frame and rises from
there, and is dropped again once it has sunk (FINDINGS "Gallery fixes A"), so an idle root has
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

`Ds`'s props, all named, none a `bool` (`CONVENTIONS.md#11-quire-addenda-2026-09-24`):

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
| `extent` | `RootExtent` | `RootExtent::Content` | `Content`: as tall as the root's content (a window, the bar, a card). `Viewport`: at least the viewport (`min-height:100vh; min-width:100vw`), **the option for an overlay surface** (an OSD, a sheet, a click catcher) whose content is all positioned and would otherwise leave the root, and everything it places, 0 px tall (sill F172; FINDINGS "Sheet and modal parts") |
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
overlay root, a wallpaper, the launcher's catcher (sill F172, F225, Q104). **The frame (the
document's first box, the one around the `Ds`) is in normal flow and has a height; it is never
absolutely or fixed positioned, and never left to be sized by content that is itself out of
flow.** Blitz lays out `#main` and `.ds` at `height:auto`: a frame with `position:absolute;
inset:0` or `position:fixed` resolves against that 0 px box (Taffy places a fixed box against
its parent, not the viewport), a `height:100%` resolves against it too, and a frame whose only
content is a card hung from its anchor is 0 px tall. Then every box from `html` down is 0 px,
nothing paints (sill's wallpaper, F172) and in the shell no press lands (the bar popup, F225).

- **The surface fills its surface** (popup frame, overlay, wallpaper, catcher): if your quire
  has `Ds { extent: RootExtent::Viewport }` (the sheet-parts branch, sill Q94), pass it. Until
  then the floor is on your frame: `display:grid; width:100vw; min-height:100vh` in flow, the
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

A consumer still on a Dioxus desktop **webview** (mailo Phase A) is not affected either way:
`dioxus-desktop` already runs inside its own Tokio runtime, so `use_environment` and
`ds_settings::watch` both work as documented there.

### `use_env` — reading the resolved scope

Inside any component under a `Ds`, `ds::use_env() -> Env` gives you what that scope resolved to:
`resolved: Resolved{scheme, accent, motion}`, `scheme`, `material`, `blur`, `modality`
(`root/env.rs`). Components use this to size icons, choose overlay placement and read the motion
level for their own timers (section 6); you will rarely need it directly unless you are building
your own component.

## 4. `Surface` — a nested material, scheme, accent or blur state

A subtree in a different `Material`, or forced to a scheme, accent or blur state other than the
root's (a popover over a dark card, an always-light preview pane, a specimen in another accent)
is a `Surface`, not a second `Ds`: no stylesheet, no frame layers, just a nested `div.ds` that
re-stamps `data-theme`/`data-typeface`/`data-accent`/`data-motion`/`data-material`/`data-blur`
(the typeface is always the root's) and updates the
`Env` every component under it reads.

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
has a golden per override; FINDINGS "Gallery fixes B").

## 5. The four coherence rules

`ORCHESTRATION.md#coherence-rules-in-every-brief` states four rules every downstream crate
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
        ..LintConfig::default()
    });
}
```

Every exception needs a `reason`, and `assert_clean` panics listing which exceptions suppressed
zero offences, so a stale one cannot hide silently (mailo gaps 3: before it, it only printed
them). A consumer mid-migration that must keep an exception for code another branch is about to
land sets `stale: Stale::Report` (`ds::lint::Stale`): the stale exception is printed and the
test passes. `Stale::Fail` is the default. A `LintConfig` written with every field named needs
the new `stale` field, or `..LintConfig::default()`. Prefer `Profile::Strict` even though
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
    let offences = markup(&render_ssr(), &full_css(), &LintConfig::default());
    assert!(offences.is_empty(), "{offences:#?}");
}

#[test]
fn no_raw_button_is_rendered() {
    let offences = markup(&render_ssr(), &full_css(), &LintConfig::default());
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
use ds::{Anchor, Button, ButtonVariant, Menu, MenuKind, MountedRef, Switch, UndoToken, use_toasts};

let toasts = use_toasts();
toasts.push("Sent".to_owned(), None);   // ToastHost is already rendered by Ds — nothing else to mount
// With an undo: the handler of the toast on screen runs when the person undoes it.
toasts.push_undoable("Archived".to_owned(), UndoToken(7), EventHandler::new(move |token| restore(token)));

// A menu anchored to the button that opens it: the button hands over its own element.
let mut open = use_signal(|| Switch::Off);
let mut more = use_signal(|| None::<MountedRef>);
rsx! {
    Button {
        variant: ButtonVariant::Secondary,
        label: "More".to_owned(),
        onclick: move |_| open.set(Switch::On),
        mounted: move |event: MountedEvent| more.set(Some(MountedRef(event.data()))),
    }
    if let (Switch::On, Some(button)) = (open(), more()) {
        Menu { kind: MenuKind::Rich, anchor: Anchor::Mounted(button), entries, onpick, onclose: move |()| open.set(Switch::Off) }
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

A few props worth knowing about before you read the signatures, added in wave 2 integration:

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

And in Gallery fixes B (FINDINGS "Gallery fixes B"):

- `Button` and `IconButton` take `mounted: Option<EventHandler<MountedEvent>>`: the element
  itself, for `Anchor::Mounted` (the Overlays example above). It writes no attribute.
- `HoverCard` takes `parts: Vec<HoverCardPart>` (`Title`, `Sub`, `Person`, `Stats`, `Flag`,
  `Messages`, `Foot`, `Actions`), drawn in order before its children: no hand-written
  `ds-hovercard-*` markup.
- `ToastHub::push_undoable(text, token, on_undo)` calls `on_undo` with the token when the person
  undoes that toast; a later push replaces it. The handler belongs to the scope that made it,
  so that scope must outlive the toast. `last_undo()` still reports the last undo.
- `Surface` takes `accent` and `blur` beside `theme` (section 4).

And in the tray gaps (FINDINGS "Tray gaps", sill Q6-Q8):

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

And in the bar gaps (FINDINGS "Bar gaps", sill Q9-Q13, G7):

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
  side (FINDINGS "Tune wave").
- **New glyph**: `Icon::Ethernet` (Lucide `ethernet-port`) for a wired network.

And in the launcher gaps (FINDINGS "Launcher gaps", sill Q40-Q45, and the dock's Q15-Q17):

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

And in the palette follow-ups (FINDINGS "Palette follow-ups", sill Q60-Q63):

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

### The macOS polish pass (2026-09-24): what changed under you, and what to opt into

Everything below is additive or token-level; no prop was removed or renamed. What you get with no
change of your own:

- **Shell cards look deeper.** Every card on a material (a menu or popover on a Popover root, a
  Sheet, a Toast, an OSD, a widget, the dock pill) paints material stack v2: a 0.5 px dark outer
  hairline, a tight contact shadow, a wide ambient one, a 1 px inner top highlight, and a tint with
  a vibrancy boost baked in (design/03 section 17.4). The bar gains a bottom hairline. Paper
  roots (a mail window) are unchanged.
- **Text menus are 22 px.** Slim, Context and Dropdown rows are 22 px with 13/400 text, a 6 px
  inset highlight and hairline separators with 5 px margins; Rich keeps its 34 px tiles.
- **Tooltips are 12 px** (the Fly was 10). **`IconButton { Status }`'s pill is 4 px** (was 9).
- **`--shadow-window`** is now a contact shadow under a wide soft ambient one.
- **A `Popover` fades out** over `--t-quick` on Escape or an outside click before `onclose`.
- **A palette in a surface is as tall as its content** (up to its container), with the launcher
  scale: a 22/500 query beside a 20 px glyph, 14 px row titles, 12 px details. If you measured the
  card to size a blur region, measure it again after each query.

What to opt into:

- `Corner::Squircle(Px(r))` anywhere a `Corner` goes (`Ds { radius }`, `Surface { radius }`,
  `CommandPalette { corner }`): the continuous-curvature corner. Its shadow follows a circle of
  about .884 r; a blur region should use that radius (`Corner::css()` gives it).
- `Ds { stack: Some(MaterialStack { .. }) }`: the stack's highlight, hairline, shadow strength
  and vibrancy from your settings (inherited by every card under the root).
- `ShellMetrics::style_attr()` and `DockMetrics::style_attr()` on any element around your
  surfaces: the shell type scale and the dock's geometry from settings; absent, the keys'
  defaults apply.
- `MenuBarItem { open, emphasis, children }` around a bar title or the clock;
  `WorkspacePills { label, WorkspacePill { label, current, onclick } }` for the workspace
  indicator; `RunningDot {}` in a tile and `DockFloor {}` in the dock root;
  `IconView { plate: Some(PlateFamily::Blue), .. }` for a plate with depth, and
  `plate_tint: PlateTint::of(style, tint)` on it to re-colour the plate for `icons.style`
  (below, "App icons and the icon style").

### The mailo gaps (2026-09-24): the window frame, status inks, person colours, two glyphs

What you get with no change of your own:

- **The window frame paints over its root.** A `Material::Window` root stamps
  `data-frame="opaque"` and is its own stacking context (`position:relative`, `--z-raise`), so its
  two `.ds-layer`s and `.ds-grain` paint over the root's own gradient background instead of
  beneath it: a `look` change cross-fades over `--t-scene`, and the Space's grain shows. Neither
  layer takes the pointer. If a pixel test of yours assumed a flat window ground, give its look
  `Grain(0)`.
- **Dark `--danger-ink` is dark** (`#1A0B08`, 6.06:1 on `#E0705A`; white was 3.17:1).

What to use:

- **Status inks.** `--ok-ink`, `--warn-ink`, `--danger-ink` (`ColourToken::{OkInk, WarnInk,
  DangerInk}`): text on `--ok`, `--warn`, `--danger`, each at least 4.5:1 in both schemes, as
  `--accent-ink` is on `--accent`.

#### Person colours

A person or account colour never needs a hex constant of yours:

- A person with no stored colour: `ds::person_hue(address) -> PersonHue`, design/03 section 13's
  hash (`h = (h x 31 + code) mod 360` over UTF-16 code units, drawn `hsl(h, 38%, 42%)`). Paint it
  with `Avatar { face: AvatarFace { tone: AvatarTone::Person(hue), .. } }`, or pass
  `hue.colour()` where a component takes a `Colour`.
- A stored colour, handed out in order: `ds::PersonSwatch` (eight, `--c-person-1` to
  `--c-person-8`, the same in both schemes: identity is data, not theme).
  `PersonSwatch::nth(i)` wraps, `.colour()` is the `Colour` for `AvatarTone::Account`, `.var()`
  the custom property for your own CSS (`background:var(--c-person-3)` lints clean).

- **Glyphs.** `Icon::Printer`, `Icon::FolderInput` (Lucide `printer`, `folder-input`), listed in
  `Icon::ACTIONS`.

### The mailo gaps 2 (2026-09-25): controls and tiles

Every row is additive: leave the prop out and the markup is what it was, except where a row
says the markup changed. FINDINGS "mailo gaps 2 (controls and tiles)" has the why of each.

| Component | Prop or variant | Type (default) | What it does |
| --- | --- | --- | --- |
| `Button` | `title` | `Option<String>` (`None`) | the hover hint, written as `title` |
| `Button` | `aria_label` | `Option<String>` (`None`) | names the button for assistive technology in place of its visible label (a `+`, an `All`) |
| `Button` | `expanded` | `Option<Expanded>` (`None`) | `Expanded::{Open, Closed}` as `aria-expanded`, for a button that opens a menu or panel; `IconButton` already had `tooltip`, `label` and `expanded: Option<Switch>` and is unchanged |
| `TextInput` | `onfocus`, `onblur` | `EventHandler<()>` (no-op) | the caret arrived or left: a click or Tab, and the focus seam (`Focus::OnMount`, `Focus::Controlled`), which on Blitz moves the caret with no event, so the field calls `onfocus` itself |
| `TextInput` | `kind` | `TextInputKind` (`Text`) | `Password` writes `type="password"` and `data-kind="password"`; Blitz draws a password's characters as typed, so the field's text is transparent and one dot per character is laid over it |
| `Slider` | (none) | | already the range input: `value: Fraction` in thousandths, keys and drag; map your 0-100 to `Fraction(n * 10)` |
| `AccountTile` | `mark` | `MarkStyle` (`Letter`) | how the provider is drawn on the tile: pass your provider-marks setting, `MarkStyle::Image(ImageSource(data_uri))` for the favicon you hold |
| `AddAccountTile` | (new component) | `label: String` ("Add account"), `title: Option<String>`, `onclick: EventHandler<()>` | the tile after the accounts: the Pin plate with no ground at rest, a plus in a dashed `--f-ink-faint` ring; never pressed, no count |
| `SendPill` | `mood` | `SendMood` (`Calm`) | `Nudge`, `Shake`, `Fatal`: the one-shot `nudge` or `shake` (design/05 4.4.8-9) plays each time the mood changes to one of them, never on mount, and settles at `ds::settle`; the pill stays up; `Fatal` also paints it `--danger`/`--danger-ink`; `data-mood` is written for each. Pass `Calm` for a render to play the same mood again |
| `SendPill` | `action` | `PillAction` (`Undo`) | the button's word while counting: `Undo`, `Cancel` (a held send), or `Nothing` (no button); `onundo` hears either word |
| `SendPill` | `ring` | `SendRing` (`Drain`) | `Spin`: a 20/37 arc turning at the Spinner's `spin` while the send waits on the outbox, `progress` ignored |
| `SendPill` | `refusal` | `Option<String>` (`None`) | a second, lighter line under the text: why a take-back was refused, or "No recipients" |
| `SidebarItem` | `trailing` | `Option<TodayTrailing>` (`None`) | a scheduled Today row's `time` (data type, `--f-ink-faint`) and a cancel button named by `cancel`, calling `on_cancel` without opening the row; Today only |
| `SidebarItem` | (markup changed) | | a Today item's close button is now named `Close {label}`, not `Close`: an assertion or selector on `aria-label="Close"` needs the row's label |
| `SpaceEditor` | `on_rename` | `Option<EventHandler<String>>` (`None`) | the title becomes an inline `TextInput` ("Space name", placeholder "Name this Space") holding `name`; each keystroke is reported |
| `SpaceEditor` | `motion` | `Option<MotionChoice>` (`None`) | a Motion row after Appearance: a `SegmentedControl` over `ds::Motion` (System, Calm, Standard, Extra, Reduced), `MotionChoice { level, on_motion }`; feed the pick to your root's `appearance.motion` |
| `SpaceEditor` | `measured` | `MeasuredIn` (`ThisScheme`) | `EachScheme`: the contrast readout under "Measured, this Space", once per scheme the Space's theme shows (Light and Dark for System), each under its own small-caps heading |
| `SpaceEditor` | (markup changed) | | each preset button is named (`aria-label` and `title`) by `Preset::name` (Dusk, Orchard, Harbour, Ember, Lagoon, Heather, Moss, Stone), not "Preset n" |
| `Preset` | `name` | `&'static str` | the preset's name, design/21 section 4 order; a new public field, so a struct literal of `Preset` needs it |

### The mailo gaps 2 (2026-09-25): lists and overlays

Every row below is additive: a prop that defaults to what the component did before, or a new
variant. FINDINGS "mailo gaps 2 (lists and overlays)" has the reasons and the proofs. One prop
per row:

| Component | Prop or type | What it does |
| --- | --- | --- |
| `ListRow` | `subject: Text` (`#[props(into)]`) | A `String`, `&str` or `format!` still works; `Text::Runs(vec![Run::new("UIDL", RunTone::Mark), ..])` draws a search hit as `mark.ds-mark` and a `Strong`/`Faint` run as `span.ds-run[data-tone]`. |
| `ListRow` | `snippet: Option<Text>` | Accepted, each as `Some` unless it is an `Option`: a `&str` (a literal, or `"{formatted}"` in `rsx!`) or a `String` (both through quire's `SuperFrom`); a `Text` (`Plain` or `Runs`); an `Option<Text>`, `Some(text)` or `None`. Not accepted: `Some(string)` or `Some("literal")` (the `Some` hides the `String` from the conversion: write the string alone, or `Some(string.into())`), a `&String` (write `string.clone()` or `string.as_str()`), and an `Option<String>` you hold (write `.map(Text::from)`). `crates/ds/tests/snippet_forms.rs` compiles and renders every accepted form. |
| `ListRow` | `on_sender: Option<PartHooks>` | `PartHooks { onpointerenter, onpointerleave }` (`EventHandler<PointerEvent>` each) on the name: the sender card. |
| `ListRow` | `on_time: Option<PartHooks>` | The same on the time: the time tip (`HoverKind::Tip`, below). |
| `ListRow` | `onpointerenter` / `onpointerleave: Option<EventHandler<PointerEvent>>` | The row itself: the thread card. |
| `ListRow` | `onpointerdown: Option<EventHandler<PointerEvent>>` | A press on the row: a drag's start. A strip button's or the star's press still reaches it (only their clicks stop). |
| `ListRow` | `aria_label: Option<String>` | The row's accessible name ("Open Re: UIDL stability"); absent, its contents name it as before. |
| `HoverStrip` | `shown: Option<Shown>` | `Some(Shown::Visible)` shows the strip on a keyboard-selected or focused row (Blitz never matches `:focus-within`); `Some(Shown::Hidden)` keeps it down under the pointer; `None` is the hover reveal. |
| `HoverStrip` | `titles: Titles` | `Titles::FromLabel` writes each button's label as its `title`; `Titles::Omitted` (default) writes none, as before. |
| `HoverStrip` | `expanded: Vec<(ActionId, Expanded)>` | The buttons that open a menu, and whether it is open: `aria-haspopup="menu"`, `aria-expanded`. |
| `HoverStrip` | (behaviour) | A strip button's click stops at the button: it never opens the row. |
| `Text`, `Run`, `RunTone` | new types | `Text::{Plain(String), Runs(Vec<Run>)}`, `Run { text, tone: RunTone::{Plain, Mark, Strong, Faint} }`, `Text::plain_text()`. You compute the runs; quire never parses markup out of a string. |
| `CommandPalette` | `entrance: PaletteEntrance::Opaque` | The card springs with `cmdk-rise` (`cmdk-in`'s scale and lift, no fade): opaque on its first frame. Over a window the scrim then appears at once too (a fading wrap would hold the card at its own opacity). `PeekIn` and `CmdkIn` are unchanged and still start at opacity 0. |
| `MenuEntry` | `Row(MenuRow<T>)` | A choice whose `title: Text` and `detail: Option<Text>` are your runs, with `trailing: Option<RowAction>`; otherwise an `Item` (`value`, `tile`, `trail`, `check`, `availability`). Build with `MenuRow::new(value, title)` and struct update, or `.into()` a `MenuEntry`. A plain `Text` title is still marked by the palette's query and a menu's filter; runs are drawn as given. A `match` over `MenuEntry` needs an arm for it. |
| `RowAction` | new type | `RowAction { icon: Icon, label: String, on_press: EventHandler<Press> }`: a Strip `IconButton` (label as `aria-label` and `title`) at the row's end. Its click, press, release and pointer moves stop inside it: the row is not picked, its menu or palette stays open, and the selection does not move onto it. |
| `Menu` | `active: Cursor` | `Cursor::Auto` (default) is the menu's own highlight, as before. `Cursor::Controlled(Some(i))` shows choice `i` (numbered as `expanded` numbers them, clamped), `Controlled(None)` highlights nothing; the menu then does not take the keyboard as it opens, so the field beside it (the composer's `/`, `@`, a people input) keeps typing, and Enter in that field is the field's to handle with the value it knows. |
| `Menu` | `on_active: Option<EventHandler<Option<usize>>>` | Under `Auto`, every change of the highlight (the first one included). Under `Controlled`, a request: Up or Down in the menu, or the pointer coming over another choice; set your cursor from it or not. |
| `Menu` | `onquery: Option<EventHandler<String>>` | With `filter: Filter::Typing`, the typed text on every change (Backspace included), for an entry that names it ("Create label “…”"): rebuild `entries` from it. |
| `Menu` | trailing action | Through `MenuEntry::Row(MenuRow { trailing: Some(RowAction { .. }), .. })`, as in the palette: the menu stays open and nothing is picked. |
| `HoverTarget` | `as_: TargetElement` | The element the target is drawn as: `Span` (default, as before), `Div` or `Li` (a list's own item, which a `span` cannot hold). Same handlers, same hub: the card opens, places and closes exactly as for a span. `display:contents` is not offered: the card is placed against the target's rect, and such an element has none. A `ListRow` is already an `li`: hook its thread card through the row's `onpointerenter`/`onpointerleave` instead. |
| `HoverKind` | `Tip` | A value's small tip (a row's time): `HoverCard { kind: HoverKind::Tip, "Wed 23 Sep 2026, 09:41" }` is one line, auto width up to 260, 12 px (the Card tooltip's size), placed 6 below the target's left edge like a sender card, on the hub's timing (450 ms, 0 when warm, 150 ms to close). A `match` over `HoverKind` needs an arm for it. |


### The mailo gaps 3 (2026-09-25): motion, tokens, lint

Additive unless a row says otherwise. FINDINGS "mailo gaps 3 (motion, tokens, lint)" has the
reasons and the proofs.

| Where | Prop, type or value | What it does |
| --- | --- | --- |
| `Roster` | `stay(key) -> Result<Stayed, StayError>` | Takes a leaving row's exit back before it settles (an undo): the row is present again in place, its settle timer is cancelled, nothing below heals. List the key again in the same handler. `Stayed::{Restored, Unchanged}` (a row that was not leaving is left alone); `StayError::UnknownKey` for a key the roster no longer holds (its exit settled: list it again and it enters), `StayError::Unmounted` when the list is gone |
| `RosterState` | `stay(self, &key) -> (Self, Result<Stayed, StayError>)` | The pure transition, for a machine of your own |
| `use_roster` | (behaviour) | Its rest timer is started by an effect after the render, never spawned from the component body (a spawn from render may never be polled on the webview); two reconciles before the effect run start one timer |
| `Anim` | `PillUp` (`pill-up`, `--t-big --e-spring`) | A pill centred by `translateX(-50%)` springs up from below: `animation: pill-up var(--t-big) var(--e-spring)` on your own toast. quire's `SendPill` and `Toast` keep their `data-shown` transition (it also carries them back down) |
| `Anim` | `RingDrain` (`ring-drain`, `--t-send-ring --e-linear forwards`) | A send ring's `stroke-dashoffset` from 0 to 57 (the SendPill's dash) over the undo window, where CSS reaches the ring (the webview). `--t-send-ring` is now a hold: Reduced keeps its 5 s |
| `Anim` | `FadeIn` (`fade-in`, `--t-move --e-out`) | An ink veil fades to `--veil` (.16), where `fade` runs to 1 |
| `Anim` | `Busy` (`busy`, `--t-ambient --e-in-out infinite`) | A busy word pulses between 1 and .45; `Breathe` (the sync halo's) fades to nothing |
| `Anim` | `CmdkRise` | mailo-gaps-2a's, unchanged: the sheet rise with no fade. `Anim::ALL` is 52 long; a `match` over `Anim` needs the four new arms |
| `--veil` | `OpacityToken::Veil` (.16) | The resting opacity of an ink veil laid over content. Not `--scrim`, which is a colour (black at .22) |
| `gulp`, `bump`, `seal-pop` | (keyframes changed) | Their departure from rest scales by `--overshoot`, capped at Standard's: Calm and Reduced flatten them (as `pop-in`), Standard and Extra keep the catalogue's shape |
| `SpaceDot`, `SpaceEditor` | (markup changed) | A Space dot, a preset, the title swatch, a handle and a stop disc write their colours as `--dot-c1`, `--dot-c2`, `--dot-c3` (with `data-stops` on the gradients) and the stylesheet paints them; no inline `background`. An exception for `button.ds-space-dot` (or the editor's `button.ds-preset`, `div.ds-handle`, `i.ds-stop-disc`, `span.ds-space-swatch`) is now stale: delete it. `FrameVars` gains `stops: Vec<String>` |
| `--font-serif` | `Family::Serif`, `Family::ALL` | Noto Serif (400-700, upright and italic, OFL), for a message written in a serif and the control that offers it: `font-family: var(--font-serif)` lints clean |
| `Avatar` | `muting: AvatarMuting` (`Plain`) | `Muted` keeps an account's or person's hue at .55 of its chroma, lightness kept (S's `saturate(.55)`, computed, since Blitz paints no `filter`): an account that is not the one in view. `AvatarMuting::apply(colour)` for your own use. Writes `data-muting="muted"` |
| `LintConfig` | `stale: Stale` (`Fail`) | See section 5: a stale exception fails `assert_clean`; `Stale::Report` prints it instead |
| `Rule::UnknownAnimation` | (reads the shorthand) | See section 5 |

### The mailo gaps 4 (2026-09-25): controls

Additive: leave a prop out and the markup is what it was. Four enums gained variants
(`Provider::Local`, `ButtonVariant::Frame`, `InputVariant::Bare`, and `TextInputKind`'s three), so
a `match` of yours over one of them needs the new arms. FINDINGS "mailo gaps 4 (controls)" has
the reasons and the proofs.

| Component | Prop, type or variant | Type (default) | What it does |
| --- | --- | --- | --- |
| `Provider` | `Local` | new variant | A local-folders account: `AccountFace::One { provider: Provider::Local, .. }`, so the literal keeps its shape. Its mark is a folder glyph in the neutral IMAP grey (`data-kind="local"`, titled "Local folders"), under either `MarkStyle`: there is no favicon to show |
| `Button` | `variant: ButtonVariant::Frame` | new variant | Words on the Space frame: `--f-ink-soft` on nothing, `--f-ink` on `--f-pill-hover` under the pointer, `--f-pill` held or `pressed: Some(Switch::On)` (the sidebar item's current chrome) |
| `Button` | `trailing` | `Option<Trailing>` (`None`) | `Trailing::Caret` (a 12 px chevron) after a dropdown's value; `Trailing::Glyph(icon)` for any other glyph after the label. Pair it with `expanded` for a trigger |
| `Button` | `face` | `ButtonFace` (`Label`) | `Bold`, `Italic`, `Underline`, `Strike` draw the label as `B`, `i` (serif italic), `U`, `S` in their style (`span.ds-button-face[data-face]`), and name the button by `label` as `aria-label` (your `aria_label` wins) |
| `FaceMark` | new component | `face: ButtonFace`, `label: String` | The same face on its own, for a `BubbleButton { label: rsx! { FaceMark { face: ButtonFace::Bold, label: "Bold" } }, .. }` in the selection bubble |
| `TextInput` | `kind: TextInputKind::Secret` | new variant | mailo's secret field: the text lives in the field's own state and reaches you only through `oninput` and `onchange`; no `value` attribute is ever written (your `value` is ignored), only a dot per character. Clear it by remounting under a new `key`. `Password` is unchanged (controlled, writes its `value`) |
| `TextInput` | `kind: TextInputKind::File`, `on_pick` | new variant; `EventHandler<()>` (no-op) | A file's name (your `value`, the placeholder while empty) and a "Choose…" Tool button; a click on either calls `on_pick`. Blitz has no file picker: open your own and pass the chosen name back as `value` |
| `TextInput` | `kind: TextInputKind::Multiline { rows, grow }` | new variant | A `textarea` of `Rows(n)` lines; `Grow::ToContent` adds a row per hard line beyond `n`, `Grow::Fixed` scrolls. Enter types a newline (it does not commit) |
| `TextInput` | `variant: InputVariant::Bare` (`FieldFace::Bare`) | new variant | No box: the parent's font, size, weight, tracking, line height and colour, with an `--accent` caret; for a title or a property row edited in place. `FieldFace` is `InputVariant` under the name the gap asked for, not a second prop. On Blitz the typed value takes the size, line and colour but not the family or weight (blitz-dom's editor reads only those three); a webview takes all of it |
| `TextInput` | `onchange` | `EventHandler<String>` (no-op) | The value committed: Enter in a one-line field, or the caret leaving any field (Blitz sends no `change`; the field makes it on both renderers) |
| `Rows`, `Grow`, `FieldFace` | new types | | Above |
| `SpaceEditor` | `motion_levels` | `MotionLevels` (`All`) | `MotionLevels::Contact` offers Calm, Standard and Extra, a Space's own three (mailo's per-Space motion); a `level` outside them presses no segment. A prop, not a `MotionChoice` field, so your `MotionChoice { level, on_motion }` literal still compiles |

### The mailo gaps 4 (2026-09-25): overlays and lists

Every row is additive: a prop that defaults to what the component did before, a new type, or
a new function. No existing golden changed. FINDINGS "mailo gaps 4 (overlays and lists)" has
the reasons and the proofs.

| Where | Prop, type or function | What it does |
| --- | --- | --- |
| `HoverStrip` | `on_press: Option<EventHandler<ActionId>>` | Hears which button was pressed inside the click, before any rect is read: an archive acts at once, and with no layout (a server render, a host that cannot measure) it still acts. The action's own `onclick: EventHandler<Rect>` still follows when the rect resolves, after `on_press`, for the menu it anchors. A strip prop, not a `StripAction` field, so every `StripAction { .. }` literal compiles as it is |
| `use_hover_intent() -> HoverDriver` | new function and type | The `Ds`'s hover hub and the anchor book every card places against, for a caller that keys cards on its own pointer hooks (a `ListRow`'s `on_sender`, a pin, a Today item) rather than wrapping them in a `HoverTarget`. `driver.over(key, kind, anchor)` on entry, `driver.out()` on exit, `driver.press()` on a press in the list, `driver.hub()` for `open()`/`leaving()`; the 450 ms open, 0 when warm, 150 ms close and 400 ms warm window are the hub's, as for a target. Nothing opens while a peek, the palette or a menu is open |
| `HoverAnchor` | new type | `Rect(Rect)`: a rect you already have (the pointer's point, your own measurement); `Element(MountedRef)`: measured a frame later; `Unplaced`: nothing to place against (no layout), the card opens at the overlay's top-left corner, or in place with `flow: Flow::Inline`. `Unplaced` drops any rect filed earlier for the key |
| `HoverCard` | `flow: Flow` | `Flow::Floating` (default) as before. `Flow::Inline` draws the card where you render it, `position:static` in your container, with no `left`/`top` and `data-flow="inline"`: a test with no layout asserts its content where it stands, and a card that is part of a page can be one |
| `Flow` | new type | `Flow::{Floating, Inline}`, shared by `HoverCard` and `Menu`: in the overlay, placed against the anchor, or in the caller's flow |
| `Menu` | `dismiss: PickDismiss` | `PickDismiss::Close` (default): a pick calls `onpick`, then `onclose`, as before. `PickDismiss::Stay`: a pick calls `onpick` only, so a toggle checklist (Labels, page Properties) flips the row's `check` in your `entries` and stays open, the cursor on the row and, when the menu holds the keyboard, the focus on the menu (Enter picks the next one). Escape and an outside press still close it |
| `Menu` | `flow: Flow` | `Flow::Inline` draws the same rows where you render the menu, inside your card: `div.ds-menu[data-flow=inline]` with no `.ds-popover` surface, no entrance, no overlay host, no outside-press catcher, no layer on the stack (Escape and outside presses are yours) and no focus taken; its keys work while the focus is inside it, or drive it with `active: Cursor::Controlled`. `anchor` is ignored. Fixed for the menu's life: key the menu by it to switch |
| `SidebarItem` | `place: Option<PlaceId>` | `PlaceId(String)`, your name for the place (`inbox`, `label:7`), written as `data-place` so a drag can tell which place is under the pointer |
| `SidebarItem` | `onpointerenter` / `onpointerleave` / `onpointermove` / `onpointerup: Option<EventHandler<PointerEvent>>` | The item's pointer, on every kind: set `drop: DropState::Target` on the place a dragged thread is over (lit `--accent-soft`, scaled 1.045, design/06 section 6.1) and apply the drop on the release. `drop` itself is not new |

### The mailo gaps 5 (2026-09-25)

Additive: leave a prop out and the markup is what it was, and every existing golden is unchanged
(the stylesheet golden grew by the new rules). Two things a consumer can notice: `DropState` and
`HoverCardPart` gained a variant (a `match` of yours over either needs the new arm), and `Filter`
is `Clone` but no longer `Copy` (its new variant holds a `String`: pass it by value or clone it).
FINDINGS "mailo gaps 5" has the reasons and the proofs.

| Component | Prop, type or variant | Type (default) | What it does |
| --- | --- | --- | --- |
| `Button`, `IconButton` | `propagation` | `Propagation` (`Bubble`) | `Propagation::Stop` keeps the press at the button: its propagation is stopped and its default prevented before `onclick` runs, so a header action inside a `<summary>` does not toggle the `<details>`. On Blitz both are needed (a click's default action walks up the ancestors to the summary). `Press` is unchanged: it already carries the button, the modifiers and the point |
| `Scrim` | `flow` | `Flow` (`Floating`) | `Flow::Inline` draws the scrim where you render it (`button.ds-scrim[data-flow=inline]`), at your container's stacking level: `position:absolute; inset:0` in the nearest positioned ancestor, no overlay, no layer on the stack, no Escape of its own. A press still calls `onclose`. **Layering:** it dims what your container drew before it and lies under what the container draws after it (the peeked reader) and under every floating surface; to put something above it, render it after the scrim in the same positioned container |
| `Button` | `label` | `Text` (`#[props(into)]`) | A `String`, `&str` or `"{formatted}"` as before; `Text::Runs(vec![Run::new(who, RunTone::Strong), Run::new(when, RunTone::Faint)])` draws the runs inside the label's span. A label of runs names the button by `Text::plain_text()` as `aria-label` (your `aria_label` wins). `FaceMark`'s `label` takes a `Text` the same way |
| `Menu` | `filter: Filter::Field { placeholder }` | new variant | Typing filters and `onquery` fires as under `Typing`, and the query is drawn in `div.ds-menu-filter` (`role=searchbox`), a row at the top styled as the inline `TextInput`: the placeholder while empty, then the typed text with a drawn caret. The row is not a choice: the cursor stays on the rows below. `Typing` is unchanged |
| `DropState` | `Accepts` | new variant | `data-drop="accepts"`: a place that can take the live drag while the pointer is elsewhere. On a `SidebarItem`, a dashed `--accent` hairline inside the item's box (the label does not move), quieter than `Target`. Set it on every accepting place as the drag starts, `Target` on the one under the pointer |
| `Button` | `leading` | `Option<Leading>` (`None`) | `Leading::Mark(element)` before the label, for a quire mark you build (`rsx! { ProviderMark { provider, size: MarkSize::Inline, style } }` in the From dropdown's value); `Leading::Glyph(icon)` a glyph. `span.ds-button-lead` |
| `HoverCardPart` | `FlagText { tone, icon, text: Text }`, `HoverCardPart::flag(tone, icon, impl Into<Text>)` | new variant and constructor | A flag of either tone whose words are runs (the spoof warning's brand and domain in `Strong`). Drawn exactly as `Flag`; `Flag { text: String }` is unchanged, so its literals compile |

### The mailo gaps 6 (2026-09-25)

Additive: leave a prop out and the markup is what it was. One markup change: `SidebarItem`'s
class list is `ds-sidebar-item ds-drop-place` (its drop rules moved to the shared class), which
its goldens show; a selector of yours on `.ds-sidebar-item` still matches. `Icon` gained two
variants (a `match` of yours over it needs the arms). FINDINGS "mailo gaps 6" has the reasons and
the proofs.

| Component | Prop, type or variant | Type (default) | What it does |
| --- | --- | --- | --- |
| `Icon` | `Ellipsis`, `EllipsisVertical` | new variants, in `Icon::ACTIONS` | Lucide `ellipsis` and `ellipsis-vertical` (1.47.0): a row's or a header's ⋯ on the glyph grid, in place of a typed `⋯` |
| `Button`, `IconButton` | `data` | `Vec<DataAttr>` (empty) | Your own `data-*` on the button itself: `DataAttr::new(DataName::parse("folder")?, path)` writes `data-folder="{path}"`. `DataName::parse` takes lowercase letters, digits and `-`, starting with a letter, and refuses (`PassThroughError::Reserved`) a `ds-` name and the names quire writes or reads (`variant`, `size`, `theme`, `accent`, `motion`, `material`). Build names from constants: each distinct name is interned once for the program's life. The wrapping `span` you kept for `data-folder` goes |
| `Button`, `IconButton` | `extra_class` | `Option<ExtraClass>` (`None`) | Your own class, or a space-separated list, after quire's: `ExtraClass::parse("fold-more")?` gives `class="ds-icon-button fold-more"`. A `ds-` class is refused when it is built; style yours in your own sheet (the markup lint reads your sheet for it, as for any class of yours) |
| `Scrim` | `layer` | `Option<ZLayer>` (`None`) | An inline scrim's own stacking layer, written `z-index: var(--z-…)` on it. Without one it sets no z-index, so a positioned row your pane draws after it (a row that is `position:relative` for its strip) paints over it. **Pick** a layer above your rows (`ZLayer::Raise` over rows that set none) and below your floating surfaces (the peeked reader, a menu: give those a layer above the one you picked). A floating scrim ignores it (it is on `--z-scrim`) |
| `TreeItem` | new component | | A place in a sidebar tree: `details.ds-tree-item > summary.ds-tree-item-row` in the sidebar item's chrome, children in `div.ds-tree-item-children[role=group]` one `--s-12` step in. Props: `label: impl Into<Text>`, `open: Disclosure::{Open, Closed}` (controlled), `on_toggle: EventHandler<Disclosure>` (the state a press on the row asks for; the summary's own toggle is prevented), `shape: TreeShape::{Branch, Leaf}` (a leaf is a row with no `details`, its chevron's space kept), `glyph: Option<Icon>`, `count: Option<u32>`, `here: Here`, `onselect: Option<EventHandler<Press>>` (the label becomes a button that selects without toggling), `trailing: Option<Element>` (the ⋯ `IconButton`: the slot keeps every press from the summary; `Propagation::Stop` on the button as well costs nothing), `drop: DropState`, `place: Option<PlaceId>`, and `onpointerenter`, `onpointerleave`, `onpointermove`, `onpointerup` as on `SidebarItem`. The chevron (`chevron-right`, 12) turns a quarter over `--t-quick` as it opens; the ⋯ shows on the row's hover, while its menu is open (`aria-expanded`) and under keyboard focus |
| `SidebarItem`, `TreeItem` | `.ds-drop-place` | shared class | One rule set for `data-drop="target"`, `data-drop="accepts"` and `data-drag="source"` on either item, last in the component order so it wins over their hover and current rules |

### The mailo gaps 7 (2026-09-25)

Additive but for one behaviour: under `FocusFallback::Ancestor` (the default) the keyboard no
longer goes nowhere when the element that had it is removed. One markup change: `TreeItem`'s
trailing slot carries `data-slot="trailing"`, which its goldens show. `DataName::parse("slot")`
is now refused (`PassThroughError::Reserved`). FINDINGS "mailo gaps 7" has the reasons and the
proofs; `docs/mailo-migration.md` §2 and §6.6 say what mailo deletes.

| Component or seam | Prop, type or behaviour | Type (default) | What it does |
| --- | --- | --- | --- |
| ds-native (`launch`, `Harness`) | the focused element is removed | behaviour, under `FocusFallback::Ancestor` | Blitz resets the focus to nowhere when the focused node leaves the document. ds-native now looks at the focus after each settled frame (harness) and before each window event reaches the document (window), and when the element it last saw focused is gone and the focus is nowhere, focuses the first live focusable element among: the opener a surface registered (`HostHandBack`), the removed element's own focusable ancestors (remembered while it was there), the element focused before it, and the element under the pointer when the focus moved to it. A focus cleared on purpose (a blur, a click on nothing) is left alone. `FocusFallback::BlitzDefault` turns it off. Not provided by `focus::provide()`: it needs the host's loop |
| `Menu` | closing gives the keyboard back | behaviour | A floating menu that took the keyboard (`Cursor::Auto`) and is anchored with `Anchor::Mounted(el)` registers its panel with the host as it mounts; when the panel is removed with the keyboard, `el` (or its nearest focusable ancestor, if `el` is not focusable) has it next. Anchored at a point or a rect, the host's order above applies: in practice the element focused before the menu opened, or the button whose press opened it. `Button { mounted }` gives you the button's element for the anchor |
| `HostClickFocus::restore` | liveness at the moment of focusing | behaviour | The click's fallback remembers every focusable element from the click's target up; `restore` focuses the first still in the document when it runs, so a button that removes itself on press ("Show images") leaves the keyboard on its focusable ancestor (`.app`), not on nothing |
| `ds::HostHandBack` | new seam | `HostHandBack(Rc<dyn Fn(&MountedData, &MountedData)>)` | `(surface, opener)`: when `surface` is removed with the keyboard, `opener` has it next. ds-native provides it under `Ancestor`; a webview has none. A component of yours that takes the keyboard into a floating surface may call it from the surface's `onmounted` |
| `TreeItem` | `editing` | `Option<Element>` (`None`) | An in-place rename: drawn in the label's place, in `span.ds-tree-item-label.ds-tree-item-edit[data-slot=editing]`, at the label's metrics (same left edge and width, the row's height, the count and the next row unmoved), instead of the label and its select button. Give it `TextInput { variant: FieldFace::Bare, focus: Focus::Controlled(use_focus_request().with_select_all()), onkey, .. }`: it takes the row's face, gets the keyboard with its text selected, and its `onkey` hears Enter and Escape first; take the slot away (`None`) to end the rename. A press in it never toggles or selects the row (the slot fences the click; the press's pointer-down still places the caret) |
| `TreeItem` | `[data-slot="trailing"]` | markup | The trailing slot's documented seam: style your own element inside the hover-revealed slot as `[*\|data-slot=trailing] .fold-more`. The stylesheet lint's `DsInternals` leaves `data-slot` alone (`CONSUMER_SEAMS` in `crates/ds/src/lint/selector.rs`); `.ds-tree-item-trail .fold-more` is still `DsInternals` |
| `ds::delays` | `HOVER_OPEN`, `HOVER_CLOSE`, `HOVER_WARM` | `Duration` (450, 150, 400 ms) | The hover-intent delays as constants for tests (a test waits `HOVER_OPEN` plus slack before asserting a card is open), equal to `DelayToken::HoverOpen`, `HoverClose` and `HoverWarm` at any motion level (a test holds them equal). Test-facing: components read the token |

### OSD parts (2026-09-25)

Additive (sill FINDINGS Q74 to Q76; FINDINGS "OSD parts" and "Level control"). No existing prop
changed; `Slider` is unchanged (its pointer-to-value function moved to a shared module). `Anim`
gained three variants (`Anim::ALL` is 59 long with the control center's four; a `match` over `Anim` needs the new arms).

| Component | Prop, type or variant | Type (default) | What it does |
| --- | --- | --- | --- |
| `LevelControl` | `label`, `value`, `glyph` | `String`, `Fraction`, `LevelGlyph` | A level with a glyph that follows it: `LevelGlyph::Volume(Muting::{Audible, Muted})` shows no wave at 0, then one, two, three waves by thirds, and a slash when muted; `LevelGlyph::Brightness` a sun whose rays grow with the level. Parts cross-fade over `--t-quick` |
| `LevelControl` | `mode` | `LevelMode` (`Interactive`) | `Interactive`: `role=slider`, focusable, pointer and keys (arrows step a sixteenth, Shift a sixty-fourth). `ReadOnly`: `role=progressbar`, not focusable, no handlers; `onchange` may be left out |
| `LevelControl` | `look` | `LevelLook` (`Capsule`) | `Capsule` (26 px, fill in the material's bright ink, glyph inside and knocked out by the fill), `CapsuleKnob` (a round knob at the fill's end, glyph before), `Segments` (sixteen squares filling in by `--stagger`) |
| `LevelControl` | `tick`, `availability`, `onchange` | `Tick` (`Off`), `Availability`, `EventHandler<Fraction>` | `Tick::Quiet` marks the fill's edge (`Anim::LevelTick`) each time the level crosses a sixteenth; the sound is yours |
| `LevelControl` | motion | | Set from outside: the fill slides over `--t-quick --e-out`. Under the pointer: no easing. A press swells the track (`scaleY(1.08)`, `--e-spring`); past an end the capsule stretches up to 6 px and springs back on release (not under Reduced). The track is measured once per press, after layout |
| `Osd` | `shown`, `on_hidden` | `Shown`, `EventHandler<()>` | The OSD card and its presence: shown, `data-presence=entering` (`Anim::OsdIn`) then `present`; hidden, `leaving` (`Anim::OsdOut`), then `on_hidden` at `settle(OsdOut)` and the card is `data-shown=hidden` (not laid out): unmap the surface there. A show while it fades takes the hide back (present at once, no `on_hidden`). The hold is yours (`osd.hold_ms`) |
| `Osd` | `level`, `label`, `look`, `id`, `children` | `Option<Level { value, glyph }>`, `Option<String>`, `LevelLook`, `Option<String>`, `Element` | A title line (`label`, 13/600) over a read-only `LevelControl` in `look`; `id` for the blur region (`Element("osd")`); children after |
| `Osd` | `position` | `OsdPosition` (`TopRight`) | `TopRight` (under the bar, the default) or `BottomCentre` (above the dock): which edge takes the card's margin `--osd-margin` and which way `OsdIn`/`OsdOut` move (`--osd-dy`: from above and back up at the top right, from below and back down at the bottom centre) |
| `OsdMetrics` | `margin`, `style_attr()` | `Px` (24) | `osd.margin_px`, written as `--osd-margin-px` on any element around the card |
| `Anim` | `OsdIn`, `OsdOut`, `LevelTick` | new variants | `osd-in` `--t-quick --e-out` (from `--osd-dy` at .96 and transparent, no overshoot), `osd-out` `--t-move --e-exit` forwards (to half of `--osd-dy`, transparent), `level-tick` `--t-tap --e-out` |

**Where `Osd` goes.** Directly inside one transparent Osd root, not a painted root nested in a
transparent one:

```rust
Ds { appearance, system, look, material: Material::Osd, chrome: Some(RootChrome::Transparent),
     blur: BlurState::Available, tint_alpha: Some(env.tint_alpha()), stack: Some(env.material_stack()),
    div { style: OsdMetrics { margin: Px(f32::from(osd.margin_px.0)) }.style_attr(),
        Osd { shown, label: "MA270U", level: Level { value, glyph: LevelGlyph::Volume(Muting::Audible) },
              position: OsdPosition::TopRight, id: "osd", on_hidden: move |()| unmap() }
    }
}
```

The root paints nothing and gives the card every token; the card paints the Osd material with the
Space gradient at its tint (its own `.ds-frame` group) at a control-center module's shape
(`--r-tile`, 12 padding, 296 wide). Size the surface to the card, its margins and the material's
shadow, anchored to the `position` edge; the layer margin is then 0.

### Sheet and modal parts (2026-09-25)

Additive: leave a prop out and the markup is what it was, except that a disabled `Button` or
`IconButton` now also carries `disabled="true"` and draws at .35 (the two disabled goldens gained
the attribute). `ColourToken` gained `ScrimModal` (a `match` of yours over it needs the arm).
FINDINGS "Sheet and modal parts" has the reasons and the proofs (sill Q90-Q95).

| Component | Prop, type or variant | Type (default) | What it does |
| --- | --- | --- | --- |
| `Sheet` | `shown`, `on_hidden` | `Option<Shown>` (`None`), `Option<EventHandler<()>>` (`None`) | `None`: shown while mounted, as before. `Shown::Hidden` plays `sheet-out` (`Anim::SheetOut`, `--t-move --e-exit`, forwards), fades the scrim, leaves the layer stack at once, and calls `on_hidden` at `settle(SheetOut)` (284 ms at Standard), never before: unmap the surface there. `Shown::Visible` again while it leaves enters again and `on_hidden` does not run. Mounted hidden it draws nothing (`data-presence` is `entering`, `present` or `leaving`) |
| `Sheet` | `placement` | `SheetPlacement` (`Top`) | `Top`: 36 px from the top, as before. `Centre`: centred both ways in its root (`div.ds-sheet-stage` around `.ds-sheet[data-placement=centre]`, flex, no transform), inside the same 36 px inset. Needs a root with a height: `Ds { extent: RootExtent::Viewport }` |
| `Sheet` | `scrim` | `ScrimStrength` (`Standard`) | `Modal` dims with `--scrim-modal` |
| `Scrim` | `strength` | `ScrimStrength` (`Standard`) | `Modal`: `--scrim-modal` (black .40 light, .55 dark) instead of `--scrim` (.22), `data-strength="modal"`, for a dialog that asks for a decision |
| `ColourToken` | `ScrimModal` | new variant, `--scrim-modal` | A colour like `--scrim`, since its strength differs by scheme |
| `Button` | `size` | `Option<ButtonSize>` (`None`) | `None` keeps the variant's own size (Danger and Mini are Mini-sized). `Some(ButtonSize::Regular)` draws any variant in Primary's box (8 x 14, `--fs-control` 700), so a Danger Restart stands level with Shut Down; `Some(ButtonSize::Mini)` in Mini's. `data-size` |
| `Button`, `IconButton` | `availability: Availability::Disabled` | existing prop, new look | `aria-disabled="true"` and `disabled="true"`, opacity .35 (design/13's disabled item), default cursor, `pointer-events:none`: no hover lift, wash or press squish, and `onclick` never runs |
| `Ds` | `extent` | `RootExtent` (`Content`) | `Viewport`: the root is at least the viewport (`min-height:100vh; min-width:100vw`, `data-extent="viewport"`). **Use it for every overlay surface** whose content is positioned (an OSD, a sheet, a click catcher): a `Content` root holding only such content is 0 px tall, and so is everything it places (sill F172) |
| `Kbd` | nothing | | `←` and `→` now come from Space Mono (they fell back to a system face and read as dashes at `KbdSize::Small`), and the cap is `inline-block`, so its border and padding paint on Blitz |

**What sill switches to** for the power menu: `Ds { material: Material::Sheet, extent:
RootExtent::Viewport, .. }` with `Sheet { placement: SheetPlacement::Centre, scrim:
ScrimStrength::Modal, shown, on_hidden: unmap, .. }`; `Button { size: Some(ButtonSize::Regular) }`
on the Danger actions (drop any local size CSS); `availability: Availability::Disabled` on an
action logind refuses (drop any local dimming); `Kbd { size: KbdSize::Small }` for the arrow
hints (drop any drawn-arrow workaround); and the wallpaper's `100vw/100vh` rule can become
`extent: RootExtent::Viewport` on its root.

### Button heights and small key caps (2026-09-25)

Two follow-ups from the power menu review, additive: no prop is new, so nothing you called
before changes shape. FINDINGS "Button heights and small key caps" has the reasons and the
proofs (sill Q110, Q111).

| Component | Prop, type or variant | Type (default) | What it does |
| --- | --- | --- | --- |
| `Button` | nothing | | `ButtonVariant::Primary` (and `Secondary`, which shares its rule) now carries a transparent `--hair` border rather than `border:0`; the paint is the same, but Primary, Secondary and Danger now stand the same height at a given `ButtonSize` (Primary drew 2 px shorter than a Danger beside it at Regular before this) |
| `Kbd` | nothing | | At `KbdSize::Small`, the arrow keys' (`←↑→↓`) cap draws its glyph at `--fs-control` with a tighter line-height and gains `data-glyph="arrow"` (every other key writes no `data-glyph`); the cap's own height is unchanged, only its glyph is bigger, so a Small `←` no longer reads as a dash |

### Native phase B (2026-09-25): the Blitz host for an app window

What `ds-native` gives an app that moves its window onto `ds_native::launch` (mailo Phase B).
FINDINGS.md "Native phase B" has the reasons and the proofs.

**Breaking:** `AppConfig` is a builder: `AppConfig::new("mailo", 1200, 800)`, no longer a
struct literal.

| Need | API | Notes |
| --- | --- | --- |
| Root contexts (dioxus desktop's `with_context`) | `AppConfig::with_context(value)`, `with_contexts(RootContexts)`; `Harness::with_contexts(app, viewport, contexts)`, `HarnessConfig::with_context`, `snapshot_with(app, config, moments)` | `value: Clone + Send + Sync + 'static`, read with `use_context::<T>()`; the same values reach the window, a test and a snapshot. No globals. |
| Network policy | `AppConfig::with_net(NetPolicy)`, `HarnessConfig::with_net` | `NetPolicy::{Local, Custom(Arc<dyn AppNet>), Sealed}`. `Local` (default) is what `launch` always did. Frames get `data:` only unless `Custom`'s `AppNet::decide(&NetRequest) -> NetDecision::{Allow, Deny}` admits a request (`NetRequest::origin()` is `RequestOrigin::{Top, Frame(FrameId)}`); `AppNet::fetch(request, NetReply)` then answers with `reply.bytes(..)` from any thread. ds-native never serves `file:` to a frame. |
| `<iframe srcdoc>` | nothing: the HTML parser is on in the window and the harness | A frame is a separate document: no shared DOM, no shared cascade, no scripts. `Harness::frame(selector) -> Option<FrameView>` with `id`, `text`, `html`, `count`, `text_of`, `attr`, `width`, `centre`. |
| Links clicked in a frame | `AppConfig::with_frame_links(FrameLinks)`, `HarnessConfig::with_frame_links` | `FrameLinks::{Inert, Intercept { .. }}`, built with `FrameLinks::intercept(\|link: FrameLink\| ..)`; `FrameLink { frame, tag, href, text, title }` ("Frame tags and link text" below). The frame never navigates; the handler runs with the document free. |
| Clipboard | `ds_native::clipboard::{write_text, read_text}` | `Result<_, ClipboardError::{NoHost, Unavailable}>`; call from a handler. Ctrl+C/X/V in every text field need nothing. The harness's clipboard is in memory: `Harness::clipboard_text`, `set_clipboard_text`, `selected_text`. |
| Focus an app's own element | `ds::focus_soon(element)`, `ds::focus_soon_selecting(element, Select)` | Waits out a busy document as quire's fields do: mailo's `.app` shell after a panel closes. |
| Select a field's value as it takes the focus | `use_focus_request().with_select_all()` with `TextInput { focus: Focus::Controlled(request) }` | `ds::Select::{None, All}`; the host's `HostSelect` (`ds_native::focus::SELECT`, provided by `launch`, the harness and `ds_native::focus::provide()`). A webview does nothing. |
| Desktop application id | `AppConfig::with_app_id(AppId("dev.mailo.Mailo".into()))` | The Wayland `app_id` / X11 `WM_CLASS`, for the `.desktop` match. |

The window's app renders one frame after the host, once the document has the app's providers
(a frame in the first render would otherwise be parsed with the wrong ones).

### Frame tags and link text (2026-09-25): which frame, and what a link says

Additive to "Native phase B". FINDINGS.md "Frame tags and link text" has the reasons and proofs.

| Need | API | Notes |
| --- | --- | --- |
| Name a frame | `iframe { "data-frame-tag": "msg-42", srcdoc: .. }` | Read as the frame's document is found under its element; blank means untagged. One tag per frame: a lookup returns the newest document carrying it. |
| Which frame asked | `NetRequest::frame_tag() -> Option<&FrameTag>` beside `origin()` | `None` for the app's own document and an untagged frame. A frame's requests reach `AppNet::decide` on the frame after its document is built (they wait for the tag); `data:` never waits. |
| Which frame a click came from | `FrameLink { frame, tag, href, .. }` | `tag: Option<FrameTag>`. |
| Tie an id to the app's frame | `ds_native::frames::tag_of(FrameId) -> Option<FrameTag>`, `frame_by_tag(&FrameTag) -> Option<FrameId>` | Any document on the calling thread: the window's UI thread, or a test's. A frame is forgotten with its document. |
| A stable key for a frame | `FrameId::index() -> usize` | Never reused within the process. `FrameTag::new(text)`, `as_str()`, `Display`. |
| What a clicked link says | `FrameLink { frame, tag, href, text, title }` | `text`: the anchor's text content, whitespace-collapsed (empty if the anchor is gone); `title: Option<String>`. Compare `text` with `href` for link honesty. |
| A link pill | `FrameLinks::intercept(on_click).with_hover(\|hover: FrameLinkHover\| ..)` | `FrameLinkHover { frame, tag, href, text, title, at, phase }`, `phase: HoverPhase::{Enter, Leave}`, `at: ds::Point` in the app document's coordinates (where the pointer is). Once per crossing, never per move: onto a link, off it, from one to the next (a `Leave` then an `Enter`), or out of the window. On `Inert` it reports nothing. |

**Breaking, narrowly:** `FrameLinks::Intercept` is `Intercept { click, hover }` (it was a tuple
variant); `FrameLinks::intercept(..)` and `Inert` are unchanged. `FrameLink` has two more
fields, so a struct literal of it (a test's expected value) names `tag`, `text` and `title`.

### Edit surface (2026-09-25): an app's own editor on Blitz

`ds::EditSurface` hosts an app's own rendered text (mailo's composer) and hands the app its
input; the app keeps its document model and draws its own caret and selection. FINDINGS.md
"Edit surface" has the route (no blitz fork) and the limits. mailo's adapter is in
`docs/mailo-migration.md` §6.5.

| Need | API | Notes |
| --- | --- | --- |
| A focusable block over the app's own markup | `EditSurface { on_input, on_pointer, on_focus, handle, ime_area, label, id, children }` | Mark every addressable element `"data-edit-node": "{key}"` (`ds::EDIT_NODE_ATTR`); an object the caret goes around also `"data-edit-kind": EditKind::Atom.slug()`. The surface is `white-space: pre-wrap` and shows no focus ring, caret or selection. |
| Typed input | `on_input: EventHandler<EditInput>` | `EditInput::{Text(String), Key(KeyInput { key, modifiers }), Composition(..), Paste(Pasted), Cut, Copy}`. A printable key is `Text`; Ctrl/Cmd chords and named keys are `Key`; Ctrl+V/X/C (Shift+Insert, Shift+Delete, Ctrl+Insert) are `Paste`/`Cut`/`Copy`. Tab still moves the focus. |
| IME composition | `EditInput::Composition(Composition::{Start, Update { text, cursor }, End { text }})` | `Start` comes with the first preedit; an `Update` with empty text means the preedit was cleared (winit does that before every commit); `End { text }` is the commit, empty when cancelled. Keys are withheld while a preedit shows. A commit outside a composition is `Text`. |
| Paste with HTML | `EditInput::Paste(Pasted::{Text(String), Html { html, text }})`; `ds_native::clipboard::read_html()` | The window reads `text/html` from arboard; the harness from its memory clipboard. The HTML is untrusted: sanitise it. |
| Pointer to text position | `on_pointer: EventHandler<EditPointer>` | `EditPointer { phase: PointerPhase::{Press, Drag, Release}, at, position: Option<TextPosition>, extend: Extend::{Fresh, FromAnchor}, clicks: Clicks }`. `TextPosition { node: EditNode(key), offset: TextOffset(bytes) }`: UTF-8 bytes into the element's own text; an atom is 0 (before) or 1 (after). The surface focuses itself on a press. |
| Caret, selection and surface rects | `let handle = use_edit_handle();` then `handle.caret_rect(&pos)`, `handle.selection_rects(&TextRange { anchor, focus })`, `handle.hit_test(point)`, `handle.bounds()`, `handle.focus()` | Each answers `Probe::{Found(T), Busy, Unknown}`. Window logical pixels, the same coordinates as `HostMeasure`; a caret is zero wide and its line tall; a selection is one rect per line plus each whole atom. Read a frame after the text changes. |
| The IME's candidate window | `EditSurface { ime_area: Some(caret_rect) }` | Applied while the surface has the keyboard and each time it takes it; the IME is switched on at focus and off at blur. |
| Focus in and out | `on_focus: EventHandler<EditFocus>` | `EditFocus::{In, Out}`: show or hide the app's caret. |
| The host seam | `ds::HostEdit` = `ds_native::edit::EDIT` | `launch` and the harness provide it; another Blitz host calls `ds_native::edit::provide()` (and must forward its own IME events; see FINDINGS). Without a host the geometry reads are `Unknown` and keys still arrive. |
| Tests | `Harness::ime_start()`, `ime_update(text, cursor)`, `ime_commit(text)`, `ime_end()`, `paste_html(html, text)`, `hit_test(selector, point)`, `ime_switch()`, `ime_cursor_area()`, `within(f)` | `within` runs a closure in the app's runtime, so a test reads an `EditHandle` as a handler would. `cargo run -p ds-native --example edit` prints every input from a real window, for trying an IME by hand. |

#### Edit surface 2 (2026-09-25)

| Need | API | Notes |
| --- | --- | --- |
| The surface as the app's own styled body | `EditSurface { extra_class: ExtraClass::parse("c-body").ok(), data: vec![DataAttr::new(DataName::parse("draft")?, id)] }` | The same `ExtraClass`/`DataAttr` as `Button` (mailo gaps 6): a `ds-` class, a `ds-` name or a name quire writes is refused. The class follows `ds-edit`. |
| Focus the surface from code | `handle.focus()`, `handle.blur()` | Exactly what a press and a blur do: `on_focus` hears `In`/`Out`, the IME is switched on (pointed at `ime_area`) or off, the surface is the IME's target while it has the keyboard; `blur` ends an open composition. |
| A drag past the surface | nothing: a press captures the pointer | Every move and the primary release reach `on_pointer` until the release, wherever the pointer is (positions resolve to the nearest text). |
| The caret's width | `caret_rect(..).size.width` = `--caret-w` (`ds::PixelToken::CaretW`) | 1 px floored to whole device pixels (one device pixel at 1.25-1.75), from the insertion point rightwards. Draw the caret at the rect as given, or with `width: var(--caret-w)`. |
| Stacking a selection layer | the layer before the surface, the surface `position: relative`, neither with a `z-index` | Blitz follows CSS 2.1 Appendix E here: a positioned box with `z-index: auto` paints in tree order, so the later positioned surface's text is over the layer; a static surface is under every positioned box, the layer included. FINDINGS "Edit surface 2". |
| Driving it in a test | `Harness::click_with(at, Modifiers::SHIFT)`, `pointer_move_with`, `button_down_with`, `button_up_with`, `drag(from, to, steps)`, `held_buttons() -> HeldButtons`, `Key::{Home, End, Delete, PageUp, PageDown, Insert}` | A move between a press and its release carries the held buttons, so it is a drag. |

### Native focus (2026-09-25): a field by handle, any element by selector, and keep-focus

Three gaps mailo's window hit on `ds_native::launch`. FINDINGS.md "Native focus" has the
reasons, the strip's measurement and the proofs; `docs/mailo-migration.md` §6.6 says what mailo
deletes.

| Need | API | Notes |
| --- | --- | --- |
| Focus, select or blur one of your fields from a handler | `let handle = ds::use_field_handle();` then `TextInput { handle: Some(handle), .. }`; `handle.focus(Select::All)`, `handle.blur()`, `handle.element()` | The field fills the handle as it mounts; before that every call does nothing. On Blitz the field's `onfocus`/`onblur` are called once by the handle (a host write dispatches no event); in a webview the renderer's own event fires instead. `element()` is the field's `Rc<MountedData>` (subscribes when read in render). The task runs in the handle owner's scope, so a closing panel's handler still lands it. |
| Focus an element you hold no handle for | `ds::focus_by_selector(".find input", Select::All).await -> Result<(), FocusError>` | Await it from a task of a scope that outlives the ask. Waits up to twenty frames for the element to be drawn. A `TextInput` found this way hears `onfocus` once. `FocusError::{NoHost, BadSelector { selector }, NoSuchElement { selector }, Busy, Refused { selector }}`; `NoHost` in a webview (no `HostFind`). Provided by `launch` and the harness, not by `ds_native::focus::provide()` (it needs the document). |
| The keyboard after a click on nothing focusable | nothing: `FocusFallback::Ancestor` is the default; `AppConfig::with_focus_fallback(FocusFallback::BlitzDefault)` / `HarnessConfig::with_focus_fallback` to turn it off | The nearest focusable ancestor of the click's target (a `tabindex`, or a natively focusable element such as a `button`, the target included) keeps or takes the keyboard a frame later, as in a browser, instead of Blitz clearing it; if the click left the focus somewhere (a handler moved it) that wins. A field the click leaves still hears its blur, an ancestor that already had the focus hears no blur, `dblclick` still arrives, and an `EditSurface` (which takes its own click) keeps its focus. Needs `Ds` at the root (its click handler hands the click to the host); another Blitz host provides `ds_native::CLICK_FOCUS` itself. |
| The keyboard after a click a quire control keeps to itself (mailo, v0.1.10) | nothing | A strip button, a `TreeItem`'s select button, anything in its trailing slot, a tree row, any `Propagation::Stop` control: the pressed control (or its nearest focusable ancestor) has the keyboard afterwards, as for any other click, instead of `html`. The control hands its kept click to the host itself. With the default prevented (`Propagation::Stop`, a tree row) it takes the keyboard at once, inside the click, unless a text field or `role=textbox` surface has it (a host write cannot blur it, so it keeps it). Another Blitz host provides `ds_native::PRESS_FOCUS` beside `CLICK_FOCUS`. The editing slot of a `TreeItem` no longer stops `pointerup`: your `onpointerup` above it hears a press that ends in the rename field. |
| The hover strip's rect | nothing | The strip is centred by auto margins now, so the rect `HostMeasure` reads (and a strip button's `onclick: Rect`) is where it paints on Blitz too. A hidden strip takes no hits; a shown one covers the row's centre only when it is wider than half the row. |
| Tests | `Harness::hits(point, selector)` | Whether a press at `point` lands on (or inside) the element: Blitz's own hit test, with transforms and `pointer-events` applied. |

**New seams** (ds-native provides them; a webview has none): `ds::HostBlur`
(`ds_native::focus::BLUR`, also in `focus::provide()`), `ds::HostFind { find, same }` with
`ds::Found::{Element, Missing, Busy, BadSelector}`, `ds::HostClickFocus { fallback, restore }`
with `ds::Fallback::{Renderer, Ancestor}`, and `ds::HostPressFocus` (`ds_native::PRESS_FOCUS`,
provided with `CLICK_FOCUS` under `FocusFallback::Ancestor`).

### Window frame (2026-09-25): a client-decorated window

A window that asks for no server decorations draws its own frame. FINDINGS.md "Window frame" has
the reasons, the winit and Wayland findings and the proofs; design/04 "Window frame" and design/13
section 13.3.11 the metrics and rules.

| Need | API | Notes |
| --- | --- | --- |
| The frame | `Ds { window: WindowFrame::titlebar("Mailo", TrafficLights::Shown), .. }` | The root stamps `data-window-frame="titlebar"` and is a column of `100vh`: the titlebar, then `div.ds-window-body` (`flex:1; min-height:0; position:relative`) holding your children, so your shell fills it with `height:100%`. `TrafficLights::Hidden` keeps the titlebar without the lights. Pass `timing: FrameTiming { move_threshold, menu_press, menu_hover }` from `window.move_threshold_px`, `window.tile_menu_press_ms` and `window.tile_menu_hover_ms` (design/22 section 3.11) once you read settings; `FrameTiming::default()` is their defaults. |
| No server frame on the window | `AppConfig::with_decorations(ds_native::Decorations::Client)` | Default `Server`: winit's frame (xdg-decoration's server mode on KWin and cosmic-comp, winit's own adwaita frame where a compositor has none). An app that draws `WindowFrame::Titlebar` passes `Client`. |
| The host seam | `ds::HostWindow` (`begin_move`, `begin_resize(ResizeEdge)`, `zoom(Zoom::{Toggle, Maximize, Restore})`, `minimize`, `close`, `tile(WindowTile::{Fill, LeftHalf, RightHalf, Centre}) -> Result<(), TileError::Unsupported>`, `supports(WindowTile) -> Support::{Yes, No}`, `state() -> WindowState`) | A trait, provided as `ds::WindowHost` context with `use_window_host_provider(\|\| Rc::new(host))`. `ds_native::launch` provides `ds_native::WinitWindow` for you. `ds::WindowTile` is named so because `ds::Tile` is a menu row's tile. |
| The window's state | `ds::use_window_state() -> WindowState { maximized: Maximized::{On, Off}, fullscreen: Fullscreen::{On, Off}, activated: Activation::{Active, Inactive} }` | Redraws on every change the host reports: `WindowHost::refresh()` (re-read `state()`) or `publish(state)`. `launch` refreshes on every resize and focus change. With no host: its own size, not fullscreen, active. |
| The titlebar alone (a gallery, a picture) | `WindowTitlebar { title, lights, timing, pose: TilePose::Open }` | Draws the titlebar in any root; `pose` opens the tiling menu as it mounts. |
| Tests | a stub `impl HostWindow` provided above your `Ds` | `crates/ds-native/tests/window_frame_controls.rs` is the pattern: the stub logs each request into a signal the page shows. |

**What each host implements.** ds-native: everything, with the halves and the centre
`Support::No` where winit cannot read the window's position (Wayland) and placed from
`current_monitor()` elsewhere (X11). sill, for a shell-host toplevel (shell-host master
`cb9078c`): a `HostWindow` over its `SurfaceHandle`: `begin_move()` and
`begin_resize(edge) -> Result<(), GrabRefused>` (the error dropped: the frame has nothing to do
when the compositor refuses), `zoom` as `set_maximized(Maximize::{Set, Unset})` (Toggle reads the
last `ToplevelState`), `minimize` as `set_minimized()`, `close` as sill closes its own toplevel;
`supports` is `Yes` for Fill only and `tile` answers `TileError::Unsupported` for the rest (no
compositor lets a client place a toplevel: shell-host FINDINGS F50, ours "Window frame"). Its
`ResizeEdge` has the same eight variants as `ds::ResizeEdge`. It provides the host with
`use_window_host_provider` and calls `WindowHost::publish` from `use_toplevel_state()` on each
configure (`Maximize::Set` is `Maximized::On`, `Fullscreen::Set` is `Fullscreen::On`, its
`Activation` maps variant for variant). quire does not depend on shell-host.

### App icons and the icon style (2026-09-25): what sill does

quire ships its own app icons as files and a pure re-colouring function; loading, caching and
reading the settings are `sill`'s (design/08-ICONS.md 2.11).

**The files.** `assets/icons/apps/<app>/<px>.png` is the shipped (Colour) set;
`assets/icons/apps/<app>/muted/<px>.png` the Muted set (chroma cap 0.04);
`assets/icons/apps/<app>/monochrome/<px>.png` the Monochrome set, exported **neutral grey**. Apps:
`mail`, `files`, `terminal`, `notes`, `photos`. Sizes (every one drawn directly, not
downscaled): 16, 22, 24, 32, 36, 44, 48, 64, 72, 96, 128, 256, 512. Pick the file whose size
is the icon's size times the output scale: a 48 px dock tile at scale 1 is `48.png`, at 1.5
`72.png`, at 2 `96.png`; the freedesktop `hicolor/<N>x<N>@2` of 24 is `48.png`. They are
RGBA PNGs with the plate, bevel and, from 48 px, the baked drop shadow (design/08 2.5); show
them through `IconSource::Image(ExternalIcon { url: IconUrl::file(&path)?, size })`.

**Finding them** (sill FINDINGS Q71). Do not search yourself; `ds-settings` owns the order:

| Call | Answers |
| --- | --- |
| `ds_settings::apps_dir() -> Option<PathBuf>` | the apps directory: the first of the order below that is a directory |
| `ds_settings::app_icon_path(app: &str, size: ds_settings::Px, style: ds::icon::IconStyle) -> Option<PathBuf>` | `<apps>/<app>/<px>.png`, `<app>/muted/<px>.png` or `<app>/monochrome/<px>.png`; `size` in physical pixels |
| `ds_settings::icon_assets::{candidates, find_apps_dir, find_app_icon, sizes_to_try}` | the same steps as pure functions of an `AssetsEnv` and a probe, for your own tests |

The order: `$QUIRE_ICON_ASSETS` (the apps directory itself); `$XDG_DATA_HOME/quire/icons/apps`
(`~/.local/share` when unset); each `$XDG_DATA_DIRS` entry's `quire/icons/apps`
(`/usr/local/share:/usr/share` when unset); then quire's own `assets/icons/apps` from
`ds-settings`'s `CARGO_MANIFEST_DIR` at compile time (`AssetsOrigin::DevAssets`: development
only, it exists for a path-dependency checkout and never for an installed program). The first
directory that exists wins whole. The size: the exact one of 16, 22, 24, 32, 36, 44, 48, 64, 72,
96, 128, 256, 512 (`APP_ICON_PX`), else the nearest shipped size above it, else the largest below
it (a 1024 request gets 512). An app name is lower-case letters, digits, `-` and `_`
(`AppIconName::parse`); anything else is `None`. sill's `$SILL_ICON_ASSETS` and
`../quire/assets/icons/apps` search go; `QUIRE_ICON_ASSETS` replaces the first.

**Installing them.** `cargo run -p icons -- install` copies the repository's `assets/icons/apps`
to `$XDG_DATA_HOME/quire/icons/apps` (`--from DIR` and `--to DIR` override either end); a
package copies the same tree to `/usr/share/quire/icons/apps`. Either way it is a plain copy of
the `.png` tree: `cp -r assets/icons/apps "$XDG_DATA_HOME/quire/icons/"` does the same.

**The style.** Map `IconsSettings` (ds-settings) onto the `ds::icon` pair:

| `icons.style` | our icons | third-party icons |
| --- | --- | --- |
| `Colour` | `<app>/<px>.png` | as they come |
| `Muted` | `<app>/muted/<px>.png` | `retint(.., IconStyle::Muted, _)`: chroma x 4/7 |
| `Monochrome` | `<app>/monochrome/<px>.png`, then `retint(.., IconStyle::Monochrome, tint)` | `retint(.., IconStyle::Monochrome, tint)` |

`icons.monochrome_tint` gives the `ds::icon::Tint`: `Space` -> `Tint::space(&look.dots)` (the
workspace's SpaceLook dots, the same `ds::space::derive` accent the frame uses), `Accent` ->
`Tint::from_hex(&card_accent)`, `Neutral` -> `Tint::NEUTRAL`.

**The function.** `ds::icon::retint(pixels: &mut [u8], style: ds::icon::IconStyle, tint:
ds::icon::Tint)` re-colours an RGBA8 buffer (straight alpha) in place: lightness and alpha kept;
Muted scales chroma by `ds::icon::retint::MUTED_SCALE`; Monochrome sets the tint's hue and chroma,
shaped by lightness and alpha, pulled into sRGB. `Colour` is a no-op. It takes bytes, not an image:
decode (the `image` crate, as for `classify`) and re-encode yourself.

**The plate.** A third-party icon sits on `IconView { plate: Some(PlateFamily::Neutral) }`, and
the plate is drawn by quire's stylesheet, so re-colouring the raster alone leaves a dark
`#2A2E28` plate under a tinted icon (sill FINDINGS Q72). Hand the plate the same pair:

| `icons.style` | `IconView` |
| --- | --- |
| `Colour` | `plate_tint: None` (or `PlateTint::of(IconStyle::Colour, _)`, which is `None`) |
| `Muted` | `plate_tint: Some(PlateTint::Muted)` |
| `Monochrome` | `plate_tint: Some(PlateTint::Monochrome(tint))` |

`ds::PlateTint::of(style, tint)` builds it from the pair you already pass `retint`. The plate's
two stops and its glyph ink go through `retint`'s own rule (one implementation) for the light
and the dark scheme, are written on the plate as `--plate-base-l` ... `--plate-ink-d`, and the
stylesheet picks the pair under the root's `data-theme`: nothing for you to style, nothing for
the lint to flag. `ds::icon::PlateStops::of(family, scheme).tinted(tint)` gives the colours if
you need them elsewhere (a cached composite, a tooltip swatch). The light paper stays near
white under a tint: `retint` eases chroma to nothing at white, as it does for a white icon.

**When to call it.** After loading an icon and before caching it, keyed by (icon, size, scale,
style, tint): for a third-party icon in Muted or Monochrome (inside our plate, after the 72 %
inset of design/08 4.1, so the plate and the icon take one hue), and for our own Monochrome
files. Never on a symbolic icon (it takes the text colour) and never per frame: a workspace
switch that changes the Space tint re-keys the cache, the next paint loads the new entries.

### Control center parts (2026-09-25)

Additive: four new components, four new `Anim` variants, eleven new `Icon` variants and a public
`use_entrance`; no existing golden changed (the stylesheet golden grew by the new rules and two
keyframes). A `match` of yours over `Anim` or `Icon` needs the new arms, and an array sized by
`Anim::ALL` is now 56 long. FINDINGS "Control center parts (2026-09-25)" has the reasons and the
proofs (sill Q78-Q81).

| Where | Prop, type or function | What it does |
| --- | --- | --- |
| `ModuleTile` | new component | `glyph: Icon`, `title` (`#[props(into)] Text`), `status: Option<Text>`, `state: ModuleState`, `chevron: Chevron` (`None`), `span: TileSpan` (`Half`), `onclick: EventHandler<Press>`, `on_detail: Option<EventHandler<Press>>`, `expanded: Expanded` (`Closed`, the chevron's `aria-expanded`), `availability`. A `div[role=button]` at `--r-tile` 12: a press, Enter or Space toggles (`onclick`); the chevron is its own button, `Propagation::Stop`, named "{title} details", and a press, Enter or Right on it calls `on_detail` and never `onclick`. Writes `data-state`, `data-span`, `aria-pressed` (`mixed` while busy) and `aria-busy` |
| `ModuleState` | `Off`, `On`, `Busy` | Off: a paper disc with ink on the Mini's plate. On: the disc in `--accent` with `--accent-ink`, the plate `--accent-soft`. Busy: the Off disc with the Spinner's breathe around it |
| `Chevron`, `TileSpan` | `None`/`Detail`; `Half`/`Full` | Whether the tile ends in the detail chevron; one grid column or both (a Full tile spans the `ModuleGrid`) |
| `ModuleGrid` | new component | `children`: two equal columns, gap 8 (design/13 13.3.7). Your panel keeps its own padding of 12 |
| `SettingsRow` | new component | `glyph: Option<Icon>`, `title` (`Text`), `detail: Option<Text>`, `trailing: RowTrailing` (`None`), `availability`, `onclick: EventHandler<Press>`. 44 px, a hairline above every row after the first, `MenuEntry::Row`'s shell type (title 13/400, detail `--fs-help` faint), a 16 px glyph in a 22 px column; Enter or Space runs `onclick`. Outside any `Menu`: no overlay, no layer, no focus taken |
| `RowTrailing` | `None`, `Check(Switch)`, `Toggle { value, on_toggle }`, `Chevron`, `Text(Text)`, `Glyph(Icon)` | A check in `--accent` when `On` (the row writes `aria-pressed`); a `Toggle` named by the row's title whose press and keys stay in the switch (the row's `onclick` does not run); a chevron, a value or a glyph in `--ink-faint` |
| `PaneSwitcher` | new component | `shown: Pane`, `root: Element`, `detail: Element`, `on_settled: Option<EventHandler<Pane>>`. Changing `shown` plays the arriving pane in (`slide-r` for the detail, `slide-l` for the root, `--t-move --e-spring`) and the outgoing one out the other way (`pane-out-l`/`pane-out-r`, `--t-move --e-exit`) at once; the leaving pane is out of the flow so the height is the arriving pane's; both settle at `settle(Anim::PaneInR)` and `on_settled` hears the pane. A change mid-slide reverses (a new round; the old settle is dropped). Markup: `div.ds-panes[data-shown][data-moving]` > `div.ds-pane[data-pane][data-presence]`, the leaving one `aria-hidden` |
| `Pane`, `PaneSlide`, `PaneRole`, `PaneRound` | new types | `Pane::{Root, Detail}`; the pure machine the switcher runs (`show`, `settle`, `role`), for a switcher of your own |
| `Anim` | `PaneInR`, `PaneInL`, `PaneOutL`, `PaneOutR` | `slide-r`/`slide-l` at `--t-move --e-spring` (the catalogue's rows are `--t-big`), and the two new keyframes `pane-out-l`/`pane-out-r` at `--t-move --e-exit forwards`. `Anim::ALL` is 56 |
| `use_entrance(anim) -> Presence` | now public (`ds::use_entrance`, `ds::motion::use_entrance`) | `Entering` until `settle(anim)`, then `Present`, started on mount: the entrance every floating quire surface plays, for a surface of your own. Moved from `popover.rs`; behaviour unchanged |
| `Icon` | `Play`, `Pause`, `SkipBack`, `SkipForward`, `LogOut`, `Restart` (Lucide `rotate-ccw`), `Headphones`, `Speaker`, `Mouse`, `Gamepad`, `Phone` (Lucide `smartphone`) | Lucide 1.47.0, ISC, the set's 24 grid and 2 px round stroke; `Icon::CONTROL` lists them and `Icon::ALL` ends with them. `Power` was already in the shell set |

**What sill switches to.** The control center's hand-built tile, network row and sub-page slide
become `ModuleGrid` of `ModuleTile`, `SettingsRow` and `PaneSwitcher`; delete their CSS (every
class they styled is a quire class now, and a local animation would trip `UnknownAnimation`).
Keep `shown` (and which module's detail is open) in your own state: the chevron's `on_detail`
sets it to `Pane::Detail`, the detail's back button to `Pane::Root`. Now Playing and the power
menu take the new glyphs instead of text or a local SVG.

### Control center parts 2 (2026-09-25)

sill FINDINGS Q100-Q105; FINDINGS "Control center parts 2" has the reasons and the proofs. Two
visible changes: `ModuleGrid` pads itself by 12 now, and the light level well is darker.

| Where | Prop, type or rule | What it does |
| --- | --- | --- |
| `ModulePanel` | new component: `glyph: Option<Icon>`, `title: Option<Text>`, `trailing: Option<Element>`, `span: TileSpan` (`Full`), `plate: PanelPlate` (`Tile`), `children` | A module with content on the tile's frame: `--r-tile`, the Off tile's `--surface-2` plate and hairline, padding 10/12; a header row (16 px glyph, the title at 13/600, the trailing slot at the end in the status type, tabular figures) when any of the three is given, then `children`. Not pressable: its content takes the presses |
| `PanelPlate` | `Tile`, `Bare` | `Bare`: no plate, border or padding, for a module inside a popover card that already is its plate (a bar item's dropdown) |
| `ModuleGrid` | `columns: GridColumns` (2), `gap: Px` (8), `padding: Px` (12) | From `control_center.grid_columns`, `grid_gap_px`, `grid_padding_px`; written inline (`GridMetrics::style_attr`). **Changed:** the grid pads itself; a panel that pads its panes too passes `padding: Px(0.0)` |
| `GridColumns`, `GridMetrics` | new types | `GridColumns(u16)` (0 reads as 1); `GridMetrics { columns, gap, padding }` |
| `TileSpan::Full` | now "every column" | Was "both columns"; same class, now `grid-column:1 / -1` under any column count |
| `AppearancePicker` | `layout: PickerLayout` (`Full`) | `Compact`: small segmented rows that fill the width and never pass it, for the control center's 296 px module; `Full` is unchanged |
| `Icon::Switches` | new variant, end of `Icon::SHELL` | Two toggle tracks, knobs at opposite ends: the control center's bar item. A `match` over `Icon` needs the arm |
| `--m-level-well` (light) | `rgba(0,0,0,.18)`, was `.10` | Fill/well contrast on light is now 1.66 to 1.93:1 (was 1.37 to 1.64); dark unchanged |
| Section 2 | "A document's frame must have a height" | The frame around a `Ds` root is in flow with a height: `RootExtent::Viewport` where it exists, else the floor on the frame |

**What sill switches to.** `module_box.rs` and `.sill-cc-module` (and the dropdown's
`[data-drop=module] > .sill-cc-module` rule) go: each full-width module is a `ModulePanel`, the
dropdown's with `plate: PanelPlate::Bare`. `LevelModule` passes `glyph`, `title` and the
percentage as `trailing: rsx! { "{percent}%" }` and drops `.sill-cc-level-head`, `.sill-cc-title`,
`.sill-cc-value` for it (the Battery module's head can do the same). The Appearance module passes
`layout: PickerLayout::Compact`. The panel's `ModuleGrid` takes `columns`, `gap` and `padding`
from `control_center.grid_*`; since the panel's padding also pads the detail panes, either keep
it and pass `padding: Px(0.0)` to the grid, or move `grid_padding_px` to the grid and pad the
panes on their own. `ControlCenterItem` takes `Icon::Switches` instead of `Settings`. The popup
frame of F225 is what CONSUMING section 2 now states; `dock_popup.css` and the tray's popup
should be checked against it.

### Notification parts (2026-09-25)

sill M6 (sill Q120-Q125); FINDINGS "Notification parts" has the reasons and the proofs. Additive:
six new components, four new `Anim` variants (`Anim::ALL` is 64; a `match` of yours over `Anim`
needs `BannerOut`, `BannerIn`, `PanelIn`, `PanelOut`), `Exit::BannerOut`, `RunTone::{Italic,
Underline}`, `DelayToken::SwipeQuiet`, and `Surface { chrome }`. No existing golden moved but the
stylesheet's.

| Where | Prop, type or function | What it does |
| --- | --- | --- |
| `NotificationCard` | `app: AppMark { icon: IconSource, name: Text }`, `age`, `summary` (`Text`), `body: Option<Rich>`, `count: Option<GroupCount>`, `actions: Vec<CardAction { label, on_press }>`, `on_close`, `on_open` (`EventHandler<Press>`), `on_link: Option<EventHandler<String>>`, `on_hover: Option<EventHandler<Hover>>`, `material` (`Toast`), `icon_size: IconPx` (32), `id`, `swipe: Swipe` (`Off`), `swipe_metrics: SwipeMetrics` | One notification. A plate in its material inside a transparent scope of that material, so it paints Toast in any root. First line: the summary (13/700, one line), the group's count chip (from 2), the age (data face, caption size). A press on the plate, Enter or Space opens it; the close button, the actions (Mini) and body links keep their press (`Propagation::Stop`). Under the pointer (`data-hover="on"`, told to `on_hover` so your hold timer pauses) the body opens from two lines to six over `--t-move --e-out`, the actions row opens, and the 18 px close button fades in at the top left. The body's fade is drawn only where the card measured text being cut (`data-clip` `rest` or `always`). `id` names the plate for its blur region (`Element("toast-<id>")`) |
| `GroupCount`, `Layers` | `{ count: u32, layers: Layers(u8) }` | The chip and up to `Layers::MAX` (3) plates behind the card, each `--notifications-group-offset` (4) lower and 6 narrower each side |
| `Hover` | `Away`, `Over` | What `on_hover` hears |
| `Swipe` | `Off`, `Dismiss(EventHandler<()>)` | Swipe to dismiss: a drag follows 1:1 right, a quarter left; released under 80 px and 600 px/s it springs back (`--t-move --e-spring`; snaps under Reduced), past either it flies out right from where it is (`Anim::BannerOut`). A horizontal scroll is summed into the same offset and decided once no delta has come for `DelayToken::SwipeQuiet` (120 ms; Blitz carries no scroll phase). The click that ends a drag never opens the card. Alone, the card reports at `settle(BannerOut)`; in a `BannerStack` it reports at the release and its row carries it out |
| `SwipeMetrics` | `{ dismiss: Px, velocity: Speed, damping: Fraction }` (80, 600, 250) | From `notifications.swipe_dismiss_px`, `swipe_dismiss_velocity_px_s`, `swipe_damping` |
| `SwipeState`, `SwipeInput`, `SwipeEffect`, `SwipeLook`, `Click`, `Stamp`, `Speed`; `use_swipe`, `Swiper`, `Held` | the pure machine and its hook | For a swipe of your own; `SwipeState::step(input, metrics)` is table-tested |
| `BannerStack` | `banners: Vec<Banner { key: BannerKey(u32), card: Element }>`, `position: BannerPosition::{TopRight, BottomRight}`, `entry: BannerEntry::{FromRight, FromBelow}`, `gap: Option<Px>`, `on_hidden: Option<EventHandler<BannerKey>>` | The banners you list, newest first (a column, or one running upwards at the bottom right). An arrival slides in from `entry`'s edge (`Anim::BannerIn`, `--t-move --e-spring`; map `notifications.banner_entry_direction` 1:1, default from the right); a banner you stop listing slides back out that way (`Anim::BannerOut`, `--t-move --e-exit`; a swiped one always to the right, along the swipe) and stays drawn until `settle(BannerOut)`, then the banners after it heal into its place by the height it measured, and `on_hidden` hears its key: unmap the surface there when your list is empty. Listed again while it leaves, it stays. The hold timer, the cap (`stack_max`) and grouping are yours. Rows are `div.ds-banner[data-banner]` |
| `GroupHeader` | `icon: IconSource`, `name` (`Text`), `count: u32`, `expanded: Expanded` (`Closed`), `on_toggle`, `on_clear` | One app's head in the center: the icon at 16, the name and count in `SectionHeader`'s group type, then Quiet "N more" (folded) or "Show less" (open) and "Clear" that keep their press |
| `Panel` | `label`, `shown: Shown`, `on_hidden`, `onclose: Option<EventHandler<()>>`, `width: Px` (384), `edge: PanelEdge::Right`, `material` (`Popover`), `scrim: PanelScrim::{None, Dim(ScrimStrength)}`, `children` | The notification center: 8 in from the top, bottom and right edge, full height, scrolling its content, in the Popover material (design/20 §1.6). Shown, `Anim::PanelIn` (`--t-move --e-out`); hidden, `Anim::PanelOut` (`--t-move --e-exit`) and `on_hidden` at `settle(PanelOut)`; a show while it leaves takes the hide back. Escape inside it and a press on its scrim call `onclose`. Needs a root with a height (`RootExtent::Viewport`) |
| `Rich`, `RichRun`, `RichText` | `Rich(Vec<RichRun::{Run(Run), Link { text, href }}>)`; `RichText { body, on_link }` | A body with links: a link is `a.ds-run-link` whose press or Enter stops there and hands the href to `on_link`. A `Text`, `String` or `&str` converts, and an optional `Rich` prop takes them as `Some` |
| `RunTone` | `Italic`, `Underline` | `span.ds-run[data-tone=italic]` and `[data-tone=underline]` (a body's `<i>`, `<u>`) |
| `NotificationMetrics` | `banner_width`, `banner_min_height`, `banner_padding`, `icon`, `close`, `stack_gap`, `group_offset`, `center_width` (`Px`); `style_attr()` | The eight `notifications.*` geometry keys as `--notifications-*` tokens, written once on any element around the banners or the center (the center's width held to 280-600) |
| `Surface` | `chrome: Option<RootChrome>` | `Some(RootChrome::Transparent)`: the scope's own box paints nothing and cards inside paint its material |
| `Harness::wheel(at, dx, dy)` | ds-native | A wheel delta where the pointer is, with winit's sign (positive `dx` is content moving right), as the window delivers it |

**What sill switches to.** The banner surface: one transparent Toast root (`Ds { material:
Material::Toast, extent: RootExtent::Viewport, .. }`) with `NotificationMetrics` from
`notifications.*` on a wrapper, holding `BannerStack { banners, position: banner_position,
entry: banner_entry_direction, on_hidden }`; each banner a `NotificationCard { swipe: Swipe::Dismiss(..), swipe_metrics, on_hover:
pause or resume the hold, id: "toast-<id>" }`, the server's body markup parsed into a `Rich`. Drop
the local banner card, its hover expand, its close button, its slide and its stack CSS; unmap
when `on_hidden` finds the list empty. `notifications.swipe = KeepInCenter` is yours to honour in
`on_dismiss` (move it to the history, do not drop it). The center: a Popover root holding
`Panel { shown, on_hidden: unmap, width: Px(center_width_px), onclose }` of `GroupHeader`s over
`NotificationCard`s (`Swipe::Off` or `Dismiss` as you choose).

### Month grid (2026-09-26)

sill Q180; design/04-COMPONENTS.md section 39. Additive: one component and its data types; no
existing golden moved but the stylesheet's.

| Where | Prop, type or function | What it does |
| --- | --- | --- |
| `MonthGrid` | `data: MonthGridData`, `weeks: WeekNumbers` (`Hide`), `onstep: Option<EventHandler<Step>>`, `onpick: Option<EventHandler<DayKey>>` | One month in seven 32 px columns: the title (data face, upper, `--accent`), the previous and next `IconButton { Tool }` only with `onstep` (they hand it `Step::Previous`/`Step::Next`; you shift the month), the weekday heads, then the weeks, led by their ISO week under `WeekNumbers::Show`. The neighbours' days `--ink-faint`, today on an `--accent` disc (`aria-current="date"`), a busy day's 4 px dot. With `onpick` each day is a `button` (`data-kind="pressable"`) that hands over its key. A change of `data.month` slides the new weeks in once, from the right for a later month (`a-slide-r`), from the left for an earlier one (`a-slide-l`), dropped at `settle(Anim::SlideR)`; the first month and a render that keeps the month play nothing, so you say nothing about the direction |
| `MonthGridData` | `{ month: MonthKey, title: Text, heads: [Text; 7], weeks: Vec<MonthWeek> }` | Your grid, with the words you format: "September 2026", the weekday initials from your first weekday |
| `MonthWeek`, `MonthDay` | `{ number: IsoWeek, days: [MonthDay; 7] }`; `{ key: DayKey, place: DayPlace, mark: DayMark, events: Eventful }` | Field for field your `GridWeek`/`GridDay`; the cell's label is `key.day` |
| `MonthKey`, `DayKey`, `IsoWeek` | `{ year: i16, month: i8 }`, `{ year: i16, month: i8, day: i8 }`, `IsoWeek(i8)` | The civil types as jiff hands them (`Date::year()`, `month()`, `day()`), so each `From` is a field copy |
| `DayPlace`, `DayMark`, `Eventful`, `WeekNumbers`, `Step` | `Before`/`InMonth`/`After`, `Today`/`Plain`, `Busy`/`Free`, `Hide`/`Show`, `Previous`/`Next` | Your grid's enums and the `calendar.week_numbers` setting, variant for variant |

**What sill switches to.** The calendar widget draws `MonthGrid` from `From<&grid::MonthGrid>`
(title and heads formatted by you); `onstep` shifts the month (`shift_month`) and recomputes the
grid; drop the widget's own grid CSS.

**Density (sill Q190, 2026-09-26).** Additive: `MonthGrid` gains `density: MonthDensity`
(`Auto`), and `WidgetFrame` now provides its size to its content. Every `.ds-month` now carries
`data-density`, so a golden of yours holding a grid gains `data-density="regular"`; quire's
three month goldens moved by that attribute alone, and the stylesheet's by the new rules.

| Where | Prop or type | What it does |
| --- | --- | --- |
| `MonthGrid` | `density: MonthDensity` (`Auto`) | Written as `data-density=regular\|compact`. `Auto` follows the enclosing `WidgetFrame`: compact inside `size: Small`, regular inside `Medium`/`Large` or outside any frame. `Regular`/`Compact` force it |
| `MonthDensity` | `Auto`/`Regular`/`Compact` | Compact is seven 18 px columns, a 14 px header (title at `--fs-micro`, 14 px glyph step buttons `ds-month-step` in place of the Tool buttons), 10 px heads, a 16 px today disc and a 3 px dot: 126 x 132 for a six-week month, inside a small desktop frame's 132 x 132 content box. It never shows week numbers, whatever `weeks` says (`calendar.week_numbers` applies to the regular grid only) |

**What sill switches to.** Put the calendar's `MonthGrid` straight in its `WidgetFrame` and say
nothing about density: the small widget draws compact, the medium and large regular. Style
nothing under `.ds-month`.
### Screenshot thumbnail (2026-09-26)

sill Q181; design/04-COMPONENTS.md section 41. Additive: two new components, two new `Anim`
variants (`Anim::ALL` is 66; a `match` of yours over `Anim` needs `ShotIn`, `ShotOut`),
`DRAG_THRESHOLD`, and constructors on the existing `ImageSource`. No existing golden moved but
the stylesheet's.

| Where | Prop, type or function | What it does |
| --- | --- | --- |
| `ShotThumbnail` | `image: ImageSource`, `size: ImageSize`, `shown: Shown`, `on_hidden: EventHandler<()>`, `width: Px` (240), `actions: Vec<ThumbAction { icon: Icon, label: Text, onpress: EventHandler<()> }>`, `onopen: Option<EventHandler<()>>`, `ondrag: Option<EventHandler<DragStart>>`, `onhover: Option<EventHandler<Hover>>`, `id: Option<String>`, `swipe: Swipe` (`Off`), `swipe_metrics: SwipeMetrics` | The picture letterboxed in a Toast-material card (the box follows the picture between 2:1 and 16:10, inside a 4 px mat). Shown, `Anim::ShotIn` (`rise` at `--t-big --e-spring`); hidden, `Anim::ShotOut` (a slide right, `--t-move --e-exit`) and `on_hidden` at `settle(ShotOut)`; a show while it leaves takes the hide back. The hold is yours: pause it on `onhover(Hover::Over)`. Hover shows the actions; a click on the picture (or Enter, Space) opens; a press that travels `DRAG_THRESHOLD` calls `ondrag` once and its click does not open. `swipe: Swipe::Dismiss(..)` is `NotificationCard`'s swipe to dismiss, same machine and metrics; with it on a rightward press is the swipe's, never a drag out. No position of its own: it composes in a `BannerStack { position: BottomRight }`. Reduced motion is handled inside (no spring back; fades rather than rise and slide); nothing is left for your CSS |
| `ShotGhost` | `image`, `size` | The card at 120 wide and .8 opaque, for your drag icon |
| `DragStart` | `{ from: Point, at: Point }` | The press point and where it crossed the threshold, client coordinates |
| `ImageSource` | `ImageSource(String)`; `::file(&Path) -> Result<_, DsError>`, `::png(&[u8])` | A `data:` URI or `file:` URL, written as an `<img>`'s `src` (now also ProviderMark's, moved to `components::image_source`) |
| `ImageSize` | `{ width: u32, height: u32 }` | The picture's pixels; only the ratio is read |
| `DRAG_THRESHOLD` | `Px(8.0)` | The Manhattan travel that makes a press a drag (section 34) |

### Lock, polkit and switcher (2026-09-26)

M11; design/04-COMPONENTS.md sections 42 and 43; design/20 sections 1.9 to 1.11; design/13
section 13.3.5. Additive: new components and vocabulary, a narrow `Sheet` width, two avatar
sizes (`Size48`, `Size64`: a `match` of yours over `AvatarSize` needs them), two glyphs
(`Icon::ArrowRight`, `Icon::CapsLock`, in `Icon::ACTIONS`), five colour tokens (`--lock-ink`,
`--lock-ink-soft`, `--lock-glass`, `--lock-glass-strong`, `--lock-veil`) and one size
(`--fs-lock-clock`, 140). No existing golden moved but the stylesheet's.

| Where | Prop, type or function | What it does |
| --- | --- | --- |
| `LockScreen` | `wallpaper: Option<ImageSource>`, `clock: Element`, `prompt: Element` | The lock surface's stage: the wallpaper covering it under `--lock-veil` (pass it already blurred; Blitz blurs nothing), the clock 7 % of the width from the top, the prompt near the bottom, centred. Put it in `Ds { material: Material::Window, extent: RootExtent::Viewport }`, one per output |
| `LockClock` | `time: String`, `date: String`, `look: LockLook` (`Clear`) | The date, then the time in the display face at `--fs-lock-clock` 700, white. You word both (locale, 12/24 h) and re-render each minute. `LockLook::Space` sets the date in a pill of the Space gradient |
| `LockPrompt` | `user: LockUser { name: String, picture: UserPicture }` (`LockUser::new(name, picture)`), `state: PromptState` (`Idle`), `caps: CapsLock` (`Off`), `look: LockLook` (`Clear`), `placeholder: Option<String>` ("Enter Password"), `hint: Option<Text>`, `wake: Option<WakeStamp>`, `oninput: EventHandler<String>`, `onsubmit: EventHandler<String>` | The person's picture at 64 (a face whatever size it carries, a photo cropped round, or the persona playing the prompt's own mood; see "Lock picture" below), the name, a 260 x 38 pill of flat white glass holding a `Secret` field (its text is never in the markup: there is no `value` prop), the caps-lock mark, an enter arrow that shows once something is typed, and the hint line. Enter or the arrow calls `onsubmit(text)` (never with an empty text); Escape empties the field and calls `oninput("")`. The field takes the keyboard as it mounts and again after each emptying. `LockLook::Space` paints the pill with the Space gradient |
| `PromptState` | `Idle`, `Checking`, `Wrong`, `LockedOut { until: String }`, `Accepted` | Yours to move. `Accepted` (the password was right): the field stays closed and a persona plays Happy; unlock at `settle(Anim::PersonaHop)`. `Checking`: the field and the submit are closed, the arrow spins. `Wrong`: the field plays `shake-x` once (`settle(Anim::ShakeX)`, 420 ms) and empties itself when it settles, calling `oninput("")`; it shakes again only after the state has been something else (go through `Checking` for the next try). `LockedOut`: closed, the hint line says "Try again at {until}" |
| `CapsLock` | `On`, `Off` | Read the modifier state from the keyboard and pass it; `On` draws the caps-lock arrow in the field |
| `LockLook` | `Clear`, `Space` | Where the Space's colour reaches (the date pill and the field); the time stays white |
| `PolkitPrompt` | `action: Text`, `detail: Option<Text>`, `title: Option<String>` ("Authentication Required"), `user: LockUser`, `state: PromptState`, `caps: CapsLock`, `oninput`, `onsubmit`, `oncancel: EventHandler<()>`, `shown: Option<Shown>`, `on_hidden: Option<EventHandler<()>>` | A narrow centred `Sheet` over the modal scrim (`peek-in` in, `sheet-out` out, `on_hidden` at its settle): the person's picture at 48 (a persona there stays idle), the title, `action`, "Details" with `detail` as a hover card, the name over a boxed `Secret` field, Cancel and Authenticate at equal width. Enter or Authenticate submits; Cancel, Escape and the scrim call `oncancel`. `Wrong` shakes the field once and empties it; `LockedOut` shows "Too many tries. Try again at {until}." in `--danger`. Put it in `Ds { material: Material::Sheet, extent: RootExtent::Viewport }` |
| `Sheet` | `width: SheetWidth` (`Regular`) | `SheetWidth::Narrow` draws the sheet `min(340px, 88%)` wide (`data-width="narrow"`); a regular sheet's markup is unchanged |
| `AppSwitcher` | `apps: Vec<SwitcherApp>`, `selected: AppKey`, `output: Option<Px>`, `metrics: SwitcherMetrics`, `onhover: EventHandler<AppKey>`, `onactivate: EventHandler<AppKey>` | The Cmd+Tab row on the Osd material, painted as the OSD card is (the Space gradient at the frame alpha). Cells of `metrics.cell` with icons of `metrics.icon`, `metrics.gap` apart, padding 16; the selection a `--f-pill` square behind the selected cell, moving with `--t-quick --e-spring`; the selected app's name under it (its `Tooltip { Fly }`, shown by the switcher). Past `output - 64` the icons shrink, down to `metrics.min_icon`, then the row scrolls with the selection centred where it can be. The pointer entering a tile calls `onhover(key)`, a click `onactivate(key)`: the selection is yours to move. It fades in over `--t-quick`. The show delay, the keys and the modifier's release are yours. Put it in `Ds { material: Material::Osd, chrome: Some(RootChrome::Transparent) }` |
| `SwitcherApp` | `{ key: AppKey, name: String, icon: IconSource, plate: Option<PlateFamily>, presence: TilePresence }`; `SwitcherApp::new(key, name, icon)` | One tile. A glyph is drawn at the cell's icon size (on `plate` when given); an external icon is stretched to that square, so resolve it at `metrics.icon`. `presence: TilePresence::Leaving` plays `fold` once (Q: the app quits); drop it from `apps` at `settle(Anim::Fold)` |
| `SwitcherMetrics` | `{ icon: Px, cell: Px, gap: Px, min_icon: Px }`, default 96, 112, 8, 48 | From `switcher.icon_size_px`, `cell_size_px`, `cell_gap_px`, `overflow_min_icon_px` |
| `switcher_fit` | `fn(count, selected, SwitcherMetrics, Option<Px>) -> SwitcherFit { icon, cell, view, shift }` | The row's fit, pure, if you need the numbers (to size the surface: the panel is `view + 32` wide) |

**What sill switches to.** The lock surface draws `LockScreen { clock: LockClock, prompt:
LockPrompt }` and moves `PromptState` as PAM answers; the polkit agent draws `PolkitPrompt`;
the switcher surface draws `AppSwitcher` after its 150 ms timer and maps keys to `selected`.
Style nothing under `.ds-lock*`, `.ds-polkit*` or `.ds-switcher*`.

### Persona (2026-09-26): the user's animated character

design/24-PERSONA.md. `Persona { spec: PersonaSpec, size: PersonaSize::{Small, Medium, Large},
mood: Mood, wake: WakeStamp, finish: PersonaFinish }` draws the user's character (28, 64 or
128 px) on its disc. `PersonaSpec` is user data (serde, every field defaults): save it as it is;
`PersonaSpec::from_seed(u64)` gives a pleasant random one and `PersonaSpec::default()` a blob.

| You want | Pass |
| --- | --- |
| At rest | `Mood::Idle`: blinks and breathes for 20 s after mount, a new `wake` or a mood change, then paints nothing |
| The user is typing | `Mood::Attentive` |
| A wrong password | `Mood::Wince` (shakes once and holds; set `Attentive` when typing resumes) |
| Unlocked | `Mood::Happy` (one hop) |
| The display is off | `Mood::Asleep` |
| Wake it (pointer moved, a key) | `wake: stamp.next()` |
| The icons are muted | `finish: PersonaFinish::Muted` |

Reduced motion is quire's: moods change at once and nothing blinks. A surface that can show
a letter, a photo or a persona takes `UserPicture::{Face(AvatarFace), Photo(ImageSource),
Persona(PersonaSpec)}` and draws it with `UserPortrait { picture, size, mood, wake }`; the lock
and polkit prompts take one in `LockUser` ("Lock picture" below). Never style `.ds-persona*`.

### Lock picture (2026-09-26): photo, persona and moods in the lock and polkit prompts

design/04 section 42; design/24 section 4. **Breaking, one line:** `LockUser`'s `avatar:
AvatarFace` is now `picture: UserPicture`. Write

```rust
LockUser::new(name, face)            // was: LockUser { name, avatar: face }
```

(`LockUser::new(name: impl Into<String>, picture: impl Into<UserPicture>)`; or
`LockUser { name, picture: face.into() }`). `PromptState` gained `Accepted`, so an exhaustive
`match` of yours over it needs the arm. `UserPicture` is no longer `Copy` (a photo holds a
`String`): `.clone()` where you copied one.

| Where | Prop, type or function | What it does |
| --- | --- | --- |
| `UserPicture` | `Face(AvatarFace)`, `Photo(ImageSource)`, `Persona(PersonaSpec)`; `From` each | `Photo` is the user's own picture: `$HOME/.face` or AccountsService's `IconFile`, as `ImageSource::file(path)`. Drawn in a disc (`div.ds-user-photo[data-size]` clipping an `img` with `object-fit: cover`), at the face's size: 64 at the lock, 48 in the polkit sheet, `size` in `UserPortrait` |
| `LockPrompt` mood | from its own state | A persona is `Attentive` while the field holds text and has the caret, and while `Checking`; `Wince` when `Wrong` (once per wrong: typing again turns it attentive, the next `Wrong` winces once more, never harder); `Happy` when `Accepted`; `Idle` otherwise. Set nothing: move `PromptState` as before |
| `LockPrompt` wake | `wake: Option<WakeStamp>` | A key, a pointer move or a press in the prompt wakes the persona (at most once a second), so it rests again 20 s after the last activity. Pass `Some(stamp.next())` to wake it from outside, e.g. when the display comes back on |
| Faces and photos | | Have no moods and ignore `wake` |

**The unlock, with a persona.** `onsubmit(text)` → `state: Checking` → PAM says yes →
`state: Accepted` → after `settle(Anim::PersonaHop, level, StaggerIndex::default())` unlock
(and fade the surface). With a face or photo `Accepted` just holds the field closed.

### PDF thumbnail (2026-09-26): a PDF's first page from a path

sill M9 launcher v2; design/04-COMPONENTS.md section 45. Additive: two components, two enums,
new classes (`ds-pdf-thumb`, `ds-pdf-thumb-sheet`, `ds-pdf-thumb-page`, `ds-pdf-thumb-plate`) and
attributes (`data-state` on `.ds-pdf-thumb`, `data-trouble` on the plate), and a ds-native cargo
feature. No existing golden moved but the stylesheet's.

| Where | Prop, type or function | What it does |
| --- | --- | --- |
| `ds_native::PdfFileThumb` (feature `pdf-thumb`) | `path: PathBuf`, `size: Size`, `label: Option<String>` | The first page of the PDF at `path`, fitted into `size` at its own aspect on a white sheet with a hairline edge. Read and rasterised on one `pdf-thumb` worker thread with a latest-wins queue (a new path replaces this thumbnail's request that has not started, so arrowing through a list runs a raster or two, not one per step), cached by path, modification time, device size and scale; you never rasterise. Loading shows nothing for 400 ms, then the dimmed blank sheet; no pages or an empty file, the blank sheet; not a PDF, missing, or password-locked, a file (or lock) glyph on a red plate. `label` is what a screen reader reads (pass the file name) |
| `ds::PdfThumb` | `page: PdfPage`, `size: Size`, `label` | The drawing alone, for a page you already have: `PdfPage::{Loading, Ready { image: ImageSource, sheet: ImageSize }, Empty, Failed(PdfTrouble::{Unreadable, Locked})}` |
| `ds_native::pdf_thumb_blocking` | `&ThumbRequest { path, size, scale }` -> `PdfPage` | The same page, from the cache or read now on the calling thread (a Quick Look worker's own prefetch); blocks, so never on the UI thread. `pdf_thumb_cached` answers from the cache only; `pdf_thumb_bytes(bytes, DeviceBox)` rasterises bytes with no file and no cache |
| Cargo | `ds-native = { …, features = ["pdf-thumb"] }` | Brings in pdfrum (already in the pinned block); off by default |

Never style `.ds-pdf-thumb*`; size it with `size`, not CSS.

### Colour emoji (2026-09-26): `--font-emoji` and `.ds-emoji-text`

FINDINGS.md "Colour emoji". Blitz paints the system's Noto Color Emoji (COLRv1) in colour on
both renderers, but fontique's own fallback picks Symbola (monochrome, no skin tones, flags or
families). Your CSS may not name a `font-family` (the lint), so quire carries the stack.
Additive: one token, one utility class; the stylesheet's golden moved, no other golden did.

| Where | What | What it does |
| --- | --- | --- |
| Any text quire draws | nothing | The System stacks (`--font-display`, `--font-ui`, `--font-data`) and Editorial's display and UI stacks now name `"Noto Color Emoji"` after their text faces, so an emoji in a label, a notification or a name paints in colour. Gallery snapshots of every page in both typefaces were compared before and after: no glyph moved (the only differences are the widget rings' and one list row's run-to-run noise, present between two baseline runs too). Editorial's data face and `--font-code` (Space Mono) are unchanged |
| An emoji grid, a reaction, text that is mostly emoji | `class: "ds-emoji-text"` | `font-family: var(--font-emoji)`: `"Inter","Noto Color Emoji",system-ui,sans-serif`. Inter comes first on purpose: Noto Color Emoji maps the digits, `#` and `*` to empty keycap bases, so an emoji-first stack draws "2#" as nothing |
| Your own component's CSS | `font-family: var(--font-emoji)` | Accepted by the lint like the other face tokens |
| The user's picture picker | `AnimatedEmoji { playback: EmojiPlayback::Still }` for the 42 shipped picks | Those cells must match the picture they become |

`.ds-emoji` is AnimatedEmoji's own root class; the text utility is `.ds-emoji-text`, not
`.ds-emoji`.

### Animated emoji (2026-09-26): the user's picture as a moving emoji

design/25-EMOJI.md. `AnimatedEmoji { emoji: EmojiId, size: PersonaSize, mood: Mood, wake:
WakeStamp, disc: EmojiDisc }` draws the emoji the user picked (28, 64 or 128 px) and plays it
for 20 s after mount, a new `wake`, a mood change or a new pick, then rests on its first frame
and paints nothing. Drive it exactly as a `Persona` (the same `Mood` and `WakeStamp`).

| You want | Pass |
| --- | --- |
| The user's choice | `emoji: EmojiId` (user data, serde as its slug: `"heart-eyes"`); `EmojiId::default()` is a smiling face; `EmojiId::ALL` is the picker's list, `.text()` and `.name()` its labels |
| A wrong password | `Mood::Wince`: the confounded face once through, then the pick |
| Unlocked | `Mood::Happy`: the partying face once through |
| The user is typing | `Mood::Attentive`: the pick, playing |
| The display is off | `Mood::Asleep`: the sleeping face, still |
| Wake it | `wake: stamp.next()` |
| A disc behind it | `disc: EmojiDisc::Tinted(Backdrop::Teal)` (default `EmojiDisc::None`) |
| A picker's grid of the set | `playback: EmojiPlayback::Still`: rest frames only |

Reduced motion is quire's: still frames only. **Credit**: the frames are Noto Animated Emoji,
CC BY 4.0; a product that shows them puts `ds::EMOJI_ATTRIBUTION` in its about box or credits
(design/25 section 2). `UserPicture` does not take an emoji yet (design/25 section 8). Never
style `.ds-emoji*`.

### Widget vibrancy (2026-09-26)

design/23-WIDGETS.md sections 1.1 (M26-M34) and 4.3. Values only: no class, attribute or prop
changed; the stylesheet's golden moved. The light `Widget` material now paints a neutral grey,
`rgba(228,228,228,.60)` over blur and `.94` without, a 1 px dark rim outside (`--m-hairline` at
`var(--hair)`, both schemes), a 1 px white rim at .10 inside (`--m-edge`) and a short drop
`0 2px 8px` black .12 (`--m-shadow`); the dark widget is unchanged but for the 1 px rim.

| Where | What to do | Why |
| --- | --- | --- |
| The widget frames' blur region (sill) | Keep requesting `ext-background-effect` blur behind each desktop widget card; set the compositor's blur to a Gaussian of **sigma 22 logical pixels** (44 device pixels at 2x), or the nearest strength that spreads a sharp edge 10-90 % over about 56 logical pixels | The reference card's blur, fitted (M26). Without blur the card paints `--m-tint-solid`, the same grey near-opaque |
| cosmic-gaps (sill) | Add "saturation x1.8 of what shows through a widget's blur" | Measured on the reference (M29, low confidence); a tint can only mix, so the boost is the compositor's or nobody's |
| Nothing else | The tint's alpha stays .60, above the reference's .48, because the legibility gate needs `--ink` at 4.5:1 over a black backdrop (design/23 section 1.1, "What quire paints") | A lower alpha is a change to the gate, the user's call |

### Inter, the system typeface (2026-09-26)

design/02-TYPE.md section 2; design/22-SETTINGS.md `appearance.typeface`. The desktop's type is
Inter now: `Ds` takes `typeface: Option<Typeface>` and stamps `data-typeface` on its `div.ds`
(every `Surface` re-stamps the root's). `None`, the default, takes the enclosing root's, so a
root nested in another speaks as it does and a top-level root is `Typeface::System`. Under System
`--font-ui` and `--font-data` are Inter (the data face always tabular), `--font-display` is
Inter Display (Inter's `opsz` 32 cut); `Typeface::Editorial` maps them back to Bricolage
Grotesque, Karla and Space Mono exactly as before. A new `--font-code` is Space Mono in both, and
`Kbd` uses it. Additive only: no class, attribute or prop was removed or renamed.

| Where | What to do | Why |
| --- | --- | --- |
| Every root the shell draws | Nothing, or pass `typeface: Some(env.settings.appearance.typeface())` to follow the setting | System is the default |
| mailo's roots | `typeface: Some(Typeface::Editorial)` on its top-level `Ds` (nested roots inherit it) | mail keeps its voice with one prop |
| Your CSS | `font-family:var(--font-code)` for code and aligned logs; keep `var(--font-data)` for times, counts and labels | the data face is Inter under System; a fixed pitch is only in `--font-code` |
| Text-width estimates | Re-measure: Inter's lowercase and digits average 0.552 em at 400 (7.18 px at 13 px) against Karla's 0.527 em (6.86 px); capitals 0.68 em against 0.60 em; data text is narrower than Space Mono's 0.612 em | Inter is wider than Karla, narrower than Space Mono |
| Widget and lock sizes | Re-measure anything sized from `--fs-dial` (9 to 8), `--fs-dial-large` (18 to 16.5), `--fs-widget-figure` (20 to 18), `--fs-widget-hero` (47 to 42.5) or `--fs-lock-clock` (140 to 127) under System | they follow the typeface so Inter Display's taller caps (.7275 em against .66) draw the measured cap heights; Editorial keeps the old sizes |
| New tokens | `--font-code`; the voice tokens `--tracking-heading`, `--tracking-lock-clock`, `--tracking-lock-date`, `--tracking-caps`, `--tracking-caps-narrow`, `--fw-caps`, `--fs-caps`, `--tracking-mono`, `--fs-mono` (design/02 section 2.1) | values that follow the typeface |

### PDF and printing (2026-09-25)

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
| `filter: saturate()`/any colour-matrix filter | S16 | precompute the effect into a colour token instead of a runtime filter |
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

## 9. Two gaps `examples/consumer` found, both now fixed in quire

Neither is a Blitz limit (section 8's table); both are in quire's own implementation, found
while wiring up a real, tested consumer app.

**Opening a floating component (`Menu`, `Popover`, `HoverCard`, `CommandPalette`) and then
driving it through `ds_native::Harness` used to panic — fixed, not worked around, credit where
it is due.** `ds::components::popover::use_float` — what every floating component is built on —
mounts rect-measuring probes on every open. Reading a mounted element's rect one frame later,
through a task the render spawns, could land inside `dioxus-native-dom`'s own event-dispatch
pass while it already held the document's `RefCell`: `thread '...' panicked ... RefCell already
borrowed`, at `dioxus-native-dom/src/events.rs:161`. `examples/consumer` hit this first (one
`harness.click()` on a button that opens a `Menu`, nothing more), and quire's own wave 2
integration root-caused and fixed it in the same session: `ds::geometry::measure` (and every
component built on it) now reads through a `HostMeasure`/`Measured` seam
(`crates/ds/src/geometry/measure.rs`, `crates/ds-native/src/measure.rs`) that answers `Busy`
rather than reading a document mid-render, so the reader waits a frame instead of re-entering
the borrow. `examples/consumer/tests/coherence.rs::the_menu_opens_and_closes_under_harness` is
the regression test, and `Page`'s own "More" button anchors its menu to itself (`Button`'s
`mounted` handle as `Anchor::Mounted`, section 6, the realistic pattern) rather than a fixed
`Anchor::Point` workaround — if you hit a `RefCell already borrowed` panic under `Harness` today, it is a new
bug, not this one; open one with the same reproduction shape (`Harness::new`, one `click()` that
opens a floating component) and cite this section.

**`Ds`'s own `Material::Window` frame layers did not match their own stylesheet rule — fixed.**
`crates/ds/src/root/ds.rs::FrameLayers::render` used to write the hidden layer's class as
`"ds-layer back"` — `back` as a second CSS class. `crates/ds/src/css/utilities.css` styles it as
`.ds-layer[*|data-layer=back]{opacity:0}` — an attribute, not a class. The two never matched, so
the hidden layer kept `.ds-layer`'s base (no `opacity` set, i.e. fully visible) instead of
starting hidden, which meant a Space switch's cross-fade (design/21-SPACES.md section 5) had
nothing to fade *from* zero — both layers were opaque throughout. Any consumer's own
`ds::lint::markup` test over a `Material::Window` root caught this as `Rule::UnstyledClass` on
`div.ds-layer.back`; `examples/consumer/tests/coherence.rs` used to document it as a reviewed
`Exception` with this same explanation, until the fix. `FrameLayers::render` now writes
`"data-layer": (slot != self.front).then_some("back")` instead of appending the word to `class`,
so the front layer carries no `data-layer` at all and the hidden one carries
`data-layer="back"`, which the stylesheet rule matches — `crates/ds/tests/root_ssr.rs::
the_hidden_frame_layer_carries_the_back_attribute` and `::
a_window_roots_markup_lints_clean_of_unstyled_classes` are the regression tests, and the
`Exception` in `examples/consumer/tests/coherence.rs` is gone.

## 10. Comparing your surface against the reference

`ds-gallery` (`crates/ds-gallery`) is quire's own contact sheet: every component across its
pages (tokens, type, controls, lists, overlays, materials, motion, space, gaps, matrix,
motion-lab), with a toolbar for theme, accent, motion, material, blur and Space preset. Its
"Gaps" page lists what quire does not draw yet, each with the FINDINGS section that owns it.

```bash
cargo run -p ds-gallery                              # opens the gallery interactively
cargo run -p ds-gallery -- --page controls            # opens directly on one page
cargo run -p ds-gallery --release -- --snapshot DIR   # renders every page x state to DIR as PNGs, then exits
```

Render your own surface at the same viewport and Appearance/Material combination and diff it
against the matching `--snapshot` PNG by eye (PNG snapshots are review artefacts, never a CI
gate — rasterisation drifts with Blitz revisions, per `DESIGN.md`'s "Moves verbatim" notes) —
that is the fastest way to catch a component you have subtly mis-wired (wrong `Material`, a
missing `Surface`, an icon at the wrong `IconSize`).

## 11. Menu tracking, curves, Spaces store

Three pure pieces a shell needs beside the components, and the generic settings file API they
sit on.

**Menu tracking** (`ds::MenuTrack<K>`, `crates/ds/src/overlay/menu_track.rs`; design/13
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
use ds_settings::{AppName, FileName, Format, Settings, SPACES, config_dir};

let dir = config_dir(AppName::QUIRE).expect("a config dir");
let store = SPACES.load(&dir);            // lenient: a bad key costs only itself
SPACES.save(&dir, &store)?;               // atomic temp-and-rename
let mut watch = SPACES.watch(&dir)?;      // 30 ms debounce; needs a Tokio runtime
// Your own file, the same way:
const SHELL: Settings<ShellFile> = Settings::new(FileName("settings.toml"), Format::Toml);
```

`ds_settings::{load, save, watch}` keep meaning `appearance.toml` (`APPEARANCE`); the generic
functions are `file::load(&SettingsFile)`, `file::save(&SettingsFile, &T)` and
`watch_file(SettingsFile)`.

**Settings derive, three more shapes.** Text is recognised by type (`String`, `PathBuf`,
`Cow<str>`) or by `#[settings(text)]` on a newtype. A one-variant enum is a key
(`KeyKind::Fixed`, drawn as a read-only `Widget::Readout`). A number without
`range = "min..=max"` is a compile error that starts `MissingRange { field: <name> }`; a type
the derive cannot place at all gets an error saying what to add.

## 12. Scrolling

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
