# 34 - Modern look: Sonoma/Sequoia base, then Tahoe shape

Status: spec, 2026-10-07. Docs only; no CSS or Rust changed by this document. Supersedes the
"small, uniform radii" paragraph of design/30 section 3.2 and the Mac 10.x metrics of
design/27 where they conflict (design/27 stays the HIG audit; this is the delta).

Confidence marks on every number: **H** Apple primary (HIG page JSON, AppKit docs, WWDC25
transcripts fetched 2026-10-07), **M** reliable secondary (Apple design-kit or screenshot
measurement as widely reported, or this spec's own derivation from an H rule), **UNKNOWN** no
source found; the number is a proposal. Where I could not reach Apple's Sketch/Figma kits (they
are behind a login) the Tahoe heights and radii are M-low or UNKNOWN and say so: section 6 has
the one check to do before step 2 lands.

## 1. Decision and scope

Owner decision, 2026-10-07: quire reads like macOS 10.x. Move the one Look in two steps; no
Look switch, token-level so quire, sill, detent, mailo and anyview move together.

1. **Step 1, Sonoma/Sequoia (macOS 14-15) base.** Inset grouped rounded lists; borderless filled
   buttons and fields (no 1 px grey hairline); larger radii and spacing; a hero icon above a bold
   title in alerts and sheets; bigger titles; translucent panel materials.
2. **Step 2, Tahoe (macOS 26) shape traits.** Taller, rounder controls at every size plus an
   extra-large size; capsule prominent and toolbar buttons; larger window corners with
   concentric inner radii; icons beside menu items, rounder menus; no hairline under the toolbar
   (scroll-edge fade instead); sidebar as an inset rounded panel; transparent menu bar.
3. **Motion pass (section 7).** Audit of every animation; what modern macOS does instead.

In scope: shape, size, spacing, type, separators, shadows, materials-as-flat-translucency.

**Out, always: Liquid Glass.** No glass, refraction, lensing, specular morphing, no "glass"
bezel style, no morphing from the invoking button. The translucent panel materials of step 1
are the existing blur-and-tint `Material` recipes (flat, no refraction), as Sonoma used.

Not changed: colour roles, accent handling, icon grammar (design/08), the focus ring, the
disabled .35, Reduced-motion rules.

## 2. Token table

Current values read from `crates/ds-style/src/tokens/*.rs` and `css/*.rs` at origin/master
361d9bf2. "Step 1" and "Step 2" are the values after that step; blank means unchanged.

### 2.1 Radii (`tokens/shape.rs`, `Radius::value`)

