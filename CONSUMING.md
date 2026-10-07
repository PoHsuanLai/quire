# Consuming quire

This is the guide for a program that draws with quire instead of hand-rolling its own CSS and
markup: `examples/consumer` (a minimal, fully worked example — read its `Cargo.toml`,
`src/lib.rs` and `tests/coherence.rs` alongside this doc), sill and mailo. It assumes you have read
`design/README.md`'s reading order at least through `04-COMPONENTS.md`; this doc does not repeat
what a component looks like, only how to reach it from your own crate.

Citations below follow `design/README.md#3-citation-convention`:
`design/<file>.md#<heading-anchor>`.

**Every snippet here is compiled.** A block fenced `rust` is a doc test of `examples/consumer`
(`cargo test --manifest-path examples/consumer/Cargo.toml`, run by `scripts/check-consumer.sh`).
A fragment that needs an app around it (`YourApp`, `YourPage`, a handler you own) is fenced
`rust,ignore` and has its compiled twin in `examples/consumer/tests/guide.rs`, under a doc
comment naming the section. Change a snippet and its twin together.

## 1. Adding quire

**Rust version.** quire's minimum is `rust-version = "1.92"`: pdfrum's crates (PDF output,
`ds-blitz`) declare 1.92, above the 1.91 blitz needs at the pinned rev. The toolchain quire
builds and tests on stays pinned at 1.98.1 (`rust-toolchain.toml`); a consumer on an older
compiler than 1.92 cannot build `ds-blitz`.

**zbus and your executor.** `ds-settings` builds zbus with its default `async-io` backend, which
works under any executor, tokio included. Do not enable `zbus/tokio` in an app: Cargo unifies
features, and with `tokio` on, zbus's blocking API panics on a thread that drives a tokio
runtime ("Cannot start a runtime from within a runtime"; mailo's keyring and notifications hit
this). A shell that runs zbus on tokio everywhere enables `ds-settings = { features = ["tokio"] }`
deliberately.

**Imports.** `use ds::prelude::*;` brings in the names an app draws with (the root, the appearance
axes, the vocabulary, geometry, icons, the overlays, lists, fields and the common controls); a shell
program adds `use ds_shell::prelude::*;`. The crate roots hold nothing else but the stylesheet
assembly (`ds::stylesheet()`, `ds::kits()`, `ds::component_sheets()`, `ds::KIT`, `ds::selectors`, and
the same four in `ds_shell`) and the three facades. Every other name is reached by its home path
(`ds::components::..`, `ds::host::..`, `ds::stack::..`, `ds::style::..`, `ds::base::..`,
`ds::motion::..`), one path each. A program depends on `ds` alone: `ds::base` (`ds-core`),
`ds::style` (`ds-style`) and `ds::motion` (`ds-motion`) are the three crates under it,
re-exported as modules. `#[derive(Word)]` and `#[derive(Token)]` find the crate they name through
your manifest, so a crate that depends on `ds` alone derives them (`::ds::base`, `::ds::style`).

**A path dependency.** quire has no published version, so every consumer depends on it by
path, the way `examples/consumer/Cargo.toml` does:

```toml
[dependencies]
ds          = { path = "../quire/crates/ds" }
ds-settings = { path = "../quire/crates/ds-settings", features = ["dioxus"] }  # `dioxus`: `use_environment`; `tokio`: zbus on tokio
ds-blitz    = { path = "../quire/crates/ds-blitz" }    # only if you run on Blitz; features `pdf`, `print`, `spell`
ds-shell    = { path = "../quire/crates/ds-shell" }    # only a shell program: the bar, dock, widgets, lock screen

[dev-dependencies]
ds-lint     = { path = "../quire/crates/ds-lint" }     # your own tests call it (section 5 below)
ds-harness  = { path = "../quire/crates/ds-harness" }  # the test driver: `Harness`, `Input`, `Query`, `snapshot` (feature `pdf`: `pdf_app`)
```

The crates and what each holds: `ds` (the components, the `Ds` root, the host seams, the facades),
`ds-shell` (shell components and widgets), `ds-settings` (settings files, the schema derive, the
portal, `use_environment`), `ds-blitz` (the Blitz host: `launch`, windows, PDF, print, spell),
`ds-lint` (the linter), `ds-harness` (the test driver). `ds-core`, `ds-style` and `ds-motion` sit
under `ds` and are reached as `ds::base`, `ds::style` and `ds::motion`. `ds-conformance` and
`ds-gallery` are quire's own tests and contact sheet (section 9), not for consumers.

The linter is its own crate, `ds-lint` (coherence rules 1 and 2, section 5 below), so a consumer
that never lints does not pull in `cssparser`. The shell's parts are `ds-shell`, which a shell
depends on next to `ds` and draws with `Ds { sheet: Some(ds_shell::stylesheet()) }`; it lints
against `ds_shell::kits()` where an app lints against `ds::kits()`.

**A git dependency** is how a program outside this checkout takes quire, from a tag:

```toml
ds       = { git = "https://github.com/PoHsuanLai/quire", tag = "v0.2.1" }
ds-blitz = { git = "https://github.com/PoHsuanLai/quire", tag = "v0.2.1" }  # only if you run on Blitz
```

Cargo finds each crate by name in the one repository, so the crates of a tag share one revision.
Nothing else is needed for quire's own dependencies: `blitz-kit` (its own public repo,
github.com/PoHsuanLai/blitz-kit) comes transitively at the rev quire's `Cargo.toml` pins, and so
does the rest of the pinned block. Do not add a `[patch]` for it unless you develop `blitz-kit`
alongside: quire's own `[patch]` table (its local `blitz-kit` checkout, the vello and anyrender
forks) applies only when quire is the workspace root, never to a consumer, so a consumer that
needs the forks' behaviour (the CSS `filter` functions) copies the `[patch.crates-io]` block
from quire's root `Cargo.toml` into its own workspace root. The same block carries the
`dioxus-core` fork patch (`dioxus-core`, `dioxus-core-types`, `generational-box`, `subsecond`,
`subsecond-types`, all at one rev of github.com/PoHsuanLai/dioxus): dioxus-core 0.7.10 can spin forever
in `wait_for_work` when a task is cancelled before it is polled (a `use_resource` restart), fixed
upstream only in 0.8 (DioxusLabs/dioxus#5554). Every consumer that runs a `VirtualDom` copies those
five lines too, and drops them when quire moves to dioxus 0.8. shell-host and sill add, for their
local checkouts, `[patch."https://github.com/PoHsuanLai/blitz-kit"]` pointing at a sibling
`blitz-kit` so every path-linked repo builds one copy.

**Launching on Blitz.** `ds_blitz::launch(app, AppConfig::new(title, WindowSize::new(width, height)))`
runs `app` until its window closes. A `WindowSize` is what the window opens at and, with
`.with_least(width, height)`, the least a person may resize it to: give the least your layout's
panes fit in, so no pane is squeezed past its own least (`WindowSpec::new` takes the same value
for a later window). It enters the process-wide Tokio runtime that `ds_blitz::TokioSpawner`
hands `use_environment`, registers quire's faces, provides the document host (`provide_host`,
section 6, "For a bar"), and tracks the input modality. `AppConfig` also takes
`with_app_id`, `with_decorations(Decorations::Client)` for an app whose root draws
`WindowFrame::Titlebar`, `with_context(value)` (read with `use_context` anywhere in the app),
`with_net`, `with_frame_links` and `with_focus_fallback`. `examples/consumer/src/main.rs` is the
whole call.

**Windows and when the app ends.** Windows are independent: closing any window, the first
included, closes only that one. The app's choice is `AppConfig::with_last_window(LastWindowClosed)`:
`Exit` (the default: the loop ends and `launch` returns when the last window closes) or
`StayFor(Duration)` (the loop runs on with no window for that long, so a daemon-like viewer stays
warm for its next open, and ends then unless a window opened; the linger also counts from the
start of a loop that began with no window). To open a window from outside any component (a D-Bus
or tokio task, a timer), make an `AppHandle::new()` before `launch`, give clones to those
threads and one to `AppConfig::with_handle`: `handle.open_window(spec, root)` and
`open_window_with(spec, root, props)` (props are `Send`), `handle.redraw()` (asks every window to
repaint and wakes the loop, the proxy a worker uses to have the UI thread look again) and
`handle.quit()` (ends the app whatever the policy). Each answers `Err(AppEnded)` after the loop
ended, and a request made before the loop runs waits for it. `launch_idle(config)` runs the loop
with no first window, for an app that opens all of them through its handle. A window's root
reads its app's handle with `use_app_handle()`; `open_window` still works from inside a window.

**Windows that fit their content (Preview, QuickTime).** `handle.screen_extent()`
(`Option<Extent>`, logical pixels; also through `use_app_handle()`) is the monitor the next window
opens on: the primary monitor, or the first listed, since winit cannot say where the compositor
will put it. It is the monitor's whole size, not its work area: winit has no work-area query, and
on Wayland there is none to ask, so the panel's and dock's exclusive zones are not subtracted (the
85% cap leaves room for them). It is `None` until the loop runs (the first window's size is
fixed before then) and where winit lists no monitor. `handle.screen_area()` and
`sizer.screen()` (below) say the same with the scale and what is known.
`natural.fit(screen, Some(least))` caps an `Extent` to 85% of the screen on each axis and never
below `least` (a larger `least` wins), and `WindowSize::fitting(natural, screen, least)` builds
the `WindowSize` (uncapped when `screen` is `None`). Both cap each axis on its own, so a wide
image squashes to the cap. For content with a shape of its own, `natural.fit_with(screen, least,
Fit::KeepRatio)` and `WindowSize::fitting_with(natural, screen, least, Fit::KeepRatio)` shrink
both axes by the one factor the axis that is further over its cap sets (never larger than
`natural`; `least` then raises per axis and wins over the cap); `Fit::PerAxis` is what `fit` and
`fitting` do.

