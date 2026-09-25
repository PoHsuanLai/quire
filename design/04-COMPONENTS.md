# 04 Components

Every component in the `quire` crate: its markup, its sizes, every state with the CSS that draws
it, and what must change to run on Blitz. Built from the two prototypes so that nobody needs to
open the HTML.

- **S** = `~/mailo-design/mailo-spaces.html` (newest; wins every conflict).
- **C** = `~/mailo-design/mailo-charm.html` (older; source for pieces S lacks: dropdown menu,
  tabs, drag ghost, drop target, sync halo, Candy hues).
- `S:123` means line 123 of S. CSS blocks marked "verbatim" are copied character for character;
  everything else is a restatement.

## What this governs

- Every `ds-` class in `crates/ds/src/components/*.css` and every `#[component]` in
  `crates/ds/src/components/*.rs`.
- Which token each value uses. Token values themselves live in `03-COLOR.md` (colour, shadow),
  `01-LAYOUT.md` (spacing, z), `02-TYPE.md` (fonts, sizes) and `05-MOTION.md` (durations,
  easings, full keyframes). This file names tokens; it does not define them.
- Pointer and keyboard behaviour is summarised here and specified as state machines in
  `06-INTERACTIONS.md`.
- Not governed: mail-only reader blocks (`.b-*`, `S:461-534`) and composer document styling
  (`S:727-775`). They are mailo's, not the design system's.

Rules that apply to every section (from the plan's §11 addenda):

1. Every class is prefixed `ds-`. Variants go in `data-variant` / `data-size` / `data-kind`;
   state goes in `aria-*` where an ARIA attribute exists, otherwise in `data-*`.
2. Consumers never style `.ds-*`. If mailo needs a different look, the component grows a variant.
3. No `bool` props. Two-state props are enums from the shared vocabulary below.
4. Every duration, easing, colour, radius, z-index and keyframe comes from the token table. A
   raw value in this document is either a token's current value (shown for reference) or a
   literal the prototype uses and the token table must absorb (listed in Open decisions).
5. Blitz constraints: no JavaScript, no `backdrop-filter`, no `text-overflow: ellipsis`, no
   `filter()`, no `animationend`/`transitionend`, no `line-clamp`, no `mix-blend-mode`, no
   `color-mix()` (use precomputed washes), no `position: sticky`, no native form controls.
   CSS transitions and `@keyframes` work.

## Component index

| # | Component | Prototype source | Mail usage | Shell usage (planned, see 20-SURFACES) |
| --- | --- | --- | --- | --- |
| 1 | Button (Primary, Secondary, Mini, Quiet, Danger) | S `.btn` `.mini` `.plink`; C `.btn.ghost`; Danger derived | Send, Compose, Reply, RSVP, hover-card actions | settings actions, notification actions |
| 2 | IconButton (Tool, Foot, Strip, Pin) | S `.tool` `.foot-btn` `.strip button` `.pin` | reader tools, sidebar foot, row strip, account tiles | bar buttons, dock context, control-center tiles |
| 3 | SegmentedControl | S `.seg` `.view-switch`; C `.seg` | Reader/Original, editor settings | control center, settings |
| 4 | Toggle | derived from `.seg` + accent | settings | control center (Wi-Fi, Bluetooth) |
| 5 | Slider | derived from `input[type=range]`, `.handle`, toast pull tab | Grain in Space editor | volume, brightness, OSD |
| 6 | TextInput (Boxed, Inline) | S `.inp` `.inp.inline` `.pinput` | people fields, bubble link input | settings fields |
| 7 | SearchField | S `.cmdk-in` | command menu | launcher |
| 8 | CommandPill | S `.cmd` | sidebar top | bar launcher button |
| 9 | Kbd | S `kbd`; C `.cmdk-list .kbd` | hints, menus | launcher, shortcut hints |
| 10 | Chip (Accent, Label, Neutral + person/token/status) | S `.chip` `.pchip` `.tok` `.pill`; C `.chip[data-chip]` `.tag` | row labels, recipients, search tokens | notification tags |
| 11 | Avatar | S `.av` (many), `.fav`, `.tile.round`; C `.acct-ring .av` | rows, reader, hover cards, menus | user menu, notifications |
| 12 | Tabs | C `.tabs` | settings pages | settings, control-center pages |
| 13 | SectionHeader | S `.s-h` `.ed-label` `.fmenu .g`; C `.grp` `.menu h6` | sidebar groups, list groups | launcher groups, settings |
| 14 | Count | S `.count` `.pin .n` | sidebar counts | dock badge, bar indicators |
| 15 | Spinner (Spin, Breathe) | C `.halo` | sync | bar sync/progress |
| 16 | ListRow | S `.row`; C `.row` | thread list | notification list, launcher results (rows) |
| 17 | HoverStrip | S `.strip` | row actions | notification actions |
| 18 | Tooltip (Fly, Card) | S `.fly`, `.hc.tip` | strip labels, time tooltip | dock labels, bar tooltips |
| 19 | SidebarItem (+ seal) | S `.item`; C `.view` | places, labels, pinned, Today | settings nav, launcher categories |
| 20 | Menu + MenuEntry (Dropdown, Context, Rich, Slim) | S `.fmenu`; C `.menu` | snooze, labels, `/`, `@`, pickers | bar menus, tray menus, dock context |
| 21 | Popover | shared surface of S `.fmenu` `.hc` `.bubble` | all floating surfaces | xdg_popup content |
| 22 | HoverCard | S `.hc` | thread/sender/account/pin/Today cards | dock window previews (planned) |
| 23 | Toast (+ pull tab) | S `.toast` | undo | notifications (planned) |
| 24 | Scrim, Sheet, Peek | S `.scrim` `.peek`; C `[data-peek]` | peek reader | dialogs, settings sheets |
| 25 | CommandPalette | S `.cmdk` | Ctrl T / Ctrl K | launcher |
| 26 | AppearancePicker | derived from S editor segs + C controls row | settings | control center, settings |
| 27 | AccountTile | S `.pin.acct` | sidebar tiles | user switcher (planned) |
| 28 | ProviderMark | S `.prov` | tiles, rows, From picker | none |
| 29 | LinkPill | S `.linkpill` | reader links | none |
| 30 | SelectionBubble | S `.bubble` | composer | text fields (planned) |
| 31 | SendPill | S `.sendpill`; C `.outbox` | undo send | progress pill (planned) |
| 32 | SpaceEditor pieces | S `.editor` `.field` `.handle` `.stop` `.presets` `.check` `.sp` | Space editor | Settings > Spaces, bar workspace menu |
| 33 | EdgeStrip (+ side peek) | S `.edge`, `.side-peek` | hidden sidebar | dock auto-hide edge (planned) |
| 34 | DragGhost and DropTarget | C `.ghost` `.is-drop-target`; S `.ograb` `.drop-line` | drag thread to place, move composer object | dock drag-out (planned) |
| 35 | SyncHalo | C `.acct-ring .halo` | account sync | bar sync indicator |

## Shared vocabulary

### Props enums (`components/vocab.rs`)

The plan names these; the mapping to attributes below is this document's proposal and is
binding once 04 is approved.

| Enum | Variants | Renders as |
| --- | --- | --- |
| `Availability` | `Enabled`, `Disabled` | `Disabled` adds `aria-disabled="true"` and removes the click handler. Visual: not specified in either prototype (see Open decisions O-1). |
| `Selection` | `Selected`, `Unselected` | `aria-selected="true"|"false"` (rows, menu items, tabs, palette items). |
| `Emphasis` | `Strong`, `Plain` | `data-emphasis="strong"|"plain"`. Used for unread/read weight (`S:190-194`) and the Today item's lighter text (`S:137`). |
| `Here` | `Current`, `Elsewhere` | `aria-current="true"|"false"` (sidebar items, space dots). |
| `Switch` | `On`, `Off` | `aria-pressed` on buttons (segments, tiles, star, bubble marks); `aria-checked` + `role="switch"` on Toggle. |
| `Check` | `Checked`, `Unchecked` | `aria-checked="true"|"false"` on menu items (`S:2080`, `C:1752`). |
| `Fraction(u16)` | permille 0..=1000 | Inline `style="--f: <value/1000>"` consumed by Slider fill/thumb, SendPill ring, progress. |
| `StaggerIndex(u8)` | 0..=12 (capped at 12 by `use_roster`; C capped at 8, `C:1709`) | Inline `style="--i: n"` (rows) or `--j` (strip buttons, `S:1287`). |
| `PulseKey` | opaque key from `use_pulse` | `class="a-<anim>"` + `data-pulse="a"|"b"`: the A/B alias swap restarts a keyframe without `void el.offsetWidth` (`S:1520`, `S:1525`). |
| `Shortcut(Vec<Key>)` | keys | Rendered with glyphs `⌘ ⇧ ⌥ ⌃` then the key, e.g. `⌃T`. S writes "Ctrl T" as plain text (`S:841`); see Open decisions O-2. |

Motion states set by `use_roster` / `use_motion_timer` (not props):
`data-presence="entering"|"present"|"leaving"|"healing"` and, for leaving rows,
`data-exit="fold"|"curl"|"crumple"`. These replace S's `.entering`, `.going`, `.healing` classes.

Root attributes set by `Ds` (not props; consumers never select on them, the lint's
`DsInternals`): `data-theme`, `data-accent`, `data-motion`, `data-material`, `data-blur`,
`data-modality="keyboard"|"pointer"` (spike S12) and `data-hover="warm"|"cold"`, which the
`HoverHub` stamps while cards and fly labels are warm (O-11, section 17).

### Global rules every component inherits

Focus ring (verbatim, `S:55`; same `C:190`):

```css
:focus-visible{ outline:2.5px solid var(--accent); outline-offset:2px; border-radius:4px; }
```

Icon base `Glyph` (verbatim, `S:58-59`; same `C:973-974`). Every icon is Lucide geometry on a
24 grid; components only override width and height.

```css
.ic{ width:16px; height:16px; flex:none; display:block; stroke:currentColor; stroke-width:2;
  stroke-linecap:round; stroke-linejoin:round; fill:none; }
```

Markup: `<svg class="ds-ic" viewBox="0 0 24 24" aria-hidden="true">…paths…</svg>` (`S:957`).
Sizes used across components: 11, 12, 13, 14, 16, 17, 18, 20 px (listed per component).
C also uses 15 px icons with `stroke-width:1.7`/`1.8` (`C:307`, `C:411`, `C:443`); S does not.
S wins: stroke is always 2.

Buttons reset (verbatim, `S:54`): `button{ font:inherit; color:inherit; }`.

Reduced motion (verbatim, `S:807-809`). In ds this becomes the `Reduced` motion level (every
duration 60 ms) rather than a media query with `!important`; see 05-MOTION.

```css
@media (prefers-reduced-motion: reduce){
  *, *::before, *::after{ animation-duration:1ms !important; animation-iteration-count:1 !important; transition-duration:60ms !important; }
}
```

### Truncation (applies wherever S uses ellipsis)

S truncates with `white-space:nowrap; overflow:hidden; text-overflow:ellipsis` on `.nm`,
`.row-sub`, `.row-snip`, `.item .t`, the command pill label, `.att .nm2`, `.linkpill`,
`.cmdk .snip`. Blitz has no ellipsis. Replacement (plan): `.ds-truncate` = `nowrap` +
`overflow:hidden` + a right-edge mask fade; or `text::clip_chars` in Rust when the text is known
and the width is fixed. Every section below that truncates says `.ds-truncate`.

### Surfaces and layers

Floating things render through `OverlayHost` at the end of `.ds`, never inside the component that
opened them (`position:fixed` breaks inside transformed ancestors). z values (`S` A1): link pill
7, toast 8, send pill 9, scrim 10, peek 11, focus page 12, edge 14, side peek 15, palette 20,
hover card 30, menu 40, bubble 41, drag ghost 50 (C). These become `--z-*` tokens (01-LAYOUT).
Positions come from `place(anchor, content, bounds, want, gap) -> Placed` in Rust: flip when it
does not fit, clamp 8 px from the bounds.

On shell surfaces a floating component is its own Wayland popup: the component emits a
`PopoverRequest { anchor, placement, size }`, shell-host maps it to `xdg_positioner`, and the
component fills that surface. The popup's background is the Material recipe
(`Material::Popover`, `--m-tint`, compositor blur); see 03-COLOR.

## Components

### 1. Button

**Purpose.** A labelled action. Five variants. Mail: Send (`S:1958`), Compose (`S:1587`),
Reply/Reply all/Forward (`S:1379`), RSVP (`S:1343`), hover-card actions (`S:1744`), c-warn
choices (`S:2322`). Shell: settings and notification actions.

**Markup.**

```html
<button type="button" class="ds-button" data-variant="primary|secondary|mini|quiet|danger"
        aria-pressed="true|false"   <!-- only when the button is a toggle (Mini) -->
        aria-disabled="true">       <!-- only when Availability::Disabled -->
  <svg class="ds-ic">…</svg><span>Label</span>
</button>
```

Mapping: `.btn` -> `[data-variant=primary]`; `.btn.ghost` (C) -> `[data-variant=secondary]`;
`.mini` -> `[data-variant=mini]`; `.plink` -> `[data-variant=quiet]`; Danger is derived.

**Props.**

```rust
#[component] pub fn Button(
    variant: ButtonVariant,            // Primary | Secondary | Mini | Quiet | Danger
    label: String,
    #[props(default)] icon: Option<IconSource>,  // Glyph | Symbolic | Image; Icon converts
    #[props(default)] pressed: Option<Switch>,   // Some only for toggle buttons (Mini)
    #[props(default)] availability: Availability,
    onclick: EventHandler<Press>,      // Press { button: Primary | Secondary | Middle, modifiers }
    #[props(default)] id: Option<String>,        // -> id, for a popup anchored by element id
    #[props(default)] mounted: Option<EventHandler<MountedEvent>>, // the element, for Anchor::Mounted
) -> Element
```

`mounted` is settled (Gallery fixes B): a menu or popover anchors to the button itself through
`Anchor::Mounted(MountedRef(event.data()))`, with no wrapper to measure; it writes no attribute.

Settled (tray gaps, sill Q6 and Q8): `icon` is an `IconSource` (08-ICONS §1.5: a glyph, a
symbolic external icon painted in the text colour, or an external image as it is); `id` is
written as the element's `id`; `onclick` reports which button pressed. A right-click arrives as
`contextmenu` (its default prevented) and is reported as `Secondary`, the middle button's
`mouseup` as `Middle`, a click or a keyboard activation as `Primary`. A closure `move |_| …` and
an `EventHandler<()>` still convert.

Settled (mailo gaps 4, 2026-09-25): a sixth variant, `Frame` (`data-variant="frame"`), for
words on the Space frame: the sidebar item's chrome (padding 6px 8px, `--r-item`, ui 13.5 / 600,
`--f-ink-soft` on nothing; `--f-ink` on `--f-pill-hover` under the pointer, `--f-pill` held, and
`--f-pill` with `--shadow-current` when `aria-pressed="true"`). `trailing: Option<Trailing>`
(`Caret`: `chevron-down` 12 in `span.ds-button-trail` at .7; `Glyph(Icon)` at the variant's icon
size) follows the label, for a dropdown showing its value. `face: ButtonFace` (`Label` default;
`Bold`, `Italic`, `Underline`, `Strike`) draws the label as `span.ds-button-face[data-face]`
holding `B`, `i` (in `--font-serif` italic, S's Georgia), `U` (underlined) or `S` (struck
through), and names the button by `label` through `aria-label` unless `aria_label` is given;
`FaceMark { face, label }` is the same span on its own, for a `BubbleButton`'s `label`.

**Geometry.**

| Variant | Padding | Radius | Gap | Font | Colours | Icon |
| --- | --- | --- | --- | --- | --- | --- |
| Primary | 8px 14px (C 8px 15px) | `--r-btn` (9) | 6 | ui 13 / 700 | bg `--accent`, text `--accent-ink`, `--shadow-1` | 14 |
| Secondary | as Primary | `--r-btn` | 6 | ui 13 / 700 | bg `--surface-2`, text `--ink-soft`, 1px `--line` border | 14 |
| Mini | 5px 10px (Candy 6px 13px) | `--r-btn` | 6 | ui 12 / 600, line-height 1 | bg `--surface-2`, text `--ink-soft`, 1px `--line` | 14 |
| Quiet | 2px 4px | 6px | not specified | ui 12 | text `--ink-faint`, no bg | not specified |
| Danger (derived) | as Mini | `--r-btn` | 6 | ui 12 / 600 | at rest as Mini | 14 |

Mini is `display:inline-flex; align-items:center; white-space:nowrap`.

**States.**

Primary (verbatim, `S:386-391`):

```css
.btn{ display:inline-flex; align-items:center; gap:6px; border:0; cursor:pointer; border-radius:var(--r-btn); padding:8px 14px;
  font-weight:700; font-size:13px; background:var(--accent); color:var(--accent-ink); box-shadow:var(--shadow-1);
  transition:transform var(--t-tap) var(--e-out), box-shadow var(--t-quick) var(--e-out); }
.btn:hover{ transform:translateY(-1px) scale(1.015); box-shadow:var(--shadow-2); }
.btn:active{ transform:translateY(1px) scale(var(--squish)); }
.btn .ic{ width:14px; height:14px; }
```

C adds `filter:saturate(1.1)` on hover and `box-shadow:var(--shadow-1)` on active (`C:529-530`).
The filter is dropped (Blitz, and S does not have it). The active shadow is not in S; S wins.

Secondary (verbatim, `C:531`; S has no `.btn.ghost`):

```css
.btn.ghost{ background:var(--surface-2); color:var(--ink-soft); border:1px solid var(--line); }
```

Mini (verbatim, `S:165-173`):

```css
.mini{
  display:inline-flex; align-items:center; gap:6px; line-height:1; white-space:nowrap;
  border:1px solid var(--line); background:var(--surface-2); cursor:pointer;
  border-radius:var(--r-btn); padding:5px 10px; font-size:12px; font-weight:600; color:var(--ink-soft);
  transition:transform var(--t-tap) var(--e-out), background-color var(--t-quick) var(--e-out), box-shadow var(--t-quick) var(--e-out);
}
.mini:hover{ color:var(--ink); background:var(--raise); box-shadow:var(--shadow-1); }
.mini:active{ transform:scale(var(--squish)) translateY(1px); }
.mini .ic{ width:14px; height:14px; }
```

Mini pressed (verbatim, `S:523`, the RSVP row; the only pressed Mini styling in S):

```css
.b-event .rsvp .mini[aria-pressed="true"]{ background:var(--accent); color:var(--accent-ink); border-color:transparent; }
```

In ds this becomes `.ds-button[data-variant=mini][aria-pressed="true"]`. C also transitions
`color` (`C:354`); add it (harmless, keeps hover colour smooth).

Quiet (verbatim, `S:601-602`):

```css
.plink{ border:0; background:none; cursor:pointer; font-size:12px; color:var(--ink-faint); padding:2px 4px; border-radius:6px; }
.plink:hover{ color:var(--ink); background:var(--surface); }
```

Danger (derived from C's destructive hover, `C:958-963`, which states "the two buttons that
destroy something go red on hover, and only on hover"):

```css
/* derived: Mini at rest; red only on hover. --danger-wash replaces color-mix (plan token). */
.ds-button[data-variant=danger]:hover{ background:var(--danger-wash); color:var(--danger); }
```

| State | Primary | Mini | Quiet | Danger |
| --- | --- | --- | --- | --- |
| default | above | above | above | = Mini |
| hover | lift -1px, scale 1.015, `--shadow-2` | `--ink` on `--raise` + `--shadow-1` | `--ink` on `--surface` | `--danger` on `--danger-wash` |
| active | translateY(1px) scale(`--squish`) | scale(`--squish`) translateY(1px) | not specified | = Mini |
| focus-visible | global ring | global ring | global ring | global ring |
| pressed | n/a | accent fill, `--accent-ink`, border transparent | n/a | n/a |
| disabled | not specified (O-1) | not specified | not specified | not specified |
| entering / leaving | none | none | none | none |

**Motion.** Transitions only: transform `--t-tap --e-out`; background, colour, shadow
`--t-quick --e-out`.

**Behaviour.** Click on pointer up inside. Space/Enter activate. A Mini with `pressed` toggles its
own `Switch` only when the consumer says so (RSVP is single-select: pressing one un-presses the
others, `S:1689`).

**Blitz notes.** Drop C's `filter:saturate(1.1)`. Nothing else changes.

### 2. IconButton

**Purpose.** An icon-only action. Variants: Tool (reader/composer tools, `S:1373`, `S:1949-1951`),
Foot (sidebar foot: hide sidebar, new Space, `S:846`, `S:1266`), Strip (row hover strip, see
§17), Pin (square tile, see §27 for the account-tile content).

**Markup.**

```html
<button type="button" class="ds-icon-button" data-variant="tool|foot|strip|pin"
        aria-label="Close peek" title="Hide the sidebar (Ctrl S)"
        aria-pressed="true|false"      <!-- Pin only -->
        aria-expanded="true|false">    <!-- Tool when it opens a menu (C:1727) -->
  <svg class="ds-ic">…</svg>
</button>
```

Mapping: `.tool` -> `[data-variant=tool]`; `.foot-btn` -> `[data-variant=foot]`;
`.strip button` -> `[data-variant=strip]`; `.pin` -> `[data-variant=pin]`.

**Props.**

```rust
#[component] pub fn IconButton(
    variant: IconButtonVariant,        // Tool | Foot | Strip | Pin | Status
    #[props(into)] icon: IconSource, label: String, // Icon converts; label -> aria-label (required)
    #[props(default)] tooltip: Option<String>,   // -> title; Strip uses Tooltip::Fly instead
    #[props(default)] pressed: Option<Switch>,   // Pin
    #[props(default)] expanded: Option<Switch>,  // Tool that owns a Menu
    #[props(default)] availability: Availability,
    onclick: EventHandler<Press>,      // settled: as Button's
    #[props(default)] id: Option<String>,        // settled: as Button's
    #[props(default)] mounted: Option<EventHandler<MountedEvent>>, // settled: as Button's
) -> Element
```

An external icon (a tray item's pixmap or theme icon) is `IconSource::Symbolic` or
`IconSource::Image` and draws at its own `ExternalIcon::size` (settled, tray gaps Q6); a glyph
takes the variant's size below.

**Geometry.**

| Variant | Box | Radius | Colours at rest | Icon | Surface it sits on |
| --- | --- | --- | --- | --- | --- |
| Tool | 28 x 26, 1px transparent border | `--r-btn` | `--ink-faint`, transparent | 16 | card (`--surface-2` reader) |
| Foot | 24 x 24 | 8px | `--f-ink-soft`, transparent | 16 | Space frame |
| Strip | 26 x 26 (C 28) | 999px | `--ink-soft`, transparent | 14 (C 15) | strip pill (`--raise`) |
| Pin | aspect-ratio 1, fills a 4-column grid cell, gap 6 | 12px | `--f-ink`, bg `--f-pill-hover` | content (§27) | Space frame |
| Status | `--bar-status-box` square (default 22) | `--r-item` | `--f-ink-soft`, transparent; hover `--f-ink` on `--f-pill-hover`; pressed/expanded `--f-pill` | `--bar-status-glyph` (default 16), sized in CSS | Space frame (the bar) |

Status is settled (bar gaps, sill Q12; 13 §13.3.1): a consumer writes the two properties with
`ds::StatusMetrics` from `bar.status_icon_box_px`, `bar.status_glyph_px` and
`bar.glyph_size_policy` (22-SETTINGS §3).

**The frame ground** (settled, bar gaps): the sidebar item's pattern (§19, `--f-*` inks on the
frame) is a scope, not a per-component variant. Under `data-ground="frame"` (a Bar or Dock root,
or `Surface { on: Some(Ground::Frame) }`) the paper inks and fills are the frame's (`--ink*`
→ `--f-ink*`, `--surface` → `--f-pill-hover`, `--surface-2`/`--raise` → `--f-pill`, `--line*` →
`--f-line`), so every component on it draws in the frame inks; overlays opened from it are
paper.

**States.**

Tool (verbatim, `S:204-206`):

```css
.tool{ display:inline-grid; place-items:center; width:28px; height:26px; border:1px solid transparent; border-radius:var(--r-btn);
  background:transparent; color:var(--ink-faint); cursor:pointer; transition:background-color var(--t-quick) var(--e-out), color var(--t-quick) var(--e-out); }
.tool:hover{ color:var(--ink); background:var(--surface); }
```

