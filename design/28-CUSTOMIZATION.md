# 28 Customization: what the person chooses and places

Status: this whole document is **proposed** (2026-09-27) unless a line says settled. Status and
confidence legend as in `23-WIDGETS.md`: **H** primary source (Apple's HIG or Apple Support),
**M** reliable secondary source, **L** memory of the shipping product, not checked against a
page. The reference is the pre-Liquid-Glass Mac (macOS 14 Sonoma and 15 Sequoia;
`27-HIG-PARITY.md#01-which-hig`). The reference platform is named here as a source only; code,
class names, tokens and asset names never name it.

## 1. What this governs

The user's intent (2026-09-27): "the reason to make the card an interface is to let user choose
what to be placed. please check what other thing should be like this, for example i think the
spotlight item as well probably?"

The widget card is being built as the first instance of that idea (quire branch
`widget-interface`, design/23 section 9, in progress): a **registry of kinds**, **one Rust trait
per kind**, a **picker gallery with live previews**, and **placements stored as data** (kind,
size, place, position). This document:

1. lists every surface where the reference lets the person choose *what* appears and *where*
   (section 3 is the summary table, section 4 one subsection per surface);
2. says for each what the person picks, the reference's picking UI, whether other apps can add
   items, and our mapping: what sill has today, which repo owns the trait, the trait's rough
   shape, the picker component, the settings rows (design/22 style, all proposed) and a priority;
3. defines the one shared pattern every such surface uses (section 5): `Registry<K>`, a trait
   per surface, the picker components, `Placement` data, persistence, and how an app in another
   process registers later (D-Bus, documented, not built);
4. names the existing quire and sill parts that become generic (section 5.8), the order of work
   (section 6) and the open decisions with a recommendation each (section 7).

Out of scope: the look of each picker beyond naming its components (04 owns components), the
settings of scalar values (22 owns them), and any Rust (this is a design pass).

Priority vocabulary: **must** = needed for "feels like a Mac" parity or already asked for by the
user; **nice** = the reference has it, the absence is noticed only by a power user; **skip** =
not in the reference era, or not worth its cost here, recorded so nobody re-researches it.

## 2. Sources and era

HIG pages were read through their JSON (`developer.apple.com/tutorials/data/design/human-interface-guidelines/<page>.json`)
from Wayback snapshots before the June 2025 redesign, as `27-HIG-PARITY.md#02-snapshots` does.

| HIG page | Snapshot | Used for |
| --- | --- | --- |
| widgets | 20250224003541 | 4.1: gallery, edit mode, sizes, configuration, preview, description |
| controls | 20250303222558 | 4.3: Control Center controls are **"Not supported in macOS"** in this era |
| the-menu-bar | 20241214023016 | 4.4: menu bar extras, "Let people — not your app — decide" |
| searching | 20240210073513 | 4.2, 4.8: index content for Spotlight, importers, Quick Look generators |
| toolbars | 20240302055431 | 4.6: "consider letting people customize the toolbar"; every item also a menu command |
| activity-views | 20250224003701 | 4.7: share and action extensions on macOS (Share button, context menu, Quick Actions) |
| sidebars | 20250228024414 | 4.12: "When possible, let people customize the contents of a sidebar" |
| notifications | 20250531072050 | 4.14: people choose styles; badges a per-app choice |
| managing-notifications | 20250502150210 | 4.14, 4.15: Focus, delivery scheduling, Time Sensitive |
| keyboards | 20250225175135 | 4.11: respect standard shortcuts, custom ones sparingly |
| settings | 20231102161025 | 5.6: system-wide options belong to System Settings, not the app |
| dock-menus | 20250407102757 | 4.5 |

Apple Support (Mac User Guide), user-facing settings, macOS 14 or 15 editions where one exists:

| Page | Id | Used for |
| --- | --- | --- |
| Change Control Center settings (15.0) | mchlad96d366 | 4.3: the three module groups and their options |
| What's in the menu bar (15.0) | mchlp1446 | 4.4: Command-drag to rearrange and remove |
| Add and customize widgets (14.0) | mchl52be5da5 | 4.1: Edit Widgets, placing, sizes, iPhone widgets |
| Change Desktop & Dock settings (15.0) | mchlp1119 | 4.1, 4.5, 4.9: widget settings, recent apps, hot corners |
| Use the Dock (15.0) | mh35859 | 4.5: add, remove, rearrange, sections |
| Spotlight settings (15.0) | mchl54d95e8a; Change Spotlight preferences (12.0) mchlp2811 | 4.2: Search results checklist, Search Privacy |
| Login Items & Extensions (15.0) | mtusr003 | 4.7, 4.8: extensions by category, the Share menu checklist |
| Change Notifications settings (15.0) | mh40583 | 4.14: global and per-app settings |
| Customize the Finder toolbar | mchlp3011 | 4.6: View › Customize Toolbar, Command-drag |
| Customize the Finder sidebar | mchl83c9e8b8 | 4.12: Finder Settings › Sidebar checkboxes, drag to Favorites |
| Perform quick actions / Preview pane | mchl97ff9142, mchl1e4644c2 | 4.8: Show Preview Options, Customize Quick Actions |
| Create keyboard shortcuts for apps (13.0) | mchlp2271 | 4.11: App Shortcuts by menu title |
| Change Lock Screen settings (14.0) | mh11784 | 4.13: no widgets on the Mac lock screen |
| Set up a Focus; Calendar Focus filters | mchl613dc43f, icld13f9da17 | 4.15: Add Filter, per-app filters |

Our own: `20-SURFACES.md`, `22-SETTINGS.md` (sections 2, 3, 9), `23-WIDGETS.md`,
`13-BEHAVIOUR-menus-windows.md`, `27-HIG-PARITY.md`. Code read (read-only): sill
`crates/sill-launcher/src/{provider,ids,preview}.rs`, `sill-settings/src/{control_center,widgets,notifications}.rs`,
`sill-surfaces/src/surfaces/{bar,control_center,desktop_widgets,dock}/` (at master `0096872`:
`desktop_widgets/{saved,placement,drag,arrange,view}.rs`, `widgets/mod.rs` `size_of`),
`sill-services/src/{pins,tray,notifications}/`, `sill-launcher/src/rank/sections.rs`,
`sill-settings/tests/keys.rs`,
sill `FINDINGS.md` F270-F271; quire `crates/ds-settings/src/schema/key.rs`, and the in-progress
`crates/ds/src/widget/{contract,registry,wire}.rs` in the `widget-interface` worktree.

## 3. The surface table

"Exists" = what sill (or quire) has today. "Third party" = whether the reference lets other apps
add items. "Before/during" = before the app pass or during it (section 6).

| § | Surface | What the person picks | Reference picker | Third party | Exists in sill | Priority | When |
| --- | --- | --- | --- | --- | --- | --- | --- |
| 4.1 | Widgets (desktop, notification center) | which widgets, size, place (grid cell or list order), per-widget config | widget gallery (Edit Widgets): app list, search, live previews per size, Add; drag to place; Edit "<widget>" | yes (WidgetKit) | fixed kinds as `widgets.center`/`widgets.desktop_widgets` keys, one size per kind and host (`size_of`); cells per output in the state file `desktop-widgets.json`, dragged any time (F880-F892); no gallery | must (in progress) | first |
| 4.2 | Launcher result categories and providers | which categories show; (ours: their order); folders excluded | Settings › Spotlight › Search results checklist; Search Privacy list | yes (Core Spotlight, importers) | 9 providers, closed `ProviderKind` enum, fixed Tab order; per-provider on/off only for web, clipboard, files backend | must | before |
| 4.3 | Control center modules | which optional modules join; per module: show in the bar Always / When Active / Never | Settings › Control Center: three groups, a pop-up per row | no (controls not on macOS before 26) | 8 modules, reorderable today through `control_center.modules` (membership-only removes that); `InMenuBar::{Show,Hide}` per module, no Focus key, fixed bar order | must | before |
| 4.4 | Menu bar status items | order, remove, add back | Command-drag in the bar; each app's own setting to show its extra | yes (status items) | tray (SNI) in arrival order keyed by bus address, SNI `Id` not read; module items in `MENU_BAR_ORDER`; no reorder, no hide | must | before |
| 4.5 | Dock items and stacks | pinned apps and their order, folders/files as stacks, recent apps on/off | drag in, drag out ("Remove"), Options › Keep in Dock; Settings › Desktop & Dock | apps are items; no plug-ins | pins with order (`dock.json`, `PinEvent`), Keep in Dock, drag reorder; no stacks, no recents section | must (exists), stacks nice | before (migrate), stacks later |
| 4.6 | App toolbars | which items, order, spaces, icon/text | View › Customize Toolbar sheet; Command-drag | per app; extensions add Quick Action items | no toolbars yet | must for apps with toolbars | during |
| 4.7 | Share destinations | which extensions show in the Share menu | Settings › Login Items & Extensions › Sharing checklist | yes (share and action extensions) | none | nice | during |
| 4.8 | Quick Look and preview-pane previewers, Quick Actions | which extensions are on; per type, which preview fields | Extensions settings (Quick Look, Finder); View › Show Preview Options; Customize Quick Actions | yes (Quick Look and Finder extensions) | launcher preview pane with a closed `Preview` enum | nice | during |
| 4.9 | Hot corners | an action per corner (the reference's modifier waits on the compositor fork, G121) | Settings › Desktop & Dock › Hot Corners: a pop-up per corner | no | `hot_corners.*` keys, closed `CornerAction` | must (exists) | before (fold into actions) |
| 4.10 | Gestures | an action per gesture | Settings › Trackpad / Mouse pop-ups | no | `gestures.gesture_action_map` | must (exists) | before (fold into actions) |
| 4.11 | Keyboard shortcuts | on/off and a chord per system shortcut; per-app menu shortcuts | Settings › Keyboard › Keyboard Shortcuts: list, checkboxes, capture; App Shortcuts "+" | apps add their own menu commands | sill writes COSMIC's `custom` and `system_actions` files; no page | must (system list), nice (app shortcuts) | before (system), during (app) |
| 4.12 | Sidebars in our apps | which built-in items show, favourites and their order | Finder Settings › Sidebar checklist; drag into Favorites, out to remove | apps add locations (file providers) | none (apps not built) | must for Files, nice elsewhere | during |
| 4.13 | Lock screen | large clock, message, user list (no widgets on the Mac) | Settings › Lock Screen | no | `session.lock_clock`, `session.user_picture` | nice (scalar keys only) | before, small |
| 4.14 | Notifications per app | allow, style, lock screen, center, badge, sound, previews, grouping | Settings › Notifications: app list, one page per app | every app is a row | one global `banner_style`; per-app deferred (F270) | must | before |
| 4.15 | Focus filters | per Focus: allowed people and apps, per-app filters | Settings › Focus › a Focus › Add Filter | yes (App Intents filters) | Focus is a control center module (DND only) | skip for now (nice later) | after |

The pattern holds everywhere a row says "which" and "order": a **registry** of candidates, a
**picker**, and a **list of placements** in a file. The rows that pick one action per slot (4.9,
4.10, 4.11) are the same pattern with a slot of size one and a registry of actions.

## 4. Per surface

Each subsection has the same fields: **Reference** (what the person picks, the picker, third
party, source), **Today**, **Mapping** (owner, trait, picker, keys) and **Priority**. Every row
here is **proposed (design/28, 2026-09-27)** and is not yet a design/22 row. Two kinds of table:
**Layout entry** tables are `layout.toml` contents (decision 1, settled B) and land in design/22's
new `## 10. Layouts`, outside §3 (5.6); **Key** tables are `settings.toml` keys and land in
design/22 §3 only in the order of 6.0. Positions on a surface are state and have no row. Where a
setting belongs to an app not yet named, it is described in prose, never as a placeholder key.

### 4.1 Widgets (desktop and notification center)

**Reference (H).** People open the gallery by "Control-click the wallpaper, then choose Edit
Widgets" or "Edit Widgets" at the bottom of Notification Center (Support mchl52be5da5). The
gallery has a search field and a list of apps on the left; each app's widgets show as realistic
previews grouped by size with one description ("Group your widget's sizes together, and provide a
single description", HIG widgets). On the desktop a click places the widget automatically or a drag
places it; in Notification Center widgets are "reordered by dragging up or down". Control-click
a widget for its sizes; "Edit <widget>" edits its configuration, whose form "the system
automatically generates" (HIG widgets); Remove deletes it. Since Sonoma a desktop widget is moved
by pressing and dragging it, no edit mode needed, and it snaps to an invisible grid, pushing
others aside (sill F880, from MacMost and Intego). Settings › Desktop & Dock: Show Widgets (On
Desktop, In Stage Manager), Widget style (Automatic, Monochrome, Full-color), Use iPhone widgets
(mchlp1119). Third party: any app, through WidgetKit.

**Today (sill master 0096872, "Widget drag", F880-F892).**
- Kinds: `WidgetKind` closed enum in `sill-settings` (Calendar, UpNext, Battery, NowPlaying,
  WorldClock). Membership and order: the keys `widgets.center` and `widgets.desktop_widgets`
  (22 §3.20). The size is not chosen: `sill-surfaces/src/widgets/mod.rs` `size_of(kind, host)`
  fixes one size per kind and host (every notification center tile Medium, the month Large there;
  on the desktop Calendar and Battery Small, Up Next, Now Playing and World Clock Medium).
- Positions: **state**, `$XDG_STATE_HOME/sill/desktop-widgets.json`, no settings key (F881, and
  22 §1 rule 3). Each entry is a `saved::SavedEntry { output: OutputKey, placement: Placement {
  kind, slot: GridSlot { column, row } } }`; `OutputKey::{Any, Named(connector)}`. An output with
  entries of its own reads only those; one with none reads the `Any` entries, so a pre-F881 file
  carries over; saving an output replaces its own entries and keeps the others (`with_output`).
- Output changes: a saved place is clamped into the grid (`placement::clamp`, `nearest_free`),
  and a clamped place is **never written back** (`view::recorded`): only a widget seen for the
  first time is recorded, and a drop records the arrangement dropped, so the place comes back
  with the output (F882).
- Drag, always on: `desktop_widgets/drag.rs` `Hold::{Idle, Pressed, Live}`; only Manhattan travel
  past `window.move_threshold_px` (4) makes a press `Live`, so a click still reaches a widget's
  control (F883, F887); `arrange.rs` `snap` and `make_room` push covered widgets to their nearest
  free spot and back (F884); the motion is quire's `use_tween` in `placed.rs` (F885).
- Scalars: `widgets.desktop` (the layer on or off), `widgets.desktop_cell_px`,
  `widgets.desktop_gap_px`, `widgets.world_clocks` (the zones, `;`-separated, read by
  `widgets/world_clock.rs`).

quire (`widget-interface`, in progress): `ds::widget::Widget` trait (`Entry`, `Intent`, `kind`,
`name`, `sizes`, `placeholder`, `view`, `title`), `WidgetKind` as a reverse-DNS string,
`WidgetRegistry` with `WidgetInfo::of::<W>()` and a type-erased `preview`, `wire.rs` for
timelines from another process.

**Mapping.** Owner: quire (the trait, the card, the registry, the gallery), sill (the hosts:
desktop layer and notification center; the providers that feed entries; the state file).

- Trait: `Widget` as built; add `type Config: Default + Serialize + DeserializeOwned +
  SettingsSchema` so the edit form is generated from the config struct by the same derive the
  Settings app uses (22 §9.1), matching "the system automatically generates it".
- **Membership, order and config** are `layout.toml` (5.6): `layout.center_widgets` (ordered)
  and `layout.desktop_widgets` (membership only). Desktop membership is **global**: one list for
  every output, as `widgets.desktop_widgets` is today.
- **Cells stay state**, in `desktop-widgets.json`, per output, unchanged in meaning; the entry
  gains the placement's `instance` (5.2) so two widgets of one kind keep separate cells; an
  entry whose instance is no longer in `layout.desktop_widgets` is ignored and dropped on the
  next save of that output. F882's rule stands: a clamped place is never written back.
- **Size**: one size per widget per host (the user, 2026-09-28: "no different sizes, just one";
  built as `Widget::size_in` / `WidgetInfo::size_in` / `WidgetRegistry::sized`, design/23 section
  9.4; the gallery offers no size, sill Q520). `Placement.shape` is `None` by default, meaning the host's size for the kind
  (`size_of(kind, host)`, which moves with the kinds into the registry as `KindInfo::size_in(host)`).
  A shape is written only when the person picks one from the widget's context menu, and only a
  shape the kind lists for that host is offered; a stored shape the kind no longer offers falls
  back to `size_in(host)`. Today every kind has one size per host, so no shape is ever written
  until a kind offers two.
- **Instances**: two Batteries (one showing this computer, one the headphones) are two
  placements of `quire.battery` with different `instance` ids and configs.
- What becomes of today's keys: `widgets.desktop` stays a scalar key (the reference's "Show
  Widgets"); `widgets.desktop_cell_px` and `widgets.desktop_gap_px` stay scalar keys (the grid's
  geometry, not a placement); `widgets.world_clocks` becomes the World Clock instance's config
  (`zones: Vec<Zone>`), migrated once from the key into every World Clock placement, after which
  sill drops the key (row order, section 6.0); `widgets.center` and `widgets.desktop_widgets`
  move to the layout and are dropped the same way.
- Picker: `KindGallery` (5.4) in a panel from the desktop's context menu and the notification
  center's "Edit Widgets" button. Dragging on the desktop needs no mode (F883); `EditMode` only
  adds the remove badges and the size choices.

| Layout entry (design/22 §10 when it lands) | Type | Default | Range / Alt | Source | Status |
| --- | --- | --- | --- | --- | --- |
| `layout.center_widgets` | `Vec<Placement>`, ordered | Calendar, Up Next, Batteries (today's `widgets.center`) | order is the list's; shape `None` = `size_in(Tile)` | 4.1; 22 §3.20 | proposed (design/28) |
| `layout.desktop_widgets` | `Vec<Placement>`, membership | Calendar, Batteries (today's `widgets.desktop_widgets`) | global; cells per output stay in `desktop-widgets.json` (state) | 4.1; F881 | proposed (design/28) |

| Key | Type | Default | Range / Alt | Source | Status |
| --- | --- | --- | --- | --- | --- |
| `widgets.style` | `WidgetStyle::{Automatic,Monochrome,FullColor}` | `Automatic` | the reference's Widget style; Automatic recedes while an app is active | mchlp1119 | proposed (design/28) |

**Priority.** must; first (it is the pattern's first instance and is being built).

### 4.2 Launcher result categories and providers

**Reference (H, list L).** Settings › Spotlight › "Search results: Select the categories that
Spotlight includes or excludes when it searches"; "Search Privacy: Exclude specific files and
folders"; "By default, Spotlight results include Siri Suggestions" (mchl54d95e8a, mchlp2811).
The picker is a **checklist**; in the Sonoma/Sequoia era the order is **not** editable (the
Support pages describe no ordering, L: dragging categories to reorder existed in older OS X and
was removed). The categories (L, from the Sequoia pane): Applications, Bookmarks & History,
Calculator, Contacts, Conversion, Definition, Developer, Documents, Events & Reminders, Folders,
Fonts, Images, Mail & Messages, Movies, Music, Other, PDF Documents, Presentations, Siri
Suggestions, Spreadsheets, System Settings, Tips, Websites. Third party (H, HIG searching): apps
"make your app's content searchable" through Core Spotlight and "Supply a Spotlight File Importer
plug-in" for custom file types; their items then show under the matching category or the app's
own heading.

**Today (sill master 0096872).** `sill_launcher::Provider` (`kind`, `query(&Query, &mut dyn
ResultSink)`, `activate(&Item, &Choice) -> Activation`), nine providers in the closed
`ProviderKind` enum whose order is the Tab order and the tie-break (`ids.rs`). The results are
placed in sections by `rank/sections.rs`: Top Hit first (typed text; with nothing typed, Actions
and Recent), then one section per kind, Web "always last". Three providers already have a
switch the user settled (22 §3.6): `launcher.web_search` (On/Off), `launcher.clipboard_history`
(Memory/Off; Off watches nothing), `launcher.files_backend` (Auto/Tracker/Baloo/Fd/Off). The user
named this surface.

**Mapping.** Owner: sill (`sill-launcher` keeps the trait; the registry of providers is sill's,
non-UI); quire owns the `OrderedChecklist` picker and the `KindId` type. Trait change:
`fn kind(&self) -> ProviderKind` becomes `fn info(&self) -> ProviderInfo { id: KindId, name,
icon, section: SectionTitle, default_shown: Shown }`, so a provider from another process
(section 5.7) is a row like ours; `ProviderKind` stays as the ids of the built-in nine
(`quire.launcher.apps`, ...). Picker: `OrderedChecklist` on the Settings app's Spotlight page (22
§5 already names the page), plus a "Search Privacy" `List` of folders.

What the person orders (settled 2026-09-27, decision 2 B), and what stays fixed:
- **Reorderable**: the kind sections between Top Hit and Web (Applications, Documents, Windows,
  Calculator, Settings, Clipboard, Emoji, System, and any app's provider); their order is the
  section order and the Tab order.
- **Fixed**: Top Hit (and, with nothing typed, Actions and Recent) stays first; **Web stays pinned
  last** as the fallback row and has no drag handle; the ranking inside a section, the frecency
  tie-break and how rows compete for Top Hit (`SectionPlan` weights) do not follow the list.
- A provider absent from the list and new since the last write is appended before Web with its
  `default_shown` (so a new app's provider appears, as a new category does on the reference).

**One switch per provider, and which one is the source.** For Web, Files and Clipboard the
settled key **is the source** and the checklist row is a view of it: the layout entry for those
three carries no `shown`. Unchecking Web sets `launcher.web_search = Off`; unchecking Clipboard
sets `launcher.clipboard_history = Off`, which also stops the clipboard watcher; unchecking Files
sets `launcher.files_backend = Off`, which stops asking any index; checking Files again sets
`Auto` (a named backend is chosen in the Advanced list, not by the checkbox). For every other
provider the layout entry's `shown` is the source; Off means the provider's `query` is never
called.

| Layout entry (design/22 §10 when it lands) | Type | Default | Range / Alt | Source | Status |
| --- | --- | --- | --- | --- | --- |
| `layout.launcher_categories` | `Vec<Placement>`, ordered, with `shown` | the eight non-Web providers in `ProviderKind::ALL` order, all On | Web is not listed (pinned last); Web, Files and Clipboard take `shown` from their keys | 4.2; mchl54d95e8a | proposed (design/28) |

| Key | Type | Default | Range / Alt | Source | Status |
| --- | --- | --- | --- | --- | --- |
| `launcher.excluded_folders` | `Text` | `""` | folders separated by `;` (as `calendar.sources`); the Files provider and its index skip them | Search Privacy, mchlp2811 | proposed (design/28) |

**Priority.** must; before the app pass (a checklist over the nine), with third-party providers
landing during the app pass (Mail and Notes search, section 6).

### 4.3 Control center modules and their menu bar presence

**Reference (H).** Settings › Control Center (mchlad96d366, 15.0), three groups:
- **Control Center Modules**, always in Control Center: Wi-Fi, Bluetooth, AirDrop, Focus, Stage
  Manager, Screen Mirroring, Display, Sound, Now Playing. Per row a pop-up: "Show in Menu Bar",
  "Show When Active" (Focus, Screen Mirroring, Display, Sound, Now Playing only), "Don't Show in
  Menu Bar".
- **Other Modules**, optional: Accessibility Shortcuts, Battery, Music Recognition, Hearing, Fast
  User Switching, Keyboard Brightness. Per row two toggles: "Show in Menu Bar", "Show in Control
  Center"; Battery adds "Show Percentage"; Fast User Switching shows Full Name, Account Name or
  Icon.
- **Menu Bar Only**: Clock (always), Spotlight, Siri, Time Machine, VPN, Weather: Show / Don't
  Show.
The grid's arrangement is fixed; the person picks membership and bar presence, not positions.
Third party: none; HIG controls: "Not supported in macOS" in this era (Tahoe's editable Control
Center is the Liquid Glass era and out of scope, `27-HIG-PARITY.md#04-what-the-june-2025-redesign-changed-which-we-do-not-adopt`).

**Today (sill master 0096872).** `ControlCenterModule` has eight variants (Wifi, Bluetooth,
Focus, Display, Sound, NowPlaying, Appearance, Battery); `control_center.modules` is
`Vec<ControlCenterModule>` ("Which modules show, in order"), so the person can reorder the
modules today by editing it. `control_center.menu_bar_{wifi,bluetooth,sound,display,battery,now_playing}`
are `InMenuBar::{Show,Hide}`; there is no Focus key (`in_menu_bar` returns `Hide` for Focus and
Appearance); `MENU_BAR_ORDER` is fixed in `control_center/menu_bar.rs`. In `control_center/modules/`,
`wired.rs` is not a module: it wires the Sound and Display modules to their services (and feeds
the bar's Sound and Display dropdowns); `power_profile.rs` is `PowerProfileControl`, a
`SegmentedControl` drawn inside the Battery module, not a module of its own. A keyboard-brightness
module does not exist yet: it arrives with design/26's wave D2 (G25).

**Mapping.** Owner: quire (the `ModuleTile`/`ModulePanel` contract as a trait, so a module's
tile, panel and bar glyph come from one implementation), sill (the modules, which need
services). Trait (quire, UI side): `ControlModule { type State; fn info() -> ModuleInfo { id,
name, glyph, span: TileSpan, group: Option<ModuleGroup::{Core,Other}>, bar: BarPresence
capabilities }; fn tile(state) -> Element; fn panel(state) -> Option<Element>; fn bar_glyph(state)
-> Glyph; fn active(state) -> Active }` (`active` drives "When Active"). Groups: Core = Wi-Fi,
Bluetooth, Focus, Display, Sound, Now Playing (always shown); Other = Battery and, from D2,
Keyboard brightness (the person adds or removes them); **Appearance has no group**: it is ours,
not a reference module, always shown, never a bar item. Picker: the Settings app's Control
Center page, one `SettingsRow` per module with a pop-up for the bar presence and a toggle for
"Show in Control Center" on Other modules: the reference's exact shape, no gallery.

**Membership only (settled 2026-09-27, decision 5 A).** This removes a freedom sill has today:
reordering the modules through `control_center.modules`. The order becomes the design's, fixed
in code (design/13 §13.3.7's order); the person picks only whether each Other module shows. sill
drops `control_center.modules` (row order, section 6.0) when `layout.control_center` lands.

| Layout entry (design/22 §10 when it lands) | Type | Default | Range / Alt | Source | Status |
| --- | --- | --- | --- | --- | --- |
| `layout.control_center` | `Vec<Placement>`, membership | Battery shown | holds only Other modules (Battery; Keyboard brightness from D2) with `shown`; Core modules and Appearance are always shown and not listed; order is not the person's | 4.3; 22 §3.13 | proposed (design/28) |

| Key | Type | Default | Range / Alt | Source | Status |
| --- | --- | --- | --- | --- | --- |
| `control_center.menu_bar_focus` | `InMenuBar::{Show,WhenActive,Hide}` | `WhenActive` | new: Focus as its own bar item while a Focus (Do Not Disturb) is on | mchlad96d366 | proposed (design/28) |
| `control_center.menu_bar_now_playing` | `InMenuBar::{Show,WhenActive,Hide}` | `WhenActive` | was `Hide` with two variants | mchlad96d366; 22 §3.13 | proposed (design/28), amends 22 §3.13 |

`InMenuBar` gains `WhenActive` for every existing `menu_bar_*` key; it is offered in the picker
only where the module reports `Active` (Focus, Display, Sound, Now Playing), and the other keys'
defaults are unchanged. The keys stay scalar keys (a closed choice per known module); their bar
**order** moves to 4.4.

**Priority.** must; before the app pass.

### 4.4 Menu bar status items and their order

**Reference (H).** "To rearrange status menus, press and hold the Command key while you drag an
icon"; "To quickly remove a status menu, press and hold the Command key while you drag the icon
out of the menu bar" (mchlp1446). HIG the-menu-bar: "Let people — not your app — decide whether
to put your menu bar extra in the menu bar"; the system may hide extras when space is short;
Clock is essential and fixed. Third party: yes, any app's status item.

**Today (sill master 0096872).** Tray items (StatusNotifierItem) in arrival order, each keyed by
`TrayItemId`, which is the item's **bus address** (`<unique name><object path>`,
`sill-services/src/tray/model.rs`): it changes every time the app restarts. sill does not read
the SNI `Id` property today. Control center module items follow `MENU_BAR_ORDER`; the control
center item and the clock sit at the right end.

**Mapping.** Owner: sill (the bar and its item kinds), quire (the reorder machine, 5.8, and the
drag ghost). Trait: none new; a bar item is `BarItem { id: KindId, source: BarSource::{Module(ControlCenterModule),
Tray(sni_id), Fixed(ControlCenter | Clock)} }`. **Order and hiding are keyed by the SNI `Id`**
(the app's own stable name, e.g. `nm-applet`), prefixed `sni.`, never by `TrayItemId`; two items
with the same `Id` fall back to arrival order among themselves. P4 adds reading `Id` to the tray
service (`TrayItem.sni_id`). Picker: in place, **Command-drag** (Super on our keyboards through
the Mod layer, 27 §6.2) past `window.move_threshold_px` reorders; dropping outside the bar hides
a module item (sets its `menu_bar_*` key to `Hide`) or hides a tray item (adds its id to
`layout.bar_hidden`); the control center item and the clock do not move (the reference's
Control Center and Clock are fixed at the right end). Hidden tray items come back from the
Settings app's Control Center page, a "Menu Bar Only"-style list.

| Layout entry (design/22 §10 when it lands) | Type | Default | Range / Alt | Source | Status |
| --- | --- | --- | --- | --- | --- |
| `layout.bar_items` | `Vec<Placement>`, ordered | `[]` (empty = today's order: tray, then `MENU_BAR_ORDER`) | items not in the list take their default place, left of the listed ones (new items appear at the left end, as the reference adds new extras) | mchlp1446 | proposed (design/28) |
| `layout.bar_hidden` | `Vec<KindId>` | `[]` | `sni.<Id>` of the tray items the person dragged out | mchlp1446 | proposed (design/28) |

**Priority.** must; before the app pass.

### 4.5 Dock items and stacks

**Reference (H).** "Drag apps to the left side of (or above) the line that separates the
recently used apps", files and folders to the right; "Drag the item out of the Dock until Remove
is shown"; Control-click › Options › Keep in Dock; drag to rearrange (mh35859). Sections: apps,
recent apps (Settings: "Show suggested and recent apps in Dock", mchlp1119), files, folders and
stacks, Trash. A folder in the Dock is a stack (Display as Folder/Stack, View content as
Fan/Grid/List/Automatic, Sort by: L, from its context menu). Third party: apps are items; no
plug-ins; an app adds items to its own Dock menu (HIG dock-menus).

**Today.** sill: `DockPins { version, pinned: Vec<AppId> }` in `dock.json`, `PinEvent::Pin {
app, at: PinAt::{End,Index} }` and unpin, applied by one writer task (`pins::run`: load once,
reduce, save atomically, watch for outside edits); `sill dock pin` and Keep in Dock send it
events. Drag reorder and drag-out remove live in the dock machine (`dock/machine/press.rs`),
always on, no mode. Stacks deferred (`10-BEHAVIOUR-dock.md` R25).

**Mapping.** Owner: sill (pins service, dock), quire (`DockParts`, drag ghost). It already is the
pattern with a fixed kind (an app). Change: pins become an ordered `Vec<Placement>` whose `kind`
is `quire.dock.app` or `quire.dock.stack` and whose `config` carries the `AppId` or the folder
path and the stack's view options; `dock.json` migrates once (read old, write new, leave the old
file, as the mailo appearance migration in 22 §2); `pins::run` stays its one writer (rule
5.2.4). Picker: in place (drag), no gallery.

| Layout entry (design/22 §10 when it lands) | Type | Default | Range / Alt | Source | Status |
| --- | --- | --- | --- | --- | --- |
| `layout.dock` | `Vec<Placement>`, ordered | today's `dock.json` pins | kinds `quire.dock.app` (config `app`), `quire.dock.stack` (config `path`, `display: StackDisplay::{Stack,Folder}`, `view: StackView::{Automatic,Fan,Grid,List}`, `sort: StackSort::{Name,Added,Modified,Created,Kind}`) | mh35859; 10 §10.6 | proposed (design/28) |

| Key | Type | Default | Range / Alt | Source | Status |
| --- | --- | --- | --- | --- | --- |
| `dock.recent_apps` | `RecentApps::{Show,Hide}` | `Hide` | the reference's section between apps and stacks | mchlp1119 | proposed (design/28) |

**Priority.** must (exists; migrate to the shared shape before the app pass); stacks nice,
after.

### 4.6 App toolbars (Customize Toolbar)

**Reference (H).** "In iPadOS and macOS apps, consider letting people customize the toolbar"
(HIG toolbars); "Make every toolbar item available as a command in the menu bar. Because people
can customize the toolbar or hide it, it can't be the only place that presents a command". The
picker: View › Customize Toolbar opens a sheet of every item plus a default set; people "drag
items into and out of the toolbar, add a space between items, and choose whether to show text
with the icons"; outside the sheet, Command-drag rearranges, adds and removes (mchlp3011).
Third party: per app; Finder's Quick Action extensions can appear as toolbar items (L).

**Today.** No toolbars. 27 §5.11's rule: `ToolbarItem { command: CommandId }`, so no
toolbar-only action exists (wave H4, `MenuModel`).

**Mapping.** Owner: quire (the `Toolbar` component, the `ToolbarItem` registry, the customize
sheet); each app registers its items. Trait: `ToolbarItem { fn id() -> KindId; fn label() ->
Text; fn palette_label() -> Text; fn glyph() -> Icon; fn command() -> CommandId; fn width() ->
ItemWidth::{Fixed,Flexible} }` plus built-ins `Space`, `FlexibleSpace`, `Separator`. Picker:
`CustomizePalette` sheet (the reference's sheet: a grid of every item, "or drag the default set",
Show: Icon and Text / Icon Only / Text Only) and Command-drag in place. The layout is per app:
each app with a toolbar keeps a `[[toolbar]]` array (an ordered `Vec<Placement>`, kinds from that
app's registry, default the app's default set) in its own `layout.toml`, and a `toolbar_display`
key (`ToolbarDisplay::{IconAndText,IconOnly,TextOnly}`, default `IconOnly`) in its own settings
file; the concrete rows are written with the app (Files first), not before.

**Priority.** must for apps with toolbars (Files, Mail, Photos); during the app pass, after H4.

### 4.7 Share destinations

**Reference (H).** "Even though macOS doesn't provide an activity view, you can create share and
action app extensions that people can use on a Mac"; "people access share extensions by clicking
a Share button in the toolbar or choosing Share in a context menu" (HIG activity-views).
Settings › General › Login Items & Extensions lists extensions by category; people "Select the
sharing extensions to include in the Share menu"; "Default extensions, such as Mail and AirDrop,
can't be turned off" (mtusr003). The Share menu's own "Edit Extensions…" item opens that list (L).
Third party: yes.

**Today.** None. No freedesktop standard exists for share targets (KDE's Purpose framework is a
library, not a protocol).

**Mapping.** Owner: quire (the `ShareMenu` component and the `ShareTarget` trait, since every app
shows the same menu), sill (none). Trait: `ShareTarget { fn info() -> ShareInfo { id, name,
icon, accepts: Vec<Mime> }; fn share(items: &[ShareItem]) -> ShareOutcome }`; built-ins: Mail
(compose with attachments), Copy, Open With (per `accepts`), Save to Files; apps in another
process register through the manifest (5.7). Picker: `OrderedChecklist` (on/off only, fixed
system rows first) on the Settings app's Extensions page, and "Edit Extensions…" at the end of
every Share menu.

In `quire/layout.toml` (every app reads it):

| Layout entry (design/22 §10 when it lands) | Type | Default | Range / Alt | Source | Status |
| --- | --- | --- | --- | --- | --- |
| `layout.share_menu` | `Vec<Placement>`, ordered, with `shown` | built-ins On, others On when installed | Mail and Copy cannot be turned off | mtusr003 | proposed (design/28) |

**Priority.** nice; during the app pass (Files, Mail, Photos have the Share button).

### 4.8 Quick Look and preview-pane previewers, Quick Actions

**Reference (H).** HIG searching: "Implement a Quick Look generator if your app produces custom
file types". Extensions settings list Quick Look and Finder extensions to turn on or off
(mtusr003). Finder's preview pane: "View › Show Preview Options, then select the checkboxes for
the options you want to show for the file you selected (available options depend on the file
type)"; "Customize" under Quick Actions opens Extensions settings (mchl1e4644c2, mchl97ff9142).
Third party: yes.

**Today.** sill launcher: `Preview` closed enum (None, Image, Text, text file, PDF, ...) decided
by `preview_for` from a row; quire `PreviewPane`, `PreviewContent`, `PdfThumb`. design/20 §2.4
plans Quick Look as a standalone previewer service.

**Mapping.** Owner: quire (the `Previewer` trait and the pane; the Quick Look app and the
launcher are both hosts), sill (the launcher host). Trait: `Previewer { fn id() -> KindId; fn
accepts() -> &'static [MimePattern]; fn load(path) -> PreviewData (off the UI thread); fn
view(&PreviewData, PreviewPlace) -> Element; fn fields() -> &'static [PreviewField] }`. The
registry resolves a MIME type to the most specific previewer (exact type, then `image/*`, then
the fallback icon-and-facts). Picker: the Extensions page's checklist (on/off per previewer) and
the pane's own "Show Preview Options" checklist per type. Out of process: the freedesktop
**Thumbnail Managing Standard** (`.thumbnailer` files under `$XDG_DATA_DIRS/thumbnailers`) is
read for thumbnails, which gives many file types without our own code.

In `quire/layout.toml`:

| Layout entry (design/22 §10 when it lands) | Type | Default | Range / Alt | Source | Status |
| --- | --- | --- | --- | --- | --- |
| `layout.previewers` | `Vec<Placement>`, ordered, with `shown` | all On | order breaks ties between two previewers of one type | mtusr003 | proposed (design/28) |

The per-type preview options (nice) would be one key per file type naming the fields shown
(`Vec<PreviewField>`; image: dimensions, colour space, created; document: pages, created,
modified); their names are written when the Quick Look app lands.

**Priority.** nice; the trait during the app pass (Files, Quick Look app); the options per type
skip until someone asks.

### 4.9 Hot corners

**Reference (H/L).** Settings › Desktop & Dock › Hot Corners: a pop-up per corner, "Modifier keys
(Command, Shift, Option, Control) can be combined with hot corner actions" (mchlp1119). The
actions (L): Mission Control, Application Windows, Desktop, Notification Center, Launchpad,
Quick Note, Start Screen Saver, Disable Screen Saver, Put Display to Sleep, Lock Screen, none.

**Today.** `hot_corners.*` keys with the closed `CornerAction` enum and `*_command` text keys
(22 §3.23); design/13 §13.3.12. The modifier key was retired on 2026-09-26 (sill FINDINGS
"hot_corners.modifier retired"): a keyboard-less layer surface is never sent
`wl_keyboard.modifiers`, so a corner cannot know what is held (shell-host F61, sill G121); the
wish is a fork item in `sill/docs/cosmic-gaps.md`.

**Mapping.** Owner: sill (actions are shell actions). It is the pattern with one slot per corner
and a registry of **actions**; the action registry is shared with 4.10 and 4.11 (one
`ShellAction` registry: `id`, `name`, `glyph`, `run`), so an app's action (Quick Note from Notes,
a Mail "New Message") can be a corner, a gesture or a shortcut without three lists. Picker: a
pop-up per slot (`KeyKind::Menu`, 22 §9.1). No settings row changes. **The modifier waits on the
compositor fork** (cosmic-gaps, G121): when the compositor can report held modifiers to a
keyboard-less surface, a per-corner modifier returns as a key; until then none is proposed.

`CornerAction` stays the closed set until the action registry exists; then its variants become
the built-in action ids and `Command` stays as the escape hatch.

**Priority.** must (exists); fold into the action registry before the app pass (small); the
modifier waits on the fork.

### 4.10 Gestures

**Reference (L).** Settings › Trackpad / Mouse: per gesture a checkbox and a pop-up of variants
(swipe between pages with one or two fingers, and so on). The reference's choice set is closed
per gesture.

**Today.** `gestures.gesture_action_map: Map<Gesture, Action>` in `palmrest/gestures.toml`
(22 §3.9; 12 §12.6).

**Mapping.** Owner: palmrest (the map), sill (the action registry, 4.9). The map's values become
`ShellAction` ids; no other change. Picker: the Mouse & Gestures page's remap table (exists in
22 §5).

**Priority.** must (exists); fold into actions with 4.9.

### 4.11 Keyboard shortcuts

**Reference (H).** Settings › Keyboard › Keyboard Shortcuts: a sidebar of groups (Launchpad &
Dock, Display, Mission Control, Keyboard, Input Sources, Screenshots, Presenter Overlay, Services,
Spotlight, Accessibility, App Shortcuts, Function Keys, Modifier Keys; L), each a list of rows
with a checkbox and a captured chord. App Shortcuts: "+", pick an app, type the menu title
"exactly as the command appears in the app, including the > character (type ->)"; "You can
create keyboard shortcuts only for existing menu commands" (mchlp2271). HIG keyboards: respect
standard shortcuts, custom ones only for frequent commands.

**Today.** Global keys belong to cosmic-comp. sill ships and writes COSMIC's shortcut files,
`com.system76.CosmicSettings.Shortcuts/v1/custom` and `.../system_actions` (under `dist/cosmic`;
sill FINDINGS: the launcher, screenshots, the switcher, brightness and lock are bound there).
quire has `StandardAction` and `Shortcut::custom` refusing reserved chords (27 §6.2, H0).

**Mapping.** Owner: sill (system shortcuts are `ShellAction` bindings, written to COSMIC's two
files), quire (`MenuModel` commands for App Shortcuts, `KeyKind::Shortcut` capture field). The
registry is the action registry of 4.9 plus every app's `MenuModel` commands. Picker: the
Settings app's Keyboard Shortcuts page, a `SettingsRow` list per group with a toggle and a
capture field; App Shortcuts as a `List` of `{app, menu_path, chord}` rows.

**Who wins a conflict (proposed, open decision 10).** COSMIC's files are the live truth for
global keys, and the person or COSMIC Settings may edit them behind sill's back. So: **the chord
COSMIC holds wins**. sill writes a binding only to a chord COSMIC does not already hold for
another action; when `shortcuts.bindings` asks for a chord COSMIC holds (a clash), sill leaves
COSMIC's binding, keeps its own row, and the Settings app shows the clash on that row (a
warning glyph and "Used by <action> in COSMIC") until the person picks another chord or frees it.
sill rereads both files on change (directory watch, 22 §2) so the page never shows a stale
chord.

| Key | Type | Default | Range / Alt | Source | Status |
| --- | --- | --- | --- | --- | --- |
| `shortcuts.bindings` | `Vec<Binding { action: KindId, chord: Shortcut, shown: Shown }>` | the reference's bindings for the actions we have | `Shortcut::custom` rules apply; a clash with COSMIC is shown, not written | mchlp2271; 27 §6.2 | proposed (design/28) |
| `shortcuts.app_shortcuts` | `Vec<AppShortcut { app: AppId, menu_path: Text, chord: Shortcut }>` | `[]` | the menu path in `File->Export as PDF…` form | mchlp2271 | proposed (design/28), nice |

Both are behaviour, not placement: they are `sill/settings.toml` keys (a new §3 domain in design/22,
added after sill registers them or lists them in `AWAITING_ROWS`, section 6.0), and they need the
record list kind of P5.

**Priority.** must for the system list (before the app pass), nice for App Shortcuts (during,
after H4 `MenuModel`).

### 4.12 Sidebars in our apps

**Reference (H).** HIG sidebars: "When possible, let people customize the contents of a sidebar
... it works well when people can decide which areas are most important and in what order they
appear." Finder: Settings › Sidebar has checkboxes for the default items ("These checkboxes are
for default items, not any custom item that you might have added"); a folder dragged to
Favorites is added; "drag the icon for the item out of the sidebar until you see the remove
sign"; a section heading hides or shows its items (mchl83c9e8b8). Mail: Favorites bar and
mailbox list (L). Third party: file providers add Locations (mtusr003's File Providers category).

**Today.** quire `SidebarItem`, `TreeItem`; the Arc-lineage sidebar (01 §4, 09 H2) with pinned
tiles. No app has a customizable sidebar yet.

**Mapping.** Owner: quire (a `Sidebar` host with `SidebarSection` kinds and the drag-in/drag-out
interaction), each app (its item kinds: Files places and tags; Mail mailboxes; Notes folders).
Trait: `SidebarSource { fn id() -> KindId; fn section() -> SectionTitle; fn items(&self) ->
Vec<SidebarEntry> }` so a file provider or an account adds a section. Picker: the app's Settings
› Sidebar `OrderedChecklist` (built-ins), plus drag in place for favourites.

Each app with a sidebar keeps a `[[sidebar]]` array in its own `layout.toml`: an ordered
`Vec<Placement>` with `shown`, default the app's items; kinds are the app's built-in items (one
id each) and `favourite` (config `path` or `mailbox`). The concrete rows are written with the
app (Files first).

**Priority.** must for Files, nice elsewhere; during the app pass.

### 4.13 Lock screen

**Reference (H).** Settings › Lock Screen: screen saver and display-off timings, require password,
"Show large clock", "Show password hints", "Show message when locked", login window shows list of
users or name and password, Sleep/Restart/Shut Down buttons (mh11784, 14.0). **No widgets** on
the Mac lock screen in this era (the page names none; iPhone and iPad Lock Screen widgets and
controls do not apply).

**Today.** `session.lock_clock`, `session.user_picture` (22 §3.19); quire `LockScreen`.

**Mapping.** Scalar keys only; no registry. Owner: sill.

| Key | Type | Default | Range / Alt | Source | Status |
| --- | --- | --- | --- | --- | --- |
| `session.lock_message` | `Text` | `""` | shown under the clock when not empty | mh11784 | proposed (design/28), nice |
| `session.lock_large_clock` | `LargeClock::{Show,Hide}` | `Show` | | mh11784 | proposed (design/28), nice |

**Priority.** nice (two keys); lock screen widgets skip.

### 4.14 Notifications per app

**Reference (H).** Settings › Notifications: global "Show previews" (Always, When Unlocked, Never),
allow when the display is sleeping, when locked, when mirroring; then an app list, one page per
app: Allow notifications; alert style None / Banners / Alerts; Allow critical alerts; Allow time
sensitive alerts; Show on lock screen; Show in Notification Center; Badge application icon; Play
sound; Show previews (Default, Always, When Unlocked, Never); Notification grouping (Automatic,
By Application, Off) (mh40583). HIG notifications: "People can choose whether to allow an app to
display badges in their notification settings." Every app that has ever notified is a row.

**Today.** sill: one global `notifications.banner_style`; per-app deferred (F270, F271: "a table
key needs a design/22 shape for a map first"). This document gives that shape.

**Mapping.** Owner: sill. The registry is the set of apps seen by the notification server (by
desktop-entry id from the `desktop-entry` hint, else the app name), recorded on first
notification so the Settings page lists them. No trait. Picker: the Settings app's Notifications
page, `SettingsRow` list of apps (icon, name, current style as the trailing value), each opening a
pane of the per-app keys.

**The seen apps are state, not a key**: `$XDG_STATE_HOME/sill/notifying-apps.json`, a list of
`{app, name, first_seen}`, written by the notifications task. It has no design/22 row. It does
not reuse the history file (`notifications.json`, `history_file.rs`) because the history is
capped (`notifications.history_cap`, which may be 0), is emptied when the person clears the
notification center, and holds the notifications' content; the app list must outlive all three
and must not keep content only to remember that an app exists.

| Key | Type | Default | Range / Alt | Source | Status |
| --- | --- | --- | --- | --- | --- |
| `notifications.apps` | `Vec<AppNotify>`; each `{ app: AppId, allow: Allow::{On,Off}, style: BannerStyle::{None,Banner,Alert}, lock_screen: Shown, center: Shown, badge: Shown, sound: Shown, previews: Previews::{Default,Always,WhenUnlocked,Never}, grouping: Grouping::{Automatic,ByApp,Off} }` | `[]` (an app without a row uses the defaults: allow, `banner_style`, all Shown, `Default`, `Automatic`) | an array of tables in `sill/settings.toml` (behaviour, not placement); needs P5's record list kind | mh40583; F270 | proposed (design/28) |
| `notifications.previews` | `Previews::{Always,WhenUnlocked,Never}` | `Always` | the global default the per-app `Default` follows | mh40583 | proposed (design/28) |

**Priority.** must; before the app pass.

### 4.15 Focus filters

**Reference (H).** A Focus (Do Not Disturb, Work, Sleep, ...) has allowed people and apps and
Focus filters: "Add Filter", then per app (Calendar: which calendars; Mail: which accounts;
Messages: filter by people list; Safari: Tab Groups) and system filters (Appearance, Low Power
Mode, L) (mchl613dc43f, icld13f9da17). HIG managing-notifications: people identify "the contacts
and apps that can break through a Focus". Third party: yes (apps declare filters).

**Today.** sill: a Focus module showing DND only; no Focus modes.

**Mapping.** Would be the pattern again (a registry of `FocusFilter` kinds that apps provide, a
placement list per Focus). Recorded, not specified: it needs Focus modes first.

**Priority.** skip for now; nice after Mail and Calendar exist.

### 4.16 Also considered, not this pattern

| Candidate | Why not here |
| --- | --- |
| Default apps ("Open With", default browser) | one choice per MIME type, the xdg `mimeapps.list` already stores it; a Settings page, no registry |
| Login items | a list of apps with on/off; the XDG autostart directory is the store; nice, Settings page only |
| Services menu | removed from our scope with the App menu rule (27 §5.2 c) |
| Wallpaper, Spaces look | a picker but one value per output or Space: 21-SPACES owns it |
| Menu bar auto-hide | a scalar key (`bar.autohide`), not a placement |
| Stage Manager | not adopted |

## 5. The shared pattern

### 5.1 Vocabulary

| Word | Meaning |
| --- | --- |
| **Kind** | a thing the person can place: a widget, a launcher provider, a module, a toolbar item, a share target, a previewer, an action. Named by a `KindId`. |
| **KindId** | a reverse-DNS string, unique across all apps: `quire.battery`, `quire.launcher.apps`, `org.example.weather.today`. Today's `ds::widget::WidgetKind`, generalised. |
| **Host** | a surface that shows placed kinds: the desktop, the notification center, the bar, a toolbar. One host has one layout. |
| **Registry** | every kind a host can place, with what a picker needs to show it without knowing its type. |
| **Placement** | one placed instance of a kind in a host: kind, instance, shape (optional), shown, config. Membership, order and config; never a position. Data. |
| **Layout** | a host's ordered list of placements, in `layout.toml`. |
| **Cell** | where one instance sits on one output (or on every output); state, in `$XDG_STATE_HOME`. |
| **Picker** | the UI that adds, removes, orders and configures placements. |

### 5.2 `KindId`, `Shape`, `Placement`, `Layout`, `Cell` (data; crate `ds-settings`, no UI)

Non-UI crates (sill-launcher, sill-services, palmrest) need these, so they live beside the
settings loader and schema in `ds-settings` (decision 6, settled A). `AppId` moves there too
(from `sill_launcher::ids`), so layouts, `notifications.apps` and the launcher share one type.

```rust
pub struct KindId(Cow<'static, str>);        // serde transparent; fixed() / named()
pub struct InstanceId(Cow<'static, str>);    // unique within one layout
pub struct AppId(pub String);                // moved from sill_launcher::ids

pub enum Shape { One, Small, Medium, Large, ExtraLarge } // One: a list item; the rest: widget sizes
pub enum Shown { On, Off }                    // checklist state; no bool (CONVENTIONS §11)

/// One placed kind in a layout: membership, order (its index in the list) and its config.
/// No position: where a thing sits on a surface is state (below).
pub struct Placement {
    pub kind: KindId,
    #[serde(default)] pub instance: InstanceId,   // default: the kind id; a second one gets "<kind>#2"
    #[serde(default)] pub shape: Option<Shape>,   // None: the host's size for the kind (KindInfo::size_in)
    #[serde(default)] pub shown: Shown,           // checklists; gallery layouts are always On
    #[serde(default)] pub config: toml::Table,    // the kind's own Config, checked by the kind
    #[serde(flatten)] pub extra: toml::Table,     // unknown keys preserved (22 §2)
}
pub struct Layout { pub version: u16, pub items: Vec<Placement> }

/// State, not layout.toml: where one instance sits on one output. Today's sill
/// `desktop_widgets::saved::SavedEntry { output: OutputKey, placement }`, generalised.
pub struct Cell {
    pub instance: InstanceId,
    pub output: Option<OutputName>,   // None = the fallback for every output (sill's OutputKey::Any)
    pub column: u16,
    pub row: u16,
}
```

`output: Option<OutputName>` keeps sill's rule exactly (F881): `Some(name)` is sill's
`OutputKey::Named`, `None` is `OutputKey::Any`; an output with cells of its own reads only those,
one with none reads the `None` cells; saving an output replaces its own cells and keeps the rest.
**Membership is global, cells are per output**: the layout says *which* widgets are on the desktop,
once; the state file says *where* each instance sits on each output.

Rules (all layouts):

1. **Unknown kinds are kept, not dropped.** A placement whose kind is not registered (an app
   uninstalled, a provider not started yet) stays in the file and is not drawn; it comes back
   when the kind does. The picker shows nothing for it.
2. **New kinds join by their own default.** A registered kind not in a checklist layout is
   appended with its `default_shown` on first read and written back on the next save. Gallery
   layouts (widgets) never auto-add.
3. **Config is the kind's.** `config` is parsed into the kind's `type Config` leniently (22 §2:
   an unknown value falls back to the field's default); the host never reads inside it.
4. **One writer task per program, in its services.** Each program's layout file (and each state
   file) is written by exactly one task in that program's services crate (sill:
   `sill-services`, the way `pins::run` owns `dock.json` today): it loads once, applies events
   through a pure reducer, saves atomically (temp file and rename), and picks up an outside edit
   through the directory watch (30 ms debounce, 22 §2). Surfaces and CLI commands never write:
   they send the task events (`sill dock pin` sends `PinEvent::Pin` today; a widget drop sends
   the dropped arrangement; the Settings app's picker writes the file like an outside editor
   does, and the task's watch picks it up).
5. **Order is list order.** No `index` field; a reorder rewrites the list.
6. **A derived place is never written back.** A cell clamped to fit a smaller output, or a
   default place computed for an unplaced item, is drawn but not saved; only a person's drop or
   a first placement records one (sill F882, `view::recorded`).
7. **Hosts with a slot per entry** (hot corners, gesture maps, shortcut bindings) do not use
   `Placement`: each slot is its own key naming an action id (4.9-4.11).

### 5.3 `Registry<I>` and the per-surface traits (quire `ds::place`, UI)

```rust
pub trait KindInfo: Clone + PartialEq {
    fn id(&self) -> &KindId;
    fn name(&self) -> &Text;                         // "Batteries"
    fn summary(&self) -> Option<&Text>;              // gallery description, begins with a verb (HIG widgets)
    fn provider(&self) -> &ProviderApp;              // app id, name, icon: the gallery's grouping
    fn shapes(&self) -> &'static [Shape];            // [One] for list kinds
    fn size_in(&self, host: HostId) -> Shape;        // the host's size for it (sill's size_of); Placement.shape None = this
    fn default_shown(&self) -> Shown;
    fn preview(&self, shape: Shape, host: HostId) -> Element; // placeholder data, live component
}

pub struct Registry<I: KindInfo> { infos: Vec<I> }
impl<I: KindInfo> Registry<I> {
    pub fn with(self, info: I) -> Result<Self, TakenKind>;   // today's WidgetRegistry::with
    pub fn get(&self, id: &KindId) -> Option<&I>;
    pub fn all(&self) -> &[I];
    pub fn by_provider(&self) -> Vec<(ProviderApp, Vec<&I>)>; // the gallery's sidebar
    pub fn resolve(&self, layout: &Layout) -> Vec<(&Placement, &I)>; // rule 5.2.1
}
pub fn provide_registry<I>(r: Registry<I>) -> Registry<I>;   // context, as provide_widget_registry
pub fn use_registry<I>() -> Registry<I>;
```

Each surface keeps its **own typed trait** (the thing an implementer writes) and an erased
`XInfo::of::<T>()` that implements `KindInfo`, exactly as `WidgetInfo::of::<W>()` does today:

| Surface | Typed trait | Erased info | Owner |
| --- | --- | --- | --- |
| widgets | `Widget` (Entry, Intent, Config, view) | `WidgetInfo` | quire `ds::widget` |
| launcher | `Provider` (query, activate) + `ProviderInfo` | `ProviderInfo` (no preview: the checklist row shows icon and name) | sill `sill-launcher`; data types from quire |
| control center | `ControlModule` (tile, panel, bar glyph, active) | `ModuleInfo` | quire contract, sill modules |
| bar | none (items are sources) | `BarItemInfo` | sill |
| toolbar | `ToolbarItem` (command, glyph, width) | `ToolbarItemInfo` | quire |
| share | `ShareTarget` (accepts, share) | `ShareInfo` | quire |
| previewers | `Previewer` (accepts, load, view) | `PreviewerInfo` | quire |
| actions (corners, gestures, shortcuts) | `ShellAction` (run) | `ActionInfo` | sill |
| sidebars | `SidebarSource` (section, items) | `SidebarInfo` | quire |

Launcher providers and actions live in non-UI crates; for them `preview` is not needed, so
`KindInfo::preview` has a default that draws the icon and name as a `ListRow` (the checklist row).

### 5.4 Picker components (quire, 04 new sections, proposed)

| Component | Shape | Used by | Built from |
| --- | --- | --- | --- |
| `KindGallery` | a panel: sidebar of providers (`SidebarItem`) with a `SearchField` on top, a grid of live `preview`s per shape, one description per kind, an Add button under each group; drag a preview out to place it | widgets (4.1) | the `widget-interface` gallery, generalised over `KindInfo` |
| `OrderedChecklist` | `SettingsRow`s: a check (or a `Toggle`), icon, name, a drag handle at the end; drag reorders with the row roster's insert/remove motion; rows the kind marks fixed have no check | launcher (4.2), control center Other modules (4.3), share (4.7), previewers (4.8), sidebars (4.12) | `SettingsRow`, `ds::motion::roster`, `DragGhost`, `DropPlace` |
| `SlotMenu` | one pop-up button per slot, the registry's actions as items, grouped by provider | hot corners (4.9), gestures (4.10), bar presence (4.3) | `Menu{Dropdown}`, the Settings app's `KeyKind::Menu` widget |
| `ChordList` | `SettingsRow`s: toggle, name, captured chord | shortcuts (4.11) | `KeyKind::Shortcut` capture field |
| `CustomizePalette` | a sheet: grid of every item, the default set as one draggable row, a display pop-up, Done | toolbars (4.6) | `Sheet`, `KindGallery`'s grid at `Shape::One` |
| `EditMode` | in place on the host: items take a remove badge and, where the kind offers more than one, a size choice; Done or Escape leaves. It does **not** gate dragging: moving is always on, past `window.move_threshold_px` (desktop widgets, dock), or with Command held (bar, toolbars), in or out of the mode | desktop widgets (4.1), bar (4.4), toolbars (4.6) | `ds::motion::reorder` (5.8) for the drag; `Chip`/`IconButton` for the badge |
| `ConfigForm` | the kind's `Config` rendered by the Settings app's widget table (22 §9.1) inside a popover on the placed item ("Edit Batteries") | widgets, stacks | `SettingsSchema` derive on `Config` |

### 5.5 Motion and detail (26 applies)

Add: the placed item lands with `drop-place`; a removed item leaves with `fold` and the rest
heal (the roster); a reorder slides neighbours at `--t-move --e-spring`. A gallery preview is a
live component at placeholder data (HIG widgets: "Design a realistic preview"); it plays its
appear motion once when it scrolls into view (the wake stamp), never on a loop.

### 5.6 Persistence

- **Placements** (membership, order, per-instance config) live in a layout file per program,
  `$XDG_CONFIG_HOME/<program>/layout.toml` (decision 1, settled B), one `[[<host>]]` array of
  tables per host (`[[center_widgets]]`, `[[launcher_categories]]`, ...): `sill/layout.toml`,
  `quire/layout.toml` (shared by every app: share menu, previewers), `<app>/layout.toml`
  (toolbar, sidebar).
- **Positions stay state** under `$XDG_STATE_HOME`: `sill/desktop-widgets.json` (cells per
  output, F881; 22 §1 rule 3), and any later host whose items sit at coordinates. A position is
  where the person dropped something on one output; it is not a preference to sync or to show in
  Settings.
- **Scalars and behaviour** stay in `settings.toml` (22 §2): the `menu_bar_*` presences,
  `widgets.desktop`, the grid geometry, per-app notification rules, shortcut bindings.
- **design/22 gets a section for layouts outside §3**: a new `## 10. Layouts (layout.toml)`,
  after §9 and after `## 4. Rust shape`, so sill's keys test (`sill-settings/tests/keys.rs`,
  which reads §3.4-3.8 and §3.11 through `## 4. Rust shape` in both directions) never parses a
  `layout.*` entry as a `settings.toml` key. Its tables use the `Layout entry` column head.
- The layout structs derive `SettingsSchema` with `page` and `exposure = Advanced` except where
  the page shows a picker (the picker is the UI; the schema lets the launcher deep link "widgets"
  or "spotlight categories" to it, 22 §9.3).
- **Migration** (once, at first read, old value left in place, as 22 §2's mailo rule):
  `widgets.center`, `widgets.desktop_widgets` and `widgets.world_clocks` into the layout,
  `dock.json` into `layout.dock`, `control_center.modules` into `layout.control_center`
  (its Other modules only; the order is dropped). `desktop-widgets.json` is **not** migrated:
  it stays the state file and gains the `instance` field (an entry without one means the kind's
  first instance).
- HIG settings: a system-wide choice (share menu, previewers, shortcuts) is made in the Settings
  app, not in each app; an app's own toolbar and sidebar are the app's.

### 5.7 Out-of-process registration (documented, not built)

Two halves, as the reference: a **static manifest** so a picker lists an app's kinds without
running it (the widget gallery shows apps that are closed), and a **live interface** for data.

1. **Manifest**: `$XDG_DATA_DIRS/quire/kinds/<app-id>.toml`, installed with the app's `.desktop`
   file (the same place and discovery rule as the settings schemas, 22 §9.2):

   ```toml
   version = 1
   app = "org.example.Weather"
   [[kind]]
   host = "widget"                 # widget | launcher | share | previewer | action | sidebar
   id = "org.example.weather.today"
   name = "Weather"
   summary = "See the current conditions and forecast for a location."
   shapes = ["small", "medium"]
   template = "figure"             # widget only: see below
   placeholder = "today.placeholder.json"
   config_schema = "today.settings.toml"   # the ConfigForm, 22 §9.2 format
   ```

2. **Live**: the app owns `org.quire.Kinds1` at `/org/quire/Kinds1` on the session bus:
   `Describe() -> manifest JSON`; per host, `Timeline(id, instance) -> wire timeline` and signal
   `TimelineChanged(id, instance, wire)` (widgets; `ds::widget::wire` is the format);
   `Query(query_json) -> results` and `Activate(item, choice)` (launcher);
   `Share(target, items)` (share); `Preview(path) -> data` (previewer); `Run(action)` (actions);
   `Intent(id, instance, json)` (a widget's controls). The host starts the app by D-Bus
   activation (`.service` file), never keeps it alive for a widget: a timeline is cached with its
   refresh policy, as design/23 §9.5 says.
3. **Drawing another app's kind**: the host draws; an app never draws on a shell surface. A widget
   from another process names a **template** from a fixed set in quire (`figure`, `gauge`,
   `list`, `month`, `image_caption`, `text`) and sends entries in that template's schema, so its
   card is our card (open decision 7.3).
4. **Adopt existing Linux interfaces where they exist**, so apps already on the system appear
   without porting: tray = StatusNotifierItem (built); launcher = **KRunner D-Bus runners**
   (`org.kde.krunner1`, `.desktop` files in `krunner/dbusplugins`) and **GNOME search providers**
   (`org.gnome.Shell.SearchProvider2`, `.ini` files in `gnome-shell/search-providers`), both
   read as launcher providers with a `KindId` from their desktop id; thumbnails = the freedesktop
   thumbnailer files; notifications = the `desktop-entry` hint for per-app rows. Share targets,
   control modules and widgets have no Linux standard; only ours applies.

### 5.8 What in quire and sill becomes generic

| Today | Becomes | Where |
| --- | --- | --- |
| `ds::widget::WidgetKind` (`widget-interface`) | `KindId` | `ds-settings` (data) |
| `sill_launcher::ids::AppId` | `AppId`, shared by launcher, layouts and `notifications.apps` | `ds-settings` |
| `ds::widget::WidgetRegistry`, `WidgetInfo::of`, `TakenKind`, `provide_/use_widget_registry` | `Registry<I>`, `KindInfo`, `provide_/use_registry` | `ds::place` |
| the widget gallery page (`ds-gallery` pages `widgets`, the picker being built) | `KindGallery` | `ds::components::kind_gallery` |
| `ds::components::widget_kind::WidgetSize`; sill `widgets::size_of(kind, host)` | `Shape`; `KindInfo::size_in(host)` | `ds-settings`; `ds::place` |
| sill `desktop_widgets::saved::{SavedEntry, OutputKey}` | `Cell { instance, output: Option<OutputName>, column, row }` (`OutputKey::Any` is `None`) | `ds-settings`; sill keeps the file and `with_output` |
| sill `desktop_widgets::placement::{GridSlot, clamp, nearest_free}` | stay sill's (the desktop grid is sill's); `clamp` keeps rule 5.2.6 | sill |
| sill `desktop_widgets/drag.rs` (`Hold::{Idle,Pressed,Live}`, threshold, grab offset) with `arrange.rs` (`snap`, `make_room`), and sill `dock/machine/press.rs` (press, threshold, drag reorder, drag-out) | `ds::motion::reorder`: one press-travel-lift-drop machine and the make-room rule for a grid and for a row, used by the desktop, the dock, the bar's Command-drag, toolbars and checklists | quire |
| sill `pins::model::{DockPins, PinEvent, PinAt}` and `pins::run` | `Layout` plus the dock's own events; `pins::run` is the model for rule 5.2.4's writer task | sill, over the shared data |
| sill `ControlCenterModule`, `MENU_BAR_ORDER`, `in_menu_bar` | `ModuleInfo` in a registry; presence from the `menu_bar_*` keys, bar order from `layout.bar_items` | sill, contract in quire |
| sill `ProviderKind` closed enum and its `ALL` order | built-in `KindId`s; order from `layout.launcher_categories` (Web pinned last) | sill-launcher |
| sill `CornerAction` closed enum; palmrest `Action` | `ShellAction` ids in one action registry | sill (palmrest reads ids) |
| `ds-settings` `KeyKind::List` ("a rows editor") | its widget is `OrderedChecklist`; `KeyKind::Record` (P5) makes `List(Record)` a table editor | ds-settings + Settings app |
| `SettingsRow`, `ds::motion::roster`, `DragGhost`, `DropPlace`, `ds::detail::use_tween` | reused unchanged by the pickers and the reorder machine | quire |
| `AppearancePicker`, `UserPicture` picker | stay single-value pickers (one choice, no placement) | unchanged |

## 6. Order of work

### 6.0 Row order (sill's keys test)

sill's `sill-settings/tests/keys.rs` checks design/22 against sill's registered `KeySpec`s in
both directions over §3.4-3.8 and §3.11 through `## 4. Rust shape` (that is, §3.4-3.23 less
palmrest's §3.9-3.10), reading quire's doc by path from sill's own checkout. A row with no spec,
or a spec with no row, fails sill master at once. So every change to those sections goes in this
order:

1. **sill first.** A new key: sill registers it and lists its path in `AWAITING_ROWS`. A retired
   key (`widgets.center`, `widgets.desktop_widgets`, `widgets.world_clocks`,
   `control_center.modules`): sill drops it (keeping the old value readable for the one-time
   migration, as `extra`).
2. **Then quire** adds, changes or removes the design/22 row, and tells sill.
3. **Then sill** removes the `AWAITING_ROWS` entry once the row has landed.

This document's own tables are proposals, not design/22 rows; nothing in sill reads them. Layout
entries (`layout.*`) never enter §3: they go to design/22's new `## 10. Layouts (layout.toml)`
(5.6), which the test skips. State files (`desktop-widgets.json`, `notifying-apps.json`) get no
row anywhere. No placeholder path (a `<module>` or `<app>` in a key name) is written in a first
table cell anywhere in design/22 or here, since the parse takes any backticked first cell.

### 6.1 Before the app pass

Shell; each a quire lane plus a sill lane where named.

1. **P0 Land the widget interface** (in progress on `widget-interface`) as the reference
   implementation. No generalisation inside that lane.
2. **P1 Extract the pattern** (quire): `KindId`, `InstanceId`, `Shape`, `Shown`, `Placement`,
   `Layout`, `Cell` and `AppId` in `ds-settings` with the lenient loader and the atomic writer;
   `Registry<I>`/`KindInfo` (with `size_in`) in `ds::place`; `WidgetRegistry` becomes
   `Registry<WidgetInfo>`; `ds::motion::reorder` from sill's `desktop_widgets/drag.rs`,
   `arrange.rs` and `dock/machine/press.rs`; `OrderedChecklist`; `KindGallery` from the widget
   gallery; design/22 `## 10. Layouts`. Warn sill before any rename lands (sill reads quire by
   path).
3. **P2 Widgets on the shared shape** (sill): the hosts read `layout.center_widgets` and
   `layout.desktop_widgets` (membership) through one writer task in `sill-services`; cells stay
   in `desktop-widgets.json` with the new `instance` field; World Clock's zones move to its
   instance config; the gallery opens from the desktop menu and the notification center;
   `EditMode` adds remove badges and size choices (dragging stays always on). Row order 6.0 for
   the three retired `widgets.*` keys.
4. **P3 Launcher categories** (sill + Settings page): `ProviderInfo`,
   `layout.launcher_categories` with Web pinned last and Web/Files/Clipboard reading their keys,
   `launcher.excluded_folders` (AWAITING first).
5. **P4 Control center and bar** (sill): `layout.control_center` (Other modules; the order leaves
   the person's hands), `InMenuBar::WhenActive`, the new `control_center.menu_bar_focus` key;
   the tray service reads the SNI `Id` property (`TrayItem.sni_id`), and `layout.bar_items` /
   `layout.bar_hidden` are keyed by it; Command-drag in the bar on `ds::motion::reorder`.
6. **P5 Notifications per app** (quire + sill): quire adds a record kind to ds-settings'
   `KeyKind` (`Record { fields: Vec<FieldSpec> }`, so `List(Box<Record>)` describes a
   `Vec<struct>` key and the Settings app draws it as a table editor; today `List` holds only
   scalar kinds); sill adds `notifications.apps` and `notifications.previews` (AWAITING first),
   the `notifying-apps.json` state file written by the notifications task, and the Settings page.
7. **P6 Actions** (sill, palmrest): the `ShellAction` registry; hot corners, gestures and the
   system shortcuts list read ids; `shortcuts.bindings` written to COSMIC's `custom` and
   `system_actions` with COSMIC winning a clash (4.11); design/22 §5's Keyboard / Shortcuts line
   now points here.
8. **P7 Dock on the shared shape** (sill): `layout.dock`, migration of `dock.json`, `pins::run`
   as its writer task; stacks stay deferred.

### 6.2 During the app pass (with each app)

9. **A1 Toolbars** (quire, after H4 `MenuModel`): `Toolbar`, `ToolbarItem`, `CustomizePalette`;
   first in Files and Mail.
10. **A2 Sidebars**: `SidebarSource`, the app's Sidebar settings; first in Files.
11. **A3 Previewers**: `Previewer` registry shared by the launcher pane and the Quick Look app;
    thumbnailer files.
12. **A4 Launcher providers from apps**: Mail and Notes search as in-tree providers first, then
    the KRunner and GNOME search provider bridges (5.7.4), then `org.quire.Kinds1` `Query`.
13. **A5 Share menu**: `ShareTarget`, `ShareMenu`, the Extensions page.
14. **A6 App Shortcuts**: after `MenuModel`.

### 6.3 After

Focus modes and filters (4.15), stacks (4.5), out-of-process widgets over `org.quire.Kinds1`
with templates (5.7.3), preview options per type (4.8), the hot-corner modifier once the
compositor fork reports held modifiers (4.9).

## 7. Open decisions (for the user; each has a recommendation)

**Settled 2026-09-27.** The user picked 2 (B, reorderable), 3 (A, quire templates), 4 (add When Active) and 5 (A, membership only); 1, 6, 7, 8 and 9 follow the recommendations (1 B `layout.toml`, 6 A `ds-settings`, 7 consume KRunner and GNOME providers, 8 and 9 as recommended).

1. **Where placements live.** (A) as list keys in `settings.toml`; (B) a `layout.toml` per
   program beside `settings.toml`, schema'd the same way; (C) keep today's per-surface JSON files
   (`dock.json`, `desktop-widgets.json`) and add more. **Recommend B**: placements change by
   dragging, often, and would churn the hand-editable settings file; one file per program keeps
   one writer and one watch; the Settings app still renders it through the schema.
2. **Launcher category order.** (A) a checklist only, fixed order, as the reference; (B)
   checklist plus drag to reorder. **Recommend B**: the order is already visible as the Tab order
   and the section order, and the user asked for the launcher to be placeable; the default order
   stays the reference-like one.
3. **How another process's widget is drawn.** (A) the app sends entries for a quire template
   (figure, gauge, list, month, image and caption, text), the host draws; (B) the app renders a
   buffer the host composites; (C) the app ships a sandboxed view (WASM). **Recommend A**: every
   card stays our card (design/23's whole point), no GPU or protocol work, and the reference's
   own widgets are archived views drawn by the system, not app buffers.
4. **"When Active" in the bar.** Add `InMenuBar::WhenActive` (Focus, Display, Sound, Now Playing,
   and later screen sharing), or keep two states. **Recommend add**: it is the reference's
   default for Focus and Now Playing and costs one variant plus `ControlModule::active`.
5. **Control center arrangement.** (A) membership of Other modules only, the design's order, as
   Sonoma/Sequoia; (B) free reorder and resize of tiles (the post-2025 editable Control Center).
   **Recommend A** (target era; B is Liquid Glass era).
6. **Where the data types live.** (A) `ds-settings` (non-UI, already owns persistence and the
   schema; sill-launcher and palmrest can depend on it); (B) a new small crate `ds-place`;
   (C) `ds` (pulls dioxus into non-UI crates). **Recommend A**; split to B only if `ds-settings`
   grows past its purpose.
7. **Third-party launcher providers in v1.** Consume KRunner D-Bus runners and GNOME search
   providers during the app pass (cheap, many apps exist), or only our own interface later.
   **Recommend consume both** during A4, marked as foreign rows with the app's icon.
8. **One gallery or two for widgets.** The notification center and the desktop share one
   registry and one `KindGallery`, with two layouts (an ordered list; a desktop membership list
   whose cells per output are state), as the reference; or one layout with a host field. **Recommend two layouts, one registry.**
9. **Bar order key when the person never reorders.** Keep `layout.bar_items` empty (default
   order computed) or write the full order on first run. **Recommend empty until the first
   Command-drag**, so a new default order can still ship to people who never touched it.
10. **Shortcut clashes with COSMIC** (new in the revision, 4.11). (A) the chord COSMIC holds
    wins: sill writes only chords COSMIC does not hold for another action, and the Settings app
    shows the clash on the row; (B) sill's `shortcuts.bindings` wins and overwrites COSMIC's
    binding; (C) refuse to save a clashing chord. **Recommend A**: COSMIC's files are the live
    truth for global keys and may be edited outside sill (COSMIC Settings, by hand); overwriting
    silently breaks a binding the person made there, and refusing hides why.

Note on decision 1 (revision): `layout.toml` holds only membership, order and per-instance
config. Positions and cells stay state (`$XDG_STATE_HOME/sill/desktop-widgets.json`, sill F881,
22 §1 rule 3); the settled choice is unchanged by this.

## 8. Sources

- Apple Human Interface Guidelines, JSON through Wayback: widgets (20250224003541), controls
  (20250303222558), the-menu-bar (20241214023016), searching (20240210073513), toolbars
  (20240302055431), activity-views (20250224003701), sidebars (20250228024414), notifications
  (20250531072050), managing-notifications (20250502150210), keyboards (20250225175135),
  settings (20231102161025), dock-menus (20250407102757). URL form:
  `http://web.archive.org/web/<timestamp>/https://developer.apple.com/tutorials/data/design/human-interface-guidelines/<page>.json`.
- Apple Support, Mac User Guide (support.apple.com/guide/mac-help/<id>/<version>/mac/<version>):
  mchlad96d366 (15.0) Change Control Center settings; mchlp1446 (15.0) What's in the menu bar;
  mchl52be5da5 (14.0) Add and customize widgets; mchlp1119 (15.0) Desktop & Dock settings;
  mh35859 (15.0) Use the Dock; mchl54d95e8a (15.0) Spotlight settings; mchlp2811 (12.0) Change
  Spotlight preferences; mtusr003 (15.0) Login Items & Extensions settings; mh40583 (15.0)
  Notifications settings; mchlp3011 Customize the Finder toolbar; mchl83c9e8b8 Customize the
  Finder sidebar; mchl97ff9142 Perform quick actions in the Finder; mchl1e4644c2 Use the Preview
  pane; mchlp2271 (13.0) Create keyboard shortcuts for apps; mh11784 (14.0) Lock Screen settings;
  mchl613dc43f Set up a Focus; Calendar User Guide icld13f9da17 Use Focus filters.
- freedesktop.org: StatusNotifierItem; Thumbnail Managing Standard (thumbnailer files). KDE
  KRunner D-Bus runner interface `org.kde.krunner1`; GNOME Shell search provider interface
  `org.gnome.Shell.SearchProvider2` (named for the bridge in 5.7.4; not fetched in this pass).
- Our docs: 10, 12, 13 (13.3.7, 13.3.9, 13.3.12), 20 (1.1-1.6, 1.9, 1.14, 1.16, 2.2-2.4), 22
  (2, 3.4-3.6, 3.12-3.13, 3.19-3.23, 5, 9), 23 (4.3, 6; section 9 on `widget-interface`), 27
  (0.2, 4.16, 5.2, 5.11, 5.12, 6.2).
- Code (read-only): sill at `45f9af0`, re-read at master `0096872` for this revision (widget drag
  F880-F892, `saved.rs`, `size_of`, the tray model, `rank/sections.rs`, the keys test); quire `master` at `a2cbcb7a` and the `widget-interface`
  worktree's uncommitted `crates/ds/src/widget/`.
