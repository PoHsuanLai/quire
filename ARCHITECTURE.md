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
| `ds-motion` | `Anim` and recipes, keyframes, `Presence`, `Timeline`, rosters, gestures, pulse, the details grammar; feature `dioxus` adds the hooks, timers and event conversions over Dioxus (`ds` turns it on) |
| `ds-lint` | stylesheet and markup linter; reads its vocabulary from `Kits` |
| `ds-behaviour` | pure input machines a shell and a compositor both run: the app switcher, hot corners, the Command key's double tap and hold, the live Space swipe; no Dioxus, no clock |
| `ds-intents` | pure presentational context for the companion: the marks an app reports (`ThingMark`, `ContextChip`, `FieldMode`, the summon values) and the `ContextModel` seam; no agent type |
| `ds` | generic components, the `DocumentHost` seam and its hooks, overlay stack, root, `Ds`, stylesheet assembly, `ds::prelude` |
| `ds-shell` | app-facing parts above `ds`, cross-platform: the account sheets and consent alert, the missing-helper sheet, the confirm card, with their sheets and `KIT`. The surfaces only our desktop shell draws live in sill's `sill-shell-kit` (see "Moved to sill" in section 3) |
| `ds-settings-derive` | proc macro: `#[derive(SettingsSchema)]` |
| `ds-settings` | `SettingsDoc` + `Store`: lenient load, atomic save, watch, schema, appearance file, portal, icon-theme lookup |
| `ds-helpers` | missing helpers: the helpers file (capability, probe, per-distro package alternatives), the PATH probe, the distro family, the PackageKit installer (and a fake), the availability feed |
| `ds-desktop` | which extras of our desktop are here: `Capability` with its D-Bus name, `Desktop::probe` and `watch` over name ownership (feature `dbus`), `Outputs` (the shell's per-output work area and scale from `org.quire.Outputs1`, empty without it; `Outputs::read`/`watch` need `dbus`), `use_desktop` (feature `dioxus`); everything `Absent` without `dbus` |
| `ds-blitz` | `DocumentHost` on Blitz, app window host, launch, clipboard, `TextureLayer` and the window's GPU; features `pdf`, `print`, `spell`, `menus`, `debug-probe` (a test build only: `QUIRE_DEBUG_PROBE=<dir>` makes each window write its controls by role and name with their boxes, `ds-blitz::probe`) |
| `ds-harness` | dev-only test driver: `Driver`, `DocQuery`, `Harness`, snapshots, painters |
| `ds-conformance` | test-only crate: every component's behaviour tests, one file per component |
| `ds-gallery` | the visual reference binary: every component across theme, accent, motion, material |
| `tools/icons` | app-icon post-process (binary) |
| `blitz-kit` (own repo) | portable Blitz repairs shared with shell-host: hover sync, net provider, fonts, adapter, hit test, pixel snap; names no `ds*` crate |

### Allowed edges (workspace crates; everything else is forbidden)

| Crate | May depend on |
| --- | --- |
| `ds-core-derive`, `ds-settings-derive` | nothing of ours |
| `ds-core` | `ds-core-derive` |
| `ds-style` | `ds-core` |
| `ds-motion` | `ds-style`, `ds-core` |
| `ds-lint` | `ds-style`, `ds-core` |
| `ds-intents` | `ds-core` |
| `ds-behaviour` | `ds-core` |
| `ds` | `ds-intents`, `ds-motion`, `ds-style`, `ds-core` |
| `ds-shell` | `ds`, `ds-motion`, `ds-style`, `ds-core` |
| `ds-settings` | `ds-style` (system prefs, appearance enums), `ds-core`, `ds-behaviour` (the switcher and hot-corner settings convert into its machine params), `ds-settings-derive` |
| `ds-helpers` | nothing of ours |
| `ds-desktop` | nothing of ours |
| `ds-blitz` | `ds`, `ds-style`, `ds-core`, `ds-desktop` (the shell's `Outputs` as data; feature `desktop-outputs` adds its D-Bus reading), `blitz-kit`; feature `pdf`: `pdfrum-anyrender` (git dependency from the pdfrum repo) |
| `ds-harness` | `ds-blitz`, `ds-core`, `blitz-kit` |
| `ds-conformance` | dev-dependencies only: `ds`, `ds-shell`, `ds-lint`, `ds-settings`, `ds-blitz`, `ds-harness` |
| `ds-gallery` | `ds`, `ds-shell`, `ds-lint`, `ds-settings`, `ds-blitz`, `ds-harness` (snapshots) |
| `tools/icons` | `ds`, `ds-style`, `ds-settings` (`ds` for the Space palette derivation) |

Dev-dependencies follow the same table, plus: every crate may dev-depend on `ds` with feature
`testing`; `ds`, `ds-shell` and `ds-style` may dev-depend on `ds-lint`. Consumers (sill, mailo,
anyview, `examples/consumer`) depend on `ds` (+ `ds-shell` for the account and helper sheets,
`ds-settings`, `ds-blitz`, and `ds-lint`,
`ds-harness` as dev-dependencies); they never name `ds-core`, `ds-style` or `ds-motion` because
`ds` re-exports what they need (section 6).

### External boundaries (`scripts/check-boundary.sh`)

| Crate | Never reaches |
| --- | --- |
| `ds-core`, `ds-intents`, `ds-style`, `ds-motion`, `ds-lint`, `ds`, `ds-shell` | `zbus`, `notify`, `tokio`, `winit`, every `blitz*`, `stylo_taffy`, `dioxus-native*`, every `anyrender*`, `wgpu`, `wgpu_context`, `pdfrum*`, `arboard` |
| `ds-behaviour` | `zbus`, `notify`, `tokio`, `winit`, `dioxus`, every `blitz*`, `stylo_taffy`, `dioxus-native*`, every `anyrender*`, `wgpu`, `wgpu_context`, `pdfrum*` (a compositor links it, so like `ds-core` it is plain data and integer maths over `Stamp`) |
| `ds-core`, `ds-intents`, `ds-lint`, `ds-core-derive`, `ds-settings-derive` | `dioxus` (`ds-core` is plain data and maths, so sill's pure crates can use it; the linter reads strings; the derives generate paths: `::ds_core` or `::ds_style` where the caller's manifest names it, else `::ds::base` or `::ds::style`, found with `proc-macro-crate`) |
| `ds-style` | `dioxus` unless feature `dioxus` (appearance, tokens, palettes, icons and CSS text are plain data, so a compositor links them without it) |
| `ds-motion` | `dioxus` unless feature `dioxus` (recipes, keyframes, springs, throws, timelines, `Touch` as data; the hooks, timers and `Touch`'s event conversions are the feature) |
| `ds-settings` | every `blitz*`, `dioxus-native*`, `anyrender*`, `wgpu`, `wgpu_context`; `tokio` (it takes a `Spawner`); `dioxus` unless feature `dioxus` |
| `ds-helpers` | every `blitz*`, `dioxus*`, `anyrender*`, `wgpu`, `wgpu_context`; `tokio` (zbus brings its own executor; the tests use tokio as a dev-dependency) |
| `ds-desktop` | every `blitz*`, `anyrender*`, `wgpu`, `wgpu_context`; `tokio` (dev-dependency only); `zbus` unless feature `dbus`; `dioxus` unless feature `dioxus` |
| `ds-blitz` | `zbus` unless feature `print` or `menus`; `memfd` unless `print`; `pdfrum*` unless `pdf` |
| `blitz-kit` | every `ds*` crate, `dioxus*` |

`tokio` is named only by `ds-blitz` (the `launch` entry point owns the one runtime and hands a
`Spawner` to everything below) and by dev-dependencies of tests that need a runtime.

## 2. Modules inside each crate, in layer order

A module names only modules above it in its crate's list (earlier rows are lower). `lib.rs` holds
declarations and re-exports. Directories group a concept; role files follow `CONVENTIONS.md` §2
(`model.rs`, `step.rs`, `view.rs`, `style.css`, `tests.rs`).

| Crate | Modules, lowest first |
| --- | --- |
| `ds-core` | `word`, `vocab` (Availability, Selection, Emphasis, Switch, Expanded, Check, Shown, Fraction, Percent, StaggerIndex, ShortcutKey, Shortcut), `command` (names, `CommandFace`, `AppCommand`, a shortcut's chord text), `press`, `standard_action`, `geometry` (units, scale, scroll, placement), `colour` (srgb, oklab, fit, contrast), `time` (clock, virtual clock, `Stamp`, `FrameClock`), `machine` (the `Machine` trait, `Elapsed`), `spawner` (the `Spawner` trait), `error`, `text` (clip, `TypedText`), `codec` (png, base64) |
| `ds-style` | `appearance` (theme, accent, motion, blur, material choice, peek, system prefs, resolve, typeface), `scope` (the enclosing `Scope`), `tokens` (one file per token family, `set.rs` = `TokenSet`, `tuned.rs`), `kit` (`Kit`, `Kits`, `Section`, `Vocabulary`), `material`, `space` (palette, look, presets, frame vars, store, and `list`: the pure list of an app's Spaces), `icon` (glyph tables by family, plate, classify, render, url), `fonts`, `scale`, `css` (emission per cascade section, `reset.css`, `utilities.css`), `emit` |
| `ds-motion` | `anim` (`Anim`), `recipe` (the table), `keyframes` (generated CSS + `motion.css`), `settle` (settle, timers, wake, reduced), `presence`, `timeline` (`Timeline` + `use_timeline` + one file per implementor), `roster`, `pulse`, `gesture` (drag, swipe, velocity, hover intent), `machine` (the hooks that run a `Machine`: `use_machine`, `use_machine_in`, `use_machine_state`, `MachineRef`, `MachineState`), `details` (grammar, `Moment`, `Detailed`, cues, one-shots, glyph morphs) |
| `ds-behaviour` | `dir` (`Dir`) < `switcher`, `hot_corner`, `modifier_tap` (`tap`, `hold`), `space_swipe` (`numbers`, `velocity`, `finish`; each concept holds `model`, `step`, `tests`) |
| `ds-lint` | `rule` (`Rule`, `Severity`, `Profile`, `Exception`), `tokenize`, `walk`, `stylesheet` rules, `markup` rules, `hig`, `details`, `assert` |
| `ds` | `host` (the seam traits: `DocumentHost`, `FocusHost`, `CaretHost`, `GeometryHost`, `ClickFocusHost`, `EditHost`, `ImeHost`, `FileDropHost`, `NoHost`, `HostSignals`) < `focus`, `edit`, `file_drop`, `keys` (the window's chordkit keymap), `machine` (re-exports `ds_motion::machine`), `spell`, `window` (hooks and pure logic over the seams) < `stack` (overlay stack, hover hub, toast hub, menu tracker, pull tab) < `root` (`Surface`, chrome, extent, typeface) < components `content` < `controls` < `overlays` < `forms` < `lists` < `fields` < `menus` < `editor` < `chrome` < `companion` < `app` (mail-only, named by nothing but `assembly`) < `assembly` (`Ds`, stylesheet, sheet registration) < `prelude`, `testing` |
| `ds-shell` | `confirm`, `accounts` (`adapter` is the one file that reaches the components R2 makes controlled), `helpers` < `sheets` < `kit` < `prelude` |
| `ds-settings` | `error` < `root` (`ConfigRoot`, `AppName`) < `lenient` < `doc` (`SettingsDoc`, `Format`, `FileName`) < `store` < `spaces_storage` < `watch` < `schema` < `appearance` (file, settings structs) < `portal` < `live` (feature `live`: `org.quire.SettingsModule1`) < `environment` (feature `dioxus`) < `icon_assets` < `units` |
| `ds-blitz` | `error`, `contexts`, `setup`, `document` (node ref, origin, wake) < `focus`, `measure`, `reveal`, `phase` (the frame phase: `Phase`, its book of watches and queued writes, the reads), `edit` (+ ime, tree, geometry), `file_drop`, `clipboard`, `frames` < `host` (`BlitzHost`, `provide_host`) < `window` (build, place, platform, requests, hover, drop, shell), `window_scroll` (`WindowScroll`: winit's wheel and keys as `blitz_kit::scroll` inputs and as gestures, the fingers' glide for eased listeners, `ScrollHandle`), `window_fit` (the cap), `size_ledger`, `sized_window` < `window_sizer`, `screen_area` < `window_screen`, `startup_token`, `window_activate` (raising a window, with or without a token) < `xdg_activation` over `wayland_surface` (Linux; the one `unsafe`) < `app_life` (`LastWindowClosed`, the pure `Lifecycle`), `app_handle` (`AppHandle`, `AppHold`) < `launch`, `open_window` < `texture_layer` (model, fit, convert, gpu, widget, view) < features `pdf`, `print`, `spell`, `menus` (`service` over `wire`, over the pure `ds::menus::export`) |
| `ds-harness` | `input` (`Input` and its parts) < `driver` (`Driver`, `DocQuery`, `Query`) < `headless` (document, painter, backend, gpu paint, clock, settle) < `harness` (`Harness`) < `snapshot`; `inset` (the content inset check: `scene` < `read`, `measure`, `policy`, `offence`) |
| `ds-conformance` | `tests/<component>.rs`, `tests/support/` |
| `ds-gallery` | `axes`, `args`, `page`, `registry`, `pages/<group>/<component>.rs`, `sheet`, `snapshot`, `app` |

Component groups inside `ds` and the layer order inside the components tier: `content` (icons,
images, avatars, status glyphs, rich text, marks, PDF thumbs) < `controls` < `overlays` (alert,
popover, sheet, tooltip, toast, scrim, hover card, drag ghost, panel, flow) < `forms` (`Form`, `FormSection`, `IconTile`, `PaneStack` with `PageHeader` and `PanePath`: the grouped page, the tile a row leads with and the in-pane drill-in; `PaneStack` sits here, above `controls` for its back `Button` and below `lists` since the caller's pages hold the lists, and its motion is `ds-motion::use_pane_slide`, shared with the preview switcher) < `lists` (rows,
settings rows, headers, animated, leaving and virtual lists, preview pane, emoji grid, appearance picker)
< `fields` (text input, search field, selection bubble) < `menus` (menu, palette, entries, `ui_manifest`, `export`) <
`editor` (`EditSurface` and its spell menu) < `chrome` (window frame, traffic lights) < `companion` (orb, chips, answer cards, plan list, replace bar, run row, activity strip, memory view, served-by chip) < `app`.

## 3. From -> to (mechanical move)

<!-- paths: skip -->
Where each old path went. Paths are `crates/<crate>/src/...`; `ds/src/x` is the old location, the target is `crate::module`.
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
| `ds/style/tokens/*` except the files listed under `ds-shell` (now sill's `sill-shell-kit/src/tokens`, except `dock_floor.rs`, which stays in `ds-style` because the shape section draws `.ds-dock-floor`) | `ds-style::tokens::*` (`status.rs`, `emoji_face.rs`, `person.rs`, `plate.rs`, `size_*.rs`, `control_size.rs` stay: their consumers are generic) |
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
| `shell/{bar,battery,clock,control_center,emoji,lock,month_grid*,notifications,now_playing*,osd*,switcher,thumbs,user_picture*,dock*,idle_dim*}` | `ds-shell::` same names |
| `shell/widget/{contract,registry,scope,slot,layout,kind,wire,card,frame,gallery*,use_widget,timeline}.rs`, `shell/catalog/*` | `ds-shell::widgets::{contract, registry, scope, slot, layout, kind, wire, card, frame, gallery, use_widget, timeline, catalog}` |
| `shell/widget/{battery,calendar,clock}.rs` | `ds-shell::widgets::{battery, calendar, clock}/` (one directory each) |
| `style/tokens/{control_center,dock,notifications,osd,shell,shell_scale,widgets,widget_paint}.rs` | `ds-shell::tokens::*`, each implementing `Token` and listed in `ds_shell::KIT` |
| `lib.rs` root re-exports | deleted; replaced by `ds::prelude` and each crate's curated root |
| `detail.rs`, `icon.rs`, `time.rs`, `widget.rs`, `catalog.rs` (public shim modules) | deleted; their items are reached by the owning crate's root |

### Moved to sill (2026-10-07: `ds-shell` -> sill's `sill-shell-kit`)

Quire's `ds-shell` kept what an app on any platform takes: `accounts`, `helpers`, `confirm`. The
surfaces only our desktop shell draws moved, with their tokens, sheets, goldens, behaviour tests
and emoji assets, to `crates/sill-shell-kit` in the sill workspace under the same module names:
`bar`, `battery`, `catalog`, `clock`, `control_center`, `date_picker`, `dock`, `emoji`,
`idle_dim`, `lock`, `month_grid`, `notifications`, `now_playing`, `osd`, `switcher`, `thumbs`,
`user_picture`, `widget`, `tokens` (and the pieces they share: `kept`). The evidence is that
mailo and anyview name only `ds_shell::stylesheet()`, `ds_shell::helpers::model::HelperPhase`
and `ds_shell::prelude::HelperSheet`. `ds-style::tokens::dock_floor` stays here because the shape
section draws `.ds-dock-floor`. The moved sheets are consumer CSS there: a `sill_shell_kit::KIT`
of rank `Shell` (placed first among the shell-rank kits, so the cascade is byte for byte the old
one) next to `ds_shell::KIT`. The gallery drops the pages and sections for the moved components (the shell, chrome targets, level,
widget, lock and emoji pages, the notification and thumbnail overlays, the date picker and the
Settings window's Date & Time pane, the control center, player and status-item details, and the
`--level-sheet` mode); sill's
`sill-shell-kit/tests` holds their SSR goldens and behaviour tests, and `examples/sizing_audit.rs`
(the sizing audit's specimen sheet) lives there too.

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
<!-- paths: end -->

## 4. One home per concept

The single place a concept lives. Extend it; never write a second one.

| Concept | Home |
| --- | --- |
| Closed vocabulary (slug, label, ALL, parse) | `ds-core::word::Word` + `#[derive(Word)]` |
| Colour maths (sRGB, linear, OKLab, OKLCH, contrast, gamut fit) | `ds-core::colour` |
| Pixel units, points, rects, device scale | `ds-core::geometry` |
| Scroll state (`Scroll`: offset, viewport, content) and what follows from it: clamp, by, reveal, the rows that show, near the end | `ds-core::geometry::scroll` |
| A list that mounts only the rows near the viewport (one row height or one per key, cursor kept in view, paging cue, keyed exit or replacement) | `ds::components::lists::virtual_list` (`VirtualList`, `RowHeight`, `Change`; the prefix sum and window search in `layout`, its other pure steps in `model`) |
| A scroll container and its scroll state as an owned value | `ds::components::controls::scroller` (`use_scroller` -> `ScrollerRef`, drawn by `Scroller`) |
| Time, sleep, virtual clock | `ds-core::time` (`now`, `since`, `sleep`) |
| A machine's "when" (ms from an origin) and the clock that makes it | `ds-core::time::stamp` (`Stamp`, `FrameClock`); every timed machine in `ds-behaviour` and `ds-motion` takes the same `Stamp` |
| The app switcher's transition (Command-Tab over apps, quick tap, panel, Q, H, App Exposé), generic over the app key | `ds-behaviour::switcher` |
| A hot corner's dwell, fire and re-arm | `ds-behaviour::hot_corner` |
| Command's double tap (summon) and hold (talk) | `ds-behaviour::modifier_tap` (`Tap`, `HoldKey`) |
| The live Space swipe: follow, rubber band, commit, finish | `ds-behaviour::space_swipe` (`Swipe`; the numbers in `numbers`) |
| The switcher and hot-corner settings, and their conversion into machine params | `ds-settings::schema::behaviour` (`SwitcherSettings`, `HotCornerSettings`, `params()`) |
| A timed pure state machine and the one timer that drives it | `ds-core::machine` (`Machine`, `Elapsed`), `ds-motion::machine` (`use_machine`, `use_machine_in`, `use_machine_state`, `MachineRef`, `MachineState`; named `ds::machine` for apps) |
| A word or state shown for one `DelayToken` and then gone ("Copied", "Sent") | `ds::components::app::hold_for` (`HoldFor`, `use_hold_for`) |
| A recording's progress bar: loaded stretches, a time tooltip, a captured drag | `ds::components::controls::scrubber` (`Scrubber`; drawing in `scrubber_face`, machine in `scrubber_machine`, `BufferedRange`, `merged` and `time_text` in `scrubber_model`) |
| A floating pill of controls, and its media slots (progress bar, level) | `ds::components::chrome::capsule` (`Capsule`, `CapsuleSlot::{Item, Readout, Divider, Scrub, Level}`, `ScrubEvent`, `SlotPriority`, `RankedSlot`, `fit`) |
| Scope-owned tasks, spawning | `ds-style::task` (`spawn_in`); the `Spawner` trait in `ds-core::spawner` |
| Base vocabulary (`Availability`, `Switch`, `Shown`, `Fraction`) | `ds-core::vocab` |
| Standard actions, app actions, chords, `Primary`, the keymap and its sources, shortcut display | chordkit (own repo); quire keeps none of it. `ds-core::standard_action` re-exports `StandardAction`, `SpaceNumber` and `Conflict`; a `Shortcut` is the design vocabulary over a chordkit `DefaultChord` (`ds-core::command::shortcut_chord`, `shortcut_text`) |
| A key event becoming a chordkit key press, resolving to an action, and the type-ahead and wheel-zoom tests | `ds-core::command` (`key_input`, `resolve`, `types_text`, `holds_primary`); the one place a toolkit modifier is read |
| The window's keymap: loading from the injected source, reload, registering apps' actions, `on_action`, drawing a shortcut for the platform | `ds::keys` (`Keys`, `KeySource`, `use_keys`, `on_action`); quire's own actions are in `keys::quire_actions`; the launch's source and the detected default are `ds-blitz` (`AppConfig::with_keymap_source`, `keymap_detect`) |
| PNG, base64 | `ds-core::codec` |
| The crate error | `ds-core::error::DsError`; settings: `ds-settings::error::SettingsError` |
| Design tokens (colour, duration, easing, spacing, shape, type, layer) | `ds-style::tokens` (one file per family, `impl Token`) |
| Shell metric tokens (dock, OSD, notifications, control center, widgets) | sill's `sill-shell-kit::tokens` (`--dock-floor` is `ds-style::tokens::dock_floor`) |
| The account sheets (consent alert, add-account steps, account picker, no-account state, badge, limited note) and the secret text they carry | `ds-shell::accounts` (`model` the props, `wording` every sentence, `hidden::Hidden`; `adapter` the one file that reaches the components R2 makes controlled) |
| Window chrome geometry (titlebar, lights, resize edges) and the lights' state | `ds-style::tokens::chrome` (`ChromeToken`, `CHROME_SCALE`, `light_state`); `window_frame.css` reads the variables, `ds::components::chrome` draws |
| What each crate adds to the stylesheet, and the lint vocabulary | `ds-style::kit::Kit`; ordered by `Kits` |
| Appearance choice and resolution (theme, accent, motion, system prefs) | `ds-style::appearance` |
| The enclosing scope a component reads | `ds-style::scope::Scope` |
| Material, blur, frame ground | `ds-style::material`; `ds::root::chrome` |
| Space palettes and `SpaceLook` | `ds-style::space` |
| The Space a `Ds` root draws, read by what is under it (`use_space_look`, `use_space_frame`) | `ds-style::space::provided` |
| An app's list of Spaces: `Space<P>` (with its optional desktop `link`), `Spaces<P, R>`, switching with the place restored and the slide direction, `Today<I, K>` (items that expire after 12 h, parked things that never do), the lenient `spaces.json` reading | `ds-style::space::list` (pure, generic over the app's payload; `ds::components::app::spaces` re-exports it) |
| An app's `spaces.json` (config dir) and `today.json` (state dir): boot with a first run, a raw import and a payload fix-up; atomic writes | `ds-settings::SpacesStorage` (the app's data, not a `SettingsDoc`: the payload is the app's type) |
| The Spaces controller (`use_spaces`, `use_today`), the switch chord, the sidebar head and foot, the Space's menu and its parts, the Today section | `ds::components::app::spaces` |
| The Look (one value set, the Mac values) | `ds-style::tokens` (no Look type; design/30 section 3) |
| Glyphs (the `Icon` enum), plate, retint, classify | `ds-style::icon` |
| Font faces as bytes | `ds-style::fonts`; registration with the renderer: `blitz-kit::fonts` |
| Keyframes and recipes | `ds-motion::{anim, recipe, keyframes}` |
| Show/hide (entering, present, leaving) | `ds-motion::presence::{Presence, use_presence}` |
| Tweens, springs, count-up, sweeps | `ds-motion::timeline::{Timeline, use_timeline}` |
| List row motion (enter, exit, heal) | `ds-motion::roster` |
| Pulse (a replayed keyframe) | `ds-motion::pulse` |
| An icon's own motion (bounce, pulse, wiggle, breathe, rotate, draw on, a lid that lifts) | `ds-motion::symbol::{Symbol, SymbolEffect}`; part annotations `ds-style::icon::parts` |
| Drag, swipe, hover intent | `ds-motion::gesture` |
| What a state change means (moments) | `ds-motion::details::Detailed` |
| Stylesheet and markup linting | `ds-lint` |
| The document seam (focus, caret, geometry, edit, drop) | `ds::host::DocumentHost` |
| A mounted element's rect, kept current | `ds::host::layout::use_layout` (typed `MountedRef` in, `ReadSignal<Option<Rect>>` out); `ds::host::measure::use_rect` is the probe over it |
| Scrolling a window's containers: wheel, touchpad, keys, `ScrollCmd` | `blitz_kit::scroll` (the engine, shared with shell-host) driven by `ds-blitz::window_scroll::WindowScroll`, which the window loop runs on every winit event and the harness runs before each layout |
| What a touchpad's run and a zoom wheel say to an eased listener (the glide after a lift, Control delivered whole) | `ds-blitz::window_scroll::{coast, state}` over `blitz_kit::scroll::{momentum, velocity}`; the vocabulary is `ds::host::gesture::{Gesture, ScrollSource}` |
| A window's size: the cap to the screen, who resized it, whether a request was answered | `ds-blitz::window_fit` (`Fit`, `Extent::fit_with`, `WindowSize::fitting_with`), `size_ledger` (`SizeOrigin`, `SizeRequest`, the 500 ms answer window), `window_sizer` (`WindowSizer`, `use_window_sizer`) over `sized_window::SizedWindow` |
| The output a window is on: size, work area, scale | `ds-blitz::screen_area` (`ScreenArea`, `ScreenOf`, `WorkBasis`, `Reserve`), read from winit's monitors by `window_screen` and, under our shell, replaced by `ds-desktop::Outputs` (`ScreenArea::on_desktop`, `AppHandle::set_desktop_outputs`) |
| A scroller's range, its clamp and its rubber-band stretch | `ds-blitz::window_scroll::ScrollBounds`, over `blitz_kit::scroll::rubber::Band` |
| A stand-in window for a component that sizes its window | `ds-harness::fake_window` (`SizerAck`, `WindowScreen`) behind `HarnessConfig::with_sizer_ack` and the `Harness::{window_requests, window_size, resize_window, window_sizer}` methods |
| What a component asks the host to publish after layout, and the writes it queues for it | `ds::host::phase` (`Observe`, `Watch`, `PhaseWrite`) through `GeometryHost::{observe, write}`; the step is `ds-blitz::phase::Phase`, run by the window loop after each frame and by the harness after each layout. `Observe::Caret` is also run **before** a frame paints when the caret or selection moved (`Phase::early`: lay out, publish, render, then the draw paints), so an app's caret and selection (`ds::edit::caret::use_caret_rect`, `ds::edit::selection::use_selection_rects`) is drawn in the frame of the text it follows |
| Focus hooks, focus requests, whether a pressed control takes the keyboard (`FocusOnPressScope`) | `ds::focus` |
| Text editing surface | `ds::editor::EditSurface`; host half `ds-blitz::edit` |
| File drop | `ds::file_drop`; host half `ds-blitz::file_drop` |
| Spellcheck | `ds::spell::SpellService`; implementation `ds-blitz::spell` |
| Window frame, traffic lights, window host | `ds::chrome`; `ds::window::HostWindow`; implementation `ds-blitz::window` |
| A window's handle (raise, close) and the activation token it is created with | `ds-blitz::{use_window_handle, WindowHandle}`; `ds-blitz::startup_token`, `ds-blitz::window_platform` |
| Overlay stack, hover hub, toast hub | `ds::stack` |
| Menu data | `ds::menus::{MenuItem, MenuPlacement, MenuImage}` |
| What a menu item or shortcut is the face of (an action, or UI-only with a reason) and the `<AppName>.ui.toml` written from it | `ds-core::command` (`AppCommand`, `CommandFace`, `ActionName`); `ds::menus::ui_manifest` |
| Menu pointer tracking | `ds::stack::menu_track` |
| Popup menu view | `ds::menus::Menu` |
| A search field with suggestions under it: the panel it owns, its keys (Up, Down, Enter, Escape), its sections; and a menu hung from any field | `ds::menus::search` (`SearchField`, `SuggestionSection`, `step`); `ds::menus::menu::hung` (`Hung`, `PanelWidth`) |
| A result row's marked characters, avatar and second line | `ds::menus::item::text` (`Marks`, `ItemText`); `MenuImage::Avatar` |
| A toolbar's search item that collapses to a magnifier | `ds::chrome::toolbar::search` (`ToolbarSearch`, `SearchSeat`); the threshold is `toolbar::model::search_fit` |
| A palette's query, selection and what Enter runs, over any row type: the pure `Machine` a command palette, a launcher or a new-tab bar runs beside `CommandPalette`. Rows come in as params, ranked by the caller and kept in order; `Refreshed` re-clamps the selection when a provider changes them under an open palette | `ds::menus::palette::machine` (`PaletteState<T>`, `PaletteIn`, `PaletteOut<T>`, `PaletteParams<T>`, `PaletteIndex`, `PaletteMove`); the typed query is `ds_core::text::typed::TypedText` |
| A filterable list in a popover | `ds::menus::pick_list::PickList` (its rows and keys are the palette's) |
| Toasts | `ds::overlays::toast` + `ds::stack::toast_hub` |
| The root component and stylesheet assembly | `ds::assembly::{Ds, stylesheet}` |
| A status glyph and its state (Wi-Fi, battery, Bluetooth, volume) | `ds::content::status` |
| The battery drawing | `ds::content::status::battery::BatteryGlyph`; device models in sill's `sill-shell-kit::battery` |
| Widget contract, registry, placement data | sill's `sill-shell-kit::widget::{contract, registry}` and `catalog` |
| Emoji: the grid / the picker sheet | `ds::lists::emoji_grid` / sill's `sill-shell-kit::emoji` |
| Settings file I/O (lenient load, atomic save, watch) | `ds-settings::store::Store` |
| Settings schema | `ds-settings::schema` (derived) |
| Live (D-Bus) settings modules: schema, skeleton, proxy | `ds-settings::live` (feature `live`; the daemon owns the state) |
| Desktop preferences (portal) | `ds-settings::portal::SystemPrefsSource` |
| Test driver, document queries | `ds-harness::{Driver, DocQuery, Query}` |
| SSR rendering and golden files in tests | `crates/ds/tests/it/support/` (`golden.rs`, `scoped.rs`); `dioxus_ssr` |
| Clipboard | `ds-blitz::clipboard::Clipboard` |
| A GPU texture inside the document, the window's wgpu device, CPU pixels uploaded to it | `ds-blitz::texture_layer::{TextureLayer, use_gpu, Gpu, TextureHandle}`; headless device `ds-harness::Harness::gpu` |
| Net policy, `data:` URLs | `blitz-kit::net`, `blitz-kit::data_url` |
| Hover sync, pixel snap, GPU adapter choice, transform-aware hit test | `blitz-kit::{hover, snap, adapter, hit}` |
| PDF output, printing | `ds-blitz::{pdf, print}` |
| An app's menu bar as data for other processes: the tree, its `com.canonical.dbusmenu` layout and properties, an event mapped back to a `CommandId`, the revision book, the bus address derived from the `app_id` | `ds::components::menus::export` (`MenuTree`, `dbusmenu`, `MenuState`, `AppMenuAddress`); pure, no zbus |
| Serving that menu on the session bus (`com.canonical.dbusmenu`) | `ds-blitz::menus::MenuExport` (feature `menus`) |
| The companion's presence (idle, listening, working, acting, waiting) | `ds-core::vocab::CompanionPresence`; derived only by sill's `presence_of` |
| What an app tells the companion (a thing, a chip, a summon, a field's mode) | `ds-intents`; re-exported once from `ds::components::companion` |
| The companion's components: orb, chips, answer cards, plan, replace, run row, activity, memory, served-by | `ds::components::companion` |
| The mark of a result the person has not seen yet (still, ok or danger), on the orb and on a run row; the consumer passes `Some(Outcome)` and clears it with `None` | `ds::components::companion::outcome` (`Outcome`, `OutcomeMark`) |
| A Space's look as compact JSON of at most 1024 bytes, both ways | `ds_style::space::look_json` (`SpaceLook::to_json`, `SpaceLook::from_json`) |
| Text cut to a width in Rust, from the end (`clip_chars`) or the middle keeping an extension (`clip_middle`) | `ds_core::text::clip` |
| The confirmation card | `ds-shell::confirm` |
| The window glow's values (`GlowLook`, `GlowSpec`, `glow_spec`) | `ds-style::tokens::glow` (sill's `sill-shell-kit::tokens::glow` re-exports them) |
| Mail-only components | `ds::components::app` |

## 5. Traits and closed enums

### Canonical traits

| Trait | Crate | Implementors | Add an implementor when | Add a trait when |
| --- | --- | --- | --- | --- |
| `Word` | `ds-core` | every closed vocabulary enum (derived) | a new closed set | never |
| `Token` | `ds-style` | each token family; shell families in sill's `sill-shell-kit` | a new family | never |
| `Timeline` | `ds-motion` | `Ease`, `Spring`, `Glide`, `Pending` | a Rust-driven animation | never |
| `Detailed` | `ds-motion` | 13 component state enums | a component with moments | never |
| `Widget` | sill's `sill-shell-kit` | battery, world clock, month, and each widget kind | a new widget kind | never |
| `DocumentHost` (+ parts) | `ds` | `ds_blitz::BlitzHost`, `ds::host::NoHost` | a new renderer | a new capability of the document a component needs |
| `SpellService`, `HostWindow` | `ds` | `ds_blitz::spell::Hunspell`, a test fake; `ds_blitz::window::WinitWindow`, sill's shell-host window, a test stub | a new platform | never |
| `SettingsDoc` | `ds-settings` | `AppearanceFile`; each consumer's file | a new settings file | never |
| `Machine` | `ds-core` | each consumer's timed pure state (sill's dock, OSD, banners, ...; anyview), `ds::menus::palette::machine::PaletteState<T>`, `ds_behaviour::{Tap, HoldKey, Swipe, Switcher, Corner}`, `ds_motion::{Life (presence), RosterState, HoverIntent, SwipeState}`, `ds::stack::menu_track::MenuTrack` (the menu tracker), the components' own timers (the stepper's hold, the green light's hold, a pill's hold, a thread row's pointer, the voice orb's turn), `ds_shell`'s widget follower, track position and emoji frames | a state that changes by input and by time | never |
| `AppCommand` | `ds-core` | each app's command type (what its menu items yield and its shortcuts fire) | an app with menus or shortcuts | never |
| `Spawner` | `ds-core` | `ds_blitz::TokioSpawner`, a test's inline spawner | a new runtime | never |
| `Clipboard` | `ds-blitz` | `System`, `Memory` | never | never |
| `Driver`, `DocQuery` | `ds-harness` | `Harness`; later shell-host's headless surface | a new driver | never |
| `SizedWindow` | `ds-blitz` | `WinitSized` (a winit window), the harness's `FakeWindow` | a host that is not winit | never |

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
    fn css_value(self, scope: TokenScope) -> CssValue;   // TokenScope { scheme: Scheme, motion: MotionLevel, typeface: Typeface }
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
pub enum KitRank { Style, Motion, Components, Shell, User }  // cascade order; fixed by rank, not by argument order; User is last (section 10)
pub struct Kits { ... }
impl Kits { pub fn of(kits: &[&'static Kit]) -> Kits; pub fn stylesheet(&self) -> String;
            pub fn vocabulary(&self) -> Vocabulary; }
// Each crate exports `KIT`: ds_style::KIT, ds_motion::KIT, ds::KIT, ds_shell::KIT (and sill's
// sill_shell_kit::KIT, which a shell lists before ds_shell's so the sheets keep their cascade).
// ds::kits() = style + motion + ds; ds_shell::kits() = ds::kits() + the app-facing parts;
// sill_shell_kit::kits() = ds_shell's kits + the shell's own.

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
    fn observe(&self, el: &MountedData, what: Observe) -> Observed { ... }   // Unsupported by default
    fn write(&self, el: &MountedData, write: PhaseWrite) -> Queued { ... }   // Unsupported by default
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

// ds-core::machine (time is ds-core::time::stamp::Stamp: whole ms from the caller's origin)
pub trait Machine: Clone + PartialEq + 'static {      // no Default: the caller supplies the first state
    type In: From<Elapsed>;                   // what moves it; the clock alone can wake it
    type Out: 'static;                        // what it wants done
    type Params: Clone + PartialEq + 'static; // settings (timing, thresholds); a change applies from the next step
    type Ctx: 'static;                        // facts from outside the machine that a step reads (`()` if none)
    fn step(self, input: Self::In, at: Stamp, params: &Self::Params, cx: &Self::Ctx)
        -> (Self, Vec<Self::Out>);
    fn wake(&self) -> Option<Stamp>;          // when to step again with no input; none at rest
}
// ds-motion::machine (feature dioxus; `ds::machine` re-exports it): runs it on ds-core::time's
// clock, so a harness's virtual clock drives it.
pub struct MachineState<M: Machine> { pub state: Signal<M>, pub clock: FrameClock }   // Copy
pub fn use_machine_state<M: Machine>(initial: impl FnOnce(Stamp) -> M) -> MachineState<M>;
pub fn use_machine<M: Machine>(initial: impl FnOnce(Stamp) -> M, params: M::Params,
    ctx: impl Fn() -> M::Ctx + 'static, on_out: impl FnMut(M::Out, MachineRef<M>) + 'static) -> MachineRef<M>;
pub fn use_machine_in<M: Machine>(held: MachineState<M>, params: M::Params,
    ctx: impl Fn() -> M::Ctx + 'static, on_out: impl FnMut(M::Out, MachineRef<M>) + 'static) -> MachineRef<M>;
impl<M: Machine> MachineRef<M> {
    pub fn send(&self, input: M::In);              // from a handler, task or effect
    pub fn send_from_render(&self, input: M::In);  // from a component body: state now, outputs after the render
    pub fn state(&self) -> ReadSignal<M>;
    pub fn now(&self) -> Stamp;
    pub fn set_params(&self, params: M::Params);
}

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

### Machine: state that changes by input and by time

A timed machine is one pure type that the CALLER owns. The library decides; the caller keeps the
state, feeds the inputs and carries out the outputs. Roughly twenty timed machines had grown six
home-made timer shapes (request-a-tick outputs, tokens, `Instant` deadlines, per-hook tasks)
because the first `Machine` could not take a fact from outside, could not be seeded, and could not
be reached from its own output handler. The rules that replace them:

1. **`step` is pure and takes the time as a `Stamp`.** `(self, input, at, params, cx) -> (Self,
   Vec<Out>)`. A machine never reads a clock, never spawns, never sleeps.
2. **Deadlines live in the state.** `wake(&self)` reads the state alone, so a state that is waiting
   keeps the `Stamp` it waits for (`Armed { until }`), never the start time plus a duration it
   would need params to turn into a deadline. An elapsed wake is the input `Elapsed`; a step that
   is woken early or late checks `at >= until` itself, so a stale wake changes nothing and no
   token or serial is ever needed. An idle machine returns `None` and the hook runs no timer.
3. **Params are settings; `Ctx` is facts.** `Params` is what a settings key says (a delay, a
   threshold) and the surface passes it on each render; a change applies from the next step. `Ctx`
   is what the step must READ from outside the machine to decide (the switcher's list of running
   apps, a roster's measured row heights): read-only, owned by the caller, never settings. The hook
   asks for it with a reader closure at EVERY step, including the step a timer wakes, because
   between two renders the fact can change (a measurement arrives, an app quits). A machine with no
   such fact has `type Ctx = ();`. A fact a machine would otherwise have to keep a copy of (and
   keep in step) is `Ctx`, not state.
4. **No `Default`.** The caller supplies the first state (`initial: impl FnOnce(Stamp) -> M`, given
   the stamp of the moment the hook mounts, so a state seeded with a deadline counts on the hook's
   clock). A type may still implement `Default` where a rest state is natural; the trait does not
   ask for it.
5. **Two hook forms, one implementation.** The OWNED form, `use_machine`, keeps the state in the
   hook and is seeded by the caller. The CONTROLLED form, `use_machine_in`, takes a
   `MachineState` the caller made with `use_machine_state` (the signal that holds the state, and the
   `FrameClock` its stamps count on, both `Copy`): the caller can put it in a context, read it in a
   sibling, persist it or replace it. A write to the signal outside `send` re-arms the timer (the
   hook follows `wake()` of whatever the state is). The owned form is `use_machine_in` over a state
   the hook made, so there is one timer driver.
6. **The output handler can send.** `on_out(out, machine)` gets the `MachineRef`; an input it sends
   runs after the outputs already queued, never inside the handler that is running. A follow-up is
   an ordinary input, so it is stamped, stepped and tested like any other.
7. **Render-time input.** A hook whose input comes from a prop (a surface shown or hidden by its
   caller) sends from the component body with `send_from_render`: the state is stepped at once so
   this render draws it, the outputs wait until after the render, and the timer follows the state
   through the same effect. `send` is for handlers, tasks and effects.
8. **Tests.** The pure part is a table of `(state, input, at, ctx) -> (state, outputs, wake)`;
   `Elapsed` at each `wake()` drives a machine to rest with no clock. The hooks are tested once,
   in `ds-conformance/tests/machine.rs`, on the virtual clock (`Clock::Virtual`): each form, a
   context read at a timer-driven step, a follow-up send, and an idle machine that runs no timer.

What this does not cover: a machine that must read the wall clock for something other than
deadlines (none exists), and a hub that shares one machine across components (that is a
`MachineState` in a context, owned by the root).

### Closed enums (never traits)

Tokens' values and the `Rule`/`Severity`/`Profile` of the linter; `MenuItem`, `MenuPlacement`,
`Accessory`, `RowLeading`; `Icon`, `IconSource`; `Anim`, `Exit`, `Moment`; `Presence`; `Painter`, `Backend`,
`Clock`, `NetPolicy`, the print route; `ConfigRoot::{Xdg, Scratch}`, `SystemPrefsSource::{Portal,
Fixed}`; `Input`; `KitRank`. A new variant is a compile error at every `match`, which is the
point.

## 6. The public surface

Every crate re-exports its public items once, at its root or from one public module; modules are
private; no glob re-exports; no `#[doc(hidden)]`. An item another crate needs is `pub` at its
home module (`ds_style::css::grain`), one path, and absent from every prelude.

`ds` re-exports its three lower crates as facade modules, `ds::base` (`ds-core`), `ds::style`
(`ds-style`) and `ds::motion` (`ds-motion`), the only three root re-exports besides the stylesheet
assembly. A consumer that needs a name outside the prelude writes its home path under a facade
(`ds::style::tokens::shape::Radius`, `ds::base::word::Word`) and depends on `ds` alone, which is
why consumers never name `ds-core`, `ds-style` or `ds-motion` in their manifests.

`ds::prelude` (129 names) is what a consumer's `use ds::prelude::*` brings in; it re-exports
from `ds-core`, `ds-style`, `ds-motion` and `ds`, one `pub use` per name:

| Group | Names |
| --- | --- |
| Root and appearance | `Ds`, `Surface`, `Accent`, `Appearance`, `Material`, `Motion`, `MotionLevel`, `SystemPrefs`, `Scheme`, `Theme`, `Typeface`, `use_scope`, `SpaceLook` |
| Vocabulary | `Availability`, `Check`, `DropState`, `Emphasis`, `Fraction`, `Percent`, `Selection`, `Shortcut`, `ShortcutKey`, `Shown`, `Word` |
| Geometry | `Placement`, `Scale`, `Point`, `Px`, `Rect`, `Size`, `Alpha` |
| Icons | `ExternalIcon`, `IconSource`, `IconView`, `BatteryGlyph`, `BluetoothGlyph`, `StatusState`, `VolumeGlyph`, `WifiGlyph`, `Icon`, `IconSize` |
| Menus | `MenuCursor`, `Hung`, `PanelWidth`, `ItemText`, `Marks` |
| Fields | `EditSurface`, `TextField`, `SearchField`, `SuggestionSection`, `FieldFocus`, `FieldBezel`, `FieldKind`, `Validity` |
| Overlays | `Alert`, `DragGhost`, `HoverCard`, `InlineBanner`, `Loadable`, `Phase`, `Popover`, `Sheet`, `Skeleton`, `SkeletonShape`, `SkeletonLines`, `SkeletonRow`, `use_toasts`, `Tooltip`, `use_overlays` |
| Progress and pending work | `ProgressIndicator`, `Progress`, `ProgressStyle`, `RingGap`, `Operation`, `PendingToken` |
| Lists and content | `Avatar`, `ImageSource`, `PdfThumb`, `ProviderMark`, `TextLine`, `TextRun`, `EmojiGrid`, `List`, `ListItem`, `PreviewPane`, `Accessory`, `RowAction`, `RowChord`, `RowLeading`, `Row`, `SectionHeader` |
| Chrome | `TrafficLights`, `WindowFrame`, `WindowHost`, `ResizeEdge`, `WindowState` |
| Motion | `Anim`, `Cue`, `Detailed`, `Moment`, `use_detail` |
| Host | `DocumentHost`, `use_document_host`, `Focused`, `Measured`, `HostSignals`, `SpellService`, `HostWindow` |
| Controls, menus and root pieces | `Label`, `Button`, `Choice`, `RadioGroup`, `SegmentedControl`, `Slider`, `Toggle`, `MenuItem`, `MenuImage`, `Menu`, `MenuPlacement`, `CommandPalette`, `CommandPaletteHost`, `PaletteState`, `PaletteIn`, `PaletteOut`, `PaletteParams`, `PaletteIndex`, `PaletteMove`, `EmptyState`, `Flow`, `SidePanel`, `RootChrome`, `RootExtent`, `settle`, `use_motion_timer`, `resolve` |

`ds_shell::prelude` (16 names) holds the account sheets' components and the missing-helper sheet.
sill's `sill_shell_kit::prelude` holds the shell components' names and `Widget`, `WidgetKind`,
`WidgetRegistry`, `WidgetSize`, `WidgetHost`, `WidgetContext`. Mail-only components (`ds::components::app`)
are reached as `ds::components::app::X`, not through the prelude.

### Renamed names (every public type name is unique per crate)

| Old | Current |
| --- | --- |
| `Layers` (`motion/detail/pending`) / (`shell/notifications/parts`) | `PendingLayers` / `StackLayers` |
| `Placement` (`core/geometry`) / (`shell/catalog`, generic over kind, size, anchor) | `Placement` / `Placed` |
| `Ground` (`root/chrome`) / (`style/tokens/accent_band`) | `Ground` / `TextGround` |
| `Weight` (`style/fonts`) / (`style/tokens/accent_band`) | `FontWeight` / `BandWeight` |
| `Held` (`overlay/menu_track`) / (`motion/use_swipe`) | `MenuHold` / `SwipeHold` |
| `Span` (`shell/battery/ring`) / (`spell/words`) | `RingSpan` / `WordSpan` |
| `Level` (`shell/osd`), `Swipe` (`shell/notifications`) | `OsdLevel`, `NotificationSwipe` |
| `Cursor` (menus) | `MenuCursor` |
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
6. Tests: `crates/ds/tests/<name>_ssr.rs` (a `VirtualDom` rendered with `dioxus_ssr`, a golden through
   `tests/support/golden.rs`, then `ds_lint::markup` filtered to the rule under test), and
   `crates/ds-conformance/tests/<name>.rs` when it responds to input or time (through `Driver`).
7. Gallery: `crates/ds-gallery/src/pages/<group>/<name>.rs`, one `Page` registered in
   `registry.rs`. Add the `DESIGN.md` row that names the design section.

### Add an app-facing part above `ds` (`ds-shell`)

1. Component: the same as above under `crates/ds-shell/src/<part>/{model.rs, step.rs, view.rs,
   style.css}`; its sheet goes in `crates/ds-shell/src/sheets.rs`; its tests in `crates/ds-shell/tests`.
2. It must work on every platform (no D-Bus, no compositor). A surface only our desktop shell draws
   (bar, dock, widgets, lock, notifications, OSD) is not added here: it goes in sill's
   `sill-shell-kit` (its recipe is in sill's ARCHITECTURE.md).
3. A component the shell and another app both draw is generic: it belongs in `ds`.

### Add a token

1. Family exists: add the variant to its enum in `ds-style/src/tokens/<family>.rs` (shell
   families: sill's `sill-shell-kit/src/tokens/`); the `Word` derive gives its slug; add the value arm to
   `Token::css_value` (an exhaustive `match`, one value per scope where it differs).
2. New family: create the file, `#[derive(Word)]`, `impl Token`, and list `TokenSet::of::<T>()` in
   the crate's `KIT.tokens`. Nothing else: CSS emission and the lint vocabulary read `Kits`.
3. A tuned token (a consumer writes its value inline) sets `KIND = Tuned` and its default is the
   settings key's default (`design/22-SETTINGS.md`).
4. Tests: `ds-shell/tests/tokens.rs` and sill's `sill-shell-kit/tests/tokens.rs` (every name a scheme or level block sets is declared on
   `.ds`); a new token with a raw value fails `ds-lint` by design.

### Add a motion recipe or a Rust-driven animation

1. Keyframes: add the `Anim` variant in `ds-motion/src/anim.rs` and one row in
   `ds-motion/src/recipe.rs` (name, duration token, easing token, fill); the keyframes come from
   `motion.css` and `css.rs`. A variant a details moment plays is a row in `recipe_detail.rs`.
2. A Rust-driven value (level, count, glide): `impl Timeline` in
   `ds-motion/src/timeline/<name>.rs` and call `use_timeline`; never a new hook, never a `sleep`.
3. Show/hide: `use_presence`; never a new phase enum.
   A symbol effect is an `Anim` row in `recipe_symbol.rs` plus an arm in `symbol/route.rs`
   (`once_anim`, `loop_anim`); a motion inside an SVG (Blitz's stylesheet does not reach there)
   is a gesture in `symbol/pose.rs` and an annotation in `ds-style/src/icon/parts.rs`.
4. Tests: `ds/tests/motion_drift.rs` (table against CSS), the implementor's `at` at
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
2. Build `Harness::new(app, viewport_or_config)`, drive with `Driver::send`, read with
   `Query`; the assertion names the difference the action makes (`CONVENTIONS.md` §8).
3. Timing on the wall clock uses `settle_until`; virtual-clock tests assert the boundary exactly.

### Add a gallery page

`crates/ds-gallery/src/pages/<group>/<component>.rs` exporting one `Page`; register in
`registry.rs`. Pages read tokens, never write CSS values.

## 8. Test harness

> Test paths below written `tests/<name>.rs` mean `tests/it/<name>.rs`: a crate's integration tests are
> modules of one executable (`tests/it/main.rs`), and a separate target needs a stated reason
> (CONVENTIONS.md section 8).

| Kind of thing | Test | Where |
| --- | --- | --- |
| Pure function or step | table test: `const CASES: &[(Input, Expected)]`, one loop | beside the code |
| Word, Token, Timeline, Detailed | table over `ALL` / sample instants / `moment_table` | the owning crate |
| Component markup | SSR render, golden, `ds_lint::markup` | `crates/<crate>/tests/it/<component>_ssr.rs` |
| Stylesheet | `ds_lint::assert_clean(css, &LintConfig::new(kits))` (`self_lint`) | `ds-shell/tests/self_lint.rs` (sill: `sill-shell-kit/tests/self_lint.rs`) |
| Component behaviour (pointer, keys, focus, time, paint) | `Harness` through `Driver` and `Query` | `ds-conformance/tests/<component>.rs` |
| Host behaviour (frames, clipboard, edit, drop, spell, PDF) | `Harness` on the real `BlitzHost` | `ds-blitz/tests/<topic>.rs` |
| Settings | `Store` on `ConfigRoot::Scratch`, `SystemPrefsSource::Fixed` | `ds-settings/tests` |
| Pixels | `Harness::render`, golden in `tests/snapshots/`, re-blessed only with a reason in the commit message | the owning crate |
| Content inset (text, glyphs, images 8 px off a painted box edge) | `ds_harness::inset::assert_insets(&harness, &Policy::quire())` | `ds-gallery/src/inset_tests.rs` (every page, light and dark); `ds-harness/tests/inset.rs` (the detector) |
| A consumer | `examples/consumer/tests/coherence.rs` | `examples/consumer` |

The one harness is `ds-harness`: `Harness` implements `Driver` and `DocQuery` over one headless
Blitz document with the same providers `launch` installs (`ds-blitz::provide_host`, `blitz-kit`
net, fonts and hover repair), so a test runs on the code production runs. Its clock is
`Clock::Wall` or `Clock::Virtual` (`with_clock`); a `Virtual` test asserts window boundaries
exactly, a `Wall` test polls with `settle_until` and asserts order. Every harness has a window
sizer over a fake window (`use_window_sizer()` is `Some`): it records the sizes asked and answers
as `with_sizer_ack` says. Tests never touch the real
system: no real XDG, no portal, no D-Bus, no clipboard, no GPU unless `Backend::Hybrid` is asked
for. SSR needs no harness: a test renders a `VirtualDom` with `dioxus_ssr` (`crates/ds/tests/support/`).
`sill` drives its surfaces through the same `Driver` trait once shell-host's headless surface
implements it (the traits build with `ds-harness` default features off).

The content inset check lives in `ds-harness`, not `ds-lint` or `ds-conformance`. It reads
*laid-out* geometry (the glyph runs and border boxes of a Blitz document), which `ds-lint` (strings
only, no Blitz) cannot, and `ds-conformance` is test-only (a consumer cannot depend on its
tests), while `ds-harness` is the dev-dependency every consumer already has and the one crate whose
`DocQuery` reads any rendered document, quire's or an app's. Its `Policy::quire()` allow table names
the components the design holds under `--s-8`, each with its reason; an app adds its own.

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
  ./scripts/check-portable.sh .
  ./scripts/check-consumer.sh
  cargo deny check licenses
  ```

  `check-portable.sh .` builds `cargo check --workspace --no-default-features` and fails when a
  file outside `.portable-allow` names zbus: ds-settings, ds-helpers and ds-blitz keep their
  desktop integration (portal, live modules, PackageKit, xdg-activation, print portal, menus)
  behind `quire-desktop` (default on), and each workspace member forwards it.

  `check-consumer.sh` builds, clippies (`-D warnings`) and tests `examples/consumer`, its own
  cargo workspace that `--workspace` does not reach, so `CONSUMING.md`'s snippets and coherence
  rules stay true to the public API.

- **One module allowed `unsafe`:** `crates/ds-blitz/src/wayland_surface.rs` (`#![allow(unsafe_code)]`;
  Linux only), everywhere else `unsafe_code = "deny"` at the workspace. It holds two blocks, in
  `adopt`: `Backend::from_foreign_display` over the `wl_display*` of a window's
  `DisplayHandle`, and `ObjectId::from_ptr` over the `wl_surface*` of its `WindowHandle`
  (class-checked by libwayland). The result, `Adopted<'a>`, borrows both handles, so neither
  the connection nor the surface outlives the window's handles; `xdg_activation` is safe code
  on top of it, and `WindowHandle::focus_with_token` is the only caller. The module's tests
  run it against a fake compositor (`crates/ds-blitz/src/xdg_activation/fake_compositor.rs`). Its test-only
  `Foreign` holds the other two `unsafe` blocks (`borrow_raw`).
- **One Look.** The token table is the Look's values (`design/30` section 3); there is no Look type
  and no `data-look`. What the design takes from Arc are features, not values.
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
  `Instant::now()` or `futures_timer` directly (`ds/tests/clock_rule.rs`), so the virtual
  clock reaches it. Library crates never call `tokio::spawn`; they take a `Spawner`.
- **Tests run on the virtual clock.** Every test harness names its clock
  (`HarnessConfig::new(view).with_clock(Clock::Virtual)`; `ds/tests/clock_rule.rs` fails a
  harness that does not), and a bare `VirtualDom` that needs timers uses `ds/tests/support/dom_time.rs`.
  Several sessions build on one machine, so load averages of 70-120 are normal and a wall-clock
  test fails on a different run each time. Virtual time asserts exact instants (`settle_until`
  returns the instant the timer fired; `assert_eq!` it). `Clock::Wall` is for a test that asserts
  something about real time (Blitz's wall-clock double click, the split-clock repro), and it says
  so at the call. A test that waits on a real thread (the spell worker, a PDF raster) waits on
  the event through `ds-blitz/tests/support/worker.rs` with a generous hang guard no passing run
  reaches; a negative ("it must not have answered") gives the thread real time and can only let a
  wrong answer through, never fail a right one. A wall-time budget is not a gate test: mark it
  `#[ignore = "perf: run with --ignored on a quiet machine"]` and add it to `scripts/perf.sh`.
- **On `Clock::Wall`** (the exceptions) never assert a state at one fixed instant near a settle,
  hover-intent, submenu or toast boundary: `Driver::advance` guarantees at least the time asked
  for. Poll with `settle_until`, assert order; a "not yet" check asserts a fixed elapsed time only
  at or under half the window.
- **A settings key nobody reads is reported on load and not preserved.** A new settings key is a
  `design/22` row first.
- **`Px` and the geometry built on it are `f32`**, like the renderer's layout: `PartialEq`
  without `Eq`, the one float type in the data. Settings units (`ds-settings::units`) are stored
  integers and are never re-exported next to `ds-core`'s.
- **Every `Host*` context is a `DocumentHost` part or `HostSignals`.** A new seam is a part,
  never a new context newtype or a partial `provide`.
- **The pinned dependency block's source of truth is `docs/workspace-deps.toml`**; the root
  `Cargo.toml` and every consumer copy it verbatim. `blitz-kit` copies it too. The block holds the
  `blitz-kit` line, a git dependency at a pinned rev (public repo). Each workspace root that
  builds it from a sibling checkout (quire, shell-host, sill) adds
  `[patch."https://github.com/PoHsuanLai/blitz-kit"] blitz-kit = { path = "../blitz-kit/crates/blitz-kit" }`,
  so local builds use one checkout while a git consumer of quire, whose `[patch]` is ignored,
  gets the pinned rev. A blitz-kit bump is a rev change in the block, in all three repos.
- **`docs/licensing-references.md`** is the verified licence table the borrowing rules cite.
- **Docs:** `DESIGN.md` names each module as `crate::module` and moves with every crate move.

## 10. User styles

A person tunes their desktop in real time by editing one CSS file. quire owns the mechanism so
every consumer (sill, mailo, any app on quire) gets it the same way.

| Piece | Home | What it is |
|---|---|---|
| The file | `ds-style::kit::UserStyle`, `ds-settings::user_style` | `UserStyle(String)` (a value in `ds-style`, so `ds` can take it as a prop; re-exported by `ds-settings`), a `SettingsDoc` with `FILE = "style.css"` under the app's config dir (`~/.config/<app>/style.css`), `Format::Css` (raw text, never parsed at load). A missing file is an empty style. |
| Live reload | `ds-settings::Store::watch::<UserStyle>()` | the same watch every settings file uses (rename-safe, debounced); each change publishes the whole text |
| Cascade layers | `ds-style::css::layers` (`DS`, `APP`, `ORDER`), `ds::prelude::AppStyle` | the whole design-system stylesheet (every kit) is `@layer ds { ... }`, opened by the statement `@layer ds, app;`; a consumer's own sheets are `@layer app` (`AppStyle { css }` writes the same statement, so the order holds whichever sheet parses first); the person's `<style data-ds-user>` is unlayered. An unlayered rule beats every layered rule and a later layer beats an earlier one, whatever the specificity, so the order of who wins is user, then app, then ds. Inside `ds` the kits keep the order and specificity they always had (the marker order below); there are no per-kit sub-layers, because a sub-layer would let a Shell rule beat a Components rule by rank alone. |
| Cascade slot | `ds-style::kit::KitRank::User` | always after every kit: a user kit's sections are written after `@layer ds { }`, unlayered. A user rule wins over the design system by the cascade layer, never by `!important` or specificity |
| Rendering | `ds::prelude::Ds { user_style: ReadSignal<UserStyle> }` | the root renders the text in its own `<style data-ds-user>` after the design-system stylesheet, unlayered (no element while the style is blank); every surface root re-renders when it changes |
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
  and restyles the document (`ds-blitz/tests/user_style_reload.rs` proves colours, custom
  property overrides and an emptied sheet), so no host call is involved.
- **Recipe, expose a new public part**: add the part's class to the component, add its row to
  `ds::selectors`, add a gallery example that restyles it, and add a `lint::user_stylesheet`
  case that accepts it.

- **A consumer's sheets go in `app`**: draw each with `ds::prelude::AppStyle`, never a bare
  `style { {CSS} }`. A bare sheet is unlayered and would outrank every `ds` rule whatever its
  specificity. `ds_lint::Rule::LayerDs` rejects a consumer sheet that opens `@layer ds`.
- **Selector spelling**: an unprefixed attribute selector (`[data-surface=bar]`) matches on
  Blitz since the fork's `1b23fbd9` (dioxus-native-dom writes attributes with no namespace, as
  the DOM does); the `*|` form still matches, and quire's own generated sheet keeps writing it.
