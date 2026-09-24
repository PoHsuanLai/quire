# 01 Layout

## 1. What this governs

This file fixes where things sit and how big they are: the window grid, the sidebar, the card
with its list and reader, rows, overlays and their placement, the composer page, every radius,
fixed control sizes, the spacing scale, z-order, responsive behaviour, and the shell surfaces
(bar, dock, launcher and the rest). All values are CSS pixels, which the implementation treats
as logical pixels (`P:459`: integer logical px, scale in 120ths). Colour is `03-COLOR.md`, type
is `02-TYPE.md`, motion is `05-MOTION.md`; a layout rule that names a colour or a curve points
there. Source keys `S`, `C`, `P` are defined in `00-PRINCIPLES.md` section 1; `S` wins every
conflict with `C`.

## 2. Units and the spacing scale

All lengths are logical pixels; the scale is dense and not a geometric ramp. Every padding, gap
and margin in `S`'s app surfaces is one of these values:

| Group | Values (px) |
| --- | --- |
| Common steps | 1, 2, 3, 4, 5, 6, 7, 8, 9, 10, 11, 12, 14, 16, 18, 22, 26, 36 |
| Odd values | 1.5 (chip padding-y), 40 (composer scroll bottom), 46 (gutter offset), 50 (inline reply body padding-left), 56 (page wrap bottom) |
| Floating-surface clamp margin | 8, always (`S:1724`, `S:2064`, `S:2144-2145`) |
| Card inset inside the window | 8 (`S:81`) |

`P:955-956`, derived from the `S` rules cited in the sections below.

**Settled (Gallery fixes B, 2026-09-24).** The common steps are tokens: `ds::SpacingToken`
(`crates/ds/src/tokens/spacing.rs`) emits `--s-1`, `--s-2`, … `--s-12`, `--s-14`, `--s-16`,
`--s-18`, `--s-22`, `--s-26`, `--s-36` on `.ds`, each named by its own pixel value, the same in
both schemes and at every motion level. The lint's Strict profile has `Rule::RawSpacing`: a
literal `px` in `margin`, `padding` (and their sides and logical forms) or a `gap` is an
offence in consumer CSS. The odd values stay with their one component each.

**Settled (Polish pass, 2026-09-24).** quire's own component sheets read the tokens, and
`Rule::RawSpacing` runs on them. Three steps were added because 04-COMPONENTS quotes them
exactly for a quire component: `--s-1-5` (1.5, the chip and the image provider mark), `--s-13`
(13, the hover card's `12px 13px`) and `--s-15` (15, the toast's `5px 5px 5px 15px`).

Measures: reading and writing text is capped at `66ch` (`S:462`, `S:608`, `S:728`); focus-mode
composer at `70ch` (`S:574`).

### 2.1 Pixel snapping (settled 2026-09-25)

**The rule: every line is a whole number of device pixels, at every scale.** A panel runs at
1.25, 1.5 or 1.75 as readily as at 1 or 2 (a 27 inch 4K panel at 1.5 is about Apple's 109
points per inch). macOS renders a fractional scale at 2x and downsamples; we render at the true
scale, so a `1px` line would be 1.25-1.75 device pixels and blur into two half-ink rows. So:

- Line widths come from the pixel tokens (`ds::PixelToken`), which the root writes for its
  device scale (`Ds { scale }`, else the host's): `--hair` for a 1 px line (a border, a
  separator, a rule, a 1 px ring or inset highlight), one device pixel at 1.25-1.75 and 1 px at 1x
  and 2x; `--hairline` for the material stack's .5 px hairline, one device pixel everywhere but
  1x; `--px` for exactly one device pixel; `--ring` (3 px) and `--focus-ring` (2.5 px) rounded to
  whole device pixels; `--dpr` the scale itself. At 1x every token is its design value, so
  nothing drawn at 1x changes.
- Positions are snapped by the host: Blitz rounds boxes to whole logical pixels, which at a
  fractional scale starts a box half-way through a device pixel. `ds_native::snap_to_device`
  re-rounds the laid-out document on the device grid after every resolve (ds-native's snapshots
  and harness do; a host that resolves its own documents calls it).
- A glyph's stroke is an even number of device pixels at a fractional scale (08-ICONS §1.4.1).
- No consumer stylesheet writes a literal `1px`/`.5px` line width: `Rule::RawHairline`.

What stays unsnapped is recorded in FINDINGS "Pixel snapping": text, a transform that is not a
pure translation (a motion part-way through), a glyph's diagonals and curves, and the portable
`ds_native::launch` window, which has no hook between Blitz's resolve and its paint.

