# Release notes

Quire has published its release notes as the tag message. This file keeps them in the tree, newest
first, with what a consumer has to change.

## v0.3.58 (unreleased)

### wh-qpin

- **`docs/workspace-deps.toml` names chordkit, chordkit-kde and chordkit-gnome at 99eec9dd**, the rev quire builds; it said eb0deff1, so a consumer that copied the block got two chordkits. `scripts/check-boundary.sh` now compares every git row of that block with the root manifest, not only Blitz and blitz-kit.
  **Consumer change:** a lock that holds chordkit at eb0deff1 moves its three chordkit rows to 99eec9dd.

## v0.3.57

### wh-bz

- **Blitz moves to the fork's 841386a7** (branch quire-pin-next): 7649ca62 plus 7f8f6ace (a table row's block-level child is laid out in an anonymous cell instead of being dropped as a caption: a newsletter's block-in-table layout keeps its text width) and 5bcb2b4d (dragging a text selection past a scroll container's edge scrolls it). blitz-kit moves to fd08b468 on the same rev; shell-host moves with it. Consumers that mirror quire's blitz block move all its rows to 841386a7.

### Missed in v0.3.55 (wave 1, 26f00b01)

- **`TagColour` and `FileKindColour` are gone.** A tag colour is a `ds::style::tokens::system_hue::SystemHue`: the hue is the token (`SystemHue::Red.var()` is `--tag-red`, the same thirteen names with Gray), so `ChipDot::Tag`, `MenuImage::Swatch` and anything that stored a `TagColour` take a `SystemHue`. `FileKind::colour()` returns the `SystemHue` and `FileKind::hex()` that hue in the light scheme (the file mark's colour).
  **Consumer change:** `TagColour::X` becomes `SystemHue::X`, `tag.hue()` is the hue itself, `use ds::style::tokens::tag_colour::TagColour` becomes `use ds::style::tokens::system_hue::SystemHue`, and `FileKindColour::of(kind).hex()` becomes `kind.hex()`.

## v0.3.56

### wh-ar2

- **`use_resolved` works under any Blitz host.** The readings it registers live in blitz-kit's `after_resolve::AfterResolve`, not in the frame phase's own book: `Phase::after_resolve()` hands out the phase's one, and a ds-blitz window and the headless `Harness` provide it as a root context. A host that does not use the frame phase (shell-host) provides its own `AfterResolve` and runs it after each resolve. No API change for components.
- blitz-kit moves to bb56e04c (adds `after_resolve`; private scroll flags became bools). Consumers that pin blitz-kit move with it, and shell-host must be on a rev that provides `AfterResolve` for `use_resolved` to read under it.

## v0.3.55

### wh-qres2

- **ds-blitz `use_resolved`.** A component builds a value from the tokens its element sees with `use_resolved(|t: &Lookup| ..)`, gives `ResolvedTokens::mounted` to the element's `onmounted`, and reads `ResolvedTokens::value()` in render (`None` until the first frame after mount). The frame phase runs the reader after every resolved frame, outside render, and sets the signal only when the value changed, so a restyle re-renders the component with the new values and nothing waits on a timer. `Tokens` stays for reads in a handler or a task; its docs now say a read during render is always `None` (the document is borrowed then). The headless `Harness` provides the frame phase as a root context, as a window does.

### wh-qw2

Public items that were two-way enums, mirror types or enum-wrapped trait objects become plain Rust (bool, Option, a trait) under the 2026-10-28 type rule. Wire, JSON and TOML shapes, stylesheet class names and settings keys are unchanged.

- **ds-blitz frame events.** `FrameLinks` is a struct of optional handlers: `FrameLinks::intercept(click)` plus `.with_hover(..)`, `.with_context_menu(..)` and `.with_image_click(..)` build it as before, and `FrameLinks::default()` is the inert set (the old `FrameLinks::Inert`; the `with_*` calls still do nothing on it). `FrameHover`, `FrameHoverHandler`, `FrameImages`, `FrameImageHandler`, `FrameMenu`, `FrameMenuHandler` and `FrameLinkHandler` are deleted; one alias `FrameHandler<E> = Arc<dyn Fn(E) + Send + Sync>` stands for them, and `FrameLinks::hover()`, `menu()` and `images()` return `Option<FrameHandler<..>>`. The seam `report_frame_hover`, `report_frame_menu` and `report_frame_image` take `Option<&FrameHandler<..>>`.
  **Consumer change:** `FrameLinks::Inert` becomes `FrameLinks::default()`.
- **`RequestOrigin` is gone.** `NetRequest::origin()` returns `Option<FrameId>`: `None` for the app's own document, `Some(frame)` for a frame.
  **Consumer change:** `RequestOrigin::Top` becomes `None`, `RequestOrigin::Frame(f)` becomes `Some(f)`, and a stored `RequestOrigin` becomes `Option<FrameId>`.
- **`Shown` replaces three copies.** `TrafficLights` (`Shown`/`Hidden`), `NewSpace` and `TilePose` (`Closed`/`Open`) are deleted; `WindowFrame::titlebar(title, lights)`, `WindowTitlebar { lights, pose }` and `SpacesFoot { new_space }` take `ds_core::vocab::Shown`, and their defaults are unchanged (lights and `+` visible, tiling menu closed).
  **Consumer change:** `TrafficLights::Shown` and `NewSpace::Shown` become `Shown::Visible`; `TrafficLights::Hidden`, `NewSpace::Hidden` stay `Shown::Hidden`; `TilePose::Open` becomes `Shown::Visible`.
- **`ButtonFocus` and `ListFocus` are `autofocus: bool`.** `Button { focus: ButtonFocus::OnMount }` is `Button { autofocus: true }`, and `List { focus: ListFocus::OnMount }` is `List { autofocus: true }`. `FieldFocus` is unchanged (it has a third state).
- **ds-helpers installer is a trait.** `Installer` is deleted. `trait Install` (with `BoxFuture`) is implemented by `PackageKit` and `FakeInstaller`; `Helpers::new(catalog, environment, installer: impl Install + 'static)` keeps the backend behind an `Arc<dyn Install>`, so `Helpers` stays one type.
  **Consumer change:** `Installer::PackageKit(PackageKit::system())` becomes `PackageKit::system()`; `Installer::Fake(fake)` becomes `fake`.
- **`ToolPresence` is `bool`.** `Helpers::available` and `Helpers::refresh` return `bool`, and `Availability { capability, present: bool }` replaces `presence: ToolPresence`.
  **Consumer change:** `change.presence != ToolPresence::Present` becomes `!change.present`.
- **`ds_helpers::mime::Installed` is `bool`.** The `installed` check of `Associations::new` and `Associations::load` returns `bool`.
- **ds-harness.** `Harness::has_class` and `Harness::focus_of` return `bool`; `ClassPresence` and `FocusState` are deleted.
  **Consumer change:** `h.focus_of(sel) == FocusState::Focused` becomes `h.focus_of(sel)`, `assert_eq!(h.focus_of(sel), FocusState::Unfocused)` becomes `assert!(!h.focus_of(sel))`, and the same for `has_class` with `ClassPresence`.
- **Window host.** `HostWindow::supports(tile)` returns `bool`; `Support` is deleted. `ds::host::caret::caret_at(text, focus, collapsed: bool)` replaces `Collapsed`. `use_swipe_glue(on: bool, ..)` replaces `SwipeOn`. `SwipeParams.reduced` is `bool` (`Reduced` is deleted).
  **Consumer change:** `fn supports(&self, tile) -> Support` becomes `-> bool` (`Support::Yes` is `true`); `use_swipe_glue(SwipeOn::Yes, ..)` is `use_swipe_glue(true, ..)`.
- **Component props.** `SearchField { ends: Ends::Stop }` is `SearchField { stop_at_ends: true }` (`Ends` is deleted; the default still wraps). `Column { gap }` is `Option<SpacingToken>` (default `Some(SpacingToken::S8)`); `ColumnGap::None` is `None`, a step is `Some(step)`, and `ColumnGap` is deleted. `Slider { ticks }` is `Option<Fraction>` (`Ticks::Every(share)` is `Some(share)`, `Ticks::None` is `None`; `Ticks` is deleted). `PaneContent::Text { mono }` is `bool` (`PaneMono::Monospace` is `true`; `PaneMono` is deleted). `SuggestionsPresent`, `Panel`, `RowHeight` and `AvatarSize` stay: a presentation choice, a who-decides choice, two payload shapes and a stylesheet class per size.
- Private two-way enums that restated a bool are gone (`LabelFit`, `SlotShown`, `CaretMark`, the level glyph's `Showing`, the text field's `Blink`).
- **IconTile glyph scales with the tile.** `.ds-icon-tile > .ds-ic` takes `width` and `height` of `--plate-tile-glyph` (60%, input `--icons-tile-glyph-share`, a new `PlateShare::TileGlyph`), so a 28 px tile holds a 17 px glyph and a 48 px tile a 29 px one. The glyph used to stay at 14 px.
- **`EmptyState { face: Option<TileFace> }`** draws the tile (`tile(face)`) 48 px square above the title, inside the empty state's padding, as System Settings' empty state does. A `face` replaces `icon`; `icon` keeps its faint 48 px glyph for the old case.
- Expected to move: `crates/ds-shell/tests/snapshots/stylesheet.css` (the icon tile and empty state rules, and the `--plate-tile-glyph` declaration), and any consumer's own stylesheet snapshot after a re-pin (sill-shell-kit's).

## v0.3.54

### wh-qstat

- `StatusMetrics` is gone: the bar status item's `--bar-status-box` and `--bar-status-glyph` are stylesheet tokens (sill's sheet declares 22 and 16), so nothing writes them inline. `ds::style::tokens::status::{BOX_VAR, GLYPH_VAR}` still name them for the lint.
  **Consumer change:** drop `StatusMetrics` and its `style_attr()` write; to move the box or glyph, set the property in a stylesheet.
- design/22 drops the rows of the 29 pixel keys sill retired (bar, dock, launcher, scroll bar, notifications, control center and OSD sizes, now stylesheet tokens) and retires §3.16 `osd`.

## v0.3.53

### wh-qres

- `Resolved` reads a token where a component can see it. `Resolved::length(doc, node, name)` and `Resolved::number(doc, node, name)` read the custom property from `node`'s computed style, not the root's: tokens are declared on `.ds` (ds-lint bans `:root`) and inherit, so an element inside a surface sees them with a look's or the person's override. `use_tokens() -> Tokens` gives a component a reader of its own element: pass `Tokens::mounted` to `onmounted`, then `tokens.length("--dock-gap")` / `tokens.number(..)`; before the element mounts, or while the document is borrowed, a read is `None`.
  **Consumer change:** `Resolved::length` and `Resolved::number` take the node after the document.

## v0.3.52

### wh-qtok

- A focus that reaches a sheet after it closed no longer panics.
- `ShellMetrics`, `BarType`, `MenuType`, `LauncherType`, `FontWeight` and `ShellMetrics::style_attr()` are deleted from `ds_style::tokens::shell_type`: the shell type scale's defaults were already the stylesheet's `var(--shell-menu-font, 13px)` fallbacks, so nothing writes them inline. `ShellType` (the token names and inputs) stays, and so does the `--shell-*` input a look pack or the person's own sheet can set, and `ds_style::tokens::tuned::px` is gone with its only caller.
  **Consumer change:** drop the `ShellMetrics::default().style_attr()` write on the bar, popup and launcher roots (sill used only the default); to move a size, set its `--shell-*` input in a stylesheet. `StatusMetrics` and `DockMetrics` still write inline and are unchanged.