**The screen a window is on, as an app can know it.** `ScreenArea { output, work, scale, of, basis
}` (`handle.screen_area()` before a window exists, `sizer.screen()` inside one, both
`Option<ScreenArea>`): `output` is the whole output in logical pixels, `work` the area a window may
use (fit to this), `scale` the output's `Scale` in 120ths (`Scale(180)` is 1.5x), `of` whose output
it is (`ScreenOf::Window`, or `ScreenOf::Primary` before the window is open: the compositor
chooses where a window opens and does not say), and `basis` what `work` is made from.
`area.logical_for_pixels(Extent::new(1200, 800))` is the logical size of a window that shows
1200 x 800 device pixels one to one on this output. What a client cannot know on Wayland: the
panel, dock or menu bar (layer-shell exclusive zones go to the compositor, never to clients) and
so the work area is the whole output (`WorkBasis::WholeOutput`), and an app that knows a reserve
gives it with `area.less(Reserve { top: 32, .. })` (`WorkBasis::LessReserve`); an output's
transform (winit reads the mode's pixels, unrotated); and, before the window is mapped, the
fractional scale (the output's whole-number scale stands in, so the logical sizes read smaller than
the window will see). What it does know once the window is mapped: its own output
(`Window::current_monitor`) and the scale it draws at (`wp_fractional_scale_v1`), so `sizer.screen()` is exact
on a fractionally scaled output where `handle.screen_area()` is not. `handle.screen_extent()` stays
the primary output's `output`.

Inside the window, `use_window_sizer()` (`Option<WindowSizer>`; `Some` in the harness too, see
below) gives `sizer.request_size(Extent)` (winit's `request_surface_size`, logical pixels; the
window never goes below its least), `sizer.size()` (logical), `sizer.origin()`
(`Option<SizeOrigin>`, a read subscribes the component) and `sizer.request()` (`SizeRequest`, a
read subscribes too). The origin is `None` until the window is resized, `SizeOrigin::Requested`
when the resize is the answer to our own request, `SizeOrigin::Person` for any other (a drag, a
tile, a zoom, the compositor's own choice). A resize is ours when it arrives within 500 ms of
`request_size` at the requested size, to within 1 physical pixel per axis (scale rounding); a
configure that changes nothing is no resize. The request has its own state: `SizeRequest::Pending(size)`
from the call, `Idle` once a resize settles it, `Expired(size)` when nothing answered in 500 ms
(the compositor ignored it, or took the size the window already had). Expiry leaves the origin
as it was before the request, and the request is spent: a resize after it is the person's, however
close to the asked size. A component that retries or gives up on a request reads `request()`; one
that only stops fighting a person's size reads `origin()`. On Wayland the compositor places
windows; only the size is the app's.

```rust,ignore
// Open at the picture's natural size, then fit the real size once it is known.
let screen = handle.screen_extent();
let size = WindowSize::fitting(Extent::new(1200, 800), screen, Extent::new(320, 240));
handle.open_window_with(WindowSpec::new("Photo", size), viewer, path)?;

fn viewer(path: PathBuf) -> Element {
    let sizer = use_window_sizer();
    let app = use_app_handle();
    let loaded = use_signal(|| None::<Extent>); // set when the decode reports its size
    use_effect(move || {
        let (Some(sizer), Some(natural)) = (sizer.clone(), loaded()) else { return };
        if sizer.origin() == Some(SizeOrigin::Person) {
            return; // the person chose a size: the content stops asking
        }
        let screen = app.as_ref().and_then(|app| app.screen_extent());
        let size = screen.map_or(natural, |s| natural.fit(s, Some(Extent::new(320, 240))));
        sizer.request_size(size);
    });
    rsx! { /* ... */ }
}
```

**A window's own handle, and keyboard focus at launch.** A component reads the handle of the window
it renders in with `use_window_handle()` (`Option<WindowHandle>`, `None` in the harness or a
snapshot, where no loop runs): the first window's too, so `handle.focus()` asks for the window
that holds the component to be raised and `handle.close()` closes it. A process started with an
activation token (`XDG_ACTIVATION_TOKEN` on Wayland, `DESKTOP_STARTUP_ID` on X11: a launcher, a
notification click or a D-Bus activation sets it) hands it to the first window it creates, so the
compositor gives that window the keyboard; the variable is cleared as the window is created, so
the token is spent once and child processes do not inherit it.

**Raising a window that exists: `focus_with_token`.** A running app is handed a token by a second
launch (D-Bus `org.freedesktop.Application.Activate` or `Open`, whose `platform_data` carries it
as `activation-token`; on X11 `desktop-startup-id`) or by a notification click. The window it
should raise is already open, so the environment is no help: pass the token to the window's own
handle, `handle.focus_with_token(Some(token))`. On Wayland the token goes to the compositor as
xdg-activation's `activate(token, surface)` on the app's own connection, for that window's
surface, which is the only way a compositor lets a window that exists take the keyboard; on X11,
with `None`, with an empty token, or on a compositor without xdg-activation it is
`handle.focus()`. A token is good once, so pass it to one window. In mailo's service handler:

```rust,ignore
// `platform_data: HashMap<String, OwnedValue>` is the a{sv} of the call; the token is a string.
let token = platform_data
    .remove("activation-token")
    .and_then(|value| String::try_from(value).ok());
window_handle.focus_with_token(token);
```

The request is queued and the event loop answers it, like `focus`; `WindowHandle` is
`use_window_handle()`'s, or the handle `open_window` returned.

**The pinned dependency block.** `ds`'s own manifest resolves its dependencies (`dioxus`,
and for `ds-blitz`, the whole blitz/anyrender/wgpu stack) against *quire's own* workspace —
a path dependency does not inherit your workspace's `[workspace.dependencies]`, because
`crates/ds/Cargo.toml` has no `[workspace]` of its own and so joins whichever ancestor manifest
does (quire's root, not yours). You only need to pin the lines **you** name directly: at minimum
`dioxus` (to write `rsx!`), plus `dioxus-ssr` as a dev-dependency if you lint an SSR render
(section 5). Copy those lines **verbatim** from `docs/workspace-deps.toml` in quire
(`CONVENTIONS.md#10-change-discipline`: "the pinned block ... is copied verbatim into
every workspace"; `docs/workspace-deps.toml` is quire's own copy of the source of truth, not
owned by this doc). If you also run on Blitz (`ds-blitz`), your own crate that calls
`dioxus_native::*` directly (rare — most consumers only call `ds_blitz::launch`) needs the
`dioxus-native`/`blitz-*` lines too, exactly as pinned, never a different revision.

**If your own crate sits inside a Cargo workspace tree it does not own** (as `examples/consumer`
does, under quire's own directory), give it an empty `[workspace]` table so Cargo does not treat
it as an orphaned member of the workspace above it. A workspace root is also the only place
`[patch.crates-io]` applies, so a consumer that builds a renderer copies quire's root patch block
(the vello and anyrender forks) verbatim into its own root manifest, as `examples/consumer/Cargo.toml`
does; `scripts/check-consumer.sh` builds, lints and tests that example in the gate.

## 2. The `Ds` root

**A bare document, with no `Ds` root** (a never-painted input catcher, a 2 px hot-corner
surface) still carries Blitz's user-agent stylesheet, which sets `body { margin: 8px }`: its
content never reaches the true (0, 0) corner, and `position: absolute`/`fixed` with `inset: 0`
paints nothing on it (sill's hot corners). shell-host injects `body { margin: 0 }`
into every document it hosts; a bare document elsewhere (`ds_blitz::launch`,
a test) must cancel the margin itself or draw inside a `Ds`.

Every quire component must be drawn inside one `Ds` (design/03-COLOR.md
section 17.1; `crates/ds/src/assembly/ds.rs`). It resolves your appearance to a scheme, an accent
and a motion level; stamps `data-theme`, `data-typeface`, `data-accent`, `data-motion`,
`data-material`, `data-blur`, `data-modality` and `data-hover` on its own `div.ds`; injects the stylesheet
(unless you ask it not to); and provides the `Scope`, `HoverHub`, `ToastHub`, `LayerStack` and
`Overlays` contexts every component reads. It also renders `OverlayHost` and `ToastHost` after
your children, so menus, popovers and toasts always have somewhere to mount. `ToastHost` lays
out nothing while the hub is empty: a pushed toast mounts hidden for one frame and rises from
there, and is dropped again once it has sunk, so an idle root has
no toast element in its markup or its picture.

```rust,ignore
use ds::prelude::*;
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
| `system` | `SystemPrefs` | `SystemPrefs::default()` | the desktop's own scheme/motion/contrast, from `use_environment`'s `Environment::system` (`SystemPrefsSource::Portal`) |
| `look` | `SpaceLook` | `SpaceLook::default()` | the Space (design/21-SPACES.md §1): today its dots, theme override and card accent; **target (clean-up phase)**: one hue, a chroma factor, grain and theme, tinting a sidebar's ground only |
| `material` | `Material` | required | which of the eight materials this root paints (design/03-COLOR.md §17.1) |
| `blur` | `BlurState` | `BlurState::default()` | whether the compositor blurs behind this surface |
| `stylesheet` | `Inject` | `Inject::Inline` | `Inline` puts a `<style>` inside `.ds` (spike S1); `Host` lets you inject `ds::stylesheet()` yourself |
| `sheet` | `Option<&'static str>` | `None`: `ds::stylesheet()` | the text `Inject::Inline` writes; a shell passes `Some(ds_shell::stylesheet())` |
| `typeface` | `Option<Typeface>` | `None`: the appearance's own | overrides the appearance's typeface for this root |
| `user_style` | `ReadSignal<UserStyle>` | empty | the person's own stylesheet, drawn after the design system's, unlayered (section 12) |
| `surface` | `Option<&'static str>` | `None` | stamped as `data-surface`, the name a user stylesheet selects this root by |
| `chrome` | `Option<RootChrome>` | `None`: `RootChrome::of(material)` | `Painted` or `Transparent`: whether the root box paints its material. A Popover, Sheet or Toast root is transparent by default (it hosts floating cards, which paint the material themselves); pass `Painted` for a root that *is* the panel (the launcher) — section 6, "bar gaps" |
| `ground` | `Option<Ground>` | `None`: `Ground::of(material)` | `Paper` or `Frame`: which inks the content takes; Bar and Dock default to `Frame` |
| `frame` | `Option<FrameTint>` | `None`: `FrameTint::of(material, chrome)` | `Opaque` (the window's frame), `Tinted` (the Space gradient at the tint alpha: bar, dock, a painted popover, OSD, widget) or `None` (the material's flat tint) |
| `radius` | `Option<Corner>` | `None`: the material's own corner | `Corner::Token(Radius::…)` or `Corner::Px(Px(n))`: overrides `--m-radius` inline, for a root whose corner is a setting (the dock's `dock.pill_radius_px`) |
| `tint_alpha` | `Option<Alpha>` | `None` (the tint's default alpha) | the materials' tint alpha over compositor blur (design/22-SETTINGS.md §3.1 `appearance.material_tint_alpha`); pass `ds_settings::Environment::tint_alpha()` (thousandths: `Alpha(800)` is 80%) once you are reading a live `Environment` (section 3) rather than leaving it at the default |
| `stack` | `Option<MaterialStack>` | `None` (the keys' defaults) | the material stack's six alphas (highlight and hairline per scheme, shadow strength, vibrancy; design/22-SETTINGS.md §3.1 `appearance.material_*`); pass `ds_settings::Environment::material_stack()` once you read a live `Environment`, as with `tint_alpha` |
| `extent` | `RootExtent` | `RootExtent::Content` | `Content`: as tall as the root's content (a window, the bar, a card). `Viewport`: at least the viewport (`min-height:100vh; min-width:100vw`), **the option for an overlay surface** (an OSD, a sheet, a click catcher) whose content is all positioned and would otherwise leave the root, and everything it places, 0 px tall (FINDINGS "Root height") |
| `scale` | `Option<Scale>` | `None`: the host's `ds::prelude::HostSignals` scale, else 1x | the device scale this root draws for, in 120ths (`Scale(180)` is 1.5x, the `wp_fractional_scale_v1` unit and shell-host's `Scale`); the root writes the pixel tokens for it (below). `ds_blitz::launch`, `Harness` and `snapshot` provide `HostSignals` themselves; a host that is not `ds-blitz` passes `scale` |
| `window` | `WindowFrame` | `WindowFrame::None`: the root is what it was | `WindowFrame::Titlebar { title, lights: TrafficLights::{Shown, Hidden}, timing }` (or `WindowFrame::titlebar(title, lights)`): the client-decorated window's frame, a 28 px titlebar that moves and zooms the window, the traffic lights, the body and eight resize edges, all acting through the host's `ds::prelude::HostWindow` — section 6, "Window frame" |

### Pixel snapping: `scale`, the pixel tokens and `snap_to_device`

At 1.25, 1.5 or 1.75 a `1px` line covers a fractional number of device pixels and paints as one
full row and one half row. Three pieces keep every line whole device pixels (design/01-LAYOUT.md
§2.1; FINDINGS "Pixel snapping"):

1. **The root knows the scale.** `Ds { scale: Some(Scale(180)) }` (or the scale in the `ds::prelude::HostSignals`
   ds-blitz provides) makes the root write the pixel tokens' inputs inline. With no scale, or
   at `Scale::ONE`, it writes nothing and every token is its 1x value, so nothing changes.
2. **Lines read the pixel tokens** (`ds::style::tokens::pixel::PixelToken`, on `.ds`):

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
   `ds_blitz::snap_to_device(&mut BaseDocument)` re-rounds the laid-out document on the device
   grid (and rounds a pure translation to whole device pixels). `Harness` and `snapshot` run it
   every frame. **A host that resolves its own documents (shell-host) calls it after every
   `resolve` and before painting**; it does nothing at a whole scale. `ds_blitz::launch`'s
   window cannot (blitz-shell resolves and paints in one call), so there the tokens apply but a
   line may still sit half a device pixel off.

Glyphs need nothing from you: under a root at a fractional scale `Glyph` writes a stroke width
that is an even number of device pixels (design/08-ICONS.md §1.4.1).

### A document's frame must have a height

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
  absolutely placed (`crates/ds-conformance/tests/root_frame.rs` measures all three).
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
fine for a first cut, but it means "System theme, Blue accent, System motion" every time,
ignoring whatever the person picked last session. Section 3 below is how you read the real
value.

## 3. Reading appearance

### `ds_settings::use_environment`

`ds_settings::use_environment(store, system, spawner) -> ReadSignal<Environment>` (feature
`dioxus` of `ds-settings`; `ds-settings/src/environment.rs`) is the one hook that gives you a live
`Environment { settings: AppearanceFile, system: SystemPrefs }`: it loads `appearance.toml` from
the `Store`, watches the directory for edits (30 ms debounce), reads the freedesktop settings
portal (`SystemPrefsSource::Portal`), and watches it for changes, merging both into one signal.
Unknown and invalid keys in the file are printed to stderr as they are read and dropped on the
next save.

```rust,ignore
use ds_settings::{AppName, ConfigRoot, Store, SystemPrefsSource, use_environment};
use ds::prelude::*;
use ds_blitz::TokioSpawner;
use dioxus::prelude::*;
use std::sync::Arc;

#[component]
fn App() -> Element {
    let store = Store::new(ConfigRoot::Xdg, AppName("your-app-id"));
    let env = use_environment(store, SystemPrefsSource::Portal, Arc::new(TokioSpawner::current()));
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

**Nothing here names a runtime.** The file watch and the portal watch are tasks handed to the
`ds::base::spawner::Spawner` you pass (`ds-settings` may not depend on `tokio`, `scripts/check-boundary.sh`).
On Blitz, `ds_blitz::TokioSpawner::current()` is the implementor: `ds_blitz::launch` enters a
process-wide, lazily built Tokio runtime (multi-thread, two workers;
`crates/ds-blitz/src/launch/runtime.rs`) and holds the guard for the process's life, and
`ds_harness::Harness` enters it (`ds_blitz::enter_runtime`) in `Harness::new` for the harness's own life, so `current()` works
in anything launched with `launch` or rendered inside a `Harness`. A test never reaches the real
config or the session bus: it passes `ConfigRoot::Scratch(dir)` and
`SystemPrefsSource::Fixed(prefs)` (`crates/ds-conformance/tests/driven_components.rs::
use_environment_does_not_panic_under_the_harness`). `examples/consumer::App` shows the real
wiring.

### `use_scope` — reading the resolved scope

Inside any component under a `Ds`, `ds::prelude::use_scope() -> Scope` gives you what that scope resolved to:
`resolved: Resolved{scheme, accent, motion}`, `scheme`, `material`, `blur`, `modality`
(`crates/ds-style/src/scope.rs`). Components use this to size icons, choose overlay placement and read the motion
level for their own timers (section 6); you will rarely need it directly unless you are building
your own component.

## 4. `Surface` — a nested material, scheme, accent or blur state

A subtree in a different `Material`, or forced to a scheme, accent or blur state other than the
root's (a popover over a dark card, an always-light preview pane, a specimen in another accent)
is a `Surface`, not a second `Ds`: no stylesheet, no frame layers, just a nested `div.ds` that
re-stamps `data-theme`/`data-typeface`/`data-accent`/`data-motion`/`data-material`/`data-blur`
(the typeface is always the root's) and updates the
`Scope` every component under it reads.

```rust,ignore
use ds::prelude::*;
use ds::style::appearance::blur::BlurState;

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
use ds_lint::{assert_clean, Exception, LintConfig, Profile, Rule};

const OUR_CSS: &str = ".row { color: var(--ink); }\n.fade { mask-image: linear-gradient(#000, transparent); }";
const EXCEPTIONS: &[Exception] = &[Exception {
    rule: Rule::HexColour,
    selector: ".fade",
    reason: "a mask's alpha, never painted",
}];

#[test]
fn our_css_is_clean() {
    assert_clean(OUR_CSS, &LintConfig {
        profile: Profile::Strict,   // the default; `Profile::Details` adds the details grammar's timing
        exceptions: EXCEPTIONS,
        ..LintConfig::new(&ds::kits())
    });
}
```

`Rule::LayerDs`: a stylesheet that opens `@layer ds` (a block, an order statement or a sub-layer)
is an offence: that layer is quire's, and `AppStyle` puts yours in `app`.

Every exception needs a `reason`, and `assert_clean` panics listing which exceptions suppressed
zero offences, so a stale one cannot hide silently. A `LintConfig` is made from the kits it lints
against (`LintConfig::new(&ds::kits())`), which give it the variables, keyframes and timing tokens
it accepts; name only the fields you change and end with `..LintConfig::new(&ds::kits())`
(`own_vars` lists the custom properties your own sheet declares). `Profile::Strict` is the
default and runs every rule but `OffGrammarTiming`; quire's own `self_lint` tests run it, and a
raw `border-radius`/`font-size`/`z-index` your CSS writes is exactly the kind of drift the token
table exists to prevent.

`Rule::RawSpacing`: a literal `px` in `margin`, `padding` (their sides and
logical forms included) or `gap`/`row-gap`/`column-gap` is an offence; `0`, `auto`, a
percentage, an `em` and `var()` pass. The steps are `ds::style::tokens::spacing::SpacingToken`, emitted on `.ds` as
`--s-1`, `--s-1-5`, `--s-2` … `--s-12`, `--s-13`, `--s-14`, `--s-15`, `--s-16`, `--s-18`,
`--s-22`, `--s-26`, `--s-36`, each named by its pixel value (design/01-LAYOUT.md §2). A length
between steps takes the nearest one, as quire's own sheets and the gallery's do. `examples/consumer/src/style.css` is Strict-clean.

`Rule::UnknownAnimation` reads the `animation` shorthand as well as `animation-name`: in each comma-separated animation the name is the first identifier that is not a keyword
(an easing, `infinite`, a direction, a fill mode, a play state, a CSS-wide keyword), with times,
counts and functions skipped. `animation: sparkle 1s` is an offence; a name only a `var()`
holds, or one written as a string, is not judged.

`Rule::RawHairline`: a literal hairline (`1px`, `.5px`) as a
`border*` or `outline*` width, or as a box's whole `width`/`height`, is an offence whose text
names the token (`border: 1px (use var(--hair))`, `.5px` points at `var(--hairline)`). A 2 px
border and a `border-radius: 1px` pass: stylo already floors a border width to whole device
pixels, and only a line of one pixel or less goes blurry.

### Rule 2 — no raw markup, only quire components

Two tests, both against an SSR render of a real page (never a hand-built HTML string — the
markup lint runs on what your app *actually renders*, not on what you believe it renders):

```rust,ignore
use dioxus::core::VirtualDom;
use ds_lint::{markup, LintConfig, Rule};

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

Every `style` attribute is checked declaration by declaration (`ds_lint`'s `inline_style`): a
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
drive a class toggle. Use `ds::prelude::use_motion_timer` (`MotionTimer::start(on_settled)`, which runs for exactly
`ds::prelude::settle(anim, level)`); both read the enclosing `Ds`'s resolved motion level, so
`MotionLevel::Reduced` collapses them automatically. Prove it with `ds_harness::Harness`, which
drives a real Blitz document on a real (if fast-forwarded) clock — the test below is
`examples/consumer/tests/coherence.rs::the_sent_badge_times_out_on_ds_motions_own_clock`,
shortened:

```rust,ignore
use ds::prelude::*;
use ds_harness::{Clock, Driver, Harness, HarnessConfig, Input, Query, Viewport};
use std::time::Duration;

let resolved = resolve(Appearance::default(), SpaceLook::default().theme, SystemPrefs::default());
let hold = settle(Anim::Fade, resolved.motion); // never a millisecond literal

let view = Viewport { width: 480, height: 360, scale_percent: 100 };
let mut harness = Harness::new(YourApp, HarnessConfig::new(view).with_clock(Clock::Virtual));
if let Some(target) = harness.centre(".ds-button") {
    harness.send(Input::click(target));
}
harness.advance(hold - Duration::from_millis(10));
// assert it has NOT settled yet
harness.advance(Duration::from_millis(20));
// assert it HAS settled now
```

Computing `hold` through `ds::prelude::settle` rather than copying today's millisecond value is what makes
this a real test of rule 4: a `Duration::from_millis(1200)` literal would still pass if `Anim::
Fade`'s token were retuned tomorrow, which is exactly the drift `ds::motion` is supposed to make
impossible.

**Exact timing: the virtual clock.** `Clock::Wall` (the default) runs quire's timers on the wall
clock, so `advance(hold - 10ms)` can overshoot the boundary on a loaded machine. `Clock::Virtual`
(`HarnessConfig::with_clock`, as above) makes `advance` move one clock that drives both the CSS
animations and every ds timer (motion timers, presence and roster rests, hover intent, toast
holds, pending, detail tweens, menu submenu delays): it steps to each timer's due instant in
order, runs the renders that queued and resolves the CSS at that same instant, and returns at
once. The two assertions above then hold exactly, every run.

**Driving and reading.** `Harness::new(app, config)` takes a `HarnessConfig` (or a bare `Viewport`
for the defaults) and renders the first frame. A test sends `Input` and lets time pass through the
`Driver` trait (`send`, `advance`, `render`) and reads the document through the `Query` trait
(both are in `ds_harness`; a second driver, such as a shell surface's headless host, gets
`Query` by implementing `DocQuery`):

| Want | Call |
| --- | --- |
| Fingers on a touchpad, a wheel under Control | `Input::fingers(point, dx, dy, GesturePhase::Began / Changed / Ended)` (advance a frame between moves so the lift has a speed), `Input::detents_held(point, x, y, Modifiers::CONTROL)` | Fingers go through the window's scroll as winit's pixel `MouseWheel` does, with phases. |
| Click, press or move | `Input::click(point)`, `Input::press(point, PointerButton::Secondary)`, `Input::pointer_move(point)`, `Input::drag(from, to, steps)`, `Input::wheel(point, dx, dy)` |
| Type | `Input::key(ShortcutKey::Enter)`, `Input::chord(&[ShortcutKey::Ctrl], ShortcutKey::Char('k'))`, `Input::paste(html, text)`, `Input::ime_commit(text)` |
| Where something is | `harness.centre(selector) -> Option<Point>`, `harness.rect(selector)`, `harness.hits(point, selector)` |
| What it says | `harness.text_of(selector)`, `harness.attr(selector, name)`, `harness.count(selector)`, `harness.has_class(selector, class)`, `harness.focus_of(selector)` |
| How it looks | `harness.ink_of(selector)`, `harness.fill_of(selector, Part::..)`, `harness.render()`, `harness.html()` |
| Pictures without a harness | `ds_harness::snapshot(app, viewport)` and `snapshot_at`, `snapshot_with`, `snapshot_placed` |

| Want | Call | Notes |
| --- | --- | --- |
| Timers on the harness's clock | `HarnessConfig::with_clock(Clock::Virtual)` | Default `Clock::Wall`. `Harness::clock() -> Clock` |
| A window a component sizes | `use_window_sizer()` is `Some` in every harness. `HarnessConfig::with_sizer_ack(SizerAck::Now / After(Duration) / Never)` (default `Now`); `with_window_screen(WindowScreen::Standard / Absent / Area(ScreenArea))`; `harness.window_requests()` (the sizes asked, logical, oldest first), `harness.window_size()`, `harness.resize_window(Extent)` (a person's drag), `harness.window_sizer()` | The answer to a request is a resize that arrives later, as in a window: `Now` is the next turn with no time passing, `After(d)` is `d` on the harness's clock, `Never` is a compositor that ignores it (the request then expires after 500 ms, see `SizeRequest`). The standard screen is 1920 x 1080 logical at the viewport's scale. Use `Clock::Virtual` for `After` and `Never`. |
| "Now" in a test | `Harness::now() -> Instant` | The virtual clock's now (or the wall clock's); `settle_until` returns instants on the same clock, and its 3 s bound is the harness's time. On the virtual clock, time a window from `harness.now()`, never `Instant::now()` |
| Read the time in your own component | `ds::base::time::clock::now()`, `ds::base::time::clock::since(instant)`, `ds::base::time::clock::sleep(d)` | Whatever clock the thread has installed: the wall clock in a window, the harness's in a test. A component that calls `Instant::now()` or `futures_timer` itself stays on the wall clock and drifts from the harness |
| Install a virtual clock yourself (another harness) | `ds::base::time::clock::VirtualClock::new()`, `.install() -> ClockGuard`, `.advance_to(d)`, `.next_due()`, `.now()`, `.elapsed()`, `.waiting()`, `.due_times()` | Thread-local, restored when the guard drops. Step through `next_due` and poll your executor between steps, as `Harness::advance` does |

Not on the virtual clock: work off the harness's thread (a Tokio task such as `ds_settings`'
file watch, a D-Bus reply, a resource fetched by a custom `AppNet` on another thread).
`advance` on the virtual clock never waits for the wall clock, so a test that needs such work
to land stays on `Clock::Wall`. Nor Blitz's own clock reads (rev e99fbdbd): its double-click
count (a press within 500 ms and 2 px of the last) and scrollbar fade read `Instant` in fields a
host cannot set, so on the virtual clock two clicks at one spot are always a double click, however
far apart the test advanced them; click a second spot in between, or keep that test on the wall
clock.

**`ds_harness::harness::assert_settles_to_zero_frames` is only a strict check on `Clock::Virtual`.** It drains
`VirtualClock::next_due` — advancing straight to each pending sleep's due instant, earliest
first, until none remain — then asserts a quiet window right after, so "at rest" means exactly
design/26 R3: no CSS animation, no pending `ds` timer, nothing woke the document. On
`Clock::Wall` it cannot see what is pending, only what just happened, so it instead polls in
`QUIET` windows and returns as soon as *one* window is quiet; a timer that started later in the
same test (a 900 ms hold begun at the top of a window shorter than 900 ms) can still be pending
when it returns. Use `Clock::Virtual` for any test that needs the strict guarantee, not the poll.

### Content inset check

Text, glyphs and images keep 8 px (`--s-8`, the menu and row text inset) from the painted left and
right edge of the box they sit in. Rule 1 cannot see this (a row with no `padding` is clean CSS),
so one test renders your surface and measures the laid-out document. `ds-harness` is the
dev-dependency you already have:

```rust,ignore
use ds_harness::inset::{assert_insets, Allow, Policy};
use ds_harness::{Harness, HarnessConfig, Viewport};
use ds::prelude::Px;

#[test]
fn every_surface_keeps_content_off_the_box_edges() {
    let view = Viewport { width: 900, height: 700, scale_percent: 100 };
    // quire's policy is the 8 px default plus quire's own allowances; add yours with a reason.
    let policy = Policy::quire().allowing(&[Allow {
        class: "unread-count",
        min: Px(4.0),
        reason: "a count badge: design/04 section 12, 4 px either side",
    }]);
    for open in [Surface::Inbox, Surface::Settings] {
        let harness = Harness::new(open.app(), HarnessConfig::new(view));
        assert_insets(&harness, &policy); // lists every offence: box path, side, gap, minimum
    }
}
```

A *visible box* is an element that paints an edge: a background that differs from what is behind it,
a gradient, a border on a side, an outline, a shadow, or a `aria-selected`/`data-selected` state.
`insets(&harness, &policy)` returns the `Findings` instead of panicking (`offences`, and `excused`,
one per gap an `Allow` forgave, so a test can fail an entry that excuses nothing). Fix an offence
with padding on the box (or the content's margin) in `--s-*` steps; widen the allow table only for a
class the design really holds narrower, and say why. A lone icon centred in its box (an icon button)
is never flagged. It does not see a filled child box (dot, avatar, switch thumb) as content, only
text, glyphs and images. Set `Harness` states first (`send`, `advance`) to check a hover, a popover
or an open menu.

## 6. The component catalogue

Every component is `ds::<Name>`, one `.rs`/`.css` pair per `design/04-COMPONENTS.md` section
(`DESIGN.md` "`ds`: components"). One worked example per family below; the full list is the
table after it.

### Controls

```rust,ignore
use ds::prelude::*;
use ds::components::controls::button_model::Answers;

rsx! {
    Button {
        answers: Answers::Return,
        label: "Send".to_owned(),
        icon: Some(Icon::Send),
        onclick: move |_| send(),
    }
}
```

### Lists

```rust,ignore
use ds::components::app::thread_row::ThreadRow;
use ds::base::vocab::RowState;

rsx! {
    ThreadRow {
        state: RowState::default(),
        name: "Ada Lovelace".to_owned(),
        via: None,
        subject: "Re: the analytical engine".to_owned(),
        snippet: Some("I have translated the memoir...".into()),
        time: "2:14 PM".to_owned(),
        tags: rsx! {},
        star: None,
        strip: None,
        onclick: move |_| open_thread(),
    }
}
```

`ThreadRow` is the mail-only row (`ds::components::app`), built on `Row`. Its `more` slot holds one
quiet trailing action in flow beside the tags, never over the time: `more: rsx! { RowMore { expanded,
onclick: move |rect: Rect| open_menu_at(rect) } }` (`ds::components::app::row_more`; a 26 px icon-only
`Icon::Ellipsis` button labelled "More actions", shown on row hover, focus, `shown` or while `expanded`
is `Shown::Visible`, which the caller owns). `strip` is the older raised pill and still works. A card
row's height is `thread_card_height(ThreadLines::Two | Three)` (`ds::components::app::thread_height`,
the row's own box from the same tokens as its CSS, at scale 1) and the gap below it is
`thread_card_gap()`, so a `VirtualList`'s pitch is their sum. Everything else that lists uses `List`
of `Row`s: `Row { leading: RowLeading::Icon(..), title, detail, accessory: Accessory::Chevron,
state: RowState { selection, ..RowState::default() }, size: RowSize::Settings, onclick }`, where
`Accessory` is `None`, `Check`, `Toggle`, `Chevron`, `Text`, `Glyph`, `Battery`, `Spinner`, `Badge`
or `Slot`; a busy row (`Availability::Busy`) shows the spinner in place of its accessory.
`List { label, items: Vec<ListItem<K>>, style: ListStyle }` roves with the arrows and Home/End.
`Disclosure { title, shown, ontoggle, children }` collapses a body; `SectionHeader { title, value,
actions: Vec<HeaderAction>, collapse }` heads a group (`HeaderAction::new(label, onclick)`, any
number of them at its end).

### Forms: drill-in vs sheet

A settings page that lists things (accounts, networks, devices) opens the **detail of an existing
item** by drilling in: a `PaneStack` pushes the detail page into the same pane, under a header
whose back button names the page it came from (System Settings > Internet Accounts > an account).
A **sheet** is for what leaves the page: creating a thing, or confirming a destructive step. One
rule: detail of an item that exists = drill-in; create, or destructive confirm = sheet card.

The caller owns the path (`PanePath<K>`, `K` names a page); the stack keeps only the transition.
`opener` names the id of the row that opens a page, and a pop returns the keyboard to it; a push
moves it to the new page's back button (`<id>-back-root` / `<id>-back-detail`, `id` being
`common.id` of the stack). Escape, Command+`[` and Alt+Left call `on_back`, never at the root.

```rust,ignore
use ds::prelude::*;

#[derive(Clone, PartialEq)]
enum Page { Accounts, Account(AccountId) }

let mut path = use_signal(|| PanePath::new(Page::Accounts));
rsx! {
    PaneStack::<Page> {
        path: path(),
        title: Callback::new(|page: Page| match page {
            Page::Accounts => "Accounts".to_owned(),
            Page::Account(id) => name_of(id),
        }),
        // A chevron row in the accounts Form pushes: `path.set(path.peek().pushed(Page::Account(id)))`.
        page: Callback::new(move |page: Page| draw(page, path)),
        opener: Callback::new(|page: Page| match page {
            Page::Accounts => None,
            Page::Account(id) => Some(format!("open-{id}")),   // the id on that account's `Row`
        }),
        on_back: move |()| path.set(path.peek().popped()),
        common: Common { id: Some("accounts".to_owned()), ..Common::default() },
    }
}
```

### Overlays

```rust,ignore
use ds::prelude::*;
use ds::host::measure::{Anchor, MountedRef};
use ds::root::common::Common;
use ds::stack::toast_hub::UndoToken;

let toasts = use_toasts();
toasts.push("Sent".to_owned(), None);   // ToastHost is already rendered by Ds — nothing else to mount
// With an undo: the handler of the toast on screen runs when the person undoes it.
toasts.push_undoable("Archived".to_owned(), UndoToken(7), EventHandler::new(move |token| restore(token)));

// A menu anchored to the button that opens it: the button hands over its own element.
let mut open = use_signal(|| Shown::Hidden);
let mut more = use_signal(|| None::<MountedRef>);
rsx! {
    Button {
        label: "More".to_owned(),
        onclick: move |_| open.set(Shown::Visible),
        common: Common {
            mounted: Some(EventHandler::new(move |event: MountedEvent| more.set(Some(MountedRef(event.data()))))),
            ..Common::default()
        },
    }
    if let (Shown::Visible, Some(button)) = (open(), more()) {
        Menu { placement: MenuPlacement::Popup, anchor: Anchor::Mounted(button), items, onpick, onclose: move |()| open.set(Shown::Hidden) }
    }
}
```

`examples/consumer/src/lib.rs::Page` is this pattern, whole.

### Frame

```rust,ignore
use ds::prelude::*;
use ds::components::app::command_pill::CommandPill;

rsx! {
    CommandPill {
        label: "Search or run a command".to_owned(),
        shortcut: Shortcut(vec![ShortcutKey::Super, ShortcutKey::Char('k')]),
        onclick: move |()| open_palette(),
    }
}
```

Full catalogue (design doc section in parentheses):

| Family | Components |
| --- | --- |
| Controls | `Label`, `Button` (push, toolbar, inline and help bezels; an image-only button is a toolbar `Button`), `Toggle`, `Checkbox`, `RadioGroup<T>`, `SegmentedControl<T>` (also the tab strip), `Slider` (linear and capsule looks), `TextField` (plain, secure, search and multi-line), `ProgressIndicator` (bar, spinner, ring), `LevelIndicator`, `Badge`, `KeyEquivalent`, `CommandPill`, `Chip`, `Avatar`, `SectionHeader` |
| Lists | `List`, `Row`, `SectionHeader`, `Disclosure` (design/30 §2), `ThreadRow` (`ds::components::app`), `HoverStrip` (§17) |
| Overlays | `Tooltip`/`HoverTarget`/`HoverCard` (§18, §22), `Menu`/`MenuItem`/`PopUpButton` (design/30 §2.4), `Popover` (§21), `Toast`/`use_toasts` (§23), `Sheet`/`Alert`/`SidePanel`/`Peek` (§24), `CommandPalette<T>` (§25), `EmptyState`, `InlineBanner` (a message in a pane's own flow), `Skeleton`/`SkeletonRow` (static placeholders), `Loadable` (placeholder, content or failure by phase) |
| Frame | `Capsule` (design/30 §2.7a), `PinTile`/`PinTiles` (design/30 §2.11), `ProviderMark` (§28), `LinkPill` (§29), `SendPill` (§31), `SpaceEditor` and `SpaceDot` (§32), `EdgePeek`, `TodayTabs`, `space_pressed` (§2.11), `DragGhost` (§34) |

Every component's exact props are its own `#[component] pub fn` signature in
`crates/ds/src/components/<family>/<name>.rs` — read that, not this table, before wiring one up; this doc
only orients you to which family a component is in and where its design spec lives.

A few props worth knowing about before you read the signatures:

- `TextField` takes `#[props(default)] focus: FieldFocus` — `FieldFocus::OnMount` focuses it as soon as it
  mounts (the command palette's input, a bubble's link field); the default, `FieldFocus::Manual`, is
  what every other field wants; `FieldFocus::Controlled(request)` focuses it on mount and again at
  every `request.request()` (the launcher gaps, below).
- `TextField { kind: FieldKind::Multiline, rows: FieldRows::Six }` is a `textarea`: Enter adds a line and
  the caret leaving commits (`onchange`). `Label { severity: Some(Severity::Ok) }` sets a status
  colour (`Severity` is `Info`, `Ok`, `Warn`, `Danger`, also the tone of `InlineBanner { severity, text,
  detail, actions, onclose }`). `EmptyState`'s `description` is a `TextLine`, so a `RunTone::Code` run
  can hold a command. `FactList { facts: vec![Fact::new("When", ..)] }` is a read-only list of labels
  and values; a `FieldRow` takes one control or several, which wrap.
- **Controlled components** (CONVENTIONS section 3, "Controlled components"): the caller holds the state
  and the component shows it. A `TextField` is `value` in, `oninput` out for each change, `onsubmit` for
  Enter alone (`onchange` is Enter or the caret leaving; neither fires `onsubmit` in a multi-line field),
  `validity: Validity::Invalid(Invalid { message, stamp })` for the error text under the field (it
  replaces `help` in the danger ink) and `availability`. A secure field keeps its own text unless it is
  given `text: FieldText::Caller`; then its `value` is the text, it keeps no copy, one dot per character
  is all it draws, copy and cut (Ctrl or Super with C or X, Ctrl+Insert, Shift+Delete) are refused,
  paste works, and a `value` the caller empties empties it (it needs no new `key`).
- `List { cursor: Option<K>, onselect, onpick }` is a selection the caller holds by key: arrows, Home
  and End and type-ahead call `onselect(key)`, Enter and Space call `onpick(key)`, and the list passes
  over headings and disabled rows. The caller draws the selected row (`Row { state: RowState { selection,
  .. } }`) from its own `cursor`; reordering the items moves nothing, since the cursor is a key.
- `PopUpButton { value: Option<T>, onpick }` is controlled in its value; its open state is the button's
  own until the caller passes `open: Some(Shown)`, after which every press, arrow key, pick or dismissal
  reaches `on_open_change(Shown)` and the caller stores it (`start` is then ignored).
- A button's `availability` is its controlled disabled state. `answers: Answers::Return` draws the
  default button and `Answers::Escape` the Cancel button; the keys themselves are routed by the dialog:
  `Alert` routes both, and `Sheet { on_return: Some(handler), onclose }` routes Return (anywhere a
  control did not take it, a multi-line field's Enter included) to `on_return` and Escape to `onclose`.
  Pass `on_return: None` while the default button is disabled. The first stop of the keyboard in a sheet
  is the caller's: `TextField { focus: FieldFocus::OnMount }` for its first field or `Button { focus:
  ButtonFocus::OnMount }` for the default button.
- `Button { availability: Availability::Busy, busy: BusyLook::TurnIcon, icon, .. }` shows work in progress
  by turning the button's own icon (Mail's Get Mail arrow) instead of the default spinner in the leading
  slot: one linear turn a second (`turn`, `--t-turn`), no leading mark, input dropped and `aria-busy`
  written as for any busy button. Reduced motion holds the glyph still. `BusyLook::Spinner` is the default.
  It is the symbol effect `SymbolEffect::While(LoopEffect::Rotate, ..)` on the button's icon wrapper.
- `Symbol { icon, size, effect }` plays a symbol effect on an icon (design/35-SYMBOL-EFFECTS.md):
  `SymbolEffect::Once(OnceEffect::Bounce, trigger)` plays each time `trigger` (a `Trigger`, bumped
  with `.next()`) changes and never on the first render or a re-render with the same value;
  `SymbolEffect::While(LoopEffect::Pulse, Activity::Active)` runs until it is `Idle`;
  `SymbolEffect::Transition(TransitionEffect::Appear, Shown::Visible)` plays when the state changes
  (`Appear`, `Disappear`, `DrawOn`, and `Replace`, which cross-fades when `icon` changes). `Part`
  and `VariableColor` move the piece an icon annotates (trash lid, bell, mail flap, folder, lock
  shackle, star and heart, refresh; the bars of Wi-Fi, the waves of volume, the cells of a full
  battery) and fall back to Bounce and Pulse on any other icon. Nothing plays by itself, and Reduced
  motion holds or fades each. A component that wants only the wrapper's class calls
  `use_symbol_attrs(effect)`.
- Copying to the clipboard from an action is `ds_blitz::clipboard::write_text(&str)` (read: `read_text`),
  reached through the document's host, so call it from a handler; a test reads it back with
  `Harness::clipboard_text()`. ds-shell has no clipboard of its own.
- A settings enum key's menu is built by the caller from `KeySpec::grouped_choices()`: a
  `MenuItem::Header(group)` over each group's items, an unavailable choice as `MenuItem::new(..)
  .with_availability(Availability::Disabled)` with `MenuItem::Info { title: reason, detail: None }` right
  under it (detent's `menu_items` is the worked example). ds does not depend on ds-settings, so quire
  has no converter.
- `Row { edit: Some(field) }` puts a field where the words stood (rename in place: a `TextField` with
  `FieldBezel::Plain`); the keys and presses typed in it do not reach the `List` around it. `PinItem`
  carries its own `mark: MarkStyle`; `TodayTab` its row's `common` and `onpointerenter`/`onpointerleave`.
  A test builds an `EditPointer` with `EditPointer::new(phase, at)` and `Clicks(n)`.
- `Row` takes `#[props(default)] drop: DropState` (`Idle`, `Target`,
  `Source`) — drag-and-drop visual state (design/04-COMPONENTS.md §34); leave it `Idle` unless
  you are wiring up drag and drop for that row.
- `ds_shell::accounts::model::SignInForm` fields for a hand-typed server: `FormField::new(role,
  requirement, text)` plus `.choosing(Vec<Choice>)`, `.hinted(text)` and `.in_part(FormPart)`.
  `FieldRole` is one-to-one with porter's `FieldKind` (`Protocol`, `Port`, `Security`,
  `OutgoingServer`, `OutgoingPort`, `OutgoingSecurity`, `SessionUrl`; `Token` stays the secret);
  `ProblemKind::Invalid` joins `Missing` and `Refused`. A field with `choices: Some(..)` is a pop-up
  button (the host words each `Choice { slug, label }`) and holds the chosen slug as
  `FieldText::Plain(slug)`; a pick reaches `on_input` as `(role, FieldText::Plain(slug))`, exactly the
  slug, like any edit, so the host maps it (porter's `FieldKind::choices()` slugs) and refits the
  field list; quire reshapes nothing. A port is a plain entry, empty meaning the usual port, with
  the host's number as `hint`. When any field has choices or a `part`, the form is a `Form` of
  titled `FormSection`s (neighbouring fields of one `FormPart`: Incoming, Outgoing, SignIn), one
  `FieldRow` each; otherwise it is the plain stack of entries as before.
- `SpaceEditor` (and `SpaceDot`) live in `ds`, not `ds-shell`, and take two more optional props: `name: Option<String>` (the Space's own name
  field, drawn inside the editor rather than beside it) and
  `on_active_dot: Option<EventHandler<ActiveDot>>` (fires when the person's focus moves to a
  different dot, separately from `onchange`, which fires on an actual edit).
- `PinFace::Account` has an `address: Option<String>` field — a `PinTile`'s accessible name falls
  back to its letter and provider without one; pass the account's address when you have it.
  (A selection bubble is a context menu.)

Loading, failing and arriving (design/30 §2.9; all in `ds::prelude`, none moves except through the
motion machinery, so Reduced is automatic):

- `Loadable { phase, placeholder, onretry, action, children }` is the one place that picks what a pane
  shows: `Phase::Loading(Operation)` draws `placeholder` (default: a centred `ProgressIndicator`
  Spinner, Regular, which turns only while the `Operation` is `Running`: Rule 4's R4 lives in the
  indicator), `Phase::Ready` draws `children`, `Phase::Failed { title, description }` draws an
  `EmptyState` of form `Failure`, with Retry when `onretry` is given and your own `action` element (a
  secondary "Connection Doctor…" button) in the same action row, before Retry. A change of phase *kind*
  cross-fades the arriving layer in over `--t-quick` (the §1.3 cross-fade primitive, `use_cross_fade`;
  a new `Operation` or a new title plays nothing, and under Reduced nothing fades: the swap is at
  once). Keep passing a custom `placeholder` (a list of `SkeletonRow`s) while the phase leaves
  `Loading` and it fades out over the arriving layer before it is dropped; the default spinner is
  simply replaced. The root is `aria-busy` while loading and a `region` when `common.aria_label`
  names it. Start the operation in the handler that began the work
  (`Operation::Running(PendingToken::start())`), never in render.
- `SkeletonRow { lines: SkeletonLines::{One, Two}, shown, on_hidden }` is a placeholder list row (avatar
  circle, a title bar 62% of the text column, a second bar 38%) at a settings row's size (the row
  tokens `--row-settings-h` 44 and `--row-avatar` 34, which `Row` at `RowSize::Settings` reads too);
  pass several as a `Loadable`'s `placeholder`. Static, no shimmer, like
  `Skeleton { shape: SkeletonShape::{Line, Block, Circle}, .. }`; both fade in over `--t-quick` and,
  when `shown` goes `Hidden`, fade out and call `on_hidden` once they have settled.
- `InlineBanner` is static, as Mail's remote-content bar is: it appears and disappears in place with
  no motion. `shown: Shown` (default `Visible`) is only a render switch: `Hidden` draws nothing and
  what is under it takes its place at once, so you may equally wrap it in an `if`. It needs no `Ds`
  root of its own.
- A `Button` or `Row` with `Availability::Busy` cross-fades both ways over `--t-quick`: its spinner
  fades in, and when the work ends the leading mark (Button) or accessory (Row) it covered fades back
  in; a control that was never busy plays nothing on first show. Nothing to wire (the control wears
  a `ds-busy-seen` class once it has been busy).

Account tiles (design/30 §2.11, `ds::components::app`): `PinTiles { label, items, selected, onpick,
onreorder, onstatus, onmenu, add }`, with `PinItem::new(key, face)` and its setters `.unread(n)`, `.mark(style)`
and `.status(PinStatus)` (build items with these, not struct literals, so a new field does not break
you). `PinStatus` is the account's problem mark, in the tile's top-left corner (the unread count has the
top right, the provider mark the bottom right): `Quiet` (the default) draws none, `Busy(Operation)` a Mini
spinner that turns only while the `Operation` is `Running`, `Attention { why }` the warning glyph with
`why` as its tooltip and accessible name. `onstatus: Option<EventHandler<K>>` hears the key of a tile whose
warning was pressed; that press is neither a pick (`onpick`) nor the start of a drag.
`onmenu: Option<EventHandler<PinMenu<K>>>` hears a secondary press (a right-click) on a tile, as
`PinMenu { key, at }` with the point to hang a menu from; the grid draws no menu itself. Only a primary
press picks, so a right-click never changes the filter.

Window layout (design/30 §2.7, `ds::components::chrome`): a source-list sidebar is
`Sidebar { label, sections, size, cursor, onselect, header, foot, fill }`. `sections` is a
`Vec<SidebarSection<K>>`, top to bottom: `SidebarSection::List(items)` is a source-list `List` (its group
headings are its own `ListItem::heading`s) and `SidebarSection::Custom(element)` is anything else the
sidebar holds between its lists, such as `PinTiles` or `TodayTabs`, whose heading and selection you draw.
The sections scroll; `header` stays above them and `foot` (a Space's name, dots and buttons) stays under
them. One `cursor` runs across every list, so the app keeps one selected place. It is a plain Mac source list.
`fill` is `SidebarFill::Material` (the default: the sidebar's own ground) or `SidebarFill::Clear`, which paints
nothing, so the window's flat Space tint shows through behind the rows; the inks are the ordinary ones in both,
and rows inside need no variant.

A sidebar that folds away and peeks: give the `SplitView` pane a body that is an `EdgePeek` (holding the
`Sidebar`) and mark the pane `SplitPane::new(spec, body).shown(pinned).peeking()`. A folded pane clips its
body, so an `EdgePeek` inside one stayed out of reach; a peeking pane stops clipping once it has folded
away, so the 10 px strip at the window's edge takes the pointer, the sidebar floats out over the content,
and a click on the strip pins it (`EdgePeek { onpin }`, which sets the pane's `shown` again). Keep one
`Shown` for both (`pinned` of the `EdgePeek`, `shown` of the pane, `on_shown` of the `SplitView` writing it),
so a drag of the divider, a toolbar button and the edge click move the same state. Pinned, the `EdgePeek`
fills its column's height, so the `Sidebar` inside keeps its foot at the bottom. No host element outside the
pane is needed.

A menu from a toolbar button: `Toolbar { onpick }` is an `EventHandler<Picked<T>>` (a handler is
`move |pick: Picked<Cmd>| ...`, not `move |cmd: Cmd| ...`) and hears a `Picked<T> { value, anchor: Option<Anchor> }`;
an app that only wants the item's value reads `pick.value`.
`anchor` is the button the pick came from (`Anchor::Mounted`, the chevron for an item that was behind it),
so keep it and hang the `Menu { anchor, placement: MenuPlacement::Popup }` or `Popover` from it; no
button of your own in the title is needed.

A sheet from a pane, not the window: `Sheet { attach: Attach::Within(anchor) }` hangs from the top edge of
the pane `anchor` names (`Anchor::Mounted` of the card's element, kept from its `onmounted` with
`use_rect().anchor()`, or `Anchor::Rect` in client coordinates), centred over that pane and `min(width, 88%)`
of it wide, clipped by the pane as it slides in, so the sidebar and the rest of the window stay clear. A
mounted pane is measured once it has laid out and again when the overlay settles, so a Space or a layout
that is still moving needs nothing from you; a pane that is resized while the sheet is up needs a fresh
`Anchor::Rect`. Render the `Sheet` once the anchor exists (`if let Some(anchor) = card.anchor()`); no
class of your own on the sheet to move its top.

Anchors, hover-card parts and undo:

- `Button` takes `common.mounted: Option<EventHandler<MountedEvent>>`: the element
  itself, for `Anchor::Mounted` (the Overlays example above). It writes no attribute.
- `HoverCard` takes `parts: Vec<HoverCardPart>` (`Title`, `Sub`, `Person`, `Stats`, `Flag`,
  `Messages`, `Foot`, `Actions`), drawn in order before its children: no hand-written
  `ds-hovercard-*` markup.
- `ToastHub::push_undoable(text, token, on_undo)` calls `on_undo` with the token when the person
  undoes that toast; a later push replaces it. The handler belongs to the scope that made it,
  so that scope must outlive the toast. `last_undo()` still reports the last undo.
- `Surface` takes `accent` and `blur` beside `theme` (section 4).

External icons and a caller-driven tooltip (FINDINGS "Pointer events"):

- **External icons.** An icon slot takes `ds::prelude::IconSource`: `Glyph(Icon)`, `Symbolic(ExternalIcon)`
  or `Image(ExternalIcon)`. `ExternalIcon { url: IconUrl, size: IconSize }` is a `data:` or
  `file:` URL and the square size it is drawn at. Build the URL with `IconUrl::png(&bytes)` (a
  tray pixmap you have PNG-encoded; quire does the base64), `IconUrl::svg(&document)`,
  `IconUrl::file(&absolute_path)` (an icon theme lookup), or `IconUrl::parse(url)`, which refuses
  any other scheme (`DsError::IconScheme`). `Symbolic` paints the icon's alpha in the text colour
  (`mask-image` over `currentColor`), so it follows `--ink`, hover, pressed and `--f-ink*` exactly
  like a glyph; `Image` shows the bitmap as it is. Which to use is design/08-ICONS.md §1.5's rule
  (freedesktop `*-symbolic`, or a pixmap whose opaque pixels all have OKLCH chroma < 0.04, is
  symbolic; anything coloured is an image) and is the caller's decision. `Button { icon }`
  takes an `IconSource`, and an `Icon` (or `Option<Icon>` for `Button`)
  still converts, so existing call sites are unchanged. `ds::prelude::IconView { source, size }` draws one
  anywhere else. The URL loads through the document's net provider (ds-blitz and shell-host's
  `LocalNet` answer `data:` and `file:`), one frame late.

  ```rust,ignore
  let icon = ds::prelude::IconSource::Symbolic(ds::prelude::ExternalIcon {
      url: ds::style::icon::url::IconUrl::file(&theme_path).expect("an absolute path"),
      size: ds::prelude::IconSize::Base,
  });
  rsx! { Button { bezel: Bezel::Toolbar, image: ImagePosition::Only, icon: Some(icon), label: title, onclick } }
  ```
- **Pointer buttons and ids.** `Button` takes `common.id: Option<String>`, written as
  the element's `id` (a popup anchors to `tray-3` with no wrapper span). Their `onclick` is
  `EventHandler<ds::base::press::Press>`, `Press { button: PointerButton::{Primary, Secondary, Middle},
  modifiers }`: a right-click (which Blitz and browsers deliver as `contextmenu`, never as a
  click; its default is prevented) arrives as `Secondary`, the middle button as `Middle`, and
  Enter or Space on the focused control as `Primary`. A closure written `move |_| ...` compiles
  unchanged and ignores the button; so does an `EventHandler<()>` you already hold (passed
  straight through). A closure written `move |()| ...` does not: write `move |_|`.

  ```rust,ignore
  onclick: move |press: Press| match press.button {
      PointerButton::Secondary => open_menu(),
      PointerButton::Primary | PointerButton::Middle => activate(),
  },
  ```
- **Menu submenus and disabled items.** `MenuItem::Item` has `availability: Availability` (`with_availability`);
  a disabled item is drawn at .35 opacity with `aria-disabled="true"`, skipped by Up and Down,
  and a click on it does nothing. `MenuItem::Submenu { title, image, availability, children }`
  is a row with a chevron that opens `children` beside the menu on a 200 ms rest, or at once on
  Right, Enter or a click; Left or Escape closes one level, and a picked child's value reaches
  the menu's `onpick` (the whole menu closes first). Submenus nest to any depth. The timing is
  the `timing: MenuTiming` prop (default 200 ms delay, 300 ms safe-triangle timeout; read
  `menus.submenu_delay_ms` into it), driven by the same `MenuTrack` machine as the bar's menus.
  `expanded: Option<usize>` opens a choice's submenu as the menu mounts (choices count items and
  parents, not headers or rules). A submenu is placed in the same document as its menu, so on a
  shell surface whose popup is sized to the menu, the popup must leave room for it.

For a bar (FINDINGS "Bar gaps"):

- **A Blitz host that is not `ds_blitz::launch`** (shell-host's surfaces, a popup's document)
  calls `ds_blitz::provide_host()` at the top of its root component, before any quire
  component reads the document. It installs every part of `ds::prelude::DocumentHost` at once (rects,
  focus, caret, scroll, edit, drop), so a root can never hold a subset; a root under a window's
  or the harness's host keeps that one. Drop your own copy of the twelve-line measurer. With no
  host at all (`ds::host::no_host::NoHost`, a server render) a rect read answers unknown and the anchor is not
  placed.

  ```rust,ignore
  #[component]
  fn BarRoot() -> Element {
      ds_blitz::provide_host();
      rsx! { Ds { appearance, material: Material::Bar, look, /* … */ } }
  }
  ```
- **Chrome materials draw the Space (target (clean-up phase): this goes).** The design is one quiet
  Look with a flush window: the Space tints a sidebar's ground only and shell chrome is the
  material alone (design/30 §3.4, design/21 §3), so a consumer will pass `look` to the window
  that has a sidebar and nothing to Bar, Dock, Osd or Widget roots; the A/B layers, `.ds-frame`,
  `--t-scene` and `appearance.material_tint_alpha` below are the code that is being removed.
  Until it lands, a `Ds` in `Bar`, `Dock`, `Osd`, `Widget` (and a
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
  spare room is alpha 0 (`crates/ds-conformance/tests/bar_frame.rs` proves it over
  `Harness::render_over(Backdrop::Clear)`).
- **The frame ground (target (clean-up phase): removed with the frame model; the `--f-*` inks go).** Under `data-ground="frame"` (a Bar or Dock root, or `Surface { on:
  Some(Ground::Frame) }`) `--ink`, `--ink-soft`, `--ink-faint` are the Space's `--f-ink*`,
  `--surface` is `--f-pill-hover`, `--surface-2` and `--raise` are `--f-pill`, `--line*` is
  `--f-line`: `Button`, `Chip`, `Badge`, a menu's trigger and your text all draw in
  the frame inks with no variant of their own. Overlays opened from it (menus, popovers,
  tooltips) are paper again.
- **Status items.** `ds_shell::prelude::MenuBarItem { image: ImagePosition::Only, icon, label, .. }` is a square of
  `--bar-status-box` holding its glyph (or external icon) at `--bar-status-glyph`,
  `--f-ink-soft` at rest, `--f-ink` on `--f-pill-hover` under the pointer, `--f-pill` when
  `value` is `On` or `shown` is `Visible`. Write the two properties on any element around your items
  with `ds::style::tokens::status::StatusMetrics` (the bar's `status_icon_box_px` and `status_glyph_px` settings, 22 and 16 by default):

  ```rust,ignore
  let metrics = StatusMetrics { box_size: Px(22.0), glyph: Px(16.0) };
  rsx! { div { class: "status", style: metrics.style_attr(), /* MenuBarItem { image: Only, .. } … */ } }
  ```
  (`style_attr()` is `--bar-status-box:22px;--bar-status-glyph:16px;` at the defaults; custom
  properties with lengths on your own element pass the markup lint.) An external icon in a
  status item is sized by the property too, whatever its `ExternalIcon::size`.
- **Menu control.** `Menu` takes `on_hover: Option<EventHandler<Option<usize>>>` (the choice
  under the pointer, numbered as `expanded` counts; `None` once it is over none),
  `on_release: Option<EventHandler<Press>>` (every button released over the menu),
  and plays `Anim::MenuOut` (`--t-quick --e-exit`, `data-presence="leaving"`) before `onclose`
  when Escape or an outside click closes it. A pick calls `onpick`, then `onclose`, once. A
  release over an enabled item after a press that began outside the menu picks it
  (press-drag-release); over a disabled item, a header or the padding it closes picking
  nothing. The owner removing the menu (a hover switch) is immediate, no fade.
- **Commands for the conformance check.** Every menu item and shortcut is the face of an action
  or says it is interface only. The command type your items yield and your shortcuts fire
  implements `ds::base::command::AppCommand` (`id()`: your stable id, `face()`:
  `CommandFace::Action(ActionName("mail.draft.create".into()))` or
  `CommandFace::UiOnly(UiOnlyReason::new("window chrome")?)`; `ActionName` is a plain string, so
  `ds` does not depend on the manifest that declares it). A shortcut outside any menu is a
  `ShortcutBinding { keys, command }`. A test walks the bar, your own menus and the table into
  `ds::components::menus::ui_manifest::{bar_rows, menu_rows, shortcut_rows}` and writes the file
  `docket-eval --check-app` reads, beside the intents manifest:

  ```rust,ignore
  let rows = [bar_rows(&bar), menu_rows(&tray_items), shortcut_rows(&SHORTCUTS)].concat();
  let text = UiManifest::new(IntentsApp("org.quire.Mail".into()), rows)?.to_toml()?;
  std::fs::write("dist/intents/org.quire.Mail.ui.toml", text)?;
  ```

  A menu item in the bar and in a context menu is one row; a shortcut and a menu item for the same
  command are a row each; one id declared with two faces is an error. The file is
  `vocab = 1`, `app`, `[[commands]] id, source = "menu" | "shortcut", chord?, action?` and
  `[[ui_only]] id, reason`; a chord is written `cmd+n` (`Shortcut::chord`).
- **Status lines.** `MenuItem::Info { title, detail: Option<String> }` is a row at an item's
  weight without the header's eyebrow, never a choice: keys, hover and picks pass it by. Use it
  for a network's address or a battery's time left, instead of headers.
- **`Press { button, modifiers, at }`**: `at` is the surface-local point (the event's client
  point); a keyboard activation reports the origin. `Press` is `PartialEq`, not `Eq`. Hand
  `at.x`, `at.y` (converted to screen coordinates if you know the surface's origin) to SNI's
  `Activate` and `ContextMenu`.
- **Symbolic or image.** `ds::style::icon::classify::classify_with(&png_bytes, ChromaLimit::default()) ->
  Result<IconKind, DsError>` (`IconKind::{Symbolic, Image}`; `ChromaLimit` is
  `ds::style::icon::classify::ChromaLimit`) is design/08-ICONS.md §1.5 step 2: symbolic when every
  pixel with at least half coverage has OKLCH chroma below the limit; `ChromaLimit(n)` takes
  another threshold in thousandths. Pick `IconSource::Symbolic` or `Image` from it. The
  0.04 default is `ChromaLimit::default()`; a settings value (design/22-SETTINGS.md §3.3
  `icons.symbolic_chroma_max`, plain chroma, not thousandths) goes through
  `ChromaLimit::try_from(value)`, which refuses anything outside `0.0..=0.2`
  (`DsError::ChromaLimitRange`); reading the key is the settings-owning side's job.
- **New glyph**: `Icon::Ethernet` (Lucide `ethernet-port`) for a wired network.

For a launcher and a dock (FINDINGS "Launcher gaps"):

- **Unmounting early is safe.** Every task quire spawns (a motion timer's settle, a roster's
  exit and rest, the hover hub's and the toast hub's timers, a focus retry) is a task of the
  scope that owns it and is dropped with that scope, and writes only through fallible handles.
  A palette, menu or popover may be unmounted at any moment, mid entrance included: drop any
  keep-mounted guard you added for it. `MotionTimer::start`'s `on_settled` never runs for a
  component that is gone.
- **Focus waits out a busy document.** `Focus::OnMount`, a menu taking the keyboard and every
  other focus change go through the host's `ds::host::parts::FocusHost` (`Focused::{Done, Busy, Unknown}`),
  tried again a frame later while the renderer holds the document. A Blitz host that is not
  `ds_blitz::launch` provides it with the rest of the host:

  ```rust,ignore
  #[component]
  fn LauncherRoot() -> Element {
      ds_blitz::provide_host();
      rsx! { Ds { appearance, material: Material::Sheet, look, /* … */ } }
  }
  ```
- **Giving a field the keyboard back.** `let field = ds::focus::request::use_focus_request();` then
  `TextField { focus: FieldFocus::Controlled(field), .. }` (or `CommandPalette { focus: Some(field),
  .. }`), and `field.request()` from a handler (a menu's `onclose`) whenever the field should
  have the keyboard again. The field takes it as it mounts and at each request; nothing is
  remounted, so a palette does not replay its entrance.
- **An embedded palette.** `CommandPalette { host: CommandPaletteHost::Surface, .. }` draws no
  scrim and no overlay: the card fills its container (give the container the panel's size) and
  paints the enclosing material's tint, edge and radius-14 corner (`--r-panel`), its list taking
  the height under the field. `id: Some("…")` goes on the card, for a blur region. `Overlay` is
  the default host. Escape in the field still closes it through the layer stack, so a
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

  ```rust,ignore
  let mut row = use_signal(|| None::<Rect>);
  let field = use_focus_request();
  rsx! {
      CommandPalette::<Hit> {
          label, placeholder, query, tokens: Vec::new(), groups, empty, oninput, onpick, onclose,
          host: CommandPaletteHost::Surface,
          id: "launcher-card".to_string(),
          focus: field,
          on_select_rect: move |rect: Rect| row.set(Some(rect)),
          onkey: move |key: KeyboardEvent| if is_actions(&key) {
              key.prevent_default();
              actions.set(true);
          },
      }
      if let (true, Some(rect)) = (actions(), row()) {
          Menu { placement: MenuPlacement::Popup, anchor: Anchor::Rect(rect), items, onpick,
                 onclose: move |()| { actions.set(false); field.request(); } }
      }
  }
  ```
- **App icons in rows.** `RowLeading::Source(IconSource)` (and `MenuImage::Source` in a menu) puts any icon in a
  row: an `Image` (an app's icon file) is drawn as it is with no plate under it, whatever size it was
  resolved at; a `Symbolic` is drawn in the text colour; a `Glyph` is `RowLeading::Icon`'s.
- **Icon sizes for tiles.** `IconSize::Tile48` (48), `IconSize::Tile96` (96) and
  `IconSize::Px(IconPx(n))` for any size a caller resolves (a magnified dock tile): an icon is
  drawn at that size, not scaled from 22.
- **A caller-driven tooltip or label.** `Tooltip { shown: Some(Shown::Visible | Shown::Hidden), .. }`
  (and the shell's `DockLabel`) shows or hides on the caller's say alone, at once, whatever the
  pointer does (the dock's label machine: hide on press, while a menu is open, while dragging).
  `None` follows the pointer through the hover hub: a tooltip waits by the Tip profile (1 s), a
  dock label by the Label profile (100 ms), both at once while the hub is warm.

Palette behaviour:

- **The selected row's rect waits for layout.** A palette mounted on a surface that has not been
  laid out yet (a launcher surface mapped again) reads its selected row as 0 x 0; quire now
  reads it again every frame for about half a second, then every 100 ms for three seconds more,
  and reports it only once it has an area. It measures again when the selection, the row
  element or the results (groups, tokens) change, and drops a read still waiting when a newer
  one starts. Drop any "empty rect means unknown" fallback: `on_select_rect` never hands you
  one.
- **`onkey` hands on the event.** `CommandPalette { onkey: Option<EventHandler<KeyboardEvent>> }`
  (and `TextField`'s `onkey: EventHandler<KeyboardEvent>`): call
  `event.prevent_default()` on a key you take, and Blitz does not also act on it (Tab no longer
  moves the focus off the field). A closure typed `|key: KeyboardData|` becomes
  `|key: KeyboardEvent|`; `key.key()`, `key.modifiers()` read as before.
- **A palette kept mounted.** `CommandPalette { shown: Some(Shown::Visible | Shown::Hidden),
  }` (`Shown` is the tooltip's). Hidden, the card is
  `display:none` (nothing laid out or painted, no scrim) and leaves the layer stack, while its
  rows and field stay in the document. Each change to `Visible` replays the entrance
  (`cmdk-in`/`peek-in`, restarted through the keyframe's `X--b` alias), reports the selected
  row's rect afresh, gives the field the keyboard and, asks for an
  empty query through `oninput("")` and goes back to the first choice (a controlled selection is
  asked for 0 through `on_select`); a re-shown palette always resets. A palette mounted
  hidden does not take the keyboard until it is first shown. `None` (the default) is the
  palette as before: shown, its entrance played as it mounts.

  ```rust,ignore
  let mut shown = use_signal(|| Shown::Hidden);
  // The shell's toggle: shown.set(Shown::Visible) on open, Shown::Hidden on hide.
  rsx! {
      CommandPalette::<Hit> {
          label, placeholder, query: query(), tokens: Vec::new(), groups, empty,
          oninput: move |text: String| query.set(text),
          onpick, onclose: move |()| shown.set(Shown::Hidden),
          host: CommandPaletteHost::Surface,
          shown: shown(),
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
  (`crates/ds-conformance/tests/palette_actions_key.rs`):

  ```rust,ignore
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
the `pdfrum-anyrender` painter writes it through pdfrum (text stays text in embedded, subsetted
faces at the layout's variable instance; a JPEG or an opaque PNG is embedded as it arrived). FINDINGS.md "PDF output" has the reasons, the limits and the
measured numbers.

| Need | API | Notes |
| --- | --- | --- |
| An HTML document as a PDF | feature `pdf`: `ds_blitz::pdf(&html, PageSpec::default()) -> Result<Vec<u8>, PdfError>` | A whole document (`<!DOCTYPE html>...`). quire's faces are registered; the network is sealed: only `data:` URLs load (no `file:`, no fetch). `@media print` applies. Laid out once at the page's content width, at scale 1. |
| A Dioxus tree as a PDF | `ds_harness::pdf_app(app, HarnessConfig::new(viewport), spec)`, ds-harness feature `pdf` | Built as `config` says (its contexts and `NetPolicy`; the viewport is replaced by the page's content box), rendered until its mount-time work and images have landed (as `snapshot`), then printed. |
| In a test | `Harness::pdf(spec) -> Result<Vec<u8>, PdfError>` | Prints the harness's document as it is now, at the page width with `@media print`; the harness's own viewport and media come back afterwards. Read the PDF back with `pdfrum` (dev-dependency) to assert on text. |
| The sheet | `PageSpec { size, margins }`; `PageSize::{A4, Letter, Custom { width: Pt, height: Pt }}`; `Margins { top, right, bottom, left }`, `Margins::uniform(Pt)`, `Margins::symmetric(vertical, horizontal)`; `Pt::from_mm`, `Pt::from_inches` | `PageSpec::default()` is A4 with 18 mm above and below, 16 mm at the sides. `@page` is not read: the spec's margins are the only ones. `PdfError::NoContentArea` when the margins meet. |
| Start a new page at an element | `"data-break-before": "page"` | CSS `break-before`/`page-break-before` do nothing on Blitz (stylo drops them). A marker at the document's top makes no blank page. |
| Keep a box on one page | `"data-break-inside": "avoid"` | Moved whole to the next page when it would straddle the cut, unless it is taller than a page (then it is cut between its lines). Lines, images, `svg`, `canvas`, `iframe` and table rows are kept whole on their own; a text block keeps its first two and last two lines together (orphans and widows of 2). |
| Screen-only or print-only styling | `@media print { .. }` | Applies in `pdf`, `pdf_app` and `Harness::pdf`. |
| CJK text | name the family: `font-family: "Noto Sans CJK TC", sans-serif` | Otherwise fontique's fallback picks (DroidSansFallback on this machine). A variable face prints at the instance the layout used. |
| Print it | `ds_blitz::print_dialog(&pdf, title) -> Result<PrintOutcome, PrintError>`, feature `print` | Linux: the desktop portal's print dialog (GTK or KDE backend), then the portal prints the PDF. No portal or no print backend, and other systems: the PDF is written to a temp file and opened in the viewer. `PrintOutcome::{Printed, Cancelled, Opened(PathBuf)}`, `PrintError::{Portal, Write, NoViewer}`. **Blocks** until the dialog is answered: call it off the UI thread. The dialog is not parented to the window. |
| Try it by hand | `cargo run --release -p ds-blitz --example pdf -- out.pdf`; `cargo run -p ds-blitz --features print --example print` | The first writes the fixture and prints its size and time; the second opens the real dialog. |
| The painter alone | the `pdfrum-anyrender` crate, in the pdfrum repo | Pages recorded into anyrender's `Scene` by any anyrender renderer, written as one PDF; Blitz-free. `ds-blitz`'s `pdf` feature depends on it as a git dependency pinned to pdfrum rev 61371040, so a consumer that only prints through `ds_blitz::pdf` names nothing new; one that calls the painter directly adds the same git dependency at the same rev. Its own docs describe its page and source types. |

### Missing helpers

An app that runs distro tools (mpv, ffmpeg, heif-dec) can ask for one that is not installed, like
Totem's codec prompt: a sheet says what the app needs, the person says yes, PackageKit installs it
(and asks polkit for the password itself), and the app retries without restarting. Two parts:
`ds-helpers` (I/O, no renderer; zbus only) and the `HelperSheet` in `ds-shell`.

**The rule: ask at the moment of use.** Probe when the person tries the feature that needs the
tool (`Helpers::available`), show the sheet then, and never at launch or in the background. A
person who says Not Now is not asked again until they try the feature again.

**The data file.** Ship `$XDG_DATA_DIRS/quire/helpers/<app>.toml`. One top-level table per
capability (lowercase letters, digits, hyphens):

```toml
[heic-decode]
tool = "libheif tools"          # the name the person knows
purpose = "open HEIC photos"    # finishes "{app} needs {tool} to ..."
probe = ["heif-dec", "heif-convert"]   # any one on PATH proves it is there
[heic-decode.packages]          # per family, alternatives in preference order
dnf = ["libheif-tools"]
apt = ["libheif-examples"]
pacman = ["libheif"]
```

Families are `dnf`, `apt`, `pacman`, `zypper`, chosen from `ID` then `ID_LIKE` in
`/etc/os-release`. A family you leave out, or any other distro, is `Outcome::Unsupported`.

**The API.**

| Need | API |
| --- | --- |
| Load the file | `Catalog::load("anyview", &data_dirs(std::env::var_os("XDG_DATA_DIRS").as_deref()))` (or `Catalog::parse(text)`) |
| The handle | `Helpers::new(catalog, Environment::system(), Installer::PackageKit(PackageKit::system()))`; cheap to clone |
| Is it there? | `helpers.available(&Capability::new("heic-decode")?) -> Presence::{Present, Missing}` |
| Words for the sheet | `helpers.entry(&capability) -> Option<&Entry>` (`tool`, `purpose`, `probe`) |
| Install it | `helpers.provide(&capability).await -> Outcome::{Installed, Declined, NotFound(Missing), Unsupported(Missing), Failed(reason)}`; call it after the person pressed Install... `Missing { capability, package: Option<PackageName>, program: Option<Executable> }` is the first candidate package for this distro and the first probe program |
| Hear of changes | `let mut feed = helpers.subscribe(); feed.next().await -> Availability { capability, presence }`; `helpers.refresh(&capability)` probes again (a tool installed in a terminal) |
| Tests and the gallery | `Installer::Fake(FakeInstaller::new(Outcome::Installed))`; `Environment { path, os_release }` takes scratch paths; `.leaving(dir, &["mpv"])` drops empty stand-in tools into `dir` on install, `.leaving_with(dir, vec![StandIn { name, body }])` gives each one a script |

`provide` returns `Installed` at once when the tool is already there, resolves the candidates
through PackageKit (the first that exists in the repositories wins), installs it, probes again,
and announces the change to subscribers. It never runs a package manager or sudo. A Flatpak
sandbox cannot reach the system PackageKit: the app gets `Unsupported`.

**Availability is announced on `refresh` and `provide` only.** quire does not watch `PATH`. An
app that wants live updates (a tool installed in a terminal while the app runs) watches `PATH`
itself and calls `refresh` when it changes. `provide` also announces: the window that asked sees
both its `Outcome` and an `Availability` on its feed, so handle the two as one event, not two.

**The sheet's style.** The `.ds-helper-progress` rule is in the shell stylesheet, so the app's
`Ds` root must draw with it, or the Installing phase has no progress styling:

```rust,ignore
rsx! { Ds { sheet: Some(ds_shell::stylesheet()), HelperSheet { /* ... */ } } }
```

`ds_shell::helpers::{HelperSheet, HelperBody, HelperPhase}` (also in `ds_shell::prelude`).

**The sheet.** `HelperSheet { app, tool, purpose, phase, icon, on_install, on_dismiss }` is
controlled: you own the `HelperPhase` (`Ask`, `Installing`, `Failed { reason }`, `NotFound {
package }`, `Unsupported { program }`). Show `Ask` first; on `on_install` set `Installing` and
await `provide`; map the outcome: `Installed` closes the sheet and retries the feature, `Declined`
returns to `Ask` or closes, `Failed` / `NotFound` / `Unsupported` show their phase (`NotFound` names
the first candidate package, `Unsupported` the first probe executable: take them from the `Missing` in the outcome). `on_dismiss` is Not Now and Close;
Escape does nothing while `Installing`. `HelperBody` is the same column without the sheet.

### Exporting the menu bar

A quire app's `MenuBarModel` can be served over the session bus as `com.canonical.dbusmenu`, so the
shell's bar shows the focused app's App, File, Edit, View, Window and Help menus and an agent can
list and run them (design/27 section 5.2 rule a). Off by default: feature `menus` of `ds-blitz`.

1. Make every command type answer `AppCommand::id` (the ids your UI manifest already lists). A
   command that wraps `BarCommand` answers with `BarCommand::id()` (`standard.undo`, `bar.about`).
2. At startup, with the window's Wayland `app_id` (the one you give `AppConfig::with_app_id`):

   ```rust,ignore
   let tree = MenuTree::from_bar(&bar)?;                      // ds::components::menus::export
   let export = MenuExport::start(                            // ds_blitz::menus, feature `menus`
       &MenuExportConfig { app_id: AppId("dev.mailo.Mailo".into()), bus: Bus::Session },
       tree,
       move |command: CommandId| { let _ = tx.send(command); }, // runs on a bus thread
   )?;
   ```

3. On the UI thread, turn the id back into your command and run the handler your own menu and
   shortcuts run: `bar.command(&id)` gives `Option<&T>`. Hand the id over from the bus thread with a
   channel and `AppHandle::redraw`, as for any other off-thread request.
4. Whenever the bar changes (an item enables or disables, a check flips, an item appears), call
   `export.update(&bar)`. It returns what changed and announces it (`LayoutUpdated`, and
   `ItemsPropertiesUpdated` for a property-only change); the same bar is no change and no signal.

| What | Value |
| --- | --- |
| Bus name | `org.quire.AppMenu.<app_id>`, each dot-separated element made a legal bus-name element (`AppMenuAddress::for_app_id`); `NameOutcome::Queued` when another process of the same `app_id` owns it (this one inherits the name when that process exits, and is served on its unique name meanwhile) |
| Object path | `/org/quire/AppMenu` |
| Interface | `com.canonical.dbusmenu` version 3: `GetLayout`, `GetGroupProperties`, `GetProperty`, `Event`, `EventGroup`, `AboutToShow(Group)`; signals `LayoutUpdated`, `ItemsPropertiesUpdated` |
| Item properties | `type`, `label` (underscores doubled), `enabled`, `toggle-type`, `toggle-state`, `shortcut` (`[["Super","Z"]]`: Command is `Super`), `children-display`; plus `x-quire-command-id`, the `CommandId` the item runs |
| Ids | an `i32` hashed from the item's `CommandId` (a menu, submenu or rule from its place), so the same bar has the same ids in every run; the bar itself is 0 |
| Event | `Event(id, "clicked", _, _)` on an enabled command item calls your callback with its `CommandId`; a disabled or unknown item is an `InvalidArgs` error |

Tests start a private `dbus-daemon` and pass `Bus::Address(..)`; never export to the real session
bus from a test (`crates/ds-harness/tests/menu_export.rs` is the model). Under a shell that turns on
zbus's `tokio` feature, `MenuExport::start` enters the process-wide runtime itself.

### Pointer capture, gestures and the capsule

For a viewer: a drag that keeps following the pointer outside its element, a touchpad's pinch and
scroll with their phases, and the pill of controls that floats over content. FINDINGS "Events"
has the reasons: Blitz has no pointer capture, no pinch event and no wheel phase, but the window
sees winit's events before the document does, so ds-blitz publishes them and nothing in the
Blitz fork is needed.

| Need | API | Notes |
| --- | --- | --- |
| A drag that leaves its element | `ds::host::pointer_capture::use_pointer_capture(on_pointer) -> PointerCapture`; wire `capture.on_mounted(event)` to `onmounted`, call `capture.begin() -> PointerHold` from `onpointerdown` | `on_pointer` hears every move (`PointerPhase::Drag`) and the primary release (`PointerPhase::Release`) in the window's logical pixels, wherever the pointer is, until the button comes up. `PointerHold::Local` means the host cannot (no host, not mounted yet): the element's own `onpointermove` and `onpointerup` are all there is. The edit surface uses the same route. |
| A pinch or a phased scroll | `ds::host::gesture::use_gestures(on_gesture)`; `Gesture::Pinch { phase, by: Magnification, at }`, `Gesture::Scroll { source: ScrollSource, phase, by: Point, at, held: Modifiers }`, `ScrollSource::{Wheel, Finger}`, `GesturePhase::{Began, Changed, Ended, Cancelled}` | A gesture reaches every listener with the pointer's place; the listener checks it is over its own element. `Magnification(50)` is 5% larger. A scroll's `by` is how far the content moves in logical pixels (winit's sign; a wheel detent is 60 px, the distance a native container moves for it, and a wheel with no phases reports `Changed` only; `source` says which: `Wheel` for detents, `Finger` for a touchpad (winit reports a pointing stick as pixels too, so it reads as `Finger`); `held` is the modifier keys down, so a wheel under Control can zoom). `use_gestures_with(WheelDelivery::Eased, ..)` hears the detents eased over frames instead, and a touchpad's run with its glide (section 11). Pinch exists on Wayland and macOS, where winit has it. |
| In a test | `Input::gesture(Gesture::Pinch { .. })`; `Input::wheel(..)` publishes a scroll too | `ds_blitz::launch` and `Harness` provide the `GestureBus`; with no host `use_gestures` hears nothing. |
| A measure that follows the window | `ds::host::measure::use_rect()` | It reads again after each `ds::host::resized::WindowResized` bump, which `launch` makes on every window resize or scale change; a host with no such source (a shell surface) never bumps, and the rect stays where it was measured. |
| A machine whose parameters come from its own state | `MachineRef::set_params(params)` before `send` | `use_machine` takes parameters at each render, one render behind a machine whose parameters are derived from its state (a zoom step reads the scale the last step made): compute them from the state just read and set them, then send. |
| A timed machine your surface owns | `ds::machine::use_machine(|now| first_state, params, || ctx, |out, machine| ...) -> MachineRef<M>`; the controlled form is `use_machine_state(|now| first_state)` then `use_machine_in(held, params, || ctx, on_out)` | There is no `Default` bound: you give the first state, and `now` is the stamp of the mount, so a seeded deadline counts on the hook's clock. `ctx` is a closure that reads the facts the step needs (`type Ctx`, `()` if none); it runs at every step, including the one a wake causes. `on_out` gets the `MachineRef`, so a follow-up is `machine.send(input)`. With `MachineState { state, clock }` you hold the signal yourself: read it from a sibling, persist it, or write it (the timer follows the state's `wake()`). From a component body (an input made by a prop) use `send_from_render`, before reading the state. |
| A floating pill of controls | `ds::components::chrome::capsule::view::Capsule { label, slots: Vec<CapsuleSlot<T>>, shown, on_hidden, onpick, onpointerenter, onpointerleave }`; `CapsuleSlot::{Item(ToolbarItem<T>), Readout(String), Divider}` | Bottom centre of its parent, which must have a size and `position:relative`. A card in the `Osd` material; fades in and out over `--t-quick` by `shown` and calls `on_hidden` once it has gone. `onpointerenter` and `onpointerleave` are how an owner keeps it up while the pointer is on it. |

### GPU textures

A GPU texture inside the document, for decoded images, PDF page tiles and video frames: a
`TextureLayer` fills its parent's box and draws a `TextureHandle` the app writes into from any
thread. The device is the window renderer's own, so a texture you make on it is drawn without a
copy. FINDINGS.md "Texture layer" has the reasons and limits.

| Need | API | Notes |
| --- | --- | --- |
| The window's device | `ds_blitz::use_gpu() -> Gpu`; `gpu.device()`, `queue()`, `adapter()`, `instance()` | `None` until the window's first frame; the component calling `use_gpu` re-renders when it arrives (and when `gpu.generation()` changes, after which textures made on the old device stop drawing). One `Gpu` per window. |
| A handle to show | `gpu.handle()` (empty), `gpu.register(Texture)`, `gpu.register_view(TextureView)`, `gpu.upload(&Pixels)` | A registered texture is premultiplied, non-sRGB (`Rgba8Unorm`, `Bgra8Unorm`, `Rgba16Float`), one mip, `TEXTURE_BINDING`. |
| CPU pixels | `Pixels::new(PixelFormat, width, height, &bytes)`; `handle.update(&pixels)` | `Rgba8Premultiplied` (pdfrum's `Pixmap`) as is, `Rgb8` made opaque, `Rgba8Straight` multiplied at upload. The same size writes in place; another size makes a new texture. Errors: `GpuError::{NotReady, TooLarge}`. |
| The element | `TextureLayer { texture, fit, source, sampling, pace, common }` | `fit: TextureFit::{Contain, Cover, Fill, Actual, Tile}`; `source: Option<TexelRect>` in texture pixels (crop; the whole texture when `None`); `sampling: Sampling::{Nearest, Bilinear, Bicubic}`; `pace: Pace::{OnDemand, EveryFrame}`. It fills the box it is in (give the parent a size, as for any frame), and takes part in paint order, `opacity`, `filter` and `border-radius` like an image. `Actual` and `Tile` are one texel per device pixel. |
| A new frame | `handle.replace(texture)` or `replace_view`, or write into the registered texture on `gpu.queue()`; then `handle.redraw()` | `redraw` asks each window showing the handle to repaint and is cheap to call per decoded frame (one paint per display refresh, of the latest texture); submit your GPU work first. `replace` and `update` redraw by themselves. `handle.clear()` shows nothing. |
| In a test | `HarnessConfig::new(viewport).with_backend(Backend::Hybrid)`; `Harness::gpu()`; `harness.render_over(Backdrop::Clear)` | The harness device is the app's `use_gpu()` from the first render. The default vello_cpu backend has no device and draws nothing. |
| Try it by hand | `cargo run -p ds-blitz --example texture_layer` | A thread uploads a moving gradient every frame. |

## 7. Settings schema: `#[derive(SettingsSchema)]`

If your app has its own settings struct (not `AppearanceSettings`/`IconsSettings`, which quire
already derives), give it a schema the same way so the Settings app can render it
(design/22-SETTINGS.md section 9):

The struct takes the derive (a `KeySpec` per field, from `#[settings(...)]`); every enum one of
its fields holds derives `ds::prelude::Word` with `#[word(case = snake)]`, whose `ALL` gives the derive
the variant words to pick a widget by arity (`ds_settings::schema::kind_from_variants`, section
9.1). The trait `.schema()` calls through is
`ds_settings::schema::SettingsSchema` — a different item from the `SettingsSchema` the derive
macro re-exports at the crate root (`crates/ds-settings/tests/schema.rs` is quire's own worked
example of both imports together):

```rust,ignore
use ds::prelude::*;
use ds_settings::SettingsSchema;               // the derive macro (macro namespace)
use ds_settings::schema::{Page, SettingsSchema};   // the trait `.schema()` needs (type namespace)
use serde::{Deserialize, Serialize};

#[derive(Debug, Clone, Copy, PartialEq, Eq, Hash, Default, Serialize, Deserialize, Word)]
#[serde(rename_all = "snake_case")]
#[word(case = snake)]
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
}                                // every field needs its own #[settings(label = "...")]

impl Default for ReaderSettings {
    fn default() -> Self {
        ReaderSettings { open_links: OpenLinks::default() }
    }
}
```

Wire `--write-schema <dir>` into your `main`'s argument parsing with
`ds_settings::schema::maybe_write_schema` (also in the `schema` module, not the crate root):

```rust,ignore
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
`Toggle` when its words are an on/off pair and two segments otherwise, three to five a
`SegmentedControl`, more a `Menu`, a `#[settings(range = "..")]` newtype a `Slider` — see
`ds_settings::schema::KeyKind::widget` (design/22-SETTINGS.md section 9.1) for the full table; you
never choose a widget yourself. quire's own schema is written the same way by the `ds-settings`
example: `cargo run -p ds-settings --example write_schema -- --write-schema target/schema`.

## 8. What Blitz cannot do, and what to use instead

Everything below is from `FINDINGS.md` spikes S1-S16 (blitz @ `e99fbdbd`, dioxus 0.7.10) — the
authority; this table is a pointer. `ds_lint::Rule::BlitzUnsupported`
(`crates/ds-lint/src/blitz.rs`) catches most of the CSS-level ones for you under
`Profile::Strict` (well, under any profile — it is not one of the three raw-geometry rules
`Profile::Standard` skips) if you write the banned property in your own stylesheet.

| Blitz cannot | Finding | Use instead |
| --- | --- | --- |
| `backdrop-filter: blur()` | S15 | ask the compositor to blur behind the surface (`Material`/`BlurState`), not CSS |
| CSS `stroke`/`fill` reaching `<svg>` children | S6 | `ds::style::icon::render::Glyph` (renders `.ds-ic` with `stroke="currentColor"` as an attribute, not a rule); `Rule::SvgPaintInCss` |
| `text-overflow: ellipsis` | S13 | `.ds-truncate` (a mask-image fade) or `ds::base::text::clip::clip_chars` for a real character-count ellipsis |
| `:focus-visible` / `:focus-within` (hard-coded `false`) | S12 | `.ds[*|data-modality=keyboard] :focus` — `Ds`/`ds_blitz::launch` track modality for you; `Rule::FocusPseudoClass` |
| `onmounted` + `get_client_rect()` inside the handler itself (returns 0×0) | S9 | `ds::host::measure::use_rect()` — measures one frame later, never inside the handler; to anchor an overlay, `Anchor::Mounted` does this for you |
| a click on a `Button` whose parent holds only inline content (the button alone, or beside text) | blitz-dom hit test | put the button in a flex row (every quire container is one) or a block; the parent of an atomic inline is hit instead (`crates/ds-conformance/tests/button_click.rs`, FINDINGS "Polish pass") |
| `mask-image:url(data:...)` / `background-image:url(data:...)` without a `data:` `NetProvider` | S7, S8 | `ds_blitz::launch`/`Harness` already install one; nothing to do if you use them |
| `mix-blend-mode`, `position: sticky`, `line-clamp`, `text-shadow` | risk table | avoid outright; `ds::base::text::clip::clip_chars` covers the line-clamp case |
| `line-clamp` for a multi-line clamp that opens on hover | notification parts | a `max-height` in whole `em` lines with a transition, and a fade decided by measuring (`NotificationCard`'s body); the hidden lines are still hit-tested, so give them `pointer-events:none` |
| a wheel phase (a touchpad gesture's end), a pinch | notification parts | `ds::host::gesture::use_gestures` (above): winit's phased wheel and pinch, published by the window; a window with no such source still ends a swipe by a quiet spell (`DelayToken::SwipeQuiet`, `use_swipe`) |
| a clean removal of a running animation | notification parts | Blitz keeps the last animated value when an animation is taken off an element before a frame resolved past its end: put an entrance on an element that mounts with it rather than on a presence attribute that changes (`BannerStack`) |
| `break-before`, `break-inside`, `page-break-*`, `@page` (printing) | FINDINGS "PDF output" | `data-break-before="page"`, `data-break-inside="avoid"`, and `PageSpec` margins, read by `ds_blitz::pdf` (section 6, "PDF and printing") |

What *does* work and needs no fallback: a `<style>` in the body (S1), the `.ds[data-*]` custom
property cascade with plain `[data-x=v]` attribute selectors (S2, fixed in the Blitz fork at `1b23fbd9`; quire's own sheet still writes `[*|data-x=v]`, which matches too), `@keyframes` including `var()` inside them (S3),
`transition` on a `var()`-driven value (S4), restarting an animation by swapping its name (S5,
what a keyframe pulse does), `futures-timer` sleeps from render or a handler (S10, what `ds::base::time::clock::sleep`
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

**Menu tracking** (`ds::stack::menu_track::types::MenuTrack<K>`, `crates/ds/src/stack/menu_track.rs`; design/13
§13.3.2-13.5). One machine drives bar menus and every ds `Menu`: open on press, click mode,
press-drag-release, hover switch between open menus, the submenu delay and the safe triangle.
`K` is your menu key (a bar title id). It is a `Machine`: step it with an event and a `Stamp`,
and perform what it returns, in order; its `wake()` is the one time it wants to be stepped again
(the submenu delay's end, the safe triangle's timeout), with `MenuTrackEvent::Tick` (from
`Elapsed`), and none at rest.

```rust,ignore
use ds::base::machine::Machine;
use ds::base::time::stamp::Stamp;
use ds::stack::menu_track::types::{MenuTarget, MenuTiming, MenuTrack, MenuTrackEffect, MenuTrackEvent};

let track: MenuTrack<u32> = MenuTrack::closed();
let timing = MenuTiming::default(); // 200 ms delay, 300 ms triangle
let (track, effects) = track.step(MenuTrackEvent::PressTitle(1), Stamp(0), &timing, &());
// effects == [MenuTrackEffect::Open(1, MenuAnim::Pop)]
// Prepare(item) says the pointer rests on a parent item (measure its submenu while the delay
// runs); SubPlaced{top, bottom} reports a submenu's near-edge corners so the safe triangle can
// arm.
```

Build `MenuTiming` from `menus.submenu_delay_ms` and `menus.submenu_triangle_timeout_ms`. Arrow
keys inside a menu, wrap, disabled skipping and filtering stay with the `Menu` component; the
machine takes `MenuKey::{Escape, Left, Right, Enter}` (map Space and Tab to `Enter`).

**Curves** (`crates/ds-style/src/tokens/curve.rs`). For motion you drive from a frame clock rather
than CSS: `EasingToken::Out.easing(level).at(Fraction(t))` gives progress in thousandths at time
`t` (thousandths). `Easing::curve()` gives the `CubicBezier` (`linear` is `(0,0,1,1)`), and
`CubicBezier::at` evaluates it. Integer arithmetic throughout; a spring's overshoot reads above
1000.

**Opening a `PopUpButton` in a test with no renderer.** The menu is placed against the button's
element, which a renderer reports after layout; a `VirtualDom` rendered with `dioxus-ssr` never
does, so the menu stays shut. Give it a place and it opens: `PopUpButton { start: Shown::Visible,
anchor: Some(Anchor::Point(Point { x: Px(40.0), y: Px(20.0) })), .. }` (or click the button once
`anchor` is set). Let the root's overlay take the menu with a few `render_immediate` rounds, as
`crates/ds/tests/pop_up_button_open_ssr.rs` does. On a real document `start` opens the menu once
the button is laid out, and `anchor` moves it elsewhere.

**Spaces store** (`ds::style::space::store::SpaceStore`, `crates/ds-style/src/space/store.rs`; design/21 §4, §10). Which
look each workspace wears. `store.look_for_workspace(&Workspace { id, index }, defaults)` looks
up by compositor id, then by position, then falls back to `PRESETS[index % 8]`;
`store.look_for(WorkspaceIndex(i), defaults)` skips the id. `SpaceDefaults` carries
`spaces.default_grain` (default 0; was 40) and `spaces.default_card_accent` (retiring; a Space has no card accent). `store.with_look(&workspace, look)`
records a look under the id (when there is one) and always under the position.

**Settings files** (`ds_settings::{SettingsDoc, Store}`). A file is a serde type that names
its file and format; a `Store` is one program's config directory, and the only way to load, save
and watch it:

```rust,ignore
use ds_settings::{AppName, ConfigRoot, FileName, Format, SettingsDoc, Store};

impl SettingsDoc for YourFile {
    const FILE: FileName = FileName("your-file.toml");
    const FORMAT: Format = Format::Toml;
}
let store = Store::new(ConfigRoot::Xdg, AppName("your-app-id"));
let loaded = store.load::<YourFile>();   // lenient: a bad key costs only itself; never fails
store.save(&loaded.value)?;              // atomic temp-and-rename; drops unknown keys
let mut watch = store.watch::<YourFile>(&spawner);   // 30 ms debounce, a task on `spawner`
```

`Loaded { value, unknown, invalid }` carries what the file held that did not become part of the
value; `loaded.diagnostics(YourFile::FILE)` is one printable line per entry. A key nobody reads
is reported and not preserved. `watch.changed().await` yields the whole re-read file as a
`Loaded`, and keeps the last good value when the file is not valid text. Tests use
`ConfigRoot::Scratch(dir)`, never real XDG.

**Settings derive, three more shapes.** Text is recognised by type (`String`, `PathBuf`,
`Cow<str>`) or by `#[settings(text)]` on a newtype. A one-variant enum is a key
(`KeyKind::Fixed`, drawn as a read-only `Widget::Readout`). A number without
`range = "min..=max"` is a compile error that starts `MissingRange { field: <name> }`; a type
the derive cannot place at all gets an error saying what to add.

## 11. Scrolling

One engine owns scroll physics for every Blitz document (design/11-BEHAVIOUR-scroll.md section
11.3.1): `blitz_kit::scroll`, which shell-host (a shell's surfaces) and `ds-blitz` (an app's
windows, `ds_blitz::launch`) both run. It does not let Blitz's default scroll act (Blitz jumps 20 px
a line), it writes raw offsets itself every frame, and a wheel detent is 60 px eased over at most
200 ms, detents accumulating; a touchpad flick glides and may stretch past an edge where a
container opts in; arrows scroll 40 px, Page Up/Down and Space a page (`max(0.8 v, v - 40)`),
Home/End jump to an edge. Everything below is quire's side of that split (design/11 section 11.7).

**Native overflow needs nothing from you.** A container with `overflow: auto` or `scroll` scrolls
by all of the above in a `ds_blitz` window the moment it has content to scroll, under the wheel,
a touchpad and the keys (a scroll key acts on the focused container, else on the one under the
pointer, else on the page; it leaves a key to the document when an element on the focus path has
its own `onkeydown`, is a text field, or is marked `data-keys="capture"`). Do not call Blitz's own
scroll (`MountedData::scroll_to`, `scroll-behavior: smooth`).

**Programmatic scrolls go through the engine.** `ds_blitz::use_scroll_handle()` gives the window's
`ScrollHandle` (`None` outside a window), and `handle.send(ScrollCmd::To { element, axis, offset,
animate })`, `ScrollCmd::By { element, axis, delta, animate }` or `ScrollCmd::IntoView { element,
animate }` runs at the next frame; `element` is a `ds_blitz::ElementId` naming the container by its
`id` attribute, `animate` is `ScrollAnimate::Smooth` (the 200 ms ease) or `Instant`. The same
`ScrollCmd` is shell-host's `SurfaceHandle::scroll`. `ScrollerRef::scroll_to` (the `Scroller`
component's own offset) is a separate thing and writes at once.

**A component that moves its own content from the wheel listens to gestures, and may ask for the
ease.** `use_gestures` still hears `Gesture::Scroll { by }` as the wheel turns, but `by` is now a
detent, 60 px, where it used to be a third of that, and it arrives whole. A viewer that moves its
own offset (a PDF or image canvas) opts into the engine's smooth step with
`ds::host::gesture::use_gestures_with(WheelDelivery::Eased, on_gesture)`: its wheel detents then
arrive as one `Gesture::Scroll { phase: Changed, by, .. }` per frame, `by` that frame's share of the
distance, summing to the detents' (60 px each, at most 200 ms, accumulating as in a native
container). Apply `by` as the `AsReceived` listener did; clamp as you did. Every gesture says
what turned it (`source`: `Wheel` or `Finger`). **A touchpad's run** is `Began`, `Changed`,
`Ended` as the fingers touch, move and lift: an `AsReceived` listener hears exactly that, the
fingers' own motion. An `Eased` listener hears the same run with the glide the engine gives a
native container: the fingers' motion as it comes, then, after a lift fast enough to glide (the
engine's own release speed, curve and `scroll.momentum` setting), one `Changed` per frame for the
momentum, and `Ended` when the glide is over, not at the lift; a touch during the glide ends it
at once (an `Ended`, then the new `Began`). A slow lift is an `Ended` at the lift, with no glide.
Stretching past an edge is the listener's: the host does not know its bounds, so clamp or stretch
the offset yourself, and drop the rest of a glide by ignoring `Changed` once you are at an edge.
**Under Control a wheel is a zoom, not a scroll**: every listener, eased or not, hears each click
whole (`by` is one detent, 60 px, `phase: Changed`, `held` has Control), one gesture per
click, with nothing eased.

**Every scroll container hides Blitz's own scrollbar.** Every ds component whose content scrolls
(`Menu`, `Panel`, `Sheet`, `EditSurface`) carries `scrollbar-width: none` in its own
CSS. Blitz honours this at the DOM level (`blitz-dom`'s `Node::wants_scrollbar` returns `false`
immediately when the computed `scrollbar-width` is `none`, before it even asks whether the content
overflows) — the host's thumb is the only one that ever paints. Give your own scroll containers
the same rule; `Rule::BlitzUnsupported`/self_lint do not check for its absence (there is nothing to
flag: an *absent* `scrollbar-width: none` is not, by itself, an offence), so this is a convention
to follow, not a lint you can lean on.

**`data-wheel="capture"` marks a component that consumes wheel input itself.** A ds component
that reads the wheel directly — today, `Slider` and the scrubber (design/04-COMPONENTS.md section 5; a
zoomable canvas would be the other kind design/11 names) — carries this attribute so the window
hands it the raw `BlitzWheelEvent` through `handle_ui_event` instead of scrolling whatever
contains it, and the engine does nothing for that wheel; the component's own handler must call
`prevent_default`, or Blitz's own scroll (20 px a line, in one jump) still acts (design/11
section 11.3.1 item 2). An element with a plain `onwheel` and no marker never hears the wheel in a
`ds_blitz` window: the engine takes it first, so mark it, or listen with `use_gestures`. It is a marker only: quire does not itself drive a wheel-triggered value
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
`ds_lint::Rule::BlitzUnsupported` flags it (`crates/ds-lint/src/blitz.rs`) because Blitz's own
300 ms `ScrollTo` would fight the host's engine. Drive a programmatic scroll through the host's
`ScrollCmd` instead (design/11 sections 11.3.1 and 11.3.11); `scroll-behavior: auto` (the
default) lints clean.

**`--scroll-thumb`**: the colour the host paints its overlay scrollbar thumb in
(design/11 section 11.3.12) — `--ink` at .5 alpha over the layer's own .8 opacity (effective .4)
in light, `--paper`'s equivalent (white at the same effective alpha) in dark. It is a token like
any other (`ds::style::tokens::colour::ColourToken::ScrollThumb`, registered in the lint's known-variable table the same
way every other colour is); quire's stylesheet declares it, the host reads it, and no component
CSS references it directly, since no ds component paints the thumb itself.

## 12. User styles

A person restyles your app by editing one CSS file, `~/.config/<app>/style.css`, and the change
shows as they save it (ARCHITECTURE.md section 10). quire owns the mechanism; you wire two things.

**Load and watch it into `Ds`.** `ds_settings::UserStyle` is a `SettingsDoc` for `style.css`: a
missing file is an empty style and no text is refused at load. `Ds` takes it as
`user_style: ReadSignal<UserStyle>` and renders it in a `<style data-ds-user>` after the design
system's own sheet. The design system is `@layer ds` and the person's text is unlayered, so a
person's rule wins by the cascade whatever its specificity (`.ds-button { ... }` beats quire's
`.ds-button[data-variant=push]`) and never needs `!important`:

```rust,ignore
use ds_settings::{AppName, ConfigRoot, Store, UserStyle};

let store = Store::new(ConfigRoot::Xdg, AppName("your-app-id"));
let mut user_style = use_signal({
    let store = store.clone();
    move || store.load::<UserStyle>().value
});
use_future(move || {
    let store = store.clone();
    async move {
        let spawner = ds_blitz::TokioSpawner::current();
        let mut watch = store.watch::<UserStyle>(&spawner);
        while let Some(loaded) = watch.changed().await {
            user_style.set(loaded.value);
        }
    }
});
rsx! {
    Ds { appearance: Appearance::default(), material: Material::Window, user_style, YourPage {} }
}
```

**Publish what a person may rely on.** `ds::selectors` is the table of the selector surface a
user stylesheet may use: `[data-surface=<name>]` (give a surface root `Ds { surface: Some("bar") }`),
`.ds-<component>` on every component root, each component's listed parts, `data-variant`,
`data-size`, `data-state`, `aria-*` and every token variable. Renaming anything on it is a
breaking change for the person's file; everything off it may change freely. Token overrides are
the recommended form (`.ds { --accent: ...; }` restyles every surface consistently).

**Put your own sheets in the `app` layer.** Your own stylesheets are unlayered unless you say
otherwise, and an unlayered rule beats every `@layer ds` rule, so a `.card { ... }` of yours would
beat quire's more specific rule where before it lost. Draw each one with `ds::prelude::AppStyle {
css: MY_CSS }` instead of `style { {MY_CSS} }`: it wraps the text in `@layer app`, after `ds` and
before the person's unlayered rules, so your rules and quire's keep the order and specificity they
always had, and the person's still beat both (`ds_style::css::layers` holds the names). Never write
`@layer ds` in a sheet of your own (`Rule::LayerDs`).

**Report, never block.** `ds_lint::user_stylesheet(css, &ds::kits())` returns notes (unknown
variables, selectors outside the public surface, `!important`, `url()` that is not a local `file:` or `data:`
URL, parse errors with line numbers). A settings page or a `--check-style` command can print
them; loading never waits on them. User CSS is exempt from the design-system lint rules.

## 13. Portable core, desktop extras

Your app is complete without our desktop; on it, the app does more (design/36-PORTABLE-CORE.md
is the rule and the table of who is what). Three things follow for a consumer.

- **Feature.** Put desktop integration in a `desktop/` module behind a `quire-desktop` feature,
  default on for Linux (`[target.'cfg(target_os = "linux")'.dependencies]` or a `default`
  entry your packaging sets). Core modules never import `crate::desktop` or zbus, and
  `cargo check --workspace --no-default-features` must pass. Run
  `quire/scripts/check-portable.sh <your-workspace-dir>` from your lane gate to check both.
- **Probe.** `ds-desktop` (features `dbus`, `dioxus`) answers `Capability::{Intents, Accounts,
  Memory, Companion, Appearance, Materials, Peek, Share, Helpers}` as `Presence::{Here,
  Absent}`. `Desktop::probe().await` once, `Desktop::watch()` to follow services starting and
  stopping, or `use_desktop()` in a component for a signal. Without `dbus` everything is `Absent`.
- **Gate the entry point, not the behaviour.** An extra's menu item, button or settings pane is
  rendered only under `Here`; there is no disabled twin.

```text
let desktop = use_desktop();
rsx! {
    Menu {
        MenuItem { label: "Open", on_select: open }
        if desktop.read().here(Capability::Companion) {
            MenuItem { label: "Ask the companion", on_select: ask }
        }
    }
}
```