## 3. The window

The window is a coloured frame with the sidebar on the colour and a card inset 8 px on three
sides. Verbatim:

```css
.win{
  --f-ink:#1b1b22; --f-ink-soft:#44444f; --f-ink-faint:#5d5d6a;
  --f-pill:rgba(255,255,255,.58); --f-pill-hover:rgba(255,255,255,.34); --f-line:rgba(0,0,0,.08);
  position:relative; min-width:980px; height:680px; border-radius:18px; overflow:hidden;
  display:grid; grid-template-columns:232px minmax(0,1fr); padding:8px 8px 8px 0;
  color:var(--f-ink); isolation:isolate; box-shadow:0 18px 40px -22px rgba(0,0,0,.45);
  transition:grid-template-columns var(--t-move) var(--e-out), padding var(--t-move) var(--e-out);
}
.win.no-side{ grid-template-columns:0 minmax(0,1fr); padding-left:8px; }
.layer{ position:absolute; inset:0; z-index:-2; transition:opacity 380ms var(--e-out); }
.grain{ position:absolute; inset:0; z-index:-1; pointer-events:none; mix-blend-mode:overlay; background-size:128px 128px; }
```

`S:77-87`

| Property | Value | Source |
| --- | --- | --- |
| Minimum width | 980 | `S:80` |
| Height (prototype) | 680 | `S:80` |
| Corner radius | 18 | `S:80` |
| Columns | sidebar 232, card `minmax(0,1fr)` | `S:81` |
| Padding | top 8, right 8, bottom 8, left 0 (the sidebar supplies its own left padding) | `S:81` |
| Sidebar hidden | columns `0 minmax(0,1fr)`, padding-left 8 | `S:85` |
| Frame layers | two `.layer` elements (a and b) for the Space cross-fade, then `.grain`, then content | `S:835-838` |

`C` has no frame. Its shell is one bordered panel with three columns
`206px minmax(0,1fr) minmax(0,1.05fr)`, height `min(72vh,700px)`, min-height 560, border 1 px
`--line`, radius `--r-panel`, background `--surface`, shadow `--shadow-2` (`C:262-268`). `S` wins.

## 4. The sidebar

The sidebar is a column drawn straight on the colour: command pill, a scrolling slide, a foot.

```css
.side{ display:flex; flex-direction:column; min-width:0; padding:10px 10px 8px 12px; overflow:hidden; }
.win.no-side .side{ visibility:hidden; }
.slide{ display:flex; flex-direction:column; min-height:0; flex:1; overflow-y:auto; scrollbar-width:none; margin-right:-4px; padding-right:4px; }
.side-foot{ margin-top:auto; padding-top:10px; display:flex; align-items:center; gap:6px; }
```

`S:90-91`, `S:101`, `S:145`

Order, top to bottom (`S:840-848`, `S:1259`):

1. `.cmd` command pill: full width, padding 8/10, radius 10, gap 8 (`S:92-97`).
2. `.slide` (scrolls, scrollbar hidden) containing, in order: account tiles `.pins`, Places,
   labels, Pinned, Today (`S:1259`).
   - `.pins`: 4 equal columns, gap 6, margin-top 12; tiles `aspect-ratio:1`, radius 12
     (`S:107-109`).
   - Section header `.s-h`: padding `14px 6px 5px`, gap 6, then a 1 px rule filling the rest
     (`S:120-122`).
   - `.item`: padding `6px 8px`, radius 9, gap 9 (`S:126-131`).
3. `.side-foot`: Space name (margin-right auto, padding-left 4), Space dots (gap 5), `+`, hide
   button (`S:145-153`, `S:843-847`).

Hidden sidebar and edge peek:

```css
.edge{ position:absolute; left:0; top:0; bottom:0; width:10px; z-index:14; display:none; }
.win.no-side .edge{ display:block; }
.win.no-side.side-peek .side{ visibility:visible; position:absolute; left:8px; top:8px; bottom:8px; width:226px; z-index:15;
  background:var(--f-solid); border-radius:14px; box-shadow:0 20px 40px -16px rgba(0,0,0,.45); animation:slide-r var(--t-move) var(--e-spring); }
```

`S:451-454`

`C`'s sidebar is a `--surface-2` column with a right border, padding `12px 10px`, gap 2, items
`.view` padding `7px 9px` radius `--r-chip` (`C:271-274`, `C:300-306`). `S` wins.

## 5. The card

The card is Post, inset 8 px, with a flap corner and a list and reader side by side.