| Token | Old | Step 1 | Step 2 | Source / confidence |
|---|---|---|---|---|
| `--r-window` | 10 | 10 (real windows are the host's) | 26 toolbar windows; 16 titlebar-only | 26: M (widely measured Tahoe window radius; forum/Tidbits reports "Tahoe 26, Big Sur..Sequoia 10"). HIG/WWDC25: larger radius for windows with toolbars, smaller for titlebar-only: H (qualitative). 16: UNKNOWN |
| `--r-panel` | 10 | 12 | 20 | sheets/popovers/alerts. 12: M (Sonoma sheet ~10-12). 20: UNKNOWN, rule below |
| `--r-card`, `--r-tile`, `--r-media` | 10 | 12 | 12 | inset grouped group radius: M (System Settings groups measured ~10-12; iOS grouped 10-12) |
| `--r-btn` | 5 | 6 | by size, see 2.2 | Sonoma push button ~5-6: M |
| `--r-field` | 5 | 8 | by size | Sonoma search/field 6-8: M; Tahoe fields follow control size: H (WWDC25 "rounded rectangle mini..medium, capsule large/XL") |
| `--r-menu` | 8 | 10 | 14 | Sonoma menu ~10: M; Tahoe rounder menus: H qualitative, 14 UNKNOWN |
| `--r-menu-item` / `--r-shell-highlight` | 8 / 5 | 6 | concentric: `--r-menu` - inset 6 = 8 | derived, rule R6 (design/29) and WWDC25 "concentric shapes subtract padding from the parent's": H for the rule |
| `--r-chip`, `--r-item`, `--r-small` | 6 | 6 | 8 | sidebar item 6 -> 8 with inset sidebar: M-low |
| `--r-tiny`, `--r-micro` | 4 / 3 | 5 / 4 | 6 / 5 | provider marks follow tile rule 25 % of side (below) |
| `--r-pill` | 999 | | | capsule = height/2: H (WWDC25 "capsules use half the height") |
| new `--r-icon-tile` | - | 6 (24 px tile) | 6 | System Settings 24-ish px tile, ~25 % radius: M |
| new `--r-group` | - | 12 | 12 | alias of card for a form group |

**Concentric rule (new, normative):** inner radius = outer radius - inset to that outer edge,
floored at 4 (`SizeScale::inner_radius` already does this). Window 26 with 8 inset puts a
panel at 18; with 12 inset a group at 14. A capsule's radius is height/2, never a token.

### 2.2 Control heights per `ControlSize` (`tokens/control_size.rs`)

The current ladder 16/19/22/28 (font 9/11/13/15) is the **Sonoma ladder** (H: AppKit standard
heights mini 16, small 19, regular 22, large 28 are long documented), so step 1 does not change
the heights; step 1's button change is the look (borderless fill, section 3.1) and that sheets,
alerts and form fields use Large 28 where a row or hero layout calls for it (M).

Step 2 (Tahoe): Apple says "mini, small and medium controls are now slightly taller", large and
new extra-large are capsules, mini..medium stay rounded rectangles (all H, WWDC25 sessions 310
and 356). AppKit gained `NSControl.ControlSize.extraLarge` in macOS 26 (H: AppKit symbol page).
The numbers below are **M-low/UNKNOWN**: I could not reach the Figma/Sketch kit.

| Size | Old h / r / font | Step 2 h | Step 2 radius | Font | Pad-x | Glyph |
|---|---|---|---|---|---|---|
| Mini | 16 / 4 / 9 | 20 | 6 | 10 | 8 | 12 |
| Small | 19 / 5 / 11 | 24 | 7 | 11 | 10 | 14 |
| Regular | 22 / 5 / 13 | 28 | 8 | 13 | 12 | 16 |
| Large | 28 / 5 / 15 | 32 | capsule 16 | 15 | 16 | 18 |
| **ExtraLarge (new)** | - | 40 | capsule 20 | 17 | 20 | 20 |

`ControlSize` gains `ExtraLarge` (additive; `Word` order puts it last; every `match` on it
needs an arm, see 4). New `--ctl-*-xl` tokens. Slider track/knob, switch, checkbox, spinner
follow: switch Regular 22 -> 26 high (38 -> 45 wide by the 1.73 rule R4), knob 24; checkbox
14 -> 16; slider knob 20 -> 22 (all M-low, the same "slightly taller"). Sidebar rows
(`SidebarSize` 24/28/32): unchanged in step 1; 28/32/36 in step 2 (M-low).

### 2.3 Rows, tiles, avatars (`tokens/row_scale.rs`)

| Token | Old | Step 1 | Step 2 | Source |
|---|---|---|---|---|
| compact row (table, source list) | 24 | 28 | 32 | table row Sonoma 24 for sidebar/compact, 28 medium: M; Tahoe +4: M-low |
| `settings_height` (inset grouped row) | 44 | 44 (content-fitting, min 44) | 48 | System Settings grouped rows: M; 44 is the HIG minimum comfortable target: M |
| `avatar` | 34 | 32 circular | 32 | Settings account avatar: M |
| new `icon_tile` | - | 24 (rounded, `--r-icon-tile` 6) | 28 / r 7 | System Settings tile ~24: M (owner asked "20-ish") |
| row gap tile -> label | 8 | 12 | 12 | M |
| group header (section title) | - | 13/600 secondary, 8 above, 6 below | | M |

**Separator inset rule (new):** a row separator is 0.5 px (`--hair`) `--line`, starts at the
leading edge of the row's text (row pad 14 + tile 24 + gap 12 = 50, a row without a tile
starts at 14) and runs to the trailing edge of the group; none above the first row or below
the last; none against a selected row. M (System Settings measured). Tables and the
sidebar keep no row separators.

**Chevron rule (new):** a row that opens a pane/sheet/popover has a trailing `ds-ic`
chevron.right (12 px, 600 weight, `--ink-faint`); a row that only toggles or picks does not.
H for the principle ("lists/disclosure indicator", HIG), size M.

### 2.4 Spacing (`tokens/spacing.rs`)

Scale today ends 16, 18, 22, 26, 36. Additive tokens: `--s-20`, `--s-24`, `--s-28`, `--s-32`,
`--s-40`, `--s-48` (keeps R7's 4 px grid; the test "the scale ascends" holds when inserted in
order). Then the layout numbers: window content margin 20 (was 12-16), group-to-group gap 20,
form row inner pad 14 x 0, panel pad 20 (was 12-14), alert/sheet pad 24 (was 22/18). M.

### 2.5 Type (`tokens/type_scale.rs`)

macOS built-in styles, H (HIG typography JSON, 2026-10-07): Large Title 26/32, Title 1 22/26,
Title 2 17/22, Title 3 15/20, Headline 13 bold, Body 13/16, Callout 12/15, Subheadline 11/14,
Footnote 10/13. The quire ramp already holds 13/15/16/20/21/22/24/26; what changes is *use*:

