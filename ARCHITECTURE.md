# Architecture

quire is the design system of the desktop shell program: tokens, palette, icons, components,
motion, the Blitz glue that draws them, and the tools that test them. `design/` says what
everything looks like, `DESIGN.md` maps each design section to its module, `CONSUMING.md` is the
guide for a downstream crate, `CONVENTIONS.md` holds the rules every repo shares. This file is the
map: which crate owns what, what may depend on what, where each concept lives, the exact shape of
each trait, and how to add each kind of thing. Where it and `CONVENTIONS.md` disagree, this file
wins for quire.

Reading order for a contributor: section 1 (find the crate), section 4 (find the home), section 5
(find the trait), section 7 (copy the recipe).

## 1. Crates and allowed edges

Workspace crates (`crates/<name>`), plus one sibling repo (`blitz-kit`) and one dev tool.

| Crate | Purpose |
| --- | --- |
| `ds-core-derive` | proc macros: `#[derive(Word)]`, `#[derive(Token)]` |
| `ds-core` | pure base: vocabulary, geometry units, colour, time, the `Spawner` trait, errors, text clip, PNG/base64, the `Word` trait |
| `ds-style` | appearance, tokens and the `Token` trait, material, Space palettes, fonts (bytes), icons, CSS emission, the `Kit` seam |
| `ds-motion` | `Anim` and recipes, keyframes, `Presence`, `Timeline`, rosters, gestures, pulse, the details grammar |
| `ds-lint` | stylesheet and markup linter; reads its vocabulary from `Kits` |
| `ds` | generic components, the `DocumentHost` seam and its hooks, overlay stack, root, `Ds`, stylesheet assembly, `ds::prelude`, `ds::testing` |
| `ds-shell` | shell-only components, shell tokens, widget contract, registry and catalog, the widgets |
| `ds-settings-derive` | proc macro: `#[derive(SettingsSchema)]` |
| `ds-settings` | `SettingsDoc` + `Store`: lenient load, atomic save, watch, schema, appearance file, portal, icon-theme lookup |
| `ds-blitz` | `DocumentHost` on Blitz, app window host, launch, clipboard; features `pdf`, `print`, `spell` |
| `ds-harness` | dev-only test driver: `Driver`, `DocQuery`, `Harness`, snapshots, painters |
| `ds-conformance` | test-only crate: every component's behaviour tests, one file per component |
| `ds-gallery` | the visual reference binary: every component across theme, accent, motion, material |
| `tools/icons` | app-icon post-process (binary) |
| `anyrender_pdfrum` | anyrender scenes as vector PDF; leaves for the pdfrum repo as `pdfrum-anyrender` |
| `blitz-kit` (own repo) | portable Blitz repairs shared with shell-host: hover sync, net provider, fonts, adapter, hit test, pixel snap; names no `ds*` crate |

### Allowed edges (workspace crates; everything else is forbidden)

| Crate | May depend on |
| --- | --- |
| `ds-core-derive`, `ds-settings-derive` | nothing of ours |
| `ds-core` | `ds-core-derive` |
| `ds-style` | `ds-core` |
| `ds-motion` | `ds-style`, `ds-core` |
| `ds-lint` | `ds-style`, `ds-core` |
| `ds` | `ds-motion`, `ds-style`, `ds-core` |
| `ds-shell` | `ds`, `ds-motion`, `ds-style`, `ds-core` |
| `ds-settings` | `ds-style` (system prefs, appearance enums), `ds-core`, `ds-settings-derive` |
| `ds-blitz` | `ds`, `ds-style`, `ds-core`, `blitz-kit`; feature `pdf`: `anyrender_pdfrum` |
| `ds-harness` | `ds-blitz`, `ds-core`, `blitz-kit` |
| `ds-conformance` | dev-dependencies only: `ds`, `ds-shell`, `ds-lint`, `ds-settings`, `ds-blitz`, `ds-harness` |
| `ds-gallery` | `ds`, `ds-shell`, `ds-lint`, `ds-settings`, `ds-blitz`, `ds-harness` (snapshots) |
| `tools/icons` | `ds-style`, `ds-core`, `ds-settings` |
| `anyrender_pdfrum` | none of ours; no Blitz crate |

Dev-dependencies follow the same table, plus: every crate may dev-depend on `ds` with feature
`testing`; `ds`, `ds-shell` and `ds-style` may dev-depend on `ds-lint`. Consumers (sill, mailo,
`examples/consumer`) depend on `ds` (+ `ds-shell`, `ds-settings`, `ds-blitz`, and `ds-lint`,
`ds-harness` as dev-dependencies); they never name `ds-core`, `ds-style` or `ds-motion` because
`ds` re-exports what they need (section 6).

### External boundaries (`scripts/check-boundary.sh`)

