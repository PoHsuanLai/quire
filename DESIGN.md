# Design map

Which design doc section each module implements. The docs in `design/` are canonical; this file
only says where in the code a section lives, so a reviewer can go from a paragraph to a file and
back. Paths are under `crates/ds/src/` unless another crate is named first (`ds-core/colour/srgb.rs`
is `crates/ds-core/src/colour/srgb.rs`).

The direction is one quiet Look (`design/30-CATALOGUE.md` section 3, which wins). Rows for the
frame model (`space/frame_vars.rs`, `ground_css.rs`, `root/chrome.rs` `FrameTint`/`Ground`, the
`.ds-frame` group, `CardAccent`, `Ds { tint_alpha }`) describe code that is **target (clean-up
phase)** to delete or reshape; the design docs, not these rows, are the target. Moved sections
keep a "moved to" stub in their old file, so a section number here still resolves.

The design system's crates each name only the ones below them (`scripts/check-boundary.sh`
holds the allowed edges); `ds-settings`, `ds-blitz`, `ds-harness`, `ds-conformance` and
`ds-gallery` sit beside and above them (the last table). The layers: `ds-core` (vocabulary, geometry units, time, errors, text, colour, PNG
and base64, the `Spawner` trait: plain data and maths with no Dioxus), `ds-style` (appearance,
tokens, materials, Space palettes, fonts, icons, the stylesheet's sections, the scope a component
draws in, the tasks a scope owns; the scope, scale, task, busy and `Glyph` parts are its
`dioxus` feature, which `ds-settings` leaves off), `ds-motion` (animation data, timers, machines,
the keyframes, and `ds-motion/detail/`), `ds-lint` (the linter: strings in, offences out), `ds`
(the host seams `host/`, `focus/`, `edit/`, `file_drop/`, `spell/`, `window/`, then `stack/`,
`root/` and `components/`, and `assembly/` on top: the stylesheet's order, the one registration
of every component sheet, and `Ds`), and `ds-shell` (the app-facing parts above `ds`: the account sheets,
the consent alert, the missing-helper sheet and their sheets; the shell surfaces' parts, the widgets
and their catalog and the shell's tokens are sill's `sill-shell-kit`, see ARCHITECTURE.md section 3). Inside `ds` the layers are directories named in
that order. `lib.rs` names `ds`'s public surface: `ds::prelude` and `ds_shell::prelude` hold what a
consumer draws with, `ds::base`, `ds::style` and `ds::motion` re-export the three lower crates,
and every other name is at its home path (the root keeps only the stylesheet assembly).

## `ds`: appearance, space, material

