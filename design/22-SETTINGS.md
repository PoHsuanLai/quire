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
magnification on/off, autohide, gesture-to-action mapping, theme/accent/look/motion-level — the
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
3. **Per-workspace instance data is not a global default.** `21-SPACES.md#10-storage-settled-path-proposed-schema`'s
   `$XDG_CONFIG_HOME/quire/spaces.json` stores each workspace's chosen `SpaceLook` (its dots,
   grain, theme, accent) — that is saved *state*, like `dock.json`'s pinned items, not a
   settings default. The *defaults* applied when a workspace has none (preset table, fallback
   grain, fallback accent) are keys in the `spaces` domain below.

## 2. Storage

One TOML file per owning program, replacing the plan's `appearance.json` (PLAN "Design:
`<ds>`") and the docs' scattered `~/.config/quire/scroll.json` / `~/.config/sill/settings.json`
/ `~/.config/palmrest/config.toml` mentions (`11-BEHAVIOUR-scroll.md#11-6-configuration`,
`10-BEHAVIOUR-dock.md#10-6-configuration`, `12-BEHAVIOUR-gestures.md#12-6-configuration`): this
doc is the authority on where each key actually lives, and supersedes those inline paths.

| File | Owner | Domains |
| --- | --- | --- |
| `$XDG_CONFIG_HOME/quire/appearance.toml` | `ds-settings` (crate `crates/ds-settings`, PLAN "Design: `<ds>`") | `appearance`, `motion` (level selector only), `icons` |
| `$XDG_CONFIG_HOME/sill/settings.toml` | `sill` (crate `sill-services`/`sill-surfaces`) | `bar`, `dock`, `launcher`, `scroll`, `scrollbar`, `menus`, `switcher`, `notifications`, `control_center`, `spaces`, `osd`, `power_menu`, `display`, `session`, `widgets`, `calendar`, `screenshot`, `hot_corners` |
| `$XDG_CONFIG_HOME/palmrest/gestures.toml` | `palmrest` (the gesture daemon, PLAN Appendix B); `sill`/`shell-host` read it read-only for `PointerOver` suppression and the scroll `feel` module | `gestures`, `palm_rejection` |

Rules, all three files:

- **Atomic write**: temp file + `rename` (same mechanism as `ds-settings`'s `appearance.json`
  writer, PLAN "Design: `<ds>`": "atomic write").
- **`version = 1`** top-level field. A future incompatible change bumps it and ships a
  migrator; today nothing reads it but its absence.
- **Unknown keys preserved**: a round-trip through a newer binary must not drop a key an older
  or newer version wrote (mirrors `CONVENTIONS.md#3-serde` "never `deny_unknown_fields` on a
  persisted type"; `#[serde(flatten)]` extra: `Table` catches the rest per struct).
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

**Mailo migration**: `appearance.rs`'s old `~/.config/mailo/appearance.json` (or wherever mailo
currently writes it) is read **once**, on first run of the new `ds-settings` loader if
`appearance.toml` does not yet exist, mapped field-for-field into `Appearance`, and written out
as `appearance.toml`; the json file is left in place (not deleted), so a downgrade is
non-destructive. This is the "adopt old json" step in PLAN "mailo consumption. Phase A".

## 3. The key catalogue

Types used below (see section 4 for the full list): `Px(u16)` logical pixels, `Ms(u16)`
milliseconds, `Percent(u8)` 0..100, `Fraction(u16)` per-mille (1000 = 1.0; used for ratios,
gains and constants like `c = 0.55` -> `550`), `Count(u16)` a plain quantity, `Scalar(f32)` a
dimensionless physics constant that does not fit the above (momentum model exponents), `Units`
a signed raw touchpad/report unit (device space, not px). Every enum is named; **no key is a
`bool`** (`CONVENTIONS.md#11-quire-addenda-2026-09-24`).

Status column values: **proposed** = doc marks the value proposed, this is the "make it a key"
case; **settled (preference)** = doc marks it settled but it is a user-facing choice, so it is
still a key per section 1; **settled default, range proposed** = the doc's own phrasing, kept
verbatim. **advanced** in the last column (used in section 5) means file-only, no Settings UI
control in v1.

### 3.1 `appearance` (quire/appearance.toml)

| Key | Type | Default | Range / Alt | Source | Status |
| --- | --- | --- | --- | --- | --- |
| `appearance.theme` | `Theme::{System,Light,Dark}` | `System` | | `07-LOOKS.md#2-the-look-model` | settled (preference) |
| `appearance.look` | `Look::{Post,Riso,Tide,Candy}` | `Post` | Riso/Tide/Candy reachable, not default | `07-LOOKS.md#11-desktop-default` | settled (preference) |
| `appearance.warmth` | `Warmth::{Cool,Neutral,Warm,Paper}` (Candy only) | `Neutral` | applies only when `look = Candy` | `07-LOOKS.md#7-warmth-candy-only` | settled (preference) |
| `appearance.accent` | `Accent` (6 variants) | `Postmark` | other 5 not named in any doc (03-COLOR open decision 6) — **could not find full default set**, see handback | `03-COLOR.md#open-decisions` item 6 | settled (preference), partial |
| `appearance.motion_level` | `MotionLevel::{System,Calm,Standard,Extra,Reduced}` | `System` | `System` follows the portal's `prefers-reduced-motion` | `07-LOOKS.md#11-desktop-default` ("Motion levels ... apply on top of whichever look is active | proposed") | proposed |
| `appearance.material_tint_alpha` | `Percent` | `80` | | `21-SPACES.md#3-where-the-tokens-apply` ("`--m-tint` = ... alpha .80 (proposed)") | proposed |
| `appearance.material_highlight_light` | `Percent` | `30` | `0..=100` | `FINDINGS.md` "macOS polish"; `04-COMPONENTS.md` | proposed (polish pass, 2026-09-25) |
| `appearance.material_highlight_dark` | `Percent` | `12` | `0..=100` | `FINDINGS.md` "macOS polish"; `04-COMPONENTS.md` | proposed (polish pass, 2026-09-25) |
| `appearance.material_hairline_light` | `Percent` | `14` | `0..=100` | `FINDINGS.md` "macOS polish"; `04-COMPONENTS.md` | proposed (polish pass, 2026-09-25) |
| `appearance.material_hairline_dark` | `Percent` | `60` | `0..=100` | `FINDINGS.md` "macOS polish"; `04-COMPONENTS.md` | proposed (polish pass, 2026-09-25) |
| `appearance.material_shadow_strength` | `Percent` | `100` | `0..=100` | `FINDINGS.md` "macOS polish"; `04-COMPONENTS.md` | proposed (polish pass, 2026-09-25) |
| `appearance.material_vibrancy` | `Percent` | `100` | `0..=100` | `FINDINGS.md` "macOS polish"; `04-COMPONENTS.md` | proposed (polish pass, 2026-09-25) |
| `notifications.banner_material` | `Material::{Toast,Inverse}` | `Toast` | `Inverse` = mail's ink-on-paper toast | `03-COLOR.md#open-decisions` item 10; `13-BEHAVIOUR-menus-windows.md#13-9-open-decisions` item 7 | proposed — **flagged for review** |
| `control_center.material` | `Material::{Sheet}` (fixed for v1) | `Sheet` | | `03-COLOR.md#17-3-material-per-surface` | proposed |
| `launcher.material` | `Material::{Sheet}` (fixed for v1) | `Sheet` | | `03-COLOR.md#17-3-material-per-surface` | proposed |