| Crate | Never reaches |
| --- | --- |
| `ds-core`, `ds-style`, `ds-motion`, `ds-lint`, `ds`, `ds-shell` | `zbus`, `notify`, `tokio`, `winit`, every `blitz*`, `stylo_taffy`, `dioxus-native*`, every `anyrender*`, `pdfrum*`, `arboard` |
| `ds-core`, `ds-lint`, `ds-core-derive`, `ds-settings-derive` | `dioxus` (`ds-core` is plain data and maths, so sill's pure crates can use it; the linter reads strings; the derives generate paths) |
| `ds-settings` | every `blitz*`, `dioxus-native*`, `anyrender*`; `tokio` (it takes a `Spawner`); `dioxus` unless feature `dioxus` |
| `ds-blitz` | `zbus`, `memfd` unless feature `print`; `pdfrum*` unless `pdf` |
| `anyrender_pdfrum` | `blitz*`, `parley`, `stylo_taffy`, `dioxus*` |
| `blitz-kit` | every `ds*` crate, `dioxus*` |

`tokio` is named only by `ds-blitz` (the `launch` entry point owns the one runtime and hands a
`Spawner` to everything below) and by dev-dependencies of tests that need a runtime.

## 2. Modules inside each crate, in layer order

A module names only modules above it in its crate's list (earlier rows are lower). `lib.rs` holds
declarations and re-exports. Directories group a concept; role files follow `CONVENTIONS.md` §2
(`model.rs`, `step.rs`, `view.rs`, `style.css`, `tests.rs`).

| Crate | Modules, lowest first |
| --- | --- |
| `ds-core` | `word`, `vocab` (Availability, Selection, Emphasis, Switch, Expanded, Check, Shown, Fraction, Percent, StaggerIndex, ShortcutKey, Shortcut), `press`, `standard_action`, `geometry` (units, scale, placement), `colour` (srgb, oklab, fit, contrast), `time` (clock, virtual clock), `spawner` (the `Spawner` trait), `error`, `text` (clip), `codec` (png, base64) |
| `ds-style` | `look` (`Look`), `appearance` (theme, accent, motion, blur, material choice, peek, system prefs, resolve, typeface), `scope` (the enclosing `Scope`), `tokens` (one file per token family, `set.rs` = `TokenSet`, `tuned.rs`), `kit` (`Kit`, `Kits`, `Section`, `Vocabulary`), `material`, `space`, `icon` (glyph tables by family, plate, classify, render, url), `fonts`, `scale`, `css` (emission per cascade section, `reset.css`, `utilities.css`), `emit` |
| `ds-motion` | `anim` (`Anim`), `recipe` (the table), `keyframes` (generated CSS + `motion.css`), `settle` (settle, timers, wake, reduced), `presence`, `timeline` (`Timeline` + `use_timeline` + one file per implementor), `roster`, `pulse`, `gesture` (drag, swipe, velocity, hover intent), `details` (grammar, `Moment`, `Detailed`, cues, one-shots, glyph morphs) |
| `ds-lint` | `rule` (`Rule`, `Severity`, `Profile`, `Exception`), `tokenize`, `walk`, `stylesheet` rules, `markup` rules, `hig`, `details`, `assert` |
| `ds` | `host` (the seam traits: `DocumentHost`, `FocusHost`, `CaretHost`, `GeometryHost`, `ClickFocusHost`, `EditHost`, `ImeHost`, `FileDropHost`, `NoHost`, `HostSignals`) < `focus`, `edit`, `file_drop`, `spell`, `window` (hooks and pure logic over the seams) < `stack` (overlay stack, hover hub, toast hub, menu tracker, pull tab) < `root` (`Surface`, chrome, extent, typeface) < components `content` < `controls` < `overlays` < `lists` < `fields` < `menus` < `editor` < `chrome` < `app` (mail-only, named by nothing but `assembly`) < `assembly` (`Ds`, stylesheet, sheet registration) < `prelude`, `testing` |
| `ds-shell` | `tokens` < leaf parts `battery`, `clock`, `emoji`, `month_grid`, `now_playing`, `notifications`, `osd`, `idle_dim`, `dock_parts`, `space_editor`, `bar`, `control_center`, `switcher` < `user_picture`, `thumbs` < `lock` < `widgets` (`contract`, `registry`, `catalog`, `wire`, `card`, `frame`, `gallery`, and one directory per widget kind) < `prelude` |
| `ds-settings` | `error` < `root` (`ConfigRoot`, `AppName`) < `lenient` < `doc` (`SettingsDoc`, `Format`, `FileName`) < `store` < `watch` < `schema` < `appearance` (file, settings structs) < `portal` < `environment` (feature `dioxus`) < `icon_assets` < `units` |
| `ds-blitz` | `error`, `contexts`, `setup`, `document` (node ref, origin, wake) < `focus`, `measure`, `reveal`, `edit` (+ ime, tree, geometry), `file_drop`, `clipboard`, `frames` < `host` (`BlitzHost`, `provide_host`) < `window` (build, place, requests, hover, drop, shell) < `launch`, `open_window` < features `pdf`, `print`, `spell` |
| `ds-harness` | `input` (`Input` and its parts) < `driver` (`Driver`, `DocQuery`, `Query`) < `headless` (document, painter, backend, gpu paint, clock, settle) < `harness` (`Harness`) < `snapshot` |
| `ds-conformance` | `tests/<component>.rs`, `tests/support/` |
| `ds-gallery` | `axes`, `args`, `page`, `registry`, `pages/<group>/<component>.rs`, `sheet`, `snapshot`, `app` |

Component groups inside `ds` and the layer order inside the components tier: `content` (icons,
images, avatars, status glyphs, rich text, marks, PDF thumbs) < `controls` < `overlays` (alert,
popover, sheet, tooltip, toast, scrim, hover card, drag ghost, panel, flow) < `lists` (rows,
settings rows, headers, animated and leaving lists, preview pane, emoji grid, appearance picker)
< `fields` (text input, search field, selection bubble) < `menus` (menu, palette, entries) <
`editor` (`EditSurface` and its spell menu) < `chrome` (window frame, traffic lights) < `app`.

## 3. From -> to (mechanical move)

Paths are `crates/<crate>/src/...`. `ds/src/x` is today's location; the target is `crate::module`.
A name in `{...}` is a set of files. Anything not listed keeps its file name.

### `ds` -> `ds-core`

| From | To |
| --- | --- |
| `ds/core/vocab.rs` | `ds-core::vocab`; `Key` -> `ShortcutKey` |
| `ds/components/overlays/tooltip.rs::Shown` | `ds-core::vocab::Shown` |
| `ds/core/{press,standard_action}.rs` | `ds-core::{press, standard_action}` |
| `ds/core/geometry/{units,scale,placement}.rs` | `ds-core::geometry::{units, scale, placement}` |
| `ds/core/colour/*` | `ds-core::colour::*` (the one copy) |
| `ds/core/time/*` | `ds-core::time::*`; the private `Timeline` struct in `clock/timeline.rs` becomes `VirtualQueue` |
| `ds/core/error.rs`, `ds/core/text/clip.rs`, the `Spawner` half of `ds/core/task.rs` | `ds-core::{error, text::clip, spawner}` |
| `ds/core/{task,busy,guarded}.rs` (the Dioxus halves: scope-owned tasks, busy and guarded hooks) | `ds-style::task` (the lowest crate that uses Dioxus) |
| the Dioxus event conversion in `ds/core/press.rs` | `ds::controls::press` (`ds-core::press` keeps the plain `Press`/`PointerButton` data) |
| `ds/core/{png,base64}.rs` | `ds-core::codec::{png, base64}` |
| new | `ds-core::word` (`Word`), `ds-core::spawner::Spawner` |

### `ds` -> `ds-style`

| From | To |
| --- | --- |
| `ds/style/appearance/*`, `ds/root/typeface.rs` (the `Typeface` choice) | `ds-style::appearance::*` |
| `ds/style/env.rs` | `ds-style::scope` (`Env` -> `Scope`, `use_env` -> `use_scope`) |
| `ds/style/tokens/*` except the files listed under `ds-shell` | `ds-style::tokens::*` (`status.rs`, `emoji_face.rs`, `person.rs`, `plate.rs`, `size_*.rs`, `control_size.rs` stay: their consumers are generic) |
| `ds/style/{material,space,icon,fonts.rs,scale.rs,css,emit.rs}` | same names in `ds-style` |
| `ds/style/icon/geometry_shell.rs` | `ds-style::icon::glyphs::shell` (`Icon` is one closed enum owned here) |
| `ds/assembly/stylesheet.rs` (the section list) | `ds-style::kit` (`Kits::stylesheet`); the ordered call site stays in `ds::assembly` |

### `ds` -> `ds-motion`

| From | To |
| --- | --- |
| `motion/{anim,recipe,recipe_own,recipe_detail}.rs` | `ds-motion::{anim, recipe::{table, own, details}}` |
| `motion/{css.rs,motion.css}` | `ds-motion::keyframes` |
| `motion/{settle,timer,wake,reduced,entrance,pane_slide,projection}.rs`, `motion/detail/{settle,use_settle}.rs` | `ds-motion::settle::*` |
| `motion/presence.rs` (`Presence`, `ListPresence`, `Exit`), `components/overlays/{osd_phase,shown_phase,spring_presence}.rs` | `ds-motion::presence` (one enum, one hook; `Healing` becomes `roster::Heal`) |
| `motion/{level_run,use_level_run,spring,spring_spec,spring_point,use_spring}.rs`, `motion/detail/{tween,sweep,glide,count_up,pending,use_pending,level,motor,reveal}.rs` | `ds-motion::timeline::{ease, spring, sweep, glide, count_up, pending}` + `use_timeline` |
| `motion/{roster,roster_exits,roster_rest,use_roster,batch_roster}.rs` | `ds-motion::roster::*` |
| `motion/{pulse,pulse_key}.rs` | `ds-motion::pulse` |
| `motion/{drag,drag_return,swipe,use_swipe,velocity,hover_intent}.rs` | `ds-motion::gesture::*` |
| `motion/detail/*` not listed above (`grammar, moment, detailed, use_detail, cue, once, armed, stamp, touch, operation, use_operation, first_show, check_mark, layer_glyph, morph, morph_glyph, roll_digits, layer.css, morph.css, reveal.css, roll.css`) | `ds-motion::details::*` |
| `motion/detail/idle_dim.rs` | `ds-shell::idle_dim::drive` (a shell component's driver) |

### `ds` -> `ds-lint`, `ds`, `ds-shell`

| From | To |
| --- | --- |
| `ds/lint/*` | `ds-lint::*`; `registry.rs` reads `Kits::vocabulary()`, its `crate::motion`/`crate::style::tokens::*` imports are gone |
| `host/*`, `focus/{host,caret,click,select,hand_back,selector}.rs`, `edit/host.rs`, `file_drop/host.rs`, `spell/host.rs`, `window/host.rs` | `ds::host::*` (the `Host*` newtypes are deleted) |
| `focus/*`, `edit/*`, `file_drop/*`, `spell/*`, `window/*` (the rest) | same module names in `ds` |
| `overlay/*` | `ds::stack::*` (the module is `stack` so `overlays` names only components) |
| `root/{surface,chrome,extent,typeface}.rs` | `ds::root::*` |
| `components/{content,controls,overlays,lists,fields,menus,chrome,app}` | `ds::{...}` at the crate top level; `components/mod.rs` is gone |
| `components/content/{emoji_grid,emoji_grid_nav}.rs`, `components/content/{preview_pane,preview_cue,preview_content,pane_switcher}.rs` | `ds::lists::{emoji_grid, preview}` |
| `components/controls/level/{glyph,vocab}.rs` | `ds::content::level_glyph` (a status glyph draws it) |
| `components/controls/appearance_picker.rs` | `ds::lists::appearance_picker` |
| `components/fields/edit_surface*.rs` | `ds::editor::*` |
| `components/content/shot_frame.rs` | ratio and mat -> `ds-shell::thumbs::shot_frame`; `picture_style` -> `ds::content::picture_fit` |
| `components/content/status/*`, `status_glyph.rs` | stay: `ds::content::status` |
| `components/lists/row_battery.rs` | `ds-shell::battery` (composes `BatteryGlyph`) |
| `assembly/{ds,sheets,stylesheet}.rs` | `ds::assembly::{ds, sheets, stylesheet}`; `ds.rs` loses its `WindowFrame`/`ToastHost` imports by naming them from `chrome`/`overlays` (assembly is above both) |
| `shell/{bar,battery,clock,control_center,emoji,lock,month_grid*,notifications,now_playing*,osd*,space_editor*,switcher,thumbs,user_picture*,dock_parts*,idle_dim*}` | `ds-shell::` same names |
| `shell/widget/{contract,registry,scope,slot,layout,kind,exit,wire,card,frame,gallery*,use_widget,timeline}.rs`, `shell/catalog/*` | `ds-shell::widgets::{contract, registry, scope, slot, layout, kind, exit, wire, card, frame, gallery, use_widget, timeline, catalog}` |
| `shell/widget/{battery,calendar,clock}.rs` | `ds-shell::widgets::{battery, calendar, clock}/` (one directory each) |
| `style/tokens/{control_center,dock,notifications,osd,shell,shell_scale,widgets,widget_paint}.rs` | `ds-shell::tokens::*`, each implementing `Token` and listed in `ds_shell::KIT` |
| `lib.rs` root re-exports | deleted; replaced by `ds::prelude` and each crate's curated root |
| `detail.rs`, `icon.rs`, `time.rs`, `widget.rs`, `catalog.rs` (public shim modules) | deleted; their items are reached by the owning crate's root |

### `ds-settings`, `ds-native`, tests

| From | To |
| --- | --- |
| `ds-settings/{file,dirs,lenient,watch}.rs` | `ds-settings::{doc, root, lenient, store, watch}` |
| `ds-settings/{appearance_file,settings}.rs` | `ds-settings::appearance::{file, settings}` |
| `ds-settings/{diff,test_dir}.rs`, `schema/foreign.rs`, `SchemaVariants` | deleted (`SettingsChange`; `Word` gives every enum its variants) |
| `ds-settings/{portal,environment,icon_assets,units,error}.rs`, `schema/*` | same module names |
| `ds-native/{focus,focus_chain,focus_keep,click_focus,measure,reveal,edit*,drop_hit,clipboard,frame_*,frames,frame*,node_ref,origin,route,contexts,setup,install,host,app_id,error,scheme,wake}.rs` | `ds-blitz::{focus, measure, reveal, edit, file_drop, clipboard, frames, document, host, ...}` |
| `ds-native/{window,window_*,launch,open_window,native_providers,runtime}.rs` | `ds-blitz::{window, launch, open_window}`; `runtime.rs` becomes the `Spawner` `launch` provides |
| `ds-native/{pdf,print,spell}/`, `pdf.rs`, `print.rs`, `pdf_thumb/` | `ds-blitz::{pdf, print, spell}` under their features (`pdf-thumb` folds into `pdf`, `spellcheck` into `spell`) |
| `ds-native/{hover_sync,hover_replay,net,net_policy,data_url,fonts,gpu_adapter,snap}.rs` and the Blitz-only halves of `harness_hit.rs`, `harness_style.rs` (`part_rect`, `painted_rect`) | `blitz-kit::{hover, net, data_url, fonts, adapter, snap, hit}` (the parts naming a `ds` type stay in `ds-blitz`) |
| `ds-native/{harness*,headless,snapshot,painter,gpu_paint,memory_shell}.rs`, `tests/support/*` | `ds-harness::*`; `memory_shell.rs` becomes `ds-blitz::clipboard::Memory` |
| `ds-native/tests/*` naming a component (`alert`, `bar_menu`, `banner_*`, `cc_*`, `details_*`, `mailo*`, `launcher*`, `palette*`, `month_grid*`, `notification_*`, `widget_*`, `status_*`, `send_pill_moods`, ...) | `ds-conformance/tests/<component>.rs`, merged per component; a test named for work (`mailo4_*`, `*_gaps`, `*_fixes`, `polish`, `macos_polish`, `launcher_v2`) is renamed for the behaviour it checks |
| `ds-native/tests/*` naming a host behaviour (`native_*`, `hover_sync`, `file_drop`, `spell_*`, `pdf*`, `virtual_clock*`, `snapshot`, `harness*`, `hybrid_backend`, `pixel_snap`, `text_stack_fonts`, `cancelled_animation`) | `ds-blitz/tests/<topic>.rs`, or `ds-harness/tests/<topic>.rs` for driver and painter behaviour |
| `ds/tests/*_ssr.rs`, `ds/tests/{lint_*,self_lint,hig_lint,...}.rs`, `snapshots/` | beside the crate that owns the component (`ds`, `ds-shell`, `ds-lint`); `mailo_gaps*` files renamed by component |
| `ds-native/examples/*` | `ds-blitz/examples/*` (pdf, print, window, edit, file_drop, second_window); `sizing_audit` -> `ds-gallery` |
| `ds-gallery/src/pages/*_mailo*.rs`, `*_gaps.rs`, `polish*.rs` | `pages/<group>/<component>.rs` |

## 4. One home per concept

The single place a concept lives. Extend it; never write a second one.

| Concept | Home |
| --- | --- |
| Closed vocabulary (slug, label, ALL, parse) | `ds-core::word::Word` + `#[derive(Word)]` |
| Colour maths (sRGB, linear, OKLab, OKLCH, contrast, gamut fit) | `ds-core::colour` |
| Pixel units, points, rects, device scale | `ds-core::geometry` |
| Time, sleep, virtual clock | `ds-core::time` (`now`, `since`, `sleep`) |
| Scope-owned tasks, spawning | `ds-style::task` (`spawn_in`); the `Spawner` trait in `ds-core::spawner` |
| Base vocabulary (`Availability`, `Switch`, `Shown`, `Fraction`) | `ds-core::vocab` |
| PNG, base64 | `ds-core::codec` |
| The crate error | `ds-core::error::DsError`; settings: `ds-settings::error::SettingsError` |
| Design tokens (colour, duration, easing, spacing, shape, type, layer) | `ds-style::tokens` (one file per family, `impl Token`) |
| Shell metric tokens (dock, OSD, notifications, control center, widgets) | `ds-shell::tokens` |
| What each crate adds to the stylesheet, and the lint vocabulary | `ds-style::kit::Kit`; ordered by `Kits` |
| Appearance choice and resolution (theme, accent, motion, system prefs) | `ds-style::appearance` |
| The enclosing scope a component reads | `ds-style::scope::Scope` |
| Material, blur, frame ground | `ds-style::material`; `ds::root::chrome` |
| Space palettes and `SpaceLook` | `ds-style::space` |
| Looks (Mac, Arc: values only) | `ds-style::look::Look`; token values per Look in `ds-style::tokens` |
| Glyphs (the `Icon` enum), plate, retint, classify | `ds-style::icon` |
| Font faces as bytes | `ds-style::fonts`; registration with the renderer: `blitz-kit::fonts` |
| Keyframes and recipes | `ds-motion::{anim, recipe, keyframes}` |
| Show/hide (entering, present, leaving) | `ds-motion::presence::{Presence, use_presence}` |
| Tweens, springs, count-up, sweeps | `ds-motion::timeline::{Timeline, use_timeline}` |
| List row motion (enter, exit, heal) | `ds-motion::roster` |
| Pulse (a replayed keyframe) | `ds-motion::pulse` |
| Drag, swipe, hover intent | `ds-motion::gesture` |
| What a state change means (moments) | `ds-motion::details::Detailed` |
| Stylesheet and markup linting | `ds-lint` |
| The document seam (focus, caret, geometry, edit, drop) | `ds::host::DocumentHost` |
| Focus hooks, focus requests | `ds::focus` |
| Text editing surface | `ds::editor::EditSurface`; host half `ds-blitz::edit` |
| File drop | `ds::file_drop`; host half `ds-blitz::file_drop` |
| Spellcheck | `ds::spell::SpellService`; implementation `ds-blitz::spell` |
| Window frame, traffic lights, window host | `ds::chrome`; `ds::window::HostWindow`; implementation `ds-blitz::window` |
| Overlay stack, hover hub, toast hub | `ds::stack` |
| Menu data | `ds::menus::{MenuEntry, MenuKind, MenuRow}` |
| Menu pointer tracking | `ds::stack::menu_track` |
| Popup menu view | `ds::menus::Menu` |
| Toasts | `ds::overlays::toast` + `ds::stack::toast_hub` |
| The root component and stylesheet assembly | `ds::assembly::{Ds, stylesheet}` |
| A status glyph and its state (Wi-Fi, battery, Bluetooth, volume) | `ds::content::status` |
| The battery drawing | `ds::content::status::battery::BatteryGlyph`; device models in `ds-shell::battery` |
| Widget contract, registry, placement data | `ds-shell::widgets::{contract, registry, catalog}` |
| Emoji: the grid / the picker sheet | `ds::lists::emoji_grid` / `ds-shell::emoji` |
| Settings file I/O (lenient load, atomic save, watch) | `ds-settings::store::Store` |
| Settings schema | `ds-settings::schema` (derived) |
| Desktop preferences (portal) | `ds-settings::portal::SystemPrefsSource` |
| Test driver, document queries | `ds-harness::{Driver, DocQuery, Query}` |
| SSR rendering and golden files in tests | `ds::testing::{ssr, golden}` |
| Clipboard | `ds-blitz::clipboard::Clipboard` |
| Net policy, `data:` URLs | `blitz-kit::net`, `blitz-kit::data_url` |
| Hover sync, pixel snap, GPU adapter choice, transform-aware hit test | `blitz-kit::{hover, snap, adapter, hit}` |
| PDF output, printing | `ds-blitz::{pdf, print}` |
| Mail-only components | `ds::app` |

## 5. Traits and closed enums

### Canonical traits

| Trait | Crate | Implementors | Add an implementor when | Add a trait when |
| --- | --- | --- | --- | --- |
| `Word` | `ds-core` | every closed vocabulary enum (derived) | a new closed set | never |
| `Token` | `ds-style` | each token family; shell families in `ds-shell` | a new family | never |
| `Timeline` | `ds-motion` | `Ease`, `Spring`, `Glide`, `Pending` | a Rust-driven animation | never |
| `Detailed` | `ds-motion` | 13 component state enums | a component with moments | never |
| `Widget` | `ds-shell` | battery, world clock, month, and each widget kind | a new widget kind | never |
| `DocumentHost` (+ parts) | `ds` | `ds_blitz::BlitzHost`, `ds::host::NoHost` | a new renderer | a new capability of the document a component needs |
| `SpellService`, `HostWindow` | `ds` | `ds_blitz::spell::Hunspell`, a test fake; `ds_blitz::window::WinitWindow`, sill's shell-host window, a test stub | a new platform | never |
| `SettingsDoc` | `ds-settings` | `AppearanceFile`; each consumer's file | a new settings file | never |
| `Spawner` | `ds-core` | `ds_blitz::TokioSpawner`, a test's inline spawner | a new runtime | never |
| `Clipboard` | `ds-blitz` | `System`, `Memory` | never | never |
| `Driver`, `DocQuery` | `ds-harness` | `Harness`; later shell-host's headless surface | a new driver | never |

Exact signatures (bodies omitted; `...` marks provided methods):

```rust
// ds-core::word
pub trait Word: Copy + Eq + std::hash::Hash + 'static {
    const ALL: &'static [Self];
    fn slug(self) -> &'static str;            // kebab-case attribute value
    fn label(self) -> &'static str;           // "Read only"
    fn parse(slug: &str) -> Option<Self> { ... }   // ALL.iter().find(slug)
}
// #[derive(Word)]: slug = kebab-case of the variant; #[word(slug = "x")] on a variant;
// #[word(case = snake)] on an enum that is stored and so matches its serde name;
// #[word(label = "x")]. ds_core::testing::word_matches_serde::<T>() checks the last.

// ds-style::tokens
pub trait Token: Word {
    const PREFIX: &'static str;               // "dur-", "s-", "shell-"
    const KIND: TokenKind;                    // Fixed | Tuned (default declared, override written inline)
    fn var(self) -> VarName { ... }           // "--" + PREFIX + slug
    fn css_value(self, scope: TokenScope) -> CssValue;   // TokenScope { look: Look, scheme: Scheme, motion: MotionLevel, typeface: Typeface }
}
pub struct TokenSet { ... }
impl TokenSet { pub const fn of<T: Token>() -> TokenSet; }
// #[derive(Word, Token)] is the only way to write a token family: the enum-level
// #[token(prefix = "dur-", kind = fixed | tuned)] and per-variant values
// #[token(value = "90ms")] or #[token(standard = "..", reduced = "..")]
// or #[token(light = "..", dark = "..")] generate `var`, `css_value` and the `TokenSet`, so a
// token cannot reach CSS without also reaching the linter's vocabulary.

// ds-style::kit
pub struct Section { pub name: &'static str, pub css: fn() -> std::borrow::Cow<'static, str> }
pub struct Vocabulary { pub keyframes: &'static [&'static str], pub inline_vars: &'static [&'static str],
                        pub grammar_durations: &'static [DurationToken], pub grammar_easings: &'static [EasingToken] }
pub struct Kit { pub rank: KitRank, pub tokens: &'static [TokenSet], pub sections: &'static [Section],
                 pub vocabulary: Vocabulary }
pub enum KitRank { Style, Motion, Components, Shell, User }  // cascade order; fixed by rank, not by argument order; User is last (section 11)
pub struct Kits { ... }
impl Kits { pub fn of(kits: &[&'static Kit]) -> Kits; pub fn stylesheet(&self) -> String;
            pub fn vocabulary(&self) -> Vocabulary; }
// Each crate exports `KIT`: ds_style::KIT, ds_motion::KIT, ds::KIT, ds_shell::KIT.
// ds::kits() = style + motion + ds; ds_shell::kits() = ds::kits() + shell.

// ds-motion::timeline
pub trait Timeline: Clone + PartialEq + 'static {
    type Frame: Clone + PartialEq + 'static;
    fn total(&self) -> std::time::Duration;
    fn at(&self, elapsed: std::time::Duration) -> Self::Frame;
    fn settled(&self, elapsed: std::time::Duration) -> bool { elapsed >= self.total() }
}
/// A changed `timeline` starts a run from now; frames tick until it settles; `Reduced` motion
/// returns `at(total)` at once. The only tween hook.
pub fn use_timeline<T: Timeline>(timeline: T) -> T::Frame;

// ds-motion::presence (an enum and one hook, not a trait)
pub enum Presence { Hidden, Entering, Present, Leaving(Exit) }
pub struct PresenceSpec { pub enter: Anim, pub exit: Exit }
pub fn use_presence(shown: Shown, spec: PresenceSpec, on_hidden: Option<EventHandler<()>>) -> Presence;

// ds-motion::details (kept)
pub trait Detailed: Clone + PartialEq + 'static {
    fn moment(from: &Self, to: &Self) -> Moment;
    fn first(state: &Self) -> Moment;
}

// ds-shell::widgets::contract (kept)
pub trait Widget: Clone + PartialEq + Default + 'static {
    type Entry: Clone + PartialEq + Serialize + DeserializeOwned + 'static;
    type Intent: Clone + PartialEq + Serialize + DeserializeOwned + 'static;
    fn kind() -> WidgetKind;
    fn name() -> TextLine;
    fn description() -> TextLine;
    fn sizes() -> &'static [WidgetSize];
    fn size_in(host: WidgetHost) -> WidgetSize { ... }
    fn placeholder(size: WidgetSize) -> Self::Entry;
    fn preview(size: WidgetSize) -> Self::Entry;
    fn view(entry: &Self::Entry, cx: WidgetContext<Self::Intent>) -> Element;
    fn title() -> Option<WidgetTitle> { ... }
}

// ds::host: one document seam, split into parts of at most six methods
pub trait DocumentHost {
    fn focus(&self) -> &dyn FocusHost;
    fn caret(&self) -> &dyn CaretHost;
    fn geometry(&self) -> &dyn GeometryHost;
    fn click_focus(&self) -> Option<&dyn ClickFocusHost>;
    fn edit(&self) -> Option<&dyn EditHost>;
    fn file_drop(&self) -> Option<&dyn FileDropHost>;
}
pub trait FocusHost {
    fn focus(&self, el: &MountedData) -> Focused;
    fn blur(&self, el: &MountedData) -> Focused;
    fn select(&self, el: &MountedData) -> Focused;
    fn hand_back(&self) -> &HandBack;
}
pub trait CaretHost {
    fn caret(&self, el: &MountedData) -> Caret;
    fn place_caret(&self, el: &MountedData, at: InitialCaret) -> Focused;
    fn selection(&self, el: &MountedData) -> FieldSelection;
}
pub trait GeometryHost {
    fn measure(&self, el: &MountedData) -> Measured;
    fn reveal(&self, list: &MountedData, item: &MountedData) -> Scrolled;
    fn find(&self, selector: &str) -> Found;
    fn same(&self, a: &MountedData, b: &MountedData) -> SameNode;
}
pub trait ClickFocusHost {
    fn fallback(&self, root: &MountedData) -> Fallback;
    fn restore(&self, ancestor: &MountedData) -> Focused;
    fn press(&self, root: &MountedData) -> Focused;
}
pub trait EditHost {
    fn hit_test(&self, surface: &MountedData, at: Point) -> Probe<TextPosition>;
    fn caret_rect(&self, surface: &MountedData, at: &TextPosition) -> Probe<Rect>;
    fn selection_rects(&self, surface: &MountedData, range: &TextRange) -> Probe<Vec<Rect>>;
    fn paste(&self) -> Option<Pasted>;
    fn capture(&self, surface: &MountedData, sink: EventHandler<CapturedPointer>) -> Probe<()>;
    fn ime(&self) -> &dyn ImeHost;
}
pub trait ImeHost {
    fn switch(&self, surface: &MountedData, to: ImeSwitch) -> Probe<()>;
    fn cursor_area(&self, surface: &MountedData, area: Rect) -> Probe<()>;
    fn listen(&self, surface: &MountedData, sink: EventHandler<ImeEvent>) -> Probe<ImeListener>;
    fn forget(&self, listener: ImeListener);
}
pub trait FileDropHost { fn hit(&self, targets: &[Rc<MountedData>], at: Point) -> DropHit; }
// The host provides one `Rc<dyn DocumentHost>` as root context (`ds_blitz::provide_host`
// installs every part, so a root can never install a subset) and one `HostSignals
// { modality: Signal<InputModality>, scale: Signal<Scale>, activity: Signal<Activity> }`;
// components read `use_document_host()` and get `NoHost` (every method answers `Busy`/absent) under SSR.

// ds::spell, ds::window (kept)
pub trait SpellService { fn languages(&self) -> Vec<Lang>;
    fn paragraphs(&self, surface: &MountedData) -> Probe<Vec<Paragraph>>;
    fn check(&self, langs: Vec<Lang>, words: Vec<String>) -> SpellFuture<Vec<String>>;
    fn suggest(&self, langs: Vec<Lang>, word: String) -> SpellFuture<Vec<String>>;
    fn ignore(&self, word: String);
    fn learn(&self, lang: Lang, word: String) -> SpellFuture<Learned>; }
pub trait HostWindow { fn begin_move(&self); fn begin_resize(&self, edge: ResizeEdge);
    fn zoom(&self, zoom: Zoom); fn minimize(&self); fn close(&self);
    fn tile(&self, tile: WindowTile) -> Result<(), TileError>;
    fn supports(&self, tile: WindowTile) -> Support; fn state(&self) -> WindowState; }
// (HostWindow is 8 methods today and splits into `WindowMove { begin_move, begin_resize, state }`
//  and `WindowPlace { zoom, minimize, close, tile, supports }` at the crate split.)

// ds-core::spawner
pub trait Spawner: Send + Sync {
    fn spawn(&self, task: std::pin::Pin<Box<dyn std::future::Future<Output = ()> + Send>>);
}

// ds-settings
pub trait SettingsDoc: Serialize + DeserializeOwned + Default + Clone + PartialEq + 'static {
    const FILE: FileName;
    const FORMAT: Format;
}
pub struct Store { ... }
impl Store {
    pub fn new(root: ConfigRoot, app: AppName) -> Store;
    pub fn load<D: SettingsDoc>(&self) -> Loaded<D>;                  // never fails
    pub fn save<D: SettingsDoc>(&self, doc: &D) -> Result<(), SettingsError>;   // atomic
    pub fn watch<D: SettingsDoc>(&self, spawn: &dyn Spawner) -> Watch<D>;
}
pub struct Loaded<D> { pub value: D, pub unknown: Vec<UnknownKey>, pub invalid: Vec<InvalidKey> }
// A key nobody reads is reported in `unknown`, never preserved on save; a bad value costs only
// its own key (`invalid`, the field's default is used). Feature `dioxus`: `use_environment`.

// ds-blitz::clipboard
pub trait Clipboard {
    fn read_text(&self) -> Option<String>;
    fn read_html(&self) -> Option<Pasted>;
    fn write_text(&self, text: &str);
}

// ds-harness
pub trait Driver {
    fn send(&mut self, input: Input);
    fn advance(&mut self, by: std::time::Duration);
    fn render(&mut self) -> Result<image::RgbaImage, HarnessError>;
}
pub trait DocQuery {
    fn with_doc<T>(&self, read: impl FnOnce(&blitz_dom::BaseDocument) -> T) -> T;
}
pub trait Query: DocQuery {   // blanket-implemented extension trait
    fn rect(&self, selector: &str) -> Option<Rect>;
    fn text_of(&self, selector: &str) -> Option<String>;
    fn attr(&self, selector: &str, name: &str) -> Option<String>;
    fn count(&self, selector: &str) -> usize;
    fn has_class(&self, selector: &str, class: &str) -> ClassPresence;
    fn ink_of(&self, selector: &str) -> Option<Srgba>;
    fn fill_of(&self, selector: &str, part: Part) -> Option<Srgba>;
    fn focus_of(&self, selector: &str) -> FocusState;
}
pub enum Input {
    Pointer(PointerInput), Key(KeyInput), Wheel { at: Point, dx: Px, dy: Px },
    FileDrag(FileDragInput), Ime(ImeInput), Paste { html: String, text: String },
}   // each part carries its `Modifiers`; the `_with` method pairs are gone
```

### Closed enums (never traits)

Tokens' values and the `Rule`/`Severity`/`Profile` of the linter; `MenuEntry`, `MenuKind`,
`MenuRow`; `Icon`, `IconSource`; `Anim`, `Exit`, `Moment`; `Presence`; `Painter`, `Backend`,
`Clock`, `NetPolicy`, the print route; `ConfigRoot::{Xdg, Scratch}`, `SystemPrefsSource::{Portal,
Fixed}`; `Input`; `KitRank`. A new variant is a compile error at every `match`, which is the
point.

## 6. The public surface

Every crate re-exports its public items once, at its root or from one public module; modules are
private; no glob re-exports; no `#[doc(hidden)]`. An item another crate needs is `pub` at its
home module (`ds_style::css::grain`), one path, and absent from every prelude.

`ds::prelude` (about 150 names) is what a consumer's `use ds::prelude::*` brings in; it re-exports
from `ds-core`, `ds-style`, `ds-motion` and `ds`, one `pub use` per name:

| Group | Names |
| --- | --- |
| Root and appearance | `Ds`, `Surface`, `Kits`, `Theme`, `Accent`, `Motion`, `MotionLevel`, `Material`, `Look`, `Scheme`, `SystemPrefs`, `Appearance`, `Resolved`, `Typeface`, `SpaceLook`, `Scope`, `use_scope` |
| Vocabulary | `Word`, `Availability`, `Selection`, `Emphasis`, `Switch`, `Expanded`, `Check`, `Shown`, `Fraction`, `Percent`, `ShortcutKey`, `Shortcut`, `Here`, `DropState` |
| Geometry | `Px`, `Point`, `Size`, `Rect`, `Scale`, `Placement`, `Alpha` |
| Icons | `Icon`, `IconSource`, `IconView`, `IconSize`, `ExternalIcon`, `StatusState`, `BatteryGlyph`, `WifiGlyph`, `VolumeGlyph`, `BluetoothGlyph` |
| Controls | `Label`, `Button`, `Bezel`, `ButtonRole`, `Answers`, `ImagePosition`, `IconSwap`, `Chip`, `ChipVariant`, `Toggle`, `Checkbox`, `RadioGroup`, `Choice`, `SegmentedControl`, `Tracking`, `Slider`, `SliderLook`, `Ticks`, `ProgressIndicator`, `ProgressStyle`, `Progress`, `LevelIndicator`, `LevelStyle`, `Bands`, `Badge`, `BadgeContent`, `BadgeTone`, `KeyEquivalent`, `KeyStyle`, `ControlSize` |
| Fields | `TextField`, `FieldKind`, `FieldBezel`, `Validity`, `FieldFocus`, `SelectionBubble`, `EditSurface`, `SpellMarks` |
| Menus | `Menu`, `MenuKind`, `MenuEntry`, `MenuRow`, `MenuTile`, `MenuTrail`, `MenuFilter`, `MenuCursor`, `CommandPalette`, `PaletteGroup`, `RowAction`, `RowChord` |
| Overlays | `Alert`, `Popover`, `Sheet`, `Tooltip`, `TooltipKind`, `HoverCard`, `Toast`, `ToastHost`, `Scrim`, `Panel`, `DragGhost`, `use_overlays`, `use_toasts` |
| Lists and content | `AnimatedList`, `LeavingList`, `ListRow`, `SettingsRow`, `SectionHeader`, `Avatar`, `ImageSource`, `RichText`, `TextLine`, `TextRun`, `ProviderMark`, `PreviewPane`, `EmojiGrid`, `PdfThumb` |
| Chrome | `WindowFrame`, `TrafficLights`, `WindowHost`, `ResizeEdge`, `WindowState` |
| Motion | `Anim`, `Presence`, `PresenceSpec`, `use_presence`, `Timeline`, `use_timeline`, `Exit`, `use_roster`, `use_pulse`, `use_swipe`, `use_drag`, `Detailed`, `Moment`, `use_detail`, `Cue` |
| Host | `DocumentHost`, `use_document_host`, `SpellService`, `HostWindow`, `HostSignals`, `Focused`, `Measured` |

`ds_shell::prelude` (about 60 names) holds the shell components' names and `Widget`, `WidgetKind`,
`WidgetRegistry`, `WidgetSize`, `WidgetHost`, `WidgetContext`. Mail-only components (`ds::app`)
are reached as `ds::app::X`, not through the prelude.

### Renamed names (every public type name is unique per crate)

| Today | Target |
| --- | --- |
| `Layers` (`motion/detail/pending`) / (`shell/notifications/parts`) | `PendingLayers` / `StackLayers` |
| `Placement` (`core/geometry`) / (`shell/catalog`, generic over kind, size, anchor) | `Placement` / `Placed` |
| `Ground` (`root/chrome`) / (`style/tokens/accent_band`) | `Ground` / `TextGround` |
| `Weight` (`style/fonts`) / (`style/tokens/accent_band`) | `FontWeight` / `BandWeight` |
| `Held` (`overlay/menu_track`) / (`motion/use_swipe`) | `MenuHold` / `SwipeHold` |
| `Span` (`shell/battery/ring`) / (`spell/words`) | `RingSpan` / `WordSpan` |
| `Level` (`shell/osd`), `Swipe` (`shell/notifications`) | `OsdLevel`, `NotificationSwipe` |
| `Filter`, `Cursor`, `Tile`, `Trail` (menus) | `MenuFilter`, `MenuCursor`, `MenuTile`, `MenuTrail` |
| `Focus` (text field), `Key` (shortcut), `Env` | `FieldFocus`, `ShortcutKey`, `Scope` |
| `Text`, `Run` (text runs) | `TextLine`, `TextRun` |
| `Step` (month grid), `Provider` (provider mark), `Mono` (preview) | `MonthStep`, `MarkProvider`, `PaneMono` |
| `OsdPhase`, `Alias` (`shown_phase`), `ListPresence`, `Healing` | deleted / `Presence` / `roster::Heal` |

## 7. Recipes

### Add a generic component (`ds`)

1. Pick the group by dependency (section 2) and create `crates/ds/src/<group>/<name>.rs` (view),
   `<name>.css` (its sheet), and, when it decides anything, `<name>_model.rs` with a pure `step`.
   State that is not a decision stays in the view.
2. Declare the module in `<group>/mod.rs`; derive `Word` on every closed prop enum.
3. Register the sheet in `assembly/sheets.rs`: one `Section { name, css }` at its cascade position.
4. Name every colour, duration, easing, radius and spacing through tokens (`var(--...)`); a
   missing token is added first (recipe below). Classes are `ds-<name>`; variants `data-variant`,
   state `aria-*`.
5. Add the name to `ds::prelude` only if a consumer draws it; otherwise it is reached by its
   group path.
6. Tests: `crates/ds/tests/<name>_ssr.rs` (SSR through `ds::testing::ssr`, golden through
   `ds::testing::golden`, then `ds_lint::assert_clean_markup`), and
   `crates/ds-conformance/tests/<name>.rs` when it responds to input or time (through `Driver`).
7. Gallery: `crates/ds-gallery/src/pages/<group>/<name>.rs`, one `Page` registered in
   `registry.rs`. Add the `DESIGN.md` row that names the design section.

### Add a shell component or widget (`ds-shell`)

1. Component: the same as above under `crates/ds-shell/src/<part>/{model.rs, step.rs, view.rs,
   style.css}`; its sheet goes in `ds_shell::KIT`'s sections; its tests in `crates/ds-shell/tests`.
2. Widget: create `crates/ds-shell/src/widgets/<kind>/{mod.rs, model.rs, view.rs, style.css}`;
   `impl Widget for <Kind>Widget` (unit type, `Default`); register it in
   `widgets/registry.rs::builtin()`; add its `Entry` wire round-trip test in
   `tests/widget_<kind>.rs`, its contract render (`tests/widgets_ssr.rs` row) and its gallery page
   `pages/shell/widget_<kind>.rs`. Its paint tokens are `ds-shell::tokens::widgets` variants.
3. A component the shell and another app both draw is generic: it belongs in `ds`.

### Add a token

1. Family exists: add the variant to its enum in `ds-style/src/tokens/<family>.rs` (shell
   families: `ds-shell/src/tokens/`); the `Word` derive gives its slug; add the value arm to
   `Token::css_value` (an exhaustive `match`, one value per scope where it differs).
2. New family: create the file, `#[derive(Word)]`, `impl Token`, and list `TokenSet::of::<T>()` in
   the crate's `KIT.tokens`. Nothing else: CSS emission and the lint vocabulary read `Kits`.
3. A tuned token (a consumer writes its value inline) sets `KIND = Tuned` and its default is the
   settings key's default (`design/22-SETTINGS.md`).
4. Tests: `ds-style/tests/tokens.rs` (every name a scheme or level block sets is declared on
   `.ds`); a new token with a raw value fails `ds-lint` by design.

### Add a motion recipe or a Rust-driven animation

1. Keyframes: add the `Anim` variant in `ds-motion/src/anim.rs` and one row in
   `recipe/table.rs` (name, duration token, easing token, fill); the keyframes come from
   `motion.css` and `keyframes.rs`. A variant a details moment plays is a row in `recipe/details.rs`.
2. A Rust-driven value (level, count, glide): `impl Timeline` in
   `ds-motion/src/timeline/<name>.rs` and call `use_timeline`; never a new hook, never a `sleep`.
3. Show/hide: `use_presence`; never a new phase enum.
4. Tests: `ds-motion/tests/motion_drift.rs` (table against CSS), the implementor's `at` at
   `0`, `total/2`, `total` as a table test, and a conformance test with `Clock::Virtual`.

### Add a lint rule

1. Add the `Rule` variant in `ds-lint/src/rule.rs` with its `Severity` (a HIG guardrail is
   `Warning`), and the rule file `ds-lint/src/{stylesheet,markup}/<name>.rs` returning
   `Vec<Offence>`; list it in the rule table of that directory's `mod.rs`.
2. Vocabulary it needs (known variables, keyframes, grammar) is read from the `Kits` in
   `LintConfig`, never from a hand list.
3. Tests: `ds-lint/tests/<name>.rs` with a failing and a passing fixture; then
   `ds`, `ds-shell` and `examples/consumer` must still pass their own lint tests.
4. Message the sill session before it lands: sill reads quire by path, so a new failing
   rule breaks sill master at once.

### Add a closed vocabulary (a `Word` enum)

1. Put it where its owner lives (section 4); `#[derive(Word)]`; add `#[serde(rename_all =
   "snake_case")]` and `#[word(case = snake)]` when it is stored.
2. Never write `slug`, `ALL`, `parse` or `label` by hand. Test: table over `T::ALL` that
   `T::parse(x.slug()) == Some(x)`; stored types also `word_matches_serde::<T>()`.

### Add a settings key or file

1. A `design/22-SETTINGS.md` row first (key, type, default, unit).
2. Field on the settings struct (`ds-settings/src/appearance/settings.rs`, or the consumer's
   struct) with `#[derive(SettingsSchema)]`, a typed value (a `Word` enum or a unit newtype, never
   a string or `bool`), default from the doc row.
3. New file: a struct with `impl SettingsDoc` (`FILE`, `FORMAT`); no other load or save path.
4. Tests in `ds-settings/tests/`: round trip; a bad value keeps the rest of the file; an unknown
   key lands in `Loaded::unknown`; `Store::new(ConfigRoot::Scratch(dir), app)`, never real XDG.

### Add a document-host capability

1. Extend the part trait in `ds/src/host/<part>.rs` (or add a part of at most six methods and one
   accessor on `DocumentHost`); give `NoHost` its answer.
2. Implement it in `ds-blitz/src/<part>.rs` and add it to `host::BlitzHost`; `provide_host`
   installs it with the rest.
3. The component that needs it reads `use_document_host().<part>()` and handles `Busy`.
4. Tests: a `ds-blitz` integration test through `ds-harness`; a conformance test for the
   component.

### Add a host input (harness)

1. A variant or field in `ds-harness/src/input.rs`; the delivery in `headless/`; the assertion
   helper as a `Query` method with a bool-free return.
2. A test in `ds-harness/tests/<input>.rs` that shows the input reaches the document.

### Add a conformance test

1. `crates/ds-conformance/tests/<component>.rs`, named for the component (never a work item).
2. Build `Harness` through `tests/support::app(...)`, drive with `Driver::send`, read with
   `Query`; the assertion names the difference the action makes (`CONVENTIONS.md` §8).
3. Timing on the wall clock uses `settle_until`; virtual-clock tests assert the boundary exactly.

### Add a gallery page

`crates/ds-gallery/src/pages/<group>/<component>.rs` exporting one `Page`; register in
`registry.rs`. Pages read tokens, never write CSS values.

## 8. Test harness

| Kind of thing | Test | Where |
| --- | --- | --- |
| Pure function or step | table test: `const CASES: &[(Input, Expected)]`, one loop | beside the code |
| Word, Token, Timeline, Detailed | table over `ALL` / sample instants / `moment_table` | the owning crate |
| Component markup | SSR render, golden, `ds_lint::assert_clean_markup` | `crates/<crate>/tests/<component>_ssr.rs` |
| Stylesheet | `ds_lint::assert_clean(css, &LintConfig::new(kits))` (`self_lint`) | `ds/tests/self_lint.rs`, `ds-shell/tests` |
| Component behaviour (pointer, keys, focus, time, paint) | `Harness` through `Driver` and `Query` | `ds-conformance/tests/<component>.rs` |
| Host behaviour (frames, clipboard, edit, drop, spell, PDF) | `Harness` on the real `BlitzHost` | `ds-blitz/tests/<topic>.rs` |
| Settings | `Store` on `ConfigRoot::Scratch`, `SystemPrefsSource::Fixed` | `ds-settings/tests` |
| Pixels | `Harness::render`, golden in `tests/snapshots/`, re-blessed only with a reason in the commit message | the owning crate |
| A consumer | `examples/consumer/tests/coherence.rs` | `examples/consumer` |

The one harness is `ds-harness`: `Harness` implements `Driver` and `DocQuery` over one headless
Blitz document with the same providers `launch` installs (`ds-blitz::provide_host`, `blitz-kit`
net, fonts and hover repair), so a test runs on the code production runs. Its clock is
`Clock::Wall` or `Clock::Virtual` (`with_clock`); a `Virtual` test asserts window boundaries
exactly, a `Wall` test polls with `settle_until` and asserts order. Tests never touch the real
system: no real XDG, no portal, no D-Bus, no clipboard, no GPU unless `Backend::Hybrid` is asked
for. `SSR` needs no harness: `ds::testing::ssr` (feature `testing`) renders a component inside `Ds`.
`sill` drives its surfaces through the same `Driver` trait once shell-host's headless surface
implements it (the traits build with `ds-harness` default features off).

## 9. Repo rules

- **Effect boundary** (`scripts/check-boundary.sh`): the tables in section 1. The script has three
  parts: the external-dependency table, the allowed-edges table (every workspace crate's
  `cargo metadata` dependency list against section 1), and the in-crate layer order of section 2
  (a file names only its own module and those below it, doc links included). A pure crate that
  seems to need an effect returns a description of it to its caller.
- **Gate:**

  ```bash
  cargo fmt --all --check
  cargo clippy --workspace --all-targets --all-features -- -D warnings
  cargo test --workspace --all-features
  ./scripts/check-boundary.sh
  cargo deny check licenses
  ```

- **No `unsafe`** anywhere in the workspace; `unsafe_code = "deny"`.
- **A Look is values only.** A Look supplies tokens (colour, radius, font family, grain, shadow), the
  material recipe and the backdrop (`design/30` section 3). It never swaps a component, adds a code
  path or changes a size, duration or behaviour; no component stylesheet selects on the Look.
- **Design-system rules:** every class is prefixed `ds-`; variants go in `data-variant` and
  `data-size`, state in `aria-*`; every colour, duration, easing, keyframe and font comes from
  the token table. A missing component or token is added here, never patched in a consumer.
- **Coherence rules for every consumer:** no stylesheet outside quire contains a hex/rgb/hsl
  colour, a raw `ms`/`s` duration, a raw `cubic-bezier`, a `@keyframes`, a `font-family` or a
  `:root` selector, and none styles `.ds-*` or `[data-theme|accent|motion|material]`
  (`ds_lint::stylesheet`, which every downstream crate runs in its own tests); no surface or app
  writes raw `button`/`input`/menu markup (`ds_lint::markup` over an SSR render).
  `CONSUMING.md` section 5 has the tests.
- **Lint profile:** `Profile::Strict` is the default and runs every rule but `OffGrammarTiming`
  (HIG guardrails report as warnings); `Profile::Details` adds the details grammar's timing. Consumers read quire by path, so
  a new failing rule breaks every consumer's tests at once: tell their owners before it lands.
- **Time in `ds*`** is read only through `ds_core::time::{now, since, sleep}`, never
  `Instant::now()` or `futures_timer` directly (`ds-core/tests/clock_rule.rs`), so the virtual
  clock reaches it. Library crates never call `tokio::spawn`; they take a `Spawner`.
- **Timing tests** on `Clock::Wall` never assert a state at one fixed instant near a settle,
  hover-intent, submenu or toast boundary: `Driver::advance` guarantees at least the time asked
  for. Poll with `settle_until`, assert order; a "not yet" check asserts a fixed elapsed time only
  at or under half the window. `Clock::Virtual` tests are exempt.
- **A settings key nobody reads is reported on load and not preserved.** A new settings key is a
  `design/22` row first.
- **`Px` and the geometry built on it are `f32`**, like the renderer's layout: `PartialEq`
  without `Eq`, the one float type in the data. Settings units (`ds-settings::units`) are stored
  integers and are never re-exported next to `ds-core`'s.
- **Every `Host*` context is a `DocumentHost` part or `HostSignals`.** A new seam is a part,
  never a new context newtype or a partial `provide`.
- **The pinned dependency block's source of truth is `docs/workspace-deps.toml`**; the root
  `Cargo.toml` and every consumer copy it verbatim. `blitz-kit` copies it too.
- **`docs/licensing-references.md`** is the verified licence table the borrowing rules cite.
- **Docs:** `DESIGN.md` names each module as `crate::module` and moves with every crate move.

## 10. Migration order

Each step lands through the gate worktree, keeps master green, and compiles sill by path. A step
never leaves a re-export "for callers" except the one named: `ds` keeps today's root names, one
path each, until step 12 replaces them with the prelude.

1. **Rename collisions** (done) (section 6) and the group order of section 2 inside `ds` (moves of
   `emoji_grid`, `preview_*`, `level glyph`, `appearance_picker`, `edit_surface*`; `overlay` ->
   `stack`); `check-boundary.sh` encodes the layers. sill is told the renames.
2. **`Word`** (done): `ds-core-derive`, `ds_core::word` in-crate, then convert the 111 hand-written
   `slug`s in batches by module; `SchemaVariants` and `schema/foreign.rs` go with the settings
   derive (step 6).
3. **`Token`, `TokenSet`, `Kit`, `Kits`** (done): `#[derive(Token)]` in `ds-core-derive`; every token
   family converted to it (the hand-written `var`/`css` fns and hand-kept lint lists go); shell
   metric tokens move to `shell/tokens/`, the stylesheet and `lint::registry` read `Kits`;
   `lint` stops naming `motion`.
4. **`Presence`, `Timeline`, `Shown`** (done): one `Presence` and `use_presence`, delete `osd_phase`,
   `shown_phase`, `ListPresence`; one `use_timeline` with the six implementors; `Shown` to core;
   `idle_dim` driver into `shell/`; `shot_frame` split; one battery drawing.
4a. **Catalogue**: build design/30's merges and additions, foundations first, each sub-step through
    the gate worktree:
    1. tokens: (done) the pruned duration, delay and easing tables (30 section 1.2), `ControlSize {Mini,
       Small, Regular, Large}` and `SizeScale` (1.6), `ds-style::look::Look` with the Mac values,
       Motion levels reduced to Standard and Reduced;
    2. vocabulary: (done) (1.5) `Shown`, `Check`, `Availability::Busy`, `PressPhase`, `Muting`, `Dismiss`,
       `RowState`, `Activity`, `FocusStyle`; the `Common` props; `ds::selectors`;
    3. motion primitives: (done) (1.3) `Roster` (one hook), `use_collapse`, `rubber`, spring only, drop the
       deleted keyframes, scalars and tokens; then interaction primitives (1.4): `LongPress`,
       `Roving` + `Typeahead`, `HoverIntent` profiles, focus ring and `Highlight`, drag threshold;
    4. P1 controls and fields: (done) `Label`, `Button` (+ `IconButton`), `Toggle`, `Checkbox`,
       `RadioGroup`, `SegmentedControl`, `Slider`, `TextField`, `ProgressIndicator`,
       `LevelIndicator`, `Badge`, `KeyEquivalent`;
    5. menus and lists: `Menu`, `MenuItem`, `PopUpButton`, `Disclosure`, `List`, `Row`,
       `SectionHeader`;
    6. overlays and feedback: (done) `Popover`, `Sheet`, `Alert`, `SidePanel`, `Tooltip`, `HoverCard`,
       `Toast`, `DockLabel`, `EmptyState`, `Skeleton`;
    7. shell-only pieces and app features (30 sections 2.10 and 2.11), then the P2 components;
    8. sill's switch-over (the local pieces L1-L16 of the component inventory) and the Arc Look;
       the gallery and goldens per component; delete every name in 30 Part 4.
5. **Crate-private crossers** (done): every `pub(crate)` item that crosses a layer (33 today) becomes
   `pub` at its home module or moves to its only consumer; the boundary script keeps the list
   empty.
6. **`ds-settings`** (done): `SettingsDoc`, `Store`, `ConfigRoot`, `SystemPrefsSource`, `Spawner` in
   `ds-core::spawner`, `dioxus` as a feature, delete `diff.rs` and `test_dir.rs`; sill's settings
   follow (unknown keys are reported).
7. **`DocumentHost`** (done): the part traits and `NoHost` in `ds::host`, `ds_native::provide_host`
   installing them all, `HostSignals`, delete every `Host*` newtype and the partial `provide`
   sets, `Clipboard` trait; sill roots call `provide_host` (this fixes the partial seam).
8. **Split bottom-up** (done), one crate per commit series, each with `ds` re-exporting: `ds-core`,
   `ds-style`, `ds-motion`, `ds-lint`, then `ds-shell` (assembly stays in `ds`). Each split updates
   the allowed-edges table in `check-boundary.sh` and `DESIGN.md`.
9. **`ds-native` -> `ds-blitz` + `ds-harness`**: rename, then move the harness; `pdf`, `print`,
   `spell` become features; `tokio` is named only in `launch`.
10. **`Driver`/`DocQuery`**: replace the five constructors and `_with` pairs with `Input`;
    `Query` extension trait.
11. **`ds-conformance`**: move `ds-native/tests` by the section 3 rules, renaming work-named
    files; move SSR tests beside their crates; `ds-gallery` pages regrouped.
12. **`ds::prelude`** replaces the root re-exports; a mechanical import rewrite in sill in the
    same change; `ds_shell::prelude` likewise.
13. **`blitz-kit`** (cross-repo): create the repo from the section 3 rows, switch `ds-blitz` and
    shell-host to it, delete both copies.
14. **`anyrender_pdfrum`** moves to the pdfrum repo as `pdfrum-anyrender`; `ds-blitz`'s `pdf`
    feature depends on it there.
15. **Docs**: `CONSUMING.md` for the new crate names, `DESIGN.md` paths, delete this section.
16. **User styles** (done) (section 11): `KitRank::User`, `UserStyle` + its watch in `ds-settings`, the
    `user_style` prop on `Ds`, `ds::selectors`, `lint::user_stylesheet`.

## 11. User styles

A person tunes their desktop in real time by editing one CSS file. quire owns the mechanism so
every consumer (sill, mailo, any app on quire) gets it the same way.

| Piece | Home | What it is |
|---|---|---|
| The file | `ds-style::kit::UserStyle`, `ds-settings::user_style` | `UserStyle(String)` (a value in `ds-style`, so `ds` can take it as a prop; re-exported by `ds-settings`), a `SettingsDoc` with `FILE = "style.css"` under the app's config dir (`~/.config/<app>/style.css`), `Format::Css` (raw text, never parsed at load). A missing file is an empty style. |
| Live reload | `ds-settings::Store::watch::<UserStyle>()` | the same watch every settings file uses (rename-safe, debounced); each change publishes the whole text |
| Cascade slot | `ds-style::kit::KitRank::User` | always after every kit, so a user rule wins over the design system by order, never by `!important` |
| Rendering | `ds::Ds { user_style: ReadSignal<UserStyle> }` | the root renders the text in its own `<style data-ds-user>` after the design-system stylesheet (no element while the style is blank); every surface root re-renders when it changes |
| Public selector surface | `ds::selectors` (a table, and its doc page) | what a user stylesheet may rely on: `[data-surface=<name>]` on every surface root; `.ds-<component>` on every component root; the parts each component lists as public (`.ds-<component>-<part>`); `data-variant`, `data-size`, `data-state`, `aria-*`; every token variable (`--<prefix><slug>`). Anything else is internal and may change without notice. |
| Report | `ds-lint::user_stylesheet(css, &Kits) -> Vec<UserStyleNote>` | report-only, never blocks loading: unknown variables, selectors outside the public surface, `url()` that is not a local `file:`/`data:` URL, parse errors with line numbers |

Rules:

- **User CSS is exempt from the design-system rules** (raw colours, durations, `font-family`
  are the point of it). The linter only reports the notes above.
- **Token overrides are the recommended form**: `.ds { --accent: ...; --t-tap: 120ms }`
  re-themes every surface consistently; full selectors are for what tokens cannot express.
- **Renaming anything on the public selector surface is a breaking change** for the person's
  file: it updates `ds::selectors`, and the consumer's release notes name it. Everything off the
  table stays freely renamable.
- **The reload path is the `<style>` element**: Blitz re-parses a `<style>` whose text changes
  and restyles the document (`ds-native/tests/user_style_reload.rs` proves colours, custom
  property overrides and an emptied sheet), so no host call is involved.
- **Recipe, expose a new public part**: add the part's class to the component, add its row to
  `ds::selectors`, add a gallery example that restyles it, and add a `lint::user_stylesheet`
  case that accepts it.

