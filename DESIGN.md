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
| `material/{stack,layer,vibrancy}.rs` | 03-COLOR §17.4 (material stack v2, the macOS polish pass) | `MaterialStack` (the highlight, hairline, shadow-strength and vibrancy keys, written inline by `Ds { stack }`); each layer written with its alpha read from its input; the vibrancy boost baked into the tint in OKLab |

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
| `tokens/spacing.rs` | 01-LAYOUT §2 (the 18 common steps plus the 1.5, 13 and 15 04-COMPONENTS quotes, as `--s-1`, `--s-1-5` … `--s-36`, emitted on `.ds`; every component sheet reads them) |
| `tokens/elevation.rs` | 03-COLOR §10, §17.2 (`--shadow-pop`, `--shadow-sheet`) |
| `tokens/type_scale.rs` | 02-TYPE §2, §4 |
| `tokens/layer.rs` | 01-LAYOUT §12 |
| `tokens/{tuned,shell,dock}.rs` | 13-BEHAVIOUR §13.3.1, §13.3.3, §13.3.9 (the shell type scale, `ShellMetrics`); 10-BEHAVIOUR §10.3.1-10.3.2 (`DockMetrics`) | a tuned token is declared on `.ds` from an input a consumer writes inline, with the settings key's default behind it |
| `css/shape_css.rs`, `icon/{plate,family}.rs` | 08-ICONS §2.1-2.5, §4.1 | `Corner::Squircle` (a six-layer mask; the shadow on the unmasked box), the plate (`PlateFamily` on `IconView`), the dock floor |
| `css/tokens_css.rs`, `accents_css.rs`, `materials_css.rs` | the plan's token model and cascade order; `materials_css.rs` also writes `--m-box` and `--m-frame-alpha` and the root chrome rules (`data-frame=tinted`, `data-chrome=transparent` and its cards; FINDINGS "Bar gaps") |
| `css/ground_css.rs` | 03-COLOR §4 (the frame inks); 04-COMPONENTS §19's sidebar item generalised: `data-ground=frame` redirects `--ink*`, `--surface*`, `--raise`, `--line*` to the `--f-*` inks and fills, and `.ds-overlay` under it takes the paper values back |
| `css/motion_css.rs`, `css/motion.css` | 05-MOTION §4 (keyframes), §9 rule 2 (`X`/`X--b` aliases) |
| `css/emit.rs` | spike S2: every attribute selector written `[*|attr=value]` |
| `css/reset.css`, `utilities.css`, `stylesheet.rs` | 02-TYPE §3 (base text on `.ds`; the element rules scope through `:where(.ds)` so a lone component class outranks them); 04-COMPONENTS "Global rules" (`.ds-ic`) and "Truncation" (`.ds-truncate`) |
| `fonts.rs`, `build.rs` | 02-TYPE §2 (faces as bytes; `webview-fonts` keeps the base64 path) |

## `ds`: motion, geometry, overlays, root