```css
.card{
  position:relative; display:grid; grid-template-columns:minmax(0,1fr) minmax(0,1.08fr); min-width:0;
  background:var(--surface); border-radius:12px 12px 12px 4px; overflow:hidden;
  box-shadow:0 0 0 1px rgba(0,0,0,.06), 0 10px 30px -14px rgba(0,0,0,.45);
}
.list-col{ display:flex; flex-direction:column; min-width:0; border-right:1px solid var(--line); min-height:0; }
.list-bar{ display:flex; align-items:center; gap:8px; padding:11px 14px; border-bottom:1px solid var(--line-soft); }
.list-bar .spacer{ margin-left:auto; }
.list{ list-style:none; margin:0; padding:8px; overflow-y:auto; flex:1; min-height:0; }
```

`S:156-164`, `S:174`

| Part | Rule | Source |
| --- | --- | --- |
| Columns | list `minmax(0,1fr)`, reader `minmax(0,1.08fr)` | `S:157` |
| Radius | `12px 12px 12px 4px` (the flap corner is bottom-left) | `S:158` |
| List bar | padding 11/14, gap 8, bottom border `--line-soft`; title, spacer, "Sidebar" mini (only when hidden), Compose mini | `S:162-164`, `S:852-857` |
| List | padding 8, scrolls | `S:174` |
| Divider | list column right border 1 px `--line` | `S:161` |

## 6. Rows

A row is a three-column grid: dot, text, tail; about 74 px tall.

```css
.row{
  position:relative; display:grid; grid-template-columns:10px minmax(0,1fr) auto; gap:10px; align-items:start;
  padding:10px 11px; margin-bottom:5px; border-radius:var(--r-card); background:var(--surface);
  border:1px solid transparent; cursor:pointer; user-select:none;
}
.row-dot{ padding-top:6px; }
.dot{ display:block; width:8px; height:8px; border-radius:999px; background:var(--accent); }
.row-main{ min-width:0; }
.row-from{ display:flex; align-items:baseline; gap:7px; min-width:0; }
.row-tail{ display:flex; flex-direction:column; align-items:flex-end; gap:5px; }
```

`S:175-178`, `S:185-189`, `S:196` (transitions and animation omitted; see `05-MOTION.md`)

| Part | Rule | Source |
| --- | --- | --- |
| Height | not declared; about 74 px with three text lines. The heal distance is `offsetHeight + 5` (row plus its bottom margin). `C` names "a 74px row" | `S:1537`, `C:789` |
| Row text | from line (name, `via` mark), subject, snippet, stacked | `S:1279-1280` |
| Tail | time, then a flex line (gap 4) of attachment clip and chips | `S:1281-1283` |
| Star | absolute, left 6, bottom 7, 22x22, icon 14 | `S:299-302` |
| Hover strip | absolute, right 8, vertically centred; padding 3, gap 3, pill; buttons 26x26, icons 14 | `S:312-322` |
| Strip preview (`.fly`) | above its button: `bottom:calc(100% + 7px)`, centred, padding 3/7, radius 6 | `S:442-444` |

`C` differences, `S` wins: `C` first declares the dot column as 26 px then overrides it to 10 px
(`C:362`, `C:1035`); dot padding-top 5 (`C:1036`, `S`: 6); star at left 8 bottom 8 with a 15 px
icon (`C:405-411`); strip gap 4 with 28 px buttons and 15 px icons (`C:423-443`). Candy look
pads rows 13/13 (`C:927`).

## 7. The reader

The reader is a column on `--surface-2`: header, scrolling body, reply bar.

```css
.reader{ display:flex; flex-direction:column; min-width:0; background:var(--surface-2); min-height:0; }
.reader-head{ padding:14px 18px 12px; border-bottom:1px solid var(--line-soft); }
.reader-tools{ display:flex; gap:4px; justify-content:flex-end; margin-bottom:6px; }
.meta{ display:flex; align-items:center; gap:9px; margin-top:9px; }
.reader-body{ padding:16px 18px; overflow-y:auto; flex:1; font-size:14px; color:var(--ink-soft); line-height:1.65; }
.reader-body p + p{ margin-top:11px; }
.reader-body .blocks{ max-width:66ch; display:grid; gap:11px; }
.reply-bar{ display:flex; gap:6px; margin-top:18px; padding-top:12px; border-top:1px solid var(--line-soft); }
```

`S:201-203`, `S:208`, `S:212-213`, `S:462`, `S:718`