C's Tool is a text+icon button with active and expanded states (verbatim, `C:993-1002`). Its
geometry is not used (S's icon-only box wins); its `:active` and `[aria-expanded]` rules are
adopted because S has none:

```css
.tool:active{ transform:scale(var(--squish)); }
.tool[aria-expanded="true"]{ color:var(--ink); background:var(--surface-2); border-color:var(--line); }
```

Foot (verbatim, `S:151-153`):

```css
.foot-btn{ width:24px; height:24px; border-radius:8px; border:0; background:transparent; cursor:pointer;
  display:grid; place-items:center; color:var(--f-ink-soft); }
.foot-btn:hover{ background:var(--f-pill-hover); color:var(--f-ink); }
```

Strip: see §17 for the full block. Summary: hover `--accent-soft` bg + `--ink`; active
`scale(.88)`.

Pin (verbatim, `S:108-114`, `S:118`):

```css
.pin{
  aspect-ratio:1; border:0; border-radius:12px; cursor:pointer; background:var(--f-pill-hover);
  display:grid; place-items:center; position:relative; color:var(--f-ink);
  transition:background-color var(--t-quick) var(--e-out), transform var(--t-tap) var(--e-out);
}
.pin:hover{ background:var(--f-pill); }
.pin:active{ transform:scale(var(--squish)); }
.pin[aria-pressed="true"]{ background:var(--f-pill); box-shadow:0 1px 0 rgba(255,255,255,.4) inset, 0 2px 6px -3px rgba(0,0,0,.25); }
```

| State | Tool | Foot | Strip | Pin |
| --- | --- | --- | --- | --- |
| hover | `--ink` on `--surface` | `--f-ink` on `--f-pill-hover` | `--ink` on `--accent-soft` | bg `--f-pill` |
| active | scale(`--squish`) (from C) | not specified | scale(.88) | scale(`--squish`) |
| focus-visible | global ring | global ring | global ring | global ring |
| pressed / expanded | expanded: `--ink` on `--surface-2` + `--line` border | n/a | n/a | `--f-pill` + current-item shadow |
| disabled | not specified (O-1) | not specified | not specified | not specified |
| entering | none | none | `pop-in` (§17) | none |

**Motion.** Transitions: background and colour `--t-quick --e-out`; transform `--t-tap --e-out`.

**Behaviour.** `aria-label` is mandatory. Tool with `expanded` toggles its Menu (C:1770-1775).

**Blitz notes.** None beyond the shared rules. The Pin's inset white highlight is a literal
`rgba(255,255,255,.4)`; the token table needs `--shadow-current` (O-3).

**Status radius (the macOS polish pass, settled 2026-09-24).** `Status`'s hover and open pill is
`--r-shell-bar-item` 4, the bar item's, not `--r-item` 9.

### 3. SegmentedControl

**Purpose.** One choice out of 2 to 4, all visible. Mail: Reader/Original (`S:1376`), Space
editor Appearance / Accent / Provider marks (`S:879-893`). Shell: control center, settings.
Also the building block of AppearancePicker (§26).

**Markup.**

```html
<div class="ds-segmented" data-size="regular|small" role="group" aria-label="Appearance">
  <button type="button" class="ds-segment" aria-pressed="true">System</button>
  <button type="button" class="ds-segment" aria-pressed="false">Light</button>
  <button type="button" class="ds-segment" aria-pressed="false">Dark</button>
</div>
```

Mapping: `.seg` -> `.ds-segmented[data-size=regular]`; `.view-switch` ->
`.ds-segmented[data-size=small]`.

**Props.**

```rust
#[component] pub fn SegmentedControl<T: Clone + PartialEq + 'static>(
    label: String,                         // aria-label of the group
    options: Vec<(T, String)>,             // value, visible text
    value: T,
    #[props(default)] size: SegSize,       // Regular | Small
    onchange: EventHandler<T>,
) -> Element
```

**Geometry.**

| Part | Regular (`.seg`) | Small (`.view-switch`) |
| --- | --- | --- |
| Track padding | 3px | 2px |
| Track gap | 2px | 2px |
| Track bg / border | `--surface-2`, 1px `--line` | `--surface`, 1px `--line` |
| Track radius | 999px | 999px |
| Segment padding | 5px 11px (C 6px 13px) | 3px 10px |
| Segment font | ui 12 / 600 (C 12.5 / 600, letter-spacing .01em) | ui 11.5 / 600 |
| Segment radius | 999px | 999px |
| Segment text | `--ink-soft` | `--ink-soft` |

**States.**

Regular (verbatim, `S:265-268`):

```css
.seg{ display:inline-flex; padding:3px; gap:2px; background:var(--surface-2); border:1px solid var(--line); border-radius:999px; }
.seg button{ border:0; background:transparent; padding:5px 11px; border-radius:999px; font-size:12px; font-weight:600;
  cursor:pointer; color:var(--ink-soft); transition:background-color var(--t-quick) var(--e-out), color var(--t-quick) var(--e-out); }
.seg button[aria-pressed="true"]{ background:var(--ink); color:var(--paper); }
```

Small (verbatim, `S:537-539`):

```css
.view-switch{ display:inline-flex; padding:2px; gap:2px; background:var(--surface); border:1px solid var(--line); border-radius:999px; margin-left:auto; }
.view-switch button{ border:0; background:transparent; padding:3px 10px; border-radius:999px; font-size:11.5px; font-weight:600; color:var(--ink-soft); cursor:pointer; }
.view-switch button[aria-pressed="true"]{ background:var(--ink); color:var(--paper); }
```

S has no hover or active rule. C does (verbatim, `C:236-237`); adopt both:

```css
.seg button:hover{ color:var(--ink); }
.seg button:active{ transform:scale(var(--squish)); }
```

(with `transform var(--t-tap) var(--e-out)` added to the transition, `C:233-234`).

| State | CSS |
| --- | --- |
| default | above |
| hover | `--ink` text (from C) |
| active | scale(`--squish`) (from C) |
| focus-visible | global ring on the segment |
| pressed (current) | `--ink` bg, `--paper` text |
| disabled | not specified (O-1) |
| entering / leaving | none |

`margin-left:auto` on `.view-switch` is layout of the reader head, not the component; the consumer
places it.

**Motion.** Background and colour `--t-quick --e-out`; transform `--t-tap --e-out`. The pressed
fill jumps from one segment to the next; there is no sliding thumb in either prototype.

**Behaviour.** Click selects. Keyboard: not specified in the prototypes (each segment is a plain
button, Tab moves between them). 06-INTERACTIONS decides whether Left/Right move the selection.

**Blitz notes.** None.

### 4. Toggle

**Derived.** Neither prototype has an on/off switch. The plan adds it ("div track/knob, spring")
because Blitz has no native form controls. Built from the SegmentedControl track (`.seg`), the
accent fill of a pressed Mini (`S:523`), and the Space dot's spring (`S:147-150`).

**Purpose.** A setting that is on or off and applies at once. Mail: settings. Shell: control
center tiles (Wi-Fi, Bluetooth, Do Not Disturb).

**Markup.**

```html
<button type="button" class="ds-toggle" role="switch" aria-checked="true|false" aria-label="Wi-Fi">
  <span class="ds-toggle-knob"></span>
</button>
```

**Props.**

```rust
#[component] pub fn Toggle(label: String, value: Switch,
    #[props(default)] availability: Availability, onchange: EventHandler<Switch>) -> Element
```

**Geometry.** Proposal (every size below is derived, none is in a prototype; O-4):

| Part | Value | Derived from |
| --- | --- | --- |
| Track padding | 3px | `.seg` padding |
| Track border | 1px `--line` | `.seg` |
| Track radius | 999px | `.seg` |
| Track bg Off | `--surface-2` | `.seg` |
| Track bg On | `--accent`, border transparent | pressed Mini (`S:523`) |
| Knob | circle, `--raise`, `--shadow-1` | not specified; `--raise` is the "floats above" surface (A3) |
| Knob size / track width | not specified (O-4) | |

**States (derived CSS).**

```css
/* derived */
.ds-toggle{ display:inline-flex; padding:3px; border:1px solid var(--line); border-radius:999px;
  background:var(--surface-2); cursor:pointer;
  transition:background-color var(--t-quick) var(--e-out), border-color var(--t-quick) var(--e-out); }
.ds-toggle-knob{ border-radius:999px; background:var(--raise); box-shadow:var(--shadow-1);
  transition:transform var(--t-move) var(--e-spring); }
.ds-toggle[aria-checked="true"]{ background:var(--accent); border-color:transparent; }
.ds-toggle[aria-checked="true"] .ds-toggle-knob{ transform:translateX(/* track inner width - knob */); }
.ds-toggle:active .ds-toggle-knob{ transform:scale(var(--squish)); }
```

| State | Rule |
| --- | --- |
| default (Off) | track `--surface-2`, knob left |
| hover | not specified |
| active | knob scale(`--squish`) |
| focus-visible | global ring on the track |
| On | track `--accent`, knob right |
| disabled | not specified (O-1) |

**Motion.** Knob moves on `--t-move --e-spring` ("spring only on contact": the toggle moves
because it was touched). Track colour `--t-quick --e-out`.

**Behaviour.** Click or Space flips. Enter: not specified.

**Blitz notes.** Exists only because `input[type=checkbox]` is unavailable. The pressed-state
knob position must be a fixed px translate (no `calc` against the track's own width in a
transition is guaranteed); fix sizes in O-4.

### 5. Slider

**Derived.** S uses a native `input[type=range]` with `accent-color` for Grain (`S:269`,
`S:875`). Blitz has no native range. The plan: "Slider (divs + DragTracker)". Built from the
editor's `.handle` (the one draggable dot in S, `S:255-257`) and the toast pull tab's drag
model (`S:1562-1572`).

**Purpose.** A continuous value in a range. Mail: Space editor Grain 0..100. Shell: volume,
brightness, OSD.

**Markup.**

```html
<div class="ds-slider" role="slider" tabindex="0" aria-label="Grain"
     aria-valuemin="0" aria-valuemax="100" aria-valuenow="35" style="--f:.35">
  <div class="ds-slider-track"><div class="ds-slider-fill"></div></div>
  <div class="ds-slider-thumb" data-drag="idle|live"></div>
</div>
```

**Props.**

```rust
#[component] pub fn Slider(label: String, value: Fraction, #[props(default)] step: Fraction,
    #[props(default)] availability: Availability, onchange: EventHandler<Fraction>) -> Element
```

**Geometry.**

| Part | Value | Source |
| --- | --- | --- |
| Width | 100% of its container | `S:269` `width:100%` |
| Fill colour | `--accent` | `S:269` `accent-color:var(--accent)` |
| Thumb | 22 x 22 circle, 3px white border, `0 0 0 1px rgba(0,0,0,.25), 0 3px 8px rgba(0,0,0,.35)` | `.handle` `S:255-256` (proposal: reuse) |
| Thumb while dragging | scale(1.15) | `.handle.on` `S:257` |
| Track height, track colour | not specified (O-5) | |

**States (derived CSS).**

```css
/* derived */
.ds-slider-fill{ width:calc(var(--f) * 100%); background:var(--accent); }
.ds-slider-thumb{ left:calc(var(--f) * 100%); width:22px; height:22px; margin-left:-11px; border-radius:999px;
  border:3px solid #fff; box-shadow:0 0 0 1px rgba(0,0,0,.25), 0 3px 8px rgba(0,0,0,.35); cursor:grab; }
.ds-slider-thumb[data-drag="live"]{ transform:scale(1.15); }
```

| State | Rule |
| --- | --- |
| default | as above |
| hover | not specified |
| active / dragging | thumb scale(1.15) (`S:257`); no transition on `left` while dragging (toast tab sets `transition:none`, `S:1565`) |
| focus-visible | global ring on `.ds-slider` |
| disabled | not specified (O-1) |

**Motion.** While dragging: none (follows the pointer 1:1). Keyboard steps: not specified.

**Behaviour.** Pointer down on track or thumb captures the pointer and jumps the value to the
pointer (`S:1435-1441`, the field does the same). Move updates, up releases. Keys: Left/Down
minus one step, Right/Up plus one step (the handle uses 5° / .05 steps, `S:1455-1456`; slider
step is a prop). See 06-INTERACTIONS "drag tracker".

**Blitz notes.** `setPointerCapture` is JS; `DragTracker` in Rust owns the capture and writes
`--f`. The white border and shadow literals need tokens (`--handle-ring`, O-3).

### 6. TextInput

**Purpose.** "ONE FIELD": every text input in the window (`S:777-780`). Boxed for standalone
fields; Inline inside another container (recipient row, palette header, bubble link mode).

**Markup.**

```html
<input class="ds-input" data-variant="boxed|inline" type="text"
       placeholder="Add a person" aria-label="To" autocomplete="off">
```

Mapping: `.inp` -> `[data-variant=boxed]`; `.inp.inline` and `.pinput` -> `[data-variant=inline]`.

**Props.**

```rust
#[component] pub fn TextInput(variant: InputVariant /* Boxed | Inline */, label: String,
    value: String, #[props(default)] placeholder: String,
    #[props(default)] availability: Availability,
    oninput: EventHandler<String>, #[props(default)] onkey: EventHandler<KeyboardData>,
    #[props(default)] focus: Focus /* OnMount | Manual (default); W2 integration, 06 §17 */) -> Element
```

**Geometry.**

| Variant | Padding | Radius | Font | Colours | Width |
| --- | --- | --- | --- | --- | --- |
| Boxed | 7px 10px | 10px (`--r-field`) | ui 13.5 | `--ink` on `--surface`, 1px `--line` | set by consumer |
| Inline | 3px 2px | none | ui 13.5 (palette 16, bubble 12.5, `.pinput` 13) | `--ink`, transparent | min 120px, flex 1 |

**States.**

Verbatim, `S:781-786`:

```css
.inp{ font:inherit; font-size:13.5px; color:var(--ink); background:var(--surface); border:1px solid var(--line); border-radius:10px;
  padding:7px 10px; outline:none; transition:border-color var(--t-quick) var(--e-out), box-shadow var(--t-quick) var(--e-out); }
.inp::placeholder{ color:var(--ink-faint); }
.inp:focus{ border-color:var(--accent); box-shadow:0 0 0 3px var(--accent-soft); }
.inp.inline{ border-color:transparent; background:transparent; padding:3px 2px; min-width:120px; flex:1; box-shadow:none; }
.inp.inline:focus{ border-color:transparent; box-shadow:none; }
```

`.pinput` (verbatim, `S:599-600`) is the same Inline field at 13 px; fold it into Inline.

```css
.pinput{ border:0; outline:none; background:transparent; font:inherit; font-size:13px; color:var(--ink); min-width:120px; flex:1; padding:3px 2px; }
.pinput::placeholder{ color:var(--ink-faint); }
```

| State | Boxed | Inline |
| --- | --- | --- |
| default | `--line` border | transparent |
| hover | not specified | not specified |
| focus (any focus, not only visible) | border `--accent` + `0 0 0 3px var(--accent-soft)` (the plan's `--accent-ring`) | nothing (the container shows focus) |
| placeholder | `--ink-faint` | `--ink-faint` |
| disabled | not specified (O-1) | not specified |
| error | not specified; the To row uses `shake-x` on its row (`S:590`), not a red border | |

**Motion.** Border and ring `--t-quick --e-out`.

**Behaviour.** Focus on open where the surface says so (palette input focused on open,
`S:1661`). `Focus::Controlled(FocusRequest)` (settled 2026-09-24, FINDINGS "Launcher gaps", sill
Q44) focuses on mount and again at every `request()`: a menu that took the keyboard hands it
back when it closes, without remounting the field. Every focus change waits out a document the
renderer holds (`ds::HostFocus`, Q43). Recipient inputs: Enter or `,` adds, Backspace on empty removes the last chip
(`S:2265-2266`).

**Blitz notes.** `::placeholder` must be verified in the spike; if unsupported, render the
placeholder as an absolutely positioned span shown while the value is empty. `outline:none`
is required so the global focus ring does not double the accent ring.

**Settled (mailo gaps 4, 2026-09-25).**

- `Bare` face (`InputVariant::Bare`, also named `FieldFace`): no border, padding, ground or radius;
  font, size, weight, tracking, line height and colour inherited, so a title or a property row's
  value is edited where it reads. The caret is `--accent`, the selection `--accent-soft`; the
  placeholder is the parent's colour at .45.
- `TextInputKind::Secret`: drawn as `Password`, but the component keeps the typed text in its own
  state, emits it only through `oninput` and `onchange`, and never writes a `value` attribute (the
  prop is ignored). `Password` still writes its `value`, unchanged since wave 2. mailo's secret
  field is `Secret`.
- `TextInputKind::File`: the chosen name in a read-only `span.ds-input[data-kind=file]` (a
  `role=textbox`, `aria-readonly`) and a Tool `IconButton` (folder, titled "Choose…", named
  "{label}: Choose…") after it, gap 6. Blitz has no file picker (blitz-dom's `file-input` feature
  is off in quire's pin, and even on it only draws a Browse button), so a click on either calls
  `on_pick` and the host opens its own chooser and hands the name back as `value`.
- `TextInputKind::Multiline { rows: Rows(n), grow: Grow::{Fixed, ToContent} }`: a `textarea`
  with `rows`, `height:auto`, no resize handle. blitz-dom makes a `textarea` a multiline text
  editor, reads its text from the `value` attribute (not its children), sizes it at `rows` line
  heights (2 when absent; `cols` at 0.6 em each, else 300 px) and inserts a newline on Enter.
  `ToContent` raises `rows` to the value's hard line count; a soft wrap does not grow it.
- `onchange: EventHandler<String>`: the value committed, on Enter in a one-line field or when the
  caret leaves any field. Blitz dispatches no `change` event, so the field makes it on both
  renderers.

### 7. SearchField

**Purpose.** A search icon plus an Inline TextInput, as the header of a list of results. Mail:
the command menu's input row. Shell: the launcher.

**Markup.**

```html
<div class="ds-search">
  <svg class="ds-ic">…search…</svg>
  <input class="ds-input" data-variant="inline" type="text" aria-label="Search"
         placeholder="Search mail, people, actions · try from:dana or has:attachment">
</div>
<div class="ds-search-tokens"><span class="ds-chip" data-variant="token">from dana</span></div>
```

Mapping: `.cmdk-in` -> `.ds-search`; `.cmdk .tokens` -> `.ds-search-tokens`.

**Props.**

```rust
#[component] pub fn SearchField(label: String, value: String, placeholder: String,
    tokens: Vec<String>, oninput: EventHandler<String>, onkey: EventHandler<KeyboardData>,
    #[props(default)] focus: Focus /* passed to the field; the palette uses OnMount, or
                                       Controlled(request) when given a FocusRequest */) -> Element
```

**Geometry.** Row: flex, gap 9, padding 12px 14px, bottom border 1px `--line-soft`, icon and
text colour `--ink-faint`, input ui 16 `--ink` (C 14.5, padding 11px 14px, icon 18). Tokens row:
flex wrap, gap 5, padding `0 14px 8px`.

**States.** Verbatim, `S:235-236` and `S:794-797`:

```css
.cmdk-in{ display:flex; align-items:center; gap:9px; padding:12px 14px; border-bottom:1px solid var(--line-soft); color:var(--ink-faint); }
.cmdk-in input{ font:inherit; font-size:16px; border:0; outline:none; background:transparent; color:var(--ink); width:100%; }
.cmdk-in{ padding:12px 14px; }
.cmdk-in input{ font-size:16px; }
.cmdk .tokens{ display:flex; gap:5px; flex-wrap:wrap; padding:0 14px 8px; }
```

Focus is shown by the caret only (the field is inline). Hover, disabled: not specified.

**Motion.** None. **Behaviour.** Up/Down move the selection in the list below (clamped, not
wrapping, `S:1653-1654`); Enter runs; Esc closes. Operators `from: is:unread is:starred
has:attachment label:` become tokens (`S:1596-1598`). 06-INTERACTIONS "command menu search".

**Blitz notes.** None beyond TextInput.

### 8. CommandPill

**Purpose.** The full-width "Search or run a command" button at the top of the sidebar, drawn on
the Space colour (Arc-derived). Opens the CommandPalette. Shell: the bar's launcher button.

**Markup.** (from `S:841`)

```html
<button type="button" class="ds-command-pill">
  <svg class="ds-ic">…search…</svg>
  <span class="ds-command-pill-label ds-truncate">Search or run a command</span>
  <span class="ds-command-pill-keys">⌃T</span>
</button>
```

Mapping: `.cmd` -> `.ds-command-pill`; `.cmd .k` -> `.ds-command-pill-keys`.

**Props.**

```rust
#[component] pub fn CommandPill(label: String, shortcut: Shortcut, onclick: EventHandler<()>) -> Element
```

**Geometry.** Width 100%, padding 8px 10px, radius 10, gap 8, ui 13, text `--f-ink-soft`, bg
`--f-pill`, inset highlight `0 1px 0 rgba(255,255,255,.35) inset`. Keys: data 10,
`--f-ink-faint`, `margin-left:auto`, nowrap. Icon 16.

**States.** Verbatim, `S:92-100`:

```css
.cmd{
  display:flex; align-items:center; gap:8px; width:100%; border:0; cursor:pointer; text-align:left;
  padding:8px 10px; border-radius:10px; background:var(--f-pill); color:var(--f-ink-soft);
  font-size:13px; box-shadow:0 1px 0 rgba(255,255,255,.35) inset;
  transition:background-color var(--t-quick) var(--e-out), transform var(--t-tap) var(--e-out);
}
.cmd:hover{ color:var(--f-ink); }
.cmd:active{ transform:scale(var(--squish)); }
.cmd .k{ margin-left:auto; white-space:nowrap; flex:none; font-family:var(--font-data); font-size:10px; color:var(--f-ink-faint); }
```

| State | Rule |
| --- | --- |
| hover | text `--f-ink` (background unchanged) |
| active | scale(`--squish`) |
| focus-visible | global ring |
| disabled, entering, leaving | not specified / none |

**Motion.** Background `--t-quick --e-out`, transform `--t-tap --e-out`. Colour change is not
in the transition list (instant).

**Behaviour.** Click opens the palette; the same shortcut (Ctrl T or Ctrl K) opens it from
anywhere (`S:1667`).

**Blitz notes.** Label uses `.ds-truncate` (S uses inline ellipsis, `S:841`).

### 9. Kbd

**Purpose.** A key cap in hints and menus. Mail: intro hints, composer hint row (`S:1956`),
hover-card foot (`S:1735`). Shell: launcher hints, shortcut lists.

**Markup.** `<kbd class="ds-kbd">⌃</kbd><kbd class="ds-kbd">T</kbd>` (one `kbd` per key, as
`S:824`).

**Props.** `#[component] pub fn Kbd(shortcut: Shortcut, #[props(default)] size: KbdSize /* Regular | Small */) -> Element`

**Geometry.** data 10.5 (Small 9.5, `S:662`), padding 1px 5px, radius 5, 1px `--line` border with
a 2px bottom border, bg `--surface-2`, text `--ink-soft`. Keys in one shortcut are separated by a
space (`S:824`) or a thin space (`C:1175`); pick one (O-2).

**States.** Verbatim, `S:67-68`:

```css
kbd{ font-family:var(--font-data); font-size:10.5px; border:1px solid var(--line); border-bottom-width:2px;
  border-radius:5px; padding:1px 5px; color:var(--ink-soft); background:var(--surface-2); }
```

Static: no hover, active, focus, or motion. C's palette uses a flatter cap (verbatim,
`C:1079-1080`: `margin-left:auto; font-family:var(--font-data); font-size:10px;
color:var(--ink-faint); border:1px solid var(--line); border-radius:5px; padding:1px 5px;`);
S replaced it with the plain `.sc` text, so it is not a variant.

**Blitz notes.** Glyphs ⌘⇧⌥⌃ must be in the bundled Space Mono subset or fall back cleanly
(02-TYPE).

### 10. Chip

**Purpose.** A small label that states a fact. Variants in the plan: Accent (S's only chip:
every label chip is accent-soft), Label(hue) (C Candy: one hue per named label), Neutral (C
`.tag`). Related shapes that S uses and the plan's enum does not yet name: Person chip
(`.pchip`, recipients), Token (`.tok`, search operators), Status pill (`.pill.ok/.bad`,
contrast checks), Attachment count (`.clip`). See O-6.

**Markup.**

```html
<span class="ds-chip" data-variant="accent|label|neutral|token|status" data-hue="blue" data-status="ok|bad">spec</span>

<span class="ds-chip" data-variant="person">          <!-- .pchip -->
  <span class="ds-avatar" data-size="18">D</span>Dana Okafor
  <button type="button" class="ds-chip-remove" aria-label="Remove Dana Okafor"><svg class="ds-ic">…x…</svg></button>
</span>

<span class="ds-clip"><svg class="ds-ic">…paperclip…</svg>3</span>
```

**Props.**

```rust
#[component] pub fn Chip(variant: ChipVariant, text: String,
    #[props(default)] onremove: Option<EventHandler<()>>,   // Person only
    #[props(default)] pulse: Option<PulseKey>) -> Element    // chip-land / chip-in / flash
pub enum ChipVariant { Accent, Label(LabelHue), Neutral, Person(Avatar), Token, Status(Verdict) }
pub enum LabelHue { Red, Amber, Green, Blue, Violet }
```

**Geometry.**

| Variant | Font | Padding | Radius | Colours | Source |
| --- | --- | --- | --- | --- | --- |
| Accent | data 9.5 (C letter-spacing .02em) | 1.5px 7px (Candy 2px 9px) | `--r-chip` (6 6 6 2) | `--ink` on `--accent-soft` | `S:198-199` |
| Label(hue) | as Accent | as Accent | as Accent | `--c-<hue>-deep` on `--c-<hue>-soft` | `C:941-944` |
| Neutral | data 9.5 | 2px 7px | 999px | `--ink-faint` on `--surface-2`, 1px `--line` | `C:615-616` |
| Token | data 10.5 | 2px 8px | 999px | `--ink` on `--accent-soft` | `S:798` |
| Status | data 9.5 | 1px 7px | 999px | ok: `--ok` on 16% ok wash; bad: `--danger` on 16% danger wash | `S:276-278` |
| Person | ui 12.5 | 2px 8px 2px 3px, gap 6 | 999px | `--ink` on `--surface`, 1px `--line`; avatar 18; remove icon 11 `--ink-faint` | `S:592-596` |
| Clip | data 9.5, gap 2 | none | none | `--ink-faint`, icon 11 | `S:543-544` |

All chips are `white-space:nowrap` (never truncated).

**States.**

Accent (verbatim, `S:198-199`):

```css
.chip{ font-family:var(--font-data); font-size:9.5px; padding:1.5px 7px; border-radius:var(--r-chip);
  background:var(--accent-soft); color:var(--ink); white-space:nowrap; }
```

Label(hue) (verbatim, `C:941-944`; one line per label, the label-to-hue map is data):

```css
body[data-look="candy"] .chip[data-chip="spec"]{   background:var(--c-blue-soft);   color:var(--c-blue-deep); }
body[data-look="candy"] .chip[data-chip="rust"]{   background:var(--c-amber-soft);  color:var(--c-amber-deep); }
```

Neutral (verbatim, `C:615-616`):

```css
.tag{ font-family:var(--font-data); font-size:9.5px; padding:2px 7px; border-radius:999px;
  background:var(--surface-2); border:1px solid var(--line); color:var(--ink-faint); }
```

Token (verbatim, `S:798`):

```css
.cmdk .tok{ font-family:var(--font-data); font-size:10.5px; padding:2px 8px; border-radius:999px; background:var(--accent-soft); color:var(--ink); }
```

Status (verbatim, `S:276-278`; `color-mix` becomes the precomputed `--ok-wash` / `--danger-wash`):

```css
.pill{ font-family:var(--font-data); font-size:9.5px; padding:1px 7px; border-radius:999px; }
.pill.ok{ background:color-mix(in oklab, var(--ok) 16%, transparent); color:var(--ok); }
.pill.bad{ background:color-mix(in oklab, var(--danger) 16%, transparent); color:var(--danger); }
```

Person (verbatim, `S:592-598`):

```css
.pchip{ display:inline-flex; align-items:center; gap:6px; padding:2px 8px 2px 3px; border-radius:999px; background:var(--surface); border:1px solid var(--line);
  font-size:12.5px; color:var(--ink); animation:chip-in var(--t-move) var(--e-spring); }
.pchip .av{ width:18px; height:18px; border-radius:999px; display:grid; place-items:center; font-family:var(--font-display); font-weight:800; font-size:9px; color:#fff; }
.pchip .x{ border:0; background:none; padding:0; cursor:pointer; color:var(--ink-faint); line-height:0; }
.pchip .x .ic{ width:11px; height:11px; }
.pchip.flash{ box-shadow:0 0 0 3px var(--accent-soft); }
```

Clip (verbatim, `S:543-544`):

```css
.row .clip{ display:inline-flex; align-items:center; gap:2px; font-family:var(--font-data); font-size:9.5px; color:var(--ink-faint); }
.row .clip .ic{ width:11px; height:11px; }
```

| State | Rule |
| --- | --- |
| default | per variant |
| hover | not specified (chips are not interactive; the Person chip's remove button: not specified) |
| focus-visible | global ring on the remove button |
| entering | Person: `chip-in --t-move --e-spring` on mount. Accent/Label landing after a drop or label pick: `chip-land --t-big --e-spring` (C `.is-landing`, `C:402`) |
| flash | Person: `0 0 0 3px var(--accent-soft)` for 1200 ms (`S:2119`): `pulse` with `Anim::ChipFlash`, the `chip-flash` keyframe (chip-in's last frame plus the ring) over `--t-flash` |
| leaving | none (removed at once) |

**Motion.** `chip-in`, `chip-land`, `chip-flash` (05-MOTION; `chip-flash` and `--t-flash` 1200 ms are quire's, wave 1 amendment, proposed). The flash plays through the pulse class, so it needs no Rust timer.

**Behaviour.** Remove button removes the recipient; Backspace in an empty recipient input removes
the last chip (`S:2266`).

**Blitz notes.** `color-mix` replaced by `--ok-wash`/`--danger-wash`. Person avatar colour is an
inline style from the hash (see §11).

### 11. Avatar

**Purpose.** A person or account as a coloured disc with one letter. Mail: reader meta, hover
cards, recipient chips, event attendees, account tiles, palette/menu tiles, Today favicons.
Shell: user menu, notifications.

**Markup.**

```html
<span class="ds-avatar" data-size="28" data-shape="round|square" style="--av-bg:#5B4FC4; --av-fg:#fff">D</span>
```

Colour sources (A3): `Ink` = `--ink` bg, `--paper` text (reader meta, hover card); `Account` = the
account colour, white text (`S:1238`); `Person` = hash `h=(h*31+code)%360` -> `hsl(h,38%,42%)`,
white text (`S:1879`); `Stack` = `--ink-soft` bg, `--paper` text (event attendees).

**Props.**

```rust
#[component] pub fn Avatar(initial: char, size: AvatarSize, tone: AvatarTone,
    #[props(default)] shape: AvatarShape /* Round | Square */) -> Element
pub enum AvatarTone { Ink, Account(Colour), Person(PersonHue), Stack }
```

**Geometry.** Every avatar is `display:grid; place-items:center`, display font weight 800.

| Size | Font | Shape | Where | Source |
| --- | --- | --- | --- | --- |
| 16 | 9 | square r5 | sidebar pinned/Today favicon `.fav` | `S:138-139` |
| 18 | 9 | round | person chip | `S:594` |
| 20 | 9.5 | round, 2px `--surface` border, overlap -6px | event attendees | `S:519-521` |
| 20 | 9 | round | slim menu tile `.tile.round` | `S:788` |
| 22 | 10 | round | hover-card message rows | `S:409-410` |
| 26 | 12 | round | pinned tile (non-account) | `S:115-116` |
| 28 | 12 | round | reader meta | `S:209-210` |
| 28 | 12.5 | round | account tile | `S:548` |
| 30 | 13 | round | C account ring | `C:278-283` |
| 34 | 14 | round | hover-card person header | `S:417` |
| 34 | 13 | round (`.tile.round`) or square r8 | rich menu tile | `S:671-672`, `S:787` |

**States.** Verbatim, reader meta (`S:209-210`):

```css
.meta .av{ width:28px; height:28px; border-radius:999px; display:grid; place-items:center; flex:none;
  background:var(--ink); color:var(--paper); font-family:var(--font-display); font-weight:800; font-size:12px; }
```

Verbatim, attendee stack (`S:519-521`):

```css
.b-event .ppl i{ width:20px; height:20px; border-radius:999px; display:grid; place-items:center; font-style:normal; font-family:var(--font-display);
  font-weight:800; font-size:9.5px; color:var(--paper); background:var(--ink-soft); border:2px solid var(--surface); margin-left:-6px; }
.b-event .ppl i:first-child{ margin-left:0; }
```

Hover inside an account tile: `rotate(-6deg) scale(1.08)` (§27). Unpressed account tile:
`saturate(.55)` + opacity .85 (§27, filter replaced). No other states.

**Motion.** Only inside AccountTile (§27).

**Blitz notes.** The hue hash is computed in Rust and emitted as an inline custom property;
`hsl()` in inline style is a colour function the lint bans in consumer CSS, so the Rust side
converts to hex (O-7).

### 12. Tabs

**Purpose.** Switch between pages of one surface. Only in C (the page's own section tabs,
`C:1167-1171`). ds use: settings pages, control-center pages.

**Markup.** (from `C:1167-1170`)

```html
<div class="ds-tabs" role="tablist" aria-label="Sections">
  <button type="button" class="ds-tab" role="tab" aria-selected="true" aria-controls="panel-a">Inbox</button>
  <button type="button" class="ds-tab" role="tab" aria-selected="false" aria-controls="panel-b">Motion catalogue</button>
</div>
```

**Props.**

```rust
#[component] pub fn Tabs<T: Clone + PartialEq + 'static>(label: String,
    tabs: Vec<(T, String)>, value: T, onchange: EventHandler<T>) -> Element
```

**Geometry.** Bar: flex, gap 6, 1px `--line` bottom border (margins `26px 0 14px` are page layout,
not the component). Tab: padding `10px 14px 12px`, display 16 / 700, `--ink-faint`. Underline:
3px high, inset 10px left and right, bottom -1px (sits on the bar's border), radius
`999px 999px 0 0`, `--accent`.

**States.** Verbatim, `C:243-257`:

```css
.tabs{ display:flex; gap:6px; margin:26px 0 14px; border-bottom:1px solid var(--line); }
.tabs button{
  position:relative; border:0; background:transparent; cursor:pointer;
  padding:10px 14px 12px; font-family:var(--font-display); font-weight:700; font-size:16px;
  color:var(--ink-faint); transition:color var(--t-quick) var(--e-out);
}
.tabs button:hover{ color:var(--ink-soft); }
.tabs button[aria-selected="true"]{ color:var(--ink); }
.tabs button::after{
  content:""; position:absolute; left:10px; right:10px; bottom:-1px; height:3px;
  border-radius:999px 999px 0 0; background:var(--accent);
  transform:scaleX(0); transform-origin:50% 100%;
  transition:transform var(--t-move) var(--e-spring);
}
.tabs button[aria-selected="true"]::after{ transform:scaleX(1); }
```

| State | Rule |
| --- | --- |
| default | `--ink-faint`, underline scaleX(0) |
| hover | `--ink-soft` |
| active | not specified |
| focus-visible | global ring |
| selected | `--ink`, underline scaleX(1) |
| disabled | not specified (O-1) |

**Motion.** Underline grows in place (`--t-move --e-spring`); the old tab's shrinks at the same
time. The plan's "spring underline" means this; there is no sliding underline between tabs.

**Behaviour.** Click selects. Arrow keys: not specified in C (06-INTERACTIONS decides).

**Blitz notes.** `::after` pseudo-elements must be verified (spike); fallback is a real
`<span class="ds-tab-underline">`.

### 13. SectionHeader

**Purpose.** A small-caps label that names a group. Four kinds, one component.

| Kind | Where | Prototype |
| --- | --- | --- |
| Frame | sidebar groups on the Space colour (Places, Pinned, Today) with a trailing rule and an optional action button ("Clear") | S `.s-h` |
| Group | list group headers on the card, with a count and a trailing rule | C `.grp` |
| Field | a label above a control, with an optional right-aligned value | S `.ed-label` |
| Menu | group title inside a menu or palette | S `.fmenu .g`, `.cmdk-g`; C `.menu h6` |

**Markup.**

```html
<div class="ds-section-header" data-kind="frame|group|field|menu">
  <span>Today</span>
  <span class="ds-section-header-value">35</span>                 <!-- group count / field value -->
  <button type="button" class="ds-section-header-action">Clear</button>   <!-- frame only -->
</div>
```

**Props.**

```rust
#[component] pub fn SectionHeader(kind: HeaderKind, text: String,
    #[props(default)] value: Option<String>,
    #[props(default)] action: Option<(String, EventHandler<()>)>) -> Element
```

**Geometry.**

| Kind | Padding | Font | Tracking | Colour | Rule |
| --- | --- | --- | --- | --- | --- |
| Frame | 14px 6px 5px, gap 6 | data 10 upper | .14em | `--f-ink-faint` | 1px `--f-line`, flex 1, after the text |
| Group | 12px 6px 6px, gap 8 | data 10 upper | .12em | `--ink-faint` | 1px `--line-soft`, flex 1 |
| Field | margin-bottom 6 | data 10 upper | .14em | `--ink-faint` | none; value `margin-left:auto`, tracking .02em, no upper |
| Menu | 7px 8px 3px (palette 8px 10px 4px; C 5px 9px 6px) | data 9.5 upper | .14em (C .13em) | `--ink-faint` | none |

**States.** Frame (verbatim, `S:120-125`):

```css
.s-h{ display:flex; align-items:center; gap:6px; padding:14px 6px 5px; font-family:var(--font-data);
  font-size:10px; letter-spacing:.14em; text-transform:uppercase; color:var(--f-ink-faint); }
.s-h::after{ content:""; flex:1; height:1px; background:var(--f-line); }
.s-h button{ order:2; border:0; background:none; cursor:pointer; font:inherit; letter-spacing:.08em;
  color:var(--f-ink-faint); padding:0 2px; text-transform:none; }
.s-h button:hover{ color:var(--f-ink); }
```

Group (verbatim, `C:1026-1032`):

```css
.grp{
  display:flex; align-items:center; gap:8px; padding:12px 6px 6px;
  font-family:var(--font-data); font-size:10px; letter-spacing:.12em; text-transform:uppercase;
  color:var(--ink-faint);
}
.grp::after{ content:""; flex:1; height:1px; background:var(--line-soft); }
.grp .n{ font-variant-numeric:tabular-nums; }
```

Field (verbatim, `S:250-251`):

```css
.ed-label{ font-family:var(--font-data); font-size:10px; letter-spacing:.14em; text-transform:uppercase; color:var(--ink-faint); margin-bottom:6px; display:flex; align-items:center; }
.ed-label .r{ margin-left:auto; letter-spacing:.02em; text-transform:none; }
```

Menu (verbatim, `S:668`, `S:238`, `C:1011-1012`):

```css
.fmenu .g{ padding:7px 8px 3px; font-family:var(--font-data); font-size:9.5px; letter-spacing:.14em; text-transform:uppercase; color:var(--ink-faint); }
.cmdk-g{ padding:8px 10px 4px; font-family:var(--font-data); font-size:9.5px; letter-spacing:.14em; text-transform:uppercase; color:var(--ink-faint); }
.menu h6{ margin:0; padding:5px 9px 6px; font-family:var(--font-data); font-size:9.5px;
  letter-spacing:.13em; text-transform:uppercase; color:var(--ink-faint); }
```

Only the Frame action button has a state (hover `--f-ink`). S's Frame rule sits after the action
button because the button has `order:2`; the rule (`::after`) is order 0 by default, so visual
order is text, rule, button. Keep that order.

**Blitz notes.** `::after` rule: same pseudo-element check as Tabs; fallback a real
`<span class="ds-section-header-rule">`.

### 14. Count

**Purpose.** An unread or item count. Empty at zero. Bumps when it changes. Mail: sidebar items,
account tiles. Shell: dock badges, bar indicators.

**Markup.**

```html
<span class="ds-count a-bump" data-place="tile|item" data-pulse="a|b">4</span>
```

(The zero case renders the element with empty text, `S:1247` `(n || "")`, so layout does not shift.)

**Props.** `#[component] pub fn Count(value: u32, #[props(default)] place: CountPlace /* Item | Tile */) -> Element`
The component fires its own bump when `value` changes between renders (it keeps the last value
it drew; no `PulseKey` prop, wave 1 amendment), except during a Space switch (`S:1262`: counts
bump only when `dir` is falsy). For that render the consumer gives the count a new `key`, so it
mounts at rest (O-8).

**Geometry.**

| Place | Font | Colour | Position |
| --- | --- | --- | --- |
| Item | data 10.5, tabular (C) | `--f-ink-faint` (C on card: `--ink-faint`) | `margin-left:auto` |
| Tile | data 9 | `--f-ink-soft` | absolute top 4 right 5 |

**States.** Verbatim, `S:136`, `S:117`, `S:341-342`:

```css
.item .count{ margin-left:auto; font-family:var(--font-data); font-size:10.5px; color:var(--f-ink-faint); }
.pin .n{ position:absolute; top:4px; right:5px; font-family:var(--font-data); font-size:9px; color:var(--f-ink-soft); }
.count.bump, .pin .n.bump{ display:inline-block; animation:bump var(--t-move) var(--e-spring); }
```

C adds `font-variant-numeric:tabular-nums` (`C:308-311`); adopt it. C Candy forces counts to
`--ink-faint` ("the unread count loses its alarm colour", `C:932-933`); S already does.

**Motion.** `bump --t-move --e-spring` on change.

**Blitz notes.** Restart via `use_pulse` A/B alias, not class removal + reflow.

### 15. Spinner

**Purpose.** Activity without a known end. Two kinds from C's sync halo: Spin (work in
progress) and Breathe (idle but live). Mail: account sync. Shell: bar sync, network connecting.

**Markup.**

```html
<span class="ds-spinner" data-kind="spin|breathe" aria-hidden="true"></span>
```

It is drawn as a ring around its parent (the halo is `inset:-4px` of a 30 px avatar). Standalone
size: not specified (O-9).

**Props.** `#[component] pub fn Spinner(kind: SpinnerKind /* Spin | Breathe */) -> Element`

**Geometry.** absolute, `inset:-4px` of the host, radius 999px, 2px `--accent` border. Spin: border
dashed, opacity .9.

**States.** Verbatim, `C:284-292`:

```css
.acct-ring .halo{
  position:absolute; inset:-4px; border-radius:999px; border:2px solid var(--accent);
  opacity:0; transform:scale(.85);
}
.acct[data-sync="idle"] .halo{ animation:breathe var(--t-ambient) ease-in-out infinite; }
.acct[data-sync="busy"] .halo{
  opacity:.9; transform:scale(1); border-style:dashed;
  animation:spin 1.1s linear infinite;
}
```

**Motion.** `breathe` over `--t-ambient` (5 s Post), `ease-in-out`, infinite; `spin` 1.1 s linear
infinite. Both loop, contradicting "nothing loops" (A8 #7); the plan keeps them as the only
loops besides the destination pulse (§19). `ease-in-out` and `1.1s` are not tokens yet (O-3).

**Blitz notes.** Infinite animations keep `is_animating()` true and cost frames forever. Breathe
at idle means the bar never idles. See O-9: proposal is Breathe off by default on shell surfaces.

### 16. ListRow

**Purpose.** One item in a list: the thread row. Carries the unread dot, text lines, tail
(time, chips, attachment count), star, and a HoverStrip slot. Animates in on first show, out per
operation, and closes gaps. Mail: the thread list. Shell: notification list, launcher results
(planned).

**Markup.** (from `S:1277-1289`)

```html
<ul class="ds-list" role="listbox" aria-label="Threads" data-presence="entering|present">
  <li class="ds-row" role="option" aria-selected="false" data-emphasis="strong|plain"
      data-presence="present|leaving|healing" data-exit="fold|curl|crumple" style="--i:0; --dy:79px; --d:0">
    <div class="ds-row-dot"><span class="ds-dot"></span></div>
    <div class="ds-row-main">
      <div class="ds-row-from">
        <span class="ds-row-name ds-truncate">Dana Okafor</span>
        <span class="ds-row-via"><span class="ds-provider" data-size="row">…</span>gmail</span>
      </div>
      <div class="ds-row-sub ds-truncate">Re: UIDL stability…</div>
      <div class="ds-row-snip ds-truncate">Treat UIDL as…</div>
    </div>
    <div class="ds-row-tail">
      <span class="ds-row-time">09:41</span>
      <span class="ds-row-tags"><span class="ds-clip">…</span><span class="ds-chip">spec</span></span>
    </div>
    <button type="button" class="ds-star" aria-pressed="false" aria-label="Star this thread">
      <svg class="ds-ic">…star…</svg>
      <span class="ds-sparks"><i style="--a:0deg"></i>…six, --a 0..300 step 60…</span>
    </button>
    <!-- HoverStrip slot, §17 -->
  </li>
</ul>
```

Mapping: `.row[data-read=unread|read]` -> `[data-emphasis=strong|plain]`; `.list.entering` ->
`.ds-list[data-presence=entering]`; `.row.going[data-op]` -> `[data-presence=leaving][data-exit]`;
`.row.healing` -> `[data-presence=healing]`; `.star[data-on]` -> `.ds-star[aria-pressed]`.
The hover-card hooks (`data-hc` on row, name, time) become `HoverTarget` wrappers (§22).

**Props.**

```rust
#[component] pub fn ListRow(
    selection: Selection, emphasis: Emphasis, index: StaggerIndex,
    presence: Presence,                        // from use_roster: Entering | Present | Leaving(Exit) | Healing{dy, d}
    name: String, via: Option<Element>, subject: String, snippet: Option<String>,
    time: String, tags: Element,               // chips + clip
    star: Option<(Switch, EventHandler<Switch>)>, star_pulse: PulseKey,
    strip: Option<Element>,                    // HoverStrip
    onclick: EventHandler<MouseData>,          // consumer reads shift for peek
    #[props(default)] drop: DropState,         // Idle | Target (data-drop) | Source (data-drag), §34
) -> Element
```

**Geometry.**

| Part | Value | Source |
| --- | --- | --- |
| Row grid | `10px minmax(0,1fr) auto`, gap 10, align start | `S:176` |
| Row padding / margin | 10px 11px / bottom 5px (Candy 13px 13px) | `S:177` |
| Row radius | `--r-card` (12 12 12 4) | `S:177` |
| Row border | 1px transparent | `S:178` |
| Row height | about 74 (heal distance = height + 5) | `S:1537` |
| Dot column | padding-top 6 (C 5); dot 8 x 8 round `--accent` | `S:185-186` |
| From line | flex baseline, gap 7 | `S:189` |
| Name | ui 13.5 / 700 (read: 500, `--ink-soft`) | `S:190-191` |
| Via | data 9.5 `--ink-faint`, flex none; provider mark 11 | `S:192`, `S:556` |
| Subject | ui 13.5 / line 1.35 (unread 600) | `S:193-194` |
| Snippet | ui 12.5 `--ink-faint` | `S:195` |
| Tail | column, align end, gap 5; tags row gap 4 | `S:196`, `S:1281` |
| Time | data 10 `--ink-faint` tabular | `S:197` |
| Star | absolute left 6 bottom 7 (C 8/8), 22 x 22; icon 14 (C 15) | `S:299-302` |
| Sparks | six 3 x 3 round `--warn` dots from the centre | `S:305-306` |

**States.**

Row base, hover, selected (verbatim, `S:175-184`):

```css
.row{
  position:relative; display:grid; grid-template-columns:10px minmax(0,1fr) auto; gap:10px; align-items:start;
  padding:10px 11px; margin-bottom:5px; border-radius:var(--r-card); background:var(--surface);
  border:1px solid transparent; cursor:pointer; user-select:none;
  transition:transform var(--t-quick) var(--e-out), box-shadow var(--t-quick) var(--e-out),
             background-color var(--t-quick) var(--e-out), border-color var(--t-quick) var(--e-out);
  animation:rise var(--t-move) var(--e-out) backwards; animation-delay:calc(var(--i,0) * var(--stagger));
}
.row:hover{ transform:translateY(var(--lift)); box-shadow:var(--shadow-2); border-color:var(--line); }
.row[aria-selected="true"]{ background:var(--raise); border-color:var(--accent); box-shadow:var(--shadow-1); }
```

Entering only on first show (verbatim, `S:293-296`; overrides the `animation` above):

```css
.row{ animation:none; }
.list.entering .row{ animation:rise var(--t-move) var(--e-out) backwards; animation-delay:calc(var(--i,0) * var(--stagger)); }
.dot{ transition:transform var(--t-quick) var(--e-spring), opacity var(--t-quick) var(--e-out); }
.row[data-read="read"] .dot{ transform:scale(0) translateX(-6px); }
```

Read/unread text (verbatim, `S:187`, `S:191`, `S:194`):

```css
.row[data-read="read"] .dot{ transform:scale(0); opacity:0; }
.row[data-read="read"] .nm{ font-weight:500; color:var(--ink-soft); }
.row[data-read="unread"] .row-sub{ font-weight:600; }
```

Leaving and healing (verbatim, `S:326-333`):

```css
.row.going{ pointer-events:none; }
.row.going[data-op="archive"]{ animation:fold var(--t-big) cubic-bezier(.55,0,.75,.2) forwards; }
.row.going[data-op="snooze"]{ animation:curl 560ms cubic-bezier(.55,0,.75,.2) forwards; }
.row[data-read="unread"].going{ animation-duration:calc(var(--t-big) * 1.15); }
.row.healing{ animation:heal var(--t-move) var(--e-spring) backwards; animation-delay:calc(var(--d,0) * 18ms); }
```

C's third exit (verbatim, `C:714`): `.row.is-going[data-op="trash"]{ animation:crumple var(--t-big) var(--e-exit) forwards; }`.
S has no trash op (A8 #9); ds keeps `Exit::Crumple` for delete.

Restored by undo (inline style, `S:1560`): `rise var(--t-big) var(--e-spring) backwards`.
C arrival (verbatim, `C:373`): `.row.is-entering{ animation:row-in var(--t-big) var(--e-spring) backwards; }`.
C dragging (verbatim, `C:374`): `.row.is-dragging{ opacity:.35; }` (§34).

Star (verbatim, `S:299-307`):

```css
.star{ position:absolute; left:6px; bottom:7px; width:22px; height:22px; border:0; padding:0; background:transparent;
  cursor:pointer; display:grid; place-items:center; opacity:0; transition:opacity var(--t-quick) var(--e-out); }
.row:hover .star, .star[data-on="true"]{ opacity:1; }
.star .ic{ width:14px; height:14px; stroke:var(--ink-faint); transition:stroke var(--t-quick) var(--e-out), fill var(--t-quick) var(--e-out); }
.star[data-on="true"] .ic{ stroke:var(--warn); fill:var(--warn); }
.star.pop .ic{ animation:star-pop var(--t-big) var(--e-spring); }
.sparks{ position:absolute; inset:0; pointer-events:none; }
.sparks i{ position:absolute; left:50%; top:50%; width:3px; height:3px; border-radius:999px; background:var(--warn); opacity:0; }
.sparks.go i{ animation:spark 520ms var(--e-out) forwards; }
```

Candy star (verbatim, `C:955-956`): `stroke:var(--c-amber-deep); fill:var(--c-amber);` and amber
sparks. S wins (`--warn`) unless the Look is Candy (07-LOOKS).

Snooze floater "zZ" (verbatim, `S:334-335`): fixed, z 40, display 800 14, `--accent`,
`floatup 900ms var(--e-out) forwards`; placed at row right - 70, row top + 8 (`S:1536`); removed
at 950 ms.

| State | Rule |
| --- | --- |
| default | `--surface`, transparent border, star hidden, strip hidden |
| hover | lift `--lift`, `--shadow-2`, `--line` border; star and strip appear |
| active | not specified |
| focus-visible | global ring on the row; C also reveals the strip on `:focus-within` (`C:430`) |
| selected | `--raise`, `--accent` border, `--shadow-1` |
| plain (read) | dot scaled out, name 500 `--ink-soft`, subject 400 |
| entering (list first shown) | `rise --t-move --e-out backwards`, delay `--i * --stagger` |
| entering (restored by undo) | `rise --t-big --e-spring backwards` |
| entering (new arrival, C) | `row-in --t-big --e-spring backwards` |
| leaving | `pointer-events:none`; archive `fold --t-big --e-exit`, snooze `curl 560ms --e-exit`, delete `crumple --t-big --e-exit`; unread x1.15 duration |
| healing | `heal --t-move --e-spring backwards`, delay `--d * 18ms`, from `translateY(--dy)` |
| disabled | not specified |

**Motion.** Keyframes `rise`, `row-in`, `fold`, `curl`, `crumple`, `heal`, `star-pop`, `spark`,
`floatup`; tokens `--t-quick --t-move --t-big --e-out --e-spring --e-exit --lift --stagger`;
named durations 520 (`--t-spark`), 560 (`--t-curl`), 900 (`--t-float`); heal step 18 ms
(`HealStep`). Sparks only when starring, never when unstarring (`S:1526`).

**Behaviour.** Click selects and opens (marks read, adds to Today); Shift-click peeks
(`S:1511`); star toggles and pulses; the strip's buttons act. Rows stay in the roster until
`settle(exit)` elapses, then the rows below heal (`S:1537-1545`: `dy = height + 5`, index `d`
counts from the removed row). Hover never marks read. 06-INTERACTIONS "row lifecycle".

**Blitz notes.**
- `animationend` + 900 ms fallback (`S:1545`) -> `use_roster` with `settle()`.
- `curl` ends with `filter:saturate(.2)` (`S:332`): banned; drop the filter, keep transform and
  opacity (05-MOTION owns the keyframe).
- Name, subject, snippet: `.ds-truncate`.
- Restart of `star-pop` / `spark` via `use_pulse`.
- Hover lift on many rows: transforms only on the hovered row (plan perf rule), stagger capped 12.
- The floater is `position:fixed` in S; in ds it renders through OverlayHost at a Rust-computed
  point.

### 17. HoverStrip

**Purpose.** A pill of icon buttons that appears on a row while it is hovered. Each button
previews its result through a Fly tooltip. Mail: archive, snooze, label, read toggle
(`S:1286`); C varies per view (A4). Shell: notification actions (planned).

**Markup.** (from `S:1286-1288`)

```html
<div class="ds-strip">
  <button type="button" class="ds-icon-button" data-variant="strip" data-op="archive"
          style="--j:0" aria-label="Archive">
    <svg class="ds-ic">…</svg>
    <span class="ds-fly">Archive → out of Inbox</span>
  </button>
  …
</div>
```

**Props.**

```rust
#[component] pub fn HoverStrip(actions: Vec<StripAction>) -> Element
pub struct StripAction { id: ActionId, icon: Icon, label: String, fly: String,
    onhover: Option<EventHandler<Here>>,   // destination preview (§19 dest)
    onclick: EventHandler<Rect> }          // rect anchors snooze/label menus
```

**Geometry.** Strip: absolute right 8, vertically centred, flex gap 3 (C 4), padding 3, radius
999, `--raise` bg, 1px `--line`, `--shadow-2`. Buttons 26 x 26 (C 28) round, `--ink-soft`,
icons 14 (C 15). A7: "strip 26 gap 3" is S; S wins.

**States.** Verbatim, `S:312-323`:

```css
.strip{ position:absolute; right:8px; top:50%; transform:translateY(-50%); display:flex; gap:3px; padding:3px;
  border-radius:999px; background:var(--raise); border:1px solid var(--line); box-shadow:var(--shadow-2);
  opacity:0; pointer-events:none; transition:opacity var(--t-quick) var(--e-out); }
.row:hover .strip{ opacity:1; pointer-events:auto; }
.strip button{ width:26px; height:26px; border:0; border-radius:999px; background:transparent; cursor:pointer;
  display:grid; place-items:center; color:var(--ink-soft); transform:scale(.7); opacity:0;
  transition:background-color var(--t-quick) var(--e-out), color var(--t-quick) var(--e-out); }
.row:hover .strip button{ animation:pop-in var(--t-move) var(--e-spring) forwards; animation-delay:calc(var(--j) * var(--stagger)); }
.strip button:hover{ background:var(--accent-soft); color:var(--ink); }
.strip button:active{ transform:scale(.88) !important; }
.strip .ic{ width:14px; height:14px; }
```

`.strip button{ position:relative; }` (`S:441`) so the Fly anchors to its button.

Candy per-op hover (verbatim, `C:960-966`): trash/purge hover `--c-red-soft` / `--c-red-deep`;
restore hover `--c-green-soft` / `--c-green-deep`. In ds: `data-tone="danger|restore"` on the
strip button, used by the Danger rule of §1 and a derived restore rule.

| State | Rule |
| --- | --- |
| hidden | strip opacity 0, no pointer events; buttons scale(.7), opacity 0 |
| shown (row hover; C also row focus-within) | strip opacity 1; buttons `pop-in` staggered by `--j` |
| button hover | `--accent-soft` bg, `--ink` |
| button active | scale(.88) |
| focus-visible | global ring on the button |
| leaving | strip fades out (`--t-quick`); buttons snap back to scale(.7) (no exit animation) |

**Motion.** `pop-in --t-move --e-spring forwards`, delay `--j * --stagger`; strip opacity
`--t-quick --e-out`.

**Behaviour.** Hovering archive or snooze lights the destination sidebar item (`dest`, §19)
until the pointer leaves; any click clears it (`S:1832-1839`). Snooze and label open a slim Menu
anchored at the button's rect (`S:1504-1509`). 06-INTERACTIONS "destination preview".

**Blitz notes.**
- `!important` on `:active` exists to beat the `forwards` fill of `pop-in`; the lint bans
  `!important`. Replacement: after `settle(pop-in)` the component swaps to a `data-shown="settled"`
  state with no animation and `transform:scale(1)`, so `:active{transform:scale(.88)}` wins
  without `!important` (O-10).
- Reveal on `.row:hover` is pure CSS and stays; keyboard reveal uses `:focus-within` (from C).

### 18. Tooltip

**Purpose.** Text that names an action or a value. Two kinds: Fly (a small dark label above a
strip button; says what the action will do, "snooze names the time", `S:440`) and Card (a small
hover card for a value, e.g. the full date on a row time, `S:1747-1748`). Shell: dock labels, bar
tooltips.

**Markup.**

```html
<!-- Fly: child of its button -->
<span class="ds-fly" role="tooltip">Snooze until…</span>

<!-- Card: through OverlayHost, driven by HoverHub -->
<div class="ds-popover ds-tip" role="tooltip">Wed 23 Sep 2026, 09:41
  <div class="ds-tip-sub">10:41 their time (Lagos)</div></div>
```

Mapping: `.strip .fly` -> `.ds-fly`; `.hc.tip` -> `.ds-popover.ds-tip`.

**Props.**

```rust
#[component] pub fn Tooltip(kind: TooltipKind /* Fly | Card */, text: String,
    #[props(default)] sub: Option<String>,
    #[props(default)] shown: Option<Shown> /* Visible | Hidden: the caller's say */,
    children: Element /* the target */) -> Element
```

**Geometry.**

| Kind | Box | Font | Colours | Position |
| --- | --- | --- | --- | --- |
| Fly | padding 3px 7px, radius 6 | data 10, nowrap | `--paper` on `--ink` | centred above the target, bottom `calc(100% + 7px)` |
| Card | auto width, max 260, padding 6px 10px, radius 9 | ui 12; sub data 10.5 `--ink-faint`, margin-top 1 | hover-card surface (§22) | below the target: x = left, y = bottom + 6, clamp 8 |

**States.** Fly (verbatim, `S:442-446`):

```css
.strip .fly{ position:absolute; bottom:calc(100% + 7px); left:50%; transform:translateX(-50%) translateY(3px); white-space:nowrap;
  background:var(--ink); color:var(--paper); font-family:var(--font-data); font-size:10px; padding:3px 7px; border-radius:6px; opacity:0;
  pointer-events:none; transition:opacity var(--t-quick) var(--e-out) 0ms, transform var(--t-quick) var(--e-spring) 0ms; }
.strip button:hover .fly{ opacity:1; transform:translateX(-50%) translateY(0); transition-delay:var(--fly-delay, 350ms); }
.win.warm .strip button:hover .fly{ --fly-delay:0ms; }
```

Card (verbatim, `S:426-427`, on top of the `.hc` base in §22):

```css
.hc.tip{ width:auto; max-width:260px; padding:6px 10px; font-size:12px; border-radius:9px; }
.hc.tip .sub{ margin-top:1px; }
```

| State | Fly | Card |
| --- | --- | --- |
| hidden | opacity 0, translateY(3px) | not mounted |
| entering | after 350 ms (`--d-fly`), or at once when warm: opacity 1, translateY(0) | `hc-in --t-move --e-spring` after HoverOpen 450 ms (0 when warm) |
| leaving | immediate (0 ms delay on the way out) | `hc-out 120ms --e-out`, removed at 130 |

**Motion.** Fly: opacity `--t-quick --e-out`, transform `--t-quick --e-spring`, delay `--d-fly`.
Card: see §22.

**Behaviour.** Warm window: for 400 ms after any hover card closes, flies open at once and the
next card opens without the 450 ms wait (`S:1711-1712`). 06-INTERACTIONS "hover intent".
Caller-driven (settled 2026-09-24, FINDINGS "Launcher gaps", sill Q17): `shown: Some(Visible)`
shows the label with no pointer on it and `Some(Hidden)` keeps it down under one, both at once
with no delay of their own (`[data-shown]` on `.ds-fly-target`); the dock's label machine (hide
on press, while a menu is open, while dragging) owns its timing. `None` is the hover behaviour.

**Blitz notes.** The warm state is a class on `.win` set by JS. In ds, `HoverHub` owns it and
stamps `data-hover="warm|cold"` on `.ds`; the rule becomes
`.ds[data-hover=warm] .ds-fly{ --fly-delay:0ms }` (proposal, O-11). The Card is placed by
`place()`, not measured with `offsetHeight`.

**Size (the macOS polish pass, settled 2026-09-24).** Both labels are `--fs-shell-tip` 12 px (the
Fly was 10); the Fly keeps the data face.

### 19. SidebarItem

**Purpose.** A navigable place in the sidebar, drawn on the Space colour. Kinds: Place (Inbox,
Starred, …, labels; carries the seal when current), Pinned (person or saved search with a
favicon), Today (an opened thread or draft, behaves like a tab: slides in, has a close button,
collapses when closed). Receives drops (gulp), previews destinations (dest). Mail: sidebar. Shell:
settings navigation, launcher categories (planned).

**Markup.** (from `S:1242-1256`)

```html
<button type="button" class="ds-sidebar-item" data-kind="place" aria-current="true">
  <svg class="ds-ic">…inbox…</svg><span>Inbox</span><span class="ds-count">4</span>
</button>

<button type="button" class="ds-sidebar-item" data-kind="pinned">
  <span class="ds-avatar" data-size="16" data-shape="square">D</span>
  <span class="ds-truncate" data-emphasis="plain">Dana Okafor</span><span class="ds-count">2</span>
</button>

<div class="ds-sidebar-item" data-kind="today" role="button" tabindex="0" data-presence="entering|present|leaving">
  <span class="ds-avatar" data-size="16" data-shape="square">D</span>
  <span class="ds-truncate" data-emphasis="plain">Re: UIDL stability…</span>
  <button type="button" class="ds-sidebar-item-close" aria-label="Close"><svg class="ds-ic">…x…</svg></button>
</div>
```

Pulses on the item: `a-gulp` (received something), `a-dest` (destination preview, loops while
hovered). Mapping: `.item` -> `.ds-sidebar-item`; `.item.pinned` -> `[data-kind=pinned]`;
`.today-item` -> `[data-kind=today]`; `.item.gulp` -> pulse `a-gulp`; `.item.dest` ->
`[data-preview=destination]`; `.entering/.leaving` -> `data-presence`.

**Props.**

```rust
#[component] pub fn SidebarItem(kind: ItemKind /* Place{icon} | Pinned{avatar} | Today{avatar} */,
    label: String, here: Here, count: Option<u32>,
    presence: Presence, preview: Option<Preview /* Destination */>, pulse: PulseKey,
    onclick: EventHandler<()>, onclose: Option<EventHandler<()>>,
    #[props(default)] drop: DropState /* Idle | Target | Source, §34 */) -> Element
```

**Geometry.**

| Part | Value | Source |
| --- | --- | --- |
| Item | flex, gap 9, width 100%, padding 6px 8px, radius 9 (`--r-item`) | `S:127-128` |
| Text | ui 13.5 / 600, `--f-ink-soft` | `S:128-129` |
| Pinned / Today text | weight 500 | `S:558`, `S:137` |
| Icon | 16 | `.ic` |
| Favicon | 16 x 16 r5, display 800 9 white (draft: `--ink` bg, pen icon 10 white) | `S:138-139`, `S:724`, `S:1250` |
| Close (`.x`) | `margin-left:auto`, icon 13, `--f-ink-faint`, opacity 0 until hover | `S:140-143` |
| Seal | 5 x 5 round `--f-ink`, left -7, vertically centred | `S:345-346` |
| Count | §14 | |

C `.view` (card-coloured sidebar): padding 7px 9px (Candy 9px 11px), radius `--r-chip`, hover and
current on `--raise`, seal left -10 in `--seal` (`C:300-319`). S wins.

**States.** Base, hover, current (verbatim, `S:126-143`):

```css
.item{
  display:flex; align-items:center; gap:9px; width:100%; text-align:left; cursor:pointer; border:0;
  background:transparent; padding:6px 8px; border-radius:9px; font-size:13.5px; font-weight:600;
  color:var(--f-ink-soft); position:relative;
  transition:background-color var(--t-quick) var(--e-out), color var(--t-quick) var(--e-out), transform var(--t-tap) var(--e-out);
}
.item:hover{ background:var(--f-pill-hover); color:var(--f-ink); }
.item:active{ transform:scale(var(--squish)); }
.item[aria-current="true"]{ background:var(--f-pill); color:var(--f-ink);
  box-shadow:0 1px 0 rgba(255,255,255,.4) inset, 0 2px 6px -3px rgba(0,0,0,.25); }
.item .count{ margin-left:auto; font-family:var(--font-data); font-size:10.5px; color:var(--f-ink-faint); }
.item .t{ white-space:nowrap; overflow:hidden; text-overflow:ellipsis; min-width:0; font-weight:500; }
.item .fav{ width:16px; height:16px; border-radius:5px; flex:none; display:grid; place-items:center;
  font-family:var(--font-display); font-weight:800; font-size:9px; color:#fff; }
.item .x{ margin-left:auto; opacity:0; border:0; background:none; cursor:pointer; padding:0; color:var(--f-ink-faint);
  transition:opacity var(--t-quick) var(--e-out); }
.item:hover .x{ opacity:1; }
.item .x .ic{ width:13px; height:13px; }
```

Seal (verbatim, `S:345-346`), only on Place items:

```css
.item[data-place][aria-current="true"]::before{ content:""; position:absolute; left:-7px; top:50%; width:5px; height:5px;
  border-radius:999px; background:var(--f-ink); transform:translateY(-50%) scale(0); animation:seal-pop var(--t-big) var(--e-spring) forwards; }
```

Gulp, dest, Today presence (verbatim, `S:339`, `S:447-448`, `S:350-351`):

```css
.item.gulp{ animation:gulp var(--t-big) var(--e-spring); }
.item.dest{ background:var(--f-hover); color:var(--f-ink); box-shadow:0 0 0 1.5px var(--accent) inset; animation:dest 900ms var(--e-out) infinite alternate; }
.today-item.entering{ animation:tab-in var(--t-big) var(--e-spring); }
.today-item.leaving{ animation:tab-out var(--t-move) cubic-bezier(.55,0,.75,.2) forwards; overflow:hidden; }
```

Bug (A8 #1): `--f-hover` is undefined; use `--f-pill-hover`. `color-mix` inside `dest`'s keyframe
becomes a precomputed token (05-MOTION).

C drop target (verbatim, `C:320-323`), adopted as `[data-drop=target]` (§34):

```css
.view.is-drop-target{
  background:var(--accent-soft); color:var(--ink);
  transform:scale(1.045); box-shadow:var(--shadow-1);
}
```

C Candy label hue (verbatim, `C:950-951`): the icon and seal take `--tag`; the row stays grey.

| State | Rule |
| --- | --- |
| default | transparent, `--f-ink-soft` |
| hover | `--f-pill-hover` bg, `--f-ink`; close button appears |
| active | scale(`--squish`) |
| focus-visible | global ring |
| current | `--f-pill`, `--f-ink`, current-item shadow; Place: seal pops in |
| destination preview | `--f-pill-hover`, `--f-ink`, inset accent ring pulsing (`dest 900ms`, alternate, infinite) |
| drop target | `--accent-soft`, `--ink`, scale 1.045, `--shadow-1` (from C) |
| received (gulp) | `gulp --t-big --e-spring`, class removed at 460 ms (`S:1520`) |
| entering (Today) | `tab-in --t-big --e-spring` |
| leaving (Today) | `tab-out --t-move --e-exit forwards`, then removed (fallback 400 ms, `S:1485`) |
| disabled | not specified |

**Motion.** `seal-pop`, `gulp`, `dest`, `tab-in`, `tab-out`; tokens `--t-quick --t-tap --t-move
--t-big --e-out --e-spring --e-exit --squish`.

**Behaviour.** Click navigates; the close button closes the Today entry only, never archives
(`S:921`). A Today entry drops after 12 idle hours. Clear empties Today. 06-INTERACTIONS
"sidebar".

**Blitz notes.**
- `::before` seal: pseudo-element check (spike); fallback a real `<span class="ds-seal">`.
- `animationend` + 400 ms fallback for Today close -> `settle(TabOut)`.
- Gulp restart via `use_pulse`, not class removal + reflow.
- `dest` loops while the preview holds; stop it on leave. Uses `color-mix` in keyframes ->
  precomputed `--accent-ring` value.
- The Today item is a `div[role=button]` with a nested button; in ds keep the outer as
  `div[role=button][tabindex=0]` (a button cannot contain a button).
- `.t` truncation -> `.ds-truncate`.

### 20. Menu

**Purpose.** "ONE MENU: every list of choices in the window uses this" (`S:777-780`). A floating
list of entries with keyboard selection, typing to filter, and a pick callback. Kinds:

| Kind | Prototype | Used for |
| --- | --- | --- |
| Rich | S `.fmenu` (280 wide, 34 px tiles, title + one line of help + shortcut) | `/` insert menu (`S:2110`), palette results (§25) |
| Slim | S `.fmenu.slim` (220 wide, 22 px tiles) | snooze, labels, `@` mention, turn-into, from, when, people, object menu (`S:1505-2259`) |
| Dropdown | C `.menu` (min 186 wide, check column, anchored under its button's right edge) | Group by / Show in list (`C:1747-1759`), row picker (`C:1971-1986`); shell bar menus |
| Context | derived (no right-click menu in either prototype) | dock context menu, tray menus, right-click on rows (planned) |

**Markup.**

```html
<div class="ds-popover ds-menu" data-kind="rich|slim|dropdown|context" role="listbox"
     aria-label="Snooze until" data-presence="entering|present">
  <div class="ds-section-header" data-kind="menu">Snooze until</div>          <!-- MenuEntry::Header -->
  <div class="ds-menu-item" role="option" aria-selected="true" aria-checked="true">   <!-- MenuEntry::Item -->
    <span class="ds-menu-tile"><svg class="ds-ic">…clock…</svg></span>
    <span><b class="ds-menu-title">Tomorrow</b><small class="ds-menu-detail">08:00</small></span>
    <span class="ds-menu-trail">⌃S</span>            <!-- shortcut, or a check icon when checked -->
  </div>
  <div class="ds-menu-separator" role="separator"></div>                       <!-- MenuEntry::Separator -->
  <div class="ds-menu-empty">Nothing matches "zz".</div>
</div>
```

Mapping: `.fmenu` -> `.ds-menu[data-kind=rich]`; `.fmenu.slim` -> `[data-kind=slim]`; C `.menu`
-> `[data-kind=dropdown]`; `.it` -> `.ds-menu-item`; `.g` / `h6` -> SectionHeader kind Menu;
`.tile` -> `.ds-menu-tile`; `b` -> `.ds-menu-title`; `small` -> `.ds-menu-detail`; `.sc` / C
`.tail` -> `.ds-menu-trail`; `.none` -> `.ds-menu-empty`.

**Props.**

```rust
#[component] pub fn Menu<T: Clone + PartialEq + 'static>(
    kind: MenuKind,                       // Rich | Slim | Dropdown | Context
    anchor: Anchor,                       // Rect (button) | Point (caret, right-click)
    entries: Vec<MenuEntry<T>>,
    #[props(default)] filter: Filter,     // Typing | None
    onpick: EventHandler<T>, onclose: EventHandler<()>,
    #[props(default)] timing: MenuTiming, // submenu delay and triangle timeout (13 §13.3.4)
    #[props(default)] expanded: Option<usize>, // a choice whose submenu opens on mount
    #[props(default)] on_hover: Option<EventHandler<Option<usize>>>, // settled (bar gaps): choice under the pointer
    #[props(default)] on_release: Option<EventHandler<Press>>,        // settled: every release over the menu
    #[props(default)] entrance: MenuEntrance,                        // settled: Animated | Instant (bar menus)
) -> Element
// Settled (bar gaps, sill Q11): a pick calls `onpick`, then `onclose`, once; Escape and an
// outside click play `menu-out` (`--t-quick --e-exit`, 13 §13.3.2) before `onclose`; a
// release over an enabled item after a press that began outside picks it (press-drag-release).
pub enum MenuEntry<T> {
    Item { value: T, title: String, detail: Option<String>, tile: Option<Tile>, trail: Trail,
           check: Option<Check>, availability: Availability },
    Submenu { title: String, tile: Option<Tile>, availability: Availability, children: Vec<MenuEntry<T>> },
    Header(String),
    Info { title: String, detail: Option<String> }, // settled (bar gaps): a status line, never a choice
    Separator,
}
pub enum Tile { Icon(Icon), Text(String), Avatar(Avatar) }
pub enum Trail { None, Shortcut(Shortcut), Note(String) }
```

**Geometry.**

| Part | Rich | Slim | Dropdown (C) | Context (derived) |
| --- | --- | --- | --- | --- |
| Width | 280 | 220 | min 186, auto | = Slim |
| Max height | 320 (360 in palette), scroll | 320 | not specified | = Slim |
| Padding | 5 | 5 | 6 | 5 |
| Radius | 12 (`--r-menu`) | 12 | `--r-panel` 14 | 12 |
| Surface | `--raise`, 1px `--line`, `0 18px 40px -16px rgba(0,0,0,.45)` | same | `--raise`, 1px `--line`, `--shadow-2` | = Slim |
| Item grid | `34px 1fr auto`, gap 9, padding 5px 7px, radius 8 | `22px 1fr auto`, padding 6px 8px | flex, gap 8, padding 6px 9px, radius `--r-btn` | = Slim |
| Tile | 34 x 34 r8, 1px `--line`, `--surface`, display 800 13 `--ink-soft`; icon 17 | 22 x 22, no border, transparent; round avatar 20 / 9 | check icon 16 (hidden until checked) | = Slim |
| Title | ui 13 / 600 | same | ui 13 `--ink-soft` | same |
| Detail | ui 11.5 / 1.3 `--ink-faint` | same | n/a | same |
| Trail | data 10 `--ink-faint`; check icon 14 `--accent` | same | data 10 `--ink-faint` (`.tail`) | same |
| Empty | padding 10, ui 12.5 `--ink-faint` | same | not specified | same |

**States.** Rich and Slim (verbatim, `S:665-680`, `S:787-791`):

```css
.fmenu{ position:absolute; z-index:40; width:280px; max-height:320px; overflow-y:auto; background:var(--raise); color:var(--ink); border:1px solid var(--line);
  border-radius:12px; box-shadow:0 18px 40px -16px rgba(0,0,0,.45); padding:5px; animation:menu-pop var(--t-move) var(--e-spring); }
.fmenu .it{ display:grid; grid-template-columns:34px 1fr auto; gap:9px; align-items:center; padding:5px 7px; border-radius:8px; cursor:pointer; }
.fmenu .it[aria-selected="true"]{ background:var(--accent-soft); }
.fmenu .it .tile{ width:34px; height:34px; border-radius:8px; border:1px solid var(--line); background:var(--surface); display:grid; place-items:center;
  font-family:var(--font-display); font-weight:800; font-size:13px; color:var(--ink-soft); }
.fmenu .it .tile .ic{ width:17px; height:17px; }
.fmenu .it b{ display:block; font-size:13px; font-weight:600; }
.fmenu .it small{ display:block; font-size:11.5px; color:var(--ink-faint); line-height:1.3; }
.fmenu .it .sc{ font-family:var(--font-data); font-size:10px; color:var(--ink-faint); }
.fmenu .none{ padding:10px; font-size:12.5px; color:var(--ink-faint); }
.fmenu.slim{ width:220px; }
.fmenu.slim .it{ grid-template-columns:22px 1fr auto; padding:6px 8px; }
.fmenu.slim .it .tile{ width:22px; height:22px; border:0; background:transparent; }
.fmenu .tile.round{ border-radius:999px; }
.fmenu.slim .tile.round{ width:20px; height:20px; font-family:var(--font-display); font-weight:800; font-size:9px; }
.fmenu .it[aria-checked="true"] b{ color:var(--ink); }
.fmenu .it .sc .ic{ width:14px; height:14px; color:var(--accent); }
.fmenu mark, .cmdk mark{ background:transparent; color:var(--accent); font-weight:800; }
```

Dropdown (verbatim, `C:1003-1023`):

```css
.menu{
  position:absolute; top:calc(100% + 7px); right:0; z-index:12; min-width:186px;
  background:var(--raise); border:1px solid var(--line); border-radius:var(--r-panel);
  box-shadow:var(--shadow-2); padding:6px;
  transform-origin:88% 0; animation:menu-in var(--t-move) var(--e-spring);
}
.menu button{
  display:flex; align-items:center; gap:8px; width:100%; text-align:left; border:0;
  background:transparent; cursor:pointer; padding:6px 9px; border-radius:var(--r-btn);
  font-size:13px; color:var(--ink-soft);
  transition:background-color var(--t-quick) var(--e-out), color var(--t-quick) var(--e-out);
}
.menu button:hover{ background:var(--surface-2); color:var(--ink); }
.menu button .ic{ opacity:0; }
.menu button[aria-checked="true"]{ color:var(--ink); font-weight:600; }
.menu button[aria-checked="true"] .ic{ opacity:1; color:var(--accent); }
.menu .tail{ margin-left:auto; font-family:var(--font-data); font-size:10px; color:var(--ink-faint); }
```

Separator (derived from the bubble's vertical separator, `S:689`
`.bubble .sep{ width:1px; height:18px; background:var(--line-soft); margin:0 3px; }`, turned
horizontal): 1px high, full width, `--line-soft`; vertical margin not specified (O-12).

| State | Rich / Slim / Context | Dropdown |
| --- | --- | --- |
| item default | transparent | transparent, `--ink-soft` |
| item hover | not specified in S (only keyboard moves the selection); see O-13 | `--surface-2`, `--ink` |
| item selected (keyboard) | `--accent-soft` bg | not specified |
| item checked | title `--ink`; trail shows check 14 `--accent` | `--ink`, 600, check icon visible `--accent` |
| item focus-visible | not used (focus stays in the input that opened the menu) | global ring |
| item disabled | settled (13 §13.3.3): opacity .35, `aria-disabled="true"`, not selectable, skipped by Up/Down, a click does nothing | same, no hover |
| submenu parent | settled (13 §13.3.3-13.3.4): 12 px chevron in the trail, `aria-haspopup`, `aria-expanded`; open keeps the selected look | same, `--surface-2` |
| submenu | settled (13 §13.3.4): a panel of the same kind right of the menu (flips left), top at the parent row's top less the padding, gap 2, no entrance; `data-depth` | same |
| match highlight | `mark`: `--accent`, 800, no background | n/a |
| entering | `menu-pop --t-move --e-spring` | `menu-in --t-move --e-spring`, origin 88% 0 |
| leaving | removed at once (`S:2052`) | removed at once (`C:1744`) |

**Motion.** `menu-pop`, `menu-in` (05-MOTION).

**Behaviour.** (06-INTERACTIONS "menus")
- Opening a menu closes any other floating menu (`S:2069`).
- Keys (S, `S:2089-2098`): Down/Up move the selection and WRAP; Enter or Tab picks; Esc closes
  and stops propagation. Typing filters with the fuzzy ranker and resets the selection to 0
  (`S:2086-2088`). The selected item scrolls into view (nearest).
- Pointer: mousedown inside the menu is prevented so focus stays in the field that opened it
  (`S:2099`). Click on an item closes the menu, then calls pick (`S:2103`). Click outside closes,
  except on `[data-pick]`, a grip, or a strip button (`S:2104`) (those open their own menu).
- Dropdown (C): its button toggles `aria-expanded`; picking a Group closes; toggling a property
  stays open (`C:1765-1766`); outside click closes (`C:1777`).
- Placement (S `placeFloat`, `S:2060-2065`): x = anchor.left - 8, y = anchor.bottom + 6; if it
  overflows the bottom (window height - 8) flip to anchor.top - height - 6; clamp both axes to
  8 px inside the window. Dropdown (C): below the button, right edges aligned, gap 7, no clamp;
  ds uses `place(…, Placement::BottomEnd, gap 7)` with the same flip + clamp.
- Context (derived): anchored at the pointer (`Anchor::Point`), placement BottomStart, gap 0,
  same flip + clamp.

**Blitz notes.**
- `position:absolute` inside `.win` -> OverlayHost; placement by `place()` in Rust, never
  measured with `offsetHeight`.
- `scrollIntoView` -> Rust sets the scroll offset of the menu element.
- `z-index:40` literal -> `--z-menu`.
- On shell surfaces the menu is its own xdg_popup via `PopoverRequest`; the surface background is
  `Material::Popover`, and the menu's own `--raise` fill becomes `--m-tint` (03-COLOR).
- Filtering and fuzzy ranking are pure Rust (06-INTERACTIONS).

**Shell scale (the macOS polish pass, settled 2026-09-24).** Slim, Context and Dropdown are the
text menus (design/13 section 13.3.3): rows at least `--shell-menu-row` 22 px, text
`--fs-shell-menu` 13 at 400, the 22 px check column, the selection an inset highlight at
`--r-shell-highlight` 6 inside the 5 px panel padding, separators 1 px `--line-soft` with
`--shell-menu-sep` 5 px above and below and inset 8 px to the text, disabled rows at .35. Rich
keeps its 34 px tiles. The sizes are tuned tokens (`ShellMetrics`), so `menus.item_height_px`
and its siblings move them live. On a Popover root (the shell's) the card paints the material
stack v2 (03-COLOR section 17.4).

### 21. Popover

**Purpose.** The shared floating surface under Menu (§20), HoverCard (§22), Tooltip Card (§18),
SelectionBubble (§30) and CommandPalette (§25). It owns the surface look, the layer, placement,
Esc and outside-click via `LayerStack`. S has no standalone popover; the plan adds one (placement
in Rust).

**Markup.**

```html
<div class="ds-popover" data-elevation="pop|bubble|sheet" data-layer="menu|card|bubble|palette"
     style="left:120px; top:48px" data-presence="entering|present|leaving">…</div>
```

**Props.**

```rust
#[component] pub fn Popover(anchor: Anchor, placement: Placement, gap: Px,
    #[props(default)] elevation: Elevation /* Pop | Bubble | Sheet */,
    #[props(default)] dismiss: Dismiss /* EscAndOutside | EscOnly | None */,
    onclose: EventHandler<()>, children: Element) -> Element
```

**Geometry.** The surfaces the prototypes draw, one per elevation:

| Elevation | Radius | Shadow (literal) | Used by | Source |
| --- | --- | --- | --- | --- |
| Pop | 12 (menu) / 14 `--r-panel` (card) | `0 18px 40px -16px rgba(0,0,0,.45)` | Menu, HoverCard, side peek (`0 20px 40px -16px`) | `S:666`, `S:400`, `S:454` |
| Bubble | 10 | `0 12px 28px -12px rgba(0,0,0,.45)` | SelectionBubble | `S:684` |
| Sheet | 14 `--r-panel` | `0 30px 60px -20px rgba(0,0,0,.5)` (palette); `0 24px 50px -18px rgba(0,0,0,.55)` (peek) | CommandPalette, Peek | `S:233`, `S:224` |

All: background `--raise` (peek: `--surface-2`), text `--ink`, 1px `--line` border (peek: none).
The plan collapses these shadows into `--shadow-pop` and `--shadow-sheet` (O-3 lists the
literals the table must absorb).

**States.** Entering and leaving belong to the child (menu-pop, hc-in, peek-in). Popover adds
none.

**Behaviour.** Placement: `place(anchor, content, bounds, want, gap)`: flip to the other side
when it does not fit, clamp 8 px from bounds. Hover cards do not flip in S (`S:1715-1726`: they
only clamp); `Placement` carries a `Flip::Never` option for them. Esc and outside click close the
topmost layer only (`LayerStack`).

**Blitz notes.** This component exists so that no consumer writes `position:fixed` or measures
layout. Mount point: OverlayHost at the end of `.ds`. Content size for `place()` comes from
`onmounted` rects (spike S9; fallback LayoutProbe). On shell surfaces: `PopoverRequest`.

**Exit (the macOS polish pass, settled 2026-09-24).** Escape or an outside click marks the popover
`data-presence="leaving"` and fades it out (`menu-out`, `--t-quick`, `--e-exit`) before `onclose`
runs, as a menu does.

### 22. HoverCard

**Purpose.** A preview that opens after the pointer rests on something, and never marks read or
fetches (`S:396`, `S:1697-1703`). Kinds in S: thread, sender (with spoof flag), time (the
Tooltip Card, §18), account, pin, Today. Mail: list rows, sender names, row times, sidebar tiles
and items. Shell: dock window previews (planned).

**Markup.** Target and card are separate. The target wraps whatever it hooks:

```html
<span class="ds-hover-target" data-hover-key="sender:3">Dana Okafor</span>   <!-- was data-hc -->

<div class="ds-popover ds-hovercard" data-kind="thread|sender|account|side" data-presence="entering|present|leaving">
  <div class="ds-hovercard-person"><span class="ds-avatar" data-size="34">D</span>
    <div><h5 class="ds-hovercard-title">Dana Okafor</h5><div class="ds-hovercard-sub">dana@example.com</div></div></div>
  <div class="ds-hovercard-stats"><span><b>14</b>threads</span><span><b>Mon</b>last wrote</span></div>
  <div class="ds-hovercard-flag" data-tone="danger|info"><svg class="ds-ic">…</svg><span>…</span></div>
  <div class="ds-hovercard-msgs">
    <div class="ds-hovercard-msg"><span class="ds-avatar" data-size="22">D</span><div><b>Dana</b><p>…</p></div></div>
  </div>
  <div class="ds-hovercard-foot">stays unread while you look<span class="ds-hovercard-keys"><kbd class="ds-kbd">Space</kbd> peek</span></div>
  <div class="ds-hovercard-actions"><button class="ds-button" data-variant="mini">…</button></div>
</div>
```

**Props.**

```rust
#[component] pub fn HoverTarget(key: HoverKey, kind: HoverKind, children: Element) -> Element
#[component] pub fn HoverCard(
    kind: HoverKind,                       // Thread | Sender | Account | Side
    #[props(default)] parts: Vec<HoverCardPart>,   // drawn in order, before children
    children: Element,
) -> Element
pub enum HoverCardPart {
    Title(String), Sub(String),
    Person { initial: char, tone: AvatarTone, title: String, sub: Option<String> },   // avatar 34
    Stats(Vec<HoverStat>),                 // HoverStat { value, label }
    Flag { tone: FlagTone /* Danger | Info */, icon: Icon, text: String },
    Messages(Vec<HoverMessage>),           // HoverMessage { initial, tone, name, text }, avatar 22
    Foot { text: String, keys: Option<KeyHint> },   // KeyHint { shortcut, label }
    Actions(Vec<Element>),                 // Mini buttons
}
// HoverHub (context) owns Idle -> Pending -> Open -> Closing -> Warm; consumers render the card for the open key.
```

The parts are settled (Gallery fixes B): each draws exactly the block in the markup above, so
a consumer never writes `ds-hovercard-*` by hand (a sender card from parts is byte-identical to
the hand-written golden). The flag takes its glyph because the catalogue has no info glyph.

**Geometry.**

| Part | Value | Source |
| --- | --- | --- |
| Card | width 300 (Side kind 260), padding 12px 13px, ui 13, `--raise`, 1px `--line`, radius `--r-panel`, Pop shadow | `S:399-401`, `S:428` |
| Title `h5` | display 14 / 1.2, margin 0 | `S:405` |
| Sub | data 10.5 `--ink-faint`, margin-top 2 | `S:406` |
| Messages | margin-top 9, grid gap 7; row grid `22px 1fr` gap 8 align start; name 12; text ui 12 / 1.4 `--ink-soft`, 2 lines max | `S:407-412` |
| Person header | flex gap 10 centre; avatar 34 / 14 | `S:416-417` |
| Stats | margin-top 9, flex gap 14, ui 12 `--ink-soft`; value display 15 `--ink` block | `S:418-419` |
| Flag | margin-top 9, flex gap 7 start, padding 7px 9px, radius `--r-btn`, ui 12 / 1.4 `--ink`; icon 14, margin-top 1 | `S:420-422` |
| Flag danger | bg danger 12% over `--raise` (`--danger-wash`); icon `--danger` | `S:421-422` |
| Flag info | bg `--accent-soft`; icon `--accent` | `S:423-424` |
| Foot | margin-top 10, padding-top 8, top border 1px `--line-soft`, flex gap 6, data 10 `--ink-faint`; keys `margin-left:auto` | `S:413-415` |
| Actions | margin-top 10, flex gap 6 wrap, Mini buttons | `S:425` |
| Live dot | 6 x 6 round `--ok` | `S:561` |

**States.** Card (verbatim, `S:399-404`):

```css
.hc{ position:absolute; z-index:30; width:300px; background:var(--raise); color:var(--ink); border:1px solid var(--line);
  border-radius:var(--r-panel); box-shadow:0 18px 40px -16px rgba(0,0,0,.45); padding:12px 13px; font-size:13px;
  animation:hc-in var(--t-move) var(--e-spring); }
.hc.out{ animation:hc-out 120ms var(--e-out) forwards; }
@keyframes hc-in{ from{ opacity:0; transform:translateY(4px) scale(.98); } to{ opacity:1; transform:none; } }
@keyframes hc-out{ to{ opacity:0; transform:translateY(2px); } }
```

Content blocks (verbatim, `S:405-425`, `S:428`):

```css
.hc h5{ margin:0; font-family:var(--font-display); font-size:14px; line-height:1.2; }
.hc .sub{ font-family:var(--font-data); font-size:10.5px; color:var(--ink-faint); margin-top:2px; }
.hc .msgs{ margin-top:9px; display:grid; gap:7px; }
.hc .msg{ display:grid; grid-template-columns:22px 1fr; gap:8px; align-items:start; }
.hc .msg .av, .hc .person .av{ width:22px; height:22px; border-radius:999px; display:grid; place-items:center; background:var(--ink); color:var(--paper);
  font-family:var(--font-display); font-weight:800; font-size:10px; }
.hc .msg b{ font-size:12px; }
.hc .msg p{ font-size:12px; color:var(--ink-soft); line-height:1.4; display:-webkit-box; -webkit-line-clamp:2; -webkit-box-orient:vertical; overflow:hidden; }
.hc .foot{ margin-top:10px; padding-top:8px; border-top:1px solid var(--line-soft); display:flex; align-items:center; gap:6px;
  font-family:var(--font-data); font-size:10px; color:var(--ink-faint); }
.hc .foot .k{ margin-left:auto; }
.hc .person{ display:flex; gap:10px; align-items:center; }
.hc .person .av{ width:34px; height:34px; font-size:14px; }
.hc .stat{ margin-top:9px; display:flex; gap:14px; font-size:12px; color:var(--ink-soft); }
.hc .stat b{ display:block; font-family:var(--font-display); font-size:15px; color:var(--ink); }
.hc .flag{ margin-top:9px; display:flex; gap:7px; align-items:flex-start; padding:7px 9px; border-radius:var(--r-btn);
  background:color-mix(in oklab, var(--danger) 12%, var(--raise)); color:var(--ink); font-size:12px; line-height:1.4; }
.hc .flag .ic{ width:14px; height:14px; color:var(--danger); margin-top:1px; }
.hc .flag.info{ background:var(--accent-soft); }
.hc .flag.info .ic{ color:var(--accent); }
.hc .acts{ margin-top:10px; display:flex; gap:6px; flex-wrap:wrap; }
.hc.side-card{ width:260px; }
```

| State | Rule |
| --- | --- |
| idle -> pending | pointer rests on a target; open after 450 ms (HoverOpen), 0 ms when warm |
| entering | `hc-in --t-move --e-spring` |
| present | pointer may travel into the card; entering the card cancels the close timer |
| leaving | pointer out for 150 ms (HoverClose): `hc-out 120ms --e-out forwards`, unmounted at 130 ms; the hub turns warm for 400 ms (HoverWarm) |
| replaced | a new key while one is open replaces it at once, no exit (`S:1778`) |
| dismissed | a click in the list removes the card at once (`S:1809`) |
| suppressed | no card over a leaving row, while a peek is open, or while the palette is open (`S:1790`) |

**Motion.** `hc-in`, `hc-out`; `--t-move --e-spring`; 120 ms is `--t-hc-out`.

**Behaviour.** Innermost target wins (sender name inside a row beats the row, `S:1787-1788`).
Space with a thread card open (focus not in an input) closes the card and opens the peek
(`S:1804-1807`). Placement by kind (`S:1715-1726`): thread x = list column right + 10, y = row
top - 4; pin/today/account x = target right + 10, y = target top - 6; others x = target left,
y = target bottom + 6; clamp to [8, window - 8]; no flip. 06-INTERACTIONS "hover intent".

**Blitz notes.**
- `-webkit-line-clamp` (message text) -> `text::clip_chars` to two lines' worth in Rust.
- `color-mix` flag background -> `--danger-wash`.
- JS timers and `.out` removal -> `HoverHub` + `HoverIntent` state machine + `settle(HcOut)`.
- Position from `getBoundingClientRect` -> `onmounted` rects + `place()`.
- Plan: host synthesises a PointerMove before pointer down (cosmic-comp #2230) so intent arms.

### 23. Toast

**Purpose.** The undo toast: one dark pill at the bottom centre of the card saying what just
happened, with a pull tab. Pull the tab right past 46 px, or click it, to undo. Mail: every
operation (`S:1548-1553`). Shell: notifications (planned; the pull tab is mail's undo).

**Markup.** (from `S:861`, tab content `S:1563`)

```html
<div class="ds-toast" role="status" data-shown="shown|hidden">
  <span class="ds-toast-text">Archived</span>
  <span class="ds-toast-hint">pull →</span>
  <button type="button" class="ds-toast-tab" data-drag="idle|live" data-armed="armed|disarmed" style="transform:translateX(0px)">
    <svg class="ds-ic">…undo…</svg>Undo
  </button>
</div>
```

**Props.**

```rust
pub fn use_toasts() -> ToastHub     // push(text, undo: Option<UndoToken>); one visible at a time
// push_undoable(text, undo: UndoToken, on_undo: EventHandler<UndoToken>): the undo calls on_undo
// (settled, Gallery fixes B); last_undo() stays for a consumer that watches instead.
#[component] pub fn ToastHost() -> Element   // rendered by Ds; reads ToastHub
```

**Geometry.**

| Part | Value | Source |
| --- | --- | --- |
| Toast | absolute, left 50%, bottom 14, flex centre, gap 2, padding 5px 5px 5px 15px, radius 999, `--paper` on `--ink`, `--shadow-2`, ui 12.5 / 600, nowrap | `S:360-363` |
| Hidden offset | translateX(-50%) translateY(160%) (C 140%) | `S:360` |
| Hint | data 10, opacity .7 (C .72), letter-spacing .06em, padding `0 6px 0 10px` | `S:365` |
| Tab | flex gap 6, padding 5px 12px 5px 9px, radius 999, `--ink` on `--paper`, ui 12 / 700, cursor grab, icon 13 | `S:366-370` |
| Tab armed | `--accent-ink` on `--accent` | `S:369` |
| Drag range | dx clamped to [-6, 78]; armed when dx > 46 | `S:1566-1567` |

**States.** Verbatim, `S:360-370`:

```css
.toast{ position:absolute; left:50%; bottom:14px; z-index:8; transform:translateX(-50%) translateY(160%);
  display:flex; align-items:center; gap:2px; background:var(--ink); color:var(--paper); border-radius:999px;
  padding:5px 5px 5px 15px; box-shadow:var(--shadow-2); font-size:12.5px; font-weight:600; white-space:nowrap;
  transition:transform var(--t-big) var(--e-spring); }
.toast.show{ transform:translateX(-50%) translateY(0); }
.toast .hint{ font-family:var(--font-data); font-size:10px; opacity:.7; letter-spacing:.06em; padding:0 6px 0 10px; }
.toast .tab{ display:flex; align-items:center; gap:6px; cursor:grab; background:var(--paper); color:var(--ink); border:0;
  border-radius:999px; padding:5px 12px 5px 9px; font-size:12px; font-weight:700; touch-action:none;
  transition:transform var(--t-move) var(--e-spring), background-color var(--t-quick) var(--e-out); }
.toast .tab.armed{ background:var(--accent); color:var(--accent-ink); }
.toast .tab .ic{ width:13px; height:13px; }
```

C adds `.toast .tab:active{ cursor:grabbing; }` (`C:572`); adopt it.

| State | Rule |
| --- | --- |
| hidden | translateY(160%) below the card edge |
| entering (shown) | translateY(0) over `--t-big --e-spring` |
| tab hover | not specified |
| tab dragging | `data-drag=live`: `transition:none`, transform `translateX(dx)` written by the drag tracker |
| tab armed | `--accent` fill, `--accent-ink` text |
| tab released | transition restored; tab springs back to 0 over `--t-move --e-spring`; armed cleared |
| tab focus-visible | global ring |
| leaving | back to translateY(160%) after 5200 ms (ToastHold), or at once on undo |

**Motion.** Transform transitions only (`--t-big --e-spring` for the toast, `--t-move
--e-spring` for the tab, `--t-quick --e-out` for the tab fill).

**Behaviour.** Each push replaces the text, shows the toast and restarts the 5200 ms timer.
Release while armed undoes. A click with |dx| < 3 undoes (S; C requires dx == 0). Undo is
ignored if the snapshot belongs to another Space (`S:1555`). 06-INTERACTIONS "undo toast".

**Blitz notes.**
- `setPointerCapture` + inline style writes -> Rust DragTracker writes `--dx`/`transform` and
  `data-drag`; `transition:none` becomes the `[data-drag=live]` rule.
- `touch-action:none`: keep (harmless if ignored).
- Timers -> Rust (`ToastHold`).
- Position absolute in the card -> OverlayHost, bottom-centre of the content region.

### 24. Scrim, Sheet, Peek

**Purpose.** Modal layering. Scrim dims and catches the click that closes. Peek shows a thread's
reader in a centred panel over the card (shift-click a row, or Space on a thread card). Sheet is
the plan's general modal panel for settings and dialogs, derived from Peek (A0: "centred peek over
scrim (sheet)"). Shell: dialogs, settings sheets.

**Markup.** (from `S:1576-1582`)

```html
<button type="button" class="ds-scrim" aria-label="Close peek"></button>
<div class="ds-peek" data-mode="center|full" role="dialog" aria-label="Re: UIDL stability">
  <div class="ds-peek-tools">
    <button type="button" class="ds-icon-button" data-variant="tool" aria-label="Close peek">…x…</button>
  </div>
  …reader…
</div>
<div class="ds-sheet" role="dialog" aria-label="…">…</div>   <!-- derived -->
```

**Props.**

```rust
#[component] pub fn Scrim(label: String, onclose: EventHandler<()>) -> Element
#[component] pub fn Peek(mode: PeekMode /* Center | Full */, label: String, onclose: EventHandler<()>, children: Element) -> Element
#[component] pub fn Sheet(label: String, onclose: EventHandler<()>, children: Element) -> Element   // derived
```

**Geometry.**

| Part | Value | Source |
| --- | --- | --- |
| Scrim | absolute inset 0, z 10, no border, `--scrim` (rgba(0,0,0,.22)), cursor pointer | `S:221` |
| Scrim (C) | `--ink` at opacity .16 | `C:1054` |
| Peek Center | absolute `inset:36px 12% 36px 12%` (C `34px 10%`), z 11, `--surface-2`, radius `--r-panel`, `0 24px 50px -18px rgba(0,0,0,.55)`, overflow hidden, flex column | `S:223-224` |
| Peek Full (C) | inset 0, z 11 | `C:1049-1050` |
| Sheet | derived: = Peek Center geometry; width and height not specified (O-14) | |

**States.** Verbatim, `S:221-227`:

```css
.scrim{ position:absolute; inset:0; z-index:10; border:0; background:var(--scrim); cursor:pointer;
  animation:fade var(--t-move) var(--e-out); }
.peek{ position:absolute; inset:36px 12% 36px 12%; z-index:11; background:var(--surface-2); border-radius:var(--r-panel);
  box-shadow:0 24px 50px -18px rgba(0,0,0,.55); overflow:hidden; display:flex; flex-direction:column;
  animation:peek-in var(--t-big) var(--e-spring); }
@keyframes peek-in{ 0%{ opacity:0; transform:scale(.95) translateY(12px); } 100%{ opacity:1; transform:none; } }
@keyframes fade{ from{ opacity:0; } to{ opacity:1; } }
```

C Full (verbatim, `C:1049-1050`): `.shell[data-peek="full"] .reader{ position:absolute; inset:0; z-index:11; animation:peek-in var(--t-move) var(--e-out); }`

| State | Scrim | Peek |
| --- | --- | --- |
| entering | `fade --t-move --e-out` | Center: `peek-in --t-big --e-spring`; Full: `peek-in --t-move --e-out` |
| hover | not specified (cursor pointer) | n/a |
| focus-visible | global ring (it is a button) | n/a |
| leaving | removed at once (`S:1583`) | removed at once |

**Motion.** `fade`, `peek-in`. C's `peek-in` differs (scale .97 translateY 10px, `C:1052`); S wins.

**Behaviour.** Close: the tool button, the scrim, or Esc (Esc closes the palette first if open,
`S:1670`). Opening a peek closes any existing one. Hover cards are suppressed while a peek is open.
Focus: not specified (O-15: move focus into the peek and restore it on close).

**Blitz notes.** Through OverlayHost, sized to the card's content rect (the scrim covers the card,
not the window, in S). Percent insets work in Blitz; no change.

### 25. CommandPalette

**Purpose.** "The command menu is the same menu, just bigger and centred" (`S:792`). Search and
commands over a scrim: a SearchField on top and a Rich Menu below, grouped (Actions, Recent / Top
hit, Mail, People, Actions). Mail: Ctrl T / Ctrl K. Shell: the launcher.

**Markup.** (from `S:1635-1636`, items `S:1644-1646`)

```html
<div class="ds-palette-wrap">                               <!-- scrim + centring -->
  <div class="ds-palette" role="dialog" aria-label="Search and commands" data-presence="entering">
    <div class="ds-search">…§7…</div>
    <div class="ds-search-tokens">…</div>
    <div class="ds-menu" data-kind="rich" data-embed="palette" role="listbox">
      <div class="ds-section-header" data-kind="menu">Top hit</div>
      <div class="ds-menu-item" role="option" aria-selected="true">
        <span class="ds-menu-tile"><span class="ds-avatar" data-size="34">D</span></span>
        <span><b class="ds-menu-title">Re: <mark>UIDL</mark> stability</b>
              <small class="ds-menu-detail ds-truncate">Treat UIDL as…</small></span>
        <span class="ds-menu-trail">⌃1</span>
      </div>
    </div>
  </div>
</div>
```

**Props.**

```rust
#[component] pub fn CommandPalette<T: Clone + PartialEq + 'static>(
    label: String, placeholder: String, query: String, tokens: Vec<String>,
    groups: Vec<(String, Vec<MenuEntry<T>>)>, empty: String,
    oninput: EventHandler<String>, onpick: EventHandler<T>, onclose: EventHandler<()>,
    #[props(default)] host: CommandPaletteHost,          // Overlay | Surface
    #[props(default)] entrance: PaletteEntrance,         // PeekIn | CmdkIn
    #[props(default)] id: Option<String>,                // on the card
    #[props(default)] focus: Option<FocusRequest>,
    #[props(default)] selected: Option<usize>,           // controlled selection
    #[props(default)] on_select: Option<EventHandler<usize>>,
    #[props(default)] on_select_rect: Option<EventHandler<Rect>>,
    #[props(default)] onkey: Option<EventHandler<KeyboardData>>) -> Element
```

Hosts (settled 2026-09-24, FINDINGS "Launcher gaps", sill Q40-Q41): `Overlay` is the scrim and
the card 11 % down, as below. `Surface` is the shell launcher's: no wrap and no scrim, the card
fills its container (`width:100%; height:100%`, the list taking the height under the field) and
paints the enclosing material's tint and edge (`--m-tint`/`--m-tint-solid`, `--m-box`) at radius
`--r-panel`, carrying `id` for the blur region. `entrance` picks `peek-in` (S) or `cmdk-in` (C)
for either host. The selection is exposed: `selected` makes it the caller's (Up, Down and the
pointer then ask through `on_select`), uncontrolled `on_select` hears every change, and
`on_select_rect` reports the selected row's rect after layout, for an actions menu anchored to
it. `onkey` hears every key the field gets after the palette. A row's tile takes
`Tile::Source(IconSource)`: an app's icon image fills the tile with no plate (Q42).

Ranking and grouping are pure Rust (06-INTERACTIONS "command menu search").

**Geometry.**

| Part | Value | Source |
| --- | --- | --- |
| Wrap | absolute inset 0, z 20, grid `place-items:start center`, padding-top 11% (C 14%), `--scrim` bg | `S:230-231` |
| Panel | width `min(540px, 88%)` (C `min(420px,88%)`), `--raise`, `--ink`, 1px `--line`, radius 14, `0 30px 60px -20px rgba(0,0,0,.5)`, overflow hidden | `S:232-233`, `S:793` |
| Input row | SearchField (§7): padding 12px 14px, input 16 | `S:794-795` |
| List | embedded Rich Menu: static, auto width, max-height 360, no border, no shadow, no radius, no animation | `S:796` |
| Snippet | ui 11.5 `--ink-faint`, max width 380, truncated | `S:799` |
| Empty | Menu empty style; text "Nothing in this Space matches. Search checks subjects, names, addresses and the text of every message." | `S:1647` |

**States.** Verbatim, `S:230-234` then the overrides `S:793-799`:

```css
.cmdk-wrap{ position:absolute; inset:0; z-index:20; display:grid; place-items:start center; padding-top:11%;
  background:var(--scrim); animation:fade var(--t-quick) var(--e-out); }
.cmdk{ width:min(520px,86%); background:var(--raise); color:var(--ink); border:1px solid var(--line);
  border-radius:var(--r-panel); box-shadow:0 30px 60px -20px rgba(0,0,0,.5); overflow:hidden;
  animation:peek-in var(--t-big) var(--e-spring); }
.cmdk{ width:min(540px, 88%); border-radius:14px; }
.cmdk .fmenu{ position:static; width:auto; max-height:360px; border:0; box-shadow:none; border-radius:0; animation:none; }
.cmdk .snip{ display:block; font-size:11.5px; color:var(--ink-faint); white-space:nowrap; overflow:hidden; text-overflow:ellipsis; max-width:380px; }
```

`.cmdk-list` and `.cmdk-g` rules (`S:237-241`) belong to the pre-menu version and are superseded
by the embedded `.fmenu`; do not port them.

| State | Rule |
| --- | --- |
| entering | wrap `fade --t-quick --e-out`; panel `peek-in --t-big --e-spring` (C: `cmdk-in`, with overshoot) |
| item selected | Menu selected (`--accent-soft`) |
| match | `mark` accent 800 |
| leaving | removed at once (`S:1663`) |

**Motion.** `fade`, `peek-in` (S) / `cmdk-in` (C). S wins.

**Behaviour.** Open: closes floating menus, focuses the input (`S:1633`, `S:1661`). Keys: Up/Down
move the selection CLAMPED (not wrapping, unlike Menu); Enter runs; Esc closes (`S:1653-1656`).
Pointer down on the wrap outside the panel closes (`S:1660`). mousedown in the list is prevented
(`S:1658`). Running an item closes first, then runs (`S:1650`). C returns focus to the shell on
close (`C:2257`).

**Blitz notes.** Through OverlayHost. `scrollIntoView` -> Rust scroll. Snippet truncation ->
`.ds-truncate` with max width 380. On the shell the launcher is its own layer surface (plan), and
the wrap's scrim is a transparent overlay catcher surface (plan: "two surfaces kept warm").

**In a surface (the macOS polish pass, settled 2026-09-24).** With `host: Surface` the card is as
tall as its content up to its container (no empty box under the last row); the query is
`--fs-shell-field` 22 at `--fw-shell-field` 500 beside a `--shell-field-glyph` 20 px search glyph,
row titles `--fs-shell-row` 14 and details `--fs-shell-detail` 12; its shadow is the material
stack's contact and ambient pair (`--m-box`). `corner: Option<Corner>` gives the card another
radius or a `Corner::Squircle` (the launcher's: `Squircle(14)`).

### 26. AppearancePicker

**Derived.** The plan: "THE one picker for mailo, control center, settings". Neither prototype
has an appearance popover (A4: "NO appearance popover exists"; A8 #9). The pieces that exist:
S's Space editor rows (field label + SegmentedControl: Appearance System/Light/Dark, Accent "A
hint of the Space"/"Postmark", `S:877-888`) and C's controls row (label + segmented control
for Look, Warmth, Motion, Theme, `C:1131-1157`).

**Purpose.** Pick Theme, Accent and Motion for an app or the whole shell (`Appearance` in
`resolve(app, look_theme, system)`).

**Markup.**

```html
<div class="ds-appearance" role="group" aria-label="Appearance">
  <div class="ds-appearance-row">
    <div class="ds-section-header" data-kind="field">Theme</div>
    <div class="ds-segmented" role="group" aria-label="Theme">…System | Light | Dark…</div>
  </div>
  <div class="ds-appearance-row">
    <div class="ds-section-header" data-kind="field">Accent</div>
    <div class="ds-appearance-swatches" role="group" aria-label="Accent">
      <button type="button" class="ds-space-dot" aria-pressed="true" aria-label="Postmark" style="background:…"></button>
      …one per accent…
    </div>
  </div>
  <div class="ds-appearance-row">
    <div class="ds-section-header" data-kind="field">Motion</div>
    <div class="ds-segmented" role="group" aria-label="Motion">…Calm | Standard | Extra | Reduced…</div>
  </div>
</div>
```

**Props.**

```rust
#[component] pub fn AppearancePicker(value: Appearance, system: SystemPrefs,
    onchange: EventHandler<Appearance>) -> Element
```

**Geometry.** Rows stacked; the Space editor's stack gap is 14 (`S:247`); a row is a Field
SectionHeader (margin-bottom 6) over its control. Swatches reuse the Space dot (§32: 22 px
circle, 2px border, gap 5 as in the sidebar foot `S:845`). Overall width not specified (O-16).
Labels: Theme uses S's words (System, Light, Dark; C says Auto); Motion uses C's (Calm,
Standard, Extra) plus the plan's Reduced.

**States.** Inherited from SegmentedControl (§3) and Space dot (§32). No states of its own.

**Motion.** None of its own. Changing Theme cross-fades the frame (layer opacity 380 ms,
`S:86`; 05-MOTION).

**Behaviour.** Changes apply at once and persist through ds-settings (atomic write). "System"
follows the portal. The number of accents is the accent table's (03-COLOR); the plan says 6 in
one place and has a test named "exactly-four-accent" in another (O-17).

**Blitz notes.** None beyond its parts.

### 27. AccountTile

**Purpose.** The accounts in this Space, as a row of square tiles at the top of the sidebar; one
extra "All" tile when there is more than one account. Pressing a tile filters the list to that
account. Each tile shows the account's letter, its provider mark, and its unread count. Mail:
sidebar. Shell: user switcher (planned).

**Markup.** (from `S:1233-1240`)

```html
<div class="ds-account-tiles" role="group" aria-label="Accounts in this Space">
  <button type="button" class="ds-icon-button ds-account-tile" data-variant="pin"
          aria-pressed="true" aria-label="All accounts">
    <span class="ds-avatar" data-size="28" data-tone="all"><svg class="ds-ic">…inbox…</svg></span>
    <span class="ds-count" data-place="tile">4</span>
  </button>
  <button type="button" class="ds-icon-button ds-account-tile" data-variant="pin"
          aria-pressed="false" aria-label="poh@acme.example">
    <span class="ds-avatar" data-size="28" style="--av-bg:#5B4FC4">P</span>
    <span class="ds-provider" data-size="tile" data-kind="letter|image">G</span>
    <span class="ds-count" data-place="tile">2</span>
  </button>
</div>
```

The tile is also a HoverTarget (account card, §22).

**Props.**

```rust
#[component] pub fn AccountTile(account: AccountFace /* All | One{initial, colour, provider, address: Option<String>} */,
    pressed: Switch, unread: u32, onclick: EventHandler<()>) -> Element
```

**Geometry.** Grid of 4 columns, gap 6, margin-top 12 (`S:107`). Tile: Pin IconButton (§2),
aspect 1, radius 12. Avatar 28, display 800 12.5 white on the account colour; All: transparent
avatar, `--f-ink`, inbox icon 18. Provider mark bottom-right (§28). Count top 4 right 5 data 9
(§14).

**States.** Tile base, hover, active, pressed: Pin IconButton (§2). Avatar (verbatim, `S:356-357`,
`S:548-551`):

```css
.pin:hover .av{ transform:rotate(-6deg) scale(1.08); }
.pin .av{ transition:transform var(--t-quick) var(--e-spring); }
.pin.acct .av{ width:28px; height:28px; font-size:12.5px; }
.pin.acct .av.all{ background:transparent; color:var(--f-ink); }
.pin.acct .av.all .ic{ width:18px; height:18px; }
.pin.acct[aria-pressed="false"] .av:not(.all){ filter:saturate(.55); opacity:.85; }
```

| State | Rule |
| --- | --- |
| default (pressed) | `--f-pill` + current shadow; avatar full colour |
| unpressed | `--f-pill-hover`; avatar desaturated to 55% and opacity .85 |
| hover | tile `--f-pill`; avatar rotate(-6deg) scale(1.08) on `--t-quick --e-spring` |
| active | tile scale(`--squish`) |
| focus-visible | global ring |
| count changed | Count bump (§14) |

**Behaviour.** A tile is pressed when it is the selected filter, or when the Space has one
account (`S:1234`). Pressing re-renders the list with the entering stagger (`S:1494`). The list
title gains the account address in data 11 `--ink-faint` (`S:1275`).

**Blitz notes.** `filter:saturate(.55)` is banned. Replacement: Rust computes the desaturated
account colour (OKLCH chroma x .55) and passes it as `--av-bg` when unpressed; opacity .85 stays
(O-26).

### 28. ProviderMark

**Purpose.** "A letter in the provider's colour on a white chip, never the provider's logo"
(`S:1128-1129`), or, when the user picks "Their icons", the provider's own favicon cached at
account setup (`S:1139-1141`). Three sizes. Mail: account tiles, row `via`, From picker.

**Markup.** (from `S:1143-1146`)

```html
<span class="ds-provider" data-size="tile|row|inline" data-kind="letter" style="--pc:#1A73E8" title="Google">G</span>
<span class="ds-provider" data-size="tile|row|inline" data-kind="image" title="Google"><img alt="" src="data:…"></span>
```

**Props.**

```rust
#[component] pub fn ProviderMark(provider: Provider, size: MarkSize /* Tile | Row | Inline */,
    style: MarkStyle /* Letter | Image(ImageSource) */) -> Element
pub enum Provider { Google, Microsoft, Fastmail, ICloud, Yahoo, Imap, Local }
```

Letter and colour per provider (`S:1130-1136`): Google `G` #1A73E8; Microsoft 365 `M` #0F6CBD;
Fastmail `F` #2A5DB0; iCloud `i` #3A82F7; Yahoo `Y` #6001D2; IMAP `@` #5D6660. Row `via` text:
gmail, m365, fastmail, icloud, yahoo, imap.

`Local` (mailo gaps 4, settled 2026-09-25): a local-folders account has no provider, so its mark
is not a letter but the `folder` glyph (10 on a tile, 8 in a row, 9 inline) stroked in IMAP's
neutral #5D6660 on the same chip, `data-kind="local"`, titled "Local folders". It has no favicon:
`MarkStyle::Image` draws the same glyph.

**Geometry.**

| Size | Box | Radius | Font | Ring | Placement |
| --- | --- | --- | --- | --- | --- |
| Inline (base) | 13 x 13 | 4 | display 800 9, line-height 1 | `0 0 0 1px rgba(0,0,0,.08)` | inline-grid, flex none |
| Tile | 14 x 14 | 5 | 9.5 | `0 0 0 1.5px var(--f-pill), 0 1px 2px rgba(0,0,0,.2)` | absolute right 7 bottom 7 |
| Row | 11 x 11 | 3 | 7.5 | `0 0 0 1px var(--line)` | margin-right 4, vertical-align -1px |
| Image kind | as size, padding 1.5, white bg; image fills, object-fit contain | | | | |

**States.** Verbatim, `S:552-560`:

```css
.prov{ display:inline-grid; place-items:center; width:13px; height:13px; border-radius:4px; background:#FFFFFF; color:var(--pc);
  font-family:var(--font-display); font-weight:800; font-size:9px; line-height:1; box-shadow:0 0 0 1px rgba(0,0,0,.08); flex:none; }
.prov.on-tile{ position:absolute; right:7px; bottom:7px; width:14px; height:14px; font-size:9.5px; border-radius:5px;
  box-shadow:0 0 0 1.5px var(--f-pill), 0 1px 2px rgba(0,0,0,.2); }
.prov.in-row{ width:11px; height:11px; font-size:7.5px; border-radius:3px; margin-right:4px; vertical-align:-1px; box-shadow:0 0 0 1px var(--line); }
.via{ display:inline-flex; align-items:center; }
.prov.img{ background:#FFFFFF; padding:1.5px; }
.prov.img img{ width:100%; height:100%; display:block; object-fit:contain; }
```

Static. No hover, focus or motion.

**Blitz notes.** `#FFFFFF` and the provider colours are identity data, not theme colours; they
enter through `--pc` and a `--mark-ground` token (O-3). `object-fit:contain`: verify (spike);
fallback is pre-sizing the image in Rust. The favicon bytes are supplied by the app; ds never
fetches.

### 29. LinkPill

**Purpose.** "The real destination of a link, like a browser's status bar … instant, and loud
when the text lies" (`S:430`). Mail: reader and peek. Shell: none.

**Markup.** (from `S:1820-1823`)

```html
<!-- honest -->
<div class="ds-link-pill" data-truth="honest" role="status">
  <span class="ds-link-pill-dim">https://www.</span><b>rfc-editor.org</b><span class="ds-link-pill-dim">/rfc/rfc1939</span>
</div>
<!-- lying: the visible text names another registered domain -->
<div class="ds-link-pill" data-truth="lying" role="status">
  <svg class="ds-ic">…x…</svg>Goes to <b>g00gle-security.xyz</b>, not google.com
</div>
```

**Props.**

```rust
#[component] pub fn LinkPill(target: LinkTarget /* Honest{scheme_sub, registered, path} | Lying{registered, shown} */) -> Element
```

The honest/lying decision (registered domain = last two labels; compare the domain in the link
text with the href's) is a pure function in mail, not in ds (`S:1812-1819`).

**Geometry.** Absolute left 12 bottom 12 of the reader, z 7, max width `calc(100% - 24px)`,
flex gap 7, padding 5px 11px 5px 9px, radius 999, `--paper` on `--ink`, data 11, nowrap,
truncated, `--shadow-2`, no pointer events. Registered domain bold in `--paper`; scheme,
subdomain and path at opacity .6. Lying: `--danger` bg, white text, x icon 13.

**States.** Verbatim, `S:431-437`:

```css
.linkpill{ position:absolute; left:12px; bottom:12px; z-index:7; max-width:calc(100% - 24px); display:flex; align-items:center; gap:7px;
  background:var(--ink); color:var(--paper); border-radius:999px; padding:5px 11px 5px 9px; font-family:var(--font-data); font-size:11px;
  white-space:nowrap; overflow:hidden; text-overflow:ellipsis; box-shadow:var(--shadow-2); animation:hc-in var(--t-quick) var(--e-out); pointer-events:none; }
.linkpill b{ color:var(--paper); }
.linkpill .dim{ opacity:.6; }
.linkpill.warn{ background:var(--danger); color:#fff; }
.linkpill .ic{ width:13px; height:13px; }
```

| State | Rule |
| --- | --- |
| entering | `hc-in --t-quick --e-out`, instant on pointer over (no intent delay) |
| lying | danger fill, white text |
| leaving | removed on pointer leave, no animation |

**Behaviour.** One pill at a time; appears on pointer over a link in the reader or peek; clicks
on those links are prevented in the prototype (`S:1829`). 06-INTERACTIONS "link pill".

**Blitz notes.** Ellipsis -> `.ds-truncate` (fade on the right; the registered domain must stay
visible, so truncate the path first: O-30). `#fff` -> `--danger-ink` token.

### 30. SelectionBubble

**Purpose.** "Notion's inline toolbar, and the only one there is" (`S:682`). Appears over a
non-empty text selection in the composer. Mail: composer title and body. Shell: text fields
(planned).

**Markup.** (from `S:2136-2142`)

```html
<div class="ds-popover ds-bubble" role="toolbar" aria-label="Format">
  <button type="button" class="ds-bubble-button" data-kind="turn">Text ▾</button>   <!-- body only -->
  <span class="ds-bubble-sep"></span>
  <button type="button" class="ds-bubble-button" aria-pressed="true" title="Bold (Ctrl B)"><b>B</b></button>
  <button type="button" class="ds-bubble-button" aria-pressed="false" title="Italic (Ctrl I)"><i>i</i></button>
  <button type="button" class="ds-bubble-button" aria-pressed="false" title="Underline (Ctrl U)"><u>U</u></button>
  <button type="button" class="ds-bubble-button" aria-pressed="false" title="Strikethrough (Ctrl Shift S)"><s>S</s></button>
  <button type="button" class="ds-bubble-button" title="Inline code (Ctrl E)">&lt;/&gt;</button>
  <span class="ds-bubble-sep"></span>
  <button type="button" class="ds-bubble-button" title="Link (Ctrl K)"><svg class="ds-ic">…corner…</svg>Link</button>
</div>
<!-- link mode replaces the contents -->
<div class="ds-popover ds-bubble"><input class="ds-input" data-variant="inline" type="url" placeholder="Paste a link, then Enter" aria-label="Link"></div>
```

**Props.**

```rust
#[component] pub fn SelectionBubble(anchor: Rect /* selection rect */, mode: BubbleMode /* Actions(Vec<BubbleAction>) | Link */,
    onlink: EventHandler<String>, onclose: EventHandler<()>) -> Element
pub enum BubbleAction { Button(BubbleButton), Separator /* ds-bubble-sep */ }
pub struct BubbleButton { label: Element, title: String, pressed: Option<Switch>, onclick: EventHandler<()> }
```

**Geometry.** Flex, gap 1, padding 3, radius 10, `--raise`, 1px `--line`, Bubble shadow
`0 12px 28px -12px rgba(0,0,0,.45)`, z 41. Buttons: height 28, min width 28, padding `0 7px`,
radius 7, ui 13 `--ink-soft`, inline-flex gap 4. Separator 1 x 18 `--line-soft`, margin `0 3px`.
Turn button weight 600. Italic uses Georgia (`S:2138`), code uses data 11.5. Link input: ui 12.5,
width 220, padding `0 8px`.

**States.** Verbatim, `S:683-691`:

```css
.bubble{ position:absolute; z-index:41; display:flex; align-items:center; gap:1px; padding:3px; background:var(--raise); border:1px solid var(--line);
  border-radius:10px; box-shadow:0 12px 28px -12px rgba(0,0,0,.45); animation:menu-pop var(--t-quick) var(--e-spring); }
.bubble button{ border:0; background:transparent; cursor:pointer; height:28px; min-width:28px; padding:0 7px; border-radius:7px; color:var(--ink-soft); font-size:13px;
  display:inline-flex; align-items:center; gap:4px; }
.bubble button:hover{ background:var(--surface-2); color:var(--ink); }
.bubble button[aria-pressed="true"]{ color:var(--accent); }
.bubble .sep{ width:1px; height:18px; background:var(--line-soft); margin:0 3px; }
.bubble .turn{ font-weight:600; }
.bubble input{ border:0; outline:none; background:transparent; font:inherit; font-size:12.5px; width:220px; padding:0 8px; color:var(--ink); }
```

| State | Rule |
| --- | --- |
| entering | `menu-pop --t-quick --e-spring` |
| button hover | `--surface-2`, `--ink` |
| button pressed (mark active) | `--accent` text |
| button focus-visible | global ring |
| leaving | removed at once when the selection collapses (unless focus is inside the bubble) or leaves the composer |

**Behaviour.** Placement: centred over the selection rect, 8 px above it; x clamped 8 px inside
the window; top never above 8 (`S:2144-2145`); no flip below. mousedown on the bubble is
prevented except on its input, so the selection survives (`S:2135`). "Turn ▾" opens the
turn-into Slim Menu at its rect. Link mode: Enter applies, Esc hides. 06-INTERACTIONS
"selection bubble".

**Blitz notes.** S listens to `selectionchange` and `document.execCommand` / `queryCommandState`.
None exist in Blitz. ds provides only the surface, placement and actions; the composer's Rust
document model supplies the selection rect and the pressed marks (O-21). Georgia italic is a
font outside the three families (02-TYPE decides).

### 31. SendPill

**Purpose.** Undo send. After Send, a dark pill at the bottom centre counts down 5 seconds with
a ring, offers Undo, then says "Sent" and leaves. Mail: composer send. Shell: generic progress
pill (planned).

**Markup.** (from `S:2337-2338`)

```html
<div class="ds-send-pill" data-shown="shown|hidden" data-phase="counting|done" role="status" style="--f:.4">
  <svg class="ds-send-ring" viewBox="0 0 24 24" aria-hidden="true">
    <circle class="ds-send-ring-track" cx="12" cy="12" r="9"></circle>
    <circle class="ds-send-ring-run" cx="12" cy="12" r="9"></circle>
  </svg>
  <span>Sending in 3 s</span>
  <button type="button" class="ds-send-pill-undo">Undo</button>
</div>
```

**Props.**

```rust
#[component] pub fn SendPill(text: String, progress: Fraction, phase: SendPhase /* Counting | Done */,
    onundo: EventHandler<()>) -> Element
```

**Geometry.** Absolute left 50% bottom 14, z 9, flex centre gap 10, padding 6px 6px 6px 12px,
radius 999, `--paper` on `--ink`, ui 12.5 / 600, nowrap, `--shadow-2`. Ring 20 x 20 (C 22),
circles r 9, stroke 3, no fill; track `currentColor` at .25; run `currentColor`, round cap,
rotated -90°, dasharray 57. Undo: `--ink` on `--paper`, padding 4px 12px, radius 999, ui 12 / 700.

**States.** Verbatim, `S:706-715`:

```css
.sendpill{ position:absolute; left:50%; bottom:14px; z-index:9; transform:translateX(-50%) translateY(160%); display:flex; align-items:center; gap:10px;
  background:var(--ink); color:var(--paper); border-radius:999px; padding:6px 6px 6px 12px; font-size:12.5px; font-weight:600; box-shadow:var(--shadow-2);
  transition:transform var(--t-big) var(--e-spring); white-space:nowrap; }
.sendpill.show{ transform:translateX(-50%) translateY(0); }
.sendpill svg{ width:20px; height:20px; }
.sendpill circle{ fill:none; stroke-width:3; }
.sendpill .track{ stroke:currentColor; opacity:.25; }
.sendpill .run{ stroke:currentColor; stroke-linecap:round; transform:rotate(-90deg); transform-origin:50% 50%; stroke-dasharray:57; }
.sendpill button{ border:0; cursor:pointer; background:var(--paper); color:var(--ink); border-radius:999px; padding:4px 12px; font-size:12px; font-weight:700; }
.sendpill.done button{ display:none; }
```

The ring drains by transitioning `stroke-dashoffset` 0 -> 57 over 5 s linear (inline, `S:2339`).

C's outbox (the older, richer version; verbatim, `C:545-546`, `C:552-554`) adds the retry states
S does not have:

```css
.outbox.nudge{ animation:nudge 520ms var(--e-out); }
.outbox.shake{ animation:shake 560ms cubic-bezier(.36,.07,.19,.97); }
.outbox .state{ font-family:var(--font-data); font-size:10px; opacity:.7; letter-spacing:.04em; }
.outbox .fix{ border:0; cursor:pointer; background:var(--paper); color:var(--ink);
  border-radius:999px; padding:4px 11px; font-size:11.5px; font-weight:700; }
```

| State | Rule |
| --- | --- |
| hidden | translateY(160%) |
| entering | translateY(0), `--t-big --e-spring` |
| counting | ring drains linearly over 5 s; text counts down each second ("Sending in 5 s" … ) or "Scheduled · <when>" |
| undo hover / focus | not specified / global ring |
| done | Undo hidden; text "Sent" (or "Scheduled · waits in the outbox"); hides after 1600 ms (SentHold) |
| retrying (C) | `nudge 520ms --e-out`; auth failure `shake 560ms` + a Fix button ("Sign in") |
| leaving | back to translateY(160%) |

**Behaviour.** Undo reopens the composer with the identical document (`S:2344`). 06-INTERACTIONS
"undo send".

**Blitz notes.** A transition on an SVG attribute (`stroke-dashoffset`) is unlikely to animate in
Blitz (spike S6 covers inline SVG only). Replacement: `MotionTimer` drives `progress: Fraction`
and Rust re-renders the ring with `stroke-dashoffset = 57 * f` at frame rate, or a custom Widget
(O-20). `setInterval` countdown -> Rust timer.

### 32. SpaceEditor

**Purpose.** The pieces of the Space editor: a panel, a hue x chroma field with up to three
draggable dots, stop chips, a grain slider, segmented settings, eight presets, live contrast
checks, and the Space dots that switch Spaces in the sidebar foot. Mail: the editor beside the
window (`S:866-904`). Shell: Settings > Spaces and the bar's workspace menu (plan 21-SPACES).

**Markup.**

```html
<aside class="ds-space-editor" aria-label="Space editor">
  <h3 class="ds-space-editor-title"><span class="ds-space-swatch" style="background:linear-gradient(…)"></span>Work Space</h3>

  <div class="ds-section-header" data-kind="field">Colour <span class="ds-section-header-value">drag a dot</span></div>
  <div class="ds-field">                                   <!-- field image + handles -->
    <div class="ds-field-plane"></div>
    <div class="ds-handle" role="slider" tabindex="0" aria-label="Colour 1" aria-pressed="true"
         style="left:74.4%; top:28%; background:#…"></div>
  </div>
  <div class="ds-stops">
    <span class="ds-stop" aria-pressed="true"><i style="background:#…"></i>268°
      <button type="button" class="ds-stop-remove" aria-label="Remove colour">…x…</button></span>
    <button type="button" class="ds-button" data-variant="mini">…plus… Colour</button>   <!-- fewer than 3 dots -->
  </div>

  <!-- Grain: Slider §5; Appearance / Accent / Provider marks: SegmentedControl §3 -->

  <div class="ds-presets"><button type="button" aria-label="Preset 1" style="background:linear-gradient(…)"></button>…8…</div>

  <div class="ds-checks">
    <div class="ds-check"><span>Sidebar text on the colour</span><span class="ds-check-value">7.12</span>
      <span class="ds-chip" data-variant="status" data-status="ok">≥ 4.5</span></div>
  </div>
  <p class="ds-capnote">No capping needed: every stop passes at the chroma you chose.</p>
</aside>

<!-- Space dots, sidebar foot (S:1265) -->
<button type="button" class="ds-space-dot" aria-pressed="true" aria-label="Work Space" title="Work (Ctrl 1)" style="background:linear-gradient(…)"></button>
```

**Props.**

```rust
#[component] pub fn SpaceEditor(look: SpaceLook, scheme: Scheme, active_dot: DotIndex,
    onchange: EventHandler<SpaceLook>, #[props(default)] name: Option<String> /* "{name} Space" */,
    #[props(default)] on_active_dot: Option<EventHandler<ActiveDot /* = DotIndex */>>) -> Element
#[component] pub fn SpaceDot(name: String, frame: FrameVars, here: Here, shortcut: Shortcut, onclick: EventHandler<()>) -> Element
```

Derivation of every colour shown (stops, picked colours, checks) is `space/palette.rs`
(03-COLOR); the editor only renders.

**Geometry.**

| Piece | Value | Source |
| --- | --- | --- |
| Panel | padding 14, grid gap 14, `--surface`, 1px `--line`, radius `--r-panel`, `--shadow-1` | `S:246-247` |
| Title | display 15, flex gap 8; swatch 14 circle, 135° gradient of the stops | `S:248-249` |
| Field | height 176, radius 12, 1px `--line`, overflow hidden, cursor crosshair | `S:252-253` |
| Field plane | 540 x 352 drawing scaled to fit; ground #f3f4f1 (dark #1b1d1a); dots every 18 px, radius 5.2, colour `oklch(L, (1 - y/H) * .15, x/W * 360)` with L .74 (dark .66) | `S:1396-1405` |
| Handle | 22 circle, margin -11, 3px white border, `0 0 0 1px rgba(0,0,0,.25), 0 3px 8px rgba(0,0,0,.35)`; active handle scale 1.15; left = hue/360, top = 1 - chroma | `S:255-257`, `S:1416-1417` |
| Stops row | flex wrap, gap 6, margin-top 8 | `S:258` |
| Stop | inline-flex gap 5, 1px `--line`, `--surface-2`, radius 999, padding 3px 5px 3px 4px, data 10 `--ink-soft`; disc 14; remove icon 11 `--ink-faint` | `S:259-264` |
| Presets | grid 8 columns, gap 6; each aspect 1, round, 1px `--line` | `S:270-271` |
| Checks | grid gap 5, ui 12; row flex gap 8 `--ink-soft`; value data 10.5 tabular, `margin-left:auto` | `S:273-275` |
| Cap note | ui 11.5 / 1.45 `--ink-faint` | `S:279` |
| Space dot | 22 circle, 2px transparent border | `S:147` |

**States.** Verbatim, `S:246-264`, `S:270-279`, `S:147-150`:

```css
.editor{ border:1px solid var(--line); border-radius:var(--r-panel); background:var(--surface); box-shadow:var(--shadow-1);
  padding:14px; display:grid; gap:14px; position:sticky; top:14px; }
.editor h3{ font-size:15px; display:flex; align-items:center; gap:8px; }
.editor h3 .sw{ width:14px; height:14px; border-radius:999px; }
.field{ position:relative; height:176px; border-radius:12px; overflow:hidden; cursor:crosshair; touch-action:none;
  border:1px solid var(--line); }
.field canvas{ width:100%; height:100%; display:block; }
.handle{ position:absolute; width:22px; height:22px; margin:-11px 0 0 -11px; border-radius:999px; border:3px solid #fff;
  box-shadow:0 0 0 1px rgba(0,0,0,.25), 0 3px 8px rgba(0,0,0,.35); cursor:grab; touch-action:none; }
.handle.on{ transform:scale(1.15); }
.stops{ display:flex; gap:6px; align-items:center; margin-top:8px; flex-wrap:wrap; }
.stop{ display:inline-flex; align-items:center; gap:5px; border:1px solid var(--line); background:var(--surface-2);
  border-radius:999px; padding:3px 5px 3px 4px; font-family:var(--font-data); font-size:10px; color:var(--ink-soft); cursor:pointer; }
.stop i{ width:14px; height:14px; border-radius:999px; display:block; }
.stop[aria-pressed="true"]{ border-color:var(--ink-faint); color:var(--ink); }
.stop .rm{ border:0; background:none; padding:0; cursor:pointer; color:var(--ink-faint); line-height:0; }
.stop .rm .ic{ width:11px; height:11px; }
.presets{ display:grid; grid-template-columns:repeat(8,1fr); gap:6px; }
.presets button{ aspect-ratio:1; border-radius:999px; border:1px solid var(--line); cursor:pointer; padding:0; }
.presets button:hover{ transform:scale(1.1); }
.checks{ display:grid; gap:5px; font-size:12px; }
.check{ display:flex; align-items:center; gap:8px; color:var(--ink-soft); }
.check .v{ margin-left:auto; font-family:var(--font-data); font-size:10.5px; font-variant-numeric:tabular-nums; }
.capnote{ font-size:11.5px; color:var(--ink-faint); line-height:1.45; }
.sp{ width:22px; height:22px; border-radius:999px; border:2px solid transparent; cursor:pointer; padding:0;
  transition:transform var(--t-quick) var(--e-spring), border-color var(--t-quick) var(--e-out); }
.sp:hover{ transform:scale(1.12); }
.sp[aria-pressed="true"]{ border-color:var(--f-ink); transform:scale(1.08); }
```

| Piece | hover | active / drag | focus-visible | selected |
| --- | --- | --- | --- | --- |
| Handle | not specified | active dot scale 1.15; follows pointer | global ring | active dot (`.on`) |
| Stop | not specified | n/a | global ring | border `--ink-faint`, text `--ink` |
| Preset | scale 1.1 (no transition in S; O-29) | not specified | global ring | n/a |
| Space dot | scale 1.12 | not specified | global ring | border `--f-ink`, scale 1.08 |

**Motion.** Space dot: transform `--t-quick --e-spring`, border `--t-quick --e-out`. Changing the
palette cross-fades the frame (380 ms layer opacity, 05-MOTION). Switching Space slides the
sidebar content (`slide-r` / `slide-l`, `--t-big --e-spring`, `S:102-105`).

**Behaviour.** Pointer down anywhere on the field picks the handle under it (or keeps the active
one) and moves it to the pointer; drag continues with capture (`S:1435-1451`). Keys on a handle:
Left/Right hue -5°/+5° wrapping at 360; Up/Down chroma +.05/-.05 clamped to [0, 1]
(`S:1452-1457`). Up to three dots; "+ Colour" adds one 48° further round (`S:1463`); a stop's
remove button drops it (not the last). A preset replaces the dots. Space dots switch Space
(Ctrl 1..9). 06-INTERACTIONS "space editor", 21-SPACES.

**Blitz notes.**
- `canvas` -> no canvas in Blitz. Options: a `blitz_dom::Widget` painting the dot grid, or a PNG
  per scheme generated in Rust (the plane depends only on the scheme) (O-19).
- `position:sticky` is banned -> the panel is static; the host scrolls it.
- `setPointerCapture` -> DragTracker.
- The white handle border and shadow literals -> tokens (O-3).
- Gradient backgrounds come from `FrameVars` as inline style.

### 33. EdgeStrip

**Purpose.** When the sidebar is hidden (Ctrl S), a 10 px strip on the window's left edge brings
it back as a floating panel while the pointer is on it (Arc-derived). Mail: hidden sidebar.
Shell: dock auto-hide reveal edge (planned).

**Markup.**

```html
<div class="ds-edge" aria-hidden="true"></div>                      <!-- present only while the sidebar is hidden -->
<nav class="ds-side" data-side="shown|hidden|peek" aria-label="Sidebar">…</nav>
```

Mapping: `.win.no-side` -> `.ds-side[data-side=hidden]` plus the window grid change;
`.win.no-side.side-peek .side` -> `[data-side=peek]`.

**Props.**

```rust
#[component] pub fn EdgeStrip(onenter: EventHandler<()>) -> Element
// the sidebar container takes side: SideState { Shown, Hidden, Peek }
```

**Geometry.** Edge: absolute left 0, top 0, bottom 0, width 10, z 14. Peek panel: absolute left 8,
top 8, bottom 8, width 226, z 15, background `--f-solid` (the first gradient stop), radius 14,
`0 20px 40px -16px rgba(0,0,0,.45)`. Window grid when hidden: `0 minmax(0,1fr)`, padding-left 8.

**States.** Verbatim, `S:85`, `S:91`, `S:451-454`:

```css
.win.no-side{ grid-template-columns:0 minmax(0,1fr); padding-left:8px; }
.win.no-side .side{ visibility:hidden; }
.edge{ position:absolute; left:0; top:0; bottom:0; width:10px; z-index:14; display:none; }
.win.no-side .edge{ display:block; }
.win.no-side.side-peek .side{ visibility:visible; position:absolute; left:8px; top:8px; bottom:8px; width:226px; z-index:15;
  background:var(--f-solid); border-radius:14px; box-shadow:0 20px 40px -16px rgba(0,0,0,.45); animation:slide-r var(--t-move) var(--e-spring); }
```

| State | Rule |
| --- | --- |
| sidebar shown | edge not rendered |
| hidden | grid column 0 (transitions `grid-template-columns` and padding over `--t-move --e-out`, `S:83`); sidebar invisible; edge present |
| peek (entering) | sidebar floats over the card, `slide-r --t-move --e-spring` |
| peek leaving | pointer leaves the sidebar: back to hidden at once, no exit |

**Behaviour.** Pointer enters the edge -> peek; pointer leaves the floating sidebar -> hidden
(`S:1842-1843`). A "Sidebar" Mini appears in the list bar while hidden (`S:855`, `S:1584`).
Focus mode hides the sidebar; send and park restore it (`S:2330`). 06-INTERACTIONS "sidebar".

**Blitz notes.** A grid-template-columns transition may not interpolate in Stylo; if it does not,
animate the sidebar's width instead (spike S4 covers var-driven transitions). The peek panel
renders in place (not OverlayHost) because it is the sidebar itself.

### 34. DragGhost and DropTarget

**Purpose.** Moving a thing by dragging. Two patterns in the prototypes: C drags a thread onto a
sidebar place (a tilted ghost card follows the pointer; the place under it lights up and grows);
S drags a composer object by its grip (the object dims; a 3 px accent line shows where it will
land). Mail: both. Shell: dock drag-out and reorder (planned).

**Markup.**

```html
<!-- C: ghost through OverlayHost -->
<div class="ds-drag-ghost" style="left:412px; top:188px">Notes from the sync review
  <div class="ds-drag-ghost-sub ds-truncate">Sam Lindqvist</div></div>
<li class="ds-row" data-drag="source">…</li>                               <!-- was .is-dragging -->
<button class="ds-sidebar-item" data-drop="target">…</button>              <!-- was .is-drop-target -->

<!-- S: grip, dimmed source, drop line -->
<div class="ds-object" data-drag="source|idle">
  <span class="ds-grip" title="Drag to move · click for options">⋮⋮</span>…
</div>
<div class="ds-drop-line"></div>
```

**Props.**

```rust
pub fn use_drag<K>(threshold: Px /* 8, Manhattan */) -> DragTracker<K>   // motion/drag.rs
#[component] pub fn DragGhost(title: String, sub: String, at: Point) -> Element
#[component] pub fn DropLine() -> Element
#[component] pub fn Grip(label: String, onclick: EventHandler<Rect>) -> Element
```

**Geometry.**

| Piece | Value | Source |
| --- | --- | --- |
| Ghost | fixed, z 50, no pointer events, width 250, padding 9px 11px, radius `--r-card`, `--raise`, 1px `--line`, `--shadow-drag`, ui 13 / 700, `rotate(var(--tilt)) scale(1.02)` | `C:579-584` |
| Ghost sub | ui 12 / 500 `--ink-faint`, truncated | `C:585-586` |
| Ghost offset | cursor - (40, 18) | `C:2053-2054` |
| Source | opacity .35 | `C:374`, `S:751` |
| Drop target | `--accent-soft`, `--ink`, scale 1.045, `--shadow-1` | `C:320-323` |
| Grip | 20 x 24, absolute left -24 top 6, radius 6, `--ink-faint`, 12 px text "⋮⋮", cursor grab, opacity 0 until the object is hovered | `S:747-749` |
| Drop line | 3 px high, radius 3, `--accent`, margin 2px 0 | `S:752` |

**States.** Verbatim, `C:579-586` and `S:747-752`:

```css
.ghost{
  position:fixed; z-index:50; pointer-events:none; width:250px; padding:9px 11px;
  border-radius:var(--r-card); background:var(--raise); border:1px solid var(--line);
  box-shadow:var(--shadow-drag); font-size:13px; font-weight:700;
  transform:rotate(var(--tilt)) scale(1.02);
}
.ghost .g-sub{ font-weight:500; color:var(--ink-faint); font-size:12px; white-space:nowrap;
  overflow:hidden; text-overflow:ellipsis; }
.ograb{ position:absolute; left:-24px; top:6px; width:20px; height:24px; border-radius:6px; display:grid; place-items:center; color:var(--ink-faint);
  font-size:12px; cursor:grab; opacity:0; user-select:none; transition:opacity var(--t-quick) var(--e-out), background-color var(--t-quick) var(--e-out); }
.obj:hover > .ograb{ opacity:1; }
.ograb:hover{ background:var(--surface); color:var(--ink); }
.obj.dragging{ opacity:.35; }
.drop-line{ height:3px; border-radius:3px; background:var(--accent); margin:2px 0; }
```

| State | Rule |
| --- | --- |
| pending | pointer down on a row (not on a button); nothing visible until 8 px Manhattan movement |
| live | ghost at cursor - (40, 18); source .35 |
| over target | target: drop-target style; only places that accept (inbox, snoozed, archive, trash, labels; not starred) |
| dropped on label | chip added with `chip-land`, place gulps (§19) |
| dropped elsewhere | the op runs (snoozed -> snooze, inbox -> restore, else the place's op) |
| cancelled / released off target | ghost removed, source restored, no animation |
| grip hover | `--surface`, `--ink` |
| grip click | opens the object Slim Menu (Move up, Move down, Duplicate, Delete) |

**Motion.** Ghost has no enter/exit animation; the drop target change is instant in C (no
transition on `.view` transform beyond its `--t-tap` press transition). `--tilt` is 2.2deg (Post;
S drops tilt, A7). Motion levels: Calm tilt 0.

**Behaviour.** 06-INTERACTIONS "drag and drop". Hit testing under the pointer
(`elementFromPoint`, `C:2055`) becomes a Rust query of registered drop-target rects.

**Blitz notes.** HTML5 drag events (`dragstart`/`dragover`, `S:2269-2275`) do not exist; both
patterns use the Rust DragTracker. Ghost is `position:fixed` in C; in ds it goes through
OverlayHost at a Rust point. Ghost sub ellipsis -> `.ds-truncate`. Transforms on the ghost only.

### 35. SyncHalo

**Purpose.** A ring around an account avatar that says whether that account is live: breathing
while idle and listening, dashed and spinning while syncing. Only in C (`C:1179-1183`). Mail:
account in the sidebar. Shell: bar sync indicator. It is Spinner (§15) mounted on an Avatar.

**Markup.** (from `C:1179-1183`)

```html
<div class="ds-account-ring" data-sync="idle|busy">
  <span class="ds-spinner" data-kind="breathe|spin"></span>
  <span class="ds-avatar" data-size="30">P</span>
</div>
```

**Props.** `#[component] pub fn SyncHalo(initial: char, tone: AvatarTone, state: SyncState /* Idle | Busy */) -> Element`

**Geometry.** Ring box 30 x 30, relative, flex none. Avatar absolute inset 0, round, `--paper` on
`--ink`, display 800 13. Halo: §15.

**States.** Verbatim, `C:278-292`:

```css
.acct-ring{ position:relative; width:30px; height:30px; flex:none; }
.acct-ring .av{
  position:absolute; inset:0; border-radius:999px; display:grid; place-items:center;
  background:var(--ink); color:var(--paper);
  font-family:var(--font-display); font-weight:800; font-size:13px;
}
.acct-ring .halo{
  position:absolute; inset:-4px; border-radius:999px; border:2px solid var(--accent);
  opacity:0; transform:scale(.85);
}
.acct[data-sync="idle"] .halo{ animation:breathe var(--t-ambient) ease-in-out infinite; }
.acct[data-sync="busy"] .halo{
  opacity:.9; transform:scale(1); border-style:dashed;
  animation:spin 1.1s linear infinite;
}
```

| State | Rule |
| --- | --- |
| idle | `breathe` over `--t-ambient`, infinite |
| busy | dashed, opacity .9, `spin 1.1s linear infinite` (C holds busy for 1200 ms per sync, `C:2092-2100`) |
| hover, focus, disabled | not specified (not interactive) |

**Motion.** `breathe`, `spin`. See §15 for the loop cost.

**Blitz notes.** See §15. S dropped the halo (A7: "no … t-ambient"); keeping it is O-25.

### 36. MenuBarItem (the macOS polish pass, settled 2026-09-24)

**Purpose.** A bar title or text status item (the app name, a menu title, the clock) and the pill
behind it. macOS draws a rounded highlight behind the open title and the hovered item
(design/13 section 13.3.1); `IconButton { Status }` already draws its own, so only text needs this.

**Markup.** `div.ds-bar-item[aria-expanded][data-emphasis=strong]` around the control.
**Props.** `open: Switch` (the menu is showing), `emphasis: Emphasis` (`Strong` = 700 for the app
name), `id`, `children`.
**Values.** Height `--shell-bar-item` 24 (`bar.open_title_pill_height_px`), side padding
`--shell-bar-pad` 10 (`bar.title_padding_px`), radius `--r-shell-bar-item` 4
(`bar.item_radius_px`, proposed key), text `--fs-shell-bar` 13 / `--fw-shell-bar` 500; hover
`--f-pill-hover`, open `--f-pill`, no transition (a hover switch lands in one frame). A
`Button { Quiet }` inside gives up its own padding, hover and size.

### 37. WorkspacePills (the macOS polish pass, settled 2026-09-24)

**Purpose.** The bar's workspace indicator as one segmented group on the frame rather than a row
of separate `Button { Mini }`s.
**Markup.** `div.ds-ws-pills[role=group]` holding `button.ds-ws-pill[aria-current]`.
**Props.** `WorkspacePills { label, children }`; `WorkspacePill { label, current: Here, onclick:
EventHandler<Press>, id }`. A caller that drags pills to reorder wraps each pill in its own element
and listens there.
**Values.** Track `--surface` (the frame's `--f-pill-hover`), 24 high, radius 4, padding 2; the
current pill `--surface-2` (`--f-pill`) with `--shadow-current`; others `--ink-soft`, hover
`--ink`; text at the bar's shell size.

### 38. RunningDot and DockFloor (the macOS polish pass, settled 2026-09-24)

**Purpose.** The dock's running indicator and its optional reflective floor (design/10 section
10.3.2).
**Markup.** `span.ds-running-dot` inside a positioned tile; `div.ds-dock-floor` first in the dock
root.
**Values.** Dot `--dock-dot` 4 px, its centre `--dock-dot-gap` 3 px under the tile, `--ink` of the
dock scope, `fade` in over `--t-quick`. Floor: a light band rising from the pill's floor
(white .26 to 0 over 40 %), shown at `--dock-floor` (0 unless `dock.floor = On`), masked with the
pill when the pill is a squircle.

**IconView plates.** `IconView { plate: Some(PlateFamily) }` sits the icon on an app-icon plate
`size` square (design/08 sections 2.1-2.5, 4.1): `span.ds-plate[data-family]` with a masked
`span.ds-plate-face` (the whole `n = 5` superellipse, the family's 135 degree gradient, inner
highlight white .35 and rim black .08) and a drop `0 1px 1.5px .16, 0 6px 16px -6px .3` on the
unmasked box; a glyph or symbolic icon at `--plate-glyph` 56 % in the family's glyph colour, an
image at `--plate-inset` 72 %. Families `Red`, `Amber`, `Green`, `Blue`, `Violet`, `Neutral`
(the neutral plate turns `#2A2E28` to `#1D211B` in the dark scheme). `plate_tint:
Some(PlateTint)` (`Muted`, or `Monochrome(Tint)`) re-colours the stops and the ink by `retint`'s
rule for both schemes, written inline as `--plate-base-l`/`-deep-l`/`-ink-l` and `-d`, marked
`data-icon-style`, and read by the sheet under `data-theme` (sill FINDINGS Q72; design/08 4.4).

### Window frame: WindowFrame, the titlebar and the traffic lights (settled 2026-09-25)

**Purpose.** The frame of a client-decorated window: our apps on `ds_native::launch` (mailo)
and, later, the shell's apps on shell-host toplevels. It moves, resizes and zooms the window
through the host seam `ds::HostWindow` (design/13 section 13.3.11; FINDINGS "Window frame").
**Props.** `Ds { window: WindowFrame }`. `WindowFrame::None` (default) draws nothing and leaves
the root's markup as it was; `WindowFrame::Titlebar { title, lights: TrafficLights::{Shown,
Hidden}, timing: FrameTiming }` (`WindowFrame::titlebar(title, lights)` takes the settings'
default timing). `WindowTitlebar { title, lights, timing, pose: TilePose::{Closed, Open} }` is
the titlebar alone, for a gallery.
**Markup.** The root stamps `data-window-frame="titlebar"` and becomes a column:
`div.ds-titlebar[data-window][data-activation][data-first-mouse]` holding
`div.ds-lights[role=group]` (three `button.ds-light[data-light=close|minimize|zoom]`, each with
an `svg.ds-light-mark`) and `span.ds-titlebar-title.ds-truncate`; then `div.ds-window-body`
with the children; then eight `div.ds-resize-edge[data-edge]` (not while maximized or
fullscreen).

| Metric | Value | Basis |
| --- | --- | --- |
| Titlebar height | 28 | macOS standard titlebar (28 pt) |
| Light diameter | 12 | macOS (12 pt) |
| Gap between lights | 8 (`--s-8`) | macOS (20 pt centre to centre) |
| Inset of the first light | 13 (`--s-13`) from the left edge, centred on the titlebar | brief (macOS) |
| Title | 13/600 (`--fs-control`), `--f-ink-soft`, `--f-ink-faint` inactive, centred between 84 px insets, `.ds-truncate` | macOS |
| Light hues | close `--c-red`, minimize `--c-amber`, zoom `--c-green` (the Candy shelf) | design/03 section 15 |
| Mark ink | the hue's `-deep` in light, its `-soft` in dark (dark on the disc in both) | macOS |
| Resize zones | 4 px along each side, 12 x 12 at each corner, `--z-edge` | settled 2026-09-25 |

**The reveal rule.** In the active window the lights are coloured at rest; in an inactive one
they are `--f-pill` discs with a `--f-line` hairline (grey). The pointer over any light colours
all three (inactive included) and shows all three marks; the keyboard focus on a light shows its
mark; the green light shows its mark while its menu is open. This is macOS's rule: grey until
hover applies to background windows, the active window keeps its colours.
**Marks.** A cross, a bar, and the zoom mark: two outward corners, which turn inward
("restore", `aria-label="Restore"`) while the window is maximized.
**Behaviour.** A primary press on the titlebar's empty area that travels more than
`move_threshold` (4 px) on either axis asks `begin_move` once; less is a click. A double-click
on the titlebar is `zoom(Zoom::Toggle)`. Neither happens on a light (each light keeps its
`pointerdown` and `dblclick`), and the move not while the window is maximized or fullscreen. A
press on an edge zone asks `begin_resize(edge)` at once. Close, minimize and zoom on a click;
the green light held for `menu_press` (500 ms), rested on for `menu_hover` (800 ms: the 450 ms
hover intent plus 350 ms), right-clicked, or given ArrowDown opens the Move & Resize menu (a Slim
`Menu`): Fill (`zoom(Maximize)`), Left half, Right half, Centre (`tile(..)`), each
`Availability::Disabled` where `supports(..)` said `Support::No` as the menu opened. A hold that
opened the menu does not also zoom on its release. Escape closes the menu (the menu's own);
Tab reaches the three lights in order.
**Motion.** A light springs back from `--squish` over `--t-tap --e-spring` (design/05 principle
2: the press is contact); colours and marks cross-fade over `--t-quick --e-out`.

### 39. MonthGrid (the calendar widget's month; sill Q180, 2026-09-26)

**Purpose.** One month of days in seven columns: the calendar widget of the notification center
and the desktop (design/20 section 1.12, "month grid (new ds component `MonthGrid`)"; section
1.14). The shell computes the month (which days, which column comes first, which day is today,
which days have an event, the ISO week of each row); quire only draws it (sill Q180). Its data
mirrors the shell's grid field for field, so the shell's mapping is a plain `From`.

**Markup.**

```html
<div class="ds-month" data-weeks="hide|show" aria-label="September 2026">
  <div class="ds-month-header">
    <span class="ds-month-title">September 2026</span>
    <!-- only with onstep -->
    <button class="ds-icon-button" data-variant="tool" aria-label="Previous month">…</button>
    <button class="ds-icon-button" data-variant="tool" aria-label="Next month">…</button>
  </div>
  <div class="ds-month-row" data-row="heads">
    <span class="ds-month-week"></span>            <!-- only with data-weeks=show -->
    <span class="ds-month-head">M</span> … seven
  </div>
  <div class="ds-month-weeks [a-slide-l|a-slide-r]" data-pulse="a"?>   <!-- keyed by the month -->
    <div class="ds-month-row">
      <span class="ds-month-week">36</span>          <!-- only with data-weeks=show -->
      <span|button class="ds-month-day" data-kind="pressable"? data-place="before|in|after" data-events="busy|free"
           aria-current="date"?>
        <span class="ds-month-num">31</span>
        <span class="ds-month-dot"></span>           <!-- only when busy -->
      </span|button>  … seven
    </div> … four to six
  </div>
</div>
```

**Props.** `#[component] pub fn MonthGrid(data: MonthGridData, weeks: WeekNumbers /* Hide */,
onstep: Option<EventHandler<Step>>, onpick: Option<EventHandler<DayKey>>) -> Element`.
`MonthGridData { month: MonthKey, title: Text, heads: [Text; 7], weeks: Vec<MonthWeek> }`;
`MonthWeek { number: IsoWeek, days: [MonthDay; 7] }`; `MonthDay { key: DayKey, place: DayPlace,
mark: DayMark, events: Eventful }`. `MonthKey { year, month }` and `DayKey { year, month, day }`
are the civil month and date as the shell's calendar library gives them (`i16`, `i8`, `i8`);
the cell's label is the key's day. `DayPlace::{Before, InMonth, After}`,
`DayMark::{Plain, Today}`, `Eventful::{Free, Busy}`, `WeekNumbers::{Hide, Show}` (the setting
`calendar.week_numbers`, design/22 section 3.21), `Step::{Previous, Next}`. No header buttons
without `onstep`; the days are `button`s only with `onpick`, which hears the day's key.

**Geometry.**

| Part | Value | Basis |
| --- | --- | --- |
| Day cell | 32 wide, 30 high; the number on a 24 round, `--fs-help` 11.5 / 500, tabular | proposed |
| Event dot | 4 round, 2 under the number, centred | brief (sill Q180) |
| Heads | the data face, `--fs-micro` 9.5, upper, tracked .14em as `.ds-section-header` (menu), `--ink-faint`, 18 high | design/02 section 4 ("calendar month and weekday") |
| Title | the data face, `--fs-caption` 10 / 700, upper, tracked .12em, `--accent` | design/02 sections 4-5 ("calendar month", all upper) |
| Header | 28 high, the title then the two `IconButton { Tool }` (28 x 26) at the end | design/20 section 1.12 |
| Week number column | 24 wide, the data face `--fs-micro`, `--ink-faint` | proposed |
| Rows | no gap; the grid is `7 x 32` (224) wide, `24 + 224` with week numbers | proposed |

**States.**

| State | Rule |
| --- | --- |
| in the month | `--ink` |
| the neighbours' days (`data-place=before|after`) | `--ink-faint` (quieter, still legible) |
| today (`aria-current="date"`) | the number on an `--accent` disc in `--accent-ink`, 700 |
| busy (`data-events=busy`) | a 4 px dot under the number: `--accent` (today's too: the dot sits under the disc, not on it); `--ink-soft` on a neighbour's day |
| pressable hover (`data-kind=pressable`, the `button` form) | the number's disc `--surface-2`; today keeps its accent |
| pressable active | the disc squishes (`--squish`, `--t-tap`) |
| focus | the global ring on the cell |

**Motion.** A month change plays once: the weeks are keyed by the month, so a new month mounts a
new body, which plays `slide-r` (a later month, from the right) or `slide-l` (an earlier one)
at the catalogue's row, `--t-big --e-spring` (`a-slide-r`/`a-slide-l` with `data-pulse="a"`),
and drops the class at `settle(Anim::SlideR)`. The direction is the order of the two months, so
the caller says nothing; the first month drawn, and every render that does not change the month,
plays nothing (design/05 principle 7, nothing loops: every motion is keyed to a state change). The header and heads do not move.

**Blitz notes.** Rows are CSS grid (`repeat(7, 32px)`), as `ModuleGrid`; the dot is a real span;
no pseudo-elements.
### 40. Widgets: WidgetFrame, WidgetMetrics, ClockFace and LevelRing (settled 2026-09-26)

**Purpose.** The card every widget is drawn on, on the desktop layer and in the notification
center's widget column (design/20 section 1.14; design/22 section 3.20; sill FINDINGS Q182,
Q183), and the two faces sill cannot draw from text alone: an analog world clock and a battery
level ring. The card's corner, padding and title row are the frame's, so a widget writes no CSS
for them.

**WidgetMetrics.** Writes the grid unit as tuned tokens on any element around the widgets:
`--widget-cell` from `widgets.desktop_cell_px` (164, held to `120..=240`) and `--widget-gap` from
`widgets.desktop_gap_px` (16, held to `0..=48`); `WidgetMetrics::default().style_attr()` writes
what the stylesheet falls back to.

**WidgetFrame.** `WidgetFrame { size: WidgetSize::{Small, Medium, Large}, host:
WidgetHost::{Desktop, Tile}, title: Option<WidgetTitle { glyph: Icon, text: Text }>, id:
Option<String>, children }`.
Markup: `div.ds-widget[data-size][data-host]` holding an optional
`div.ds-widget-title` (a `Glyph` at 14 and the text) and `div.ds-widget-body`.

| Metric | Value | Basis |
| --- | --- | --- |
| Small | `--widget-cell` square (164) | design/22 section 3.20 |
| Medium | `2 x --widget-cell + --widget-gap` wide, one cell tall (344 x 164) | design/22 section 3.20 |
| Large | `2 x --widget-cell + --widget-gap` square (344) | design/22 section 3.20 |
| Desktop card | the `Widget` material's plate inside a transparent `Widget` scope (as the notification plate): its tint, hairline and soft drop, its own corner `--m-radius` (20), padding `--s-16` | design/20 section 1.14 (material `Widget`) |
| Tile | no material of its own, on the Popover it sits in: `--surface-2` fill, `--line` hairline, `--r-tile` (12), padding `--s-12`, as a `ModuleTile` | brief (Q182) |
| Title row | glyph 14 and `--fs-help` 600 in `--ink-soft`, `--s-6` apart, `--s-8` above the body | brief |

The tile takes the same footprint as the desktop card: the center's column (384 less its
padding) holds a medium tile's 344.

**Bump on change.** `use_bump_on(value)` returns the `PulseKey` of an `Anim::Bump` fired through
`use_pulse` each time `value` differs from the one last rendered (never on mount), and back at
rest `settle(Bump)` after each firing, so a battery's percentage or a clock's minute bumps once
and nothing loops (design/20 section 1.14: "value change `bump`"). `Bumped { on: value, children
}` wraps its children in `span.ds-bumped` wearing that pulse, for a widget that bumps a run of
text rather than a component.

**ClockFace.** `ClockFace { time: ClockTime { hour, minute, second: Seconds::{Shown(s), Hidden}
}, phase: DayPhase::{Day, Night}, look: ClockLook::{Analog, Digital}, label: Text }`.
Markup: `div.ds-clock[data-look][data-phase]`. Analog: a dial (`div.ds-clock-dial`, 72 round, in a
scope forced to the light scheme by day and the dark by night, so the day face is always paper
with ink hands and the night face ink with paper hands whatever the desktop's scheme) holding
two `svg[data-ds-svg]` drawn on `currentColor` (twelve ticks, the hour and minute hands and the
hub; the second hand in `--accent` when shown). Digital: the time in `--font-data` tabular
(`09:41`, `09:41:07` with seconds), bumping on each new minute. The label (the city) sits under
either in `--fs-help`. The hand angles are pure (`clock_angles.rs`): hour `30 x (h mod 12) + m / 2
+ s / 120` degrees, minute `6 x m + s / 10`, second `6 x s`.

**LevelRing.** `LevelRing { level: Fraction, mark: RingMark::{Plain, Charging}, label: Text,
children }`: a stroked ring (`svg.ds-ring[data-ds-svg=ring]`, circumference 100 so the dash is
the level in percent), a track at .2 of `currentColor`, the level in `--ok`, `--warn` at or under
20 %, `--danger` at or under 10 % (a charging ring stays `--ok`); `Charging` adds a bolt on a
paper disc at the top. `children` (a device glyph) sit in the middle; `role=progressbar` with
`aria-valuenow` in percent. The level bumps on change (`use_bump_on`). No `ProgressRing` existed:
the SendPill's ring is a countdown drawn inside the pill, not a level.

**Motion.** None in steady state; a value change plays `bump` once (`--t-move --e-spring`, the
Count's pulse). No transition on the ring's dash (a `stroke-dashoffset` transition does not run in
Blitz, O-20).

## Open decisions

Each needs a yes/no or a number before the owning wave starts. "Proposal" marks this document's
suggestion.

| # | Topic | What is open | Proposal (numbers here are invented candidates, not prototype values) |
| --- | --- | --- | --- |
| O-1 | Disabled | No prototype draws a disabled control. | One rule for all: reduced opacity (candidate .45), default cursor, no hover, no press. Value needs sign-off. |
| O-2 | Shortcut text | S writes "Ctrl T" as data text and `kbd` per key; the plan renders `⌘⇧⌥⌃`. Separator between keys: space (S) or thin space (C). On Linux the modifier is Ctrl (Toshy maps Cmd -> Ctrl). | Glyphs, no separator (`⌃T`), per the plan. |
| O-3 | Literals without tokens | current-item shadow `0 1px 0 rgba(255,255,255,.4) inset, 0 2px 6px -3px rgba(0,0,0,.25)`; command pill inset `rgba(255,255,255,.35)`; Pop `0 18px 40px -16px rgba(0,0,0,.45)`; Bubble `0 12px 28px -12px`; palette `0 30px 60px -20px rgba(0,0,0,.5)`; peek `0 24px 50px -18px rgba(0,0,0,.55)`; side peek `0 20px 40px -16px`; handle ring and shadow; provider rings; `#fff`/`#FFFFFF` (fav text, provider ground, handle border, lying link text); easing `ease-in-out`; durations 1.1 s, 900 ms (`dest`), 1200 ms (flash); scales .88, 1.045, 1.08, 1.1, 1.12, 1.15, rotate -6deg; radii 5, 6, 7, 8, 10, 12. | Token table absorbs each (03/01/05). |
| O-4 | Toggle size | Knob and track sizes are not in any prototype. | Candidate: knob 18, track 36 x 24 incl. 3 px padding. Needs sign-off. |
| O-5 | Slider track | Height and unfilled colour not specified (native range). | Candidate: 4 px, `--line` unfilled, `--accent` filled. |
| O-6 | Chip enum | S uses Person, Token, Status and Clip shapes; the plan's enum has Accent, Label, Neutral. | Add the four as variants (§10). |
| O-7 | Person hue | `hsl()` from the hash is a colour function the lint bans in consumer CSS. | Rust converts to hex and emits `--av-bg`. Settled for consumers (Gallery fixes B): the markup lint allows a custom property written inline on a `ds`/`ds-*` element, so a rendered `Avatar` needs no exception. |
| O-8 | Count bump on Space switch | S suppresses bumps when the Space changes (`S:1262`). | The consumer keys the count by Space, so a switch mounts it at rest (wave 1: replaces the `Motion::Quiet` render hint, which no type carries). |
| O-9 | Infinite loops | Spinner, SyncHalo and `dest` loop; loops keep Blitz animating and the shell never idles. | Breathe off by default on shell surfaces; Spin only while busy; `dest` only while hovered. |
| O-10 | Strip `!important` | `.strip button:active{ transform:scale(.88) !important }` beats the `forwards` fill; lint bans `!important`. | Settled state without animation after `settle(pop-in)` (§17). |
| O-11 | Warm hover attribute | S puts `.warm` on `.win`. | `HoverHub` stamps `data-hover="warm|cold"` on `.ds`. |
| O-12 | Menu separator | Not in S; derived from the bubble separator. | Candidate: 1 px `--line-soft`, margin 4px 0. |
| O-13 | Menu item hover | S has none (keyboard only); C Dropdown uses `--surface-2`. | Pointer move over an item moves the selection (`--accent-soft`), so hover and keyboard look the same in Rich/Slim/Context; Dropdown keeps C. |
| O-14 | Sheet size | Derived from Peek; no size given. | Candidate: width `min(560px, 88%)`, height by content, max = Peek Center inset. |
| O-15 | Modal focus | Peek and Sheet do not move or trap focus in S. | Focus first focusable on open, restore on close, Tab trapped. Partly settled (FINDINGS "W2 integration", focus on mount): `TextInput{focus: Focus::OnMount}` exists and the palette input and bubble link field use it; giving focus back is settled (2026-09-24, FINDINGS "Launcher gaps"): `Focus::Controlled(FocusRequest)` and `request()`. Peek and Sheet still neither move nor trap focus. |
| O-16 | AppearancePicker contents | Whether Look and Warmth (C) are part of it; width. | Theme, Accent, Motion only (the plan's `Appearance`); width set by the host surface. |
| O-17 | Accent count | Plan: "Accent(6)" in the gallery and a moved test named "exactly-four-accent". | 03-COLOR decides; the picker renders the table. |
| O-18 | S vs C contradictions | Resolved "S wins" in each section; listed here so nobody re-opens them silently: strip buttons 26 vs 28, gap 3 vs 4, icons 14 vs 15; star inset 6/7 vs 8/8, icon 14 vs 15; icon stroke 2 vs 1.7/1.8; toast hidden 160% vs 140%, hint opacity .7 vs .72; seg padding 5/11 12px vs 6/13 12.5px; Primary padding 14 vs 15; palette 540 vs 420, top 11% vs 14%, input 16 vs 14.5, list 360 vs 252, enter `peek-in` vs `cmdk-in`; peek inset `36px 12%` vs `34px 10%`, `peek-in` .95/12px vs .97/10px; scrim `--scrim` vs ink .16; row dot top 6 vs 5; stagger cap none (S, first show only; ds caps 12) vs 8; sidebar item radius 9 on the frame vs `--r-chip` on the card, seal -7 in `--f-ink` vs -10 in `--seal`; toast click-to-undo |dx| < 3 vs dx == 0; send ring 20 vs 22. | S wins, as written in each section. |
| O-19 | Field plane | No canvas in Blitz. | PNG per scheme generated in Rust (static per scheme), handles as divs. |
| O-20 | Send ring animation | SVG `stroke-dashoffset` transition likely does not run in Blitz. | Rust drives `Fraction` per frame; fallback a custom Widget. The ring carries `data-ds-svg="ring"`, so the markup lint knows it is quire's own vector (Gallery fixes B). |
| O-21 | Selection source | The bubble needs a selection rect and mark states that only the composer's Rust model has. | ds owns surface + placement; mailo owns selection. |
| O-22 | Pseudo-elements | Seal `::before`, header rule and tab underline `::after`. | Spike confirms; fallback real spans. |
| O-23 | `::placeholder` | Not confirmed in Blitz. | Spike; fallback overlay span. |
| O-24 | Strip keyboard reveal | S reveals on hover only; C also on `:focus-within`. | Adopt C (keyboard users can reach the actions). |
| O-25 | SyncHalo | S removed it; C has it; "nothing loops" principle. | Keep for the shell's sync indicator only, with O-9. |
| O-26 | Unpressed account tile | `filter:saturate(.55)` banned. | Rust-computed desaturated colour + opacity .85. |
| O-27 | Provider icons | "Their icons" are provider favicons fetched at setup (`S:1139-1141`); licensing not settled. | ds renders what it is given; mailo decides whether to ship icons or letters by default. |
| O-28 | Editor sticky | `position:sticky` banned. | Static panel; the host scroll view keeps it in view if needed. |
| O-29 | Preset hover | `transform:scale(1.1)` with no transition (snaps). | Add `transform var(--t-quick) var(--e-spring)` to match the Space dot. |
| O-30 | Link pill truncation | Must never hide the registered domain. | Truncate the path first, then the subdomain; never the bold part. |

## Sources

- S: `~/mailo-design/mailo-spaces.html`. CSS `S:6-809`; markup `S:812-927`; script `S:929-2367`.
  Components by range: tokens `S:7-46`; base `S:48-59`; `kbd` `S:67-68`; window and frame
  `S:77-105`; tiles `S:107-118`, `S:356-357`, `S:548-551`; section header `S:120-125`; sidebar
  item `S:126-143`, `S:339-353`, `S:447-448`, `S:724`; foot and Space dots `S:145-153`; card and
  list `S:156-199`; tool `S:204-206`; scrim and peek `S:221-227`; palette `S:230-241`,
  `S:792-799`; editor `S:246-279`; row motion, star, strip `S:289-336`; count `S:341-342`; toast
  `S:360-370`; Primary button `S:386-391`; hover card `S:396-428`; link pill `S:430-437`; fly
  `S:440-446`; edge `S:450-454`; view switch `S:537-539`; clip `S:543-544`; provider marks
  `S:552-561`; chips `S:592-598`; inputs `S:599-605`, `S:781-786`; menus `S:664-680`,
  `S:787-791`; bubble `S:682-691`; send pill `S:705-715`; grip and drop line `S:745-752`; reduced
  motion `S:807-809`. Script: sidebar render `S:1227-1267`; row render `S:1269-1290`; editor
  `S:1392-1472`; ops, undo, toast `S:1517-1572`; peek `S:1576-1583`; palette `S:1590-1671`; hover
  manager `S:1697-1809`; link pill `S:1811-1829`; destination `S:1831-1839`; edge `S:1841-1843`;
  menus `S:2050-2105`; bubble `S:2123-2165`; object drag `S:2267-2275`; send `S:2315-2345`.
- C: `~/mailo-design/mailo-charm.html`. Tokens and looks `C:12-176`, `C:776-966`; controls `C:224-257`;
  shell, sidebar, halo, view `C:262-339`; list, row, star, strip `C:341-443`; toast `C:556-576`;
  ghost `C:578-586`; keyframes `C:650-749`; tool and dropdown menu `C:992-1023`; group header
  `C:1025-1032`; peek modes, scrim, palette `C:1043-1085`; markup `C:1110-1250`; script views, rows,
  menus `C:1647-1778`; picker, drag, sync `C:1962-2116`; palette `C:2214-2260`.
- Plan: `~/.claude/plans/vast-toasting-peach.md`, sections "Design: `<ds>` design-system repo"
  (component list, props vocabulary, §11 addenda, Blitz risk table), "Findings: Blitz / Dioxus
  Native", "UX decisions settled with the user", Appendix A (A1 layout, A3 colour usage, A4
  components, A5 motion, A6 interactions, A7 looks, A8 bugs).
- House style: `~/quire/CONVENTIONS.md` §0.
- Sibling docs referenced: `01-LAYOUT.md`, `02-TYPE.md`, `03-COLOR.md`, `05-MOTION.md`,
  `06-INTERACTIONS.md`, `07-LOOKS.md`, `20-SURFACES.md`, `21-SPACES.md`.