`notifications.banner_material` is stored under `appearance` (it names a `ds::Material`
variant, appearance's vocabulary) but is *read* by `sill`'s notifications surface; see section
7 for the cross-file read this implies.

### 3.2 `motion` (quire/appearance.toml — level selector only)

Per-`MotionLevel` durations, easings and scalars (`--t-*`, `--e-*`, `--overshoot`, `--squish`,
`--lift`, `--tilt`, `--stagger`) are `ds` token-table data (PLAN "Design: `<ds>`" token model),
generated by `ds::stylesheet()`, not settings — including the ones `05-MOTION.md#10-shell-motion`
and `#12-open-decisions` mark "proposed" for *which curve* a level uses (e.g. Reduced's
`--e-spring` = `--e-out`, `05-MOTION.md#10-shell-motion`). Rule 1 in section 1 applies: only the
level itself is user-tunable.

| Key | Type | Default | Range / Alt | Source | Status |
| --- | --- | --- | --- | --- | --- |
| `motion.level` | alias of `appearance.motion_level` (3.1); kept as a separate dotted path so `ds-settings` and Settings UI code can address it without the whole `Appearance` struct | `System` | | `07-LOOKS.md#11-desktop-default` | proposed |

### 3.3 `icons` (quire/appearance.toml)

The app-icon *generation* pipeline (`08-ICONS.md#3-generation-pipeline`) is a build-time tool
(`quire-icons`), not a runtime setting, and is out of scope. These are the runtime,
per-third-party-icon rendering values `sill`'s dock/launcher apply live (`08-ICONS.md#4-third-party-app-icons`).

| Key | Type | Default | Range / Alt | Source | Status |
| --- | --- | --- | --- | --- | --- |
| `icons.style` | `IconStyle::{Colour,Muted,Monochrome}` | `Colour` (each app in the dialect its icon was designed in) | `Muted` lowers the chroma cap; `Monochrome` draws every icon in one hue, tone on tone, and desaturates and re-tints third-party icons inside our plates to the same hue | `08-ICONS.md#210-dialects-proposed-round-four-2026-09-25` | proposed (round four; implementation beyond the key waits for the user's dialect pick) |
| `icons.monochrome_tint` | `MonochromeTint::{Space,Accent,Neutral}` | `Space` (the accent `ds::space::derive` gives the workspace's Space, so the icons follow the frame) | `Accent` = the card accent; `Neutral` = no hue; read only when `icons.style` is `Monochrome` | `08-ICONS.md#210-dialects-proposed-round-four-2026-09-25`; `03-COLOR.md#5-card-accent` | proposed |
| `icons.plate_inset_percent` | `Percent` | `72` | | `08-ICONS.md#41-plate-mask-rule-settled-rule-proposed-numbers` | proposed |
| `icons.symbolic_fallback_glyph_percent` | `Percent` | `56` | | `08-ICONS.md#24-object-placement-proposed`; `08-ICONS.md#43-symbolic-fallback-proposed` | proposed |
| `icons.squircle_detect_iou` | `Fraction` | `900` (0.90) | | `08-ICONS.md#42-icons-that-are-already-squircles-or-rounded-squares-proposed` | proposed |
| `icons.plate_glyph_colour_policy` | `PlateGlyphPolicy::{Auto,ForceWhite,ForceInk}` | `Auto` (WCAG-driven per family: red/blue/violet -> white, amber/green -> ink) | | `08-ICONS.md#23-plate-gradient-settled-source-proposed-mapping` | proposed — **flagged for review** |
| `icons.dark_mode_variant` | `IconDarkVariant::{SameAsLight,Adaptive}` | `SameAsLight` | matches freedesktop convention | `08-ICONS.md#23-plate-gradient-settled-source-proposed-mapping` (Open decision 2) | proposed |
| `icons.symbolic_chroma_max` | `Fraction` | `40` (0.04) | `0..200` (0.0..0.2) | `08-ICONS.md#15-colour` (step 2); `ds::icon::ChromaLimit` | proposed |

`icons.symbolic_chroma_max` names the threshold `ds::icon::classify_with` already takes as a
`ChromaLimit` (bar gaps, `crates/ds/src/icon/classify.rs`): a tray or app icon whose opaque
pixels all sit below this OKLCH chroma is drawn as a symbolic mask in the ink colour, rather
than shown in its own colour. Advanced (file only, section 5); shown on the **Appearance** or
**Dock** page if a later wave promotes it — both read icons live. `sill` registers the key in
its own settings crate (`ds-settings`'s `IconsSettings`, `crates/ds-settings/src/settings.rs`);
that registration is not done yet (see FINDINGS "Tune wave").

### 3.4 `bar` (sill/settings.toml)

| Key | Type | Default | Range / Alt | Source | Status |
| --- | --- | --- | --- | --- | --- |
| `bar.height_px` | `Px` | `32` | alt 24-28 (macOS 24pt) | `13-BEHAVIOUR-menus-windows.md#13-3-1-bar-geometry`; `01-LAYOUT.md#13-shell-surface-layout`; Open decisions in both | proposed — **flagged for review** |
| `bar.title_hit_height_px` | `Px` | `24` | | `13-BEHAVIOUR-menus-windows.md#13-3-1-bar-geometry` | proposed |
| `bar.title_padding_px` | `Px` | `10` | | `13-BEHAVIOUR-menus-windows.md#13-3-1-bar-geometry` | proposed |
| `bar.open_title_pill_height_px` | `Px` | `24` | | `13-BEHAVIOUR-menus-windows.md#13-3-1-bar-geometry` | proposed |
| `bar.status_icon_box_px` | `Px` | `22` | | `13-BEHAVIOUR-menus-windows.md#13-3-1-bar-geometry`; `20-SURFACES.md#1-1-bar-spec-tier-1` (`IconSize::Bar`) | proposed |
| `bar.status_glyph_px` | `Px` | `16` | | `13-BEHAVIOUR-menus-windows.md#13-3-1-bar-geometry` | proposed |
| `bar.status_gap_px` | `Px` | `4` | | `13-BEHAVIOUR-menus-windows.md#13-3-1-bar-geometry` | proposed |
| `bar.glyph_size_policy` | `BarGlyphSize::{StatusIcon16,IconSizeBar22}` | `StatusIcon16` | alt = use the full `IconSize::Bar` box | `13-BEHAVIOUR-menus-windows.md#13-9-open-decisions` item 8 | proposed |
| `bar.item_font_px` | `Px` | `13` | `9..=24` | `FINDINGS.md` "macOS polish"; `04-COMPONENTS.md` | proposed (polish pass, 2026-09-25) |
| `bar.item_font_weight` | `Count` | `500` | `100..=900`; the app name is always bold | `FINDINGS.md` "macOS polish"; `04-COMPONENTS.md` | proposed (polish pass, 2026-09-25) |
| `bar.item_radius_px` | `Px` | `4` | `0..=12` | `FINDINGS.md` "macOS polish"; `04-COMPONENTS.md` | proposed (polish pass, 2026-09-25) |

### 3.5 `dock` (sill/settings.toml)

Pinned items stay in `~/.config/sill/dock.json` (state, not this file;
`10-BEHAVIOUR-dock.md#10-6-configuration`).

| Key | Type | Default | Range / Alt | Source | Status |
| --- | --- | --- | --- | --- | --- |
| `dock.magnification` | `Magnification::{On,Off}` | `On` | | `10-BEHAVIOUR-dock.md#10-6-configuration` | settled (preference) |
| `dock.tile_size_px` | `Px` | `48` | `32..80` | `10-BEHAVIOUR-dock.md#10-6-configuration` | settled default, range proposed |
| `dock.magnified_size_px` | `Px` | `96` | `tile_size..128` | `10-BEHAVIOUR-dock.md#10-6-configuration` | settled default, range proposed |
| `dock.influence_radius_px` | `Px` | `96` | alt `144` | `10-BEHAVIOUR-dock.md#10-3-3-magnification`; `10-BEHAVIOUR-dock.md#10-9-open-decisions` item 1 | proposed — **flagged for review** |
| `dock.tile_gap_px` | `Px` | `8` | was 4 before the polish pass | `10-BEHAVIOUR-dock.md#10-3-1-geometry-at-rest-bottom-dock-logical-px` | proposed |
| `dock.progress_style` | `ProgressStyle::{Ring,Bar}` | `Ring` | alt `Bar` (macOS draws a bar under the icon) | `10-BEHAVIOUR-dock.md#10-3-2-running-indicator-badge-progress-separator`; `10-BEHAVIOUR-dock.md#10-9-open-decisions` item 2 | proposed |
| `dock.autohide` | `AutoHide::{Off,On}` | `Off` | | `10-BEHAVIOUR-dock.md#10-6-configuration` | settled (preference) |
| `dock.autohide_delay_ms` | `Ms` | `200` | `0..1000` | `10-BEHAVIOUR-dock.md#10-6-configuration` | settled default |
| `dock.autohide_slide_ms` | `Ms` | `500` | `0..1500` | `10-BEHAVIOUR-dock.md#10-6-configuration` | settled default |
| `dock.autohide_trigger_strip_px` | `Px` | `4` | | `10-BEHAVIOUR-dock.md#10-3-11-auto-hide` | proposed |
| `dock.position` | `DockPosition::{Bottom,Left,Right}` | `Bottom` | Left/Right deferred | `10-BEHAVIOUR-dock.md#10-6-configuration` | settled; deferred |
| `dock.indicators` | `Indicators::{On,Off}` | `On` | | `10-BEHAVIOUR-dock.md#10-6-configuration` | proposed |
| `dock.bounce` | `Bounce::{On,Off}` | `On` | | `10-BEHAVIOUR-dock.md#10-6-configuration` | proposed |
| `dock.launch_animation` | `LaunchAnim::{On,Off}` | `On` | | `10-BEHAVIOUR-dock.md#10-6-configuration` | proposed |
| `dock.click_active_app` | `ActiveClick::{Cycle,Nothing}` | `Cycle` | `Nothing` = macOS | `10-BEHAVIOUR-dock.md#10-6-configuration` | settled (plan) |
| `dock.trash` | `TrashTile::{On,Off}` | `On` | | `10-BEHAVIOUR-dock.md#10-6-configuration` | proposed |
| `dock.pill_radius_px` | `Px` | `22` | | `10-BEHAVIOUR-dock.md#10-3-1-geometry-at-rest-bottom-dock-logical-px` | proposed |
| `dock.edge_clamp_px` | `Px` | `8` | | `10-BEHAVIOUR-dock.md#10-3-1-geometry-at-rest-bottom-dock-logical-px` | proposed |
| `dock.overflow_min_tile_px` | `Px` | `24` | | `10-BEHAVIOUR-dock.md#10-3-1-geometry-at-rest-bottom-dock-logical-px` | proposed |
| `dock.running_dot_diameter_px` | `Px` | `4` | | `10-BEHAVIOUR-dock.md#10-3-2-running-indicator-badge-progress-separator` | proposed |
| `dock.badge_size_px` | `Px` | `18` | | `10-BEHAVIOUR-dock.md#10-3-2-running-indicator-badge-progress-separator` | proposed |
| `dock.badge_radius_px` | `Px` | `9` | | `10-BEHAVIOUR-dock.md#10-3-2-running-indicator-badge-progress-separator` | proposed |
| `dock.progress_ring_diameter_px` | `Px` | `20` | | `10-BEHAVIOUR-dock.md#10-3-2-running-indicator-badge-progress-separator` | proposed |
| `dock.progress_ring_stroke_px` | `Px` | `3` | | `10-BEHAVIOUR-dock.md#10-3-2-running-indicator-badge-progress-separator` | proposed |
| `dock.separator_height_px` | `Px` | `36` | | `10-BEHAVIOUR-dock.md#10-3-2-running-indicator-badge-progress-separator` | proposed |
| `dock.magnify_enter_ms` | `Ms` | `120` | | `10-BEHAVIOUR-dock.md#10-3-3-magnification` | proposed |
| `dock.magnify_leave_ms` | `Ms` | `200` | | `10-BEHAVIOUR-dock.md#10-3-3-magnification` | settled (~200ms), easing proposed |
| `dock.hover_label_offset_px` | `Px` | `7` | | `10-BEHAVIOUR-dock.md#10-3-4-hover-label` | proposed |
| `dock.hover_label_warm_ms` | `Ms` | `400` | | `10-BEHAVIOUR-dock.md#10-3-4-hover-label` | proposed |
| `dock.bounce_period_ms` | `Ms` | `500` | | `10-BEHAVIOUR-dock.md#10-3-5-bounce` | proposed |
| `dock.bounce_launch_cap_ms` | `Ms` | `10000` | | `10-BEHAVIOUR-dock.md#10-3-5-bounce` | proposed |
| `dock.hold_to_menu_ms` | `Ms` | `600` | | `10-BEHAVIOUR-dock.md#10-3-6-clicks` | proposed |
| `dock.hold_to_menu_move_px` | `Px` | `8` | | `10-BEHAVIOUR-dock.md#10-3-6-clicks` | proposed |
| `dock.remove_threshold_px` | `Px` | `64` | | `10-BEHAVIOUR-dock.md#10-3-8-drag-inside-drag-out-drops`; `06-INTERACTIONS.md#20-desktop-interactions-settled` | proposed |
| `dock.spring_load_ms` | `Ms` | `500` | | `06-INTERACTIONS.md#20-desktop-interactions-settled` (§20.1) | proposed |
| `dock.minimize_debounce_ms` | `Ms` | `100` | | `10-BEHAVIOUR-dock.md#10-3-10-minimize-target` | proposed |
| `dock.context_menu_order` | `DockMenuOrder::{WindowsFirst,AppleOrder}` | `WindowsFirst` | plan order: windows, desktop actions, Keep in Dock, Quit | `10-BEHAVIOUR-dock.md#10-3-7-context-menu`; `06-INTERACTIONS.md#20-desktop-interactions-settled` | proposed |
| `dock.modifier_clicks` | `DockModifierClicks::{AppleMapping,Off}` | `AppleMapping` | Ctrl-click Show in Files, Alt-click switch+hide, Ctrl+Alt-click hide others | `06-INTERACTIONS.md#20-desktop-interactions-settled` (§20.1) | proposed |
| `dock.pill_padding_px` | `Px` | `6` | `0..=24` | `FINDINGS.md` "macOS polish"; `04-COMPONENTS.md` | proposed (polish pass, 2026-09-25) |
| `dock.running_dot_gap_px` | `Px` | `3` | `0..=12` | `FINDINGS.md` "macOS polish"; `04-COMPONENTS.md` | proposed (polish pass, 2026-09-25) |
| `dock.floor` | `DockFloor::{Off,On}` | `Off` |  | `FINDINGS.md` "macOS polish"; `04-COMPONENTS.md` | proposed (polish pass, 2026-09-25) |
| `dock.shipped_icons` | `ShippedIcons::{On,Off}` | `On` | Advanced. Show quire's own app icons (`assets/icons/apps`, design/08 §2.11) for the apps in the shell's mapping table, in the dock and the launcher rows, instead of the app's hicolor icon | `08-ICONS.md#2-11-the-shipped-set`; sill FINDINGS "Icon style" | proposed (2026-09-25) |

### 3.6 `launcher` (sill/settings.toml)

| Key | Type | Default | Range / Alt | Source | Status |
| --- | --- | --- | --- | --- | --- |
| `launcher.top_bar_gap_px` | `Px` | `24` | from `max(bar_h + 24, ...)` | `13-BEHAVIOUR-menus-windows.md#13-3-9-launcher-appearance-spotlight-like` | proposed |
| `launcher.top_centre_bias_px` | `Px` | `230` | from `round(0.4 x output_h - 230)` | `13-BEHAVIOUR-menus-windows.md#13-3-9-launcher-appearance-spotlight-like` | proposed |
| `launcher.top_vertical_fraction` | `Fraction` | `400` (0.4) | | `13-BEHAVIOUR-menus-windows.md#13-3-9-launcher-appearance-spotlight-like` | proposed |
| `launcher.input_row_height_px` | `Px` | `56` | | `13-BEHAVIOUR-menus-windows.md#13-3-9-launcher-appearance-spotlight-like` | settled font, height proposed |
| `launcher.result_row_height_px` | `Px` | `44` | | `13-BEHAVIOUR-menus-windows.md#13-3-9-launcher-appearance-spotlight-like` | proposed |
| `launcher.open_latency_budget_ms` | `Ms` | `100` (p95) | | `13-BEHAVIOUR-menus-windows.md#13-3-9-launcher-appearance-spotlight-like`; `05-MOTION.md#10-shell-motion` | settled (plan), not a preference but kept visible for `dev/accept-launcher.sh` tuning |
| `launcher.field_font_px` | `Px` | `22` | `12..=40` | `FINDINGS.md` "macOS polish"; `04-COMPONENTS.md` | proposed (polish pass, 2026-09-25) |
| `launcher.field_font_weight` | `Count` | `500` | `100..=900` | `FINDINGS.md` "macOS polish"; `04-COMPONENTS.md` | proposed (polish pass, 2026-09-25) |
| `launcher.field_glyph_px` | `Px` | `20` | `12..=40` | `FINDINGS.md` "macOS polish"; `04-COMPONENTS.md` | proposed (polish pass, 2026-09-25) |
| `launcher.row_title_px` | `Px` | `14` | `9..=24` | `FINDINGS.md` "macOS polish"; `04-COMPONENTS.md` | proposed (polish pass, 2026-09-25) |
| `launcher.row_detail_px` | `Px` | `12` | `9..=20` | `FINDINGS.md` "macOS polish"; `04-COMPONENTS.md` | proposed (polish pass, 2026-09-25) |

### 3.7 `scroll` (sill/settings.toml)

`scroll.speed`, `scroll.lock_*` and the gain constants are **not** duplicated here: they are
palmrest's `feel` module and live under `gestures` (3.10) in `palmrest/gestures.toml`, which
`shell-host` reads read-only (`11-BEHAVIOUR-scroll.md#11-7-integration`).

| Key | Type | Default | Range / Alt | Source | Status |
| --- | --- | --- | --- | --- | --- |
| `scroll.natural` | `NaturalScroll::{Natural,Traditional}` | `Natural` | | `11-BEHAVIOUR-scroll.md#11-6-configuration` (R16) | settled (preference) |
| `scroll.momentum` | `Momentum::{On,Off}` | `On` | | `11-BEHAVIOUR-scroll.md#11-6-configuration` | proposed |
| `scroll.momentum_model` | `MomentumModel::{MacMouseFix,Ios}` | `MacMouseFix` | alt `Ios` (`r=.998`, ~1000px glide, "feels closer" per the doc's own note) — the fully worked formula and every acceptance-test number in `11.5.1`/`11.8` assume `MacMouseFix`, so it is the sensible default | `11-BEHAVIOUR-scroll.md#11-9-open-decisions` item 1 | proposed — **flagged for review** |
| `scroll.momentum_a` | `Scalar` | `30.0` | Mac Mouse Fix drag constant | `11-BEHAVIOUR-scroll.md#11-5-1-momentum-mac-mouse-fix-drag` | proposed (gallery-only) |
| `scroll.momentum_b` | `Scalar` | `0.7` | Mac Mouse Fix drag exponent | `11-BEHAVIOUR-scroll.md#11-5-1-momentum-mac-mouse-fix-drag` | proposed (gallery-only) |
| `scroll.momentum_stop_px_s` | `Scalar` | `1.0` | | `11-BEHAVIOUR-scroll.md#11-6-configuration` (gallery-only list) | proposed (gallery-only) |
| `scroll.momentum_start_px_s` | `Scalar` | `100.0` | flick must exceed this to enter Momentum | `11-BEHAVIOUR-scroll.md#11-4-state-machine` | proposed (gallery-only) |
| `scroll.momentum_cap_px_s` | `Scalar` | `12000.0` | | `11-BEHAVIOUR-scroll.md#11-5-2-release-velocity` | proposed (gallery-only) |
| `scroll.velocity_window_ms` | `Ms` | `80` | | `11-BEHAVIOUR-scroll.md#11-5-2-release-velocity` | proposed (gallery-only) |
| `scroll.velocity_held_still_ms` | `Ms` | `50` | | `11-BEHAVIOUR-scroll.md#11-5-2-release-velocity` | proposed (gallery-only) |
| `scroll.rubber_band` | `RubberBand::{Bounded,Linear,Off}` | `Bounded` | Linear proposed as the configurable alternative | `11-BEHAVIOUR-scroll.md#11-6-configuration`; `11-BEHAVIOUR-scroll.md#11-9-open-decisions` item 2 | proposed — **flagged for review** |
| `scroll.rubber_band_c` | `Fraction` | `550` (0.55) | iOS bounded-formula constant | `11-BEHAVIOUR-scroll.md#11-3-7-rubber-band`; `11-BEHAVIOUR-scroll.md#11-5-3-rubber-band` | proposed — **flagged for review** |
| `scroll.rubber_band_linear_divisor` | `Scalar` | `20.0` | `Linear` model: `s(O) = O / 20` | `11-BEHAVIOUR-scroll.md#11-5-3-rubber-band` | proposed (config alternative) |
| `scroll.rubber_band_edge_min_px` | `Px` | `10` | | `11-BEHAVIOUR-scroll.md#11-5-3-rubber-band` | proposed |
| `scroll.rubber_band_snap_rate` | `Scalar` | `12.5` | | `11-BEHAVIOUR-scroll.md#11-5-3-rubber-band` | proposed (gallery-only) |
| `scroll.rubber_band_snap_gain` | `Scalar` | `0.31` | | `11-BEHAVIOUR-scroll.md#11-5-3-rubber-band` | proposed (gallery-only) |
| `scroll.wheel_detent_px` | `Px` | `60` | `20..200`; alt Blitz/Chromium Linux ~53 | `11-BEHAVIOUR-scroll.md#11-6-configuration`; `11-BEHAVIOUR-scroll.md#11-9-open-decisions` item 5 | proposed — **flagged for review** (task shorthand "notch_px") |
| `scroll.wheel_burst_window_ms` | `Ms` | `300` | events this close keep the previous detent-burst target | `11-BEHAVIOUR-scroll.md#11-3-11-wheel-mice-and-programmatic-scrolls` | proposed |
| `scroll.momentum_boost_threshold_px_s` | `Scalar` | `350.0` | a same-direction flick during Momentum above this speed boosts `v0` | `11-BEHAVIOUR-scroll.md#11-4-state-machine` | proposed |
| `scroll.double_scroll_grace_ms` | `Ms` | `150` | | `11-BEHAVIOUR-scroll.md#11-3-14-double-scroll-suppression` | proposed |

### 3.8 `scrollbar` (sill/settings.toml)

| Key | Type | Default | Range / Alt | Source | Status |
| --- | --- | --- | --- | --- | --- |
| `scroll.scrollbars` | `ScrollbarVisibility::{WhenScrolling,Always}` | `WhenScrolling` | | `11-BEHAVIOUR-scroll.md#11-6-configuration` | proposed |
| `scrollbar.track_thickness_px` | `Px` | `12` | hover `16` | `11-BEHAVIOUR-scroll.md#11-3-12-overlay-scrollbar-our-own-thumb` | proposed |
| `scrollbar.track_thickness_hover_px` | `Px` | `16` | | `11-BEHAVIOUR-scroll.md#11-3-12-overlay-scrollbar-our-own-thumb` | proposed |
| `scrollbar.thumb_thickness_px` | `Px` | `8` | hover `12` | `11-BEHAVIOUR-scroll.md#11-3-12-overlay-scrollbar-our-own-thumb` | proposed |
| `scrollbar.thumb_thickness_hover_px` | `Px` | `12` | | `11-BEHAVIOUR-scroll.md#11-3-12-overlay-scrollbar-our-own-thumb` | proposed |
| `scrollbar.thumb_min_length_px` | `Px` | `18` | | `11-BEHAVIOUR-scroll.md#11-3-12-overlay-scrollbar-our-own-thumb` | proposed |
| `scrollbar.hide_delay_ms` | `Ms` | `500` | Blitz uses 200 | `11-BEHAVIOUR-scroll.md#11-3-12-overlay-scrollbar-our-own-thumb` | proposed |
| `scrollbar.fade_duration_ms` | `Ms` | `240` | | `11-BEHAVIOUR-scroll.md#11-3-12-overlay-scrollbar-our-own-thumb` | proposed |
| `scrollbar.expand_zone_px` | `Px` | `16` | pointer proximity to the track edge | `11-BEHAVIOUR-scroll.md#11-3-12-overlay-scrollbar-our-own-thumb` | proposed |
| `scrollbar.expand_duration_ms` | `Ms` | `240` | | `11-BEHAVIOUR-scroll.md#11-3-12-overlay-scrollbar-our-own-thumb` | proposed |
| `scrollbar.legacy_track_width_px` | `Px` | `15` | used when `scroll.scrollbars = Always` | `11-BEHAVIOUR-scroll.md#11-3-12-overlay-scrollbar-our-own-thumb` | proposed |
| `scrollbar.other_axis_offset_px` | `Px` | `12` | vertical track stops short when horizontal is visible | `11-BEHAVIOUR-scroll.md#11-3-12-overlay-scrollbar-our-own-thumb` | proposed |
| `scrollbar.track_click_mode` | `TrackClick::{PageTowardClick,JumpToClick}` | `PageTowardClick` | macOS default is "jump to next page" | `11-BEHAVIOUR-scroll.md#11-3-12-overlay-scrollbar-our-own-thumb` | proposed |

### 3.9 `gestures` (palmrest/gestures.toml)

Base table carried verbatim from `12-BEHAVIOUR-gestures.md#12-6-configuration`, plus values
found elsewhere in the file that are not yet in that table.

| Key | Type | Default | Range / Alt | Source | Status |
| --- | --- | --- | --- | --- | --- |
| `scroll.speed` | `Count` | `22` | `0..63` | `12-BEHAVIOUR-gestures.md#12-6-configuration` | settled (Python) |
| `scroll.lock_threshold` | `Units` | `40` | `10..200` | `12-BEHAVIOUR-gestures.md#12-6-configuration` | settled |
| `scroll.lock_ratio` | `Fraction` | `1500` (1.5) | `1000..4000` | `12-BEHAVIOUR-gestures.md#12-6-configuration` | settled |
| `scroll.lock_timeout_ms` | `Ms` | `250` | `0..2000` | `12-BEHAVIOUR-gestures.md#12-6-configuration` | settled |
| `scroll.velocity_gain_max` | `Fraction` | `2000` (2.0) | `1000..4000` | `12-BEHAVIOUR-gestures.md#12-6-configuration` | settled |
| `scroll.repeat_gain_max` | `Fraction` | `2300` (2.3) | `1000..4000` | `12-BEHAVIOUR-gestures.md#12-6-configuration` | settled |
| `scroll.detent_px` | `Px` | `60` | `20..200` | `12-BEHAVIOUR-gestures.md#12-6-configuration` | proposed (mirrors `scroll.wheel_detent_px`, see 7) |
| `swipe.threshold` | `Units` | `200` | `50..1000` | `12-BEHAVIOUR-gestures.md#12-6-configuration` | settled |
| `swipe.vertical_max` | `Units` | `150` | `20..1000` | `12-BEHAVIOUR-gestures.md#12-6-configuration` | settled |
| `swipe.cooldown_ms` | `Ms` | `500` | `0..2000` | `12-BEHAVIOUR-gestures.md#12-6-configuration` | settled |
| `swipe.workspace_mode` | `WorkspaceMode::{Trigger,Live}` | `Trigger` | | `12-BEHAVIOUR-gestures.md#12-6-configuration` | settled |
| `swipe.repeat` | `SwipeRepeat::{AfterCooldown,OncePerGesture}` | `AfterCooldown` | | `12-BEHAVIOUR-gestures.md#12-6-configuration`; `12-BEHAVIOUR-gestures.md#12-9-open-decisions` item 1 | settled (Python) |
| `tap.max_ms` | `Ms` | `200` | | `12-BEHAVIOUR-gestures.md#12-6-configuration` | proposed |
| `tap.gap_ms` | `Ms` | `350` | | `12-BEHAVIOUR-gestures.md#12-6-configuration` | proposed |
| `tap.max_move_units` | `Units` | `40` | | `12-BEHAVIOUR-gestures.md#12-6-configuration` | proposed |
| `tap.centroid_max_units` | `Units` | `300` | second tap must land within this of the first | `12-BEHAVIOUR-gestures.md#12-3-4-the-gesture-set` (G6) | proposed |
| `foreign_output` | `ForeignOutput::{Touchpad,Wheel,Off}` | `Touchpad` | | `12-BEHAVIOUR-gestures.md#12-6-configuration` | proposed |
| `gestures.g4_foreign_mode` | `G4Mode::{InjectAndScroll,Native}` | `InjectAndScroll` | `Native` = scroll-only, let the browser run its own overscroll nav | `12-BEHAVIOUR-gestures.md#12-9-open-decisions` item 2 | proposed — **flagged for review** (task shorthand "one-finger swipe Inject vs Native") |
| `gestures.g4_travel_units` | `Units` | `300` | | `12-BEHAVIOUR-gestures.md#12-3-4-the-gesture-set` (G4) | proposed |
| `gestures.g4_max_duration_ms` | `Ms` | `400` | | `12-BEHAVIOUR-gestures.md#12-3-4-the-gesture-set` (G4) | proposed |
| `gestures.g4_axis_ratio` | `Fraction` | `2000` (2.0) | `|x| >= 2|y|` | `12-BEHAVIOUR-gestures.md#12-3-4-the-gesture-set` (G4) | proposed |
| `gestures.g4_settle_ms` | `Ms` | `50` | last motion before lift | `12-BEHAVIOUR-gestures.md#12-3-4-the-gesture-set` (G4) | proposed |
| `gestures.host_swipe_D_units` | `Units` | `1000` | `~38mm`; on our own surfaces | `12-BEHAVIOUR-gestures.md#12-3-6-one-finger-swipe-on-our-surfaces-host-decided` | proposed |
| `gestures.live_workspace_D_units` | `Units` | `1000` | alt `400` (R10 touchpad px) | `12-BEHAVIOUR-gestures.md#12-3-7-live-tracking-model-for-later-workspace-swipe-with-the-content-following` | proposed |
| `gestures.live_workspace_overshoot_scale` | `Fraction` | `250` (0.25) | `p' = bound + 0.25(p-bound)` | `12-BEHAVIOUR-gestures.md#12-3-7-live-tracking-model-for-later-workspace-swipe-with-the-content-following` | proposed |
| `gestures.live_workspace_overshoot_cap` | `Fraction` | `125` (0.125) | | `12-BEHAVIOUR-gestures.md#12-3-7-live-tracking-model-for-later-workspace-swipe-with-the-content-following` | proposed |
| `gestures.live_workspace_velocity_window_ms` | `Ms` | `80` | | `12-BEHAVIOUR-gestures.md#12-3-7-live-tracking-model-for-later-workspace-swipe-with-the-content-following` | proposed |
| `gestures.live_workspace_commit_velocity` | `Fraction` | `1500` (1.5 pages/s) | | `12-BEHAVIOUR-gestures.md#12-3-7-live-tracking-model-for-later-workspace-swipe-with-the-content-following` | proposed |
| `gestures.live_workspace_finish_min_ms` | `Ms` | `100` | | `12-BEHAVIOUR-gestures.md#12-3-7-live-tracking-model-for-later-workspace-swipe-with-the-content-following` | proposed |
| `gestures.live_workspace_finish_max_ms` | `Ms` | `400` | | `12-BEHAVIOUR-gestures.md#12-3-7-live-tracking-model-for-later-workspace-swipe-with-the-content-following` | proposed |
| `gestures.gesture_action_map` | `Map<Gesture, Action>` | Apple mapping shipped (see `12-BEHAVIOUR-gestures.md#12-6-configuration` table) | every `Gesture` variant remappable | `12-BEHAVIOUR-gestures.md#12-6-configuration` | settled (preference) |
| `gestures.contact_nibble_policy` | `ContactNibblePolicy::{PythonParity,KernelOnly}` | `PythonParity` | treats report nibble 1-2 as contact (current) vs only 3-4 ("down") | `12-BEHAVIOUR-gestures.md#12-9-open-decisions` item 3 | proposed |

### 3.10 `palm_rejection` (palmrest/gestures.toml)

| Key | Type | Default | Range / Alt | Source | Status |
| --- | --- | --- | --- | --- | --- |
| `rejection` | `Rejection::{On,Off}` | `On` | `Off` disables P1-P3 and P5 for Python bit-exact comparison | `12-BEHAVIOUR-gestures.md#12-6-configuration` | proposed |
| `palm_rejection.light_touch_size_max` | `Units` | `8` | P1 | `12-BEHAVIOUR-gestures.md#12-3-3-palm-and-rest-rejection` | proposed |
| `palm_rejection.side_band_x_min` | `Units` (signed) | `-900` | P2 | `12-BEHAVIOUR-gestures.md#12-3-3-palm-and-rest-rejection` | proposed |
| `palm_rejection.side_band_x_max` | `Units` (signed) | `1060` | P2 | `12-BEHAVIOUR-gestures.md#12-3-3-palm-and-rest-rejection` | proposed |
| `palm_rejection.rear_band_percent` | `Percent` | `15` | P2 | `12-BEHAVIOUR-gestures.md#12-3-3-palm-and-rest-rejection` | proposed |
| `palm_rejection.resting_move_floor_units` | `Units` | `30` | P2, within `resting_window_ms` | `12-BEHAVIOUR-gestures.md#12-3-3-palm-and-rest-rejection` | proposed |
| `palm_rejection.resting_window_ms` | `Ms` | `150` | P2 | `12-BEHAVIOUR-gestures.md#12-3-3-palm-and-rest-rejection` | proposed |
| `palm_rejection.large_contact_major_units` | `Units` | `64` | P3 | `12-BEHAVIOUR-gestures.md#12-3-3-palm-and-rest-rejection` | proposed |
| `palm_rejection.click_guard_ms` | `Ms` | `200` | P4 | `12-BEHAVIOUR-gestures.md#12-3-3-palm-and-rest-rejection` | proposed |
| `palm_rejection.mouse_moving_threshold_counts_s` | `Count` | `200` | P5 | `12-BEHAVIOUR-gestures.md#12-3-3-palm-and-rest-rejection` | proposed |
| `palm_rejection.mouse_moving_window_ms` | `Ms` | `100` | P5 | `12-BEHAVIOUR-gestures.md#12-3-3-palm-and-rest-rejection` | proposed |
| `palm_rejection.mouse_moving_lock_multiplier` | `Fraction` | `2000` (2.0x, 40->80) | P5 | `12-BEHAVIOUR-gestures.md#12-3-3-palm-and-rest-rejection` | proposed |

### 3.11 `menus` (sill/settings.toml; also carries `switcher.*`, `sound.*` and `window.*` per `13.6`'s own grouping)

| Key | Type | Default | Range / Alt | Source | Status |
| --- | --- | --- | --- | --- | --- |
| `menus.submenu_delay_ms` | `Ms` | `200` | `0..1000` | `13-BEHAVIOUR-menus-windows.md#13-6-configuration` | proposed |
| `menus.item_height_px` | `Px` | `22` | was 24 before the polish pass; alt `30` (design's Slim padding 6/8) | `13-BEHAVIOUR-menus-windows.md#13-3-3-menu-item-geometry-text-menus-bar-context-dock`; `13-BEHAVIOUR-menus-windows.md#13-9-open-decisions` item 1 | proposed — **flagged for review** |
| `menus.max_width_px` | `Px` | `420` | min 220 settled | `13-BEHAVIOUR-menus-windows.md#13-3-3-menu-item-geometry-text-menus-bar-context-dock` | proposed |
| `menus.separator_margin_px` | `Px` | `5` | was 4 before the polish pass | `13-BEHAVIOUR-menus-windows.md#13-3-3-menu-item-geometry-text-menus-bar-context-dock` | proposed |
| `menus.section_header_height_px` | `Px` | `22` | | `13-BEHAVIOUR-menus-windows.md#13-3-3-menu-item-geometry-text-menus-bar-context-dock` | settled style, height proposed |
| `menus.submenu_triangle_timeout_ms` | `Ms` | `300` | | `13-BEHAVIOUR-menus-windows.md#13-3-4-submenus` | proposed |
| `menus.pick_feedback` | `PickFeedback::{None,BlinkOnce}` | `None` | alt `BlinkOnce` (macOS blinks the chosen item once before closing) | `13-BEHAVIOUR-menus-windows.md#13-9-open-decisions` item 3 | proposed |
| `menus.first_mouse_window_ms` | `Ms` | `100` | activation-vs-click window on an inactive window's first click | `13-BEHAVIOUR-menus-windows.md#13-3-8-focus-and-raise-rules`; `06-INTERACTIONS.md#20-desktop-interactions-settled` (§20.4) | proposed |
| `menus.font_px` | `Px` | `13` | `9..=24` | `FINDINGS.md` "macOS polish"; `04-COMPONENTS.md` | proposed (polish pass, 2026-09-25) |
| `menus.highlight_radius_px` | `Px` | `6` | `0..=12` | `FINDINGS.md` "macOS polish"; `04-COMPONENTS.md` | proposed (polish pass, 2026-09-25) |
| `menus.tooltip_font_px` | `Px` | `12` | `9..=20` | `FINDINGS.md` "macOS polish"; `04-COMPONENTS.md` | proposed (polish pass, 2026-09-25) |
| `switcher.show_delay_ms` | `Ms` | `150` | `0..500` | `13-BEHAVIOUR-menus-windows.md#13-6-configuration` | proposed |
| `switcher.quick_tap_ms` | `Ms` | `100` | chord+modifier release within this = no UI | `13-BEHAVIOUR-menus-windows.md#13-3-5-app-switcher-cmd-tab` | proposed |
| `switcher.icon_size_px` | `Px` | `96` | alt macOS ~128 | `13-BEHAVIOUR-menus-windows.md#13-3-5-app-switcher-cmd-tab`; `13-BEHAVIOUR-menus-windows.md#13-9-open-decisions` item 6 | proposed |
| `switcher.cell_size_px` | `Px` | `112` | | `13-BEHAVIOUR-menus-windows.md#13-3-5-app-switcher-cmd-tab` | proposed |
| `switcher.cell_gap_px` | `Px` | `8` | | `13-BEHAVIOUR-menus-windows.md#13-3-5-app-switcher-cmd-tab` | proposed |
| `switcher.overflow_min_icon_px` | `Px` | `48` | | `13-BEHAVIOUR-menus-windows.md#13-3-5-app-switcher-cmd-tab` | proposed |
| `sound.theme` | `SoundTheme(String)` | `"freedesktop"` | | `13-BEHAVIOUR-menus-windows.md#13-6-configuration` | proposed |
| `sound.ui_sounds` | `UiSounds::{On,Off}` | `On` | | `13-BEHAVIOUR-menus-windows.md#13-6-configuration` | proposed |
| `sound.volume_feedback` | `VolumeFeedback::{On,Off}` | `On` | | `13-BEHAVIOUR-menus-windows.md#13-6-configuration` | proposed |
| `sound.event_screenshot` | `SoundName(String)` | `"screen-capture"` | | `13-BEHAVIOUR-menus-windows.md#13-3-10-ui-sounds` | proposed |
| `sound.event_trash_empty` | `SoundName(String)` | `"trash-empty"` | | `13-BEHAVIOUR-menus-windows.md#13-3-10-ui-sounds` | proposed |
| `sound.event_volume_step` | `SoundName(String)` | `"audio-volume-change"` | | `13-BEHAVIOUR-menus-windows.md#13-3-10-ui-sounds` | proposed |
| `sound.event_dock_remove` | `SoundName(String)` | `"item-deleted"` | | `13-BEHAVIOUR-menus-windows.md#13-3-10-ui-sounds` | proposed |
| `sound.event_notification` | `SoundName(String)` | `"message-new-instant"` | app hint wins if present | `13-BEHAVIOUR-menus-windows.md#13-3-10-ui-sounds` | proposed |
| `sound.event_critical_alert` | `SoundName(String)` | `"dialog-warning"` | | `13-BEHAVIOUR-menus-windows.md#13-3-10-ui-sounds` | proposed |
| `sound.event_invalid_key` | `SoundName(String)` | `"bell"` | | `13-BEHAVIOUR-menus-windows.md#13-3-10-ui-sounds` | proposed |

### 3.12 `notifications` (sill/settings.toml)

`notifications.banner_material` is in 3.1 (it is `Appearance` vocabulary, read from
`appearance.toml`); everything else lives here.

| Key | Type | Default | Range / Alt | Source | Status |
| --- | --- | --- | --- | --- | --- |
| `notifications.dnd` | `Dnd::{On,Off}` | `Off` | | `13-BEHAVIOUR-menus-windows.md#13-6-configuration` | proposed |
| `notifications.banner_style` | `BannerStyle::{Banner,Alert,None}` per app | `Banner` | | `13-BEHAVIOUR-menus-windows.md#13-6-configuration` | settled (preference), R8 |
| `notifications.position_right_px` | `Px` | `8` | | `13-BEHAVIOUR-menus-windows.md#13-3-6-notifications` | settled (R8) |
| `notifications.position_top_gap_px` | `Px` | `8` | on top of `bar.height_px` | `13-BEHAVIOUR-menus-windows.md#13-3-6-notifications` | settled (R8) |
| `notifications.banner_width_px` | `Px` | `360` | | `13-BEHAVIOUR-menus-windows.md#13-3-6-notifications` | proposed |
| `notifications.banner_min_height_px` | `Px` | `64` | | `13-BEHAVIOUR-menus-windows.md#13-3-6-notifications` | proposed |
| `notifications.banner_padding_px` | `Px` | `12` | | `13-BEHAVIOUR-menus-windows.md#13-3-6-notifications` | proposed |
| `notifications.icon_px` | `Px` | `32` | | `13-BEHAVIOUR-menus-windows.md#13-3-6-notifications` | proposed |
| `notifications.banner_hold_ms` | `Ms` | `5200` | fixed ~5s; app `expire_timeout` ignored except 0 | `13-BEHAVIOUR-menus-windows.md#13-3-6-notifications`; `05-MOTION.md#10-shell-motion` | settled (R8), preference-adjacent |
| `notifications.banner_entry_direction` | `BannerEntry::{FromRight,FromBelow}` | `FromRight` | | `05-MOTION.md#12-open-decisions` item 7 | proposed |
| `notifications.hover_min_remaining_ms` | `Ms` | `1500` | | `13-BEHAVIOUR-menus-windows.md#13-3-6-notifications` | settled (R8), number proposed |
| `notifications.close_button_px` | `Px` | `18` | | `13-BEHAVIOUR-menus-windows.md#13-3-6-notifications` | proposed |
| `notifications.swipe_dismiss_px` | `Px` | `80` | | `13-BEHAVIOUR-menus-windows.md#13-3-6-notifications` | proposed |
| `notifications.swipe_dismiss_velocity_px_s` | `Scalar` | `600.0` | | `13-BEHAVIOUR-menus-windows.md#13-3-6-notifications` | proposed |
| `notifications.swipe_damping` | `Fraction` | `250` (0.25) | leftward drag damping | `13-BEHAVIOUR-menus-windows.md#13-3-6-notifications` | proposed |
| `notifications.stack_max` | `Count` | `3` | | `13-BEHAVIOUR-menus-windows.md#13-3-6-notifications` | proposed |
| `notifications.stack_gap_px` | `Px` | `8` | | `13-BEHAVIOUR-menus-windows.md#13-3-6-notifications` | proposed |
| `notifications.group_offset_px` | `Px` | `4` | stacked-layer indicator | `13-BEHAVIOUR-menus-windows.md#13-3-6-notifications` | proposed |
| `notifications.server` | `NotificationServer::{Own,Passive}` | `Own` | `Own` asks for org.freedesktop.Notifications without replacing a holder; `Passive` leaves it to another daemon | `20-SURFACES.md#1-6-notifications`; `13-BEHAVIOUR-menus-windows.md#13-3-6`; sill M6 freeze | proposed (M6 freeze, 2026-09-25) |
| `notifications.banner_position` | `BannerPosition::{TopRight,BottomRight}` | `TopRight` |  | `20-SURFACES.md#1-6-notifications`; `13-BEHAVIOUR-menus-windows.md#13-3-6`; sill M6 freeze | proposed (M6 freeze, 2026-09-25) |
| `notifications.history_cap` | `Count` | `100` | `0..=1000` | `20-SURFACES.md#1-6-notifications`; `13-BEHAVIOUR-menus-windows.md#13-3-6`; sill M6 freeze | proposed (M6 freeze, 2026-09-25) |
| `notifications.center_width_px` | `Px` | `384` | `280..=600` | `20-SURFACES.md#1-6-notifications`; `13-BEHAVIOUR-menus-windows.md#13-3-6`; sill M6 freeze | proposed (M6 freeze, 2026-09-25) |
| `notifications.swipe` | `NotificationSwipe::{KeepInCenter,Dismiss}` | `KeepInCenter` | macOS keeps a swiped banner in Notification Center (design/13 said dismiss); to be confirmed when M6 lane a reports | `20-SURFACES.md#1-6-notifications`; `13-BEHAVIOUR-menus-windows.md#13-3-6`; sill M6 freeze | proposed (M6 freeze, 2026-09-25) |

### 3.13 `control_center` (sill/settings.toml)

Module list against the Claude Doc spec (rev 31): `13-BEHAVIOUR-menus-windows.md#13-9-open-decisions`
item 4 — **could not find a single sourced default**; the plan-derived order below is used
until that comparison happens (see handback report).

| Key | Type | Default | Range / Alt | Source | Status |
| --- | --- | --- | --- | --- | --- |
| `control_center.width_px` | `Px` | `320` | | `13-BEHAVIOUR-menus-windows.md#13-3-7-control-center` | proposed |
| `control_center.grid_columns` | `Count` | `2` | | `13-BEHAVIOUR-menus-windows.md#13-3-7-control-center` | proposed |
| `control_center.grid_gap_px` | `Px` | `8` | | `13-BEHAVIOUR-menus-windows.md#13-3-7-control-center` | proposed |
| `control_center.grid_padding_px` | `Px` | `12` | | `13-BEHAVIOUR-menus-windows.md#13-3-7-control-center` | proposed |
| `control_center.modules` | `Vec<ControlCenterModule>` | `[Wifi, Bluetooth, Focus, Display, Sound, NowPlaying, Appearance, Battery]` | plan's service list; Claude Doc spec (rev 31) may differ | `13-BEHAVIOUR-menus-windows.md#13-3-7-control-center` | proposed, partial |
| `control_center.bottom_margin_px` | `Px` | `16` | `0..=64`; the panel scrolls past `output_h - bar_h - this` | `13-BEHAVIOUR-menus-windows.md#13-3-7-control-center` | proposed (M5 freeze, 2026-09-25) |
| `control_center.menu_bar_wifi` | `InMenuBar::{Show,Hide}` | `Show` | the module as its own bar item, left of the control center item; a click opens that module's detail pane directly | `20-SURFACES.md#1-5-control-center`; user direction 2026-09-25 (controls at the top right, macOS "Show in Menu Bar") | proposed (M5 freeze, 2026-09-25) |
| `control_center.menu_bar_bluetooth` | `InMenuBar::{Show,Hide}` | `Hide` | the module as its own bar item, left of the control center item; a click opens that module's detail pane directly | `20-SURFACES.md#1-5-control-center`; user direction 2026-09-25 (controls at the top right, macOS "Show in Menu Bar") | proposed (M5 freeze, 2026-09-25) |
| `control_center.menu_bar_sound` | `InMenuBar::{Show,Hide}` | `Show` | the module as its own bar item, left of the control center item; a click opens that module's detail pane directly | `20-SURFACES.md#1-5-control-center`; user direction 2026-09-25 (controls at the top right, macOS "Show in Menu Bar") | proposed (M5 freeze, 2026-09-25) |
| `control_center.menu_bar_display` | `InMenuBar::{Show,Hide}` | `Hide` | the module as its own bar item, left of the control center item; a click opens that module's detail pane directly | `20-SURFACES.md#1-5-control-center`; user direction 2026-09-25 (controls at the top right, macOS "Show in Menu Bar") | proposed (M5 freeze, 2026-09-25) |
| `control_center.menu_bar_battery` | `InMenuBar::{Show,Hide}` | `Show` | the module as its own bar item, left of the control center item; a click opens that module's detail pane directly | `20-SURFACES.md#1-5-control-center`; user direction 2026-09-25 (controls at the top right, macOS "Show in Menu Bar") | proposed (M5 freeze, 2026-09-25) |
| `control_center.menu_bar_now_playing` | `InMenuBar::{Show,Hide}` | `Hide` | the module as its own bar item, left of the control center item; a click opens that module's detail pane directly | `20-SURFACES.md#1-5-control-center`; user direction 2026-09-25 (controls at the top right, macOS "Show in Menu Bar") | proposed (M5 freeze, 2026-09-25) |

### 3.14 `spaces` (sill/settings.toml)

Defaults applied when a workspace has no stored `SpaceLook` in `spaces.json` (section 1 rule
3). Preset table itself (`21-SPACES.md#4-presets-and-defaults-per-workspace-index`) is fixed
data, not a key.

| Key | Type | Default | Range / Alt | Source | Status |
| --- | --- | --- | --- | --- | --- |
| `spaces.material_tint_alpha` | `Percent` | `80` | alias of `appearance.material_tint_alpha` (3.1) kept in this domain's table for discoverability | `21-SPACES.md#3-where-the-tokens-apply` | proposed |
| `spaces.default_grain` | `Count` (0..100) | `40` | presets 1/2 keep their own 35/55 | `21-SPACES.md#4-presets-and-defaults-per-workspace-index` | proposed |
| `spaces.default_card_accent` | `CardAccent::{Postmark,SpaceHue}` | `Postmark` | | `21-SPACES.md#4-presets-and-defaults-per-workspace-index` | proposed |
| `spaces.overlay_tint` | `OverlayTint::{Off,On}` | `Off` | applies to notifications, power menu, lock and polkit (doc answers them as one "no"); the OSD is settled "yes" as of 2026-09-24 and always tints, so this key no longer covers it | `21-SPACES.md#3-where-the-tokens-apply` | proposed |
| `spaces.mail_frame_policy` | `MailSpacePolicy::{Own,Workspace,OwnFallbackWorkspace}` | `OwnFallbackWorkspace` | poles are `Own` / `Workspace`; doc's actual proposal is the fallback hybrid | `21-SPACES.md#11-open-decisions` item 1 | proposed — **flagged for review** ("mail-spaces policy Own vs Workspace") |
| `spaces.wallpaper_follows_space` | `WallpaperPolicy::{Independent,PerWorkspace}` | `Independent` | | `21-SPACES.md#8-wallpaper-proposed` | proposed |
| `spaces.dock_look_source` | `DockLookSource::{OwnOutput,FocusedWindow}` | `OwnOutput` | multi-output only | `21-SPACES.md#11-open-decisions` item 5 | proposed |
| `spaces.lookup_order` | `SpaceLookLookup::{ByIdThenIndex}` (single variant today; kept as an enum, not a bool, for a future `ByIndexOnly` fallback) | `ByIdThenIndex` | | `21-SPACES.md#10-storage-settled-path-proposed-schema` | proposed |
| `spaces.wallpaper_drawer` | `WallpaperDrawer::{Cosmic,Shell}` | `Cosmic` | Advanced. `Cosmic` = COSMIC's own background service; `Shell` = the shell's wallpaper surface, which cross-fades with light and dark. Default stays `Cosmic` until shell-host paints a background layer's second frame (shell-host F40, sill F171/G21) | `21-SPACES.md#8-wallpaper-proposed`; sill FINDINGS "M2 wallpaper" | proposed (2026-09-25) |

### 3.15 `display` (sill/settings.toml)

All Advanced (§5). The display service reads the EDID, classifies the panel's gamut, and drives DDC/CI.

| Key | Type | Default | Range / Alt | Source | Status |
| --- | --- | --- | --- | --- | --- |
| `display.scale` | `ScalePolicy::{Auto,Manual}` | `Auto` | `Auto` derives the scale from the EDID's physical size aiming at `density_target_ppi` (1.5 on a 27" 4K panel); `Manual` uses `scale_overrides` | sill FINDINGS "Displays" (display service, 2026-09-25); `13-BEHAVIOUR-menus-windows.md` | proposed (2026-09-25) |
| `display.scale_rounding` | `ScaleRounding::{PreferWhole,Nearest}` | `PreferWhole` | Advanced. "Round a sharp display up to a whole scale (crisper, like a Mac's default) or to the nearest step (more space)." Rule: at an ideal of 1.5 or more, take the whole scale (capped at 2) when it leaves at least 1280 px across; else the nearest step. The user preferred 2.0 to 1.5 on the 27" 4K panel (2026-09-25) | sill FINDINGS "Displays" | proposed (2026-09-25) |
| `display.scale_overrides` | `Text` | `""` | `<identity> = <scale>` pairs separated by `;`. A `Vec` of text is not derivable in the settings macro yet (quire gap: ds-settings-derive list types) | sill FINDINGS "Displays" (display service, 2026-09-25); `13-BEHAVIOUR-menus-windows.md` | proposed (2026-09-25) |
| `display.density_target_ppi` | `Count` | `109` | `72..=220`; 109 pt/inch is the target that makes text the size it is on a Mac | sill FINDINGS "Displays" (display service, 2026-09-25); `13-BEHAVIOUR-menus-windows.md` | proposed (2026-09-25) |
| `display.brightness_keys_target` | `BrightnessTarget::{PointerOutput,AllOutputs}` | `PointerOutput` | in practice the active window's output; DDC/CI | sill FINDINGS "Displays" (display service, 2026-09-25); `13-BEHAVIOUR-menus-windows.md` | proposed (2026-09-25) |
| `display.hardware_volume` | `HardwareVolume::{Off,WhenMonitorIsOutput}` | `WhenMonitorIsOutput` | DDC/CI volume when the monitor is the audio output | sill FINDINGS "Displays" (display service, 2026-09-25); `13-BEHAVIOUR-menus-windows.md` | proposed (2026-09-25) |
| `display.night_warmth` | `NightWarmth::{Off,Hardware}` | `Off` | warmth through the monitor's own DDC/CI controls | sill FINDINGS "Displays" (display service, 2026-09-25); `13-BEHAVIOUR-menus-windows.md` | proposed (2026-09-25) |
| `display.night_warmth_strength` | `Percent` | `50` | `0..=100` | sill FINDINGS "Displays" (display service, 2026-09-25); `13-BEHAVIOUR-menus-windows.md` | proposed (2026-09-25) |
| `display.font_rendering` | `FontRendering::{Auto,Off}` | `Auto` | `Auto` sets hinting and subpixel positioning from the output's ppi | sill FINDINGS "Displays" (display service, 2026-09-25); `13-BEHAVIOUR-menus-windows.md` | proposed (2026-09-25) |
| `display.color_management` | `ColorManagement::{Auto,Off}` | `Auto` | stored only: cosmic-comp 1.8.0 has no `wp_color_manager_v1`, so the palette fits sRGB there; KWin 6.7.5 offers parametric Display P3 | sill FINDINGS "Displays" (display service, 2026-09-25); `13-BEHAVIOUR-menus-windows.md` | proposed (2026-09-25) |

### 3.16 `osd` (sill/settings.toml)

All Advanced (§5). The on-screen display for volume and brightness (design/20 §1.7): a Material::Osd card at the top right under the bar (as current macOS), styled like a control-center slider module, that takes the Space gradient tint, holds, then fades.

| Key | Type | Default | Range / Alt | Source | Status |
| --- | --- | --- | --- | --- | --- |
| `osd.enabled` | `OsdEnabled::{On,Off}` | `On` | shows sill's own volume and brightness changes | `20-SURFACES.md#1-7-osd`; sill FINDINGS "OSD" | proposed (2026-09-25) |
| `osd.hold_ms` | `Ms` | `1500` | `300..=10000`; after the last change and before the fade | `20-SURFACES.md#1-7-osd`; sill FINDINGS "OSD" | proposed (2026-09-25) |
| `osd.position` | `OsdPosition::{TopRight,BottomCentre}` | `TopRight` | top right under the bar, as current macOS (user, 2026-09-25); `BottomCentre` above the dock | `20-SURFACES.md#1-7-osd`; sill FINDINGS "OSD" | proposed (2026-09-25) |
| `osd.margin_px` | `Px` | `24` | `0..=400`; the gap from the bar's reserve (at the top) or the dock's (at the bottom) to the card; was `osd.bottom_margin_px` before the top-right decision | `20-SURFACES.md#1-7-osd`; sill FINDINGS "OSD" | proposed (2026-09-25) |

### 3.17 `power_menu` (sill/settings.toml)

The power menu (design/20 §1.8): a centred sheet with Log out, Restart, Shut down, Suspend and Cancel.

| Key | Type | Default | Range / Alt | Source | Status |
| --- | --- | --- | --- | --- | --- |
| `power_menu.default_action` | `PowerMenuDefault::{LogOut,Restart,ShutDown,Suspend}` | `ShutDown` | the button Return presses when the menu opens | `20-SURFACES.md#1-8-power-menu`; sill M5 freeze | proposed (2026-09-25) |
| `power_menu.material` | `PanelMaterial::{Sheet,Popover}` | `Sheet` | | `03-COLOR.md#17-3-material-per-surface`; sill M5 freeze | proposed (2026-09-25) |

Focus (Do Not Disturb) reads the existing `notifications.dnd`; a second focus mode would need a `notifications.focus` row.

### 3.18 `window` (the host app's settings file: sill/settings.toml for shell apps; mailo reads them itself)

The client-decorated window frame (design/13 §13.3.11, design/04 "Window frame"): the titlebar drag threshold and the green light's tiling-menu hold times. All Advanced (§5).

| Key | Type | Default | Range / Alt | Source | Status |
| --- | --- | --- | --- | --- | --- |
| `window.move_threshold_px` | `Px` | `4` | `1..=16` | `13-BEHAVIOUR-menus-windows.md#13-3-11-window-frame-our-client-decorated-windows-settled-2026-09-25` | proposed |
| `window.tile_menu_press_ms` | `Ms` | `500` | `200..=2000` | `13-BEHAVIOUR-menus-windows.md#13-3-11-window-frame-our-client-decorated-windows-settled-2026-09-25` | proposed |
| `window.tile_menu_hover_ms` | `Ms` | `800` | `450..=3000`; the 450 ms hover intent plus a further hold | `13-BEHAVIOUR-menus-windows.md#13-3-11-window-frame-our-client-decorated-windows-settled-2026-09-25` | proposed |

### 3.19 `session` (sill/settings.toml)

The parts a sill session borrows until M11 draws its own (design/20 §1.9 lock screen, §1.10 polkit prompt; sill FINDINGS "M7"): which lock screen and which polkit agent `dist/sill-session` starts. Both are a name from a fixed vocabulary or a command line, so a user can point at a program this list does not know. Page Accounts, all Advanced (§5). Idle timeouts are not sill's: cosmic-idle keeps them in COSMIC's own config.

| Key | Type | Default | Range / Alt | Source | Status |
| --- | --- | --- | --- | --- | --- |
| `session.locker` | `String` | `"auto"` | `auto` (cosmic-greeter, else swaylock, else hyprlock, whichever is installed; under cosmic-session, COSMIC's own resident locker), `off`, `cosmic-greeter`, `swaylock`, `hyprlock`, or a command line run by `/bin/sh -c` | `20-SURFACES.md#1-9-lock-screen-spec-tier-1`; sill FINDINGS "M7" | proposed (2026-09-26) |
| `session.polkit_agent` | `String` | `"auto"` | `auto` (polkit-kde, polkit-gnome, lxqt-policykit, polkit-mate, then cosmic-osd last: running cosmic-osd only for polkit would add its own volume popup beside sill's OSD), `off`, one of those names, or a command line run by `/bin/sh -c` | `20-SURFACES.md#1-10-polkit-prompt-spec-tier-1`; sill FINDINGS "M7" | proposed (2026-09-26) |

### 3.20 `widgets` (sill/settings.toml)

The widgets under the notification center's history and the desktop widget layer (design/20 §1.14; sill FINDINGS "M10 freeze" F404, F405). A small widget is one grid cell, a medium two by one, a large two by two; where a desktop widget sits is saved state, not a key. Page Dock, all Advanced (§5).

| Key | Type | Default | Range / Alt | Source | Status |
| --- | --- | --- | --- | --- | --- |
| `widgets.center` | `Vec<WidgetKind>` (`Calendar, UpNext, Battery, NowPlaying, WorldClock`) | `[Calendar, UpNext, Battery]` | which widgets show under the notifications, in order | `20-SURFACES.md#1-14-desktop-widgets-spec-tier-2`; sill FINDINGS "M10 freeze" F404 | proposed (M10 freeze, 2026-09-26) |
| `widgets.desktop` | `DesktopWidgets::{Off,On}` | `Off` | the desktop widget layer (macOS Sonoma); `sill widgets toggle-desktop` flips it | same; F405 | proposed (M10 freeze, 2026-09-26) |
| `widgets.desktop_widgets` | `Vec<WidgetKind>` | `[Calendar, Battery]` | which widgets the desktop shows; where they sit is saved state (`desktop-widgets.json`), not a key | same; F405 | proposed (M10 freeze, 2026-09-26) |
| `widgets.desktop_cell_px` | `Px` | `164` | `120..=240`; a small widget is one cell, a medium 2x1, a large 2x2 | same; F405 | proposed (M10 freeze, 2026-09-26) |
| `widgets.desktop_gap_px` | `Px` | `16` | `0..=48`; between cells and around the grid | same; F405 | proposed (M10 freeze, 2026-09-26) |
| `widgets.world_clocks` | `Text` | `""` | IANA zone names separated by `;` (a `Vec` of text is not derivable yet, as for `display.scale_overrides`) | same; F404 | proposed (M10 freeze, 2026-09-26) |

### 3.21 `calendar` (sill/settings.toml)

The calendar sources and the Calendar and Up Next widgets (design/20 §1.12; sill F401–F403): `.ics` files and vdir folders, watched; no CalDAV client of sill's own. Page Notifications, all Advanced.

| Key | Type | Default | Range / Alt | Source | Status |
| --- | --- | --- | --- | --- | --- |
| `calendar.sources` | `Text` | `"auto"` | `auto` (KOrganizer's file via Akonadi's iCal resource, `~/.local/share/sill/calendars`, `~/.calendars`), `off`, or `.ics` files and folders separated by `;` | `20-SURFACES.md#1-12-calendar-popup-spec-tier-2`; F401, F402 | proposed (M10 freeze, 2026-09-26) |
| `calendar.first_weekday` | `FirstWeekday::{Monday,Sunday}` | `Monday` | the month's first column (ISO 8601) | same; F403 | proposed (M10 freeze, 2026-09-26) |
| `calendar.week_numbers` | `WeekNumbers::{Hide,Show}` | `Hide` | a column of ISO week numbers | same; F403 | proposed (M10 freeze, 2026-09-26) |
| `calendar.up_next_days` | `Count` | `7` | `1..=31`; how far Up Next looks ahead | same; F403 | proposed (M10 freeze, 2026-09-26) |
| `calendar.up_next_max` | `Count` | `3` | `1..=10`; how many events Up Next lists | same; F403 | proposed (M10 freeze, 2026-09-26) |

### 3.22 `screenshot` (sill/settings.toml)

The floating thumbnail after a screenshot and the tool that takes one (design/20 §1.13; sill F406, F408): the thumbnail holds `ToastHold` (5200 ms) and pauses under the pointer; dragging it out is a file drag once shell-host has a drag source (G120). Page Keyboard and Shortcuts, all Advanced.

| Key | Type | Default | Range / Alt | Source | Status |
| --- | --- | --- | --- | --- | --- |
| `screenshot.thumbnail` | `ShotThumbnail::{Show,Hide}` | `Show` | the floating thumbnail after a screenshot (macOS's "Show Floating Thumbnail") | `20-SURFACES.md#1-13-screenshot-thumbnail-spec-integrated-experience-priority-pick`; F408 | proposed (M10 freeze, 2026-09-26) |
| `screenshot.thumbnail_hold_ms` | `Ms` | `5200` | `1000..=30000`; paused while hovered (`ToastHold`) | same; F408 | proposed (M10 freeze, 2026-09-26) |
| `screenshot.folders` | `Text` | `"auto"` | `auto` (Pictures and Pictures/Screenshots), `off`, or folders separated by `;`, watched for new screenshots | same; F406 | proposed (M10 freeze, 2026-09-26) |
| `screenshot.tool` | `Text` | `"auto"` | `auto` (cosmic-screenshot), `off`, or a command line run by `/bin/sh -c` with the kind as `$1` and the folder as `$2` | same; F406 | proposed (M10 freeze, 2026-09-26) |

### 3.23 `hot_corners` (sill/settings.toml)

One invisible square per enabled corner (design/20 §1.16; design/13 §13.3.12 for the dwell, re-arm, modifier and action rules; sill F409). Page Dock, all Advanced.

| Key | Type | Default | Range / Alt | Source | Status |
| --- | --- | --- | --- | --- | --- |
| `hot_corners.top_left` | `CornerAction::{None,Launcher,NotificationCenter,ControlCenter,ShowDesktop,Lock,Workspaces,Command}` | `None` | what the corner does; `None` spawns no surface | `20-SURFACES.md#1-16-hot-corners-spec-integrated-experience`; F409 | proposed (M10 freeze, 2026-09-26) |
| `hot_corners.top_right` | `CornerAction` | `None` | as `top_left` | same | proposed (M10 freeze, 2026-09-26) |
| `hot_corners.bottom_left` | `CornerAction` | `None` | as `top_left` | same | proposed (M10 freeze, 2026-09-26) |
| `hot_corners.bottom_right` | `CornerAction` | `None` | as `top_left` (macOS ships Quick Note here; sill has none until M12) | same | proposed (M10 freeze, 2026-09-26) |
| `hot_corners.top_left_command` | `Text` | `""` | run by `/bin/sh -c` when the corner is `Command` | same | proposed (M10 freeze, 2026-09-26) |
| `hot_corners.top_right_command` | `Text` | `""` | as `top_left_command` | same | proposed (M10 freeze, 2026-09-26) |
| `hot_corners.bottom_left_command` | `Text` | `""` | as `top_left_command` | same | proposed (M10 freeze, 2026-09-26) |
| `hot_corners.bottom_right_command` | `Text` | `""` | as `top_left_command` | same | proposed (M10 freeze, 2026-09-26) |
| `hot_corners.dwell_ms` | `Ms` | `150` | `0..=2000`; how long the pointer rests before the corner acts | `20-SURFACES.md#3-open-decisions` item 7; F409 | proposed (M10 freeze, 2026-09-26) |
| `hot_corners.rearm_ms` | `Ms` | `500` | `0..=5000`; acts again only after the pointer has left and this long | same | proposed (M10 freeze, 2026-09-26) |
| `hot_corners.size_px` | `Px` | `2` | `1..=8`; the invisible square's side | `20-SURFACES.md#1-16-hot-corners-spec-integrated-experience` (2 x 2 px) | proposed (M10 freeze, 2026-09-26) |

## 4. Rust shape

Adds to `crates/ds-settings` (appearance/icons/motion) and a new `sill-settings` module
(bar/dock/launcher/scroll/scrollbar/menus/notifications/control_center/spaces); `gestures`/
`palm_rejection` live in `palmrest`'s own crate but follow the identical shape so the three
loaders share one macro/derive story.

Shared newtypes (beyond the four the catalogue names, added per `CONVENTIONS.md#0-design-style`
"newtype every identifier and every unit"):

```rust
pub struct Px(pub u16);
pub struct Ms(pub u16);
pub struct Percent(pub u8);           // 0..=100, clamped on construction
pub struct Fraction(pub u16);         // permille: 1000 = 1.0; not clamped (gains exceed 1.0)
pub struct Count(pub u16);
pub struct Scalar(pub f32);           // dimensionless physics constant, no natural unit above
pub struct Units(pub i32);            // raw touchpad/report units, signed
```

Every domain is one small struct, one field per key, `#[derive(Clone, Debug, PartialEq, Serialize, Deserialize)]`,
`#[serde(default)]` on every field (`CONVENTIONS.md#3-serde`: "every field added after the
first release carries `#[serde(default)]`, unless no safe default exists" — every field here
has one, by construction, since the whole point is a shipped default). Enums use
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
    pub bounce: Bounce,
    pub launch_animation: LaunchAnim,
    pub click_active_app: ActiveClick,
    pub trash: TrashTile,
    pub pill_radius_px: Px,
    pub edge_clamp_px: Px,
    pub overflow_min_tile_px: Px,
    pub running_dot_diameter_px: Px,
    pub badge_size_px: Px,
    pub badge_radius_px: Px,
    pub progress_ring_diameter_px: Px,
    pub progress_ring_stroke_px: Px,
    pub separator_height_px: Px,
    pub magnify_enter_ms: Ms,
    pub magnify_leave_ms: Ms,
    pub hover_label_offset_px: Px,
    pub hover_label_warm_ms: Ms,
    pub bounce_period_ms: Ms,
    pub bounce_launch_cap_ms: Ms,
    pub hold_to_menu_ms: Ms,
    pub hold_to_menu_move_px: Px,
    pub remove_threshold_px: Px,
    pub spring_load_ms: Ms,
    pub minimize_debounce_ms: Ms,
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
pub enum Bounce { #[default] On, Off }
#[derive(Clone, Copy, Debug, PartialEq, Eq, Default, Serialize, Deserialize)]
#[serde(rename_all = "snake_case")]
pub enum LaunchAnim { #[default] On, Off }
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
            magnified_size_px: Px(96),
            influence_radius_px: Px(96),
            tile_gap_px: Px(4),
            progress_style: ProgressStyle::Ring,
            autohide: AutoHide::Off,
            autohide_delay_ms: Ms(200),
            autohide_slide_ms: Ms(500),
            autohide_trigger_strip_px: Px(4),
            position: DockPosition::Bottom,
            indicators: Indicators::On,
            bounce: Bounce::On,
            launch_animation: LaunchAnim::On,
            click_active_app: ActiveClick::Cycle,
            trash: TrashTile::On,
            pill_radius_px: Px(22),
            edge_clamp_px: Px(8),
            overflow_min_tile_px: Px(24),
            running_dot_diameter_px: Px(4),
            badge_size_px: Px(18),
            badge_radius_px: Px(9),
            progress_ring_diameter_px: Px(20),
            progress_ring_stroke_px: Px(3),
            separator_height_px: Px(36),
            magnify_enter_ms: Ms(120),
            magnify_leave_ms: Ms(200),
            hover_label_offset_px: Px(7),
            hover_label_warm_ms: Ms(400),
            bounce_period_ms: Ms(500),
            bounce_launch_cap_ms: Ms(10_000),
            hold_to_menu_ms: Ms(600),
            hold_to_menu_move_px: Px(8),
            remove_threshold_px: Px(64),
            spring_load_ms: Ms(500),
            minimize_debounce_ms: Ms(100),
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

`AppearanceSettings` (theme, look, warmth, accent, motion_level, material_tint_alpha),
`IconsSettings`, `BarSettings`, `LauncherSettings`, `GesturesSettings` (palmrest crate:
`scroll_speed`/`lock_*`/`swipe_*`/`tap_*`/`foreign_output`/`g4_*`/`live_workspace_*`/
`gesture_action_map`), `PalmRejectionSettings`, `MenusSettings` (carries `switcher_*` and
`sound_*` as nested structs `SwitcherSettings`, `SoundSettings` per `CONVENTIONS.md#0-design-style`
"if half a struct's methods never touch half its fields, it is two types" — switcher and sound
are genuinely separate concerns, nested rather than flattened), `NotificationsSettings`,
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

/// One variant per domain, so a surface subscribes to only the domains it draws.
/// Pure: no I/O, no clock (`CONVENTIONS.md#6-time` — this is comparison, not a timed effect).
#[derive(Clone, Copy, Debug, PartialEq, Eq)]
pub enum SettingsChange {
    Appearance, Icons, Bar, Dock, Launcher, Scroll, Scrollbar, Menus, Notifications,
    ControlCenter, Spaces, Gestures, PalmRejection,
}

pub fn apply(old: &Settings, new: &Settings) -> Vec<SettingsChange> {
    let mut out = Vec::new();
    if old.appearance.appearance != new.appearance.appearance { out.push(SettingsChange::Appearance); }
    if old.appearance.icons != new.appearance.icons { out.push(SettingsChange::Icons); }
    if old.shell.bar != new.shell.bar { out.push(SettingsChange::Bar); }
    if old.shell.dock != new.shell.dock { out.push(SettingsChange::Dock); }
    if old.shell.launcher != new.shell.launcher { out.push(SettingsChange::Launcher); }
    if old.shell.scroll != new.shell.scroll { out.push(SettingsChange::Scroll); }
    if old.shell.scrollbar != new.shell.scrollbar { out.push(SettingsChange::Scrollbar); }
    if old.shell.menus != new.shell.menus { out.push(SettingsChange::Menus); }
    if old.shell.notifications != new.shell.notifications { out.push(SettingsChange::Notifications); }
    if old.shell.control_center != new.shell.control_center { out.push(SettingsChange::ControlCenter); }
    if old.shell.osd != new.shell.osd { out.push(SettingsChange::Osd); }
    if old.shell.power_menu != new.shell.power_menu { out.push(SettingsChange::PowerMenu); }
    if old.shell.display != new.shell.display { out.push(SettingsChange::Display); }
    if old.shell.spaces != new.shell.spaces { out.push(SettingsChange::Spaces); }
    if old.gestures.gestures != new.gestures.gestures { out.push(SettingsChange::Gestures); }
    if old.gestures.palm_rejection != new.gestures.palm_rejection { out.push(SettingsChange::PalmRejection); }
    out
}
```

`apply` is deliberately whole-struct `PartialEq`, not per-field: a domain is small enough
(largest is `dock` at 35 fields) that "one field changed" and "recompute the domain" cost the
same, and per-field diffing would need a second hand-written table per domain that section 6's
"every key appears in the Rust schema" test would have to check twice.

## 5. Settings UI mapping

All proposed (this whole page taxonomy did not exist before 2026-09-24). "Advanced" = file
only in v1, no widget; a later wave may promote one if the user asks.

| Settings app page | Keys shown |
| --- | --- |
| **Appearance** | `appearance.theme`, `appearance.look`, `appearance.warmth` (only when look=Candy), `appearance.accent`, `appearance.motion_level`, `icons.style`, `icons.monochrome_tint` (only when style=Monochrome) |
| **Dock** | `dock.magnification`, `dock.tile_size_px`, `dock.autohide`, `dock.autohide_delay_ms`, `dock.autohide_slide_ms`, `dock.position`, `dock.indicators`, `dock.bounce`, `dock.launch_animation`, `dock.click_active_app`, `dock.trash` |
| **Mouse & Gestures** | `scroll.natural`, `scroll.speed`, `swipe.workspace_mode`, `tap.*` (as a single "double-tap sensitivity" control), `rejection`, `foreign_output`, `gestures.gesture_action_map` (the remap table) |
| **Keyboard / Shortcuts** | none of this doc's keys are keyboard shortcuts (those are COSMIC `system_actions`/`custom` shortcut files, PLAN "Design: `<shell>`"); this page is out of `22-SETTINGS`'s scope |
| **Notifications** | `notifications.dnd`, `notifications.banner_style` (per app), `sound.ui_sounds`, `sound.volume_feedback` |
| **Spaces** | `spaces.mail_frame_policy`, `spaces.wallpaper_follows_space`; the per-workspace dots/grain/theme/accent editor writes `spaces.json` (state), not these defaults |
| **Advanced** (file only) | everything else in section 3: `bar.*`, `menus.*`, `switcher.*`, `control_center.*`, `icons.*` (except `style` and `monochrome_tint`), `scrollbar.*`, `scroll.momentum_*`/`rubber_band_*`/`wheel_detent_px`, `dock.*` geometry beyond the Dock page's list above, `palm_rejection.*`, `gestures.g4_*`/`live_workspace_*`, `spaces.default_grain`/`default_card_accent`/`overlay_tint`/`dock_look_source` |

## 6. Acceptance

1. **Round-trip test per domain**: `Domain::default()` -> `toml::to_string` -> `toml::from_str`
   -> `assert_eq!` back to `Domain::default()`, for every domain struct in section 4 (13 tests,
   one per domain; `CONVENTIONS.md#3-serde` "every persisted type has a round-trip test").
2. **Lenient-parse table**: one test per type family feeding a bad value and asserting the
   field falls back to default, file otherwise intact —

   | Bad input | Field | Expected |
   | --- | --- | --- |
   | unknown enum string (`"bounceyy"`) | `dock.bounce` | falls back to `Bounce::On`, sibling fields keep their parsed values |
   | out-of-range `Percent` (`255`) | `icons.plate_inset_percent` | clamped to `100` at construction (the newtype's constructor, not serde) |
   | negative where `Px` (`u16`) expected | any `Px` field | parse error on that field only -> default |
   | missing table entirely (`[dock]` absent) | whole `DockSettings` | `#[serde(default)]` on the struct produces `DockSettings::default()` |
   | unknown top-level key (`[dock].puppy = true`) | n/a | preserved verbatim on next write (round-trip through a generic `toml::Value` side channel, or `#[serde(flatten)] extra: toml::Table`) |
3. **Watch-fires-on-rename**: a tempdir test writes `settings.toml`, starts `SettingsWatch`,
   then does the same atomic temp+rename the real writer does (not an in-place write), and
   asserts exactly one `ShellFile` is yielded within the 30 ms debounce + a slack margin — same
   pattern PLAN "Design: `<ds>`" already names for `ds-settings`: "round-trip, watch-fires-on-rename
   tempdir, portal mapping tables".
4. **Diff-yields-only-changed-domains**: property test — construct two `Settings` differing in
   exactly one domain (proptest over `SettingsChange`'s variants), assert `apply` returns a
   `Vec` of length 1 containing that variant; construct two identical `Settings`, assert `apply`
   returns empty.
5. **Every key in this doc appears in the Rust schema**: a test that parses this file's section
   3 tables (the dotted key column) and asserts, for each one, that the corresponding
   `Domain::default()` struct has a field of that name reachable by splitting on `.` and
   stripping the domain prefix — catches a key added here and forgotten in code, or a Rust
   field with no doc entry, in either direction. (`gestures.gesture_action_map`,
   `control_center.modules`, `sound.event_*` and the `appearance.*`-owned-but-notifications-read
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
    pub kind: KeyKind,            // Enum{variants}, Bounded{min,max,unit}, Text, Colour, Shortcut, List(Box<KeyKind>)
    pub default: toml::Value,
    pub label: Label, pub help: Help,
    pub page: Page, pub section: Section,
    pub exposure: Exposure,       // Basic | Advanced (file-only)
}
pub struct Schema { pub app: AppId, pub file: FilePath, pub keys: Vec<KeySpec> }
```

Widget by kind, fixed: a two-variant enum is a `Toggle`; three to five variants a
`SegmentedControl`; more a `Menu`; `Bounded` a `Slider` with its unit; `Text` a `TextInput`;
`Colour` the `AppearancePicker`'s swatch row; `Shortcut` a key-capture field; `List` a rows
editor. All from `04-COMPONENTS.md`; the Settings app has no widgets of its own.

### 9.2 Where the schema lives

The derive's build step writes `target/.../<app-id>.settings.toml` and the package installs it
to `$XDG_DATA_DIRS/quire/settings/<app-id>.settings.toml`, next to the `.desktop` file, the way
GSettings ships schemas. The file is TOML, one `[[key]]` table per `KeySpec`, plus `app`,
`file` and `version`. Developers run `cargo run -p <app> -- --write-schema <dir>` for a local
install. A schema without a program (a stale file) is skipped with a warning.

### 9.3 What the Settings app does

1. Discover every `*.settings.toml` under `$XDG_DATA_DIRS/quire/settings/` (and
   `$XDG_DATA_HOME`), parse, group keys by `page` then `section`, render with the widget table
   above; pages come from a fixed `Page` enum (Appearance, Dock, Mouse and Gestures, Keyboard
   and Shortcuts, Notifications, Spaces, Accounts, Apps, plus one page per third-party app
   id). "Advanced" keys render under a disclosure at the end of their section.
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

### 9.5 Rules

- A settings key exists only if a `KeySpec` describes it: a struct field without the derive is
  a bug, caught by the test in 9.6.
- Schema `version` follows the file `version` (section 2); a key removed from a struct stays
  in the schema for one version, marked `deprecated`, so the Settings app can offer to clean it.
- No program renders its own settings UI for keys the schema covers (mailo's appearance
  picker becomes the `AppearancePicker` component bound to `quire/appearance.toml`); an app
  may still embed the Settings app's rendering of its own schema in-app via the shared
  `SettingsPage` component (04, proposed).
- Every "proposed" value in this doc is Basic or Advanced per section 5; nothing is file-only
  without also being in the schema.

### 9.6 Acceptance

- `every_key_in_catalogue_has_a_spec`: the section 3 tables parsed from this file match the
  union of all schemas emitted by the workspace, path for path.
- `schema_round_trip`: derive -> TOML -> parse -> equals the derive output.
- `widget_for_kind_is_total`: every `KeyKind` maps to exactly one component.
- `deep_link_resolves`: "dark mode", "magnification", "natural scrolling" each resolve to one
  key through the schemas alone.

## 7. Open decisions

1. **Whether palmrest and sill share one file.** This doc keeps them separate
   (`palmrest/gestures.toml` vs `sill/settings.toml`) because `palmrest` is a separate daemon
   that must start and apply its own config before `sill` or any compositor exists
   (`12-BEHAVIOUR-gestures.md#12-3-12-daemon-architecture`), and because `12.6`'s own doc text
   already assumes a `palmrest/config.toml`. A merged file would need `palmrest` to depend on
   `sill`'s schema or vice versa, which `CONVENTIONS.md#11-quire-addenda-2026-09-24`'s boundary
   rules (`sill-launcher`/`sill-ipc` must not reach several crates) argue against generalizing.
   Kept separate; revisit only if the two are shown to drift out of sync in practice.
2. **`appearance.accent`'s other five variants are unnamed** (`03-COLOR.md#open-decisions` item
   6) — no doc gives a set of 6 accent names beyond `Postmark`. Cannot default what is not
   named; `ds`'s token table needs this filled in before `AppearancePicker` can render six
   swatches.
3. **`control_center.modules`'s order is plan-derived, not compared against the Claude Doc spec
   (rev 31)** (`13-BEHAVIOUR-menus-windows.md#13-9-open-decisions` item 4) — the default above
   ships but is explicitly provisional.
4. **Whether `notifications.banner_material`'s read crosses from `appearance.toml` into `sill`'s
   process cleanly** — today `ds-settings::use_environment` is the only cross-file read path
   named in the plan (PLAN "Design: `<ds>`"); this doc assumes `sill` calls it for one field of
   `Appearance`, which is untested until `sill`'s `W1`/`M1` waves land.
5. **`icons.plate_glyph_colour_policy`'s `Auto` computation** (per-family WCAG lookup) has no
   named alternative besides forcing white/ink — the docs never propose a third option, so
   `PlateGlyphPolicy` may be over-built as an enum where a single fixed table would do; kept as
   an enum per the "no bool, enums for closed sets" rule, not because a real alternative exists
   yet.
6. **Bar height's macOS alternative range (24-28) is not a single number** — `bar.height_px`'s
   Range/Alt column names a range, not a variant; whichever wins per
   `13-BEHAVIOUR-menus-windows.md#13-9-open-decisions` item 2 and
   `01-LAYOUT.md#open-decisions` item 1 replaces the default, it does not become an enum (unlike
   the other flagged items, this one is a genuine tunable number, not a closed choice).

## 8. Sources

- `README.md#3-citation-convention`, `README.md#4-canonical-source` (the 2026-09-24 decision
  this whole document exists to satisfy), `README.md#5-proposing-a-change`.
- `CHECKLIST.md#14-settings`.
- `CONVENTIONS.md#0-design-style` (pure functions, make illegal states unrepresentable, no
  bool), `CONVENTIONS.md#3-serde` (default, lenient, round-trip, never `deny_unknown_fields`),
  `CONVENTIONS.md#11-quire-addenda-2026-09-24` (no bool in a settings key; "every value the
  design docs mark proposed is read from a settings key... never hard-coded").
- PLAN `~/.claude/plans/vast-toasting-peach.md` "Design: `<ds>` design-system repo" (`ds-settings`
  I/O: `appearance.json` atomic write, `notify` watch, 30 ms debounce, settings portal), "Design:
  `<shell>` repo" (`sill-services`, `dock.json`, `frecency.json`, `Services` context pattern),
  "Cross-repo order", "Verification".
- Every `#N.M Configuration` section this doc draws its base tables from:
  `10-BEHAVIOUR-dock.md#10-6-configuration`, `11-BEHAVIOUR-scroll.md#11-6-configuration`,
  `12-BEHAVIOUR-gestures.md#12-6-configuration`, `13-BEHAVIOUR-menus-windows.md#13-6-configuration`.
- Every doc cited by section, individually, in section 3's Source column.
