# mailo migration: adopting quire

The brief for the "mailo ui" Claude session, in mailo's own repository
(`/home/pohsuanlai/mailo`), covering both waves of the port from mailo's hand-rolled CSS and
markup onto quire's design system. mailo is otherwise untouched by this program; its own gates
(`ORCHESTRATION.md`, `CONVENTIONS.md`) stay authoritative — this file only adds quire to them.

Two phases, run as two separate waves, **Phase B only after Phase A is merged**: Phase A stays
on the Dioxus desktop webview (`dioxus::LaunchBuilder::desktop()`, unchanged) and swaps mailo's
own design layer for quire's, verified by mailo's existing gates plus quire's lint; Phase B
swaps the webview itself for `ds_native::launch` (Blitz) and removes everything that existed
only to work around the webview. Doing both at once would make a failure impossible to bisect:
"is this a quire-adoption regression or a renderer regression?" has to have one answer.

## Read first

- This repository's `CONSUMING.md` (all of it — sections 1-4 for the mechanics, section 5 for
  the four coherence rules verbatim, section 8 for what Blitz cannot do, section 9 for the two
  gaps `examples/consumer` found, both since fixed in quire).
- The next section, "The quire APIs this brief assumes": what quire provides today, so no step
  below rebuilds something quire already has.
- `design/README.md`'s reading order, at least through `04-COMPONENTS.md`, `05-MOTION.md` and
  `06-INTERACTIONS.md`; `21-SPACES.md` for the `Space`/`SpaceLook` split (section 3 below);
  `22-SETTINGS.md` sections 2 and 9 for the settings-file and schema mechanics.
- `DESIGN.md`, this repository's map from design doc section to the module that implements it —
  use it to find a component's exact file once the mapping table (section 2) names it.