| Role | Old | New | Source |
|---|---|---|---|
| pane / settings page title | 16-21 | 26/700 (Large Title; 22/700 inside a sheet-sized pane) | H |
| sheet title (`.ds-sheet`) | 16/700 | 20/700 | M (between Title 2 and Title 1) |
| alert title (`.ds-alert-title`) | 16/700 `--fs-title` | 17/700 (Title 2 emphasised) | H size, usage M. Sonoma's own alert uses 13 bold; the owner asked for bigger |
| alert body | 13, ink-soft | 13, ink-soft (Body) | H |
| section header over a group | 11/600 | 13/600 secondary | M |
| row title / detail | 13 / 11.5 | 13 / 11 (Subheadline) | H |
| min size | 10 | 10 | H footnote |

New `--fs-title-2` 17 and `--fs-large-title` 26 aliases are not needed: `--fs-title` (16) is
retuned to 17 (an `FontSize::Title` value change, affects every user, see 4) and `--fs-display`
(26) already exists. **Flag:** `FontSize::Title` 16 -> 17 moves every consumer; prefer adding
`FontSize::Title2` 17 and moving only alert/sheet to it.

### 2.6 Separators, shadows, materials (replace borders)

| Token | Old | Step 1 | Step 2 |
|---|---|---|---|
| `--hair` borders on button/field/list | 1 px `--line` | **removed**; fill only | removed |
| `--shadow-1` (raised control) | inset white + 0 1 2 rgba .1 | none for filled buttons (flat fill, M); kept for knobs | |
| `--shadow-pop` | menu/popover | 0 0 0 .5 rgba(0,0,0,.12), 0 10 30 -8 rgba(0,0,0,.28) (M: Sonoma menu = hairline + soft shadow) | same |
| `--shadow-sheet` | | 0 0 0 .5 rgba(0,0,0,.14), 0 24 60 -12 rgba(0,0,0,.35) | same |
| panel materials | opaque `--surface`/`--raise` | `Material::Panel`: tint alpha .72-.8 + 30 px blur (`ds-style/material/`); group fill stays opaque `--grp` (white / #2c2c2e dark) so rows read | |
| fills | `--surface-2` grey | filled button fill = `--fill-quaternary`-tier wash (rgba(0,0,0,.06) light, rgba(255,255,255,.1) dark); field fill = same | |
| toolbar hairline | 1 px under toolbar | | none; scroll-edge fade: 20 px gradient of window colour -> transparent over the content, hard variant for macOS (H: WWDC25 356 "hard ... mostly used on macOS") |

The blur uses the existing Blitz-supported path (design/03 and `blur.rs`); if a platform has no
blur the tint alpha rises to .92 (existing fallback).

## 3. Per component: current -> target

File:line is origin/master 361d9bf2.

### 3.1 Button (`ds/src/components/controls/button.css`)

Now: `.ds-button` L13-20: height `--btn-h` (22), `border:var(--hair) solid transparent`;
push L28-: `background:var(--surface-2); border-color:var(--line); box-shadow:var(--shadow-1)`;
default (`data-answers=return`): accent fill; toolbar: no bezel, hover fill `--surface`; inline:
words only, hover wash; help: round bezel with line.

Step 1: push = `background:var(--fill-btn)` (quaternary wash), `border:0`, no shadow, radius
`--btn-r` (6), Regular pressed = darker wash; prominent/default = accent fill, white ink,
no border, no shadow; Large size (28) used in sheets and alerts. Toolbar and inline keep
"no bezel at rest". Help = filled circle, no line. Disabled .35 unchanged.

Step 2: radius per 2.2; **Large/XL are capsule**; prominent (default) buttons are capsule at
every size when standing alone ("prefer capsule for a button by itself", H, HIG buttons page;
rounded rectangle in a vertical stack, capsule in a horizontal row, H); toolbar buttons are
capsules with an 8 px hover fill pill, 32 high (M-low); toolbar groups share one capsule
background. `data-image=only` square becomes a circle (radius = h/2). Button pad-x per 2.2.
API: none; a `ButtonShape` is derived (rounded rect for Mini..Regular, capsule Large/XL, an
override attribute `data-shape=capsule|rounded` additive).

### 3.2 TextField / SearchField (`fields/text_field.css`)

Now: frame L33-36 `background:var(--surface); border:var(--hair) solid var(--line);
border-radius:var(--tf-r)`; search L40 uses `--fill-quaternary` with transparent border;
focus L44 border `--accent-text` + ring `--ring --accent-soft`; invalid border `--danger`.

Step 1: bezeled field = fill only (`--fill-quaternary`), no border; focus keeps the accent
ring (H: focus ring is system behaviour), invalid shows a 1.5 px `--danger` ring (inset
shadow, no layout change); radius 8. **Inside a Form row** the field is bare: no fill, right
aligned text, as in System Settings ("variant=plain" inside `Form`). Search field: capsule
(Tahoe) / r 8 (Sonoma).
Step 2: heights and radii by 2.2 (Regular 28 / r 8; Large 32 capsule); search is a capsule at
every size in toolbars (M).

### 3.3 PopUpButton (`menus/pop_up_button.css`)

Same as push button: filled, no border, chevron.up.chevron.down 10 px trailing glyph, Regular
28 in step 2. Its menu is 3.6.

### 3.4 Toggle / Slider / Checkbox (`controls/toggle.css`, `slider.css`, `checkbox.css`)

Toggle: track loses the inset hairline (L24 `box-shadow:inset 0 0 0 var(--hair) var(--line)`)
and takes the fill wash off, on = accent; knob white with `--shadow-1` kept. Step 2 size per
2.2 (26 high). Slider: track 4 -> 6 px, capsule; knob white with shadow, no ring. Checkbox: 14 -> 16
in step 2, radius 4 -> 5, no border when checked. All values M-low. Toggle's spring stays
(motion section).

### 3.5 List, Row, Table (`lists/list/list.css`, `row/row.css`, `table/table.css`)

Now: `ds-list[data-style=inset]` L25-27: 4 px pad, `--r-card`, `--surface`, 1 hair line above
every row after the first; `.ds-row` L8-: 24 min height, 8 pad, `--r-shell-highlight`;
`data-density=settings` 44; leading `[data-leading=text]` is a bordered square (`--r-tiny`
effectively); Table: header 24 with a `--hair` bottom line, rows 24.

Target: **new list style `data-style=grouped`** (the System Settings inset grouped list) and
`Form`/`Section` wrappers (section 4 API):

- group: `margin:0 var(--s-20)`, `background:var(--grp)`, `border-radius:var(--r-group)` 12,
  overflow hidden, no border, no padding; consecutive groups 20 apart; a section header above
  (13/600 secondary, 8 below the previous group's bottom edge + 6 above) and an optional
  footer (11, faint).
- row: `min-height:44` (48 step 2), pad `0 14`, gap 12, leading `icon_tile` 24 (rounded 6, per-app
  gradient and white glyph, no outline: design/08) or circular avatar 32; detail text trailing,
  chevron per 2.3 rule; separator per 2.3 (`::before` positioned `left:50px`).
- selection inside a grouped list is not a highlight fill: a tap/press shows `--fill` wash
  on the row (rounded to the group's corners on first/last). Source-list and table
  selection stay accent.
- the old `inset` style stays one release as an alias of `grouped` (no break: same attribute
  accepted).
- **stays a table**: `Table` (multi-column, sortable, resizable) and the `source-list`/sidebar
  rows. Table changes: header 24 -> 28 with no bottom hair (a 0.5 px line appears only after
  scrolling, scroll-edge hard variant), row 28 (step 2: 32), selected row a rounded 6 inset
  highlight (already `--r-shell-highlight`); `data-style=bordered` alternating rows unused.
- tiny provider mark `[data-leading=text]` becomes an `icon_tile` (filled, no border).

### 3.6 Menu (`menus/item/item.css`, `overlays/popover.css`, `menus/menu`)

Now: item L11-12 `min-height:var(--shell-menu-row)` 22, pad `0 8 0 2`, highlight radius 8 token
(5 per shell), image column 22 hidden when none; popover L52-53 `border:var(--hair) solid
var(--line); border-radius:var(--r-menu)` 8; `--shadow-pop`.

Step 1: menu radius 10, border -> `--shadow-pop` hairline (shadow), pad 5 around the items, row
22 -> 24, highlight r 6, translucent material (blur, tint alpha .8). Group headers 11/600.
Step 2: row 24 -> 28 (M-low), highlight r = menu r - pad = 14 - 6 = 8 (concentric), image
column **filled when the menu has any icon** (symbols beside items; H: WWDC25 356 "populate
menus with symbols ... use the symbol once to introduce a group"); font 13; menu r 14.
The menu bar (sill `menu_bar`): transparent background (no fill, text on the wallpaper with a
subtle scrim in dark) in step 2 (H qualitative: Tahoe menu bar transparent).
`menus.item_height_px` setting default moves 22 -> 24 -> 28 (key keeps its meaning).

### 3.7 Alert (`overlays/alert.css`, `alert.rs`, `alert_model.rs`)

Now: L14 `.ds-alert` pad `22 22 18`; icon optional 48 (L15, `margin-bottom:12`); title L16
display font 16/700; body 13 ink-soft; footer gap 8, buttons Regular 22; sheet width Narrow 340.
So an icon slot exists but consumers (ConsentAlert) pass none.

Target (hero layout):
```
 width 340 (narrow) | pad 24 24 20 | radius 12 (step 1) / 20 (step 2)
 [ hero icon 64 x 64, 15 radius app-icon tile, 12 below ]
 Title   17/700, centred, max 2 lines, line-height 1.25
 Body    13/400 ink-soft, 6 below title, centred
 [ accessory (account pop-up) 14 above buttons ]
 footer  18 above; row of 2: equal width, gap 8, Large 28 (step 1) / 32 capsule (step 2),
         default (accent, capsule) on the right; 3+: stacked, gap 8, default on top
```
`AlertIcon` slot defaults to a 64 px hero (was 48); a consumer that has an app or provider
icon passes it; with none the slot collapses (no placeholder). The hero is the *requesting
app's* icon (consent) or the provider's icon (sign-in); error and destructive alerts use the
app icon with a 20 px badge at the lower right (warning glyph) - M (macOS Sonoma alerts do
exactly this). Danger button: red label at rest on the filled button (not a red fill).

### 3.8 Sheet (`overlays/sheet.css`, `sheet.rs`)

Now: `.ds-sheet` L12 `width:min(560px,88%)`, hanging from the top edge, bottom corners only
`0 0 var(--r-panel) var(--r-panel)`, `--shadow-sheet`, `animation sheet-in` (slides down 400 ms).

Step 1: a floating card: centred horizontally, 12 px (`--s-12`) below the toolbar/titlebar
line, all four corners `--r-panel` 12, opaque-ish material (alpha .92) so the content under
it stays dimly present, `--shadow-sheet`; width 420 default form, 340 narrow, 560 wide stays;
pad 28 28 22; title block: hero 56 (circular provider avatar or app tile), 20/700 title,
subtitle 13 soft; fields as a grouped `Form` (3.5); footer right-aligned (cancel left of
default), Large buttons. Motion: section 7 (the slide-from-top is dropped).
Step 2: radius 20-26 (concentric to the 26 window: 26 - 8 inset = 18; spec 20 flat).
`Attach::Within|Window|Centre|Bottom` unchanged in API; the visual meaning of `Window`/`Within`
becomes "card hung 12 below the top edge, no flat top", bottom stays a bottom sheet with
top-rounded corners.

### 3.9 Window frame, Titlebar, Toolbar (`chrome/window_frame.css`, `chrome/toolbar/toolbar.css`, `tokens/chrome/scale.rs`)

Now: titlebar 28 (`titlebar_height`), lights 12 / gap 8 / inset 13 (M: these are the
Sonoma lights: 12 px, 8 gap, 20 from the left edge; quire has 13, small drift), title 13/600
centred; toolbar below with a hairline; `--r-window` 10.

Step 1: `titlebar_height` 28 -> 28; window content margin 20; no behaviour change besides the
radius being the host's (cosmic-comp/casement draw the window shape).
Step 2: lights 12 -> 14 px, gap 8 -> 10, inset 13 -> 20 (M-low: Tahoe lights are larger and
sit lower in a 52 px toolbar); toolbar 52 high (with 32 buttons centred, 10 vertical pad;
M-low) and the titlebar-only window 28; **no toolbar bottom line**; content scrolls under
the toolbar with a hard scroll-edge fade 20 px (H qualitative; 20 px UNKNOWN); window radius
26/16 per 2.1. The chrome tokens are shared with a compositor ("so the two cannot drift"), so
`CHROME_SCALE` changes need the compositor lane informed: breaking for casement and the
cosmic-comp fork - flag in 4.

### 3.10 Sidebar (`chrome/sidebar.css`, `chrome/split_view`)

Now: full-height `--surface` ground, pad 10, items `--r-item` 6, rows 28; selection accent.
Step 1: translucent material (vibrancy), pad 12, row 28, selection accent r 6 (Sonoma
sidebar is full-height, not inset: M).
Step 2: **inset rounded panel**: 8 px margin from window edges (top below the lights), radius
= window 26 - 8 = 18 (concentric), its own translucent panel material, content scrolls
beneath (H: "sidebars are inset", WWDC25 356), row 32 (r 12? no: r = 18 - 8 = 10), selection a
filled rounded rectangle r 10 in the accent wash (not full accent) - M-low. A new
`data-inset=true` attribute on `.ds-sidebar` plus a split_view gutter change (the gutter
hairline goes away).

### 3.11 Popover (`overlays/popover.css`)

Now: `--raise` + `1 px --line` + `--r-menu` 8, arrow 34x8 with the line on two edges (L47-49,
56-63). Step 1: r 12, no border, `--shadow-pop`, arrow kept (HIG popovers have the arrow) but
borderless: arrow fill = surface only. Step 2: r 20, same. Popover pad 16 (was 12).

### 3.12 Others (follow tokens, no separate spec)

Segmented control, Stepper, Chip, Badge, Tooltip, HoverCard, Toast take radii/heights from
2.1 and 2.2 and lose their hairlines the same way; Toast is a capsule-ish r 16 panel material
(step 2).

## 4. Consumer impact

Goldens in quire (all must be re-recorded in the lane that changes them, never mixed):
`crates/ds/tests/snapshots/stylesheet.css` (2664 lines) and
`crates/ds-shell/tests/snapshots/stylesheet.css` (3591 lines): every step changes them;
ds-gallery `--snapshot` images; ds-conformance size-rule tests
(`tokens/size_rules_tests.rs`, ladder values), `ds-style` token tests, `ds-lint` CSS rules
that name tokens.
Consumers: sill (`launcher_card.rs`, `osd.rs`, `switcher.rs`, `screenshot_thumb`, battery
snapshot, any test asserting a px rect of a bar/menu/row; grep `row_h|menu_row|titlebar`),
detent (`crates/detent-ui/tests/snapshots.rs` - the settings window is the grouped-list
poster child), anyview (`anyview-ui/tests/snapshots`, `anyview-peek/tests/golden.rs`,
`snapshots/pane`), mailo (`mail-app/src/ui/fixtures`; its mail list rows keep Table/List
density, so only chrome moves). Run `cargo test --workspace` in each after a quire pin move.
Per memory: **batch the quire release and warn the consumer sessions first**; each move costs
anyview/mailo a re-pin and can break porter.

API: additive unless noted.
- `ControlSize::ExtraLarge` (additive variant; **breaking for exhaustive `match` in consumers**;
  mitigate by `#[non_exhaustive]`-free grep in sill/detent/mailo/anyview first).
- `RowScale` gains `icon_tile`, `compact_height`; `ROW_SCALE` const fields are public:
  adding fields breaks struct-literal users (only quire constructs it).
- `CHROME_SCALE` values change in step 2 (compositor-visible; **breaking for casement/cosmic-comp
  lane**: coordinate).
- `SpacingToken` gains S20..S48 (additive).
- `FontSize::Title` retune is value-breaking for every consumer: use new `Title2` instead.
- New components: `Form`, `Section`/`FormSection`, `IconTile`, `AlertHero` (or an `icon` size on
  the existing `Alert` slot). Old `data-style=inset` stays as an alias.
- Settings keys unchanged; their defaults move (menu row, titlebar), which only affects
  profiles that never set them.

## 5. Implementation lanes

Rule: a lane owns files; shared token files (`ds-style/tokens/*`) go to Lane A only, per
step; others read tokens.

Step 1 (Sonoma):
- **Lane 1A tokens** (ds-style): `tokens/shape.rs`, `spacing.rs`, `row_scale.rs`,
  `elevation.rs`, `css/materials_css.rs`, `type_scale.rs` (add `Title2`), colour fill tokens
  (`tokens/colour.rs`, `--fill-btn`, `--grp`). Verify:
  `cargo test -p ds-style` and `cargo run -p ds-gallery --release -- --snapshot out/` (compare
  with `before/`).
- **Lane 1B controls** (ds/components/controls, fields, menus/pop_up_button, menus/item,
  overlays/popover.css): button, text field, toggle/slider/checkbox, popup, menu, popover.
  Verify: `cargo test -p ds --test stylesheet` (re-record golden: `cargo insta` / the repo's
  `UPDATE_GOLDENS=1`, see design/30 README) plus gallery Controls and Menus pages.
- **Lane 1C lists + forms** (ds/components/lists, new `forms/`): `grouped` list style, Form,
  Section, IconTile, separator inset, chevron rule, Table header/row. Verify:
  `cargo test -p ds lists` and gallery Lists page; detent snapshot in a scratch pin.
- **Lane 1D overlays** (ds/components/overlays alert/sheet/empty_state, ds/components/app
  consent pieces): hero alert, floating sheet card, accounts sheets (design/31), motion edits of
  section 7 for sheet/alert. Verify: gallery Accounts and Overlays pages vs
  `~/rs-wt/accounts/shots-w2f/` as BEFORE.
Merge order: 1A -> (1B, 1C, 1D in parallel, each rebased on 1A) -> consumer re-pin lanes.
Gate in a gate worktree (memory: merge in ~/X-wt/gate, ff master only on GATE GREEN).

Step 2 (Tahoe): same four lanes, files as above plus:
- **Lane 2A tokens**: `control_size.rs` (ExtraLarge, new heights), `size_scale.rs` (capsule
  rule R2 per size), `size_vars.rs` (`-xl` tokens), `chrome/scale.rs`, `shell_type.rs` (menu
  row). Verify: `cargo test -p ds-style` (size_rules_tests updated to the new ladder).
- **2B controls/menus** (capsule buttons, icons in menus, menu radius); **2C chrome**
  (window_frame.css, toolbar.css, sidebar.css, split_view, scroll-edge fade, transparent menu bar
  in `ds-shell/bar`); **2D overlays** radii, concentric sheet/alert. Verify per lane as above
  plus `cargo test --workspace --all-features` and the sill gate.
Step 2 starts only after step 1 is merged and the consumers are re-pinned and green.

## 6. Owner decisions (settled 2026-10-07)

The owner took every recommended default below; they are decisions, not open questions.

1. **Tahoe control heights are not confirmed.** Apple publishes none in text; the 20/24/28/32/40
   ladder is my best M-low estimate. Recommended default: build step 2 on it and verify against
   the Apple Design Resources macOS 26 kit once (needs a developer-account download) before the
   lane merges; the token table makes any correction one file.
2. **Window corner radius** 26 (toolbar windows) and 16 (titlebar-only, UNKNOWN): the compositor
   draws windows, so does casement/cosmic-comp own these? Recommended: yes, expose
   `--r-window` as the token both read.
3. **Alert title size**: Sonoma's own alert title is 13 bold; the owner asked for bigger. Default
   17/700.
4. **Sheet entry**: keep slide-from-top (macOS AppKit still does it, M) or the fade-and-scale
   card (section 7)? Default: card, 160 ms; the owner's observation is that slide is rare.
5. **Per-app hero icon source**: apps pass their icon, or quire resolves it from the app id?
   Default: the caller passes it; quire only lays it out.
6. **Menu icons**: mandatory for built-in menus or optional? Default optional; quire's own menus
   (sill, detent) get symbols in step 2 per HIG "where it helps recognition".
7. **Keep the `inset` list style name?** Default: alias for one release, then remove.

## 7. Motion

Sources: HIG Motion (2025, H): add motion purposefully; "avoid adding motion to UI interactions
that occur frequently"; brief and precise; let people cancel motion; make motion optional.
WWDC23 "Animate with springs" (H, fetched): duration + bounce; default spring has bounce 0
("when you're not sure, use bounce 0"); bounce ~0.4 at most and for "the end of a gesture";
interruption keeps velocity. WWDC18 "Designing Fluid Interfaces" (H, fetched): start with 100 %
damping (no overshoot), bounce only when the gesture has momentum (80 % damping), constant
redirection/interruption, respond instantly.
Sheets: the claim "Big Sur made sheets centred floating cards with fade+scale" is **not
confirmed**: the current HIG Sheets page (2025 JSON) says nothing about entry motion; a
secondary source says Big Sur sheets are "vertically centred" (M-low) while AppKit documentation
and tutorials still describe window sheets sliding from the title bar (M, pre-Big Sur and
undated for 14/15). What I can state: Big Sur+ sheets are floating rounded cards not flush
with the toolbar (M, visual), and Tahoe's sheets morph from the invoking control with Liquid
Glass (excluded). So the recommendation below is a product choice that matches the
direction, not an Apple-verified behaviour; flagged as question 4.

The engine already follows WWDC18: `SpringSpec::for_touch` gives critical damping for a tap, key or remote
and 0.8 only when a hand let go with momentum (`ds-motion/src/spring_spec.rs`), and
Reduced maps every moving duration to `--t-quick` (`timing.rs`). Tokens now: `--t-quick` 150,
`--t-move` 250, `--t-big` 400; easings `--e-out` (.22,.9,.3,1), `--e-exit` (.55,0,.75,.2),
`--e-in-out` (.42,0,.58,1), spring responses 300/450.

Target tokens (token-only change unless stated): `--t-quick` 150 -> 120, `--t-move` 250 ->
200, `--t-big` 400 -> 280; `--e-out` keep; `--e-exit` -> (.4,0,1,1) with the exit never longer
than .6x the entrance; spring `Quick` 300 -> 260, `Move` 450 -> 380, damping rules kept.
(Durations: M, in the range of AppKit's 0.2-0.25 s default animation durations; H for the
principle "brief".)

| Animation (keyframe / site) | Now | Verdict | Target (what modern macOS does) | Change kind |
|---|---|---|---|---|
| `sheet-in/out` (sheet.css L18-19) | slide from above the top, 400 / 250 ms | **drop** | fade + scale .97 -> 1 with 12 px settle, 200 ms `--e-out`; exit fade 120 ms; window under it stays | component: new keyframes `sheet-card-in/out`, `Anim::SheetIn` mapping, `sheet_attach.rs` offsets |
| Bottom sheet (`--sheet-dy`) | rise from below | keep | iOS-style bottom sheet is only for narrow roots; 280 ms spring bounce 0 | token |
| Alert (`alert.css`, inherits sheet) | slides | **simplify** | alert appears in place: fade + scale .96 -> 1, 160 ms; no exit slide (fade 100) | component (follows sheet keyframes) |
| `peek-in` | scale .95 + 12 px rise | **simplify** | fade + scale .98, 180 ms (command palette / Spotlight style: fade only 120 ms) | keyframe edit |
| `menu-in` | scale .94, -6 px | **drop** | macOS menus open with no animation; close fades 100-150 ms (M). Open: instant (0 ms) or 80 ms fade; keep `menu-out` at 120 ms | keyframe edit |
| Popover `fade` | 150 ms | keep | fade 120 ms (+ scale .98 optional) | token |
| `panel-in/out` (edge panel, toast, banner, thumbnail) | slide from past the right edge 250 / 150 | keep | Notification Center banners and panels do slide from the right edge in Sonoma/Tahoe (M); spring bounce 0, 280 ms in, 200 out; `out` keeps swipe | token |
| Toast / banner | `panel-in` | keep | as above | token |
| Window open/close | (host's compositor) | out of quire | macOS: scale .95 -> 1 + fade, ~200 ms; lights fade | casement lane note |
| `row-in/out`, `heal` | fade + 8 px slide, 250/150 | **simplify** | NSTableView `.effectFade` only: fade 150 ms, rows below close with a spring (bounce 0, 260) | keyframe edit (drop the translate) |
| `slide-r/l`, `pane-out-*` (pane push) | 26 px + fade, spring | keep | matches NavigationSplitView push (H-ish, M); spring bounce 0 | token |
| `page-in` | 10 px + scale .985 | **drop** | fade 150 | keyframe edit |
| `morph-in/out` (glyph swap) | scale .7 | simplify | symbol-effect replace: scale .9 + fade, 150 ms; keep `MorphInSpring` for a press only | keyframe edit |
| `fade`, `fade-in`, `osd-out` | 150-250 | keep | 120-150 | token |
| `shake`, `shake-x` | 420 ms, 7 px | keep (rare, error) | macOS login shake 4 oscillations, ~300 ms; keep | none |
| Toggle (spring in Rust, track fade 150) | spring + tint | keep | NSSwitch ~250 ms, no overshoot when tapped, slight when dragged (matches WWDC18) | none |
| Button/row/field `transition` colour fades (32 CSS files, 150 ms `--e-in-out`) | | **simplify** | hover/press feedback in Mac is instant or <=100 ms: use `--t-quick`(120) `--e-out` for hover-in, instant for press-down (press follows the pointer, WWDC18) | token (+ drop `transition` on `:active`/`data-pressed`) |
| Hover scale (`space_editor.css` 1.1-1.15, `ds-row[data-drop=target]` scale 1.045) | | **drop** hover scale, **keep** drop-target lift | HIG: no motion for frequent interactions; Mac has no hover growth | component CSS |
| Table column hover colour | | keep | | none |
| Focus ring | instant | keep | | none |
| Spinner | 83 ms step hold | keep | | none |
| Orb, idle dim, send-ring | own tokens (hold) | keep | not UI transitions | none |

Interruptibility (H, WWDC18): sheet, popover and panel exits must be cancellable: a show
during `leaving` restarts from the current state. The `data-pulse=b`/`hold` restart machinery
already handles this; the new sheet keyframes must have identical `--b` aliases (the
`ds-motion/css.rs` generator does it automatically) and an entry in `Anim` and the lint's
known-keyframes list (`ds-lint/src/animation.rs`). Reduced motion: every above becomes the 120 ms
fade (already the rule).
Lane: motion edits belong to Lane 1D (sheet, alert, peek) and a new small **Lane 1E motion**
that owns `ds-motion/src/motion.css`, `anim.rs`, `recipe*.rs`, `ds-style/tokens/timing.rs`
and `easing.rs` (token retune, keyframe edits, golden of the motion lab). Verify:
`cargo test -p ds-motion -p ds-lint` and the gallery Motion page.

## 8. BEFORE material

Run: `CARGO_TARGET_DIR=~/rs-wt/modern-look/target cargo run -p ds-gallery --release --
--snapshot ~/rs-wt/modern-look/before` (needs `~/rs-wt/modern-look/blitz-kit` symlink to
`~/blitz-kit`, the path dependency). The accounts shots in `~/rs-wt/accounts/shots-w2f/` show
today's consent alert: no hero icon, 340 wide, three Regular buttons stacked, a 1 px outlined
search field, bordered provider list rows with 4 px marks. The mock-up of the target is
`~/rs-wt/modern-look/mock/mock.html` and `mock.png`.
