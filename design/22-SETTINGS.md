# 22 Settings

Status: this whole document is **proposed** (it did not exist before the user's decision of
2026-09-24) except where a row's Status column says otherwise. `design/<file>.md#<anchor>` is
the citation convention of `README.md#3-citation-convention`.

## 1. What this governs

Every runtime-tunable number and closed-set choice for the desktop shell: the design-system
appearance (`quire`), the shell (`sill`, its bar/dock/launcher/menus/notifications/spaces), and
the gesture daemon (`palmrest`). The rule, per the user's decision of 2026-09-24 recorded in
`README.md#4-canonical-source`: **no value any design doc marks "proposed" is hard-coded**; it
ships as the default *and* is read from a settings key listed in section 3. A **settled** value
becomes a key only where a doc treats it as a user-facing preference — natural scrolling,
magnification on/off, autohide, gesture-to-action mapping, theme/accent/motion-level — the
same pattern the docs' own `10.6`/`11.6`/`12.6`/`13.6` Configuration tables already use (settled
defaults sitting next to proposed ones in one table). Everything else settled — component
markup, colour tokens, motion keyframes, layout grids — is **not** a key: it stays in `ds`'s
fixed token tables and is out of scope here.

Three further scoping rules, applied consistently across section 3, because "every proposed
value" in these docs literally includes hundreds of component visual gaps that are not
preferences:

1. **Design tokens are not settings.** A Material's tint colour, edge, shadow, radius and blur
   recipe (`03-COLOR.md#17-2-starting-values-proposed`) is `ds` token data, tunable in the
   gallery, not per-user. The one exception the docs make settings-worthy is *which* Material
   enum variant a surface uses when a doc explicitly frames that as an open choice (e.g.
   notification banner: `Toast` vs an inverse-ink alternative) — that choice is a key; the
   Material's own recipe is not.
2. **Component gaps are not settings.** `04-COMPONENTS.md` marks dozens of hover/disabled/
   error states "not specified" with no candidate value at all (see its Open decisions O-1..
   O-29). Those are review gaps for review/build, not tunable defaults, and are excluded; see
   the "could not find a default" list in the handback report.
3. **Per-workspace instance data is not a global default.** `21-SPACES.md#10-storage-settled-path-target-clean-up-phase-schema`'s
   `$XDG_CONFIG_HOME/quire/spaces.json` stores each workspace's chosen `SpaceLook` (its hue, chroma
   factor, grain, theme; target (clean-up phase), 21 section 1) — that is saved *state*, like
   the dock's pinned items (`[[dock]]` in `sill/layout.toml`), not a settings default. The *defaults* applied when a workspace has
   none (preset table, fallback grain) are keys in the `spaces` domain below.

## 2. Storage

One TOML file per owning program, replacing the plan's `appearance.json` (PLAN "Design:
`<ds>`") and the docs' scattered `~/.config/quire/scroll.json` / `~/.config/sill/settings.json`
/ `~/.config/palmrest/config.toml` mentions (`11-BEHAVIOUR-scroll.md#11-6-configuration`,
`10-BEHAVIOUR-dock.md#10-6-configuration`, `12-BEHAVIOUR-gestures.md#12-6-configuration`): this
doc is the authority on where each key actually lives, and supersedes those inline paths.

| File | Owner | Domains |
| --- | --- | --- |
| `$XDG_CONFIG_HOME/quire/appearance.toml` | `ds-settings` (crate `crates/ds-settings`, PLAN "Design: `<ds>`") | `appearance`, `motion` (level selector only), `icons` |
| `$XDG_CONFIG_HOME/sill/settings.toml` | `sill` (crate `sill-services`/`sill-surfaces`) | `bar`, `dock`, `launcher`, `scroll`, `scrollbar`, `menus`, `switcher`, `notifications`, `control_center`, `spaces`, `osd`, `power_menu`, `display`, `session`, `widgets`, `calendar`, `screenshot`, `hot_corners`, `keyboard` |
| `$XDG_CONFIG_HOME/<daemon>/…` | the companion's daemons: inferd (porter), intentd and companiond (docket), memoryd (almanac), cuad (cua); each repo names its file and ships its schema (section 9.2). Named so far: docket `$XDG_CONFIG_HOME/docket/settings.toml` (schema `dist/settings/docket.settings.toml`); inferd's schema is porter's `dist/inferd.settings.toml` | `ai`, `agent`, `memory`, `cua` (sections 3.26 to 3.29) |
| `$XDG_CONFIG_HOME/anyview/settings.toml` | `anyview` (the file viewer; its settings module ships the schema, section 9.2) | `viewer` (section 3.31) |
| `$XDG_CONFIG_HOME/palmrest/gestures.toml` | `palmrest` (the gesture daemon, PLAN Appendix B); `sill`/`shell-host` read it read-only for `PointerOver` suppression and the scroll `feel` module | `gestures`, `palm_rejection` |

Rules, all three files:

