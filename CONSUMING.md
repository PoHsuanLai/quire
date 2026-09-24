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

Every quire component must be drawn inside one `Ds` (`root/ds.rs`; design/03-COLOR.md
section 17.1; `crates/ds/src/root/ds.rs`). It resolves your appearance to a scheme, an accent
and a motion level; stamps `data-theme`, `data-accent`, `data-motion`, `data-material`,
`data-blur`, `data-modality` and `data-hover` on its own `div.ds`; injects the stylesheet
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
| `scale` | `Option<Scale>` | `None`: the host's `ds::HostScale`, else 1x | the device scale this root draws for, in 120ths (`Scale(180)` is 1.5x, the `wp_fractional_scale_v1` unit and shell-host's `Scale`); the root writes the pixel tokens for it (below). `ds_native::launch`, `Harness` and `snapshot` provide `HostScale` themselves; a host that is not `ds-native` passes `scale` |

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
re-stamps `data-theme`/`data-accent`/`data-motion`/`data-material`/`data-blur` and updates the
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
  `IconView { plate: Some(PlateFamily::Blue), .. }` for a plate with depth.

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
