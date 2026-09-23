# Design map

Which design doc section each module implements. The docs in `design/` are canonical; this file
only says where in the code a section lives, so a reviewer can go from a paragraph to a file and
back. Paths are under `crates/ds/src/` unless another crate is named. "Moved" means the code came
from mailo with its tests; everything else is a frozen signature until its wave fills it.

## `ds`: appearance, space, material

| Module | Implements | Notes |
| --- | --- | --- |
| `appearance/theme.rs` | 07-LOOKS §2 (Theme axis); 03-COLOR §3 (two schemes) | `Theme` moved from mailo `view.rs`; `Scheme` is the resolved light/dark |
| `appearance/accent.rs` | 03-COLOR §5, open decision 6; 22-SETTINGS §3.1 `appearance.accent` | Postmark plus the five Candy hues (FINDINGS) |
| `appearance/motion.rs` | 05-MOTION §3.2; 22-SETTINGS §3.1-3.2 `motion_level` | `Motion` (the preference, moved and extended) and `MotionLevel` (resolved) |
| `appearance/look.rs` | 07-LOOKS §2, §7, §11 | `Look`, `Warmth` |
| `appearance/appearance.rs` | 04-COMPONENTS §26 (O-16: Theme, Accent, Motion) | moved from mailo `view.rs`, lenient read kept |
| `appearance/system.rs` | plan "ds-settings" portal mapping | `SystemPrefs{scheme, motion, contrast}` |
| `appearance/resolve.rs` | 05-MOTION §9 rule 11 (explicit `data-motion`); 03-COLOR open decision 13 | `resolve()`, `Resolved::attrs()` |
| `appearance/peek.rs` | 04-COMPONENTS §24 `PeekMode` | moved from mailo `view::Peek`, `Side` left in mailo |
| `space/contrast.rs` | 03-COLOR §4.4 (WCAG ratio), §6 | moved with its tests; `Verdict` |
| `space/palette.rs` (+`card.rs`, `readout.rs`, `tests.rs`) | 03-COLOR §4.2-4.4, §5, §6, §9; 21-SPACES §2 | moved with its tests; `bool` parameters became `Scheme`/`Capping` |
| `space/look.rs` | 03-COLOR §18; 21-SPACES §1 | `SpaceLook`, `Grain`, `CardAccent` |
| `space/frame_vars.rs` | 03-COLOR §4.5, §8 (opacity); 21-SPACES §2 (frame tokens not in palette.rs) | replaces mailo `ui/paint.rs` `push_palette`/`grain_opacity` |
| `space/presets.rs` | 03-COLOR §7; 21-SPACES §4 | the eight presets as data; `default_look` |
| `material/{material,blur,recipe}.rs` | 03-COLOR §17.1-17.3; 21-SPACES §3 | `blur_region` is shell-host's (03-COLOR §17.1 "Blur region"); four tint alphas raised for legibility (§17.2) |

## `ds`: tokens and stylesheet

