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
| `tint_alpha` | `Option<Alpha>` | `None` (the tint's default alpha) | the materials' tint alpha over compositor blur (design/22-SETTINGS.md §3.1 `appearance.material_tint_alpha`); pass `ds_settings::Environment::tint_alpha()` (thousandths: `Alpha(800)` is 80%) once you are reading a live `Environment` (section 3) rather than leaving it at the default |

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
system: SystemPrefs }`: it loads `appearance.toml` (importing mailo's `appearance.json` once,
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
zero offences, so a stale one cannot hide silently. Prefer `Profile::Strict` even though
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
        onclick: move |()| send(),
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
        snippet: Some("I have translated the memoir...".to_owned()),
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
  what every other field wants.
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