- `ds_blitz::Resolved::length(doc, "--dock-gap") -> Option<Px>` and `Resolved::number(doc, "--x") -> Option<f32>` read a token's resolved value from the root element's computed style, after the cascade (looks and the person's sheet included), so Rust geometry follows a stylesheet token. `None` for a missing property or a value that is not a plain `<number>px` or `<number>`. Read again on the frame after the `Styles` signal changes.
- design/22 drops every palmrest key row but `scroll.speed` and the `gestures.gesture_action_map` rows; 3.10 `palm_rejection` is retired (the number stays) and the "Mouse & Gestures" page lists `scroll.natural`, `scroll.speed` and `gestures.gesture_action_map`.
- CONVENTIONS.md: "Proposed values are settings" becomes "Only choices are settings": a tuning number is a named constant beside its code, a visual value a stylesheet token.

## v0.3.51

### wh-qpru

- Settings keep only what macOS offers or a real behaviour choice. Visual knobs are stylesheet tokens, timing and sizes are constants, and the "Mac way or other way" pairs keep the Mac one. `SliderLook` is now `Linear` and `Level` (the Control Center level slider, `data-look=level`); `Capsule` and `CapsuleKnob` are deleted and the media `Capsule`'s volume uses `Level`. The seven `appearance.material_*` keys are the tokens `--m-tint-alpha`, `--m-highlight-light`, `--m-highlight-dark`, `--m-hairline-light`, `--m-hairline-dark`, `--m-shadow-strength` and `--m-vibrancy`; `Ds` no longer writes them inline, so a look or the person's own sheet sets them. `icons` keeps only `style`. `switcher.*` and `hot_corners.dwell_ms`/`rearm_ms`/`size_px` are constants (`SwitcherParams::default()`, `CornerParams::default()`, `CornerParams::HOT_SPOT_PX`). A stored file that still holds a removed key loads as before: the key is reported as unknown and kept in the file. `appearance.typeface` stays for now: quire ships no look pack, so Editorial has no other home yet.
  **Consumer change:** `SliderLook::Mac` is now `SliderLook::Level`, and `SliderLook::Capsule` and `SliderLook::CapsuleKnob` are gone (use `Level`). `Ds` loses the props `tint_alpha` and `stack`, and `ds_style::material::stack::MaterialStack` is deleted. `ds_settings::Environment::tint_alpha()` and `Environment::material_stack()` are deleted. `AppearanceSettings` loses `material_tint_alpha`, `material_highlight_light`, `material_highlight_dark`, `material_hairline_light`, `material_hairline_dark`, `material_shadow_strength` and `material_vibrancy`. `IconsSettings` loses `monochrome_tint`, `plate_inset_percent`, `symbolic_fallback_glyph_percent`, `squircle_detect_iou`, `symbolic_chroma_max`, `plate_glyph_colour_policy` and `dark_mode_variant`, and the types `MonochromeTint`, `PlateGlyphPolicy` and `IconDarkVariant` are deleted (a Monochrome icon takes `Tint::space`; the squircle test 0.90, the Auto glyph colour and the same-in-dark icon are now the plating program's constants). `ds_settings::schema::SwitcherSettings` is deleted, and `HotCornerSettings` loses `dwell_ms`, `rearm_ms`, `size_px` and `params()` (use `CornerParams::default()`). `ds-settings` no longer depends on `ds-behaviour`. The `.ds-slider-icon-lead` CSS class and the `capsule`, `capsule-knob` and `mac` slider looks are gone from the stylesheet.
- design/22 drops the rows of the 96 keys sill removed in the same pass and retires sections 3.11 `menus`, 3.14 `spaces`, 3.17 `power_menu` and 3.18 `window`; the section numbers stay.
- blitz-kit moves to 47b6ceac (one momentum curve and one rubber band: `ScrollMomentumModel`, `ScrollSettings.momentum_model`, `ScrollRubberBand` and `ScrollSettings.rubber_band`/`rubber_band_linear_divisor` are gone; a scroller that must not stretch is `Elastic::Rigid`). Consumers that pin blitz-kit move with it.

## v0.3.50

### form-owner-gone

- Blitz moves to our fork's 7649ca62 (blitz-kit 7da72633): a form control, `<style>` or `<title>` that is added and dropped before the mutator flushes is skipped, where the flush indexed its freed slot and panicked ("invalid SlotMap key used" in `reset_form_owner`; temor hit it when an input appeared and went in one batch). Consumers that build ds-blitz copy the new pinned block; docs/fork-patches.toml is unchanged.

### wh-qset

- Settings explanations go where the Mac puts them. `#[settings(note = "...")]` sets the short line under a row (`KeySpec.note`, which the derive could not set before). `#[settings(range = "...", ends = ("Small", "Large"))]` labels a slider's two ends (`KeySpec.ends: Option<SliderEnds>`). A repeatable struct-level `#[settings(footnote(section = "...", text = "..."))]` adds a section footnote (`Schema.footnotes: Vec<Footnote>`, `[[footnote]]` in the file). All three are absent when unset, so older schemas read unchanged. **Consumer change:** code that builds a `KeySpec` field by field adds `ends: None`, and code that builds a `Schema` adds `footnotes: Vec::new()`. design/22 §9.1 has the grammar; design/27 §5.12 records Settings' colour tiles as the one exception to ink sidebar icons (owner, 2026-10-27).

### wh-qcopy

- Settings wording is shorter and plainer: help lines are one short sentence, and longer explanations move into notes under the row. The Appearance page is split into Colour, Accessibility, Style, Materials and Text and time. File icons move into Icons, and time format into Text and time. Only `help`, `note` and `section` strings change, so no keys, labels or stored values move.

### menu-leave

- A menu's highlight goes out when the pointer leaves the menu, as a macOS menu's does: nothing is lit once the pointer is outside it, a key starts again from an end, and the pointer back in lights the row under it. The parent row of an open submenu stays lit, since it owns the submenu; a pending submenu the pointer leaves never opens; a menu the pointer never entered (opened from the keyboard, or on a pop-up's chosen value) keeps its highlight. Every ds `Menu` and its submenus have it (context, pop-up, pull-down, bar menus, pick lists, search suggestions); a caller's `MenuCursor::Controlled` highlight is the caller's and is not changed. No consumer change.

## v0.3.49

### wh-qframe

- Windows ask the compositor for their shadow and corner clip. On a Wayland compositor that offers our `quire_window_v1`, each window sends `set_frame(compositor_shadow, radius)` once it exists, with the radius of its own drawn edge (`--r-window`, 10), and keeps the object for as long as the window lives. Before this a quire app on our compositor had square corners and no shadow. Other compositors (GNOME, KDE) and X11 are asked nothing and look as they did. The bindings are generated from the vendored `crates/ds-blitz/protocols/quire-window-v1.xml` with `wayland-scanner` (a new optional dependency of ds-blitz, on with `quire-desktop`); no consumer change.

### wh-qstyle

- ds-style builds without warnings when its default feature `icon-classify` is off (casement builds it that way): the colour imports only that feature uses are behind it.

### wh-qpin

- Blitz moves to our fork's 41134ff3 (blitz-kit 80938e37): a text input takes its element's whole font (family, weight, style, features), Return adds one line to a textarea instead of two, an arrow press moves the caret once on macOS, and a table cell is at least its content's width while its row grows to hold it. anyrender moves to 03fcbe4c (a skipped vello_hybrid frame is not carried into the next) and vello to 5c986661 (a test of the bitmap-glyph change). Consumers that build ds-blitz copy the new docs/fork-patches.toml and pinned block.

## v0.3.48

### wh-q48

- `ds_helpers::mime::FileName` builds off Unix: names are checked through `OsStr::as_encoded_bytes`, and off Unix a name splits into stem and extension when it is text (a Windows name that is not valid Unicode keeps no extension). `files::read`'s `NONBLOCK` is Unix-only, so a Windows build has no dead-code warning. anyview's peek-windows build (anyview-fs takes ds-helpers) compiles again.
- blitz-kit moves ec1fd3e2 -> 1b8edae4 (pinned block): the Blitz modules are blitz-kit's default feature `blitz`, so a layer below Blitz takes `default-features = false, features = ["adapter"]` for the adapter ranking and `adapter::opening` with std, wgpu and serde only. Consumers that take blitz-kit with its defaults see no change.

## v0.3.47

### wh-q47a

- `Thumbnail::FileIcon(FileIconChoice)`: a `CollectionView` tile can draw a file's icon (`ds::FileIcon`) whole at the thumbnail's size, uncropped like an icon. `Thumbnail` gains a variant, so an exhaustive `match` on it needs an arm; `FileIconChoice` now derives `Eq` and `Hash`.
- A `Row` with no `common.aria_label` is spoken by its title's plain text, not by its run-together text (avatar letter, title, detail). A menu item is spoken by its title, not by its key glyph and hint.
- `PopUpButton`: Escape heard by the button while its menu is open closes the menu and leaves the keyboard on the button, as on a Mac. The cause: the open menu's own Escape lives on its panel, in the overlay outside the button's element, and the panel takes the keyboard a frame or two after it mounts (and its close plays a 150 ms fade), so an Escape that reached the button first had no handler and the menu stayed open.
- `ProgressStyle::Pie`: a small filled pie that grows clockwise from twelve inside a thin outline circle, for `Mini` and `Small` (the Finder row and sidebar download badge). It tweens like the ring (linear over `--t-move`). `ProgressStyle` gains a variant, so an exhaustive `match` on it needs an arm. New `arc::pie_path`, `PIE_RADIUS`, `PIE_RIM`.
- `ds::Fade { shown, on_hidden }`: wraps any inline content; when `shown` turns `Hidden` it fades the content out over `--t-quick` and then unmounts it (on the short reduced settle under Reduced motion). Built on `use_presence` with `Exit::Fade`. New class `ds-fade`.

### wh-q47b

- New glyphs `Icon::Cloud`, `Icon::CloudDownload` (in the cloud only, click to download) and `Icon::CloudUpload` (waiting to upload), Lucide `cloud`, `cloud-download` and `cloud-upload` (outline by default, each with a solid form) for the files app's cloud-drive states. `Icon` is not `non_exhaustive`: an exhaustive `match` on it gains three arms.

### pins

- blitz-kit moves 07f0272b -> ec1fd3e2 (pinned block): `blitz_kit::adapter::opening()`, one process-wide lock every wgpu opener holds while it makes an instance and requests a device. ds-harness takes it inside its own once-per-process open, so an app's own opener in the same test binary never runs the Vulkan loader's scan beside it (anyview's intermittent SIGSEGV). Consumers move blitz-kit with quire, and their own openers should take the lock too.

## v0.3.46

### wh-e4a

- New `ds_helpers::mime`, moved from the files app: `MimeDb`, `MimeLayer`, `NameGuess`, `Associations`, `MimeApps`, `MimeCache`, `IconName`, `XdgDirs` and the `loader`, plus the small types `MimeType` and `FileName` with `MimeError` and `NameError`. A program learns a file's type, the apps that open it and its icon name without depending on files. `DesktopId` gains the strict `DesktopId::new` and `ds_helpers::DesktopIdError`. See CONSUMING "File types".
- The user-style notes no longer say both "not drawn" and "heavy" of one property: a property Blitz does not draw gets only the `NoEffect` note.

### wh-e4b

- The standard file icon, drawn by quire with no image files: `ds::FileIcon { choice:
  FileIconChoice, size: IconSize, style: FileIconStyle }`. `FileIconChoice` is `Standard { kind:
  FileKind, label: Option<ExtLabel> } | Folder(FolderKind) | Picture(ImageSource)`. A standard icon
  is a white page with a folded corner (0.75 aspect, the fold 22 % of the width), a hairline edge
  and the soft contact shadow; the kind's mark sits on it in the kind's colour and the extension
  is a small bold label near the bottom (for `FileKind::Other` it is the only mark). The label is
  dropped under 48 px (`LabelFit::at`), and the mark grows. A folder is the blue Mac folder in two
  tones, a special folder with its glyph embossed on the front. The page stays light in the dark
  scheme; the folder takes its dark tones. Under `icons.style = monochrome`
  (`FileIconStyle::of(style, tint)`) the folder and the marks take the tint. The resolver that
  picks a choice is the next lane's.
- The data, in `ds_style::icon::file_kind`: `FileKind` (document, pdf, spreadsheet, presentation,
  image, audio, video, archive, code, font, disk_image, program, other; a `Word`, with `mark()`,
  `colour()` and `file_glyph()`), `FolderKind` (plain, home, desktop, documents, downloads,
  pictures, music, movies, trash; `emboss()`) and `ExtLabel::from_extension` (upper case, at most
  four letters or digits, else `None`). New glyphs `Icon::Lines` (Lucide `align-left`) and
  `Icon::Pdf` (the letter P); the other marks are existing glyphs (`Table`, `Monitor`, `Image`,
  `Music`, `Play`, `Archive`, `Code`, `Type`, `Disc`, `Settings`).
- `ds_style::icon::mime` grows from type to glyph into type to kind: `kind_for_mime(&MimeFacts {
  mime, parents, generic_icon }) -> MimeKind { kind, description }`. The lookup is the exact
  type, then the shared-mime-info generic-icon name (`x-office-document`, `package-x-generic`,
  `text-x-script`, ...), then the parents (nearest first), then the major type, then `Other`. The
  generic icon is before the parents on purpose: a `.docx` is a ZIP by parent and a document by
  generic icon, a shell script is plain text by parent and code by generic icon. The caller reads
  the parents and the generic icon from the MIME database; this crate takes plain strings.
  `glyph_for_mime(mime) -> MimeGlyph` keeps working for files and gives the same answers.
- Tokens: `--file-fold`, `--file-edge`, `--file-label`, `--file-folder-tab`, `--file-folder-body`,
  `--file-folder-ink`, `--file-folder-light` (`FileIconColour`, light and dark) and
  `--file-kind-<kind>` (`FileKindColour`, the kind's system hue in the light scheme in both). The
  gallery has a "File icons" page.

**What a consumer changes**

- Nothing breaks. `Icon` has two new variants (`Lines`, `Pdf`): a consumer that matches `Icon`
  exhaustively adds them. `ds::prelude` exports `FileIcon`, `FileIconChoice`, `FileIconStyle`,
  `FileKind`, `FolderKind` and `ExtLabel`. The stylesheet golden moves.

### wh-e4c

- File icons resolve: `ds_extensions::FileIcons::build(db, &found, &settings.appearance.file_icons)`
  makes a cheap-to-clone resolver (rebuild it when the existing watch fires) and
  `FileIcons::of(&FileFacts { mime, name, folder: FolderFacts })` answers `IconPick::Standard { kind,
  label } | Folder(FolderKind) | Artwork(path)`. Top wins: Your changes, then the chosen pack,
  then the standard set (`kind_for_mime` and the extension label). A pack is an Extension with a
  `file_icons` entry whose `file-icons/` folder is keyed by shared-mime-info names
  (`application-pdf.svg`, `x-office-document.svg`, `image.svg`, `other.svg`, `folder.svg`,
  `folder-downloads.svg`); `.svg` and `.png`, each up to 1 MiB; what a pack lacks falls through.
  The names are tried as the standard kinds are: the type's icon name, its generic icon, its
  parents, its major type, then `other`. The generic icon is before the parents on purpose (a
  `.docx` has the parent `application/zip`); this differs from the design's wording, which put
  the parents first. `FileIcons::standard(db)` is a resolver with only the standard set.
  `Built` also carries `stale` (a stored pack that is gone or off reads as standard; report it once)
  and `off` (the turned-off Extension, so a host can offer Turn On…).
- `ds-extensions` now depends on `ds-style` (its `FileKind`, `FolderKind` and `ExtLabel`), so the
  resolver answers in its own `IconPick`; an app maps it to `ds::FileIconChoice` in three arms
  (CONSUMING "File icons"), since neither crate may name the other. The entry type of the
  `file_icons` word, which was `ds_extensions::FileIcons`, is now `IconFolder`
  (`Provision::file_icons()` returns it); `FileIcons` is the resolver.
- New `FileIconChoice::Artwork(ImageSource)`: a finished icon from a pack, drawn whole in the square
  (`data-choice=artwork`, `img.ds-file-artwork`), where `Picture` crops a thumbnail to a page frame.
- New setting `appearance.file_icons` ("File icons", Appearance page, section "Files", a `Provided`
  key of the `file_icons` capability, default `standard`, agent-settable). Settings fills its choices
  with `choices_for(Capability::FileIcons, ..)`, "Standard" first. The agent-settable keys are now
  `appearance.theme`, `accent`, `look`, `file_icons` and `custom_styles`.
- Your changes: `ds_extensions::YourChanges::in_root(&data_root)` is the person Extension
  `user.YourChanges` ("Your changes"). `set(&icon_name, &picture)` copies a picked `.svg` or `.png`
  into its `file-icons/` under the type's icon name and writes its `extension.toml`; `clear`,
  `changed` and `removal() -> Removal::{Allowed, Blocked}` (blocked while it holds a change). Writes
  are atomic; a picture must be a PNG or an SVG with no script, foreign content or link. It is
  never offered as a choice.
- New glyph `Icon::Puzzle` (Lucide `puzzle`, outline and solid) for the Extensions page.

**What a consumer changes**

- A consumer that named `ds_extensions::FileIcons` as the entry type uses `IconFolder`. `Icon` and
  `FileIconChoice` have a new variant each (`Puzzle`, `Artwork`): a consumer that matches them
  exhaustively adds the arm. The stylesheet golden moves (`.ds-file-artwork`).

### shell ask

- Accounts forms gain `FieldRole::ServerUrl` ("Server address", hint `https://server.example.org/v1`) for porter's `FieldKind::ServerUrl` (porter e91eb7a4). An exhaustive `match` on `FieldRole` needs one more arm.

### anyview ask

- ds-style's PNG decoding for status icons (`icon::classify::classify_with`) is behind a new feature `icon-classify`, on by default and enabled by `ds`. ds-settings, ds-extensions, ds-lint and ds-motion take ds-style without it, so a program that only reads settings or Extensions no longer links the `image` decoder. A crate that names `ds-style` directly and calls `classify_with` keeps the default or names the feature.

### pins

- bayonet moves 0f5897c3 -> d66e243: an Extension entry may name a program or library outside its folder (`Provides::reach`), and an Extension's `version` may carry a semver pre-release (`0.1.0-beta.1`, ranked below its release). Pin bayonet d66e243 beside quire v0.3.46.
- tokio 1.53.3 (RUSTSEC-2026-0386).

## v0.3.45

### wh-e1a

- The cascade gains the look layer: `@layer ds, app, look;` (`ds_style::css::layers::ORDER`, with
  `LOOK` and `layers::look(css)`). `Ds { user_style }` becomes `Ds { styles: ReadSignal<Styles>,
  custom_styles: CustomStyles }`: `ds_style::kit::Styles { look, desktop, app, homes }` carries the
  chosen look (`@layer look`), the person's desktop-wide `~/.config/quire/style.css` and their
  `~/.config/<app>/style.css` (unlayered, desktop-wide first). Each non-blank sheet is its own
  `<style data-ds-style=look|desktop|app>` (was `data-ds-user`). The look is filled by the app as
  a string; `Ds` does not find look Extensions. The desktop-wide file is `UserStyle` under
  `AppName::QUIRE`; a program watches both files with `Store::watch`. A consumer that passed
  `user_style` passes `styles: Styles { app: ..., ..Default::default() }`. The stylesheet golden
  moves (the layer statement).
- New key `appearance.custom_styles` ("Use my styles", `use` | `off`, a switch on the Appearance
  page, default `use`, agent-settable): `CustomStyles::Off` makes `Ds` draw none of the three
  sheets. It is in `AGENT_SETTABLE_PROPOSED` and in design/22 sections 3.1 and 9.7.
- Crash guard: `ds_blitz::StylesGuard::begin(state_dir, program, &styles)` records the start in
  `<state>/quire/styles-guard/<program>`, `use_styles_guard(guard)` marks it good after the first
  frame plus five seconds, and two unfinished starts with the same sheets give `Load::Without`
  (the pure `styles_guard::decide`). `ds::prelude::StylesOffNotice { program, onturnon }` is the
  one-line notice with **Turn On Again** (`StylesGuard::turn_on_again`); `changed` clears the guard
  when a sheet changes. The state directory is a parameter.
- What a sheet may fetch: `url()`, `@import` and `@font-face` in the three sheets may name `data:`
  or a file inside the sheet's own folder (`StyleHomes`). Blitz's net provider cannot tell which
  sheet asked, so `ds_style::kit::reach::within` rewrites the text before it goes into `<style>`
  and `Styles::dropped()` lists the cuts for the notes.
- ARCHITECTURE section 10 and CONSUMING section 12 describe the three sheets, the guard and the
  reach rule.

### wh-e1b

- `ds::selectors::SURFACE` (1) versions the public styling surface: it goes up when a public name
  is renamed or removed, not when one is added. A renamed part carries both names for one release
  (`ds::selectors::RENAMES`, `PartRename`); an app lists its own `<app>-` parts in an
  `AppSelectors` table, and `ds_lint::user_stylesheet_with(css, kits, tables)` accepts them
  (`user_stylesheet` is unchanged). CONVENTIONS section 16 has the recipes.
- Cost notes in `ds_lint::user_stylesheet`: `HeavyBlur`, `LargeShadow`, `EndlessAnimation`,
  `AnimatesEverything` and `NoEffect`, with `UserStyleNoteKind::plain_words()` for the text
  Settings shows (every kind has one). Limits are in `ds_lint::limits`. Report only.
- The gallery has a "Styling" page: every public part on a live example, generated from
  `ds::selectors`, with the surface number.

### wh-e2b

- New crate `ds-extensions`: Extensions as a library (no UI, no runtime, no bus). `Search::new(person,
  system)` / `Search::from_env()` scan `<data>/quire/extensions/<id>/extension.toml` and return every
  copy of every id unranked, with the turned-off ids and their names (`Found`, `Off`), so a host can
  say "off" instead of "missing". `Found::resolve(Capability)` checks (paths stay in the folder, a
  look's stylesheet at most 256 KiB) and ranks (usable first, protocol, the person's copy over the
  system's, newer version, id) through bayonet; an Extension that fails is a `Problem` with a line in
  plain words and is never half used. `Capability` is `file_icons` and `look`; `Provision` claims only
  those words, so another owner (anyview-plugin) reparses the raw candidates itself.
- `quire/extensions.toml` (`ExtensionsFile`, `extensions_schema()`): a schema'd file on the new
  Extensions page (after Appearance) with one Rows key, `extensions.off`; not agent-settable.
- `KeyKind::Provided { capability }` and `ds_settings::schema::Provided`: a key whose value is
  `standard` or an Extension id, with `#[settings(provided = "look")]`. The caller fills the choices
  (`ds_extensions::choices_for`, `Choices::fill`, `KeySpec::offering`) and the `menu` control draws it;
  a stored id that is not installed or is off reads as `standard` (`ds_extensions::chosen`) and is
  returned once as `Stale`. `appearance.look` is the first such key, agent-settable.
- `look::stylesheet_for(choice, registry)` reads the chosen look's stylesheet and folder; wiring it
  into `Ds`'s style props waits for the user-styles lane.
- `SettingsTarget::Extension(ExtensionId)`, written `extension:<id>`.
- design/22 section 9.1 (kind table, Provided) and 9.8 (item 6, the Extensions page, the target).
- bayonet at 0f5897c3 is a quire dependency (outside the pinned block).

**What a consumer changes**
- `KeyKind` gains `Provided { capability }` and `SettingsTarget` gains `Extension`: an exhaustive
  match needs an arm each. A reader built before `Provided` refuses a quire schema file that has it
  (an unknown kind makes the file unreadable and `discover` skips it with a warning): rebuild
  sill and detent with this quire.
- `AppearanceSettings` gains `look` (a `Provided`).
- `ds_settings::xdg::search_dirs` is public.

### Whopper (merge)

- Published style parts as data: an app writes its `AppSelectors::classes()` with
  `ds_settings::PublishedParts::new(app, classes).write_to(<prefix>/share/quire/selectors)`, and
  Settings reads every app's with `discover_parts(&parts_dirs())`, then passes each `classes` to
  `ds_lint::user_stylesheet_with`, so a person's rule on `sill-*` is not reported as internal
  without linking sill. A class outside the app's prefix is dropped on write and read.
- A program may ship a second settings schema for a second file it owns,
  `<app>.<part>.settings.toml` (`Schema::write_part_to`); `discover` admits one per app and file.
  quire's `extensions.toml` schema is written by `cargo run -p ds-extensions --example
  write_schema -- --write-schema <dir>`; installers run it beside ds-settings' own.
- Design docs describe the dock's pins as built (`[[dock]]` in `sill/layout.toml`, sill 848a20a7).

## v0.3.44

Reorderable lists, a New Space that is named before it exists, and the CSS ellipsis allowed.
chordkit, Blitz and blitz-kit stay where v0.3.41 put them.

**What a consumer changes**
- `Icon` gains `Grip`: an exhaustive match needs an arm.
- The stylesheet golden moves (list reorder rules: `.ds-list[data-reorder=grip]`, `.ds-list-grip`,
  `.ds-list-item[data-drag]`).
- `--drag-y` is set inline on a dragged row: a test that checks every var() is declared lists it
  with the other per-element vars (sill b5ac2cc8).
- An app's lint exception for `text-overflow` can go.

### List (quire#59, mailo)

- `List { onreorder }`: each row leads with a grip (`Icon::Grip`, Lucide grip-vertical). Dragged by
  it, the row lifts past 3 px and follows the pointer while the rows it passes slide out of its
  way; the new order goes to the caller. The order arithmetic is shared with the pinned tiles
  (`lists::order`, crate-private).

### Spaces (quire#60, mailo)

- `SpacesHandle::begin_new` opens the name field on a pending Space: Return with a name adds it
  and switches to it; Escape or a click outside drops it, so nothing is added until it is named.

### Lint

- `Rule::BlitzUnsupported` no longer flags `text-overflow`: the Blitz fork quire pins draws the
  ellipsis since e7f9f877 (review fixes 851fa7dd), on a nowrap line that overflows a clipping
  block, left to right only. An app's lint exception for it can go. `.ds-truncate` keeps its fade;
  `ds-conformance` `text_overflow_ellipsis` reads the cut from the laid-out document, and the
  gallery's S13 section shows the CSS ellipsis beside the fade and the Rust clip.

## v0.3.43

anyview's Preview-style controls, as general pieces. A window titlebar takes a leading control
after the traffic lights (the sidebar button); eleven markup and view glyphs (Hand, Crop,
MousePointer, Highlighter, Type, Circle, Line, ArrowUpRight, PenLine, ListTree, Table); a Sidebar
section that fills the height the sections above it leave, so a virtualised list sits under view
rows; and F1 to F12 as shortcut keys, so F9 is the CUA Toggle Sidebar in tests and menus.
chordkit, Blitz and blitz-kit stay where v0.3.41 put them.

**What a consumer changes**
- `WindowFrame::Titlebar` gains `leading`: a struct-pattern match needs `..` or the field.
- `Icon`, `SidebarSection` and `ShortcutKey` gain variants: an exhaustive match needs arms.
- The stylesheet golden moves (`.ds-titlebar-leading`, `.ds-sidebar-body-split`,
  `.ds-sidebar-section-fill`).

### Lane A

- `WindowTitlebar` takes `leading: Option<Element>`, drawn after the traffic lights and before the
  title (where Preview and Finder put the sidebar button), and `WindowFrame::with_leading` sets it
  on a framed root. It mirrors `trailing`: a press or double-click on it is its own and neither
  moves nor zooms the window. With `TitleRoom::Centred` the title stays centred in the bar; with
  `TitleRoom::Beside` it is centred in the room between the leading and trailing controls.
  `WindowFrame::Titlebar` gains a `leading` field (a struct-pattern match needs `..` or the field).
  New class: `.ds-titlebar-leading`.
- `ds_style::Icon` gains `Hand`, `Crop`, `MousePointer`, `Highlighter`, `Type`, `Circle`, `Line`,
  `ArrowUpRight`, `PenLine`, `ListTree` and `Table` (Lucide outlines, solid forms from Tabler and
  Phosphor, all native outline). They join `Icon::ACTIONS` and `Icon::ALL`; an exhaustive match
  over `Icon` needs arms.

### Lane B

- `SidebarSection` gains `Fill(Element)`: a section that takes the height the sections above it
  leave and gives its content a definite height (a virtualised list under view rows, as in
  Preview). The body stops scrolling as a whole while a Fill is present; without one nothing
  changes. A sidebar holds one Fill: a second fails a debug assertion and is left out. An
  exhaustive match over `SidebarSection` needs an arm. New classes: `.ds-sidebar-body-split`,
  `.ds-sidebar-section-fill`.
- `ShortcutKey` gains `F(u8)`, `F1` to `F12` (`ShortcutKey::function(n)` checks the range; other
  numbers draw as `F<n>` but resolve to nothing). It has a glyph (`F9`), a chord name (`f9`), a
  dbusmenu name and parse, chordkit's `F9` in `default_chord`/`from_default_chord`, and the
  harness sends it as the DOM F-key, so `Input::key(ShortcutKey::F(9))` is chordkit's CUA
  `ToggleSidebar`. An exhaustive match over `ShortcutKey` needs an arm.

## v0.3.42

The files batch, fourth part. Finder's path bar (`PathControl`, segments that take drops, a long
path folds its middle into a menu) and a toolbar title that opens a menu of enclosing folders on the
primary click; Finder's column view (`ColumnBrowser`); document tabs (`DocumentTabs`: close, new,
drag to reorder, no tab limit); VirtualTable rows that drag onto folders or between rows with the
operation their modifiers chose, and a divider double click that fits a column to its content; a
menu row of tag dots (`MenuItem::Swatches`); and a file clipboard in ds-blitz, so ⌘C on files
reaches other apps. ds-settings declares the hidden `accounts-setup` page. chordkit, Blitz and
blitz-kit stay where v0.3.41 put them.

**What a consumer changes**
- `MenuItem` gains `Swatches(SwatchRow<T>)`: an exhaustive match over `MenuItem` needs an arm.
- `Clipboard` gains `read_files` and `write_files`, with defaults, so an implementer still compiles.
- `Toolbar` gains `title_menu` and `ontitlemenu`; `VirtualTable` gains `onrowdrop`, `drop_kind`,
  `drop_gaps` and `cell_width`; all default off.
- The stylesheet golden moves (new sheets `path_control`, `column_browser`, `document_tabs`, new
  classes for the row drop mark and the menu's tag dots; SHEETS is 107).
- A cut of files is the app's own state: the system clipboard carries the files as a copy, and the
  app turns a paste of its cut set into a move (`ClipboardFiles::same_files`).

### Lane E

- A file clipboard: `ds_blitz::clipboard::{write_files, read_files, ClipboardFiles, FileOp}` and
  `Clipboard::{read_files, write_files}`. Files go to the system clipboard as `text/uri-list`, so
  ⌘C in an app reaches other apps; reading accepts `text/uri-list`, `x-special/gnome-copied-files`
  (with its `cut`) and plain paths. A cut is the app's own state (`same_files` turns a paste of
  the cut set into a move). The harness's memory clipboard holds both file formats.

### Lane B

- `ColumnBrowser`: Finder's column view. An app lists the `columns` (rows of a glyph, a name cut
  from the middle and a chevron on a folder, with each column's `RowSelection`) and answers each
  `onpick` by cutting the columns after the picked one and supplying the next; Up, Down, Home,
  End, Right (enter), Left (back) and type-to-select move the keys, Command-Down and a
  double-click ask `onopen`, a selected leaf shows the `preview` slot, and a divider drag resizes
  the columns. New classes: `ds-colbrowser`, `-strip`, `-column`, `-item`, `-divider`, `-preview`.

### Lane D
- `VirtualTable` rows drag (E5): give `onrowdrop` and a press on a row, or on a selected row to carry the whole selection, drags it. `drop_kind` says which rows are folders (`RowDropKind::Container`), which are outlined under the pointer; `drop_gaps: RowGaps::Offered` also draws a line in the nearest gap. The drag lives in the table, so rows that are not mounted are places to drop on, and the list scrolls near its edges. `onrowdrop` hears a `RowDrop` (the dragged row indices, a `RowDropTarget` and a `DropOperation`; Option copies).
- Double-clicking a column divider in `VirtualTable` fits the column to its content (B4) when `cell_width` gives each cell's content width; only the mounted rows and the header are measured.
- `MenuItem::Swatches(SwatchRow)` (H4) is Finder's row of tag dots in one menu row (`MenuItem::swatches(title, vec![MenuSwatch::new(value, name, colour)])`): each dot is its own pick target with a tooltip and an `aria-label`, Up and Down pass the row as one stop and Left and Right walk the dots. **A consumer that matches `MenuItem` exhaustively adds an arm.**

### Lane C

- `DocumentTabs` (chrome): Finder's and Safari's tab strip. A full-width strip under the toolbar, the
  tabs sharing the width, a close button on the selected tab and under the pointer, a tooltip with
  the full title, a plus at the end, tabs dragged to reorder, no tab limit and no framed body. It
  hears `onselect(index)`, `onclose(index)`, `onnew` and `onreorder((from, to))`; the arrows, Home
  and End select a tab and the standard New and Close actions call `onnew` and `onclose`
  (chordkit has no next or previous tab action). The app draws it only when
  `document_tabs_model::strip_shown(count)` (two or more tabs). `TabView` stays `NSTabView`.

### Lane A

- `PathControl`: Finder's path bar. An app gives `PathSegment`s (name, glyph, drop state) and hears a click on a segment, or on one in the collapsed middle of a long path, by index; a segment it marks `Accepts` or `Target` takes a drop (`ondropenter`, `ondropleave`, `ondrop`), driven like a `Row`'s. New classes: `ds-path-control`, `ds-path-segment`, `ds-path-name`, `ds-path-chevron`, `ds-path-more`.
- `Toolbar` takes `title_menu` (a list of `PathSegment`s) and `ontitlemenu`: a click on the title with the primary modifier held opens a menu of the enclosing folders.
- `detent --page accounts-setup` has a page: the hidden built-in `accounts-setup`, "Sign in to Your Accounts", resolves by id and by `accounts_setup`.

## v0.3.41

The files batch, third part, and hot corners beyond the Mac. A hot corner holds several bindings
keyed by held modifiers, each a desktop action, an app's action, something to open or a command
(older files still load). CollectionView takes landscape thumbnails, can size itself to its content
and draws Finder's rubber band, as VirtualTable does; VirtualTable's first column can be an outline
with a Name cell; a file drop says which operation its modifiers chose. Also a `Picture`, a list
date formatter, a toolbar title beside its centre item, key tooltips on toolbar items, stem
selection on rename, tag dots on chips and in menus, `--tag-gray`, sheet button rows and a large
`ProviderMark`. chordkit moves to 99eec9d: Move to Trash (⌘⌫), New Folder (⇧⌘N) and Empty Trash
(⇧⌘⌫) are standard actions an app can offer. Blitz and blitz-kit stay where v0.3.40 put them.

**What a consumer changes**
- Move every chordkit pin to 99eec9d together with quire (one chordkit in the lock); relock. An
  app that binds ⌘⌫ or ⇧⌘N (or ⌥⌘N, which a terminal reads as ⇧⌘N's form) to its own action now
  gets a standard-action conflict: adopt the standard action or forgo it.
- `Chip.dot` is `Option<ChipDot>` (`ChipDot::Tag(TagColour)` or `ChipDot::Colour(String)`).
- `InitialCaret` and `Select` gain `SelectStem` and `Stem`; `SystemHue::ALL` and `TagColour` gain Gray.
- `FileDragInput::Dropped` carries `modifiers`; `FileDrop` has `modifiers` and `operation()`.
- `HotCornerSettings`' corners are `CornerBindings`; `CornerAction` loses `None` and `Command`.
  A program editing the settings file as TOML runs `ds_settings::schema::fold_older_commands`.
- The stylesheet golden moves (new classes for the toolbar, sheet actions, chip dots, pictures,
  menu swatches, the marquee, the outline and the Name cell).

### Hot corners
- A hot corner is now a list of bindings, each holding exactly a set of modifier keys
  (`CornerKeys`, written as a list of `command`, `option`, `control`, `shift`) and what it does
  (`CornerDo`: a desktop action, an app's action, open an app, file, folder or web address, or a
  command). New in `ds_settings::schema`: `CornerBinding`, `CornerBindings`, `CornerDo`,
  `CornerKey`, `CornerKeys`, `OpenItem`, `Control::Corners`. `CornerAction` gains `AppWindows`,
  `DisplaySleep` and `Assistant` and loses `None` (an empty list) and `Command` (`CornerDo::Command`).
  `HotCornerSettings::{top_left, top_right, bottom_left, bottom_right}` are `CornerBindings` and the
  `*_command` fields are gone. Older files (`top_left = "lock"`, `top_left_command`) are read and
  never written back; a bad or repeated binding is dropped and reported, the rest kept. The four
  corner rows are Basic and carry `control = "corners"`, for one custom control.
- The lenient reader reports a list whose entries were dropped on reading, and accepts a key the
  type no longer writes when it changed what the type means (an older spelling).

### Small pieces

- `ds_settings::list_date(modified, now, offset, hours)` words a file's modified time for a list
  cell as Finder does: "Today at 14:05" (or "2:05 PM" on a 12-hour clock, `ResolvedHours`),
  "Yesterday", otherwise a short date with its year ("Oct 5, 2026"). New `UnixSecs` and
  `UtcOffset`. Month names are English until the desktop has a locale seam.
- `Toolbar { title, center }` now shows both: the title after the leading items
  (`data-with-center` on `.ds-toolbar-title`) and the centre item centred in the room left
  (`.ds-toolbar-center`). A centre item with no title takes the title's place as before.
- A toolbar button's tooltip carries its item's shortcut (`Back  ⌘[`), and the overflow menu
  row shows it as its key.
- `FocusRequest::with_select_stem()` and `Select::Stem` / `InitialCaret::SelectStem`: a rename
  field opened on "photo.png" selects "photo" (up to the last dot; ".bashrc" is all stem).
  `ds::host::caret::stem_end` is the rule. A host that cannot select a range selects the whole name.
- `Chip { dot: Option<ChipDot> }`: `ChipDot::Tag(TagColour)` paints from `--tag-<hue>`,
  `ChipDot::Colour(css)` is the old string. Breaking: `dot` took a `String`. `Chip { label }` names
  a chip with a dot and no text (default: the tag colour's word, "Red"); it writes `aria-label`
  (and `role=img` on a static chip).
- `--tag-gray`: `TagColour::Gray` and `SystemHue::Gray` (systemGray, Finder's Gray tag).
- `List { focus: ListFocus::OnMount }` gives the list the keyboard as it mounts (default `Manual`).
- `SheetActions { leading, children }`: a sheet's buttons side by side at the trailing edge, the
  default last, with an optional control kept at the start (`.ds-sheet-actions`,
  `.ds-sheet-actions-leading`).
- A menu whose opener the pick removed (the pick swapped the view) hands the keyboard to the
  opener's nearest focusable ancestor, remembered when the menu mounted, else to the window.
- `ProviderMark { size: ControlSize::Large }` is a 64 px mark (it repeated Regular before).
- `oncontextmenu: Option<EventHandler<MouseEvent>>` on `Column`, `Toolbar` and `Sidebar`: a right
  click there reaches the handler and not the menu of whatever holds it.
- design/22-SETTINGS.md: `bar.autohide` (`always`, `desktop_only`, `full_screen_only` by default,
  `never`), Appearance › Bar.

### Pictures
- `CollectionView { aspect: ThumbAspect::Landscape }`: a 16:10 thumbnail box (`ThumbAspect`
  is `Square` by default); an image covers it, an icon is drawn at its shorter side, and the row
  pitch shortens to match (`CollectionSize::pitch_for`). The view carries `data-aspect=landscape`
  and `--cv-thumb-h`.
- `CollectionView { extent: CollectionExtent::Content }`: the view is as tall as all its rows
  (`ceil(len / columns) * pitch`), mounts every tile and leaves the scrolling to the page
  (`data-extent=content`). `Fill` is the default and unchanged.
- New `Picture { source, alt, shape }`: an image cropped to fill a rounded frame (`object-fit:
  cover`, `--r-media`), landscape 16:10 or square (`PictureShape`); alt text is required.
- `MenuImage::Swatch(TagColour)`: a coloured dot in a menu row, for a Finder tag. Not yet: the
  single row of tag dots in a context menu (needs several picks on one menu line).

### Tables and selection

- `CollectionView { onmarquee }` and `VirtualTable { onmarquee }`: a drag that begins on empty
  space draws a rubber band in the accent tint and selects the tiles or rows it touches. The
  owner hears the whole selection as a `RowSelection` (store it as `selection`). No modifier
  replaces the selection and a click on empty space clears it; Shift or the primary modifier adds
  to the selection the drag began with. The band scrolls the view when the pointer moves near the
  top or bottom edge. New `RowSelection::of_ranges` and `RowSelection::united`.
- `VirtualTable { outline, ontoggle }`: the first column as an outline. `outline` answers each
  mounted row's `OutlineRow` (depth, leaf or `Branch(Shown)`); the table draws the indent and a
  disclosure triangle and reports `OutlineToggle { row, shown, reach }`. Option widens the reach
  to the `Subtree`; Right and Left on the cursor's row toggle it from the keyboard. The app keeps
  which branches are open. Dragging the header edge already resizes a column, held at
  `TableColumn::min_width`; double-click to fit is not built, as the table has no measure of a
  column's content.
- `NameCell { icon, name, room }`: a glyph and a name cut from the middle, for the Name column of a
  file list; `name_chars(room)` turns the room into the character budget.
- `FileDrop` carries `modifiers` (the keys held at the release) and `FileDragInput::Dropped`
  became `Dropped { modifiers }`. New `DropOperation { Copy, Move, Link }` and
  `FileDrop::operation(platform)`: Option forces a copy, the primary modifier (Command) a move, both a
  link, and `None` leaves the choice to the app. ds-blitz reads the modifiers from the keyboard
  state the window already tracks, since the platform's drop event carries none.

## v0.3.40

The files batch, second part: the icon view's tiles take a context menu, a drop and a badge; a
toast that appears under a still pointer lets the press through; file-manager glyphs with a MIME
map; a status bar; a trailing control in the command palette's search row and a group tooltip on
a segmented control. Also a faint hover wash on source-list rows, `Row { line_height }`, and plain
words where sign-in and installs stop. Blitz moves to 0be21979 and blitz-kit to 07f0272b: a
pointer press whose default action is cancelled (a pop-up menu's items cancel `mousedown`)
resets the drag state, so a menu item now picks on its first click on macOS. Consumers bump the
blitz and blitz-kit revs with quire's. chordkit stays at ae28c23.

**ds**
- A toast that appears under a still pointer no longer takes the press (the Mac's banners
  don't). The toast carries `data-armed`: `waiting` while the pointer has not moved since it
  appeared (`pointer-events: none`, not hovered, so its hold runs), `live` after a move, when it
  holds while hovered and its button takes presses as before. New `ds::stack::toast_hub::Armed`
  and `ToastHub::armed()`. Fixes detent's "Add computer" press under the toast.
- `Row { line_height: Some(Px(..)) }`: a flush row for text laid over a grid (a terminal's block
  header). The title and detail are each exactly `px` tall, the words start at the row's left
  edge, and the row has no padding, gap, minimum height or rounding; `size` is ignored. It sets
  `data-density="lines"` and the declared custom property `--row-line` inline. `None` (the
  default) changes nothing.
- `CollectionView { oncontextmenu: EventHandler<(usize, MouseEvent)> }`: a right click on a tile
  selects it through `onreach` or `onselect` unless it is already selected, then reports the
  tile's index and the event; the click does not reach the page's own menu. A right click on
  empty space is not reported and reaches it.
- `CollectionItem::in_drag(DropState)` (field `drop`) makes a tile a place that takes a drop, as
  `RowState::drop` does for a row: `Accepts` and `Target` write `data-drop` and make the view
  report the pointer through the new `CollectionView { ondropenter, ondropleave, ondrop }`
  (each `EventHandler<usize>`, heard only for accepting tiles); `Source` writes `data-drag` and
  dims the tile. The owner sets `Target` on the tile under the pointer. The thumbnail of a
  `Target` tile is washed and ringed in the accent, an `Accepts` one is outlined.
- `CollectionItem::badged(Element)` (field `badge`): a slot centred under the caption
  (`span.ds-cv-badge`), for Finder's tag dots. It sits in the room the caption band already
  keeps, so the tile pitch is the same with and without one.
- `ds_style::icon`: thirteen glyphs for a file manager, in the actions set (outline by default,
  each with a solid form): `Home`, `Eject`, `DriveInternal`, `DriveExternal`, `Disc`, and the
  per-type file glyphs `FileText`, `FileImage`, `FileAudio`, `FileVideo`, `FileArchive`,
  `FileCode`, `FilePdf` and `FileApp`. `DriveExternal` and `FilePdf` are quire's own drawings on
  Lucide's grid (Lucide has neither); the rest are Lucide.
- `ds_style::icon::mime::glyph_for_mime(mime) -> MimeGlyph { icon, description }`: a MIME type
  to its file glyph and Finder's plain kind name ("PNG image", "PDF document", "ZIP archive").
  The exact type is looked up first, then its major type, then `File` with "Document".
- `StatusBar` (new, `ds::components::chrome::status_bar`): Finder's bottom row. `text` centred
  between an optional `leading` and an optional `trailing` element, in the secondary ink over a
  hairline; the slots are always drawn so the text stays centred. `item_count_text(count,
  available: Option<&str>)` writes "12 items, 40 GB available" ("1 item"); the app formats the
  byte count itself, as quire has no byte formatter. Classes `ds-status-bar`,
  `ds-status-bar-leading`, `ds-status-bar-text`, `ds-status-bar-trailing`.
- `CommandPalette { trailing: Option<Element> }`: a control at the end of the search field's row
  (sill's Auto/Manual switch). The row is laid out inside the card with the control's own room
  (`.ds-palette-head`, `.ds-palette-trailing`), so a long query ends at the field's edge and never
  runs under it. `None` changes nothing.
- `SegmentedControl { title, title_shortcut }`: a hover hint for the whole group, drawn through
  the hover hub as a `Button`'s is (`Name  ⌘K`); `aria-label` keeps the full wording and
  `aria-description` carries the tip. A segment with its own tip keeps it.

**Consumers**
- Since v0.3.38 the `Sheet` panel takes the keyboard, so an inner `onkeydown` Return handler
  inside a `Sheet` no longer hears Return. Use `Sheet { on_return }` (anyview moved its six
  sheets).
- A stylesheet golden picks up the status bar and palette-row rules. Re-bless.

## v0.3.39

The files batch: a windowed icon view, a middle cut for names, and the virtual table opens rows
and leaves chorded keys to the app. chordkit moves to ae28c23 (Find Next and Find Previous are
offered, not universal); Blitz and blitz-kit stay where v0.3.38 had them.

**ds**
- `CollectionView` (new, `ds::components::lists::collection_view`): Finder's icon view, a grid of
  tiles (`CollectionItem { key, caption, thumbnail }`, `Thumbnail::{Image, Icon}`) that mounts
  only the rows near the viewport. `CollectionSize::{Small, Medium, Large}` (48, 64 and 96 px
  thumbnails), `Flow::{Fit, Columns(n)}`. Arrows move in two dimensions, Home and End go to the
  ends, Shift extends, the primary modifier toggles on a click, letters type-select. A double
  click or Command-Down asks `onopen`; Return, Space and every key with a modifier besides Shift
  go on to the app's keymap (Finder's Rename and Quick Look). Tiles fade in and out through the
  list roster; `CollectionItem::editing(Element)` is the rename slot. Captions are one line, cut
  from the middle.
- `VirtualTable` and its split panes: `onopen: Option<EventHandler<usize>>` hears a double click
  on a row and Command-Down on the cursor's row (consumed only when `onopen` is given). Return
  stays `onpick`'s. Any key with Control, Option, Command or Meta now goes on to the app instead
  of moving the cursor or picking: Command-Up reaches the app's Enclosing Folder.
- `.ds-vtable-rows` and `.ds-vtable-track` are styled (`display:block`); before, they had no rule
  and laid out inline.

**ds-core**
- `text::clip::Truncate { End, Middle }` with `clip(text, chars)`; `Middle` keeps the extension
  (design/27 section 5.24).

**chordkit (eb0deff → ae28c23)**
- Find Next and Find Previous are `Reach::IfOffered`: an app with no found text to step through
  (a file manager) gives them up with `forgo_all` and takes ⇧⌘G, as Finder's Go to Folder does.
  Find stays universal.

**Consumers**
- Move every chordkit pin to ae28c23 together with quire (one chordkit in the lock); relock.
- A stylesheet golden picks up the collection view and vtable rules; a self-lint or token test
  that lists per-element custom properties adds `--cv-thumb` and `--cv-pitch`. Re-bless.
- A table owner that read Command-arrows from the table's cursor moves now hears them in its own
  keymap.

## v0.3.38

A new fontique-only emoji crate, a caret that blinks, sheets that take the keyboard, and the toast
clear of a sheet. Additive: every old path still resolves; no pin moves (Blitz, blitz-kit and
chordkit stay where v0.3.37 had them).

**quire-emoji (new crate)**
- The colour emoji face detection moves out of `ds-blitz` into `quire-emoji`, which needs only
  `fontique`: `prefer_colour(&mut Fonts, &EmojiChoice)`, `EmojiChoice`, `EMOJI_FAMILY_ENV`,
  `colrv1_emoji_file`, `table_listed`. A program without Blitz (temor's terminal) shares the one
  detection. `ds_blitz::{EmojiChoice, EMOJI_FAMILY_ENV, colrv1_emoji_file}` re-export it.
- The COLRv1 file is found and read once per process and its `Blob` shared: calling
  `prefer_colour` per pane or per window no longer walks the font folders and reads ~10 MB each
  time. A font installed while the program runs is seen after a restart.

**ds**
- A focused `TextField`'s caret blinks: 500 ms shown, 500 ms hidden, shown again at every key,
  input and press; after ten idle seconds it holds still, shown, and the field paints no more
  frames (design/26 R3); never under Reduced motion. Plain, multi-line and secure fields.
  Markup: the field's wrap carries `data-blink=off` during the hidden half (issue #13).
- `Sheet` takes the keyboard a frame after it comes up when nothing inside it took it, as a Mac
  sheet becomes key; a first stop the caller asks for (`FieldFocus`/`ButtonFocus::OnMount`, an
  alert's default button) still wins. A sheet opened from a context menu's pick no longer leaves
  the keyboard nowhere. Markup: the sheet panel has `tabindex="-1"`.
- While a sheet or a peek is up, the toast sits at the top right instead of over the sheet's
  buttons (`data-place=top`).

**Consumers**
- SSR goldens that hold a sheet gain `tabindex="-1"` on `.ds-sheet`; a stylesheet golden picks up
  the blink and toast rules. Re-bless.
- A lock that reaches `ds-blitz` gains `quire-emoji`; relock.

## v0.3.37

The settings target gets a pure home, a hidden page, and the lock clock follows the region. Additive
for `ds-core` and `ds-settings`; every old path still resolves.

**ds-core**
- `settings::{SettingsTarget, TargetError, AccountRef, PageId, KeyPath}`: the types moved here
  from `ds-settings`, with their serde, `FromStr`, `Display` and tests, so sill's pure crates can
  hold a target without `ds-settings`. No new dependency.

**ds-settings**
- `target::*` and `schema::{PageId, KeyPath}` are re-exports of the `ds-core` types (same paths).
- `schema::Listing::{Sidebar, Hidden}` and `PageDecl::listing` (`listing = "hidden"` in a
  `[[page]]` table; absent means `Sidebar`), and `PageTable::sidebar()`, the pages the sidebar
  lists. A hidden page still resolves by id.
- The built-in page `assistant-model-setup` (hidden): the model picker's first-run step.

**Design notes**
- design/22: `session.lock_clock` is gone; the lock clock follows `region.hours` (3.32) and omits
  AM/PM like the Mac. New section 3.33 with `wallpaper.picture` (text, control `picture`, page
  `wallpaper`; empty means no picture, so the desktop paints `--paper`). design/28 follows.

**ds-style**
- `Icon::Download` and `Icon::Refresh` (the sync glyph) are redrawn as quire's own drawings, in
  outline and solid. Download is an arrow with an open chevron head over a baseline; Refresh is two
  clockwise arcs with arrowheads. Download is no longer `Native::Drawn`; the Bootstrap
  `cloud-download` conversion is gone (`Upload` still uses `cloud-upload`).

- `SettingsWriter::unset(key)` takes one key out in place (comments and the rest of the file
  stay; a missing key is a no-op), and `SettingsWriter::apply(&[Edit])` runs `Edit::Set` and
  `Edit::Unset` under one lock in one write. `patch` and `patch_all` are unchanged.
- design/22: `cua.accessibility` (on by default; cua's own key, the Computer use row "Read app
  controls").

**ds (the Mac pass, from mailo)**
- `SpaceMenu { on_edit }`: an app with its own Space page gets the id instead of the kit's sheet.
- `PresetSwatches` ends with a ninth swatch, Custom, whose popover holds the colour's field, stops
  and grain; new optional `oncommit` hears when a look is settled.
- `Chip`: pressable (`pressed`, `onpress`), a colour `dot`, and `onremove` on any variant; the
  Space editor's stops and the companion's picks use it. `Button { popup }` for a button that
  opens a menu, dialog or list. `Alert` takes a content slot, with a bare `AlertColumn`.
  `SignInFailed` takes an optional detail. A refused companion answer is an `InlineBanner`.
- Kit words are shorter, with contractions (sign-in, install, consent, delete).
- Scrolling: fingers that latched a scroller keep it until they lift; with blitz-kit f3c6ea87 a
  container whose content fits never rubber-bands, so a short sheet stays put under a drag.

**What a consumer changes**
- Nothing for the moved types. A literal `PageDecl { .. }` gains `listing: Listing::Sidebar`; code
  that lists the sidebar from `PageTable::pages()` calls `PageTable::sidebar()` instead.
- The blitz-kit pin moves to f3c6ea87 (same Blitz 851fa7dd). A test that matched a kit sentence
  word for word follows the new wording.
- The helper sheet (`HelperBody`) draws through `AlertColumn`, so `.ds-alert` no longer carries
  `data-phase` (ask, installing, failed, not-found, unsupported). A test that keyed on it reads
  what the sheet shows instead: `.ds-helper-progress`, the default button's label, the title.

## v0.3.36

Settings targets, pages as data, a picture control, patch writes, ranges and the hour cycle. One
additive release for `ds-settings`; sill and detent move onto it after this tag. Two compile
changes for a consumer that builds the schema types by hand (see "What a consumer changes").

**ds-settings**
- `target::SettingsTarget { Page(PageId), Key(KeyPath), Account(AccountRef) }`, written
  `page:wallpaper`, `key:dock.magnified_size_px`, `account:<id>`; `FromStr` and `Display` round
  trip. It is the payload of `settings.page.show`.
- Pages are data: `schema::{PageId, PageDecl, PageTable, PageStatus}` and `[[page]]` tables in a
  schema (`Schema::pages`). The built-in table (`schema/pages.toml`) holds the twelve fixed pages,
  their old names as aliases, and `wallpaper`, `displays`, `wifi`, `bluetooth`. A key whose page
  nobody declares is kept (`Page::App(<id>)`, `PageTable::status` is `Unknown`) instead of failing
  the schema. `Page` stays as a typed layer (`Page::id`, `Page::declared`, `Page::fixed`).
- One control vocabulary: `schema::{Control, ControlName, TogglePair}`, `control = "<id>"` on a key
  (`KeySpec::control`, `#[settings(control = "picture")]`), and `picture` (a grid of pictures with
  "Add Picture..."; the value is a path). `KeyKind::default_control` is the one kind-to-control
  table. `Widget` and `KeyKind::widget()` are gone (nothing called them); `ds-settings-page`'s
  `Pair` is gone, it uses `TogglePair` (so `enabled/disabled` and `yes/no` are switches there too).
- `SettingsWriter::patch(key, value)` sets one key in place under the file's lock and writes it
  atomically; comments and unknown keys keep their text.
- Ranges hold on load: `SettingsDoc::limits()` (default none; `AppearanceFile` returns its
  schema's), `Limits`, `ClampedKey`, `Loaded::clamped`, reported by `Loaded::diagnostics`. A
  number outside its range is brought to the nearest end instead of being refused.
- `region.hours` (`HourCycle::{Auto, H12, H24}`) in `appearance.toml`, `AppearanceFile::region`,
  `HourCycle::resolve(locale) -> ResolvedHours`, `time_locale(lc_all, lc_time, lang)` and
  `Environment::hours(locale)`.
- `schema::{regenerate, key_table}` and `examples/doc_tables.rs`: the key tables of design/22 are
  generated (`scripts/gen-doc-keys.sh`, `scripts/check-doc-keys.sh`).

**What a consumer changes**
- A literal `KeySpec { .. }` gains `control: None`; a literal `Schema { .. }` gains
  `pages: Vec::new()`. A derive-built schema needs nothing.
- A `Loaded { .. }` literal gains `clamped`; a `SettingsError` match gains arms (it is
  `non_exhaustive`).
- Code that named `Widget` or `KeyKind::widget()` uses `Control` and `KeyKind::default_control()`.

**design**
- design/22: sections 3.1, 3.3 and the new 3.32 `region` have generated key tables; the dock's
  magnified size and influence radius are 72 (the code), with their ranges; section 9.8 describes
  the additions.

## v0.3.35

Two API changes, three pin moves and a print fix. Consumers move their whole pin block with this
release (blitz, blitz-kit, chordkit; the vello fork patches are unchanged), then move their quire tag
and relock. Every crate that links ds-style must take the same quire commit.

**Pins**
- blitz fork 851fa7dd (from 4b236792): a nowrap line that overflows a clipping block ends with its
  ellipsis (`text-overflow: ellipsis`), at fractional scales, with ligatures, selections and table
  cells.
- blitz-kit 878dc799 (the same Blitz), chordkit eb0deff (a default that collides gives way to its
  declared fallback when the fallback is free; both taken still refuses).

**API changes**
- `CommandPalette`'s `onclose` takes a `PaletteClose::{Escape, Outside, Pick}`, so a consumer can
  close a whole bar on Escape and only dismiss on a click outside. A caller that does not care
  writes `|_| ...` instead of `|()| ...` (#44). A palette in a `CommandPaletteHost::Surface` never
  reports `Outside`.
- `ds_blitz::clipboard` reaches the primary selection: `ClipboardTarget::{Clipboard, Primary}` on
  `read_from` and `write_to`; the harness keeps the selection in memory (`Harness::primary_text`)
  (#45).

**ds-blitz**
- Print and PDF: a glyph in a bitmap-only colour face (`sbix`, `CBDT`, as Ubuntu's Noto Color Emoji
  and every Mac's emoji) is written as its picture, so a printout with an emoji no longer fails
  (#50).

**design**
- design/22: the `widgets` section is retired; its six keys are ignored where a file still has them.

## v0.3.34

The type-audit wave, the Mac batch, and four pin moves. Consumers change their whole pin block with
this release: the vello fork patches (`docs/fork-patches.toml`, `scripts/fork-patches.sh check`),
blitz-kit, chordkit and latchkey. Then they move their quire tag and relock. Every crate
that links ds-style must take the same quire commit. Two copies panic at runtime with
"Could not find context Signal<Scope>".

**Pins**
- vello fork 84b2c9f7 (from 3e43d4af): bitmap colour glyphs (`sbix`, `CBDT`) draw on both backends;
  vello_hybrid no longer panics on them, and glifo's `png` feature is on.
- blitz-kit f0e9dc3 (one `Scale120`, `Elapsed`), chordkit 7b80edd (keycap 52c66c7, protocol v3,
  through its lock), latchkey b10ae1b.

**API changes (type audit)**
- Removed: `KeysHost`, `DocumentHost::keys`, `KeyExtras`, `RawKey::unshifted`; the deprecated
  `command_keys` and `is_command`; `PendingStyle`; `SheetScale` (the widths are on `SheetSize`);
  the one-variant `Trailing` (`Button.trailing` is `Option<Icon>`); `Landing` in focus (internals
  take `Option<InitialCaret>`); the span module (`Stamp::after_span`).
- Renamed or merged: `SizeToken` replaces its per-size variants; one `SidebarToken` replaces
  `SidebarRowSize` and `SidebarTileSize`; `WeightRange` and `EffectActivity` replace the font range
  and the symbol effect's activity; `CapabilityName` and `ToolPresence` in ds-helpers; one
  `SenderName` and `Verdict` in ds-peer (re-exported at the old paths).
- Typed values: `Stamp::after` takes `Ms`; `PendingSpec` is an enum; `EasingToken::easing` takes no
  motion level; `Driver`, `DocQuery` and `Query` are methods on `Harness`; the switcher machine is its
  `Phase`; ds-blitz binding, run and surface states are enums.

**ds**
- Spaces: Delete Space asks in a critical Alert (#38).
- PresetSwatches: a Space's colour as one row of the eight presets (#30).
- TextField: the placeholder goes while an input method composes (#33); a secure field keeps its
  focus ring whole while typing (#32); a plain field marks a rejected value in the danger ink with
  a line under it, with its message as `aria-description` (#37).
- Row: a source list's selection is the neutral wash, focused or not (#34).
- Search card: the command palette's launcher scale (#28).

**ds-blitz**
- `use_content_fit`: a window as tall as its content (#29).
- `system_key_source(platform, lookup)` and `KeySource`: an app takes the desktop's keymap and
  follows it.
- Colour emoji fall back to a bitmap face where the system has no vector one, as on every Mac (#36).

**ds-settings**
- On macOS, System follows the Mac's appearance, reduced motion and contrast (#31).

## v0.3.33

Two menu fixes; no public API change and no pin moves (blitz stays at 4b236792). Consumers move their
quire tag and relock.

**ds**
- A context menu draws its items' key equivalents, as every menu does (the owner's override of the
  HIG row in design/27, 2026-10-17; golden menu/context-keys).
- A row in the command palette or a pick list follows the pointer only when the pointer moves: a menu
  closing over the rows no longer selects the row under a still pointer (blitz-kit's hover repair
  replays the last move after a layout change). The `Ds` root keeps where the last move reached; new
  conformance test palette_actions_click.

## v0.3.32

The blitz fork at 4b236792: an input method can delete the text around the caret and read it. A
consumer that pins blitz by rev moves its whole blitz block to 4b236792 with this release.

**blitz** (our fork, rev 4b236792, from 4ca21b12)
- An input method's DeleteSurrounding deletes the bytes before and after the caret (or the selection)
  in a text input: a pre-edit is cleared first, the counts are clamped to the text and rounded inward to
  char boundaries, and the input raises Input as a typed deletion does.
- New `TextInputData::surrounding_text()` and `BaseDocument::focused_surrounding_text()` return a
  `SurroundingText { text, cursor, anchor }`: the text around the caret, at most
  `SURROUNDING_TEXT_MAX_BYTES` (4000) bytes cut on char boundaries, with the pre-edit left out. A host
  sends it to the input method (shell-host's text-input-v3 surrounding text). Additive.
- blitz-kit moves to 8570d0f (the same blitz rev).

**ds, ds-shell**
- Text a person sees or hears says "assistant" for the companion (the orb's label, the confirm sheet).

## v0.3.31

Two additions: `ds_helpers::DesktopFile`, a pure reader for `.desktop` files, and the blitz fork at
4ca21b12 (iframe Tab order and wheel chaining). A consumer that pins blitz by rev moves its whole blitz
block to 4ca21b12 with this release.

**ds-helpers**
- New `DesktopFile`, a pure reader for the text of a freedesktop `.desktop` file (and any key file in
  that format): `DesktopFile::parse(text, ReadCap)` / `from_bytes`, `entry()` for `[Desktop Entry]`,
  `group(name)`, `groups()`, `skipped_lines()`; on a group (`DesktopGroup`) `string`, `localized`, `list`,
  `localized_list`, `boolean`, `is_true`, `key_names`. A localized key (`Name[de_AT]`) is picked by
  `EntryLocale::parse("de_AT.UTF-8")` and its `variants()`; value escapes (`\s \n \t \r \\`, `\;` in a
  list) are taken; a repeated group merges and a repeated key keeps its last value; `Exec` field codes are
  left to the caller. A text over the cap is `DesktopFileError::TooBig`; `DESKTOP_ENTRY_CAP` is 256 KiB.
  Additive; no existing item changes.

**blitz** (our fork, rev 4ca21b12, from 5c755366)
- Tab and Shift-Tab move through a sub-document's (an iframe's) focusable nodes and then on to the
  next one after the host, instead of being forwarded into the sub-document; a sub-document with none
  is skipped.
- A wheel, touch pan or fling over a sub-document scrolls it first and passes what it cannot scroll,
  per axis, to the host element's scroll ancestors, so a reader frame that fits its content no longer
  swallows the wheel.
- blitz-kit moves to 5db6e15 (the same blitz rev). A consumer that pins blitz by rev moves to 4ca21b12 with this release (the whole `[workspace.dependencies]`
  blitz block, as before).

## v0.3.30

Two additions for the permission prompts the compositor publishes. Nothing changes for a consumer
that does not opt in, except one thing: `ds_settings::Page` gains a variant, so a consumer's
exhaustive `match` on `Page` needs an arm for `Page::Privacy` (or a wildcard).

**ds-settings**
- New `Page::Privacy`, the page for "Privacy & Security" settings (what programs may reach; casement
  publishes its per-app permission rows here). `Page::Privacy.label()` is `"Privacy & Security"`; its
  wire form is `{"kind":"privacy"}` in JSON and `kind = "privacy"` under `[key.page]` in a schema
  TOML. It sits after `Accounts` in the page list (design/22 sections 5 and 9.3). A schema written
  before it reads as it did; a Settings app that predates it fails to parse a key on this page.

**ds** (`Alert`)
- New prop `arm_after: Option<Duration>` (default `None`). With a delay the alert answers nothing
  for that long after it mounts, so a keypress or click already in flight cannot answer a permission
  question: the buttons are disabled (`aria-disabled`, the Button's disabled look, no hover or
  press) and Return, Escape, Space and clicks do nothing. Escape neither dismisses the sheet nor
  calls `on_dismiss` while unarmed. The keyboard moves to the default button when it arms. Unset is
  today's behaviour exactly. The wait is a `Machine`, so a harness's virtual clock drives it.
- New module `ds::components::overlays::alert_arming`: `Arming` (a `Machine`: `Armed`, or `Waiting {
  until }`), `ArmingIn`, and the pure rule `route(Arming, AlertInput, &[AlertRole], Hearing) ->
  Route` (`AlertInput`, `Hearing`, `Route`) that decides what an input does, which the alert now
  uses for Return, Escape, Space and clicks alike.

## v0.3.29

Consumers re-pin once; the lock gains `toml_edit` for ds-settings
and `rustix` (Unix) for ds-helpers, both already resolved in the tree.

Production audit, wave 2 (QUIRE-03, QUIRE-12 library side, DETENT-05). Two signatures change, so
consumers adapt once on re-pinning; the lock gains `peephole-bus` (and its deps) for `ds-blitz`'s new
optional `peephole` feature only.

**ds-settings**
- `LiveClient::new(connection, destination, path, check: impl OwnerCheck)` is now synchronous (it
  touches the bus on first use) and takes the check as a required argument. Before the first read or
  `Set`, and again whenever the name's owner is a different connection, the client asks
  `OwnerCheck::holds_role(name, owner)`; a refused owner is the new `LiveError::NotTheService` and
  nothing is sent to it (a typed secret never reaches a squatter). Calls then go to the owner that
  passed by its unique name. `OwnerCheck`, `OwnerAnswer` and (feature `testing`) `AnyOwner` are new.
- The client refuses replies over `ds_settings::live::limits`: a schema over `SCHEMA_JSON_MAX`
  (1 MiB), `KEYS_MAX` (1024) keys, `TEXT_MAX` (2048 bytes) per path, label, help, note or section,
  `CHOICES_MAX` (512) choices; a value over `VALUE_TEXT_MAX` (8 KiB) or `VALUE_ITEMS_MAX` (512) items.
  The refusal is `LiveError::Failed("The service sent more than a settings page can show.")`.
- New feature `testing` (implies `live`); the `live` test binary needs it.

**ds-blitz** (feature `menus`)
- `MenuExport::start(config, tree, on_activate, permit: impl MenuPermit)`: the permit is required.
  `MenuPermit::permit(&Caller) -> Verdict` is asked before every `Event` and `EventGroup`; a message
  with no sender or a refused caller gets `AccessDenied`. Reads stay open. New: `Caller`,
  `SenderName`, `Verdict`, `MenuPermit`; with feature `peephole` (Linux) `ShellOnly::new(Arc<BusPeers>)`
  (role `shell` at the full proof); with `testing` `AllowAnyCaller`.
- The menu's name is requested with `DoNotQueue`. `NameOutcome` and `MenuExport::name()` are gone;
  a name that is taken is `MenuExportError::NameTaken(name)` and nothing is exported.
- `EventGroup` carries at most 64 events a call (`InvalidArgs`) and a sender may make 8 calls a
  second (`LimitsExceeded`).
- New optional dependency `peephole-bus` (git tag v0.1.1, Linux only) behind feature `peephole`.

**ds-helpers**
- `update_atomic(path, cap, new_file, edit)` and `UpdateError<E>` (`Read`, `Edit`, `Write`; non-exhaustive)
  are new: read the file as text under a cap, let `edit(Option<String>) -> Result<String, E>` produce
  the new text, publish it as `write_atomic` does. Writers in the process queue on the path and
  writers in other programs on an `flock` of the sidecar `.<name>.lock` beside the file (0600, left in
  place; a holder that does not let go in five seconds makes the update fail with `TimedOut`).
  Do not call `write_atomic` or `update_atomic` for the same path from inside `edit`.

**ds-settings**
- `Store::save` of a TOML file now merges into the file already there (format preserving): keys the
  type does not read and every comment stay, a changed value keeps its spacing and trailing comment,
  and a key the type owns and no longer writes is removed. A new file and the JSON and CSS stores
  are written as before; a damaged file is still moved aside (`Saved::kept_at`). A save leaves a
  `.<name>.lock` file beside the settings file.
- `ds_settings::xdg` is public: `base_dir(xdg, home, under_home)` and the `CONFIG_HOME_UNDER_HOME`,
  `STATE_HOME_UNDER_HOME` and `DATA_HOME_UNDER_HOME` constants.
- `AGENT_NEVER_SETTABLE` gains `privacy.`, `network.` and `security.`. A schema that marks a key under
  one of them agent-settable is refused (`Schema::from_toml`, `LiveSchema::check`) or fails to
  compile (the derive). Programs whose schemas do must drop the mark.
- `Loaded::unknown`'s `Display` no longer says the key is "not kept".

**ds**
- `Alert { on_dismiss: Option<EventHandler<()>> }`: while set, Escape (and the sheet closing by any
  way but a button) calls it and presses no button, so a dismissal is told apart from the Cancel
  button. Unset, Escape presses the Cancel button as before.

## v0.3.28

Consumers re-pin once. Cargo.lock files are refreshed by whoever bumps the pin.

### Breaking, by crate

**ds-settings**
- `Store::save` returns `Result<Saved, SettingsError>` instead of `Result<(), SettingsError>`.
  `Saved::kept_at` is where a damaged file in the way was moved (`<name>.bad-<seconds>`); tell the
  person when it is `Some`. `SpacesStorage::save_spaces` and `save_today` return `Saved` too, and
  their type parameters now also need `DeserializeOwned` (a file that is not the type is moved
  aside, not overwritten).
- `Origin` (from `boot_spaces`) is `#[non_exhaustive]`, no longer `Copy`, and gains
  `Damaged(DamagedFile)`: a `spaces.json` that is there and unusable. The Spaces run from
  `first_run()` in memory, nothing is written, and the first save moves the file aside. Match it
  with a wildcard arm and clone where you copied it.
- `SpacesStorage::load_spaces` returns `SpacesLoad<P, R>` (`Stored`, `Missing`, `Damaged`;
  `#[non_exhaustive]`) instead of `Option<Spaces<P, R>>`. A damaged file is no longer a first run.
- `DamagedFile` and `Saved` are new, exported from the crate root.
- `data_dirs()` (schema discovery) lists the system directories first and `XDG_DATA_HOME` last, so a
  user's schema never shadows the system's. A schema is admitted only for its own program and
  location, and schema files are capped.
- Settings and Spaces files are read with a size cap and never block on a FIFO; they are written
  durably and atomically, readable by the person only.

**ds-settings, `live` feature**
- `LiveError` (already `#[non_exhaustive]`) gains `Timeout(String)`, raised by the client when a
  module does not answer.
- `LiveClient::with_timeouts(Timeouts)`; `Timeouts::new(read, action)` and `ds_settings::live::Deadline`
  are new. Every client call has a deadline.
- `Changed` is sent by unicast to the peers the module let read (those that called `Describe`,
  `Get`, `Snapshot` or `LiveClient::changes`), not broadcast. A client takes a signal only from the
  module it asked.

**ds-blitz**
- `AppConfig::with_file_roots(FileRoots)` is new, and `ds_blitz::FileRoots` is re-exported from
  blitz-kit. The app document reads a `file:` URL only from inside the directories the app names; a
  path outside them, through a symlink out of them, or with `..` is refused, and so is anything but
  a regular file of at most 32 MiB. The default names none: an app that sets no roots reads no
  `file:` URL at all (`data:` is unaffected, and a frame is never served `file:`). Apps that show
  local images or icons (wallpaper cache, icon directories) must set their roots. The same setter is
  on `ds_harness::HarnessConfig`.
- `ds_blitz::seam::DsNet::top` takes the `FileRoots` as a fourth argument; `Setup` gains `file_roots`.
- `AppConfig::with_link_failure(LinkFailureHandler)`; `LinkFailureHandler` and `LinkOpenError` are
  new. A clicked link is opened on a worker; a link that cannot be opened is reported to the handler
  (default: stderr). A `mailto:` link loses its `attach` and `attachment` fields.
- Clipboard reads run on a worker with a wait and a size cap. The workspace `arboard` no longer
  enables `wayland-data-control`.
- The print portal's calls have deadlines; the print fallback stages the PDF privately.
- Feature `pdf` now depends on ds-helpers. A thumbnail reads its PDF through `read_capped` (128 MiB,
  regular files only); a bigger file shows as a PDF with no preview.

**ds-helpers**
- New files API: `read_capped`, `read_text_capped`, `ReadCap` (`bytes`, `kib`, `mib`), `ReadError`
  (`#[non_exhaustive]`; `NotFound` is the only variant that means absent), `write_atomic` with
  `NewFileMode`, `WriteError`, `move_aside` with `UnixSeconds`.
- `PackageKit::with_timeouts(Timeouts)`; `Timeouts::new(step, install)`; `Deadline` re-exported.
  PackageKit signals are believed only from the PackageKit that made the transaction, every wait has
  a deadline, and only a package that was asked for is installed.
- `PackageName` accepts `[A-Za-z0-9+._-]` only, starting with a letter or digit.

**ds-peer** (new crate)
- `Deadline` (`READ`, `START`, `ACTION`, `PROMPT`, `of`, `duration`), `within(...)` and
  `NotAnswering`: one deadline for every call to a peer.

**Others**
- `ds-desktop`: `Desktop::probe`, `Outputs` and their watches stop waiting on a silent bus after
  `Deadline::START`; output rows with no size or no scale are dropped and the list is capped.
- `ds-shell`: `Hidden` wipes its memory when dropped (adds `zeroize`).
- `ds::root::DataName::parse` takes a `&'static str`.
- A stored `Lang` goes through `Lang::parse`; ids and times read from files are range-checked.
- `deny.toml` checks advisories, bans and sources.

### Pins

- blitz-kit `0ba642e76f3c7edca77b228f05e0399af46e2ec3`: `blitz_kit::net::FileRoots`,
  `LocalNet::new(FileRoots)`, `LocalSource::read_within(&FileRoots)`, a 32 MiB cap and
  regular-file-only reads (`MAX_RESOURCE_BYTES`); `adapter::request_watched_device` with a deadline,
  `DeviceHealth`, `block_on_timeout`, and wgpu errors logged instead of panicking.
- latchkey `99cc9deb198860293253c534c4d26b382ec8b87e`: new `Error` variants `Unsafe` and `WrongOwner`
  (match them), `standing()`.
- keycap-client: quire pins no keycap crate. keycap `fe71ad2c7de113ee6e98f7ea1afd9514536a097c`
  (client socket close-on-exec, bounded non-blocking connect, bounded `read_text`) is for the
  consuming apps and the desktop to pin themselves.