| Module | Implements |
| --- | --- |
| `tokens/colour.rs` | 03-COLOR §3, §10-12 (paper tokens, washes, `--foreign-ground`); 04-COMPONENTS O-3 (`--danger-ink`, `--mark-ground`, `--handle-ring`) |
| `tokens/accent_table.rs` | 03-COLOR §5 and open decision 6 (6 accents x 4 props) |
| `tokens/label_hue.rs` | 03-COLOR §15; 07-LOOKS §6.2 (`--c-*`) |
| `tokens/hex.rs`, `tokens/name.rs` | the value and name types every table uses |
| `tokens/timing.rs`, `tokens/delay.rs` | 05-MOTION §3.1-3.4, §7.2 (23 `DurationToken`s, `--t-flash` the wave 1 amendment; 13 `DelayToken`s) |
| `tokens/easing.rs`, `tokens/scalar.rs` | 05-MOTION §3.1-3.3 |
| `tokens/shape.rs` | 01-LAYOUT §10 |
| `tokens/elevation.rs` | 03-COLOR §10, §17.2 (`--shadow-pop`, `--shadow-sheet`) |
| `tokens/type_scale.rs` | 02-TYPE §2, §4 |
| `tokens/layer.rs` | 01-LAYOUT §12 |
| `css/tokens_css.rs`, `accents_css.rs`, `materials_css.rs` | the plan's token model and cascade order |
| `css/motion_css.rs`, `css/motion.css` | 05-MOTION §4 (keyframes), §9 rule 2 (`X`/`X--b` aliases) |
| `css/emit.rs` | spike S2: every attribute selector written `[*|attr=value]` |
| `css/reset.css`, `utilities.css`, `stylesheet.rs` | 02-TYPE §3 (base text on `.ds`); 04-COMPONENTS "Global rules" (`.ds-ic`) and "Truncation" (`.ds-truncate`) |
| `fonts.rs`, `build.rs` | 02-TYPE §2 (faces as bytes; `webview-fonts` keeps the base64 path) |

## `ds`: motion, geometry, overlays, root

| Module | Implements |
| --- | --- |
| `motion/anim.rs`, `motion/recipe.rs` | 05-MOTION §4, §5 (the recipe per assignment): 46 variants, the catalogue's 38 keyframes plus `FoldHeavy`, `CrumpleHeavy`, `CurlHeavy`, quire's `ChipFlash` (04-COMPONENTS §10), and four §5 rows that play a catalogue keyframe at their own recipe: `PaletteFade` (row 7), `LinkPillIn` (26), `BubblePop` (37), `PeekFullIn` (64); `recipe.rs` holds the table |
| `motion/settle.rs`, `time.rs` | 05-MOTION §7.1 (`settle`, `FRAME_SLACK`) |
| `motion/timer.rs` | 05-MOTION §7.2 (timers start in handlers) |
| `motion/pulse.rs` | 05-MOTION §9 rule 2; 04-COMPONENTS vocabulary `PulseKey` (`Count` keeps its own bump, §14) |
| `motion/presence.rs`, `roster.rs`, `use_roster.rs` | 05-MOTION §2 principle 10, §8; 04-COMPONENTS §16 motion states. `Exit::{Fold, Curl, Crumple, TabOut}`; an unread (`Emphasis::Strong`) row plays each row exit's heavy variant |
| `motion/hover_intent.rs` | 06-INTERACTIONS §3 |
| `motion/drag.rs` | 06-INTERACTIONS §6; 04-COMPONENTS §34 |
| `geometry/placement.rs` | 01-LAYOUT §8.2; 06-INTERACTIONS §4 (incl. §4.4 `PopoverRequest`) |
| `geometry/measure.rs` | spike S9 (two-phase `use_rect`); every rect read goes through `client_rect`, which uses the host's `HostMeasure` (ds-native's waits out a document the renderer holds; FINDINGS "W2 integration") |
| `overlay/host.rs`, `stack.rs` | 05-MOTION §9 rule 10; 06-INTERACTIONS §5, §18 |
| `overlay/hover_hub.rs` | 06-INTERACTIONS §3; 04-COMPONENTS O-11 (`data-hover="warm\|cold"`, which `Ds` stamps on the root) |
| `overlay/toast_hub.rs` | 06-INTERACTIONS §9; 04-COMPONENTS §23 |
| `root/ds.rs`, `root/surface.rs`, `root/env.rs` | 03-COLOR §17.1 (root attributes: `data-theme`, `data-accent`, `data-motion`, `data-material`, `data-blur`, `data-modality`, `data-hover`; 04-COMPONENTS "Shared vocabulary"); spike S12 (`data-modality`) |
| `text/clip.rs` | 04-COMPONENTS "Truncation"; 02-TYPE §10 |
| `icon/{mod,shape,geometry,render}.rs` | 08-ICONS §1.3-1.5 (moved with tests; stroke as attributes) |
| `icon/geometry_shell.rs` | 08-ICONS §1.6 |
| `lint/*` | ORCHESTRATION coherence rules 1-2; spike S2, S6, S12 rules. 22 `Rule`s: the stylesheet rules, plus `UnstyledClass` and `RawMarkup` for markup; `Exception{rule, selector, reason}` in `LintConfig.exceptions`; the registry is derived from the token and `Anim` tables |