| Part | Rule | Source |
| --- | --- | --- |
| Meta avatar | 28x28 circle | `S:209` |
| View switch | pushed right (`margin-left:auto`), padding 2, gap 2, pill | `S:537-538` |
| Empty state | centred; "Nothing open" over mono "pick a thread" | `S:216-217`, `S:1388` |
| Original frame | radius `10px 10px 10px 3px`; iframe height 520 | `S:540-541` |
| Link pill | absolute left 12, bottom 12, `max-width:calc(100% - 24px)`, padding `5px 11px 5px 9px`, gap 7 | `S:431-433` |

`C` reader head padding is `16px 18px 12px` (`C:450`). `S` wins.

## 8. Overlays and placement

Every floating surface is positioned in window coordinates and clamped 8 px inside the window.

### 8.1 Fixed-position overlays

| Overlay | Geometry | Source |
| --- | --- | --- |
| Scrim | `position:absolute; inset:0` over the card | `S:221-222` |
| Peek (sheet) | `inset:36px 12% 36px 12%`, radius `--r-panel` (14), flex column | `S:223-225` |
| Command menu wrap | `inset:0`, grid `place-items:start center`, `padding-top:11%` | `S:230-231` |
| Command menu | `width:min(540px, 88%)`, radius 14; input row padding 12/14, gap 9; token row padding `0 14px 8px`, gap 5; list is an `.fmenu` with `max-height:360px`, no border, no shadow | `S:793-799`, `S:235` |
| Undo toast | `left:50%; bottom:14px`, hidden at `translateX(-50%) translateY(160%)`; padding `5px 5px 5px 15px`, gap 2; pull tab padding `5px 12px 5px 9px`, gap 6 | `S:360-367` |
| Send pill | `left:50%; bottom:14px`, hidden at `translateY(160%)`; padding `6px 6px 6px 12px`, gap 10; ring 20x20 | `S:706-710` |
| Floating composer (legacy) | `right:14px; bottom:14px; width:min(340px, calc(100% - 28px))` | `S:373` |

`S` wrote `.cmdk{ width:min(520px,86%) }` first (`S:232`) and overrides it to
`min(540px, 88%)` with radius 14 (`S:793`); the override is the value. `S`'s `.cmdk-list`
(`max-height:300px`, `S:237`) is not used: the list renders as `.fmenu` (`S:1636`).

`C` values, `S` wins: centre peek `inset:34px 10% 34px 10%` (`C:1044-1048`); full peek `inset:0`
(`C:1049-1051`); command wrap `padding-top:14%`, command menu `min(420px, 88%)`, input padding
11/14, list `max-height:252px` (`C:1059-1073`); toast hidden at `translateY(140%)` (`C:559`);
floating composer `min(360px, calc(100% - 28px))` (`C:503`).

### 8.2 Anchored overlays

Anchored surfaces follow three placement rules, all clamped 8 px inside the window.

| Anchor kind | Rule | Source |
| --- | --- | --- |
| Row hover card (thread) | x = list column right edge + 10; y = row top - 4 | `S:1718` |
| Sidebar hover card (pin, today, account) | x = target right + 10; y = target top - 6 | `S:1719` |
| Other hover cards (sender, time) | x = target left; y = target bottom + 6 | `S:1720` |
| Floating menu (`/`, `@`, pickers) | x = anchor left - 8; y = anchor bottom + 6; if it would cross the bottom margin, flip to anchor top - height - 6 | `S:2060-2065` |
| Selection bubble | centred over the selection; y = selection top - height - 8, at least 8 | `S:2143-2145` |
| Clamp | x in `[8, window width - width - 8]`, y in `[8, window height - height - 8]` | `S:1724`, `S:2064` |

```js
function placeFloat(el, rect){
  const w = $("#win").getBoundingClientRect(); $("#win").appendChild(el);
  let x = rect.left - w.left - 8, y = rect.bottom - w.top + 6;
  if(y + el.offsetHeight > w.height - 8) y = rect.top - w.top - el.offsetHeight - 6;
  el.style.left = clamp(x, 8, w.width - el.offsetWidth - 8) + "px"; el.style.top = clamp(y, 8, w.height - el.offsetHeight - 8) + "px";
}
```

`S:2060-2065`