- **Atomic write**: temp file + `rename` (`ds-settings`'s `file::save`).
- **`version = 1`** top-level field. A future incompatible change bumps it and ships a
  migrator; today nothing reads it but its absence.
- **A key nobody reads is retired**: its row's status says `RETIRING` and its field is deleted
  in the same change; the row goes when the key is read again or dropped for good, and the
  tests that compare rows with fields skip a `RETIRING` row.
- **Unknown keys reported, not preserved**: a key no struct reads does not stop a load
  (`CONVENTIONS.md#12-derives-and-serde`: never `deny_unknown_fields` on a stored type), but it is
  not carried through a save either. `Store::load` returns it in `Loaded::unknown` (a dotted
  path, cut at the first table the struct has no key for) and the owning program prints it to its
  log; the next write drops it. There is no `extra` table and no compatibility for retired keys.
- **Unknown *values* fall back to the field's default**, lenient, matching mailo's `Appearance`
  loader (PLAN "moves verbatim from mailo... `appearance.rs` load/save/dirs"): a bad enum
  string or an out-of-range number logs once and uses `Default::default()` for that field only,
  never fails the whole file.
- **Live reload**: a `notify` watch on the containing directory (not the file — editors and our
  own atomic writer both replace the inode via rename), **30 ms debounce**
  (`12-BEHAVIOUR-gestures.md#12-6-configuration`'s own daemon does the same for its config; PLAN
  "Design: `<ds>`" `ds-settings`: "`notify` directory watch (rename replaces inode; debounce
  30 ms)"). A change is diffed (section 4's `apply`) and applied without restart; no surface
  animates from a settings change (it just repaints with new values on its next frame).

## 3. The key catalogue

Types used below (see section 4 for the full list): `Px(u16)` logical pixels, `Ms(u16)`
milliseconds, `Percent(u8)` 0..100, `Fraction(u16)` per-mille (1000 = 1.0; used for ratios,
gains and constants like `c = 0.55` -> `550`), `Count(u16)` a plain quantity, `Secs(u16)`
seconds and `Mins(u16)` minutes (added Q445: before this, a seconds or minutes key had no unit
of its own and used `Count` with a unit label in prose; `Ms` is milliseconds and does not reach
the tens-of-seconds to hours range idle timeouts need), `Scalar(f32)` a dimensionless physics
constant that does not fit the above (momentum model exponents), `Units` a signed raw
touchpad/report unit (device space, not px), `MicroUsd(u32)` millionths of a dollar (the companion's spend caps). Every enum is named; **no key is a `bool`**
(`CONVENTIONS.md#4-types`).

Status column values: **proposed** = doc marks the value proposed, this is the "make it a key"
case; **settled (preference)** = doc marks it settled but it is a user-facing choice, so it is
still a key per section 1; **settled default, range proposed** = the doc's own phrasing, kept
verbatim. **advanced** in the last column (used in section 5) means file-only, no Settings UI
control in v1.

### 3.1 `appearance` (quire/appearance.toml)

The table below is generated from the schema (`scripts/gen-doc-keys.sh`); the notes under it are
hand-written. Where a note's default disagrees with the table, the table (the code) is right.

<!-- keys:begin appearance -->

| Key | Type | Default | Range | Page | Shown | Agent |
| --- | --- | --- | --- | --- | --- | --- |
| `appearance.theme` | choice {system, light, dark} | `system` |  | Appearance | Settings | may set |
| `appearance.accent` | choice {blue, purple, pink, red, orange, yellow, green, graphite} | `blue` |  | Appearance | Settings | may set |
| `appearance.motion_level` | choice {standard, reduced} | `standard` |  | Appearance | Settings |  |
| `appearance.transparency` | choice {system, reduced} | `system` |  | Appearance | Settings |  |
| `appearance.typeface` | choice {system, editorial} | `system` |  | Appearance | Settings |  |
| `appearance.look` | provided (look) | `standard` |  | Appearance | Settings | may set |
| `appearance.file_icons` | provided (file_icons) | `standard` |  | Appearance | Settings | may set |
| `appearance.custom_styles` | choice {use, off} | `use` |  | Appearance | Settings | may set |

<!-- keys:end appearance -->

| Key | Notes | Source | Status |
| --- | --- | --- | --- |
| `appearance.theme` |  | `30-CATALOGUE.md` section 3.2 | settled (preference) |
| `appearance.accent` | settled 2026-09-30; a stored `postmark`, `amber` or `violet` is not migrated: it reads as an invalid value and falls to `Blue` | `03-COLOR.md#open-decisions` item 6 | settled (preference), partial |
| `appearance.motion_level` | the portal's `prefers-reduced-motion` makes `Standard` resolve to `Reduced` | `30-CATALOGUE.md` section 1.1 | settled (user, 2026-09-29) |
| `appearance.transparency` | the portal has no key for it, so ours is a setting; `Reduced` paints every material solid and drops the vibrancy boost; the compositor reads it from this file, as it reads `theme` | `27-HIG-PARITY.md` section 3.1 | settled (adopted in 27) |
| `appearance.typeface` | RETIRING: The second typeface (Editorial) is dropped (30 section 3.4): one voice, Inter, with Space Mono for code and `Kbd`. Target (clean-up phase): remove the key and `data-typeface`; a stored value is an unknown key | `02-TYPE.md#2-the-faces` | settled; to retire in the clean-up phase (2026-10-02) |
| `appearance.look` | `standard`, or the id of an installed look Extension; a stored id that is not installed or is turned off reads as `standard` where the installed Extensions are known, and is reported once (`ds_extensions::chosen`). The choices come from the caller (9.1, `Provided`) | the customization design, sections 5 and 7.7 | proposed (2026-10-23) |
| `appearance.file_icons` | `standard`, or the id of an installed file-icons Extension (a folder of pictures keyed by file type); a stored id that is not installed or is turned off reads as `standard` and is reported once (`ds_extensions::chosen`). A type the Extension has no picture for keeps the standard icon. Section "Files" of the Appearance page | the customization design, sections 5 and 6 | proposed (2026-10-24) |
| `appearance.custom_styles` | "Use my styles": `off` makes `Ds` draw none of the look, the desktop-wide `style.css` or the app's `style.css`, live; the way back from a broken style (customization design section 7.4) | `customization/DESIGN.md` section 7.4 | proposed (2026-10-23) |

**Removed 2026-10-27 (settings keep only what macOS offers or a real behaviour choice).** The seven
`appearance.material_*` keys are stylesheet tokens, not settings: `--m-tint-alpha` (.8),
`--m-highlight-light` (.3), `--m-highlight-dark` (.12), `--m-hairline-light` (.14),
`--m-hairline-dark` (.6), `--m-shadow-strength` (1) and `--m-vibrancy` (1). Their defaults live in
the material blocks, `Ds` writes none of them inline, and a look or the person's own style sheet
sets them. A stored value is an unknown key: it is reported and kept in the file, never read.

Rows for keys that sill owns but this section names (they are stored under `appearance` for the
vocabulary they use):

| Key | Type | Default | Range / Alt | Source | Status |
| --- | --- | --- | --- | --- | --- |
| `control_center.material` | `Material::{Sheet}` (fixed for v1) | `Sheet` | | `03-COLOR.md#17-3-material-per-surface` | RETIRING: no reader; the panel is always a Sheet |

`notifications.banner_material` is stored under `appearance` (it names a `ds::prelude::Material`
variant, appearance's vocabulary) but is *read* by `sill`'s notifications surface; see section
7 for the cross-file read this implies.

### 3.2 `motion` (quire/appearance.toml — level selector only)

Per-`MotionLevel` durations, easings and scalars (`--t-*`, `--e-*`, `--overshoot`, `--squish`,
`--lift`, `--tilt`, `--stagger`) are `ds` token-table data (PLAN "Design: `<ds>`" token model),
generated by `ds::stylesheet()`, not settings — including the ones `05-MOTION.md#10-shell-motion`
and `#12-open-decisions` mark "proposed" for *which curve* a level uses (e.g. Reduced's
`--e-spring` = `--e-out`, `05-MOTION.md#10-shell-motion`). Rule 1 in section 1 applies: only the
level itself is user-tunable, and it is `appearance.motion_level` (3.1).

### 3.3 `icons` (quire/appearance.toml)

The app-icon *generation* pipeline (`08-ICONS.md#3-generation-pipeline`) is a build-time tool
(`quire-icons`), not a runtime setting, and is out of scope. These are the runtime,
per-third-party-icon rendering values `sill`'s dock/launcher apply live (`08-ICONS.md#4-third-party-app-icons`).

<!-- keys:begin icons -->

| Key | Type | Default | Range | Page | Shown | Agent |
| --- | --- | --- | --- | --- | --- | --- |
| `icons.style` | choice {colour, muted, monochrome} | `colour` |  | Appearance | Settings |  |

<!-- keys:end icons -->

| Key | Notes | Source | Status |
| --- | --- | --- | --- |
| `icons.style` | `Muted` lowers the chroma cap; `Monochrome` draws every icon in one hue, tone on tone, and desaturates and re-tints third-party icons inside our plates to the same hue | `08-ICONS.md#210-dialects-proposed-round-four-2026-09-25` | proposed (round four; implementation beyond the key waits for the user's dialect pick) |

**Removed 2026-10-27.** `icons.monochrome_tint` (a Monochrome icon takes the Space's hue,
`Tint::space`), `icons.plate_inset_percent` (72) and `icons.symbolic_fallback_glyph_percent` (56)
(the `--icons-inset-share` and `--icons-glyph-share` tokens behind `--plate-inset` and
`--plate-glyph`), `icons.symbolic_chroma_max` (`ChromaLimit::PROPOSED`, 0.04),
`icons.squircle_detect_iou` (0.90), `icons.plate_glyph_colour_policy` (Auto: white ink where it
reads at 3:1 on the fill, the hue's deep ink otherwise) and `icons.dark_mode_variant` (the same
icon in both schemes) are no longer settings. The three constants with no reader in quire
(0.90, Auto, same-as-light) belong to the program that plates third-party icons (sill).

### 3.4 `bar` (sill/settings.toml)

| Key | Type | Default | Range / Alt | Source | Status |
| --- | --- | --- | --- | --- | --- |
| `bar.title_hit_height_px` | `Px` | `24` | | `13-BEHAVIOUR-menus-windows.md#13-3-1-bar-geometry` | RETIRING: no reader; a bar item's hit box is the item's own (`BarPointer`, design/30 section 2.10) |
| `bar.battery_low_percent` | `Percent` | `20` | `5..=50`; at or under this charge, on battery, the fill turns `--battery-low` and the item nudges once per discharge | `26-DETAILS.md` §5.1.3 G10, G11; sill FINDINGS F822 | proposed (D1, 2026-09-27) |
| `bar.autohide` | `BarAutohide::{Always,DesktopOnly,FullScreenOnly,Never}` | `FullScreenOnly` | TOML words `always`, `desktop_only`, `full_screen_only`, `never`; macOS's "Automatically hide and show the menu bar" (Always, On Desktop Only, In Full Screen Only, Never); Appearance › Bar | `28-CUSTOMIZATION.md` (menu bar auto-hide is a scalar key, not a placement); sill's bar auto-hide | settled (shell, 2026-10-22) |

### 3.5 `dock` (sill/settings.toml)

Pinned items live in `[[dock]]` rows of `~/.config/sill/layout.toml` (design/28 §5.6, built in sill 848a20a7; the old `dock.json` is read once and carried over), not in this file (
`10-BEHAVIOUR-dock.md#10-6-configuration`).

| Key | Type | Default | Range / Alt | Source | Status |
| --- | --- | --- | --- | --- | --- |
| `dock.magnification` | `Magnification::{On,Off}` | `On` | | `10-BEHAVIOUR-dock.md#10-6-configuration` | settled (preference) |
| `dock.tile_size_px` | `Px` | `48` | `32..=80` | `10-BEHAVIOUR-dock.md#10-6-configuration` | settled default, range proposed |
| `dock.magnified_size_px` | `Px` | `72` | `32..=72` (was `tile_size..128` and 96 before the shell owner settled it on 72, 2026-10-20) | `10-BEHAVIOUR-dock.md#10-6-configuration` | settled default, range proposed |
| `dock.progress_style` | `ProgressStyle::{Ring,Bar}` | `Ring` | alt `Bar` (macOS draws a bar under the icon) | `10-BEHAVIOUR-dock.md#10-3-2-running-indicator-badge-progress-separator`; `10-BEHAVIOUR-dock.md#10-9-open-decisions` item 2 | RETIRING: no reader; the Dock draws a bar under the icon, as macOS does (design/30 Part 4) |
| `dock.autohide` | `AutoHide::{Off,On}` | `Off` | | `10-BEHAVIOUR-dock.md#10-6-configuration` | settled (preference) |
| `dock.position` | `DockPosition::{Bottom,Left,Right}` | `Bottom` | Left/Right deferred | `10-BEHAVIOUR-dock.md#10-6-configuration` | RETIRING: no reader; the dock is always at the bottom until Left and Right are built |
| `dock.indicators` | `Indicators::{On,Off}` | `On` | | `10-BEHAVIOUR-dock.md#10-6-configuration` | proposed |
| `dock.trash` | `TrashTile::{On,Off}` | `On` | | `10-BEHAVIOUR-dock.md#10-6-configuration` | proposed |
| `dock.badge_size_px` | `Px` | `18` | | `10-BEHAVIOUR-dock.md#10-3-2-running-indicator-badge-progress-separator` | RETIRING: no reader; quire's `Badge` and `ProgressIndicator` set their own sizes (`DockTile`) |
| `dock.badge_radius_px` | `Px` | `9` | | `10-BEHAVIOUR-dock.md#10-3-2-running-indicator-badge-progress-separator` | RETIRING: no reader; quire's `Badge` and `ProgressIndicator` set their own sizes (`DockTile`) |
| `dock.progress_ring_diameter_px` | `Px` | `20` | | `10-BEHAVIOUR-dock.md#10-3-2-running-indicator-badge-progress-separator` | RETIRING: no reader; quire's `Badge` and `ProgressIndicator` set their own sizes (`DockTile`) |
| `dock.progress_ring_stroke_px` | `Px` | `3` | | `10-BEHAVIOUR-dock.md#10-3-2-running-indicator-badge-progress-separator` | RETIRING: no reader; quire's `Badge` and `ProgressIndicator` set their own sizes (`DockTile`) |
| `dock.floor` | `DockFloor::{Off,On}` | `Off` |  | `FINDINGS.md` "macOS polish"; `04-COMPONENTS.md` | proposed (polish pass, 2026-09-25) |

### 3.6 `launcher` (sill/settings.toml)

| Key | Type | Default | Range / Alt | Source | Status |
| --- | --- | --- | --- | --- | --- |
| `launcher.clipboard_history` | `ClipboardHistory::{Memory,Off}` | `Memory` | memory only, forgotten when sill stops; password-manager copies never kept; Off watches nothing | `09-ARC-HEURISTICS.md` H4; sill M9 (Q301) | settled (user, 2026-09-27) |
| `launcher.emoji_skin_tone` | `EmojiSkinTone::{Default,Light,MediumLight,Medium,MediumDark,Dark}` | `Default` | Unicode's five modifiers or the yellow default | `20-SURFACES.md` §1.3; sill M9 (Q301) | settled (user, 2026-09-27) |
| `launcher.web_search` | `WebSearch::{On,Off}` | `On` | "Search the web for …" as the last result | `20-SURFACES.md` §1.3; sill M9 (Q301) | settled (user, 2026-09-27) |
| `launcher.currency` | `CurrencyRates::{Off,Ecb}` | `Off` | Ecb fetches the ECB daily reference rates at most once a day | `20-SURFACES.md` §1.3; sill M9 (Q301) | proposed (2026-09-26) |

### 3.7 `scroll` (sill/settings.toml)

`scroll.speed`, `scroll.lock_*` and the gain constants are **not** duplicated here: they are
palmrest's `feel` module; `scroll.speed` lives under `gestures` (3.9) in `palmrest/gestures.toml`, which
`shell-host` reads read-only (`11-BEHAVIOUR-scroll.md#11-7-integration`).

| Key | Type | Default | Range / Alt | Source | Status |
| --- | --- | --- | --- | --- | --- |
| `scroll.natural` | `NaturalScroll::{Natural,Traditional}` | `Natural` | | `11-BEHAVIOUR-scroll.md#11-6-configuration` (R16) | settled (preference) |
| `scroll.momentum` | `Momentum::{On,Off}` | `On` | | `11-BEHAVIOUR-scroll.md#11-6-configuration` | proposed |

### 3.8 `scrollbar` (sill/settings.toml)

| Key | Type | Default | Range / Alt | Source | Status |
| --- | --- | --- | --- | --- | --- |
| `scroll.scrollbars` | `ScrollbarVisibility::{WhenScrolling,Always}` | `WhenScrolling` | | `11-BEHAVIOUR-scroll.md#11-6-configuration` | proposed |
| `scrollbar.track_click_mode` | `TrackClick::{PageTowardClick,JumpToClick}` | `PageTowardClick` | macOS default is "jump to next page" | `11-BEHAVIOUR-scroll.md#11-3-12-overlay-scrollbar-our-own-thumb` | proposed |

### 3.9 `gestures` (palmrest/gestures.toml)

Retired 2026-10-27 except two rows: palmrest has only `scroll.speed` and the
`gestures.gesture_action_map` rows (one per gesture). The other keys of
`12-BEHAVIOUR-gestures.md#12-6-configuration` (the scroll lock and gains, `swipe.*`, `tap.*`,
`foreign_output`, the G4 and live-workspace numbers, `contact_nibble_policy`) are constants in
palmrest; a file that still has them loads, the keys are ignored.

| Key | Type | Default | Range / Alt | Source | Status |
| --- | --- | --- | --- | --- | --- |
| `scroll.speed` | `Count` | `22` | `0..63` | `12-BEHAVIOUR-gestures.md#12-6-configuration` | settled (Python) |
| `gestures.gesture_action_map` | `Map<Gesture, Action>` | Apple mapping shipped (see `12-BEHAVIOUR-gestures.md#12-6-configuration` table) | every `Gesture` variant remappable | `12-BEHAVIOUR-gestures.md#12-6-configuration` | settled (preference) |

### 3.10 `palm_rejection` (retired)

Retired 2026-10-27: palmrest keeps only `scroll.speed` and the `gestures.gesture_action_map`
rows (3.9), so `rejection` and every `palm_rejection.*` key are constants in palmrest; a file
that still has them loads, the keys are ignored. The section number stays so later links hold.

### 3.11 `menus` (retired)

Retired 2026-10-27: Settings keeps only what macOS offers, so the `menus.*` keys are
constants at their old defaults or stylesheet tokens in sill; a file that still has them
loads, the keys are ignored. The section number stays so later links hold.

### 3.12 `notifications` (sill/settings.toml)

`notifications.banner_material` is in 3.1 (it is `Appearance` vocabulary, read from
`appearance.toml`); everything else lives here.

| Key | Type | Default | Range / Alt | Source | Status |
| --- | --- | --- | --- | --- | --- |
| `notifications.dnd` | `Dnd::{On,Off}` | `Off` | | `13-BEHAVIOUR-menus-windows.md#13-6-configuration` | proposed |
| `notifications.banner_style` | `BannerStyle::{Banner,Alert,None}` per app | `Banner` | | `13-BEHAVIOUR-menus-windows.md#13-6-configuration` | settled (preference), R8 |
| `notifications.banner_entry_direction` | `BannerEntry::{FromRight,FromBelow}` | `FromRight` | | `05-MOTION.md#12-open-decisions` item 7 | RETIRING: no reader; a banner slides in from and out to the right (design/30 section 1.3) |
| `notifications.swipe` | `NotificationSwipe::{KeepInCenter,Dismiss}` | `KeepInCenter` | macOS keeps a swiped banner in Notification Center (design/13 said dismiss); to be confirmed when M6 lane a reports | `20-SURFACES.md#1-6-notifications`; `13-BEHAVIOUR-menus-windows.md#13-3-6`; sill M6 freeze | proposed (M6 freeze, 2026-09-25) |

### 3.13 `control_center` (sill/settings.toml)

Module list against the Claude Doc spec (rev 31): `13-BEHAVIOUR-menus-windows.md#13-9-open-decisions`
item 4 — **could not find a single sourced default**; the plan-derived order below is used
until that comparison happens (see handback report).

| Key | Type | Default | Range / Alt | Source | Status |
| --- | --- | --- | --- | --- | --- |
| `control_center.modules` | `Vec<ControlCenterModule>` | `[Wifi, Bluetooth, Focus, Display, KeyboardBrightness, Sound, NowPlaying, Appearance, Battery]` | KeyboardBrightness is drawn only where UPower reports a keyboard backlight | `13-BEHAVIOUR-menus-windows.md#13-3-7-control-center`; design/26 D2 (G25) | proposed (D2, 2026-09-27) |
| `control_center.menu_bar_wifi` | `InMenuBar::{Show,Hide}` | `Show` | the module as its own bar item, left of the control center item; a click opens that module's detail pane directly | `20-SURFACES.md#1-5-control-center`; user direction 2026-09-25 (controls at the top right, macOS "Show in Menu Bar") | proposed (M5 freeze, 2026-09-25) |
| `control_center.menu_bar_bluetooth` | `InMenuBar::{Show,Hide}` | `Hide` | the module as its own bar item, left of the control center item; a click opens that module's detail pane directly | `20-SURFACES.md#1-5-control-center`; user direction 2026-09-25 (controls at the top right, macOS "Show in Menu Bar") | proposed (M5 freeze, 2026-09-25) |
| `control_center.menu_bar_sound` | `InMenuBar::{Show,Hide}` | `Show` | the module as its own bar item, left of the control center item; a click opens that module's detail pane directly | `20-SURFACES.md#1-5-control-center`; user direction 2026-09-25 (controls at the top right, macOS "Show in Menu Bar") | proposed (M5 freeze, 2026-09-25) |
| `control_center.menu_bar_display` | `InMenuBar::{Show,Hide}` | `Hide` | the module as its own bar item, left of the control center item; a click opens that module's detail pane directly | `20-SURFACES.md#1-5-control-center`; user direction 2026-09-25 (controls at the top right, macOS "Show in Menu Bar") | proposed (M5 freeze, 2026-09-25) |
| `control_center.menu_bar_battery` | `InMenuBar::{Show,Hide}` | `Show` | the module as its own bar item, left of the control center item; a click opens that module's detail pane directly | `20-SURFACES.md#1-5-control-center`; user direction 2026-09-25 (controls at the top right, macOS "Show in Menu Bar") | proposed (M5 freeze, 2026-09-25) |
| `control_center.menu_bar_now_playing` | `InMenuBar::{Show,Hide}` | `Hide` | the module as its own bar item, left of the control center item; a click opens that module's detail pane directly | `20-SURFACES.md#1-5-control-center`; user direction 2026-09-25 (controls at the top right, macOS "Show in Menu Bar") | proposed (M5 freeze, 2026-09-25) |

### 3.14 `spaces` (retired)

Retired 2026-10-27: Settings keeps only what macOS offers, so the `spaces.*` keys are
constants at their old defaults or stylesheet tokens in sill; a file that still has them
loads, the keys are ignored. The section number stays so later links hold.

### 3.15 `display` (sill/settings.toml)

All Advanced (§5). The display service reads the EDID, classifies the panel's gamut, and drives DDC/CI. sill sets no scale: casement picks a display's first scale (`outputs.first_sight_scale`, 2026-10-19) and sill only shows it.

| Key | Type | Default | Range / Alt | Source | Status |
| --- | --- | --- | --- | --- | --- |
| `display.brightness_keys_target` | `BrightnessTarget::{PointerOutput,AllOutputs}` | `PointerOutput` | in practice the active window's output; DDC/CI | sill FINDINGS "Displays" (display service, 2026-09-25); `13-BEHAVIOUR-menus-windows.md` | proposed (2026-09-25) |
| `display.hardware_volume` | `HardwareVolume::{Off,WhenMonitorIsOutput}` | `WhenMonitorIsOutput` | DDC/CI volume when the monitor is the audio output | sill FINDINGS "Displays" (display service, 2026-09-25); `13-BEHAVIOUR-menus-windows.md` | proposed (2026-09-25) |
| `display.night_warmth` | `NightWarmth::{Off,Hardware}` | `Off` | warmth through the monitor's own DDC/CI controls | sill FINDINGS "Displays" (display service, 2026-09-25); `13-BEHAVIOUR-menus-windows.md` | proposed (2026-09-25) |
| `display.night_warmth_strength` | `Percent` | `50` | `0..=100` | sill FINDINGS "Displays" (display service, 2026-09-25); `13-BEHAVIOUR-menus-windows.md` | proposed (2026-09-25) |
| `display.font_rendering` | `FontRendering::{Auto,Off}` | `Auto` | `Auto` sets hinting and subpixel positioning from the output's ppi | sill FINDINGS "Displays" (display service, 2026-09-25); `13-BEHAVIOUR-menus-windows.md` | proposed (2026-09-25) |

### 3.16 `osd` (retired)

Retired 2026-10-27: the on-screen display's margin is a stylesheet token in sill, and the
`osd` domain is gone from sill's schema; a file that still has `osd.*` loads, the keys are
ignored. The section number stays so later links hold.

### 3.17 `power_menu` (retired)

Retired 2026-10-27: Settings keeps only what macOS offers, so the `power_menu.*` keys are
constants at their old defaults or stylesheet tokens in sill; a file that still has them
loads, the keys are ignored. The section number stays so later links hold.

### 3.18 `window` (retired)

Retired 2026-10-27: Settings keeps only what macOS offers, so the `window.*` keys are
constants at their old defaults or stylesheet tokens in sill; a file that still has them
loads, the keys are ignored. The section number stays so later links hold.

### 3.19 `session` (sill/settings.toml)

The parts a sill session borrows until M11 draws its own (design/20 §1.9 lock screen, §1.10 polkit prompt; sill FINDINGS "M7"): which lock screen and which polkit agent `dist/sill-session` starts. Both are a name from a fixed vocabulary or a command line, so a user can point at a program this list does not know. Page Accounts, all Advanced (§5), except `session.lock_grace_s`: it answers sill's own idle service (F863, F908), not the locker or the polkit agent, so it moved to the new Power page (§5) alongside `idle.*` (Q445). Idle timeouts are not sill's: the desktop idle daemon keeps them in the desktop's own config.

| Key | Type | Default | Range / Alt | Source | Status |
| --- | --- | --- | --- | --- | --- |
| `session.user_picture` | `PictureChoice::{Auto,Letter,Emoji(name),Photo}` written as a string | `auto` | `auto`, `letter`, `photo` or `emoji:<name>` (a stable name from quire's 42, design/25). Auto: the photo if `~/.face` (or AccountsService's icon) exists, else the letter; Photo with no file falls back to the letter | design/25 (user picture), quire `PictureChoice::resolve` | RETIRING: no reader; the picture is wired to `resolve_picture` first |
| `session.lock_grace_s` | `Secs` | `5` | `0..=60`; 0 always asks; only after sill's own idle screen-off lock (cause `IdleScreenOff`), never after a manual lock, the lid, suspend or resume | sill F863, F908 | proposed (Q441, 2026-09-27) |

### 3.20 `widgets` (retired)

Retired 2026-10-20: sill has one widget host (the notification center's tile) and no desktop widget
layer, so the six `widgets.*` keys (`center`, `desktop`, `desktop_widgets`, `desktop_cell_px`,
`desktop_gap_px`, `world_clocks`) are gone; a file that still has them loads, the keys are ignored.
The section number stays so links to 3.21 and later hold.

### 3.21 `calendar` (sill/settings.toml)

The calendar sources and the Calendar and Up Next widgets (design/20 §1.12; sill F401–F403): `.ics` files and vdir folders, watched; no CalDAV client of sill's own. Page Notifications, all Advanced.

| Key | Type | Default | Range / Alt | Source | Status |
| --- | --- | --- | --- | --- | --- |
| `calendar.sources` | `Text` | `"auto"` | `auto` (KOrganizer's file via Akonadi's iCal resource, `~/.local/share/sill/calendars`, `~/.calendars`), `off`, or `.ics` files and folders separated by `;` | `20-SURFACES.md#1-12-calendar-popup-spec-tier-2`; F401, F402 | proposed (M10 freeze, 2026-09-26) |
| `calendar.first_weekday` | `FirstWeekday::{Monday,Sunday}` | `Monday` | the month's first column (ISO 8601) | same; F403 | proposed (M10 freeze, 2026-09-26) |
| `calendar.week_numbers` | `WeekNumbers::{Hide,Show}` | `Hide` | a column of ISO week numbers | same; F403 | proposed (M10 freeze, 2026-09-26) |

### 3.22 `screenshot` (sill/settings.toml)

The floating thumbnail after a screenshot and the tool that takes one (design/20 §1.13; sill F406, F408): the thumbnail holds `ToastHold` (5200 ms) and pauses under the pointer; dragging it out is a file drag once shell-host has a drag source (G120). Page Keyboard and Shortcuts, all Advanced.

| Key | Type | Default | Range / Alt | Source | Status |
| --- | --- | --- | --- | --- | --- |
| `screenshot.thumbnail` | `ShotThumbnail::{Show,Hide}` | `Show` | the floating thumbnail after a screenshot (macOS's "Show Floating Thumbnail") | `20-SURFACES.md#1-13-screenshot-thumbnail-spec-integrated-experience-priority-pick`; F408 | proposed (M10 freeze, 2026-09-26) |
| `screenshot.folders` | `Text` | `"auto"` | `auto` (Pictures and Pictures/Screenshots), `off`, or folders separated by `;`, watched for new screenshots | same; F406 | proposed (M10 freeze, 2026-09-26) |

### 3.23 `hot_corners` (sill/settings.toml)

One invisible square per enabled corner (design/20 §1.16; design/13 §13.3.12 for the dwell, re-arm, modifier and action rules; sill F409). Page Dock; the four corner keys are drawn by one custom control (`corners`) and the timing and the hot spot's size (150 ms dwell, 500 ms re-arm, 2 px) are constants of the machine, not settings (2026-10-27).

| Key | Type | Default | Range / Alt | Source | Status |
| --- | --- | --- | --- | --- | --- |
| `hot_corners.top_left` | `Text`, control `corners`: a list of bindings (`CornerBindings`) | `[]` | what the corner does; empty is inactive. Each binding is a table `{ keys, action, ... }`: `keys` is a list of `command`, `option`, `control`, `shift` held exactly (`[]` when absent), `action` is a `CornerAction` word (`launcher`, `notification_center`, `control_center`, `show_desktop`, `lock`, `workspaces`, `app_windows`, `display_sleep`, `assistant`), or `app_action` (with `app`, `intent`), `open` (with one of `app`, `path`, `url`) or `command` (with `command`, run by `/bin/sh -c`). At most one binding per set of keys; a bad or duplicate binding is dropped and reported. The older `top_left = "none"\|"lock"\|"command"` with `top_left_command` is read and never written back | `20-SURFACES.md#1-16-hot-corners-spec-integrated-experience`; F409; owner 2026-10-22 | proposed (hot corners beyond the Mac, 2026-10-22) |
| `hot_corners.top_right` | as `top_left` | `[]` | as `top_left` | same | proposed (hot corners beyond the Mac, 2026-10-22) |
| `hot_corners.bottom_left` | as `top_left` | `[]` | as `top_left` | same | proposed (hot corners beyond the Mac, 2026-10-22) |
| `hot_corners.bottom_right` | as `top_left` | `[]` | as `top_left` (macOS ships Quick Note here; sill has none until M12) | same | proposed (hot corners beyond the Mac, 2026-10-22) |

### 3.24 `idle` (sill/settings.toml)

sill's own idle service, replacing the desktop idle daemon in a sill session (the user's pick, 2026-09-27): dim with an overlay (never real brightness), screen off through output power, lock as the displays go off, suspend through logind; `org.freedesktop.ScreenSaver.Inhibit`, Wayland idle-inhibit and logind inhibitors are honoured (sill FINDINGS "sill idle (Q420 B)", F900-F913). Page Power (§5, Q445; before that landed, "Page Accounts for now").

| Key | Type | Default | Range / Alt | Source | Status |
| --- | --- | --- | --- | --- | --- |
| `idle.screen_off_ac_min` | `Mins` | `15` | `0..=240`, 0 never | sill FINDINGS "sill idle (Q420 B)", F901 | proposed (Q441, 2026-09-27) |
| `idle.screen_off_battery_min` | `Mins` | `15` | `0..=240`, 0 never | sill FINDINGS "sill idle (Q420 B)", F901 | proposed (Q441, 2026-09-27) |
| `idle.suspend_ac_min` | `Mins` | `30` | `0..=480`, 0 never | sill FINDINGS "sill idle (Q420 B)", F901 | proposed (Q441, 2026-09-27) |
| `idle.suspend_battery_min` | `Mins` | `15` | `0..=480`, 0 never | sill FINDINGS "sill idle (Q420 B)", F901 | proposed (Q441, 2026-09-27) |
| `idle.lock` | `IdleLockAt::{ScreenOff,Never}` | `ScreenOff` |  | sill FINDINGS "sill idle (Q420 B)", F904 | proposed (Q441, 2026-09-27) |

### 3.25 `keyboard` (sill/settings.toml)

How sill treats the keyboard per app (keycap, the key-cap overlay: which keycap profile an app's keys are drawn with). Page Keyboard and Shortcuts, Advanced (§5).

| Key | Type | Default | Range / Alt | Source | Status |
| --- | --- | --- | --- | --- | --- |
| `keyboard.overrides` | list of tables `{ app, profile }` | empty | one row per app: `app` is the desktop-entry id (`org.gnome.Nautilus`) and `profile` a keycap profile name (`[a-z0-9_-]{1,64}`); an invalid `profile` drops that row at load; beats the entry's `X-Keycap-Profile` and its category; a name keycap does not define resolves to `default` | keycap README "Profiles are open" | proposed (2026-10-01) |

### 3.26 `ai` (inferd, through its settings module)

The model layer: where data may go, which model fills each tier, how the engines are supervised. Page Intelligence, section Models (`ai.local_only`, `ai.floor.*`, `ai.model.*`) and Advanced (`ai.engine.*`, `ai.cua.*`). The owning daemon's repo names its file (its schema ships as porter's `dist/inferd.settings.toml`); the Intelligence page reads that schema (section 9.2) and the live picker rows (section 9.4).

| Key | Type | Default | Range / Alt | Source | Status |
| --- | --- | --- | --- | --- | --- |
| `ai.local_only` | `LocalOnly::{Off,On}` | `Off` | `On` refuses every cloud backend whatever the per-class floor says | models.md section 3.8; ux.md section 3.7 | proposed (companion freeze, 2026-10-02) |
| `ai.floor.<class>` | `Locality::{OnDevice,LocalNetwork,Cloud}`, one key per data class | porter's per-class floor (31 section 2.3) | `Screen` is always `OnDevice` and cannot be raised here | models.md section 3.8; actions.md | proposed (companion freeze, 2026-10-02) |
| `ai.floor.voice` | `Locality::{OnDevice,LocalNetwork,Cloud}` | `OnDevice` | the person's own voice audio (speech in); speech out uses the class of the text it speaks | voice.md section 6; porter-infer policy.rs | proposed (agent stack, 2026-10-06) |
| `ai.floor.prompt` | `Locality::{OnDevice,LocalNetwork,Cloud}` | `OnDevice` | what the person types or says to the companion | porter-infer policy.rs | proposed (agent stack, 2026-10-06) |
| `ai.model.<kind>.<tier>` | text, `"<account>/<model>"`, one key per kind and tier | empty: the catalogue's own pick | written by the Intelligence page through inferd's settings module (22 section 9.4); also `"auto"` (Automatic: inferd picks per request, never written on the person's behalf; a named pick is never overridden and refuses with a reason when it cannot serve, QUESTIONS R1, R2); the named roles (Planner, Reader, Reviewer, Operator) map onto tiers (QUESTIONS M3) | models.md section 3.8 | proposed (companion freeze, 2026-10-02) |
| `ai.auto.mode` | `AutoMode::{WarmFirst}` | `WarmFirst` | how Automatic chooses: prefer the model already loaded when it can serve, inside the floor; room for later modes | research-routing-fit.md section 4; QUESTIONS R1 | proposed (agent stack, 2026-10-06) |
| `ai.auto.allow_evict` | `AutoEvict::{Never,IdleOnly}` | `IdleOnly` | whether Automatic may unload an idle model to load another; the swap is always shown ("Loading <model>, <why>"), and an engine mid-turn is never evicted (QUESTIONS R4) | research-routing-fit.md section 4 | proposed (agent stack, 2026-10-06) |
| `ai.auto.show_reason` | `ShowReason::{Off,On}` | `On` | each answer names the model that served it and why ("already running", "needs images") | QUESTIONS R1 | proposed (agent stack, 2026-10-06) |
| `ai.spend.warn_permille` | `Permille` | `800` | `0..=1000`; the share of a spend cap that warns (31 section 6, SpendCap.warn_at) | design/31; porter-infer spend.rs | proposed (agent stack, 2026-10-06) |
| `ai.engine.idle_unload_s` | `Secs` | `600` | `0..=86400`, 0 never; the model stays warm while the companion is open (QUESTIONS M4) | models.md section 3.7 | proposed (companion freeze, 2026-10-02) |
| `ai.engine.start_timeout_s` | `Secs` | `180` | `10..=900`; vLLM's cold start | models.md section 3.7 | proposed (companion freeze, 2026-10-02) |
| `ai.engine.probe_ms` | `Ms` | `500` | `50..=5000`; the readiness probe interval | models.md section 3.7 | proposed (companion freeze, 2026-10-02) |
| `ai.engine.max_attempts` | `Count` | `3` | `1..=10`; restarts before the engine is marked unavailable | models.md section 3.7 | proposed (companion freeze, 2026-10-02) |
| `ai.engine.backoff_min_ms` | `Ms` | `2000` | `100..=60000`; the first wait between restarts | models.md section 3.7 ("2000 to 30000") | proposed (companion freeze, 2026-10-02) |
| `ai.engine.backoff_max_ms` | `Ms` | `30000` | `1000..=65000`; the longest wait | models.md section 3.7 | proposed (companion freeze, 2026-10-02) |
| `ai.engine.vram_headroom_mib` | `Count` | `1024` | `0..=16384` MiB left free when an engine loads | models.md section 3.7 | proposed (companion freeze, 2026-10-02) |
| `ai.engine.vllm.python` | text, a path | `~/vllm/.venv/bin/python` | the uv environment vLLM runs from | models.md section 3.7 | proposed (companion freeze, 2026-10-02) |
| `ai.engine.llama_server.path` | text, a path | `llama-server` | looked up on the engine unit's `PATH` when it has no slash | models.md section 3.7 | proposed (companion freeze, 2026-10-02) |
| `ai.cua.history_frames` | `Count` | `3` | `1..=8`; screenshots a computer-use step keeps in the prompt | models.md section 3.8 | proposed (companion freeze, 2026-10-02) |
| `ai.cua.repair_attempts` | `Count` | `1` | `0..=3`; re-asks after an unparseable step | models.md section 3.8 | proposed (companion freeze, 2026-10-02) |
| `ai.structured.open_text` | `Count` | `4096` | `256..=65536` characters; the longest free-text value a structured output may carry | inferd's structured-output limits (ask 143) | proposed (agent stack, 2026-10-06) |
| `ai.structured.open_list` | `Count` | `256` | `16..=4096`; the most items an open list in a structured output may carry | inferd's structured-output limits (ask 143) | proposed (agent stack, 2026-10-06) |
| `ai.structured.depth` | `Count` | `16` | `4..=64`; the deepest nesting a structured output may have | inferd's structured-output limits (ask 143) | proposed (agent stack, 2026-10-06) |
| `ai.structured.repair_budget` | `Count` | `1` | `0..=3`; re-asks after a structured output fails its schema | inferd's structured-output limits (ask 143) | proposed (agent stack, 2026-10-06) |


### 3.27 `agent` (docket)

How far the companion may act on its own and what it may spend. Page Intelligence, section Privacy (`agent.strictness`, `agent.mcp.expose`, `agent.undo.keep_h`); the rest Advanced. Every value of the budget table is a key, never a constant (QUESTIONS S5); the person may lower a budget, and only the person may raise the false-negative targets of the evaluation gates, which are not keys. The keys live in `$XDG_CONFIG_HOME/docket/settings.toml`; the schema ships as docket's `dist/settings/docket.settings.toml` (hands-off for agents), and actions-mcp reads `agent.mcp.expose` from the same file (`actions-mcp --write-schema <dir>` writes its part).

| Key | Type | Default | Range / Alt | Source | Status |
| --- | --- | --- | --- | --- | --- |
| `agent.strictness` | `Strictness::{AskMore,Default,TrustMore}` | `Default` | per Space override; `TrustMore` is the only way outbound or destructive calls run without asking (QUESTIONS S1, S2) | actions.md section 3.9, addendum A3 | proposed (companion freeze, 2026-10-02) |
| `agent.review.quick_ms` | `Ms` | `300` | `50..=10000`; a timeout is an Ask, never an Allow | actions-addendum.md (reviewers) | proposed (companion freeze, 2026-10-02) |
| `agent.review.deliberate_ms` | `Ms` | `3000` | `50..=30000` | actions-addendum.md | proposed (companion freeze, 2026-10-02) |
| `agent.review.second_ms` | `Ms` | `3000` | `50..=30000` | actions-addendum.md | proposed (companion freeze, 2026-10-02) |
| `agent.budget.calls` | `Count` | `200` | per session | actions.md section 3.7 | proposed (companion freeze, 2026-10-02) |
| `agent.budget.writes` | `Count` | `100` | per session | actions.md section 3.7 | proposed (companion freeze, 2026-10-02) |
| `agent.budget.outbound` | `Count` | `10` | per session | actions.md section 3.7 | proposed (companion freeze, 2026-10-02) |
| `agent.budget.destructive` | `Count` | `5` | per session | actions.md section 3.7 | proposed (companion freeze, 2026-10-02) |
| `agent.budget.per_minute` | `Count` | `30` | calls a minute | actions.md section 3.7 | proposed (companion freeze, 2026-10-02) |
| `agent.budget.fan_out` | `Count` | `50` | entities one call may touch | actions.md section 3.7 | proposed (companion freeze, 2026-10-02) |
| `agent.budget.chain` | `Count` | `4` | calls one call may start | actions.md section 3.7 | proposed (companion freeze, 2026-10-02) |
| `agent.budget.wall_s` | `Secs` | `1800` | a session's wall time | actions.md section 3.7 | proposed (companion freeze, 2026-10-02) |
| `agent.budget.reviews` | `Count` | `300` | reviewer calls a session | actions.md section 3.7 | proposed (companion freeze, 2026-10-02) |
| `agent.budget.denials_in_a_row` | `Count` | `3` | then the session pauses for the person (the breaker) | actions.md section 3.7; SPEC C12 | proposed (companion freeze, 2026-10-02) |
| `agent.budget.spend_microusd` | `MicroUsd` | `0` | 0 is local only; shared with inferd's spend cap | actions.md section 3.7 | proposed (companion freeze, 2026-10-02) |
| `agent.task_policy.max_min` | `Mins` | `60` | `1..=480`; a task policy lasts until the task ends, capped here (QUESTIONS S3) | QUESTIONS S3 | proposed (companion freeze, 2026-10-02) |
| `agent.undo.keep_h` | `Count` | `24` | `1..=168` h; how long the undo journal keeps an entry | ux.md section 3.7, SPEC section 2 | proposed (companion freeze, 2026-10-02) |
| `agent.mcp.expose` | `McpExpose::{Off,On}` | `Off` | whether external MCP clients see the actions (QUESTIONS S7) | QUESTIONS S7 | proposed (companion freeze, 2026-10-02) |


### 3.28 `memory` (almanac)

What the companion remembers and for how long. Page Intelligence, section Memory (`memory.files.at_rest`, `memory.consolidation.*`); the retention rows Advanced. Things (mail, files, events) are kept while their source exists and facts until forgotten; neither is a key.

| Key | Type | Default | Range / Alt | Source | Status |
| --- | --- | --- | --- | --- | --- |
| `memory.files.at_rest` | `AtRest::{Sealed,Plain}` | `Sealed` | per Space; sealed files open in the editor through tmpfs (QUESTIONS Me1) | memory.md; QUESTIONS Me1 | proposed (companion freeze, 2026-10-02) |
| `memory.retention.search_days` | `Count` | `30` | `1..=3650` d; searches and selections | memory.md section 3.2 | proposed (companion freeze, 2026-10-02) |
| `memory.retention.file_unexplained_days` | `Count` | `7` | `1..=3650` d; file changes no app explained | memory.md section 3.2 | proposed (companion freeze, 2026-10-02) |
| `memory.retention.session_days` | `Count` | `30` | `1..=3650` d; session and computer-use records | memory.md section 3.2 | proposed (companion freeze, 2026-10-02) |
| `memory.retention.audit_body_days` | `Count` | `90` | `1..=3650` d; policy, consent and memory audit bodies | memory.md section 3.2 | proposed (companion freeze, 2026-10-02) |
| `memory.retention.audit_header_days` | `Count` | `365` | `1..=3650` d; their headers | memory.md section 3.2 | proposed (companion freeze, 2026-10-02) |
| `memory.pending_ttl_days` | `Count` | `14` | `1..=365` d; a pending fact waits this long for the person (QUESTIONS Me5) | memory.md; QUESTIONS Me5 | proposed (companion freeze, 2026-10-02) |
| `memory.join_window_ms` | `Ms` | `2000` | `100..=10000`; the window in which a file change and an app's reason are one event | memory.md section 4.6 | proposed (companion freeze, 2026-10-02) |
| `memory.consolidation.when` | `ConsolidateWhen::{Nightly,Manual,Never}` | `Nightly` |  | memory.md section 4.4 | proposed (companion freeze, 2026-10-02) |
| `memory.consolidation.apply` | `ConsolidateApply::{Auto,Review}` | `Auto` | `Auto` applies tidy, stamp and trusted hunks, revertible, with the diff visible; `Review` waits for each hunk (QUESTIONS Me4) | memory.md; QUESTIONS Me4 | proposed (companion freeze, 2026-10-02) |


### 3.29 `cua` (cua)

Computer use, off by default and on per app. Page Intelligence, section Computer use (`cua.enabled`, `cua.apps`, `cua.default_mode`, `cua.autopause`, `cua.accessibility`); the rest Advanced. The per-run `denials` budget is deleted: the router's breaker (`agent.budget.denials_in_a_row`) counts them (SPEC C12).

| Key | Type | Default | Range / Alt | Source | Status |
| --- | --- | --- | --- | --- | --- |
| `cua.enabled` | `CuaEnabled::{Off,On}` | `Off` | while `Off` the computer-use provider is `Hidden` and no pixel step runs | cua.md section 3.8; QUESTIONS P2, P3 | proposed (companion freeze, 2026-10-02) |
| `cua.apps` | list of tables `{ app, allow }` | empty: no app is on | `app` a desktop-entry id; `allow` is `ask`, `session` or `always`; a browser never takes `always` (QUESTIONS S9) | cua.md; QUESTIONS S9 | proposed (companion freeze, 2026-10-02) |
| `cua.default_mode` | `CuaDefaultMode::{Auto,InPlace,AgentWorkspace}` | `Auto` | `Auto` is the agent workspace where the compositor supports it, else in place (QUESTIONS U11) | SPEC section 2; QUESTIONS U11 | proposed (companion freeze, 2026-10-02) |
| `cua.autopause` | `CuaAutopause::{On,Off}` | `On` | pause a run that shares the seat when the person touches the window | cua.md section 4 | proposed (companion freeze, 2026-10-02) |
| `cua.accessibility` | `CuaAccessibility::{On,Off}` | `On` | read the controls of the window a run is using (that window only), so a step names the control it clicks; `Off` goes on from pixels. Row "Read app controls" | owner, 2026-10-20 | settled (owner, 2026-10-20) |
| `cua.budget.steps` | `Count` | `40` | `1..=500` per run | cua.md section 7 | proposed (companion freeze, 2026-10-02) |
| `cua.budget.active_s` | `Secs` | `600` | `10..=7200` | cua.md section 7 | proposed (companion freeze, 2026-10-02) |
| `cua.budget.stall_frames` | `Count` | `3` | `1..=20`; identical frames before a run counts as stalled | cua.md section 7 | proposed (companion freeze, 2026-10-02) |
| `cua.budget.actions_per_min` | `Count` | `60` | `1..=600` | cua.md section 7 | proposed (companion freeze, 2026-10-02) |
| `cua.budget.model_failures` | `Count` | `2` | `0..=10` | cua.md section 7 | proposed (companion freeze, 2026-10-02) |
| `cua.budget.spend_microusd` | `MicroUsd` | `0` | 0 is local only | cua.md section 7 | proposed (companion freeze, 2026-10-02) |
| `cua.settle.quiet_ms` | `Ms` | `250` | `50..=2000`; the screen is settled after this much quiet | cua.md section 7 | proposed (companion freeze, 2026-10-02) |
| `cua.settle.timeout_ms` | `Ms` | `3000` | `500..=30000` | cua.md section 7 | proposed (companion freeze, 2026-10-02) |
| `cua.wait.max_ms` | `Ms` | `5000` | `500..=60000`; the longest a `Wait` action may ask | cua.md section 7 | proposed (companion freeze, 2026-10-02) |
| `cua.tree.max_nodes` | `Count` | `400` | `50..=5000`; accessibility nodes sent to the model | cua.md section 3 | proposed (companion freeze, 2026-10-02) |
| `cua.screens.keep` | `CuaKeepScreens::{Off,On}` | `Off` | `On` keeps thumbnails of each step in the Space's encrypted store (QUESTIONS S11) | cua.md section 5; QUESTIONS S11 | proposed (companion freeze, 2026-10-02) |
| `cua.screens.keep_days` | `Count` | `7` | `1..=365` d | cua.md section 5 | proposed (companion freeze, 2026-10-02) |


### 3.30 `companion` (sill/settings.toml)

The summon and the confirmation. Page Intelligence, sections Companion and Confirmations. The undo window, the default run place and the touch pause moved to `agent.undo.keep_h`, `cua.default_mode` and `cua.autopause` (SPEC section 2). `companion.double_tap_ms` and `companion.confirm_arm_ms` are Advanced.

| Key | Type | Default | Range / Alt | Source | Status |
| --- | --- | --- | --- | --- | --- |
| `companion.summon` | `SummonKey::{DoubleTapCommand,Off}` | `DoubleTapCommand` | a toggle: "Press ⌘ twice to ask" | ux.md section 3.7 | proposed (companion freeze, 2026-10-02) |
| `companion.in_field` | `InField::{Prompt,Launcher}` | `Prompt` | whether a summon in a text field makes the field the prompt or opens the launcher | ux.md section 3.7 | proposed (companion freeze, 2026-10-02) |
| `companion.notify` | `NotifyWhen::{NeedsYou,Never}` | `NeedsYou` | a toggle: "Notify when it needs you" | ux.md section 3.7 | proposed (companion freeze, 2026-10-02) |


### 3.31 `viewer` (anyview, `$XDG_CONFIG_HOME/anyview/settings.toml`)

The file viewer's timings, steps, history and the limits of its decoders. Page Viewer: section Window (`viewer.chrome.hide_after`, `viewer.warm_for`), section Viewing (`viewer.zoom.step`, `viewer.media.seek_step`), section History (`viewer.history.cap`, `viewer.versions.keep`), section Export (`viewer.export.*`); `viewer.peek.*` and `viewer.image.*` are Advanced. A key's unit is its type, not its name: `viewer.chrome.hide_after` is `Ms` and `viewer.warm_for` is `Mins`. The reveal and hide fades are the motion token `--t-quick` and are not keys. Two rows have no constant in anyview yet and are the values proposed here: `viewer.versions.keep` (30 days) and the three `viewer.peek` budget rows (taken from the launcher pane's test budget).

| Key | Type | Default | Range / Alt | Source | Status |
| --- | --- | --- | --- | --- | --- |
| `viewer.chrome.hide_after` | `Ms` | `2000` | `500..=30000`; idle time before the viewer's toolbar and title fade out | anyview `chrome/model.rs` `ChromeParams::HIDE_AFTER`; design/20 section 2.6 | proposed (viewer join, 2026-10-03) |
| `viewer.zoom.step` | `Fraction` | `1250` | `1001..=4000`; the factor of one zoom step in thousandths (1250 is x1.25); the stages floor it at 1001 | anyview `stage/raster/model.rs`, `stage/pdf/model.rs` `step: Permille(1250)` | proposed (viewer join, 2026-10-03) |
| `viewer.media.seek_step` | `Secs` | `5` | `1..=300`; how far a seek key jumps in a recording | anyview `stage/media/model.rs` `MediaParams::default` | proposed (viewer join, 2026-10-03) |
| `viewer.warm_for` | `Mins` | `10` | `0..=1440`; how long the process stays after the last window closes, so the next file opens warm; 0 quits at once | anyview `crates/anyview/src/program/start.rs` `WARM_FOR` | proposed (viewer join, 2026-10-03) |
| `viewer.history.cap` | `Count` | `200` | `1..=10000`; how many files the recently-viewed history keeps | anyview `crates/anyview-store/src/history.rs` `HistoryCap::DEFAULT` | proposed (viewer join, 2026-10-03) |
| `viewer.versions.keep` | `Count` | `30` | `1..=3650` days; how long the original of a file the viewer saved over is kept for Revert To | anyview PLAN.md section E (no constant yet) | proposed (viewer join, 2026-10-03) |
| `viewer.export.jpeg_quality` | `Percent` | `90` | `1..=100`; the quality an image or frame exported as JPEG starts at | anyview `crates/anyview-core/src/export/target.rs` `default_jpeg` | proposed (viewer join, 2026-10-03) |
| `viewer.export.avif_quality` | `Percent` | `65` | `1..=100`; the quality an image or frame exported as AVIF starts at | anyview `export/target.rs` `default_avif` | proposed (viewer join, 2026-10-03) |
| `viewer.export.paper` | `PaperSize::{A4,A3,Letter,Legal}` | `A4` | the paper a text or document export to PDF or print is laid out on | anyview `export/layout.rs` `PaperSize` default | proposed (viewer join, 2026-10-03) |
| `viewer.export.orientation` | `Orientation::{Portrait,Landscape}` | `Portrait` | which way up that paper is | anyview `export/layout.rs` `Orientation` default | proposed (viewer join, 2026-10-03) |
| `viewer.peek.bytes_mib` | `Count` | `4` | `1..=256` MiB; the most a launcher peek reads from a file | anyview-peek `tests/support/mod.rs` `pane_budget` (4,000,000 bytes; no production constant) | proposed (viewer join, 2026-10-03) |
| `viewer.peek.megapixels` | `Count` | `1` | `1..=64` Mpx; the most pixels a launcher peek decodes | same (1,000,000 pixels) | proposed (viewer join, 2026-10-03) |
| `viewer.peek.time_ms` | `Ms` | `500` | `50..=10000`; the longest a launcher peek may take before it is abandoned | same (500 ms) | proposed (viewer join, 2026-10-03) |
| `viewer.peek.lines` | `Count` | `40` | `5..=500`; lines of a text or code file a peek shows | anyview-text `peek/head.rs` `PEEK_LINES` | proposed (viewer join, 2026-10-03) |
| `viewer.peek.detect_kib` | `Count` | `1024` | `4..=65535` KiB; how much of a text file is read to guess its encoding | anyview-text `encoding.rs` `DETECT_BYTES` (1 MiB) | proposed (viewer join, 2026-10-03) |
| `viewer.peek.json_keys` | `Count` | `8` | `1..=64`; top-level keys a JSON peek names | anyview-text `peek/tree.rs` `KEYS_NAMED` | proposed (viewer join, 2026-10-03) |
| `viewer.peek.highlight_line_bytes` | `Count` | `4096` | `256..=65535`; the longest line syntax highlighting is run on | anyview-text `code/state.rs` `MAX_PARSED_LINE` | proposed (viewer join, 2026-10-03) |
| `viewer.image.max_decode_mpx` | `Count` | `256` | `16..=4096` Mpx (2^20 pixels; 256 is 16384 by 16384); an image that declares more is refused, not decoded | anyview-image `decode/mod.rs` `MAX_DECODE_AREA` | proposed (viewer join, 2026-10-03) |
| `viewer.image.max_animation_mib` | `Count` | `512` | `16..=16384` MiB; the most memory the frames of one animation may take | anyview-image `decode/mod.rs` `MAX_ANIMATION_BYTES` | proposed (viewer join, 2026-10-03) |
| `viewer.image.svg_edge_min_px` | `Px` | `1024` | `256..=8192`; an SVG shown in full is drawn with its long edge at least this long | anyview-image `decode/mod.rs` `SVG_LONG_EDGE_MIN` | proposed (viewer join, 2026-10-03) |
| `viewer.image.svg_edge_max_px` | `Px` | `4096` | `1024..=16384`; and at most this long | anyview-image `decode/mod.rs` `SVG_LONG_EDGE_MAX` | proposed (viewer join, 2026-10-03) |
| `viewer.image.avif_speed` | `Count` | `6` | `1..=10`; the AVIF encoder's speed, 1 smallest and slowest | anyview-image `encode/avif.rs` `SPEED` | proposed (viewer join, 2026-10-03) |
| `viewer.image.markdown_inline_mib` | `Count` | `8` | `1..=64` MiB; the largest local image a rendered Markdown file inlines | anyview-text `markdown/images.rs` `MAX_INLINE` | proposed (viewer join, 2026-10-03) |


### 3.32 `region` (quire/appearance.toml)

The desktop-wide language and region preferences (owner decision 2026-10-20: one 12/24-hour
preference for every clock, defaulting from the locale). The table below is generated from the
schema (`scripts/gen-doc-keys.sh`).

<!-- keys:begin region -->

| Key | Type | Default | Range | Page | Shown | Agent |
| --- | --- | --- | --- | --- | --- | --- |
| `region.hours` | choice {auto, 12, 24} | `auto` |  | Appearance | Settings |  |

<!-- keys:end region -->

`region.hours` is `auto`, `12` or `24`. `auto` follows the locale of the session
(`ds_settings::time_locale` picks it from `LC_ALL`, `LC_TIME` and `LANG`): `en_US`, `en_CA`,
`zh_TW` and a few more write 12 hours, `en_GB`, `de_DE`, `ja_JP` and `C` write 24. A program asks
`Environment::hours(locale)` (or `HourCycle::resolve`) and draws what comes back; it never reads
the key itself. The bar, the calendar and mail's snooze times read the same answer. So does the lock screen: its
clock follows `region.hours` and, like the Mac's, never shows AM/PM (`14:05`, or `2:05`); there is
no lock-clock key of its own.

### 3.33 `wallpaper` (sill/settings.toml)

The desktop picture. Page Wallpaper (the `wallpaper` page of §9.3), where the `picture` control
shows a grid of pictures with "Add Picture...".

| Key | Type | Default | Range / Alt | Source | Status |
| --- | --- | --- | --- | --- | --- |
| `wallpaper.picture` | text, control `picture` | empty | a path to an image; empty means no picture, so the desktop paints `--paper` | sill wallpaper; §9.1 `picture` control | proposed (2026-10-20) |

## 4. Rust shape

Adds to `crates/ds-settings` (appearance/icons/motion) and a new `sill-settings` module
(bar/dock/launcher/scroll/scrollbar/menus/notifications/control_center/spaces); `gestures`/
`palm_rejection` live in `palmrest`'s own crate but follow the identical shape so the three
loaders share one macro/derive story.

Shared newtypes (beyond the four the catalogue names, added per `CONVENTIONS.md#4-types`
"newtype every unit and identifier"):

```rust
pub struct Px(pub u16);
pub struct Ms(pub u16);
pub struct Percent(pub u8);           // 0..=100, clamped on construction
pub struct Fraction(pub u16);         // permille: 1000 = 1.0; not clamped (gains exceed 1.0)
pub struct Count(pub u16);
pub struct Secs(pub u16);             // seconds (Q445: idle.dim_s, idle.locked_screen_off_s, session.lock_grace_s)
pub struct Mins(pub u16);             // minutes (Q445: idle.screen_off_*_min, idle.suspend_*_min)
pub struct Scalar(pub f32);           // dimensionless physics constant, no natural unit above
pub struct Units(pub i32);            // raw touchpad/report units, signed
```

`Secs` and `Mins` are schema-rendered exactly like `Count`, `Px` and `Ms`: the field's own
`#[settings(range = "...", unit = "s")]` (or `"min"`) attribute gives the derive its
`KeyKind::Bounded` floor, ceiling and unit label (section 9.1) — the newtype itself carries no
range, so construction never fails and a lenient parse falls back to the field's default exactly
as `Count` does (section 6 acceptance item 2's "negative where `Px` (`u16`) expected" row, read
for `Secs`/`Mins` too).

Every domain is one small struct, one field per key, `#[derive(Clone, Debug, PartialEq, Serialize, Deserialize)]`,
`#[serde(default)]` on every field (every field has a shipped default, by construction). Enums use
`#[serde(rename_all = "snake_case")]` and a lenient `Deserialize` that falls back to
`Default::default()` on an unknown variant rather than erroring (section 2's "lenient" rule),
via a small `#[serde(deserialize_with = "lenient")]` helper rather than hand-rolled `Visitor`
impls per field.

### 4.1 `DockSettings` (full)

```rust
#[derive(Clone, Debug, PartialEq, Serialize, Deserialize)]
#[serde(default)]
pub struct DockSettings {
    pub magnification: Magnification,
    pub tile_size_px: Px,
    pub magnified_size_px: Px,
    pub influence_radius_px: Px,
    pub tile_gap_px: Px,
    pub progress_style: ProgressStyle,
    pub autohide: AutoHide,
    pub autohide_delay_ms: Ms,
    pub autohide_slide_ms: Ms,
    pub autohide_trigger_strip_px: Px,
    pub position: DockPosition,
    pub indicators: Indicators,
    pub click_active_app: ActiveClick,
    pub trash: TrashTile,
    pub pill_radius_px: Px,
    pub edge_clamp_px: Px,
    pub running_dot_diameter_px: Px,
    pub badge_size_px: Px,
    pub badge_radius_px: Px,
    pub progress_ring_diameter_px: Px,
    pub progress_ring_stroke_px: Px,
    pub separator_height_px: Px,
    pub magnify_enter_ms: Ms,
    pub magnify_leave_ms: Ms,
    pub hover_label_warm_ms: Ms,
    pub hold_to_menu_ms: Ms,
    pub hold_to_menu_move_px: Px,
    pub remove_threshold_px: Px,
    pub spring_load_ms: Ms,
    pub context_menu_order: DockMenuOrder,
    pub modifier_clicks: DockModifierClicks,
}

#[derive(Clone, Copy, Debug, PartialEq, Eq, Default, Serialize, Deserialize)]
#[serde(rename_all = "snake_case")]
pub enum Magnification { #[default] On, Off }
#[derive(Clone, Copy, Debug, PartialEq, Eq, Default, Serialize, Deserialize)]
#[serde(rename_all = "snake_case")]
pub enum AutoHide { #[default] Off, On }
#[derive(Clone, Copy, Debug, PartialEq, Eq, Default, Serialize, Deserialize)]
#[serde(rename_all = "snake_case")]
pub enum DockPosition { #[default] Bottom, Left, Right }
#[derive(Clone, Copy, Debug, PartialEq, Eq, Default, Serialize, Deserialize)]
#[serde(rename_all = "snake_case")]
pub enum Indicators { #[default] On, Off }
#[derive(Clone, Copy, Debug, PartialEq, Eq, Default, Serialize, Deserialize)]
#[serde(rename_all = "snake_case")]
pub enum ActiveClick { #[default] Cycle, Nothing }
#[derive(Clone, Copy, Debug, PartialEq, Eq, Default, Serialize, Deserialize)]
#[serde(rename_all = "snake_case")]
pub enum TrashTile { #[default] On, Off }
#[derive(Clone, Copy, Debug, PartialEq, Eq, Default, Serialize, Deserialize)]
#[serde(rename_all = "snake_case")]
pub enum DockMenuOrder { #[default] WindowsFirst, AppleOrder }
#[derive(Clone, Copy, Debug, PartialEq, Eq, Default, Serialize, Deserialize)]
#[serde(rename_all = "snake_case")]
pub enum DockModifierClicks { #[default] AppleMapping, Off }
#[derive(Clone, Copy, Debug, PartialEq, Eq, Default, Serialize, Deserialize)]
#[serde(rename_all = "snake_case")]
pub enum ProgressStyle { #[default] Ring, Bar }

impl Default for DockSettings {
    fn default() -> Self {
        Self {
            magnification: Magnification::On,
            tile_size_px: Px(48),
            magnified_size_px: Px(72),
            influence_radius_px: Px(72),
            tile_gap_px: Px(4),
            progress_style: ProgressStyle::Ring,
            autohide: AutoHide::Off,
            autohide_delay_ms: Ms(200),
            autohide_slide_ms: Ms(500),
            autohide_trigger_strip_px: Px(4),
            position: DockPosition::Bottom,
            indicators: Indicators::On,
            click_active_app: ActiveClick::Cycle,
            trash: TrashTile::On,
            pill_radius_px: Px(22),
            edge_clamp_px: Px(8),
            running_dot_diameter_px: Px(4),
            badge_size_px: Px(18),
            badge_radius_px: Px(9),
            progress_ring_diameter_px: Px(20),
            progress_ring_stroke_px: Px(3),
            separator_height_px: Px(36),
            magnify_enter_ms: Ms(120),
            magnify_leave_ms: Ms(200),
            hover_label_warm_ms: Ms(400),
            hold_to_menu_ms: Ms(600),
            hold_to_menu_move_px: Px(8),
            remove_threshold_px: Px(64),
            spring_load_ms: Ms(500),
            context_menu_order: DockMenuOrder::WindowsFirst,
            modifier_clicks: DockModifierClicks::AppleMapping,
        }
    }
}
```

### 4.2 `ScrollSettings` (full; `ScrollbarSettings` is its sibling, same shape, omitted for space)

```rust
#[derive(Clone, Debug, PartialEq, Serialize, Deserialize)]
#[serde(default)]
pub struct ScrollSettings {
    pub natural: NaturalScroll,
    pub momentum: Momentum,
    pub momentum_model: MomentumModel,
    pub momentum_a: Scalar,
    pub momentum_b: Scalar,
    pub momentum_stop_px_s: Scalar,
    pub momentum_start_px_s: Scalar,
    pub momentum_cap_px_s: Scalar,
    pub velocity_window_ms: Ms,
    pub velocity_held_still_ms: Ms,
    pub rubber_band: RubberBand,
    pub rubber_band_c: Fraction,
    pub rubber_band_linear_divisor: Scalar,
    pub rubber_band_edge_min_px: Px,
    pub rubber_band_snap_rate: Scalar,
    pub rubber_band_snap_gain: Scalar,
    pub wheel_detent_px: Px,
    pub wheel_burst_window_ms: Ms,
    pub momentum_boost_threshold_px_s: Scalar,
    pub double_scroll_grace_ms: Ms,
}

#[derive(Clone, Copy, Debug, PartialEq, Eq, Default, Serialize, Deserialize)]
#[serde(rename_all = "snake_case")]
pub enum NaturalScroll { #[default] Natural, Traditional }
#[derive(Clone, Copy, Debug, PartialEq, Eq, Default, Serialize, Deserialize)]
#[serde(rename_all = "snake_case")]
pub enum Momentum { #[default] On, Off }
#[derive(Clone, Copy, Debug, PartialEq, Eq, Default, Serialize, Deserialize)]
#[serde(rename_all = "snake_case")]
pub enum MomentumModel { #[default] MacMouseFix, Ios }
#[derive(Clone, Copy, Debug, PartialEq, Eq, Default, Serialize, Deserialize)]
#[serde(rename_all = "snake_case")]
pub enum RubberBand { #[default] Bounded, Linear, Off }

impl Default for ScrollSettings {
    fn default() -> Self {
        Self {
            natural: NaturalScroll::Natural,
            momentum: Momentum::On,
            momentum_model: MomentumModel::MacMouseFix,
            momentum_a: Scalar(30.0),
            momentum_b: Scalar(0.7),
            momentum_stop_px_s: Scalar(1.0),
            momentum_start_px_s: Scalar(100.0),
            momentum_cap_px_s: Scalar(12_000.0),
            velocity_window_ms: Ms(80),
            velocity_held_still_ms: Ms(50),
            rubber_band: RubberBand::Bounded,
            rubber_band_c: Fraction(550),
            rubber_band_linear_divisor: Scalar(20.0),
            rubber_band_edge_min_px: Px(10),
            rubber_band_snap_rate: Scalar(12.5),
            rubber_band_snap_gain: Scalar(0.31),
            wheel_detent_px: Px(60),
            wheel_burst_window_ms: Ms(300),
            momentum_boost_threshold_px_s: Scalar(350.0),
            double_scroll_grace_ms: Ms(150),
        }
    }
}
```

### 4.3 The rest, by name

`AppearanceSettings` (theme, accent, motion_level, transparency, typeface, look, file_icons, custom_styles; the `material_*` keys are gone, section 3.1),
`IconsSettings`, `BarSettings`, `LauncherSettings`, `GesturesSettings` (palmrest crate:
`scroll_speed`/`lock_*`/`swipe_*`/`tap_*`/`foreign_output`/`g4_*`/`live_workspace_*`/
`gesture_action_map`), `PalmRejectionSettings`, `MenusSettings` (carries `switcher_*` as the nested
struct the switcher's timing and cells are constants, not settings (2026-10-27: `SwitcherParams::default()`, show delay 150 ms; cell 80, icon 64, gap 8, overflow minimum icon 48, design/13 section 13.3.5)), `NotificationsSettings`,
`ControlCenterSettings`, `SpacesSettings`.

### 4.4 The root, the watch and the diff

```rust
#[derive(Clone, Debug, PartialEq, Serialize, Deserialize)]
#[serde(default)]
pub struct AppearanceFile {          // quire/appearance.toml
    pub version: u16,                // = 1
    pub appearance: AppearanceSettings,
    pub icons: IconsSettings,
}

#[derive(Clone, Debug, PartialEq, Serialize, Deserialize)]
#[serde(default)]
pub struct ShellFile {               // sill/settings.toml
    pub version: u16,
    pub bar: BarSettings,
    pub dock: DockSettings,
    pub launcher: LauncherSettings,
    pub scroll: ScrollSettings,
    pub scrollbar: ScrollbarSettings,
    pub menus: MenusSettings,
    pub notifications: NotificationsSettings,
    pub control_center: ControlCenterSettings,
    pub osd: OsdSettings,
    pub power_menu: PowerMenuSettings,
    pub display: DisplaySettings,
    pub spaces: SpacesSettings,
    pub keyboard: KeyboardSettings,
}

#[derive(Clone, Debug, PartialEq, Serialize, Deserialize)]
#[serde(default)]
pub struct GesturesFile {            // palmrest/gestures.toml
    pub version: u16,
    pub gestures: GesturesSettings,
    pub palm_rejection: PalmRejectionSettings,
}

/// Composes all three files; `ds-settings`, `sill-settings` and `palmrest` each own one
/// field's I/O, but a surface that needs more than one file's values (the dock reading
/// `spaces` for tint, `sill` reading `appearance.banner_material`) takes `&Settings`.
pub struct Settings {
    pub appearance: AppearanceFile,
    pub shell: ShellFile,
    pub gestures: GesturesFile,
}

/// One `notify` watch per file (three directory watches, 30 ms debounce each); yields the
/// whole owning file's parsed tree on every settled change, never a partial struct.
pub struct SettingsWatch { /* ... */ }
impl SettingsWatch {
    pub fn appearance(&self) -> impl Stream<Item = AppearanceFile>;
    pub fn shell(&self) -> impl Stream<Item = ShellFile>;
    pub fn gestures(&self) -> impl Stream<Item = GesturesFile>;
}
```

There is no change enum and no `apply`: each domain struct is `PartialEq`, and a surface or a
policy compares the one domain it draws between two reads (sill's `Section`). Whole-struct
equality, not per-field: a domain is small enough (largest is `dock` at 35 fields) that "one
field changed" and "recompute the domain" cost the same, and per-field diffing would need a
second hand-written table per domain that section 6's "every key appears in the Rust schema"
test would have to check twice.

## 5. Settings UI mapping

All proposed (this whole page taxonomy did not exist before 2026-09-24). "Advanced" = file
only in v1, no widget; a later wave may promote one if the user asks.

| Settings app page | Keys shown |
| --- | --- |
| **Appearance** | `appearance.theme`, `appearance.accent`, `appearance.look`, `appearance.file_icons`, `appearance.motion_level`, `appearance.transparency`, `appearance.custom_styles`, `icons.style`; under **Bar**: `bar.autohide` |
| **Dock** | `dock.magnification`, `dock.tile_size_px`, `dock.autohide`, `dock.indicators`, `dock.trash` |
| **Mouse & Gestures** | `scroll.natural`, `scroll.speed`, `gestures.gesture_action_map` (the remap table) |
| **Keyboard / Shortcuts** | proposed by `28-CUSTOMIZATION.md#411-keyboard-shortcuts`: `shortcuts.bindings` (the system list, toggle and chord per action) and `shortcuts.app_shortcuts`, rows added here after sill registers them (28 §6.0); sill writes them into the compositor's `system_actions` and `custom` shortcut files, and the chord the desktop already holds wins a clash, shown on the row (28 §7 decision 10) |
| **Notifications** | `notifications.dnd`, `notifications.banner_style` (per app) |
| **Spotlight** (sill M9, Q303) | `launcher.clipboard_history` (a privacy choice people should find), `launcher.web_search`, `launcher.emoji_skin_tone`; the rest of `launcher.*` stays Advanced |
| **Privacy & Security** (the compositor's per-app permission rows; `Page::Privacy`, wire `privacy`) | the rows casement publishes through its settings module (one per app and permission); no keys of quire's own | 
| **Power** (added Q445; sill's own idle service, section 3.24) | `idle.screen_off_ac_min`, `idle.screen_off_battery_min`, `idle.suspend_ac_min`, `idle.suspend_battery_min`, `idle.lock`, `session.lock_grace_s` (`Sill` under `idle.times` is what makes the `idle.screen_off_*`/`idle.suspend_*` sliders apply; under `Desktop` they still render, disabled, so the row explains itself) |
| **Intelligence** (the companion, design/32; sill's `companion` domain and the daemons' schemas) | Companion: `companion.summon`, `companion.in_field`, `companion.notify`; Models: `ai.local_only`, `ai.floor.<class>`, `ai.model.<kind>.<tier>` (the live picker, section 9.4); Memory: `memory.files.at_rest`, `memory.consolidation.when`, `memory.consolidation.apply`, and the memory view (a live module); Computer use: `cua.enabled`, `cua.apps`, `cua.default_mode`, `cua.autopause`; Privacy: `agent.strictness`, `agent.mcp.expose`, `agent.undo.keep_h` |
| **Viewer** (anyview, section 3.31) | Window: `viewer.chrome.hide_after`, `viewer.warm_for`; Viewing: `viewer.zoom.step`, `viewer.media.seek_step`; History: `viewer.history.cap`, `viewer.versions.keep`; Export: `viewer.export.jpeg_quality`, `viewer.export.avif_quality`, `viewer.export.paper`, `viewer.export.orientation`; `viewer.peek.*` and `viewer.image.*` are Advanced |
| **Advanced** (file only) | everything else in section 3: `bar.*` except `bar.autohide`, `menus.*`, `control_center.*`, `scrollbar.*`, `scroll.momentum_*`/`rubber_band_*`/`wheel_detent_px`, `dock.*` geometry beyond the Dock page's list above, `palm_rejection.*`, `gestures.g4_*`/`live_workspace_*`, `session.*` other than `lock_grace_s` (Accounts page, all Advanced) |

**Row help (2026-10-13).** Settings is for normal people: a row's label is a short noun and its details
come on request. Settings rows use `FieldRow { help_shown: HelpShown::OnRequest }` (04-COMPONENTS.md
section 56): the help sits behind the row's ⓘ. It is one or two sentences and never repeats the label.

## 6. Acceptance

1. **Round-trip test per domain**: `Domain::default()` -> `toml::to_string` -> `toml::from_str`
   -> `assert_eq!` back to `Domain::default()`, for every domain struct in section 4 (13 tests,
   one per domain; `CONVENTIONS.md#12-derives-and-serde` "every stored or wire type has a
   round-trip test").
2. **Lenient-parse table**: one test per type family feeding a bad value and asserting the
   field falls back to default, file otherwise intact —

   | Bad input | Field | Expected |
   | --- | --- | --- |
   | unknown enum string (`"maybe"`) | `dock.indicators` | falls back to `Indicators::On`, sibling fields keep their parsed values |
   | negative where `Px` (`u16`) expected | any `Px` field | parse error on that field only -> default |
   | missing table entirely (`[dock]` absent) | whole `DockSettings` | `#[serde(default)]` on the struct produces `DockSettings::default()` |
   | unknown key (`[dock].puppy = true`) | n/a | listed in `Loaded::unknown` as `dock.puppy`, and left in the file by the next save (a TOML file is saved by merging into the parsed text, so unknown keys and comments stay); every other key is read as usual |
3. **Watch-fires-on-rename**: a tempdir test writes `settings.toml`, starts `SettingsWatch`,
   then does the same atomic temp+rename the real writer does (not an in-place write), and
   asserts exactly one `ShellFile` is yielded within the 30 ms debounce + a slack margin — same
   pattern PLAN "Design: `<ds>`" already names for `ds-settings`: "round-trip, watch-fires-on-rename
   tempdir, portal mapping tables".
4. **Unknown-key report**: a file with a key no struct reads loads every other key as usual,
   lists the key in `Loaded::unknown`, and a save of the loaded value leaves it (and the file's comments) in place.
5. **Every key in this doc appears in the Rust schema**: a test that parses this file's section
   3 tables (the dotted key column) and asserts, for each one, that the corresponding
   `Domain::default()` struct has a field of that name reachable by splitting on `.` and
   stripping the domain prefix — catches a key added here and forgotten in code, or a Rust
   field with no doc entry, in either direction. (`gestures.gesture_action_map`,
   `control_center.modules` and the `appearance.*`-owned-but-notifications-read
   `notifications.banner_material` need an explicit allow-list in the test for the handful of
   keys whose dotted prefix does not match their owning Rust struct's module path 1:1 — noted
   inline in the test, not silently skipped.)

## 9. Registration: the schema is data (settled 2026-09-24)

A program does not register its settings with a Settings app at runtime. It ships a **schema
file**, generated from its own settings structs, and the Settings app renders pages from every
schema it finds. Types drive the widgets; the struct stays the single source of truth; nothing
links the Settings app.

### 9.1 The derive

Every domain struct in section 4 gains `#[derive(SettingsSchema)]` (a proc-macro in a new crate
`ds-settings-derive`, re-exported by `ds-settings`). Field attributes carry what the type cannot:

```rust
#[derive(Default, Serialize, Deserialize, SettingsSchema)]
#[settings(file = "sill/settings.toml", domain = "dock", page = Page::Dock)]
pub struct DockSettings {
    #[settings(label = "Magnification", help = "Icons grow as the pointer nears them.", section = "Magnification")]
    pub magnification: Magnification,                       // enum -> segmented control
    #[settings(label = "Magnified size", range = "48..=128", unit = "px", section = "Magnification", advanced)]
    pub magnified_px: Px,                                   // bounded newtype -> slider
    #[settings(label = "Hover label delay", range = "0..=1000", unit = "ms", advanced)]
    pub hover_label_delay_ms: Ms,
    ...
}
```

The derive emits, at build time, `KeySpec` values for every field:

```rust
pub struct KeySpec {
    pub path: KeyPath,            // "dock.magnified_px"
    pub kind: KeyKind,            // Enum{variants}, Bounded{min,max,unit}, Text, Colour, Shortcut, List(Box<KeyKind>), Rows{columns}
    pub default: toml::Value,
    pub label: Label, pub help: Help,
    pub page: Page, pub section: Section,
    pub exposure: Exposure,       // Basic | Advanced (file-only)
    pub labels: WordLabels,       // optional: a human label per enum word, by stored word
    pub agent: AgentSetting,      // HandsOff (default) | Settable; section 9.7
}
pub struct Schema { pub app: AppId, pub file: FilePath, pub keys: Vec<KeySpec> }
```

Widget by kind, fixed:

| Kind | Widget |
| --- | --- |
| one-variant enum | read-only label |
| two-variant enum whose words are an on/off pair (below) | `Toggle` |
| two-variant enum whose words are not (`ring`/`bar`, `bottom`/`left`, `system`/`editorial`) | `SegmentedControl` with two segments |
| three to five variants | `SegmentedControl` |
| more than five | `Menu` |
| `Bounded` | `Slider` with its unit |
| `Text` | `TextInput` |
| `Colour` | the appearance picker's swatch row |
| `Shortcut` | key-capture field |
| `List` | rows editor |
| `Rows` | rows editor with one cell per column |
| `Provided` | `Menu`, once the caller has filled its choices (below) |

`agent` is `AgentSetting::{HandsOff, Settable}`, set by the field attribute
`#[settings(agent_settable)]` and written into the schema's `[[key]]` table as
`agent = "settable"`; a hands-off key writes nothing, and a schema from before the mark existed
reads every key as hands-off (section 9.7).

`labels` (`[key.labels]`, word = "Label") is optional and absent from older schemas; a word
without a label is shown as its own words (`workspace_prev` as "Workspace prev").

Explanations sit where the Mac puts them (2026-10-27, detent's style pass): a short grey line under
the few rows that need one, a footnote under a section, and nothing on the rest.

- `#[settings(note = "...")]` sets `KeySpec.note`, the line drawn under the row. `help` stays as
  data (search, the assistant, a tooltip) and is not drawn inline.
- `#[settings(range = "16..=128", ends = ("Small", "Large"))]` sets `KeySpec.ends`
  (`SliderEnds { low, high }`, wire `ends = { low = "Small", high = "Large" }`): the words at the
  slider's two ends. Only with `range`.
- `#[settings(footnote(section = "Hiding", text = "..."))]` on the struct, any number of times,
  adds a `[[footnote]]` table (`Footnote { page, section, text }`, the page being the struct's):
  the paragraph under that section.

All three are absent from the file when unset, so older schemas read and write unchanged.

The schema's `kind` for a two-variant enum stays `toggle`; the Settings app decides between
switch and segments from the words, so a program writes nothing extra. Two words are an on/off
pair when either is `off`, `hide`, `never`, `none` or `nothing` (the other is the on side), or
when they are one of `natural`/`traditional` (natural on) or `reduced`/`standard` or `reduced`/`system` (reduced on: a
switch labelled like macOS's "Reduce motion" or "Reduce transparency", on for the reduced value). Everything else is a
choice between two things, not a switch. The rule and its table test live in detent's
`detent-model` (`control.rs`). All widgets are from `04-COMPONENTS.md`; the Settings app has no
widgets of its own.

A key that chooses among things somebody installed (a look, a set of file icons) is a field of
type `Provided` with `#[settings(provided = "<capability>")]`, kind `Provided { capability }`
(written `kind = "provided"` with `[key.kind.v] capability = "look"`). Its stored value is the bare
word `standard` or the id of what was chosen. The schema lists no choices: the program that draws
the key fills them in with `KeySpec::offering(words, labels)`, which turns the kind into a `Menu`
(`standard` first, labels by id), so the `menu` control draws it and no widget is added.
`ds-settings` knows nothing of what is installed. The reader of the key checks the stored id
against what it found (`ds_extensions::chosen`): an id nobody installed, or one that is turned off,
reads as `standard` and is reported once. A reader that predates the kind refuses the schema file
it is in, as it does for any unknown kind (`Schema::discover` skips the file with a warning).

A `Vec<T>` field takes its kind from `T`: `Vec<String>` is `List` of `Text`, `Vec<E>` for a
`Word` enum `E` is `List` of `E`'s kind, and `Vec<R>` for a struct `R` that derives
`SettingsRow` is `Rows`, a list of tables. The row struct is a plain serde struct; each field
carries `#[settings(label = "...")]`, and a `String` field may add `pattern = "<regex>"`:

```rust
#[derive(Clone, Debug, PartialEq, Eq, Serialize, Deserialize, SettingsRow)]
pub struct KeyboardOverride {
    #[settings(label = "App")]
    pub app: String,
    #[settings(label = "Profile", pattern = "[a-z0-9_-]{1,64}")]
    pub profile: String,
}

pub struct KeyboardSettings {
    #[settings(label = "Per-app profiles", section = "Profiles", advanced)]
    pub overrides: Vec<KeyboardOverride>,       // -> Rows { app: text, profile: text(pattern) }
}
```

A column is a `String` field (`text`, with its optional `pattern`) or a `Word` enum field
(`choice`, one word per variant); a row cannot hold rows. A column's `name` is its field's
name, so a row field carries no `#[serde(rename)]`. `pattern` is data for the Settings app to
validate an edit with: the whole value must match; the owning program validates the file's
rows again at load and decides what an invalid row costs (section 2 makes a bad *value* cost
its key; a program may choose the finer rule of dropping only the row, and its row in section 3
says so).

### 9.2 Where the schema lives

The derive's build step writes `target/.../<app-id>.settings.toml` and the package installs it
to `$XDG_DATA_DIRS/quire/settings/<app-id>.settings.toml`, next to the `.desktop` file, the way
GSettings ships schemas. The file is TOML, one `[[key]]` table per `KeySpec`, plus `app`,
`file` and `version`. Developers run `cargo run -p <app> -- --write-schema <dir>` for a local
install. quire's own schema (`quire.settings.toml`: the `appearance` and `icons` domains of
`quire/appearance.toml`) has no app binary, so it is written by the `ds-settings` example
`write_schema` (`cargo run -p ds-settings --example write_schema -- --write-schema <dir>`). A schema without a program (a stale file) is skipped with a warning.

A settings file can hold tables the program owns that are not settings (inferd's `[engines]`,
`[probe]`, `[callers]`). The schema names them in an optional top-level `foreign = ["engines",
"probe", "callers"]` (`Schema::foreign`, `ForeignTables`): the Settings app shows no rows for
them and an unknown-key check (detent's) leaves every key under them alone. Absent means none.

A `Rows` key is one `[[key]]` table whose `kind` carries one `[[key.kind.v.columns]]` table
per column (`kind` = `"text"` or `"choice"`; `pattern` is absent when there is none), and whose
`default` is the list of default rows:

```toml
[[key]]
path = "keyboard.overrides"
label = "Per-app profiles"
help = ""
section = "Profiles"
exposure = "advanced"

[key.kind]
kind = "rows"

[[key.kind.v.columns]]
name = "app"
label = "App"

[key.kind.v.columns.kind]
kind = "text"

[key.kind.v.columns.kind.v]

[[key.kind.v.columns]]
name = "profile"
label = "Profile"

[key.kind.v.columns.kind]
kind = "text"

[key.kind.v.columns.kind.v]
pattern = "[a-z0-9_-]{1,64}"

[[key.default]]
app = "org.gnome.Nautilus"
profile = "files"

[key.page]
kind = "keyboard_and_shortcuts"
```

The `Rows` kind is additive: a schema without one is byte-for-byte what it was, so `version`
stays 1 (section 2 bumps it for an *incompatible* change only). A Settings app that predates
`Rows` fails to parse the `kind` of such a key; one that reads keys leniently shows it read-only
as "edit in settings.toml".

### 9.3 What the Settings app does

1. Discover every `*.settings.toml` under `$XDG_DATA_DIRS/quire/settings/` (and
   `$XDG_DATA_HOME`), parse, group keys by `page` then `section`, render with the widget table
   above; pages come from a fixed `Page` enum (Appearance, Dock, Mouse and Gestures, Keyboard
   and Shortcuts, Notifications, Spaces, Accounts, Privacy, Power, Apps, Intelligence, Viewer, plus one page per third-party app
   id). Every "Advanced" key of a page renders under one "Advanced" disclosure at the end of the
   page, grouped by section inside it, so a section with only Advanced keys shows no empty header.
2. Read the current value from the key's `file` through the shared lenient loader; write with
   the atomic writer; the owning program's directory watch applies it live (section 2).
   The Settings app never talks to the program.
3. Deep links: the launcher's Settings provider reads the same schemas, so the query "dark
   mode" resolves to `appearance.theme` on the Appearance page, and "bluetooth" to the live
   module below; no hand-kept table.

### 9.4 Live modules (the one dynamic path)

Runtime state that is not a file (Wi-Fi networks, Bluetooth devices, audio devices, accounts)
is served by its shell service over D-Bus: `org.quire.SettingsModule1` with `Describe() ->
schema JSON` (the same `KeySpec` shape, with `kind = Live{action}`) plus `Get`/`Set`/`Changed`.
The Settings app renders these modules in the same pages, with the same widgets; the only
difference is that `Set` calls the service instead of writing a file. mailo, calendar and
contacts register their account pages this way (`20-SURFACES.md`).

The interface is frozen (quire `ds-settings`, feature `live`, 2026-10-06): `Describe() -> s`
(JSON `{version, key: [KeySpec]}`), `Get(s) -> v`, `Set(s, v)`, signal `Changed(s, v)`; values
are `b`, `x`, `s` or `as` only. Value keys of a live module use the ordinary kinds; only action
rows use `kind = Live{action}` (a label plus weight `plain` or `destructive`), are run by
`Set(key, true)` and draw as a button (`Widget::ActionButton`; a destructive one behind
`Alert{Critical}`). A toggle's value is a boolean on the wire. The serving daemon implements
`ds_settings::live::LiveModule` and holds all state; it decides who may `Get` or `Set` in
`permit` (accountd and inferd allow only the Settings role), and a refusal is
`org.quire.SettingsModule1.Error.NotPermitted`. `serve` and `LiveClient::describe` both refuse a
schema that marks an `AGENT_NEVER_SETTABLE` key agent-settable, the same check
`Schema::from_toml` and the derive use.

Who hears `Changed` (2026-10-16, production audit QUIRE-04; the signature is not touched): the signal is
never a broadcast, because a signal with no destination goes to every peer that adds a match rule and
`permit` cannot refuse a match rule. The skeleton remembers the peers `permit` let read (a call to
`Describe`, `Get` or `Snapshot`; `LiveClient::changes` makes one) and sends `Changed` to each of
them by unique name, and `Served::changed` the same. A peer `permit` refuses hears nothing, and
`LiveClient::changes` tells it `NotPermitted`. Every client call has a deadline
(`Timeouts`, 10 s for reads and 30 s for a `Set`) and a module that is silent is
`LiveError::Timeout`; a client takes `Changed` only from the module's own unique name.

Who owns the name (2026-10-16, production audit DETENT-05): a well-known name goes to whoever asks
first, so `LiveClient::new` takes an `OwnerCheck` that the consumer supplies (ds-settings knows no
caller table). Before the first call, and whenever the name's owner is another connection, the
client asks it whether that owner holds the module's role; a refusal is `LiveError::NotTheService`,
nothing is sent (no secret), and calls that pass go to the owner's unique name, so the name cannot
change hands between check and call. Replies are capped (`live::limits`): 1 MiB of schema, 1024 keys,
2 KiB per label, 8 KiB per value.

Unavailable choices (additive, 2026-10-06): a `KeySpec` may carry `unavailable`, a map from a
choice word to the reason it cannot be picked right now. JSON: `"unavailable":{"<word>":"<reason>"}`;
a file schema has the same as an `[key.unavailable]` table; both are skipped when empty, so
schemas written before the field parse unchanged and readers that predate it ignore it. The
Settings app greys such a choice, shows the reason under it ("Add an account to use") and does
not offer it. The skeleton refuses `Set(key, word)` for a listed word with the named error
`org.quire.SettingsModule1.Error.Unavailable` carrying the reason, before the module sees the
call; `KeySpec::check_available` is the same check for file validation. `Describe`, `Get`,
`Set` and `Changed` keep their signatures: the freeze above is not touched.

Snapshot (additive, 2026-10-12): `Snapshot() -> (s schema, a{sv} values)` answers `Describe` and
a `Get` per key in one call: `schema` is the `Describe` JSON, `values` maps each key to its value
encoded as `Get` encodes it (`b`, `x`, `s` or `as`, in a variant). Every key in `values` is in the
schema and the reverse, so a reader never sees a module half-changed between calls (an account
removed after `Describe` made `Get` answer `UnknownKey`). It asks `permit(Access::Read)` once and
a refusal is `NotPermitted`. `LiveModule::snapshot` has a default (describe, then each `get`,
dropping a key that answers `UnknownKey` from schema and values); a module that can read its state
once overrides it. `LiveClient::snapshot` falls back to that same client-side read when the peer
answers `UnknownMethod`, so new clients work with old modules.

Choice groups (additive, 2026-10-06): a menu `KeySpec` may carry `groups`, a map from a choice
word to the display name of its group ("Anthropic", "Google", "Moonshot", "OpenAI", "On this
computer"). JSON: `"groups":{"<word>":"<group>"}`; a file schema has an `[key.groups]` table;
both are skipped when empty, so older schemas parse unchanged. There is no order field: the
order of the groups is the order of each group's first appearance in `choices`, and the
choices keep their order inside a group. `KeySpec::grouped_choices()` yields the ungrouped
choices first (the Catalogue default `""`, `auto`), then each group with its choices; the
Settings app draws a section header per group, and a choice that is also `unavailable` stays
greyed inside its group. `Describe`, `Get`, `Set` and `Changed` keep their signatures: the
freeze above is not touched.

Choice places and note (additive, 2026-10-16): a menu `KeySpec` may carry `places`, a map from a
choice word to where its model runs, and `note`, a short line a picker shows inline under the
key (apart from `help`, which stays behind the info button). A place is an internally tagged
value: `{"kind":"this_computer","engine":"llama.cpp"}` (`engine` optional),
`{"kind":"your_computer","host":"frieren"}`, `{"kind":"cloud","company":"Anthropic"}` or
`{"kind":"assistant"}`; a file schema has `[key.places.<word>]` tables. A word without an entry
has no place. `places` is skipped when empty and `note` when absent, so older schemas parse
unchanged and a key without them serialises byte for byte as before. `KeySpec::place_of(word)`
reads one entry. `Describe`, `Get`, `Set` and `Changed` keep their signatures.

### 9.5 Rules

- A settings key exists only if a `KeySpec` describes it: a struct field without the derive is
  a bug, caught by the test in 9.6.
- Schema `version` follows the file `version` (section 2); a key removed from a struct leaves
  the schema with it.
- No program renders its own settings UI for keys the schema covers (mailo's appearance
  picker becomes the appearance picker component bound to `quire/appearance.toml`); an app
  may still embed the Settings app's rendering of its own schema in-app via the shared
  `SettingsPage` component (built: crate `ds-settings-page`, moved whole from mailo's
  `settings_window/keys.rs`). It is a crate of its own because it needs `ds` and `ds-settings`,
  which may not name each other. API:

  ```rust,ignore
  SettingsPage {
      schemas: vec![Schema],            // a titled FormSection per section, a FieldRow per key
      page: Option<Page>,               // only the keys of this page id; None draws all
      values: toml::Value,              // the file's content; a missing key reads its default
      onedit: |edit: SettingEdit| ..,   // { path: KeyPath, value }; edit.applied_to(values)
      extras: vec![SectionRows::new("Section title", rows).footer("note")],
      children,                         // whole sections the app draws itself, last
  }
  ```

  The page saves nothing; the app applies the edit and writes through its own `Store`. `extras`
  rows join the section of that title, or stand as a section of their own after the schema's, so
  a page of only app rows (a shortcut list) uses the same `Form` look with no schema.
- Every "proposed" value in this doc is Basic or Advanced per section 5; nothing is file-only
  without also being in the schema.

### 9.6 Acceptance

- `every_key_in_catalogue_has_a_spec`: the section 3 tables parsed from this file match the
  union of all schemas emitted by the workspace, path for path.
- `schema_round_trip`: derive -> TOML -> parse -> equals the derive output.
- `widget_for_kind_is_total`: every `KeyKind` maps to exactly one component. `Live` maps to `ActionButton`.
- `deep_link_resolves`: "dark mode", "magnification", "natural scrolling" each resolve to one
  key through the schemas alone.

### 9.7 Agent-settable keys (proposed)

An agent (the companion, through detent's intents) may set a key only if the owning program's
schema marks it. The mark is `AgentSetting::Settable`: the field attribute
`#[settings(agent_settable)]` on the settings struct, written into the schema file as
`agent = "settable"` in the key's `[[key]]` table and read back by the schema loader. An
unmarked key, and every key of a schema written before the mark existed, is
`AgentSetting::HandsOff`: an agent may read it, never set it. The derive is the only place a
mark is written; a person never edits it, and a settings file cannot carry it.

The list below is a **proposal for the user to accept or trim**, not a grant. It is conservative on
purpose: each key is cosmetic or a convenience that one click undoes, and none touches data,
accounts, spend or a lock. `ds_settings::schema::AGENT_SETTABLE_PROPOSED` holds the same list so
quire's, sill's and detent's tests check their schemas against one source.

| Key | Owner | Why an agent may set it | Status |
| --- | --- | --- | --- |
| `appearance.theme` | quire (`AppearanceSettings`) | "make it dark": a visible, instantly reversible preference | proposed (2026-10-03) |
| `appearance.accent` | quire (`AppearanceSettings`) | the same, one of eight colours | proposed (2026-10-03) |
| `appearance.look` | quire (`AppearanceSettings`) | "use the Paper look": the choice is among looks the person installed, and Standard undoes it in one click | proposed (2026-10-23) |
| `appearance.file_icons` | quire (`AppearanceSettings`) | "use the paper icons": the choice is among sets the person installed, and Standard undoes it in one click | proposed (2026-10-24) |
| `appearance.custom_styles` | quire (`AppearanceSettings`) | "turn my styles off": the way back from a style that broke a surface, one switch, undone in one click | proposed (2026-10-23) |
| `dock.magnification` | sill (`DockSettings`) | on/off, cosmetic | proposed (2026-10-03) |
| `dock.tile_size_px` | sill (`DockSettings`) | bounded slider, cosmetic | proposed (2026-10-03) |
| `dock.magnified_size_px` | sill (`DockSettings`) | bounded slider, cosmetic | proposed (2026-10-03) |
| `launcher.web_search` | sill (`LauncherSettings`) | on/off for a last "search the web" row; it sends nothing by itself | proposed (2026-10-03) |
| `notifications.dnd` | sill (`NotificationsSettings`) | "do not disturb for the meeting": the one setting a person asks for by voice most | proposed (2026-10-03) |

Never agent-settable, whatever a program writes: everything under `agent.*` (the companion's
own strictness and budgets), `ai.*` (models, spend, local-only), `cua.*` (computer use),
`memory.*`, `companion.*`, `session.*` and `idle.*` (the lock and its grace), `accounts.*`, and
`privacy.*`, `network.*` and `security.*` (owner decision 2026-10-16: what is shared, what is
reachable and what is checked are the person's to change). There is no allow-list at read time:
a program's own schema still decides which other keys it marks settable.
`AGENT_NEVER_SETTABLE` lists those prefixes and a test keeps the proposal clear of them. An
agent that could raise its own budget or switch computer use on would be its own gatekeeper.

The marks of sill's and detent's own keys live in their schemas; quire marks only what
`quire/appearance.toml` owns (`appearance.theme`, `appearance.accent`, `appearance.look`,
`appearance.file_icons`, `appearance.custom_styles`).

### 9.8 Pages, controls, targets, patches and ranges (v0.3.36)

Six additions to the schema contract, all additive: a schema or a file written before them reads
unchanged.

1. **Pages are data.** A schema may carry `[[page]]` tables (`id`, `title`, `symbol`, `order`,
   optional `aliases`). The built-in table is `crates/ds-settings/src/schema/pages.toml`: the
   twelve fixed pages (their old snake-case names are aliases of their kebab-case ids) and
   `wallpaper`, `displays`, `wifi`, `bluetooth` and `extensions` (title "Extensions", symbol
   `puzzle`, order 45, just after Appearance). `Page` stays a typed layer over `PageId`; a key
   whose page nobody declares is kept and reads as `Page::App(<id>)`, and
   `PageTable::status` says `Unknown`, so one new page never blanks a whole older schema. A key
   names its page with `page = Page::declared("wallpaper")`.
2. **One control vocabulary.** `control = "<id>"` on a key (`#[settings(control = "picture")]`)
   picks a control from the closed set `readout`, `toggle`, `segmented`, `menu`, `slider`, `text`,
   `colour`, `shortcut`, `rows`, `button`, `picture`, `corners`. `KeyKind::default_control` is the only
   table from kind to control, `TogglePair` the only on/off rule; an id this build does not know,
   or one that does not fit the kind, is ignored and the default control is drawn. `picture` is a
   grid of pictures with "Add Picture…" whose value is a path; it fits a text key. `corners` is the Hot Corners sheet: its value is the corner's list of bindings; it fits a text key.
3. **A target for every entrance.** `SettingsTarget` (defined in `ds_core::settings`, pure, so a compositor holds it without `ds-settings`; re-exported as `ds_settings::target::SettingsTarget`) is `Page(PageId)`,
   `Key(KeyPath)`, `Account(AccountRef)` or `Extension(ExtensionId)`, written `page:wallpaper`,
   `key:dock.tile_size_px`, `account:<id>`, `extension:<id>` (Settings > Extensions at that row). It is the payload of the `settings.page.show` intent. A page may say `listing = "hidden"`
   (`Listing::Hidden`): it resolves by id (`assistant-model-setup`, the model picker's first-run step)
   but `PageTable::sidebar()` leaves it out.
4. **One key at a time.** `SettingsWriter::patch(key, value)` sets one key in the file under its
   lock and writes atomically; comments, other keys and unknown keys keep their text. A program
   that owns one field of a shared file patches it; it never loads the whole document and saves.
5. **Ranges hold on load.** `SettingsDoc::limits()` gives a document the ranges of its schema
   (`Limits::of_schema`); a number outside its range is brought to the nearest end and reported in
   `Loaded::clamped`, whose line in `Loaded::diagnostics` is the one log line.
6. **A choice among installed things.** `KeyKind::Provided { capability }` (9.1): the stored value
   is `standard` or an id, the caller fills the choices (`KeySpec::offering`), and the `menu`
   control draws it. `appearance.look` is the first. The Extensions that supply the choices, and
   the `extensions.toml` file that turns them off (`extensions.off`, page `extensions`), are
   `ds-extensions`' (the customization design, sections 4 and 5); this crate has no knowledge of
   them, so it needs no dependency on that crate.

The key tables of sections 3.1, 3.3 and 3.32 are generated from the schema between
`<!-- keys:begin <domain> -->` and `<!-- keys:end <domain> -->` (`scripts/gen-doc-keys.sh`;
`scripts/check-doc-keys.sh` and `ds-settings`' `doc_tables` test fail when the doc differs). A
program's own sections move onto the same markers when its schema is passed with `--schema`.

## 7. Open decisions

1. **Whether palmrest and sill share one file.** This doc keeps them separate
   (`palmrest/gestures.toml` vs `sill/settings.toml`) because `palmrest` is a separate daemon
   that must start and apply its own config before `sill` or any compositor exists
   (`12-BEHAVIOUR-gestures.md#12-3-12-daemon-architecture`), and because `12.6`'s own doc text
   already assumes a `palmrest/config.toml`. A merged file would need `palmrest` to depend on
   `sill`'s schema or vice versa, which sill's crate boundaries (its ARCHITECTURE.md, "Crates":
   `sill-launcher`/`sill-ipc` must not reach several crates) argue against generalizing.
   Kept separate; revisit only if the two are shown to drift out of sync in practice.
2. **`appearance.accent`'s set is settled (2026-09-30)**: macOS's eight, Blue the default; the
   old Postmark/Amber/Violet words are not migrated.
3. **`control_center.modules`'s order is plan-derived, not compared against the Claude Doc spec
   (rev 31)** (`13-BEHAVIOUR-menus-windows.md#13-9-open-decisions` item 4) — the default above
   ships but is explicitly provisional.
4. **Whether `notifications.banner_material`'s read crosses from `appearance.toml` into `sill`'s
   process cleanly** — today `ds-settings::use_environment` is the only cross-file read path
   named in the plan (PLAN "Design: `<ds>`"); this doc assumes `sill` calls it for one field of
   `Appearance`, which is untested until `sill`'s `W1`/`M1` waves land.
5. **(Removed 2026-10-27.)** `icons.plate_glyph_colour_policy` is no longer a setting.
6. **Bar height** — settled 2026-09-28: 24, per `29-SIZING.md` §13 (the user picked the reference height; items 22). It stays a tunable number, not an enum.

## 8. Sources

- `README.md#3-citation-convention`, `README.md#4-canonical-source` (the 2026-09-24 decision
  this whole document exists to satisfy), `README.md#5-proposing-a-change`.
- `CHECKLIST.md#14-settings`.
- `CONVENTIONS.md#4-types` (illegal states unrepresentable, no bool in a settings key),
  `CONVENTIONS.md#12-derives-and-serde` (round-trip, never `deny_unknown_fields`),
  `CONVENTIONS.md#6-state-effects-and-dependencies` (pure functions; "every value the
  design docs mark proposed is read from a settings key... never hard-coded").
- PLAN `~/.claude/plans/vast-toasting-peach.md` "Design: `<ds>` design-system repo" (`ds-settings`
  I/O: `appearance.json` atomic write, `notify` watch, 30 ms debounce, settings portal), "Design:
  `<shell>` repo" (`sill-services`, `dock.json`, `frecency.json`, `Services` context pattern),
  "Cross-repo order", "Verification".
- Every `#N.M Configuration` section this doc draws its base tables from:
  `10-BEHAVIOUR-dock.md#10-6-configuration`, `11-BEHAVIOUR-scroll.md#11-6-configuration`,
  `12-BEHAVIOUR-gestures.md#12-6-configuration`, `13-BEHAVIOUR-menus-windows.md#13-6-configuration`.
- Every doc cited by section, individually, in section 3's Source column.