| Module | Implements | Notes |
| --- | --- | --- |
| `ds-style/appearance/theme.rs` | 30 §3 (the one Look), 22-SETTINGS §3.1 `appearance.theme`; 03-COLOR §3 (two schemes) | `Theme` (the choice); `Scheme` is the resolved light/dark |
| `ds-style/appearance/accent.rs` | 03-COLOR §5 and §20; 22-SETTINGS §3.1 `appearance.accent` | the eight macOS accents |
| `ds-style/appearance/motion.rs` | 05-MOTION §3.2; 22-SETTINGS §3.1-3.2 `motion_level` | `Motion` (the preference) and `MotionLevel` (resolved) |
| `ds-style/appearance/appearance.rs` | 04-COMPONENTS §26 (O-16: Theme, Accent, Motion) | lenient read |
| `ds-style/appearance/system.rs` | plan "ds-settings" portal mapping | `SystemPrefs{scheme, motion, contrast}` |
| `ds-style/appearance/resolve.rs` | 05-MOTION §9 rule 11 (explicit `data-motion`); 03-COLOR open decision 13 | `resolve()`, `Resolved::attrs()` |
| `ds-style/appearance/peek.rs` | 04-COMPONENTS §24 `PeekMode` | Center or Full |
| `ds-core/colour/contrast.rs` | 03-COLOR §6 (the gates); `archive/03-COLOR-arc.md` (the WCAG ratio code) | `Verdict` |
| `ds-core/colour/{srgb,oklab,fit}.rs` | 03-COLOR §4, §17.4, §20; 08-ICONS §1.5, §2.10 | `Srgb`, `LinearRgb`, `Oklab`, `Oklch` and their conversions, the one copy every colour computation uses; the prototype's gamut fit (`oklch_bytes`, `oklch_hex`) |
| `ds-style/space/palette.rs` (+`card.rs`, `readout.rs`, `tests.rs`) | 03-COLOR §4, §6; 21-SPACES §2 (the flat tint; target (clean-up phase): the gradient, the field and `card.rs` go) | `Scheme`/`Capping` |
| `ds-style/space/look.rs` | 21-SPACES §1 | `SpaceLook`, `Grain`, `CardAccent` (target (clean-up phase): `SpaceLook` is hue, chroma factor, grain, theme; `CardAccent` and the dots go) |
| `ds-style/space/frame_vars.rs` | 03-COLOR §8 (grain opacity); 21-SPACES §2 | the frame's custom properties and grain opacity (target (clean-up phase): deleted with the frame model; the grain opacity stays) |
| `ds-style/space/list/` (`model`, `ops`, `switch`, `remove`, `recall`, `first_run`, `wire`, `today*`, `time`) and `ds/components/app/spaces/` | 21-SPACES §13 | an app's Spaces as a kit: the pure list, the controller, head, foot, menu and Today (§13.6 is an experiment, not built) |
| `ds-style/space/presets.rs` | 21-SPACES §4 | the eight presets as data (target (clean-up phase): one hue each, grain 0); `default_look` |
| `ds-style/appearance/{material,blur}.rs`, `ds-style/material/recipe.rs`, `ds-style/tokens/tint.rs` | 03-COLOR §17.1-17.3; 21-SPACES §3 (no chrome tint) | `blur_region` is shell-host's (03-COLOR §17.1 "Blur region"); four tint alphas raised for legibility (§17.2) |
| `ds-style/material/{stack,layer}.rs`, `ds-style/tokens/vibrancy.rs` | 03-COLOR §17.4 (material stack v2, the macOS polish pass) | `MaterialStack` (the highlight, hairline, shadow-strength and vibrancy keys, written inline by `Ds { stack }`); each layer written with its alpha read from its input; the vibrancy boost baked into the tint in OKLab |

## `ds`: tokens and stylesheet