| Module | Implements |
| --- | --- |
| `motion/anim.rs`, `motion/recipe.rs` | 05-MOTION §4, §5 (the recipe per assignment): 47 variants, the catalogue's 38 keyframes plus `FoldHeavy`, `CrumpleHeavy`, `CurlHeavy`, quire's `ChipFlash` (04-COMPONENTS §10) and `MenuOut` (13 §13.3.2's close fade), and four §5 rows that play a catalogue keyframe at their own recipe: `PaletteFade` (row 7), `LinkPillIn` (26), `BubblePop` (37), `PeekFullIn` (64); `recipe.rs` holds the table |
| `motion/settle.rs`, `time.rs` | 05-MOTION §7.1 (`settle`, `FRAME_SLACK`) |
| `motion/timer.rs` | 05-MOTION §7.2 (timers start in handlers) |
| `task.rs` | every task quire spawns belongs to its owner's scope and drops with it (`spawn_in` registers it through the scope's own `spawn`); its writes are `try_set`, so a timer that finds its owner gone stops (FINDINGS "Launcher gaps", sill Q45) |
| `guarded.rs` | the guard around a call or poll into a document the renderer may hold: a panic there is "busy" (`Guarded`, `guarded_call`; FINDINGS "Bar gaps", "Launcher gaps") |
| `focus/{host,request}.rs` | 06-INTERACTIONS §17: `HostFocus`/`Focused` (every focus change waits out a busy document, sill Q43) and `FocusRequest`/`use_focus_request` (`Focus::Controlled`, giving a field the keyboard back, Q44) |
| `motion/pulse.rs` | 05-MOTION §9 rule 2; 04-COMPONENTS vocabulary `PulseKey` (`Count` keeps its own bump, §14) |
| `motion/presence.rs`, `roster.rs`, `use_roster.rs` | 05-MOTION §2 principle 10, §8; 04-COMPONENTS §16 motion states. `Exit::{Fold, Curl, Crumple, TabOut}`; an unread (`Emphasis::Strong`) row plays each row exit's heavy variant |
| `motion/hover_intent.rs` | 06-INTERACTIONS §3 |
| `motion/drag.rs` | 06-INTERACTIONS §6; 04-COMPONENTS §34 |
| `geometry/placement.rs` | 01-LAYOUT §8.2; 06-INTERACTIONS §4 (incl. §4.4 `PopoverRequest`) |
| `geometry/measure.rs` | spike S9 (two-phase `use_rect`); every rect read goes through `client_rect`, which uses the host's `HostMeasure` (ds-native's waits out a document the renderer holds; FINDINGS "W2 integration"); with none, the read's poll is guarded (`guarded.rs`) so a held document is `Busy`, not a panic (FINDINGS "Bar gaps") |
| `overlay/host.rs`, `stack.rs` | 05-MOTION §9 rule 10; 06-INTERACTIONS §5, §18 |
| `overlay/hover_hub.rs` | 06-INTERACTIONS §3; 04-COMPONENTS O-11 (`data-hover="warm\|cold"`, which `Ds` stamps on the root) |
| `overlay/toast_hub.rs` | 06-INTERACTIONS §9; 04-COMPONENTS §23 (`push_undoable` calls the push's `on_undo` handler) |
| `root/ds.rs`, `root/surface.rs`, `root/env.rs` | `Surface` overrides material and, optionally, scheme, accent, blur and ground; 03-COLOR §17.1 (root attributes: `data-theme`, `data-accent`, `data-motion`, `data-material`, `data-blur`, `data-modality`, `data-hover`; 04-COMPONENTS "Shared vocabulary"); spike S12 (`data-modality`) |
| `root/chrome.rs` | 21-SPACES §3, §5; 03-COLOR §17.1: `RootChrome::{Painted, Transparent}` (a Popover, Sheet or Toast root hosts cards and paints nothing), `FrameTint::{Opaque, Tinted, None}` (the window's loose layers; the bar, dock, popover panel, OSD and widget's `.ds-frame` group at the tint alpha), `Ground::{Paper, Frame}` (the bar and dock draw on the frame); each derived from the material with an override prop (FINDINGS "Bar gaps") |
| `text/clip.rs` | 04-COMPONENTS "Truncation"; 02-TYPE §10 |
| `icon/{mod,shape,geometry,render}.rs` | 08-ICONS §1.3-1.5 (moved with tests; stroke as attributes) |
| `icon/geometry_shell.rs` | 08-ICONS §1.6 |
| `icon/external.rs` | 08-ICONS §1.5 (settled mechanics): `IconSource`, `ExternalIcon`, `IconUrl` (`data:`/`file:` only) |
| `icon/classify.rs` | 08-ICONS §1.5 step 2: `classify(png) -> Result<IconKind::{Symbolic, Image}>`, OKLCH chroma < 0.04 on every half-covered pixel (`ChromaLimit`) |
| `error.rs` | CONVENTIONS §5: `DsError`, the crate's one error enum (a refused icon URL, an unreadable icon PNG) |
| `lint/*` | ORCHESTRATION coherence rules 1-2; spike S2, S6, S12 rules. 23 `Rule`s: the stylesheet rules (`RawSpacing` the Strict-profile spacing rule), plus `UnstyledClass` and `RawMarkup` for markup; inline custom properties on a `ds`/`ds-*` element and an `<svg>` marked `data-ds-svg` are quire's own, not offences (`lint/inline_style.rs`); `Exception{rule, selector, reason}` in `LintConfig.exceptions`; the registry is derived from the token and `Anim` tables |

## `ds`: components

`components/vocab.rs` is 04-COMPONENTS "Shared vocabulary". Every other file is the section of
04-COMPONENTS with the same name, one `.rs` and `.css` pair each: `button` §1, `icon_button` §2,
`segmented` §3, `toggle` §4, `slider` §5, `text_input` §6, `search_field` §7, `command_pill` §8,
`kbd` §9, `chip` §10, `avatar` §11, `tabs` §12, `section_header` §13, `count` §14, `spinner` §15,
`list_row` and `animated_list` §16, `hover_strip` §17, `tooltip` §18, `sidebar_item` §19,
`menu` and `menu_entry` §20 (with `menu_lines`, `menu_keys`, `menu_rows`, `menu_match`,
`menu_tracker` and `menu_panel`: the choices and keyboard as pure tables, the row drawing, the
fuzzy matcher, the `MenuTrack` effects and the panel a menu and its `SubMenu`s share; 13 §13.3.3-13.3.4),
`popover` §21, `hover_card` §22, `toast` §23, `scrim`, `sheet` and
`peek` §24, `command_palette` §25, `appearance_picker` §26, `account_tile` §27,
`provider_mark` §28, `palette_lines`, `palette_select` and `palette_rows` (§25's pure lines, the
selection, its own or the caller's, and the selected row's rect; FINDINGS "Launcher gaps"), `link_pill` §29, `icon_view` (08-ICONS §1.5: any icon slot's content),
`press` (`Press{button, modifiers, at}`, `PointerButton`: what `Button` and `IconButton` report, FINDINGS "Tray gaps", "Bar gaps"), `icon_button`'s `Status` variant and `StatusMetrics` (13 §13.3.1; FINDINGS "Bar gaps"), `selection_bubble` §30, `send_pill` §31, `space_editor`
§32, `edge_strip` §33, `drag_ghost` §34, `sync_halo` §35, and the macOS polish pass's `menu_bar_item` §36
(13 §13.3.1), `workspace_pills` §37 and `dock_parts` §38 (`RunningDot`, `DockFloor`; 10 §10.3.2).

`space_editor` is a directory: `space_editor.rs` (the panel, the field and its handles,
`SpaceDot`), `space_editor/edit.rs` (the pure edits a gesture makes to a `SpaceLook`),
`space_editor/field.rs` (the hue x chroma colour plane and the tiled round-dot cell over it,
built once per scheme, and the mapping between a dot and its place on it, O-19),
`space_editor/png.rs` (the RGB/RGBA PNG encoder they use) and `space_editor/parts.rs` (stops, grain, presets, contrast checks).

Props added at the wave 2 integration (FINDINGS "W2 integration"): `TextInput` and
`SearchField` `focus: Focus{OnMount, Manual}`; `ListRow` and `SidebarItem` `drop:
DropState{Idle, Target, Source}` (`vocab.rs`, §34); `BubbleAction::{Button(BubbleButton),
Separator}`; `AccountFace::One{address}`; `SpaceEditor` `name` and `on_active_dot:
EventHandler<ActiveDot>`; `SideState::slug`; `Ds` `tint_alpha: Option<Alpha>`, fed by
`ds_settings::Environment::tint_alpha`.

Props added in the launcher gaps (FINDINGS "Launcher gaps"): `Focus::Controlled(FocusRequest)`;
`CommandPalette` `host: CommandPaletteHost{Overlay, Surface}`, `entrance:
PaletteEntrance{PeekIn, CmdkIn}`, `id`, `focus`, `selected`, `on_select`, `on_select_rect`,
`onkey`; `Tile::Source(IconSource)`; `IconSize::{Tile48, Tile96, Px(IconPx)}`; `Surface` and
`Ds` `radius: Option<Corner>` (`tokens/shape.rs`: `Corner::{Token(Radius), Px(Px)}`); `Tooltip`
`shown: Option<Shown{Visible, Hidden}>`.

## Other crates

| Crate / module | Implements |
| --- | --- |
| `ds-settings/src/{file,dirs}.rs` | 22-SETTINGS §2 (TOML, atomic write, one-time mailo JSON import); moved from mailo `appearance.rs` with its tests |
| `ds-settings/src/{settings,units,lenient}.rs` | 22-SETTINGS §3.1-3.3, §4 (`AppearanceFile`, `AppearanceSettings`, `IconsSettings`, the unit newtypes, lenient read) |
| `ds-settings/src/watch.rs` | 22-SETTINGS §2 "Live reload", §6.3 |
| `ds-settings/src/{portal,environment,dbus}.rs` | the plan's `ds-settings` design (portal, `use_environment`, `org.quire.Appearance1` stub); `Environment::tint_alpha` feeds `Ds{tint_alpha}` |
| `ds-native` | the plan's `ds-native` design; spike S7/S8 (`data:` net provider), S11 (font registration), S12 (modality); `measure.rs` is the `HostMeasure` the host root and the harness provide, exported as `ds_native::measure::{MEASURE, provide}` for any other Blitz host; `focus.rs` is the `HostFocus` beside it (`ds_native::focus::{FOCUS, provide}`); `Harness::is_focused`; `Harness::render_over(Backdrop::Clear)` paints a document's own coverage |
| `ds-gallery` | the plan's gallery (axes, pages, `--snapshot`) |

The shell-only settings of 22-SETTINGS (§3.4-3.14, `ShellFile`, `GesturesFile`, `Settings`,
`SettingsWatch`, `apply`) belong to sill and palmrest, not to this workspace.
