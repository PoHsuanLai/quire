# Release notes

Quire has published its release notes as the tag message. This file keeps them in the tree, newest
first, with what a consumer has to change.

## v0.3.41 (unreleased)

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