| Module | Implements |
| --- | --- |
| `ds-style/tokens/colour.rs` | 03-COLOR §3, §10-12 (paper tokens, washes, `--foreign-ground`); 04-COMPONENTS O-3 (`--danger-ink`, `--mark-ground`, `--handle-ring`) |
| `ds-style/tokens/accent_table.rs` | 03-COLOR §5 and §20 (8 accents x 4 props) |
| `ds-style/tokens/label_hue.rs` | `archive/03-COLOR-arc.md` §15 (Candy hues, history); `archive/07-LOOKS.md` §6.2 (`--c-*`) |
| `ds-style/tokens/hex.rs`, `ds-style/tokens/name.rs` | the value and name types every table uses |
| `ds-style/tokens/{token,set}.rs`, `ds-core-derive` | `Token` (`#[derive(Word, Token)]`: a family's custom properties and values from its variants' attributes), `TokenScope`, `CssValue`, `TokenSet` (a family as data, placed in the `.ds` block) |
| `ds-style/kit/`, `ds-style/kit/style_kit.rs`, `ds-motion/kit.rs`, `assembly/kit.rs`, `ds-shell/kit.rs` | `Kit` (a layer's token families, stylesheet sections, component `Sheet`s each placed after the sheet it follows, and lint vocabulary), `KitRank`, `Kits` (`stylesheet`, `vocabulary`, `token_blocks`, `sheets`); `ds::kits()` is style, motion and the components; `ds_shell::kits()` adds the shell's, `ds_shell::stylesheet()` is the sheet a shell surface draws with (`Ds { sheet: Some(..) }`); `LintConfig::new(&ds_shell::kits())` |
| `ds-style/tokens/timing.rs`, `ds-style/tokens/delay.rs` | 05-MOTION §3.1-3.4, §7.2 (`DurationToken`; `DelayToken`, the timer and delay lengths) |
| `ds-style/tokens/easing.rs` | 05-MOTION §3.1-3.3 |
| `ds-style/tokens/shape.rs` | 01-LAYOUT §10 |
| `ds-style/tokens/row_scale.rs` | 30 §1.6 (the settings row's 44 height and 34 avatar as `--row-settings-h` and `--row-avatar`, emitted on `.ds` with the ladder; `Row` and `SkeletonRow` both read them) |
| `ds-style/tokens/spacing.rs` | 01-LAYOUT §2 (the 18 common steps plus the 1.5, 13 and 15 04-COMPONENTS quotes, as `--s-1`, `--s-1-5` … `--s-36`, emitted on `.ds`; every component sheet reads them) |
| `ds-style/tokens/pixel.rs`, `ds-core/geometry/scale.rs`, `ds-style/scale.rs`, `ds-style/icon/stroke.rs` | 01-LAYOUT §2.1 (pixel snapping): `Scale` in 120ths, `PixelToken` (`--hair`, `--hairline`, `--px`, `--ring`, `--focus-ring`, `--dpr`, tuned tokens the root writes for its scale), `Ds { scale }` / `HostSignals`, a glyph's stroke snapped to an even number of device pixels (08-ICONS §1.4.1); the layout snap itself is `ds_blitz::snap` |
| `ds-style/tokens/elevation.rs` | 03-COLOR §10 (the four shadows; target (clean-up phase): the code still has about twelve), §17.2 |
| `ds-style/tokens/type_scale.rs` | 02-TYPE §2, §4 |
| `ds-style/tokens/layer.rs` | 01-LAYOUT §12 |
| `ds-style/tokens/tuned.rs`, `sill-shell-kit/tokens/{shell_type,dock,osd,notifications,widgets}.rs` | 13-BEHAVIOUR §13.3.1, §13.3.3, §13.3.9 (the shell type scale, `ShellMetrics`); 10-BEHAVIOUR §10.3.1-10.3.2 (`DockMetrics`) | a tuned token (`kind = tuned`) is declared on `.ds` from an input a consumer writes inline, with the settings key's default behind it |
| `sill-shell-kit/tokens/{shell_scale,control_center,widget_paint}.rs` | 29-SIZING §5, §7 (the bar's, menus' and control center's sizes on the ladder); 23-WIDGETS §2 (the widget paints) | the shell's fixed tokens, listed in `sill-shell-kit/kit.rs` |
| `ds-style/css/shape_css.rs`, `ds-style/icon/{plate,family}.rs`, `ds-style/tokens/plate.rs` | 08-ICONS §2.1-2.5, §4.1 | `Corner::Squircle` (a six-layer mask; the shadow on the unmasked box), the plate (`PlateFamily` on `IconView`), the dock floor |
| `ds-style/kit/blocks.rs`, `ds-style/css/{accents_css,materials_css}.rs` | the plan's token model and cascade order; `materials_css.rs` also writes `--m-box` and `--m-frame-alpha` and the root chrome rules (`data-frame=tinted`, `data-chrome=transparent` and its cards; FINDINGS "Bar gaps") |
| `ds-style/css/ground_css.rs` | `archive/03-COLOR-arc.md` §4 (the frame inks; target (clean-up phase): deleted with the frame model); 04-COMPONENTS §19's sidebar item generalised: `data-ground=frame` redirects `--ink*`, `--surface*`, `--raise`, `--line*` to the `--f-*` inks and fills, and `.ds-overlay` under it takes the paper values back |
| `ds-motion/css.rs`, `ds-motion/motion.css` | 05-MOTION §4 (keyframes), §9 rule 2 (`X`/`X--b` aliases) |
| `ds-style/emit.rs` | spike S2: every attribute selector written `[*|attr=value]` |
| `ds-style/css/{reset,utilities}.css`, `ds-style/css/document.rs`, `assembly/stylesheet.rs`, `assembly/sheets.rs`, `ds-shell/sheets.rs` | 02-TYPE §3 (base text on `.ds`; the element rules scope through `:where(.ds)` so a lone component class outranks them); 04-COMPONENTS "Global rules" (`.ds-ic`) and "Truncation" (`.ds-truncate`) |
| `ds-style/fonts.rs` | 02-TYPE §2 (faces as bytes) |

## `ds`: motion, geometry, overlays, root

| Module | Implements |
| --- | --- |
| `ds-motion/anim.rs`, `ds-motion/recipe.rs` | 05-MOTION §4, §5 (the recipe per assignment): one `Anim` variant per keyframe an assignment plays, with the catalogue's recipes; `recipe.rs` holds the table |
| `ds-motion/settle.rs`, `ds-core/time/mod.rs` | 05-MOTION §7.1 (`settle`, `FRAME_SLACK`) |
| `ds-motion/timer.rs` | 05-MOTION §7.2 (timers start in handlers) |
| `ds-style/task.rs` | every task quire spawns belongs to its owner's scope and drops with it (`spawn_in` registers it through the scope's own `spawn`); its writes are `try_set`, so a timer that finds its owner gone stops (FINDINGS "Launcher gaps") |
| `host/{document,no_host,parts,signals}.rs`, `host/{focused,caret,fallback,found,hand_back,ime,pasted,position,probe,captured,drop_hit}.rs` | the document seam: `DocumentHost` and its parts (`FocusHost`, `CaretHost`, `GeometryHost`, `ClickFocusHost`, `EditHost`, `ImeHost`, `FileDropHost`), `NoHost`, `HostSignals`, and the vocabulary they speak |
| `focus/{soon,request}.rs` | 06-INTERACTIONS §17: `focus_soon` (every focus change goes through the host and waits out a busy document) and `FocusRequest`/`use_focus_request` (`FieldFocus::Controlled`, giving a field the keyboard back) |
| `ds-motion/{pulse,pulse_key}.rs` | 05-MOTION §9 rule 2; 04-COMPONENTS vocabulary `PulseKey` |
| `ds-motion/presence/`, `ds-motion/{roster,use_roster}.rs` | 05-MOTION §2 principle 10, §8; 04-COMPONENTS §16 motion states. `Presence::{Hidden, Entering, Present, Leaving(Exit)}` and `use_presence` for a surface its caller shows and hides, `roster::Heal` for a row sliding into a gap, `Exit::{Row, OsdOut, PaneOut, Fade, SheetOut, PanelOut}`; `use_roster` (`LeaveBy`, `RosterSpec`) is the one hook for a list's rows |
| `ds-motion/timeline/` | 26-DETAILS §3.2, §4.1 (Rust-driven values): `Timeline` (`total`, `at`, `settled`) with the implementors `Ease`, `Glide`, `Spring` and `Pending`; `Playback` is the one frame driver (a frame every `FRAME_TICK` while a run moves, the last at exactly its total, none at rest) and `use_timeline` follows a timeline its caller recomputes |
| `ds-motion/hover_intent.rs` | 06-INTERACTIONS §3; 30 §1.4: `HoverProfile {Tip, Card, Label}` |
| `ds-motion/{rubber,use_collapse}.rs` | 30 §1.3-1.4: `rubber::resist`, `use_collapse` |
| `stack/{roving,typeahead}.rs` | 30 §1.4: `Roving`, `Rove`, `Wrap`, `Typeahead` |
| `ds-motion/drag.rs` | 06-INTERACTIONS §6; 04-COMPONENTS §34 |
| `ds-core/geometry/placement.rs` | 01-LAYOUT §8.2; 06-INTERACTIONS §4 |
| `host/measure.rs` | spike S9 (two-phase `use_rect`); every rect read goes through `client_rect`, which uses the host's `GeometryHost::measure` (ds-blitz's waits out a document the renderer holds; `NoHost` answers `Unknown`) (FINDINGS "Bar gaps") |
| `stack/{host,layer_stack}.rs` | 05-MOTION §9 rule 10; 06-INTERACTIONS §5, §18 |
| `stack/hover_hub.rs` | 06-INTERACTIONS §3; 04-COMPONENTS O-11 (`data-hover="warm\|cold"`, which `Ds` stamps on the root) |
| `stack/toast_hub.rs` | 06-INTERACTIONS §9; 04-COMPONENTS §23 (`push_undoable` calls the push's `on_undo` handler) |
| `assembly/ds.rs`, `root/surface.rs`, `ds-style/scope.rs` | `Surface` overrides material and, optionally, scheme, accent, blur and ground; 03-COLOR §17.1 (root attributes: `data-theme`, `data-accent`, `data-motion`, `data-material`, `data-blur`, `data-modality`, `data-hover`; 04-COMPONENTS "Shared vocabulary"); spike S12 (`data-modality`) |
| `root/common.rs`, `root/pass_through.rs` | 30-CATALOGUE R8: the `Common` props (`id`, `data`, `extra_class`, `aria_label`, `mounted`) and the checked `data-*` names and classes a consumer may add |
| `assembly/selectors.rs`, `docs/selectors.md` | 30-CATALOGUE 1.7; ARCHITECTURE section 10: the public selector table a user stylesheet may rely on, and its doc page (a test keeps them equal) |
| `root/chrome.rs` | 21-SPACES §3, §5; 03-COLOR §17.1: `RootChrome::{Painted, Transparent}` (a Popover, Sheet or Toast root hosts cards and paints nothing), `FrameTint::{Opaque, Tinted, None}` (the window's loose layers; the bar, dock, popover panel, OSD and widget's `.ds-frame` group at the tint alpha), `Ground::{Paper, Frame}` (the bar and dock draw on the frame); each derived from the material with an override prop (FINDINGS "Bar gaps") |
| `ds-core/text/clip.rs` | 04-COMPONENTS "Truncation"; 02-TYPE §10 |
| `ds-style/icon/{mod,shape,geometry,render}.rs` | 08-ICONS §1.3-1.5 (stroke as attributes) |
| `ds-style/icon/geometry_shell.rs` | 08-ICONS §1.6 |
| `components/content/voice_orb/`, `ds-style/tokens/orb.rs`, `--orb-*` in `ds-style/tokens/colour.rs` | 30-CATALOGUE §2.9 (`VoiceOrb`, ADD), §3.2 (orb colours per Look): `OrbMetrics::of` (size-derived look), `Turn` and its frame timer (only while `Activity::Active`), `OrbColours` |
| `components/companion/`, `ds-intents/`, `ds-core/vocab.rs` (the companion vocabulary), `ds-shell/confirm/`, `ds-style/tokens/glow.rs`, `ds-style/tokens/timing.rs` (`OrbListen`, `OrbWork`, `OrbAct`) | 30-CATALOGUE §2.13 (proposed), §1.2; 32-COMPANION; 33-AGENT: the companion's presence, prompt, answer cards, plan, replace, runs, activity, memory, served-by chip, the confirmation card and the glow; types frozen, views placeholders, behaviour `todo!()` (FINDINGS "Companion freeze") |
| `components/content/icon_source.rs`, `ds-style/icon/url.rs` | 08-ICONS §1.5 (settled mechanics): `IconSource`, `ExternalIcon`; `IconUrl` (`data:`/`file:` only) |
| `ds-style/icon/classify.rs` | 08-ICONS §1.5 step 2: `classify_with(png, limit) -> Result<IconKind::{Symbolic, Image}>`, OKLCH chroma < 0.04 on every half-covered pixel (`ChromaLimit`) |
| `ds-core/error.rs` | CONVENTIONS §7: `DsError`, the crate's one error enum (a refused icon URL, an unreadable icon PNG) |
| `ds-lint/*` | the coherence rules (ARCHITECTURE.md "Repo rules"); spike S2, S6, S12 rules. 24 `Rule`s: the stylesheet rules (`RawSpacing` and `RawHairline` the Strict-profile spacing and line-width rules), plus `UnstyledClass` and `RawMarkup` for markup; inline custom properties on a `ds`/`ds-*` element and an `<svg>` marked `data-ds-svg` are quire's own, not offences (`ds-lint/inline_style.rs`); `Exception{rule, selector, reason}` in `LintConfig.exceptions`; the vocabulary (variables, keyframes, grammar timing) is `Kits::vocabulary()`, read into `LintConfig::new(&kits)`, and `ds-lint` names no motion crate |

## `ds`: components

`ds-core/vocab.rs` is 30-CATALOGUE 1.5, the shared state vocabulary (`Check`, `Shown`, `Availability`, `Selection`, `Emphasis`, `Muting`, `PressPhase`, `Dismiss`, `Activity`, `FocusStyle`, `InputModality`, `RowState`; with `ds-core/press.rs` and
`ds-core/standard_action.rs`). Components sit in directories by concept: `components/{controls,
fields,forms,menus,menus/palette,overlays,lists,content,chrome}` hold the general ones, `components/app`
mail's own, and sill's `sill-shell-kit/` the shell surfaces' parts (`lock`, `switcher`, `bar`, `control_center`,
`notifications`, `thumbs`, `now_playing`, `month_grid`, `clock`, `battery`, `emoji`,
`user_picture`, `osd`, `idle_dim`, `dock`, `widget`). Every file is the
section of 04-COMPONENTS with the same name, one `.rs` and `.css` pair each (the controls and fields of
30-CATALOGUE 2.1 to 2.3, 2.8 and 2.9 are directories or file groups by concept: `button` (with `button_model`
and `button_marks`; the image-only and toolbar buttons are `Button`, not a second component), `toggle`, `checkbox`,
`radio_group`, `choice` (the option shape), `segmented` (with `segmented_thumb`; the tab strip is a
`SegmentedControl`), `slider` (with `slider_linear`, `slider_bezel`, `slider_machine`, `slider_model`),
`level_indicator` (with `level_draw`, shared with the capsule slider), `progress/` (`ProgressIndicator`, the
arc geometry, the spokes and the busy operation), `badge`, `key_equivalent`, `content/label`, and `fields/text_field`
with its model, parts, mask, area (the multi-line `textarea`) and focus files): `command_pill` §8, `chip` §10, `avatar` §11, `section_header` §13,
`list_row` and `animated_list` §16, `hover_strip` §17, `tooltip` §18, `sidebar_item` §19,
`menu` and `menu_entry` §20 (with `menu_lines`, `menu_keys`, `menu_rows`, `menu_match`,
`menu_tracker` and `menu_panel`: the choices and keyboard as pure tables, the row drawing, the
fuzzy matcher, the `MenuTrack` effects and the panel a menu and its `SubMenu`s share; 13 §13.3.3-13.3.4),
`popover` §21 (with `Arrow` and the `Dismiss` policy; `use_float` places every floating surface), `hover_card` §22 (`Standing`: a card kind, a tip or a label), `tooltip` §18 (`Tooltip` and `Hint`, the one implementation the shell's `DockLabel` shares), `toast` §23 (with `toast_swipe`), `alert` and `alert_model` §55, `sheet` (`Attach`, `SheetWidth`), `side_panel` (design/20 §1.6), `empty_state`, `inline_banner` (`InlineBanner`, in a pane's flow, static), `loadable` (`Loadable` and `Phase`: placeholder, content or Failure by phase) and `skeleton` with `skeleton_row` (design/30 §2.9), `scrim` (the peek's own button only) and
`peek` §24, `command_palette` §25, `appearance_picker` §26, `pin_tile`, `pin_tiles` and `pin_order` (design/30 §2.11; the old `account_tile` §27),
`provider_mark` §28, `palette_lines`, `palette_select` and `palette_rows` (§25's pure lines, the
selection, its own or the caller's, and the selected row's rect; FINDINGS "Launcher gaps"), `link_pill` §29, `icon_view` (08-ICONS §1.5: any icon slot's content),
`press` (`Press{button, modifiers, at}`, `PointerButton`: what `Button` reports, and the keys that activate a control, FINDINGS "Pointer events", "Bar gaps"), `StatusMetrics` (`ds-style/tokens/status.rs`, read by `MenuBarItem`; 13 §13.3.1; FINDINGS "Bar gaps"), `send_pill` §31, `space_editor`
§32, `edge_peek` (the old `edge_strip` §33), `today_tabs`, `space_switch`, `hover_open`, `drag_ghost` §34, `sync_halo` §35, and the macOS polish pass's `menu_bar_item` §36
(13 §13.3.1, the bar's item button, title or glyph), `workspace_pills` §37 and `dock/` §38 (`DockTile`,
`DockLabel`, `RunningDot`, `DockFloor`; 10 §10.3.2).
The P2 components of 30-CATALOGUE 2.1 to 2.7: `fields/stepper/` (`Stepper`, `StepRange`, the hold-repeat machine), `fields/field_row` (`FieldRow`, `FieldGroup`), `fields/fact_list` (`FactList`, a read-only label and value list), `forms/` (design/34 sections 3.5 and 4: `Form` stacks `FormSection`s, a titled `--grp` group with a help footer, and `IconTile` is the 24 px tile or the circular avatar a grouped row leads with; `ListStyle::Grouped` is System Settings' inset list, `Inset` its alias for one release), `lists/table/` (`Table` over `List`, column widths), `lists/virtual_list/` (`VirtualList`, rows of one height or of a height per key windowed over a `Scroller`), `chrome/toolbar/` (`Toolbar`, the overflow rule), `chrome/capsule/` (`Capsule`, the floating pill of controls in the Osd material), `chrome/split_view/` (`SplitView`, its spring pane), `chrome/sidebar` (`Sidebar`), `chrome/tab_view` (`TabView`), `chrome/titlebar_parts` (subtitle, proxy icon, edited dot beside `WindowTitlebar`'s title), `controls/edge_grab` (the drag of a column edge or a divider), `menus/menu_bar` (`MenuBarModel`, data only), `menus/search/` (`SearchField`, a search `TextField` that owns its suggestions panel) and `overlays/drag_ghost` (`DragGhost` with its count badge); `sill-shell-kit/date_picker/` is `DatePicker`, over the shell's `MonthGrid`.
`user_picture` is a directory and 25-EMOJI section 7: `UserPicture`, `PictureSize`, the drawing the
lock and polkit prompts share (`draw.rs`), `PictureChoice`/`resolve_picture` and `UserPicturePicker`.
`emoji` is a directory and 25-EMOJI: `AnimatedEmoji`, `EmojiId`, `EmojiDisc`; the shipped sheets and
manifest (`sheet.rs`) and the task that plays the asset's animation once (`play.rs`).
`lock` holds the password field the lock screen and the polkit sheet share (`password_field.rs`,
`password.rs`); `battery` the ring (`ring.rs`) and the device glyphs; `kept.rs` the book a banner
stack and the app switcher keep their leaving rows in.

`components/app/space_editor` is a directory: `space_editor.rs` (the panel, the field and its handles,
`SpaceDot` in `space_editor/dot.rs`), `space_editor/edit.rs` (the pure edits a gesture makes to a `SpaceLook`),
`space_editor/field.rs` (the hue x chroma colour plane and the tiled round-dot cell over it,
built once per scheme, and the mapping between a dot and its place on it, O-19),
`space_editor/parts.rs` (stops, grain, presets, contrast checks); the field's PNGs are
`ds-core/png.rs`'s, as is the grain tile (`ds-style/css/grain.rs`).

Props worth knowing: `TextField`
`focus: FieldFocus{OnMount, Manual, Controlled}`; `Row` `drop:
DropState{Idle, Target, Source}` (`ds-core/vocab.rs`, §34); `BubbleAction::{Button(BubbleButton),
Separator}`; `AccountFace::One{address}`; `SpaceEditor` `name` and `on_active_dot:
EventHandler<ActiveDot>`; `Ds` `tint_alpha: Option<Alpha>`, fed by
`ds_settings::Environment::tint_alpha`.

Props added in the launcher gaps (FINDINGS "Launcher gaps"): `FieldFocus::Controlled(FocusRequest)`;
`CommandPalette` `host: CommandPaletteHost{Overlay, Surface}`, `id`, `focus`, `selected`, `on_select`, `on_select_rect`,
`onkey`; `Tile::Source(IconSource)`; `IconSize::{Tile48, Tile96, Px(IconPx)}`; `Surface` and
`Ds` `radius: Option<Corner>` (`ds-style/tokens/shape.rs`: `Corner::{Token(Radius), Px(Px)}`); `Tooltip`
`shown: Option<Shown{Visible, Hidden}>`.

## Other crates

| Crate / module | Implements |
| --- | --- |
| `ds-settings/src/{doc,root,store}.rs` | 22-SETTINGS §2 (`SettingsDoc`, `ConfigRoot`, `Store`: TOML and JSON, atomic write) |
| `ds-settings/src/{appearance,units,lenient}` | 22-SETTINGS §3.1-3.3, §4 (`AppearanceFile`, `AppearanceSettings`, `IconsSettings`, the unit newtypes, lenient read and the unknown-key report) |
| `ds-settings/src/{watch,latest}.rs` | 22-SETTINGS §2 "Live reload", §6.3 (`Store::watch` on a `Spawner`) |
| `ds-settings/src/{portal,environment}.rs` | the plan's `ds-settings` design (portal, `use_environment`); `Environment::tint_alpha` feeds `Ds{tint_alpha}` |
| `ds-blitz` | the plan's `ds-blitz` design; spike S7/S8 (`data:` net provider), S11 (font registration), S12 (modality); `blitz_host.rs` is `ds::prelude::DocumentHost` on Blitz, which `launch` and the harness wire and `ds_blitz::provide_host()` gives any other Blitz host whole; `measure.rs` and `focus.rs` are its geometry and focus parts; `texture_layer/` is `TextureLayer`, `use_gpu` and the window's `Gpu` (a GPU texture inside the document; FINDINGS "Texture layer"); features `pdf` (PDF output and `PdfFileThumb`, painted by pdfrum's `pdfrum-anyrender` crate, a git dependency pinned to pdfrum rev 61371040), `print` and `spell`; `tokio` is named by `launch/runtime.rs` alone (plus `tokio::sync` channels in the workers) |
| `ds-harness` | the plan's test driver, `Harness` (`Driver` sends every `Input`, `DocQuery` and `Query` read the document; `render_over(Backdrop::Clear)` paints a document's own coverage), `HarnessConfig`, `HarnessError`, `Clock::Virtual`, `snapshot*`, the painters; it builds its document from `ds_blitz::seam`, what a window's is built from; feature `pdf`: `pdf_app`, `Harness::pdf` |
| `ds-conformance` | `tests/<component>.rs`: every component's behaviour through `ds-harness` on a real Blitz document (`tests/support/probe.rs` is the shared probes and the how-to-write-a-test note); test-only, no code of its own |
| `ds-gallery` | the plan's gallery (axes, `pages/<group>/<component>.rs`, `--snapshot`) |

The shell-only settings of 22-SETTINGS (§3.4-3.14, `ShellFile`, `GesturesFile`, `Settings`,
`SettingsWatch`, `apply`) belong to sill and palmrest, not to this workspace.

## `ds`: the widget interface and placements

| Module | Implements | Notes |
| --- | --- | --- |
| `sill-shell-kit/widget/contract.rs` | 23-WIDGETS §9.2 | `Widget` (one trait per kind), `WidgetKind`, `WidgetContext`, `NoIntent`, `fit` |
| `sill-shell-kit/widget/timeline.rs`, `sill-shell-kit/widget/use_widget.rs` | 23-WIDGETS §9.2 | `Timeline`, `Dated`, `EntryDate`, `Refresh`, `RefreshAsk`, `REFRESH_FLOOR`; `use_widget` sleeps on `ds::base::time::clock` (virtual in tests) |
| `sill-shell-kit/widget/card.rs` | 23-WIDGETS §9.3 | `WidgetCard`: the only way a widget is drawn; the card is `WidgetFrame`'s |
| `sill-shell-kit/widget/registry.rs` | 23-WIDGETS §9.4 | `WidgetRegistry`, `WidgetInfo` (type-erased preview) |
| `sill-shell-kit/widget/wire.rs` | 23-WIDGETS §9.5 | `WireTimeline`: the out-of-process format (the transport is not built) |
| `sill-shell-kit/widget/{battery,clock,calendar}.rs`, `sill-shell-kit/widget/views.css` | 23-WIDGETS §4.1, §4.2, §5.2, §9.6 | quire's three widgets and their compositions |
| `sill-shell-kit/widget/layout.rs` | 23-WIDGETS §9.7 | `WidgetLayout`, `WidgetAt`, `WidgetEdit`, `apply`, `first_free` |
| `sill-shell-kit/widget/gallery.rs`, `sill-shell-kit/widget/gallery.css` | 23-WIDGETS §9.7 | `WidgetGallery` ("Edit Widgets"), `GalleryWords` |
| `sill-shell-kit/catalog/placement.rs` | 23-WIDGETS §9.1 | `Placed`, `Placements`, `PlacementId`: placements as data, generic over kind, size and position |
| `sill-shell-kit/battery/device_glyph.rs`, `sill-shell-kit/battery/device_forms.rs` | 23-WIDGETS §4.4 | `DeviceGlyph`, `Device`: the filled device set as data |
| `sill-shell-kit/widget/slot.rs`, `sill-shell-kit/widget/frame.css` (lift) | 23-WIDGETS §9.8 | `WidgetSlotGuide`; `Lift` on the frame |