## `ds`: components

`components/vocab.rs` is 04-COMPONENTS "Shared vocabulary". Every other file is the section of
04-COMPONENTS with the same name, one `.rs` and `.css` pair each: `button` §1, `icon_button` §2,
`segmented` §3, `toggle` §4, `slider` §5, `text_input` §6, `search_field` §7, `command_pill` §8,
`kbd` §9, `chip` §10, `avatar` §11, `tabs` §12, `section_header` §13, `count` §14, `spinner` §15,
`list_row` and `animated_list` §16, `hover_strip` §17, `tooltip` §18, `sidebar_item` §19,
`menu` and `menu_entry` §20, `popover` §21, `hover_card` §22, `toast` §23, `scrim`, `sheet` and
`peek` §24, `command_palette` §25, `appearance_picker` §26, `account_tile` §27,
`provider_mark` §28, `link_pill` §29, `selection_bubble` §30, `send_pill` §31, `space_editor`
§32, `edge_strip` §33, `drag_ghost` §34, `sync_halo` §35.

`space_editor` is a directory: `space_editor.rs` (the panel, the field and its handles,
`SpaceDot`), `space_editor/edit.rs` (the pure edits a gesture makes to a `SpaceLook`),
`space_editor/field.rs` (the hue x chroma plane, built once per scheme, and the mapping
between a dot and its place on it, O-19), `space_editor/png.rs` (the PNG encoder it uses) and `space_editor/parts.rs` (stops, grain, presets, contrast checks).

Props added at the wave 2 integration (FINDINGS "W2 integration"): `TextInput` and
`SearchField` `focus: Focus{OnMount, Manual}`; `ListRow` and `SidebarItem` `drop:
DropState{Idle, Target, Source}` (`vocab.rs`, §34); `BubbleAction::{Button(BubbleButton),
Separator}`; `AccountFace::One{address}`; `SpaceEditor` `name` and `on_active_dot:
EventHandler<ActiveDot>`; `SideState::slug`; `Ds` `tint_alpha: Option<Alpha>`, fed by
`ds_settings::Environment::tint_alpha`.

## Other crates

| Crate / module | Implements |
| --- | --- |
| `ds-settings/src/{file,dirs}.rs` | 22-SETTINGS §2 (TOML, atomic write, one-time mailo JSON import); moved from mailo `appearance.rs` with its tests |
| `ds-settings/src/{settings,units,lenient}.rs` | 22-SETTINGS §3.1-3.3, §4 (`AppearanceFile`, `AppearanceSettings`, `IconsSettings`, the unit newtypes, lenient read) |
| `ds-settings/src/watch.rs` | 22-SETTINGS §2 "Live reload", §6.3 |
| `ds-settings/src/{portal,environment,dbus}.rs` | the plan's `ds-settings` design (portal, `use_environment`, `org.quire.Appearance1` stub); `Environment::tint_alpha` feeds `Ds{tint_alpha}` |
| `ds-native` | the plan's `ds-native` design; spike S7/S8 (`data:` net provider), S11 (font registration), S12 (modality); `measure.rs` is the `HostMeasure` the host root and the harness provide |
| `ds-gallery` | the plan's gallery (axes, pages, `--snapshot`) |

The shell-only settings of 22-SETTINGS (§3.4-3.14, `ShellFile`, `GesturesFile`, `Settings`,
`SettingsWatch`, `apply`) belong to sill and palmrest, not to this workspace.