The design system's placement function takes this rule as `place(anchor, content, bounds, want,
gap) -> Placed` with flip and an 8 px clamp (`P:334`).

### 8.3 Anchored overlay sizes

| Surface | Size and padding | Source |
| --- | --- | --- |
| Hover card | width 300, padding 12/13 | `S:399-400` |
| Hover card, sidebar variant | width 260 | `S:428` |
| Tooltip (`.hc.tip`) | width auto, max-width 260, padding 6/10, radius 9 | `S:426` |
| Floating menu | width 280, max-height 320, padding 5, radius 12; item grid `34px 1fr auto`, gap 9, padding 5/7, radius 8; tile 34x34 radius 8 | `S:665-672` |
| Slim menu | width 220; item grid `22px 1fr auto`, padding 6/8; tile 22x22 | `S:678-680` |
| Selection bubble | padding 3, gap 1, radius 10; buttons height 28, min-width 28, padding `0 7px`, radius 7; separator 1x18 margin `0 3px` | `S:683-689` |

`C`'s toolbar menu is a different shape: `top:calc(100% + 7px); right:0`, min-width 186, radius
`--r-panel`, padding 6, items padding 6/9 radius `--r-btn` (`C:1003-1016`). `S` wins: one menu
shape.

## 9. The composer page

The composer replaces the reader with a page: top bar, scrolling document, foot.

```css
.cpage{ display:flex; flex-direction:column; min-height:0; height:100%; background:var(--surface-2); position:relative; }
.card.focus .cpage{ position:absolute; inset:0; z-index:12; background:var(--surface); }
.card.focus .cpage .c-scroll{ padding-inline:max(24px, calc((100% - 70ch) / 2)); }
.c-top{ display:flex; align-items:center; gap:6px; padding:10px 14px; border-bottom:1px solid var(--line-soft); }
.c-scroll{ flex:1; overflow-y:auto; padding:22px 26px 40px; min-height:0; }
.c-props{ margin:14px 0 18px; display:grid; gap:2px; }
.prop-row{ display:grid; grid-template-columns:92px 1fr; align-items:center; gap:8px; min-height:30px; border-radius:8px; padding:2px 6px; margin-inline:-6px; }
.c-body{ display:block; outline:none; font-size:14.5px; line-height:1.65; color:var(--ink); max-width:66ch; min-height:180px; padding-left:22px; margin-left:-22px; }
.c-body > *{ margin:0 0 .55em; }
.c-hint{ margin-top:18px; font-family:var(--font-data); font-size:10.5px; color:var(--ink-faint); display:flex; gap:12px; flex-wrap:wrap; }
.c-foot{ display:flex; align-items:center; gap:8px; padding:10px 14px; border-top:1px solid var(--line-soft); background:var(--surface-2); flex-wrap:wrap; }
.c-warn{ flex-basis:100%; display:flex; align-items:center; gap:8px; padding:8px 10px; border-radius:var(--r-btn); }
```

`S:567-568` (animation omitted), `S:573-575`, `S:580`, `S:583-584`, `S:661`, `S:694-695`, `S:728-729`

| Part | Rule | Source |
| --- | --- | --- |
| Top bar contents | state pip, spacer, Focus, Keep, Discard tools | `S:1948-1951` |
| Foot contents | "Plain text" mini, spacer, Send button; a warning takes a full line above | `S:1958`, `S:695` |
| Focus mode | page covers the card; sidebar hidden | `S:573-574`, `S:1997` |
| Inline reply | margin-top 16, bordered panel; props padding `8px 14px 4px`; body padding `6px 14px 10px 50px` | `S:719-722` |
| Object handle | absolute left -24, top 6, 20x24 | `S:747` |

`S:723` rounds the inline reply foot with `0 0 var(--r-panel) var(--r-panel)`. `S`'s older
block editor rules (`.blk`, `.gutter` at left -46, `.t-*`) are unused (`S:607-662`, `P:1489`).

## 10. Radii

Radius scales with element size, and the card family keeps the flap corner.

| Name | Value | Used on | Source |
| --- | --- | --- | --- |
| Window | 18 | `.win` | `S:80` |
| `--r-panel` | 14 | peek, command menu, hover card, composer, editor, side peek | `S:16`, `S:223`, `S:454`, `S:793` |
| `--r-card` | `12px 12px 12px 4px` | card, rows | `S:16`, `S:158`, `S:177` |
| Menu | 12 | `.fmenu` | `S:666` |
| Tile | 12 | account tiles, editor field | `S:109`, `S:252` |
| Field | 10 | `.inp`, command pill, bubble, code and table blocks | `S:94`, `S:684`, `S:781` |
| `--r-btn` | 9 | mini, button, tool, command items | `S:16` |
| Item | 9 | sidebar `.item`, tooltip | `S:128`, `S:426` |
| Menu item, prop row, foot button | 8 | `.fmenu .it`, `.prop-row`, `.foot-btn` | `S:151`, `S:584`, `S:669` |
| Bubble button | 7 | `.bubble button` | `S:685` |
| `--r-chip` | `6px 6px 6px 2px` | chip | `S:16`, `S:198` |
| Small | 6 | fly, gutter, `.plink`, `.pval`, samples | `S:443`, `S:601`, `S:603` |
| Kbd, favicon | 5 | `kbd`, `.item .fav`, provider mark on tile | `S:68`, `S:138`, `S:554` |
| Tiny | 4, 3 | focus ring outline, provider mark (4), inline code (4), in-row provider (3) | `S:55`, `S:552`, `S:556`, `S:659` |
| Media corner | `10px 10px 10px 3px` | images, code blocks, attachments, original frame | `S:471`, `S:489`, `S:527`, `S:540` |
| `--r-pill` | 999 | toast, strip, segmented control, dots, avatars | `S:16` |

The design system names four of the unnamed ones: `--r-field 10`, `--r-menu 12`, `--r-item 9`,
`--r-tile 12` (`P:296`).

`C`'s looks change the four named radii: Riso 18/14/22/999, Tide 14/11/18/999, Candy 20/14/24/999
for card/btn/panel/chip (`C:57`, `C:74`, `C:790`). Those belong to `07-LOOKS.md`.

## 11. Fixed sizes

These sizes are fixed in `S` and must not be derived from padding.

| Element | Size | Source |
| --- | --- | --- |
| Glyph (`.ic`) default | 16x16 | `S:58` |
| Glyph in mini, button, star, strip, c-warn, flag, prop key | 14x14 | `S:173`, `S:302`, `S:322`, `S:391`, `S:422`, `S:588`, `S:697` |
| Glyph in close x, toast tab, link pill | 13x13 | `S:143`, `S:370`, `S:437` |
| Glyph in stop remove, pchip x, clip | 11x11 | `S:264`, `S:544`, `S:596` |
| Glyph in blocked image, image picker | 18x18 | `S:493`, `S:642`, `S:756` |
| Glyph in menu tile | 17x17 | `S:673` |
| Unread dot | 8x8 | `S:186` |
| Seal dot | 5x5, at left -7 | `S:345` |
| Space dot | 22x22, 2 px border | `S:147` |
| Foot button | 24x24 | `S:151` |
| Reader tool | 28x26 | `S:204` |
| Strip button | 26x26 | `S:316` |
| Star hit area | 22x22 | `S:299` |
| Tile avatar | 26x26 (account tile 28x28) | `S:115`, `S:548` |
| Meta avatar | 28x28 | `S:209` |
| Hover card avatar | 22x22 (person 34x34) | `S:409`, `S:417` |
| Composer chip avatar | 18x18 | `S:594` |
| Favicon in sidebar item | 16x16 | `S:138` |
| Provider mark | 13x13; on tile 14x14 at right 7 bottom 7; in row 11x11 | `S:552-556` |
| Event calendar column | 58 | `S:510` |
| Attachment grid cell | `minmax(118px, 1fr)`, gap 8, thumbnail 4:3 | `S:526`, `S:530` |
| Editor field | height 176; canvas 540x352; handle 22x22 with 3 px border | `S:252`, `S:255`, `S:870` |

## 12. Z-order

Inside a window, z-index values are fixed; overlays are children of the window.

| z | Layer | Source |
| --- | --- | --- |
| -2 | frame `.layer` (a, b) | `S:86` |
| -1 | `.grain` | `S:87` |
| 7 | link pill | `S:431` |
| 8 | undo toast | `S:360` |
| 9 | floating composer, send pill | `S:373`, `S:706` |
| 10 | scrim | `S:221` |
| 11 | peek | `S:223` |
| 12 | focus-mode composer page | `S:573` |
| 14 | left edge strip (sidebar hidden) | `S:451` |
| 15 | side peek | `S:453` |
| 20 | command menu wrap | `S:230` |
| 30 | hover card | `S:399` |
| 40 | floating menu, `zZ` floater | `S:334`, `S:665` |
| 41 | selection bubble | `S:683` |

The design system names the scale `--z-raise 1 .. --z-drag 50` (`P:302`); `C`'s drag ghost is
z 50 (`C:580`). Intermediate token names are not specified.

On the desktop, shell surfaces are ordered by Wayland layer, not z-index: wallpaper on
Background; bar and dock on Top; launcher catcher and panel on Overlay (`P:574-588`).

## 13. Shell surface layout

The shell surfaces reuse the window's geometry language; only the values below are settled.

### 13.1 Bar

| Property | Value | Status | Source |
| --- | --- | --- | --- |
| Instance | one per output | settled | `P:574` |
| Layer, anchor | Top; `TOP \| LEFT \| RIGHT` | settled | `P:574` |
| Exclusive zone | `Reserve(h)`, h = bar height | settled | `P:574` |
| Keyboard | None | settled | `P:574` |
| Blur | whole surface | settled | `P:575` |
| Height | 32 | provisional: the only number in the plan is the spike's acceptance test (maximized window height == output height - 32); the plan calls it "height token" | `P:526`, `P:574` |
| Reference | macOS menu bar 24 pt since Big Sur (22 before, 37 on notched laptops); status items at most 22 pt, icons 16 pt | reference only | `P:1468-1469` |
| Left group | app name, workspace indicator (drag to reorder) | settled | `P:575` |
| Right group | tray, volume, network, battery, clock | settled | `P:575-576` |
| Menus | design-system Menu in an xdg popup anchored to its button, fit to content, seat grab | settled | `P:576-577` |
| Horizontal padding, item gap, item height, glyph size | not specified | open | |

### 13.2 Dock

| Property | Value | Status | Source |
| --- | --- | --- | --- |
| Position | bottom, centred | settled | `P:579`, `P:784` |
| Layer | Top, anchor BOTTOM | settled | `P:579-580` |
| Base icon size | 48 | settled | `P:784` |
| Maximum magnified size | 96 (magnification on by default) | settled | `P:784-785` |
| Magnification curve | Plank parabola: `offset = min(\|cursor - center\|, zoomIconSize); p = offset / zoomIconSize; zoom = 1 + (1 - p²)(zoomPercent × progress - 1)`; influence ±1 magnified icon width; progress 0 to 1 on enter and exit | settled curve, constants in `10-BEHAVIOUR-dock.md` | `P:785`, `P:1389-1391` |
| Neighbours | slide apart; the dock widens; icons rise above the background, baseline fixed | settled | `P:785`, `P:1392-1393` |
| Surface height | `base × max magnification + padding` (= 96 + padding) | settled formula, padding open | `P:580` |
| Exclusive zone | `Reserve(base + margin)` | settled formula, margin open | `P:580` |
| Gutters | 8, matching the 8 px card inset (`S:81`) and the 8 px floating clamp (`S:1724`) | settled in the orchestrator's brief for this doc | brief |
| Auto-hide | off by default; setting: 0.2 s delay, ~0.5 s slide | settled | `P:784-786` |
| Input region | element `dock-hit` | settled | `P:580` |
| Blur region | element `dock-pill`, follows the pill during magnification | settled | `P:581`, `P:493-494` |
| Contents | pinned items then running items grouped by app; indicator dots; LauncherEntry badge and progress ring | settled | `P:581-582` |
| Which gaps the 8 px gutter covers (pill to screen edge, pill padding, icon spacing) | not specified | open | |
| Pill radius, indicator dot size and offset, badge geometry, separator | not specified; macOS reference: running dot ~4-5 pt below icon, red capsule badge top-right, thin vertical separator | open | `P:1410-1411` |

### 13.3 Launcher

| Property | Value | Status | Source |
| --- | --- | --- | --- |
| Surfaces | transparent full-screen catcher (Overlay, anchored on all edges, painted once, never animated) plus a centred panel | settled | `P:586-587` |
| Panel layer | Overlay | settled | `P:587` |
| Panel size | fixed ~680x460 | settled (approximate) | `P:587` |
| Keyboard | Exclusive | settled | `P:588` |
| Blur | element `panel` | settled | `P:588` |
| Vertical position | not specified; the prototype's command menu sits at `padding-top:11%` of its container (`S:230`) | open | |
| Internal layout | not specified; the prototype's command menu layout is section 8.1 | open | |

### 13.4 Other surfaces

| Surface | Settled | Open |
| --- | --- | --- |
| Wallpaper | per output, Background layer, exclusive zone Ignore, empty input region (`P:577`) | none for layout |
| Control center | uses `AppearancePicker` (`P:355`) | position, width, anchor, layer: not specified |
| Notifications | none | position, width, stacking, gap: not specified (macOS reference: banners top-right, `P:1476`) |
| OSD | a `Material::Osd` exists (`P:315`) | position, size: not specified |
| Widgets | a `Material::Widget` exists (`P:315`) | grid, size classes, placement: not specified |
| Popovers from bar and dock | xdg popup, anchor element, fit content (`P:576-577`); design-system placement flips and clamps 8 px (`P:334`) | gap between anchor and popover: not specified (`S` uses 6 below an anchor, 10 beside one, `S:1718-1720`) |

## 14. Responsive behaviour

`S`'s window is fixed-size and scrolls horizontally inside a scroller below 980 px (`S:71`,
`S:80`); its only media query rearranges the prototype page, not the window (`S:801-806`).

`C` collapses its shell (`C:751-767`):

```css
@media (max-width:1000px){
  .shell{ grid-template-columns:186px minmax(0,1fr); height:min(78vh,660px); }
  .reader{ position:absolute; inset:0 0 0 186px; z-index:5; border-left:1px solid var(--line);
    transform:translateX(102%); transition:transform var(--t-big) var(--e-spring); }
  .reader.open{ transform:translateX(0); }
}
@media (max-width:660px){
  .shell{ grid-template-columns:minmax(0,1fr); height:auto; min-height:0; }
  .side{ flex-direction:row; overflow-x:auto; border-right:0; border-bottom:1px solid var(--line);
    align-items:center; gap:6px; padding:9px; }
  .side .acct, .side .side-h, .side-foot{ display:none; }
  .view{ width:auto; flex:none; }
  .list{ max-height:430px; }
  .reader{ inset:0; }
}
```

`C:751-767` (the 660 block's `.hero` and `.pip-nook` lines omitted)

Whether real app windows below 980 px wide follow `C`'s collapse is not specified.

## Open decisions

1. **Bar height.** The plan calls it a "height token" (`P:574`); the only number is the spike's
   32 px acceptance check (`P:526`). macOS uses 24 pt (`P:1468`). Not settled.
2. **Bar internals**: horizontal padding, item gap, item height, glyph size, tray icon size.
3. **Dock gutter scope**: which gaps the 8 px gutter covers; dock padding in the height formula;
   the margin in `Reserve(base + margin)`.
4. **Dock details**: pill radius, indicator dot size and offset, badge and progress ring geometry,
   separator, hover label position and size.
5. **Launcher**: vertical position of the panel; its internal layout; whether it is the command
   menu component at 680 wide (the prototype's command menu is `min(540px, 88%)`, `S:793`).
6. **Control center, notifications, OSD, widgets**: every layout value.
7. **Anchor gap for shell popovers**: `S` uses 6 below and 10 beside (`S:1718-1720`); the
   design-system `place()` takes a `gap` parameter with no default (`P:334`).
8. **App window size.** `S`'s window is 980 minimum and 680 high as a prototype; real app windows
   are resizable and their minimum size is not specified. Whether narrow windows collapse as in
   `C:751-767` is not specified.
9. **Row height** is emergent (about 74 px), not a token; whether the list uses a fixed pitch
   (the design system's `use_roster(keys, pitch: RowPitch)`, `P:332`) is not specified.
10. **Z-token names** between `--z-raise 1` and `--z-drag 50` are not specified (`P:302`).
11. **Sidebar width** 232 (`S`) versus 206 (`C`), side-peek width 226 (`S:453`): `S` wins; whether
    the sidebar is user-resizable is not specified.
12. **Legacy rules in `S`** (`.composer` floating composer, `.blk`, `.gutter`, `.t-*`) are unused
    (`P:1489`); whether any survives is not specified.

## Sources

- `S` `~/mailo-design/mailo-spaces.html`: tokens `S:7-24`; window and sidebar `S:73-153`; card,
  list, rows, reader `S:155-218`; peek and command menu `S:220-241`; motion-bearing layout
  `S:289-454`; parsed bodies `S:456-560`; composer `S:564-775`; one field one menu `S:777-799`;
  markup `S:833-862`; placement JS `S:1715-1726`, `S:2060-2065`, `S:2143-2145`; heal distance
  `S:1537`.
- `C` `~/mailo-design/mailo-charm.html`: shell `C:262-268`; sidebar `C:271-325`; list and rows
  `C:341-443`; reader `C:445-499`; composer `C:502-531`; toast `C:557-576`; responsive
  `C:751-767`; Candy padding `C:926-931`; menus, peek, command menu `C:1003-1081`.
- `P` `~/.claude/plans/vast-toasting-peach.md`: logical px `P:459`; token names `P:296-304`;
  placement `P:334`; region and popup anchoring `P:493-494`; spike bar check `P:526`; shell
  surfaces `P:574-592`; dock decision `P:784-786`; Appendix A1 `P:919-958`; macOS dock and menu
  bar numbers `P:1384-1412`, `P:1468-1471`; A8 `P:1486-1491`.
- Brief: the orchestrator's brief for this file settles the 8 px dock gutter.