- In mailo's own repo: `CONVENTIONS.md` (unchanged, still binding), `ORCHESTRATION.md`'s
  verification block (section 5 below runs it plus quire's own gates), and every file this brief
  names in sections 4 and 6 — read the whole file before editing any of it, several are
  1000+ lines mixing what moves with what stays.
- `examples/consumer` in this repository (`src/lib.rs`, `src/style.css`,
  `tests/coherence.rs`): the smallest correct instance of everything this migration asks mailo
  to do — one `Ds` root, components instead of raw markup, a stylesheet that lints clean, motion
  through `ds::use_motion_timer` instead of `dioxus::document::eval`.

## The quire APIs this brief assumes (polish pass, 2026-09-24)

Each is in quire today, tested, and documented at the anchor given; read the anchor before the
step that uses it. Where one of these covers something mailo hand-rolls, the hand-rolled version
is deleted, not ported.

- **The root, and an idle toast host.** `Ds` renders `OverlayHost` and `ToastHost` after your
  children; `ToastHost` lays out nothing while the hub is empty (no hidden pill in the markup
  or the picture). `CONSUMING.md#2-the-ds-root`; FINDINGS
  `#gallery-fixes-a-2026-09-24` item 2.
- **Surface overrides.** `Surface { material, theme, accent, blur }`, each `Option` inheriting
  when `None`: a nested material, scheme, accent or blur state without a second `Ds`.
  `CONSUMING.md#4-surface--a-nested-material-scheme-accent-or-blur-state`.
- **HoverCard parts.** `HoverCard { parts: Vec<HoverCardPart> }` (`Title`, `Sub`, `Person`,
  `Stats`, `Flag`, `Messages`, `Foot`, `Actions`), drawn in order before its children: mailo's
  `.hc` sender and thread cards (`ui/hover/cards.rs`) become parts, never hand-written
  `ds-hovercard-*` markup. `CONSUMING.md#6-the-component-catalogue`; design/04-COMPONENTS.md
  section 22.
- **Undo through the toast.** `ds::use_toasts().push_undoable(text, token, on_undo)` calls
  `on_undo` with the token when the person undoes that toast (the pull tab or a click); a later
  push replaces it, and the handler's scope must outlive the toast. `push(text, undo)` and
  `last_undo()` remain for a toast with no handler. `CONSUMING.md#overlays`.
- **Anchoring to a button.** `Button` and `IconButton` take `mounted`; hand the element to a
  `Menu`/`Popover` as `Anchor::Mounted(MountedRef(event.data()))`, measured when it places
  itself. No wrapper span to measure. `CONSUMING.md#overlays`; `examples/consumer/src/lib.rs`.
- **Spacing tokens.** `ds::SpacingToken`: `--s-1`, `--s-1-5`, `--s-2` … `--s-12`, `--s-13`,
  `--s-14`, `--s-15`, `--s-16`, `--s-18`, `--s-22`, `--s-26`, `--s-36` on `.ds`, each named by
  its pixel value; mailo's own layout CSS uses them for every margin, padding and gap.
  design/01-LAYOUT.md#2-units-and-the-spacing-scale.
- **The lint rules.** `ds::lint` at `Profile::Strict` over mailo's own CSS, `Rule::RawSpacing`
  included; the markup lint over rendered pages checks unstyled classes, raw form controls and
  SVG (an `<svg>` is quire's when it is `.ds-ic` or carries `data-ds-svg`), and every inline
  `style` declaration (a literal colour or duration is an offence, except a custom property on
  a quire element). `CONSUMING.md#rule-1--no-literal-design-values-in-your-own-css` and
  `#rule-2--no-raw-markup-only-quire-components`.
- **The runtime.** `ds_native::launch` and `ds_native::Harness` each enter a process-wide
  Tokio runtime for their whole life, so `ds_settings::use_environment` works under both with
  nothing entered by mailo. `CONSUMING.md#3-reading-appearance`.
- **The window frame paints (mailo gaps, 2026-09-24).** A `Material::Window` root stamps
  `data-frame="opaque"` and is its own stacking context, so its two gradient layers and grain
  paint over its own background: a `look` change cross-fades over `--t-scene` and the Space's
  grain shows. Before, both painted beneath the root and a switch was an instant swap. Nothing to
  change in mailo; a pixel test that assumed a flat window ground now sees the grain (give it
  `Grain(0)`). FINDINGS `#mailo-gaps-2026-09-24` item 1.
- **Status inks.** `--ok-ink`, `--warn-ink` and `--danger-ink` are the text on `--ok`, `--warn`
  and `--danger`, each at least 4.5:1 in both schemes (dark `--danger-ink` is now `#1A0B08`,
  mailo's own value). Use them instead of a white or a hex of your own.
- **Person colours.** `ds::person_hue(address) -> ds::PersonHue` is design/03's person hash
  (`h = (h x 31 + code) mod 360`, `hsl(h, 38%, 42%)`): paint it with
  `AvatarTone::Person(hue)`, or take `hue.colour()` where a component wants a `Colour`. The eight
  stored-colour swatches are `ds::PersonSwatch` (`--c-person-1..8`, mailo's `AVATAR` order;
  `PersonSwatch::nth(i)` wraps, `.colour()`/`.var()`). mailo deletes `space::AVATAR` and
  `compose/items.rs::hue`. `CONSUMING.md#person-colours`.
- **Action glyphs.** `Icon::Printer` and `Icon::FolderInput` (Lucide `printer`, `folder-input`,
  `Icon::ACTIONS`).
- **A Blitz click caveat.** A `Button` whose parent holds only inline content is not hit on
  Blitz (the parent is); every quire container is a flex row, so wrap a lone button in one.
  `CONSUMING.md#8-what-blitz-cannot-do-and-what-to-use-instead`; FINDINGS
  `#polish-pass-2026-09-24`.

## 1. What changed since the plan was written

The orchestrating plan (`~/.claude/plans/vast-toasting-peach.md`, "mailo consumption") names
`mail-app/src/ui/frame.rs` as the file holding `appearance_script`/`frame_statements`/
`SYSTEM_ACCENT`/`eval`. In mailo's current tree that code is in **`mail-app/src/ui/paint.rs`**
(`frame_pairs`, `appearance_script`, `paint_script`, `SYSTEM_ACCENT`); `ui/frame.rs` today is a
different, smaller file (`Boot`, `load_boot`, `keep`, `scope_ids` — loading `Spaces`/`Today` at
startup, nothing about appearance). `ui/menus.rs` is likewise `ui/menus/mod.rs` (plus
`tests.rs`), and mailo has **two** menu-shaped modules: `ui/menu/mod.rs` (menu item tiles: `Tile`,
`Trail`, the `.fmenu` floating menu itself) and `ui/menus/mod.rs` (the menus that use those
tiles — folder/label/snooze/etc.). `ui/compose.rs` is a directory, `ui/compose/mod.rs` plus
eleven sibling files (`body.rs`, `desk.rs`, `float.rs`, `items.rs`, `life.rs`, `opening.rs`,
`page.rs`, `pill.rs`, `props.rs`, `recipients.rs`, `render.rs`, `wire.rs`). Every file list below
uses mailo's actual current paths, not the plan's.

## 2. The mapping: mailo class/component -> quire component

Every row is a real class or component found in mailo's current source, not a guess. "Stays"
means the class remains mailo's own — its container/layout role, not a quire component's
concern (section 6 has the full "what mailo keeps" list).

| mailo (file:line) | quire component | Notes |
| --- | --- | --- |
| `.cmd` (`ui/sidebar/mod.rs:59`) | `CommandPill` (§8) | the "Search or run a command / ⌃T" trigger |
| `.item` (`ui/sidebar/panes.rs:318` `.item.pinned`; `ui/compose/desk.rs:168` `.item.today-item.draft`) | `SidebarItem` (§19) | both the pinned-item row and the Today draft row are the same shape |
| `.row`, `.row-dot`, `.row-main`, `.row-from`, `.row-sub`, `.row-snip`, `.row-tail`, `.row-time` (`ui/row.rs`) | `ListRow` (§16) | `ListRow`'s props (`name`, `via`, `subject`, `snippet`, `time`, `tags`, `star`, `strip`) are exactly this row's fields — see `CONSUMING.md` section 6's Lists example |
| `.strip`, `.fly`, `.floater` (`ui/row.rs:251,296,301`) | `HoverStrip` (§17) | the hover-reveal action row and its `zZ` snooze float |
| `.fmenu` (`ui/menu/mod.rs:58`, `.fmenu.slim` too) | `Menu` with `MenuKind::Rich` or `MenuKind::Slim` | mailo's own `slim` variant is `MenuKind::Slim`; everything else is `MenuKind::Rich` |
| `.cmdk`, `.cmdk-wrap`, `.cmdk-in` (`ui/command/mod.rs:72-97`) | `CommandPalette<T>` (§25) | |
| `.hc`, `.hc.beside`, `.hc.tip`, `.hc.side-card` (`ui/hover/cards.rs`) | `HoverCard` + `HoverTarget` (§22) | the three `.hc` modifiers are placement, which `ds::place`/`Placement` computes instead of being hand-picked per call site; the card's contents are `HoverCardPart`s |
| `.toast` (`ui/motion/toast.rs:71`) | `Toast` / `use_toasts` (§23) | `ds::use_toasts().push_undoable(text, token, on_undo)` (or `push(text, undo)` with no handler) replaces the whole `motion/toast.rs` state machine |
| `.seg` (`ui/space_editor/parts.rs:21,162`) | `AppearancePicker` (§26) or `SegmentedControl<T>` | the Space editor's Theme segment is literally the appearance picker's job; "Provider marks" (icons vs. letters, `ui/space_editor/parts.rs:162`) is a plain `SegmentedControl<Marks>` — `Marks` has no quire equivalent (section 3) |
| `.pin.acct`, `.av`, `.n` (`ui/sidebar/panes.rs::AccountTiles`) | `AccountTile` (§27) | `AccountFace` is the data type; `space::avatar_color` stays mailo's (an `AccountId`-keyed colour, not part of `ds::Space`) |
| `.prov`, `.prov.img`, `.prov.on-tile`, `.prov.in-row` (`provider/icon/chip.rs::ProvChip`) | `ProviderMark` (§28) | `ProvChip`'s two `ChipPlace` variants (`Tile`, `Row`) are `ProviderMark`'s own placement, not a mailo concern once moved |
| `.plink` (`ui/compose/props.rs:49`) | `LinkPill` (§29) | the "To" recipient's linked-person pill |
| `.pchip` (`ui/compose/props.rs:71`, attachment chip) | `Chip` (§10) | a plain neutral chip, not `LinkPill` — it names a file, not a person |
| `.sp`, `.sp-text`, `.track`, the send ring (`ui/compose/pill.rs`) | `SendPill` / `SendPhase` (§31) | |
| `.p-menu` (`ui/compose/props.rs:142,221,349`) | `Menu` with `MenuKind::Dropdown` | the From/Sends/pin dropdowns |
| `.c-pin` (`ui/compose/props.rs:300`) | `EdgeStrip` (§33) or stays — read the surrounding markup before choosing; not resolved by this brief |
| the Space dots editor (`ui/space_editor/{hue,mod}.rs`) | `SpaceEditor` / `SpaceDot` (§32) | this is the one component whose whole job is a settled design, not a stylesheet: read `space_editor/mod.rs` fully before starting |
| drag-and-drop ghost (search for `dragstart`/`dataTransfer` under `ui/`) | `DragGhost` (§34) | not confirmed present in the files this brief's author read — check before assuming it exists |
| a syncing indicator, if any (search `SyncState` renders under `ui/`) | `SyncHalo` (§35) | same caveat |
| raw `button`s with a `title`, an `aria_label` or `aria_expanded` (about 130 across `ui/`) | `Button { title, aria_label, expanded }` (§1), or `IconButton { tooltip, label, expanded }` (§2) for a glyph-only one | mailo gaps 2: `expanded` is `Expanded::{Open, Closed}` on `Button`, a `Switch` on `IconButton` |
| `Field { kind: FieldKind::{Boxed, Inline, Secret}, on_focus, on_blur }` (`ui/field.rs`) | `TextInput { variant, kind: TextInputKind::{Text, Password}, onfocus, onblur }` (§6) | mailo gaps 2. The "typing in a field" guard hangs off `onfocus`/`onblur`. `TextInput` is controlled: a `Password` writes its `value` like any field (mailo's `Secret` kept it out of the markup), and draws dots over it on Blitz |
| `Field { kind: FieldKind::Range { min, max } }` (the Space editor's grain) | `Slider` (§5) | already the range: `Fraction` thousandths, so grain `n` is `Fraction(n * 10)`; the `SpaceEditor` draws its own grain row |

Everything under `ui/icon/` (the glyph set) maps to `ds::Glyph`/`ds::Icon` — `08-ICONS.md` and
`DESIGN.md`'s icon row have the geometry; mailo's own `ui/icon` module is deleted, not ported
(`DESIGN.md`: "icon geometry + enum" already moved to `ds::icon` verbatim, with tests).

## 3. `Space` -> `SpaceLook`: which fields move

`crates/mail-app/src/space.rs`'s `Space` struct (full definition, so nothing is guessed):

```rust
pub struct Space {
    pub name: String,               // STAYS — mailo's own, not part of a Space's "look"
    pub dots: Vec<Dot>,              // MOVES — ds::SpaceLook::dots (ds::Dot, moved earlier, same shape)
    pub grain: u8,                   // MOVES — ds::SpaceLook::grain (ds::Grain, a newtype over the same u8)
    pub theme: Theme,                // MOVES — ds::SpaceLook::theme (ds::Theme, same three variants)
    pub motion: Motion,              // STAYS — motion is global in quire (design/21 section 1); see below
    pub card_accent: CardAccent,     // MOVES — ds::SpaceLook::card_accent (ds::CardAccent; mailo's
                                      //   `Hint` is ds's `SpaceHue`, `Postmark` is unchanged — rename
                                      //   the variant at every call site, `CardAccent::Hint` no longer exists)
    pub scope: Scope,                // STAYS — account scoping, a mail concept, not a look
    pub pins: Vec<Pinned>,           // STAYS — pinned people/searches, a mail concept
    pub colors: BTreeMap<AccountId, String>,  // STAYS — per-account avatar tint, keyed by a mail-domain type
}
```

**Not a gap: `ds::SpaceLook` has no `motion` field, by design.** design/21-SPACES.md section 1
keeps motion global: a Space is a look (`dots`, `grain`, `theme`, `card_accent`), and how much
the window moves is part of the `Appearance` (`ds::Appearance.motion`, resolved with
`SystemPrefs` into the root's `data-motion`), not of any Space. quire will not grow a per-Space
motion field. mailo's per-Space `motion: Motion` (Calm/Standard/Extra) is therefore mailo's own
preference and stays in mailo's `Space` struct exactly as it is. If mailo wants the window's
motion to follow the Space, it passes that Space's motion into the root itself: where it builds
the `ds::Appearance` it hands to `Ds`, it sets `appearance.motion` from the current Space's
motion (`Calm -> ds::Motion::Calm`, `Standard -> Standard`, `Extra -> Extra`) instead of the
stored appearance's. Everything under the root then follows through `use_env`/`data-motion`.
A Space switch that changes the motion re-renders the root with the new level; the cross-fade
itself runs at the level the root had when it started. (FINDINGS "mailo gaps", item 6.)

`crates/mail-app/src/view.rs` lines 149-389 (`Theme`, `Motion` — wait, **not** `Motion`: mailo's
own three-variant `Motion` has no `System` option and stays for `Space::motion` per the
paragraph above; only `Theme`, `Marks`, `Appearance`, `Peek` actually move) map onto `ds` types one for
one, with two differences to fix at every call site, not paper over:

- `ds::Motion` has five variants (`System`, `Calm`, `Standard`, `Extra`, `Reduced`) where
  mailo's `Motion` has three (`Calm`, `Standard`, `Extra`, default `Standard`, no `System`).
  `ds::Appearance.motion` is a `ds::Motion`; mailo's own stored `motion_level` in the imported
  settings file should read as `ds::Motion::System` only where mailo genuinely had no opinion —
  check `appearance.rs`'s current default before assuming `Standard` maps to `System`; it does
  not (`Standard` is mailo's *explicit* default, not "follow the desktop", so it maps to
  `ds::Motion::Standard`, never `System`).
- `ds::Appearance` is `{ theme, accent, motion }` — three fields. mailo's own `Appearance` is
  `{ theme, motion, marks }` — no `accent` (mailo retired its six accent hues; see
  `view.rs`'s own doc comment: "mailo's `marks`... is a mail preference and stays in mailo;
  `accent` came back as one of six", `crates/ds/src/appearance/appearance.rs`'s module doc).
  Construct `ds::Appearance` from mailo's stored value with `accent: ds::Accent::Postmark`
  (the design default) until Phase A adds an accent picker to `AppearancePicker`'s call site, and
  keep `marks: Marks` (`Icons`/`Letters`, whether a provider chip shows its cached icon or a
  letter) in mailo's own settings file — it is a `ProviderMark`/`ProvChip` policy, not part of
  any quire `Appearance`.

## 4. `appearance.json` -> `appearance.toml`

`ds_settings::file::load_or_import(dir, legacy_json)` (`crates/ds-settings/src/file.rs`,
re-exported as `ds_settings::load_or_import`) is already built and tested for exactly this: it
reads `dir/appearance.toml` if it exists, else reads `legacy_json` (mailo's
`~/.config/mailo/appearance.json`), maps its `theme`/`accent`/`motion` fields onto a fresh
`AppearanceFile`, writes `dir/appearance.toml`, and **never modifies or deletes the JSON file**
(design/22-SETTINGS.md section 2, "Mailo migration"). Since mailo's own JSON has no `accent`
field any more (section 3 above), the import gives every migrated user `Accent::Postmark`,
which is correct — there is nothing to carry over.

`ds_settings::use_environment(app)` imports the same file for **every** app, mailo included:
on its first load, when the app's own `appearance.toml` does not exist yet, it calls
`load_or_import(<app's config dir>, <mailo's config dir>/appearance.json)`. For
`AppName::MAILO` both are `~/.config/mailo/`, so `use_environment(AppName::MAILO)` imports
mailo's JSON on first run and watches `appearance.toml` from then on (the private
`environment::initial_from` is the one code path; its test,
`every_app_mailo_included_imports_mailos_json_once`, covers mailo and a shell app). Until the
2026-09-24 mailo gaps wave mailo itself was skipped, so a mailo that called `use_environment`
without importing first started from the defaults; `mail-app/src/appearance.rs`'s own
`load_or_import` call on startup is now redundant and can go. Phase A wires `use_environment`
in directly (mailo is still on the webview, where its Tokio requirement is already satisfied by
`dioxus-desktop`); Phase B does **not** need to change this call at all, only the window it runs
inside of, since `ds_native::launch` enters its own runtime (`CONSUMING.md` section 3).

`Space`'s own file (`spaces.json`, `crates/mail-app/src/space.rs`'s `FILE_NAME`) is **not**
`ds-settings`'s concern — it stays exactly as it is, a mailo-owned JSON file, since `Space` is
mostly mail-domain data (section 3) that quire has no settings schema for. Do not route it
through `ds_settings::load_or_import`; only `appearance.json` moves.

## 5. Phase A — still the webview, quire by path

### 5.1 Files you own exclusively

Everything under `mail-app/src/ui/` and `mail-app/src/` that this brief names below. Nothing
outside `mail-app` changes (`mail-domain`, `mail-mime`, `mail-proto`, `mail-store` are untouched
— `scripts/check-boundary.sh`'s sans-I/O boundary is not this migration's concern).

**Deleted outright** (moved to `ds`, verbatim, with their own tests already in this repository —
do not re-port their logic, only delete mailo's copy and fix the call sites that used it):

- `crates/mail-app/src/palette.rs` (-> `ds::space::palette`, `ds::Dot`, `ds::gradient`,
  `ds::swatch`, `ds::card`)
- `crates/mail-app/src/contrast.rs` (-> `ds::space::contrast`, `ds::ContrastCheck`, `ds::ratio`)
- `crates/mail-app/src/ui/icon/**` (-> `ds::icon`, `ds::Glyph`, `ds::Icon`)
- `crates/mail-app/src/ui/style/tokens.css`, `tokens.dark.css` (-> `ds::stylesheet()`'s own
  token cascade)
- the font pipeline in `crates/mail-app/build.rs` and `crates/mail-app/src/ui/style/mod.rs`'s
  `include_str!(concat!(env!("OUT_DIR"), "/fonts.css"))` line (-> `ds::FACES` +
  `ds::font_face_css` under the `webview-fonts` feature — Phase A is still a webview, so this is
  the one place Phase A needs that feature; Phase B drops it for `ds_native::register_fonts`)

**Rewritten in place** (the file stays, most of its content does not):

- `crates/mail-app/src/view.rs` — delete `Theme`, `Motion`'s **enum body only if you also add
  a `From`/`Into` conversion at every `Space::motion` call site** (see section 3's gap: the type
  itself stays, only re-export where useful), `Marks` stays as-is, `Appearance` narrows to
  whatever fields do not move (likely just `marks` after the move, or the whole struct is
  replaced by reading `ds::Appearance` directly plus a separate `marks: Marks` — pick one and be
  consistent across every call site), `Peek` deleted (`-> ds::PeekMode`, though `ds::PeekMode`
  is `{Center, Full}` per `DESIGN.md` F10 while mailo's `Peek` is `{Side, Center, Full}` — mailo
  keeps `Side` itself, per `DESIGN.md`: "`PeekMode{Center, Full}`... mailo keeps `Side`"; do not
  drop the `Side` variant, wrap `ds::PeekMode` in mailo's own `Peek` that adds it back)
- `crates/mail-app/src/appearance.rs` — `load`/`save`/`config_dir`/`state_dir`/`cache_dir`
  become thin wrappers over `ds_settings::{load_or_import, save, config_dir, state_dir,
  cache_dir}` with `AppName::MAILO`, or are deleted in favour of calling `ds_settings` directly
  at each call site — `WindowDirs` (mailo's own struct bundling config+state) stays, since
  `ds_settings` has no equivalent bundle
- `crates/mail-app/src/space.rs` — per section 3: `Space` keeps `name`/`motion`/`scope`/
  `pins`/`colors`, gains a `look: ds::SpaceLook` field (or four separate fields mirroring
  `SpaceLook`'s — a field wins if `Space` round-trips through `spaces.json` with `#[serde]`
  attributes that would fight `SpaceLook`'s own; check before choosing), `CardAccent` deleted in
  favour of `ds::CardAccent`, `Dot`/`NEUTRAL_DOT` deleted in favour of `ds::Dot`/`ds::NEUTRAL_DOT`
- `crates/mail-app/src/ui/paint.rs` — **deleted in favour of `Ds`'s own frame rendering**, not
  rewritten: `frame_pairs`, `push_palette`, `push_accent`, `grain_opacity`, `look_lines`,
  `token_lines`, `accent_rule`, `appearance_script`, `paint_script`, `SYSTEM_ACCENT`, `NAMES`
  are **all** what `Ds`'s own `FrameVars`/`use_frame_layers` (`crates/ds/src/root/ds.rs`) and
  `ds::FrameVars::of`/`style_attr` (`crates/ds/src/space/frame_vars.rs`) already do, driven by
  `Ds`'s `look: SpaceLook` prop instead of a hand-built `<script>` string. Every caller of
  `paint_script`/`appearance_script` (section 5.1's later bullets name them: `switch.rs`,
  `command/mod.rs`, `space_editor/{mod.rs,hue.rs}`) stops calling `dioxus::document::eval` and
  instead sets the `Space` (or, during a live edit, a signal `Ds`'s `look` prop reads) — the
  cross-fade `Ds` already animates (`root/ds.rs`'s `FrameLayers::show`) is the `Fade::Cross`
  mailo's own `paint_script` hand-rolled; **do not re-implement `Fade` — `Ds` has one fade
  behaviour, not two**, so `space_editor`'s live-drag preview (`Fade::None` today, no transition
  while dragging) needs its own plan: either accept the cross-fade during drag too, or hold the
  preview in a `Surface` override (`theme`, `accent`, `blur`) instead of changing `Ds`'s own `look` until the drag
  ends. Decide and record which in this file's own follow-up notes; this brief does not resolve
  it.
- `crates/mail-app/src/ui/launch.rs` — **keep** `KEEP_FOCUS` and its injection into
  `with_custom_head` through all of Phase A (the webview still needs it to hold the keyboard;
  section 5.3 asks that it still works, and 5.4's grep expects its injection site); it is
  removed in Phase B, where `ds_native::focus` takes over (section 6.1). Delete
  `appearance_head` (calls the deleted `paint.rs::appearance_script`), wrap the launched root in
  `Ds` (`material: Material::Window`, `stylesheet: Inject::Inline` or `Host` — Phase A can use
  either; `Host` plus mailo's own `with_custom_head` avoids a second `<style>` tag if that
  matters to mailo's own head-injection tests), pass `appearance: ds::Appearance` (from
  section 4's settings load) and `look: ds::SpaceLook` (from the current `Space`, section 3)
- `crates/mail-app/src/ui/app.rs` — `App`'s two `dioxus::document::eval("...'.app'?.focus()")`
  calls (`app.rs:339,361`) either stay (focus management is not part of coherence rule 4, which
  is about *motion timing*, not focus) or move to whatever quire ends up offering for focus —
  not resolved by this brief, not blocking; `App` gains whatever plumbing threads
  `ds::Appearance`/`ds::SpaceLook` from `launch.rs` down to `Ds`
- `crates/mail-app/src/ui/switch.rs`, `crates/mail-app/src/ui/command/mod.rs` (the
  `paint_script`/`Fade::Cross` calls at `command/mod.rs:280`), `crates/mail-app/src/ui/
  space_editor/{mod.rs,hue.rs}` — every `dioxus::document::eval(&paint_script(...))` /
  `dioxus::document::eval(&format!(...))` call becomes a write to whatever signal drives `Ds`'s
  `look`/`appearance` props (see the `paint.rs` bullet above for the drag-preview question)
- `crates/mail-app/src/ui/style/mod.rs` — `STYLE` drops `reset.css` (-> `ds::stylesheet()`'s own
  reset, which already keeps `html,body` transparent — see this repository's `self_lint.rs`
  exception for exactly that selector), `tokens.css`/`tokens.dark.css` (deleted per the bullet
  above), and every `include_str!` for a `.css` chunk whose classes moved wholesale to a quire
  component (work through section 2's table file by file: once `shell.css`/`list.css`/
  `menus.css`/`reader.css`/`hover.css`/`composer.css`/`controls.css` no longer define a class
  quire now draws, delete that class's rules; do **not** delete a whole file until it is empty —
  `list.css`, for instance, keeps `.list-bar`/`.list-top-h`/`.list-g` even after `.row`/`.strip`
  leave it). `motion.css` is deleted outright (`ds::stylesheet()`'s own `@keyframes`, coherence
  rule 1: mailo may not declare `@keyframes`, period). The `#[cfg(test)] mod calm; mod
  contrast;` sub-modules move or delete alongside whatever they tested that itself moved.
- `crates/mail-app/src/ui/{row,list,reading/mod,sidebar/mod,sidebar/panes,sidebar/today,menu/
  mod,menus/mod,command/mod,hover/cards,space_editor/parts,compose/pill,compose/props,
  compose/desk}.rs` — every `onanimationend` handler (`row.rs:174,243`, `list.rs:219`,
  `compose/mod.rs:162`, `compose/props.rs:281`, `sidebar/today.rs:100`, `sidebar/panes.rs:
  246,258`) is coherence rule 4's whole point: replace the class-toggle-plus-`onanimationend`
  pattern with `ds::use_motion_timer`/`ds::use_pulse` (`CONSUMING.md` section 5 rule 4 has the
  exact API; `ds::Roster`/`use_roster` for list-row enter/exit specifically — `DESIGN.md`'s
  `motion/{presence,roster,use_roster}.rs` row). This is the single largest behavioural change
  in Phase A; budget real time for it, one file at a time, each with its own before/after test.

### 5.2 Signatures you may not change

Every `crates/ds/src/**` public signature (this repository's own freeze — Phase A codes against
it, same as every other quire consumer); every `mail-domain`, `mail-mime`, `mail-proto`,
`mail-store` public signature (outside this brief's file list entirely — if Phase A seems to
need one to change, it does not touch UI and is out of scope, stop and ask). If a `ds` signature
cannot do what Phase A needs (section 3's `SpaceLook` motion gap is the one already found),
**stop and report it** rather than working around it in mailo — the same rule `ORCHESTRATION.md`
states for every quire consumer.

### 5.3 Done means

- Every file in section 5.1's "deleted" and "rewritten" lists is either gone or no longer
  contains what it is listed as no longer containing.
- `mail-app`'s `Cargo.toml` depends on `ds` (`features = ["lint", "webview-fonts"]`),
  `ds-settings`, by path (`CONSUMING.md` section 1 has the exact shape; mailo does **not** yet
  depend on `ds-native` — that is Phase B).
- `ds::lint::assert_clean` runs on whatever remains of mailo's own `STYLE` constant, in a new
  test in `crates/mail-app/src/ui/style/mod.rs`'s existing test module, `Profile::Strict`,
  every exception named and reasoned (`CONSUMING.md` section 5 rule 1's exact snippet, adapted
  from `examples/consumer/tests/coherence.rs::our_stylesheet_lints_clean`).
- `every_class_on_the_frame_is_styled` (`crates/mail-app/src/ui/style/mod.rs:745`, already
  exists) is updated to concatenate `ds::stylesheet()` into the CSS it checks against — today it
  checks mailo's own `STYLE` alone, and once controls render `ds-*` classes it must fail without
  this change; consider replacing `unstyled_classes` (`style/mod.rs`'s own hand-rolled version)
  with `ds::lint::markup` outright, since it is the same check plus `Rule::RawMarkup`
  (`CONSUMING.md` section 5 rule 2).
- `the_load_bearing_layout_survives` (`crates/mail-app/src/ui/style/mod.rs:309`, already
  exists) still passes unmodified — it checks `.app`'s and `.row`'s `grid-template-columns`
  contain `minmax(0, 1fr)` and `.row`'s `position: relative`, all classes section 6 says stay
  mailo's own. If this test needs an edit to pass, the CSS rewrite broke mailo's grid, not quire
  — fix the grid, do not edit the test to match broken CSS.
- No `onanimationend` handler remains anywhere under `mail-app/src/ui/` (section 5.1's list
  names every current site) — coherence rule 4, and `grep -rn onanimationend crates/mail-app/
  src` in the verify step below is the exact check.
- `KEEP_FOCUS` (constant and injection) and the `MAILO_PROBE`/debug-head machinery in
  `launch.rs` still exist and still work (Phase A is still a webview; `CONSUMING.md`'s "webview does not poll futures spawned from
  render" is why these survive to Phase B, not before).
- Mailo's own existing test suite (`cargo test --workspace` in mailo) passes with no test
  deleted to make it pass, only tests updated where section 5.1 names a rewrite that changes
  what the test was asserting about (e.g. a class name).

### 5.4 Verify

Run in mailo's own repository, in a worktree (never on its `master`):

```bash
cd mail-app && grep -rn "onanimationend\|dioxus::document::eval" src/ui   # expect exactly the
    # sites section 5.1 says survive to Phase B (KEEP_FOCUS's own injection site in launch.rs,
    # and app.rs's two focus-restore evals, which are not motion) — nothing else
cd ..
cargo fmt --all --check
cargo clippy --workspace --all-targets -- -D warnings
cargo test --workspace
./scripts/check-boundary.sh
```

Plus, from `mail-app`'s own test suite: `the_load_bearing_layout_survives`,
`every_class_on_the_frame_is_styled` (updated per section 5.3), and the new
`ds::lint::assert_clean` test all green.

### 5.5 Read first

This file, all of it; `CONSUMING.md` sections 1-5; `design/21-SPACES.md` for `SpaceLook`;
`crates/ds/src/root/ds.rs` and `crates/ds/src/space/frame_vars.rs` (what replaces
`ui/paint.rs`); `crates/mail-app/src/ui/paint.rs`, `space.rs`, `view.rs`, `appearance.rs` in
full before editing any of them (this brief quotes fragments; the files have context the
fragments do not carry, especially `paint.rs`'s tests, which encode exactly what the cross-fade
must still do after the rewrite).

## 6. Phase B — Blitz, `ds-native`

Only starts once Phase A is merged and its gates (section 5.4) are green on mailo's own
`master`. Phase B's whole job is: stop running inside a WebKitGTK webview, start running inside
`ds_native::launch`'s Blitz window, and delete everything that existed only because the webview
could not poll a future spawned from render or measure a mounted element synchronously.

### 6.1 Files you own exclusively

- `crates/mail-app/src/ui/launch.rs` — `dioxus::LaunchBuilder::desktop()` replaced by
  `ds_native::launch(App, ds_native::AppConfig { title: "mailo".to_owned(), width: 1200, height:
  800 })`; `with_custom_head`, `KEEP_FOCUS` (kept through Phase A; here `ds_native::focus`,
  which `launch` provides, takes over holding and placing the focus), `NOTHING_MOUNTED`'s
  head-injection path, and the `MAILO_PROBE`/`probe()` debug-head machinery all deleted — `ds_native::Harness` is the
  verification tool the probe existed to approximate (`CONSUMING.md`'s own citation of
  `ORCHESTRATION.md`'s "run it the way its user would" is exactly why the probe existed; a real
  headless harness is strictly better once it exists, which it now does).
- `crates/mail-app/src/ui/app.rs` — the two `dioxus::document::eval("...'.app'?.focus()")` calls
  need a real replacement now that there is no `document::eval` at all on Blitz in the same
  sense (`ds_native` has no `document::eval` equivalent named in this repository's public API —
  check whether `ds::use_env`'s `modality`/`HostModality` or a direct
  `MountedData::set_focus()` call covers it before assuming a gap; not resolved by this brief).
- `crates/mail-app/src/ui/reading/**` (`mod.rs`, `blocks.rs`, `find_bar.rs`, `found.rs`,
  `image.rs`, `spans.rs`, `table.rs`) — the reader plan, section 7 below.
- `crates/mail-app/src/ui/space_editor/**`, `command/mod.rs` — the drag-preview/`Fade::None`
  question Phase A deferred (section 5.1's `paint.rs` bullet) must be settled by now, since
  Phase B has no webview `eval` to fall back on at all.
- New: `crates/mail-app/tests/native_harness.rs` (or wherever mailo's own test layout wants a
  new integration-test file) — `ds_native::Harness`-driven tests replacing whatever the deleted
  webview-only debug probe (`scripts/live-window.sh`, `MAILO_PROBE`) covered, per
  `CONVENTIONS.md`'s "run it the way its user would": these drive real pointer/keyboard/time
  through a real (headless) Blitz document, the same pattern as this repository's own
  `crates/ds-native/tests/harness.rs`.

### 6.2 Signatures you may not change

Same as Phase A's section 5.2, plus every `crates/ds-native/src/**` public signature. If
`ds_native::launch`'s bare `fn() -> Element` (no captures — `CONSUMING.md` section 3 explains
why `examples/consumer` reads its settings inside `App`'s own body rather than in `main`) is a
real obstacle for threading mailo's `Store`/`Spaces`/`WindowDirs` contexts through (today
`launch.rs`'s `run` passes them as `.with_context(...)` on the `desktop::Config` builder, which
`ds_native::launch` has no equivalent of), **stop and report it** — do not reach for a global
`static`/`OnceLock` to smuggle a `SqliteStore` handle past a function-pointer boundary without
checking first whether `ds_native::launch` is meant to grow a context-injection parameter.

### 6.3 Done means

- `mail-app`'s `Cargo.toml` depends on `ds-native` by path, and the full pinned block
  (`docs/workspace-deps.toml`, copied verbatim, `CONVENTIONS.md` section 11) replaces whatever
  `dioxus-desktop`-specific dependency lines Phase A still had.
- No `dioxus::document::eval` call remains anywhere under `mail-app/src/ui/` except any that
  section 6.1's `app.rs` bullet decides must survive with a named, documented reason (Blitz's
  `dioxus_native` line has no `document::eval` in the sense the webview did — an `eval` call
  that survives Phase B is a bug unless it is calling something that still exists).
- `KEEP_FOCUS` (`launch.rs`) is gone: `ds_native::focus` holds and places the keyboard focus on
  Blitz (`ds::HostModality`/`InputModality` tracks keyboard-vs-pointer); anything it does not
  cover is a documented, reported gap.
- `use_roster`/`use_pulse` drive every list enter/exit and pulse animation Phase A left on
  `ds::use_motion_timer` alone where a roster fits better (Phase A's job was "no more
  `onanimationend`"; Phase B's is "the right quire primitive for each case", which was not
  always obvious without a real Blitz document to test entrance/exit staggering against).
- The reader (section 7) renders through whichever of the two documented options was chosen, and
  the "sandboxed frame · no scripts, no same-origin" guarantee `crates/mail-app/src/ui/reading/
  tests.rs` already asserts (`no_div_between_article_and_iframe`, the sandbox-attribute checks)
  has an equivalent assertion for the new mechanism — the guarantee itself is not allowed to
  regress, only its implementation.
- `crates/mail-app/tests/native_harness.rs` (or equivalent) exists and covers at minimum: a menu
  opens on click and closes on Escape, a hover card appears after its delay and not before, a
  toast hides after its hold, a list row leaving heals the rows below it — the same four cases
  this repository's own `crates/ds-native/tests/harness.rs` names for `ds-native` itself, now
  un-ignored and passing there (`CONSUMING.md` section 9: a floating component opened under
  `Harness` used to panic with a reentrant document borrow, root-caused and fixed within this
  same wave via `ds::geometry::measure`'s `HostMeasure`/`Measured` seam — `examples/consumer/
  tests/coherence.rs::the_menu_opens_and_closes_under_harness` is the regression test). Mailo's
  own equivalents can open a `Menu`/`Popover` and drive it through `Harness::click`/`advance`
  freely; there is no longer a known reason to avoid it.

### 6.4 Verify

```bash
cargo fmt --all --check
cargo clippy --workspace --all-targets -- -D warnings
cargo test --workspace
./scripts/check-boundary.sh
```

Plus every Phase A gate (section 5.4) still green, plus the new `native_harness.rs` (or
equivalent) suite, plus `cargo run -p mail-app` opening a real window that shows a real inbox —
`CONVENTIONS.md`'s "run it the way its user would" applies with full force here: a headless test
suite passing is not the same claim as the window actually painting, and this migration's whole
second phase is a renderer swap, exactly the kind of change that news article warns against
concluding about from tests alone.

## 7. The reader, Phase B

Mailo's reader has two views today (`crates/mail-app/src/ui/reading/mod.rs::ViewSwitch`):
**Reader**, which is `mail_mime::{Block, Document, Span}` already parsed and laid out as normal
DOM (`reading/blocks.rs::MessageView`, no iframe, nothing sender-controlled reaches this path
unescaped) and **Original**, which is the sender's raw HTML in a real sandboxed
`iframe[sandbox=""][srcdoc=...]` (`reading/blocks.rs`, the only iframe in mailo). Only
**Original** is this section's problem — Blitz (`blitz-dom`) is one document per
`DioxusDocument`, with no nested browsing context, so there is no Blitz equivalent of an
`<iframe>` with its own script/style/same-origin sandbox. Two options, neither built by this
wave, both named in the orchestrating plan:

1. **A separate Blitz document painted through a custom widget.** `blitz-paint`'s
   `custom-widget` feature (already in this repository's pinned block, `docs/
   workspace-deps.toml`) lets a component provide its own `peniko`-painted content inside a
   rect of the parent document. Build a second, throwaway `BaseDocument` from the sender's raw
   HTML with its own minimal stylesheet (definitely *not* `ds::stylesheet()` — the whole point
   is the sender's styles do not reach the rest of the window and quire's tokens do not reach
   the sender's markup either), render it offscreen, and paint the result into the widget's
   rect. This is real isolation (two separate documents, no shared cascade, no shared DOM) but
   is new plumbing this repository has not built — `ds-native`'s `headless.rs`/`snapshot.rs`
   already build one throwaway `BaseDocument` for tests and PNG snapshots; that is the closest
   existing code to start from, not a finished answer.
2. **Strip sender styles in `mail-mime` and render inline.** If `mail-mime` can be made to
   guarantee no sender CSS, `<script>`, or dangerous URL scheme survives its own sanitisation
   (check `mail-mime`'s current `SafeUrl`/`Reached`/sanitize pass — `reading/blocks.rs` already
   imports `SafeUrl`, `Reached` from it, so some of this exists) tightly enough that "sandboxed"
   stops needing a real browsing-context boundary at all, Original could render as ordinary
   quire-scoped markup, no second document needed. This is simpler if `mail-mime` can actually
   make that guarantee, and much worse than option 1 if it cannot (a single missed selector or
   URL scheme is now a same-document injection, not sandboxed by a real iframe boundary) — this
   is a security decision, not a rendering one, and needs sign-off from whoever owns `mail-mime`
   before it is chosen, not decided unilaterally by the UI session.

Either way, `reading/tests.rs`'s existing guarantees (`no_div_between_article_and_iframe`, "a
plain-text message claimed a sandboxed frame" never happening, the two sandbox-attribute checks)
need an equivalent assertion against whichever mechanism replaces the iframe — write it before
switching, not after, so the old and new mechanisms can both be checked against it during the
transition.

## 8. What mailo keeps

Its own page layout and grid (`.app`, `.card`), its own containers around migrated components
(`.pins` — the flex/grid row `AccountTile`s sit inside, not the tiles themselves; `.list-bar`,
`.list-top-h`, `.list-g` — list-level chrome around `ListRow`s, not the rows), and its own
per-view chrome under `.reader*` (whatever of `reading/mod.rs`'s own markup is not `MessageView`
itself — the header, the `ViewSwitch`, the find bar's own layout). None of `.app`/`.card`/
`.pins`/`.list-*`/`.reader*` is a quire component's concern; all of it must still pass
`ds::lint::stylesheet()` under `Profile::Strict` (coherence rule 1: no literal colour/duration/
easing/radius/font-size/z-index anywhere, quire tokens only, even in mailo's own layout CSS) and
`ds::lint::markup` (coherence rule 2: every class on every rendered element is either one quire
exports or one this remaining CSS defines).

## 9. Acceptance gates, both phases

1. Mailo's own test suite (`cargo test --workspace` in mailo) — never weakened, never a test
   deleted to make a phase's gate pass.
2. `ds::lint::assert_clean` on whatever remains of mailo's own stylesheet,
   `Profile::Strict`, every exception named and reasoned (section 5.3).
3. `ds::lint::markup` (or `every_class_on_the_frame_is_styled`, updated to call it) on an SSR
   render of the real app, zero `Rule::UnstyledClass` and zero `Rule::RawMarkup` offences beyond
   named, reasoned exceptions.
4. `the_load_bearing_layout_survives` (`crates/mail-app/src/ui/style/mod.rs:309`) green,
   unmodified, both phases — it is mailo's own regression test for a CSS Grid overflow bug, not
   something this migration should ever need to touch, and if it needs touching the migration
   broke something real.
