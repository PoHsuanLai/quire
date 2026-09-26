# 23 Widgets: flat, bright, measured; the battery, the world clock, the calendar

Status legend as in `13-BEHAVIOUR-menus-windows.md`: **settled** = decided with the user;
**proposed** = chosen here, rendered in the gallery's "Widget looks" and "Widget reference"
pages, the user judges. Confidence of a reference value: **H** primary source (the vendor's own
guidelines or docs), **M** a reliable secondary source (a review, a teardown, a design write-up
that measured the product), **L** observed from screenshots and memory of the shipping product,
not measured, **UNKNOWN** nothing published; a value in section 1.1 is **measured** (M) from a
real screenshot with the method named. The reference platforms are named here as sources only;
code, class names, tokens and asset names never name them.

## 1. What this governs

The small desktop widgets' look: the style every widget is drawn in (section 2, "Flat, bright,
measured"), how our widgets apply it (section 3), the battery, the world clock and the card
(section 4), and the research and specification for the calendar widget (section 5, queued:
this pass does not restyle `MonthGrid`). The frame's footprint stays `04-COMPONENTS.md` section
40; the surface row stays `20-SURFACES.md` section 1.14; the material stays `03-COLOR.md`
section 17. The reference survey behind all of it is section 8; the measurements behind the
drawing are section 1.1.

**Why this file exists.** The user's verdicts, in order (2026-09-26):
1. First widgets (a flat stroked ring, a plain paper dial): "too ugly"; "this battery circle
   thing looks nothing different to other old looking linux distros".
2. Second pass (depth: the ring in a well round a boss, a glossy cell, bezel and sky dials, a lit
   card): "all widgets looks bad"; "for battery specifically drop the circle, use a battery icon
   and a number to show, like mac".
3. Third pass: Neumorphism & Soft UI (soft extruded and inset shapes in the plate's colour).
   Rejected.
4. The agreed diagnosis for the fourth pass (this one): the reference widgets are **flat and
   bright**. What makes them right is colour (a bright green ring, white day dials, black night
   dials, an orange seconds hand), no header row on the Batteries and Clock widgets, one rounded
   grotesque with small quiet labels and never a monospace face, crisp high-contrast detail at
   small size, and, from Arc, the plate may take the Space's colour. So this pass redraws the
   widgets **to match the reference's own widgets, measured side by side from real
   screenshots** (section 1.1), and the gallery's "Widget reference" page renders ours at the
   reference crops' size for the comparison.

### 1.1 Measured references (2026-09-26)

Sources (screenshots, reference-only, never committed): **R-bs** the Batteries widget small with
one device, macOS 14 desktop, 2x (Intego, "How to Use Desktop Widgets on macOS Sonoma",
`widgets3.png`); **R-bg** Batteries small with four places, 2x (same article, `widgets5.png`);
**R-bm** Batteries medium with a low and a charging device, iOS today view, about 2x of a 169 pt
card (iDownloadBlog, "How to see AirPods battery percentage", `airpods-and-case-battery-in-widget.jpg`);
**R-bmm** Batteries medium on the Mac, the widget gallery's preview at 1.25 px/pt (Intego,
`widgets4.png`); **R-cs** Clock small, analog, day, 2x (Intego, `widgets3.png`); **R-cm** World
Clock medium, three day dials and one night dial, macOS 15, native 2x (512 Pixels' Aqua
screenshot library, `15-Sequoia-Notification-Center.png`); **R-ca** the Clock app's large day and
night dials (512 Pixels, `14-Sonoma-Clock.png`); **R-kl** Calendar small and medium (512 Pixels
`15-Sequoia-Notification-Center.png`; Macworld, "How to add widgets to the macOS Sonoma Desktop").
Measured with Pillow (runs of pixels of one colour along rows, columns and rays from a dial's
centre); every length below is in points of a 164 pt small card (a 344 x 164 medium) unless it
is a ratio.

| # | Quantity | Value | How measured | Conf. |
| --- | --- | --- | --- | --- |
| M1 | Small card | 164 x 164 (330 px at 2x) | R-bs, R-cs, R-bg: the plate's bounds against the wallpaper's saturation | M |
| M2 | Medium card | 344 x 163 (688 x 325 px at 2x) | R-cm: the dark plate's bounds along a row and a column | M |
| M3 | Card corner | about 19-20 (.12 of the small card), continuous | R-cm: the inset of the first plate pixel on rows 0-40 px from the top | M |
| M4 | Small inset (ring, grid) | 12 at the top and the left | R-bs, R-bg: the ring's bounding box to the card's | M |
| M5 | Ring diameter | 64 (.39 of the small card; 63 in R-bmm, .40 of the card's height in R-bm) | R-bs, R-bg: the green run's extent | M |
| M6 | Ring stroke | .093 of the ring's diameter (6 of 64); .088 in R-bm, .10 in R-bmm | R-bs: the green run's width along the centre row | M |
| M7 | Ring fill | `#27cd41` (39,205,65) | R-bs: most common green pixel | M |
| M8 | Ring track | the plate darkened by about 12 % black, not a hue: (181,136,126) over a plate of (206,156,141) | R-bs: the ring's pixels outside the arc against the plate beside it; R-bm the same on a yellow plate | M |
| M9 | Low | red arc, `(233,78,60)` on R-bm's plate, at 13 %; no separate critical colour seen | R-bm | M |
| M10 | Arc | clockwise from twelve, round caps (the cap shows before twelve) | R-bm: 83 % ends at 300 degrees; green at 355 degrees is the start cap | M |
| M11 | Charging | a bolt .16 of the ring tall, .10 wide, centred on the stroke at twelve (its top 4 % above the ring), in a gap in the ring of about 17 degrees between the caps | R-bm: the bolt's dark pixels; non-green angles along the stroke | M |
| M12 | Glyph | the device's filled symbol, .50 of the ring wide (32 x 18 laptop), black, centred | R-bs | M |
| M13 | Small, one device | the ring at the top left; the percentage at the bottom left: cap height 33 (about 46 pt type), stems .12-.14 of the cap (Regular to Medium, not heavy), baseline 15.5 above the card's bottom | R-bs: dark rows and columns of "93%" | M |
| M14 | Small, several | a 2 x 2 grid of 64 rings, 12 inset, 11.5 apart; no numbers; an empty place is a track ring | R-bg | M |
| M15 | Medium | four 63-64 rings on an 80 pitch from 20 in; the percentage under each, cap 12.8-14.6 (about 18-20 pt Regular), 17.6-23 below the ring; the block centred vertically | R-bmm, R-bm | M |
| M16 | Small clock dial | .90 of the card (148 of 164), 8 inset; the face white, the plate around it light grey | R-cs: the white run's extent | M |
| M17 | Small clock ticks | 60, all from .885 to .96 of the radius (5.5 long), 1.5 wide; the hour ticks ink `#1c1c1e`, the minute ticks grey (179 on white: ink at .3) | R-cs: dark runs along rays every 6 degrees | M |
| M18 | Small clock numerals | 12, cap .163 of the radius (.073 of the card), centred on .73 of the radius, heavy (stem .2 of the cap) | R-cs: dark rows and columns round 12 and 3 | M |
| M19 | Medium clock dial | 62.5 across, 29.5 below the card's top, 19 in, on an 81 pitch; numerals only, no ticks; cap .096 of the diameter on .82 of the radius | R-cm | M |
| M20 | Hands | hour to .56 of the radius, minute to .9; a thin neck (.017 of the diameter) for the first .14 of the radius, then .034-.04 wide to a round tip; ink by day, white by night | R-cs, R-cm: dark runs along the hand and across it | M |
| M21 | Seconds hand | `#f99101`-`#f29a37` (system orange), about 1 wide, from .2 of the radius behind the hub to .96; a hub of ink (.054 of the radius) under an orange ring with a white pin | R-cs, R-cm: orange pixels; a row through the centre | M |
| M22 | Faces | day `#ffffff`, ink `#1c1c1e`; night `#343436` with white numerals and hands (R-ca's large night dial is black) | R-cm, R-ca: most common colours inside the discs | M |
| M23 | World Clock card | `#1c1c1e`, dark in the light scheme too | R-cm, and the same widget in Apple's newsroom images | M |
| M24 | Clock labels | the city bold white, cap about 7.5 (11 pt type); the day and the offset bold grey (`#5b5b5e` on the dark card), the same size, on two lines; 6.5 from the dial to the city | R-cm: white and grey text rows | M |
| M25 | Header row | none on Batteries or Clock: the content fills the card | every source | M |
| M26 | Card see-through, blurred (R-bm) | the card is `C = a T + (1 - a) sat(G_s * W)`: a Gaussian blur of sigma **22 pt** (45 px at 2x; the best fit lies between 40 and 50 px, rms 4.4 levels of 255; with no saturation gain 4.7 at 25 pt, 5.9 at 35 pt, 8.0 at 4 pt and 8.6 unblurred) | R-bm: the same wallpaper W stands bare in the first of the three screenshots and under the large card in the third, so W under the card is known; 908 card-plate samples clear of text and rules, W blurred at 13 radii and fitted by least squares | M |
| M27 | Tint alpha, blurred (R-bm) | **.48** (.47-.51 across the best radii) | R-bm, the fit in M26 | M |
| M28 | Tint colour, blurred (R-bm) | a near-white, `rgb(247,248,248)`; the card over black would be 119 grey | R-bm, the fit in M26: `u = a T` = (119,120,120) | M |
| M29 | Saturation | **x1.8** of what shows through (1.4-2.0 fit alike; x1.0 fits 6 % worse, rms 4.70 against 4.43); the desktop card R-bs shows none (its chroma is `1 - a` of the wallpaper's to within 6 %) | R-bm: `sat` in M26 scales the blurred wallpaper's distance from its luma; R-bs: the card's chroma against the wallpaper's beside it | L |
| M30 | Card see-through, desktop (R-bs, R-bg) | a straight mix, **no blur** (the wallpaper's band edges cross the card in 1-2 px at 2x, as sharp as outside), alpha **.65** of a mid grey `rgb(176,172,171)` (the card over black would be 115 grey); fit rms 1.0 | R-bs, R-bg (same session, the same pixels): three flat band pairs (orange, deep red, green) inside against the wallpaper beside the card, a shared alpha by least squares | M |
| M31 | World Clock card, translucent? | **opaque**: every plate pixel is `#1c1c1e` exactly over a forest backdrop that varies from (45,47,40) to (176,164,126) beside it; with a window in front (the desktop's background mode) the same widget joins the shared translucent look | R-cm: the plate's pixels along two rows and a column; Intego `widgets7.png` for the background mode | M |
| M32 | Outer rim | a 1 pt dark rim (2 px at 2x), the wallpaper times .88 (black at about .12); the blurred phone card R-bm has none | R-bs: rows and columns across the four edges | M |
| M33 | Inner rim | lighter, not darker: the first pixels inside are 4-10 levels over the plate, fading over about 6 pt; no top highlight stronger than the sides | R-bs: the same rows and columns | M |
| M34 | Shadow | short and soft, deeper below: about `0 2pt 8pt` (sigma 4 pt) black .12-.15 (the wallpaper darkened .10 at the bottom edge, .05 at the top and .04 at the sides, fading over 8 pt); R-bm has none | R-bs: the wallpaper's darkening outside each edge against 20 pt out | M |

**How M26-M34 were measured (2026-09-26, the vibrancy pass).** A card over a blur is modelled as
`C = a T + (1 - a) sat_s(G_sigma * W)`: `W` the wallpaper, `G_sigma * W` its Gaussian blur (Pillow
`GaussianBlur`, whose radius is sigma), `sat_s` a saturation gain about Rec. 709 luma, `T` the tint,
`a` its alpha. For each sigma and s on a grid, `a` and `u = a T` are the least-squares solution of
the three channels' linear equations over the samples; the best (sigma, s) is the lowest residual.
R-bm is the one source where the wallpaper under a card is visible elsewhere (its three phone
screenshots share a wallpaper at the same offset, to 1.6 levels of JPEG noise); R-bs has no clean
wallpaper, so its bands, flat inside and nearly flat outside, are paired across the card's edge.
The two sources disagree on blur (22 pt against none) and agree on the tone the tint gives: the
card over black would be 115-119 grey, the card lets .35-.52 of the wallpaper through. The
desktop screenshots with the desktop in front (Aqua library, `14-Sonoma-Desktop-Widgets.png`)
show the cards opaque; the translucent look is the desktop's background mode and its widget
editing mode.

**What quire paints (the conflict, resolved 2026-09-26, "relax the contrast then").** The
legibility gate (`material-legible-over-black-and-white`, `tests/legibility.rs`) needs the
card's `--ink` at 4.5:1 over a black and a white backdrop, and the Space-tinted card
(`CardTint::Space`) shares the tint's alpha; the lowest light alpha that passes both at 4.5:1 is
.60 (at .58 preset 2's stop `#f3dcd8` falls to 4.43:1 over black), which the vibrancy pass
(2026-09-26) shipped, at the cost of matching the reference: .48 over a near-white
(3.77:1 over black, 3.86 once boosted) fails 4.5:1. The user's decision relaxes the gate
instead of the fit: the card may be as see-through as the reference measures, and the widget's
own text — the hero and figure numerals, bold city names — is large or bold, where the
accessibility guideline for large text is 3:1, not 4.5:1 (design/03-COLOR.md section 17). So the
light tint takes the measured colour and alpha exactly, `rgb(247,248,248)` at **.48**, 3.86:1
over black and 16.70:1 over white once boosted, clearing 3:1 with margin. The dark scheme was
not measured against the reference, so it is raised only as far as the relaxed 3:1 floor needs
across every gate `tests/legibility.rs` runs: the flat dark tint alone clears 3:1 at .53
(3.09:1 over white), but the Space-tinted card's darkest preset stop (preset 2's `#291c1a`) needs
.55 to clear it with margin (3.12:1 over white); dark keeps .55 of the dark paper. Some widget
text sits below the WCAG large-text size even so; section 4.3 lists it. The x1.8 saturation
(M29) is the compositor's to add (section 4.3).

## 2. Flat, bright, measured

The widget style's name is **Flat, bright, measured** (proposed 2026-09-26, the fourth pass).
"Neumorphism & Soft UI" names the icon plates only (`08-ICONS.md` section 2).

1. **Flat.** A shape is a solid fill or a solid stroke of one colour on the plate: no shadows
   inside the card, no insets, no bevels, no gloss, no gradients. The card itself is the
   `Widget` material's plate (`03-COLOR.md` 17), optionally tinted by the Space (4.3).
2. **Bright.** Meaning is carried by a few saturated colours at full strength, measured from
   the reference (section 1.1): the battery's green and red, the white day dial and the dark
   night dial, the orange seconds hand. Tracks and minor ticks are the ink or the plate at low
   alpha, never a second hue.
3. **Measured.** Every size and ratio comes from section 1.1: ring to card, stroke to ring,
   numerals to dial, hands to radius. A new widget is measured from its reference the same way
   before it is drawn.
4. **No header on a glanceable widget.** The Batteries and Clock widgets have no title row;
   their content fills the card at the measured inset. The frame's `title` stays for widgets that
   need one.
5. **One rounded grotesque.** Numerals in the display face (`--font-display`, Bricolage
   Grotesque): the clock's at its heaviest (800); a battery's percentage at its lightest (500),
   because the reference's percentage is Regular to Medium (M13). Labels in the UI face
   (`--font-ui`, Karla), small and bold. Never the data face in a widget.
6. **Crisp at small size.** Fine ticks and thin hands at the measured widths, high contrast
   against the face; hands end in round caps; numerals are text, not paths.

### 2.1 Tokens (`ds::tokens::WidgetPaint`; proposed 2026-09-26)

| Token | Light | Dark | Role | From |
| --- | --- | --- | --- | --- |
| `--battery-fill` | `#28cd41` | `#32d74b` | the ring's arc, healthy or charging | M7 |
| `--battery-low` | `#ff3b30` | `#ff453a` | the arc at a fifth or less (low and critical) | M9 |
| `--battery-track` | `rgba(0,0,0,.12)` | `rgba(255,255,255,.14)` | the ring's full circle: the plate darkened (lightened on dark) | M8 |
| `--clock-face-day` | `#ffffff` | same | a day dial | M22 |
| `--clock-face-night` | `#343436` | same | a night dial | M22 |
| `--clock-ink-day` | `#1c1c1e` | same | numerals, ticks, hands on a day dial | M22 |
| `--clock-ink-night` | `#ffffff` | same | the same on a night dial | M22 |
| `--clock-seconds` | `#ff9500` | `#ff9f0a` | the seconds hand and its ring | M21 |

Sizes (`02-TYPE.md`): `--fs-dial` 9 and `--fs-dial-large` 18 (the numerals, M18, M19),
`--fs-widget-figure` 20 (a battery's percentage in a row, M15), `--fs-widget-hero` 47 (a small
battery's percentage, M13), each set so the display face's cap (.66 em) matches the measured cap.
The third pass's `--soft-*` tokens are removed.

## 3. Our widget language

### 3.1 The rule

**Match the reference's widget, measured.** A widget is its content on the plate at the
measured inset, drawn flat in the section 2.1 colours, with no header. Where our parts differ
from the reference's they differ only in face (Bricolage and Karla for the system face) and in
glyph (our icon set for the device symbols).

### 3.2 Type

Numerals are the display face: dial numerals 800 at `--fs-dial` or `--fs-dial-large`; a
battery's percentage 500 at `--fs-widget-figure` in a row and `--fs-widget-hero` alone; a
digital time 800, tabular, tracked -.02em. Labels are the UI face 700 at `--fs-small`: the city
`--ink`, its day and offset `--ink-faint` (M24). No widget uses `--font-data`.

### 3.3 Composition

- **Small**: the content fills the card at the frame's 12 inset (M4): one ring at the top left
  and the percentage at the bottom left, or four rings in a 2 x 2 grid, or one large dial 8 in.
- **Medium**: a row of four across the card, centred vertically: rings with the percentage 18
  under each, or 62 dials with the city, day and offset under each.

## 4. Specification per widget

### 4.1 Battery (`BatteryLevel`; `LevelRing` is its old name)

`BatteryLevel { level: Fraction, mark: RingMark::{Plain, Charging}, label: Text, children }`,
every prop kept (`LevelRing` stays an alias). Markup: `div.ds-battery[data-tone][data-mark]
[role=progressbar][aria-valuenow]`, 64 x 64, holding `svg.ds-battery-track`, `svg.ds-battery-arc`
(absent at 0), `span.ds-battery-device` around `children` when given, and `svg.ds-battery-bolt`
while charging.

| Part | Drawing | From |
| --- | --- | --- |
| Track | a full circle, stroke .093 of the ring (9.3 in the 100 box), round caps, `--battery-track`; while charging, the circle less the bolt's gap | M6, M8 |
| Arc | clockwise from twelve as far as the level, the same stroke, round caps, `--battery-fill`; `--battery-low` at a fifth or less (`data-tone` `low` at 20 % or less, `critical` at 10 % or less, both red); charging is never low; the path is computed in Rust (`battery_ring.rs`), no transition (O-20) | M7, M9, M10 |
| Device | the caller's glyph centred at .47 of the ring, `--ink` | M12 |
| Bolt | while charging: a bolt .16 of the ring tall and .10 wide, `--ink`, centred on the stroke at twelve; the track and the arc leave a 29 degree gap between their ends' centres (17 between the caps) and the arc fills the rest | M11 |

**Composition** (the shell composes, quire provides the ring):
- **Small, one device**: the ring at the top left; the percentage at the bottom left in the
  display face 500 at `--fs-widget-hero`, its baseline about 15 above the card's bottom.
- **Small, several**: a 2 x 2 grid of rings, no numbers; an empty place a ring at 0.
- **Medium**: four rings across from 20 in, the percentage 18 under each in the display face 500
  at `--fs-widget-figure`, the block centred vertically.

Motion: the ring bumps once on a new percentage (`use_bump_on`).

### 4.2 World clock (`ClockFace`)

`ClockFace { time, phase: DayPhase, look: ClockLook::{Analog, Digital}, label }`: every prop
kept. Markup: `div.ds-clock[data-look][data-phase]` holding `div.ds-clock-dial` (analog) or
`span.ds-clock-digits` (digital), then `span.ds-clock-label` around `span.ds-clock-city`.

| Part | Analog | From |
| --- | --- | --- |
| Dial | a disc, 62 across in a row; in a Small frame the card less 8 at each side (148), the city dropped; `--clock-face-day` or `--clock-face-night` | M16, M19, M22 |
| Ticks | `svg.ds-clock-ticks`: 60 from .885 to .96 of the radius, 1 unit wide, every fifth at full ink and the rest at .3; drawn only on the large dial | M17 |
| Numerals | `div.ds-clock-numerals` of twelve `span.ds-clock-numeral`, the display face 800, centred on .82 of the radius (`--fs-dial`) or .73 on the large dial (`--fs-dial-large`) | M18, M19 |
| Hands | `svg.ds-clock-hands`: the hour hand to .56 of the radius and the minute hand to .9, each a 1.7-unit neck to .14 then 3.6 units wide, round caps; a hub of 2.7; `--clock-ink-day` or `--clock-ink-night` | M20 |
| Seconds | when shown, `svg.ds-clock-second`: 1.2 units from .2 behind the hub to .96, and a ring of 1.8 at the hub, `--clock-seconds`; `svg.ds-clock-pin`, a .9 pin of the face's colour | M21 |

**Digital** (the notification center's tile; the reference's desktop widget has no digital
form): the time in the display face 800, tabular, at `--fs-widget-hero` in a Small frame and
`--fs-subject` elsewhere, bumping on each new minute; the city with the phase's mark before it
(the sun a disc in `--warn`, the moon a crescent in `--ink-faint`).

**Composition**: Small, the large dial alone. Medium, four dials across, each with the city
(UI face 700 `--fs-small`, `--ink`) and, from the shell, its day and offset on two lines in the
same face in `--ink-faint`. Day and night: the phase the shell computes (sunrise and sunset
where it knows the zone, else 06:00-18:00).

### 4.3 The frame (`WidgetFrame`)

`WidgetFrame { size, host, tint: CardTint::{Material, Space}, title, id, children }`: `tint`
is new, default `Material`. The desktop card is the `Widget` material's plate, corner
`--m-radius` 20 (M3), padding 12 (M4); the tile is unchanged (12 padding, `--r-tile`).
`CardTint::Space` writes `data-tint="space"` and lays the Space's gradient (`.ds-frame`, the
tinted chrome's layer) over the plate at the material's frame alpha, under the content, so the
Space's colour reaches the card (Arc's contribution). The optional title row is quiet: the glyph
at 12 and the name in the UI face 600 `--fs-caption` `--ink-soft`; the Batteries and Clock
widgets pass none.

**The card's see-through (contrast-relax pass, 2026-09-26, "relax the contrast then": the user's
decision, superseding the vibrancy pass's .60 grey below).** Over compositor blur the light card
fits the reference exactly, its measured near-white `rgb(247,248,248)` at its measured **.48**
(section 1.1, M27-M28), with a 1 pt dark rim outside (`--m-hairline` at `var(--hair)`), a 1 pt
white rim at .10 inside (`--m-edge`) and a short drop `0 2px 8px` black .12 (`--m-shadow`)
(M32-M34); without blur `--m-tint-solid` is the same near-white at .94. This drops the card's
ink below the small-text 4.5:1 gate over black (3.86:1); the trade is accepted because the
widget's own text is large or bold (design/03-COLOR.md section 17), where the guideline is 3:1.
The dark card keeps its tint colour and takes the smallest alpha the relaxed 3:1 floor needs
across every gate, **.55** (was .67); it is unchanged but for the alpha and the 1 pt rim. The
World Clock card of the reference is opaque `#1c1c1e` (M31); ours stays the material's (open
decision 4).

**Small text over a see-through card.** Not every widget element clears the WCAG large-text
size (18.66 px bold or 24 px regular) that justifies the relaxed 3:1 floor above, even though
its face is bold. These may be hard to read over very dark or very bright wallpapers; their
sizes are unchanged by this pass:

- The world clock's dial numerals (`ds-clock-numeral`), Bricolage 800 at `--fs-dial` (9 px) in a
  Medium row's four small dials, and at `--fs-dial-large` (18 px) on a Small frame's single
  large dial — 18 px sits just under the 18.66 px bold cutoff.
- The world clock's city label and its day and offset lines (`ds-clock-label`, `ds-clock-city`),
  Karla 700 at `--fs-small` (12 px).
- The battery percentage figure (`--fs-widget-figure`, 20 px) in a Medium row of rings: Bricolage
  weight 500 (Medium), not bold, so the regular-text floor (24 px) applies, not the bold one.

The battery's hero percentage (`--fs-widget-hero`, 47 px) and the digital clock's digits
(`--fs-subject` or `--fs-widget-hero`, weight 800) both clear the large-text size regardless of
weight; the digital clock is drawn only on the notification center's tile (the unchanged
Popover material), never on the desktop card.

**Compositor blur, recommended (for sill and the compositor).** Blur behind a desktop widget
with a Gaussian of **sigma 22 logical pixels** (M26): 44 device pixels at 2x. On a compositor that
blurs by a dual-Kawase pass count or a "strength" (COSMIC's `frosted` theme value), pick the
setting whose blur of a sharp edge spreads 10-90 % over about 56 logical pixels (2.56 sigma). The
reference also raises the saturation of what shows through by about 1.8 (M29, low confidence);
the tint cannot do that (it only mixes), so it is a compositor gap: `sill/docs/cosmic-gaps.md`
should carry "saturation x1.8 inside the widget's blur region". Blur on the desktop widget is a
choice the shell may skip: the reference's desktop card in its editing mode shows no blur at all
(M30), and the gallery's "Compositor blur" wall (the Widget reference page) shows the tint over
the blur the fit implies.

## 5. Calendar widget (research and specification only; queued)

The calendar widget becomes the future Calendar app's widget (the app will connect to cloud
calendar providers, as the mail app does). `MonthGrid` is not restyled in this pass; this is the
brief for the pass that does it.

| Size | Content | Reference | Our spec (proposed) |
| --- | --- | --- | --- |
| Small | **date face**: the weekday in the UI face, bold, upper, `--accent` (W15's red weekday in our accent), the date in the display face at `--fs-widget-hero` x 1.6 (54) weight 700; under it the next event as a pill (below) or "No more events today" in `--ink-faint` | W15, C1 | the date face sits on the plate; the compact month grid moves to Medium |
| Medium | the date face and next two events on the left half; the month on the right half as the compact grid with **load dots** (one per event, max three, 3 px, the calendar's colour) and today on the accent disc (in the pass that restyles it, an a flat accent disc per section 2) | C1 (grid + events), W15 | `MonthGrid` compact at Medium gains `Eventful::Count(n)` dots; heat map an open decision |
| Large | the month on top (regular density), the day's events under it as **event pills** | C1-C4 | pill: `--r-chip` corner, the calendar colour at .14 over the plate, a 3 px left bar in the full colour, the title in the colour's `-deep` (dark scheme: `-soft`) `--fs-help` 600, the time in the UI face `--fs-nano` `--ink-soft`; 26 high, 4 apart |

Neighbour days `--ink-faint`; weekends optionally tinted (C1); past days dimmed to `--ink-faint`
(C1). The grid keeps no boxes (common trait e); the month title in the display face, not the
data face's caps (the current `--accent` caps title reads as a form label). Motion unchanged
(`slide-l`/`slide-r`).

## 6. Open decisions (for the user; the gallery's "Widget looks" and "Widget reference" pages render the proposal)

**Settled 2026-09-26, "relax the contrast then":** the card's translucency against its own
legibility gate is no longer open. The card fits the reference's measured see-through exactly
(alpha .48 of a near-white light, .55 of the dark paper dark) rather than being held back to
whatever alpha clears 4.5:1; the widget's own text is gated at 3:1 (the large-text floor)
instead, since it is large or bold. See section 4.3 and design/03-COLOR.md section 17.

1. **Battery percentage weight:** 500, the display face's lightest, as the reference measures
   (Regular to Medium, M13; proposed), or 800 as the fourth-pass brief asked ("heaviest weight
   for numerals")? The clock's numerals are 800 either way.
2. **Battery track:** the plate darkened (`rgba(0,0,0,.12)`, as measured, M8; proposed) or a
   light tint of the ring's green, as the brief's diagnosis described it?
3. **Low colour:** one red for low (20 % or less) and critical (10 % or less), as the reference
   shows at 13 % (proposed), or amber for low and red for critical?
4. **World Clock card:** the reference's medium World Clock card is dark in the light scheme too
   (M23); ours follows the scheme (proposed). A dark card by force would need the shell to scope
   the widget dark.
5. **The laptop glyph:** our icon set has no laptop; this computer shows `Monitor`, and the
   devices' glyphs are outline where the reference's are filled. A filled device set is an icon
   task (`08-ICONS.md`).
6. **Space tint by default:** `CardTint::Material` (proposed) or `Space` for every widget?
7. **Calendar** (section 5, unchanged): month load as dots (proposed) or a heat tint; the Small
   calendar as the date face with the next event (proposed) or the compact month grid.

## 7. Sources

- Apple Human Interface Guidelines, "Widgets" (developer.apple.com/design/human-interface-guidelines/widgets,
  read through its JSON at developer.apple.com/tutorials/data/design/human-interface-guidelines/widgets.json,
  change log to 2025-12-16): margins, type, colour, rendering modes, mounting and treatment
  styles, specifications.
- Macworld, "How to add widgets to the macOS Sonoma Desktop"; Intego, "How to Use Desktop
  Widgets on macOS Sonoma": the desktop widget set (Battery, World Clock, Calendar) and the
  Automatic / Monochrome / Full-color style.
- Apple Support Communities 251854825 (Batteries widget: small rings only, medium rings and
  percentage) and 251886962 (World Clock face white by day, dark by night; switch time observed
  off sunrise and sunset).
- Microsoft Learn, "Widgets design fundamentals" (github.com/MicrosoftDocs/windows-dev-docs,
  hub/apps/design/widgets/widgets-design-fundamentals.md); Fluent 2 "Shapes".
- MacStories, "Fantastical's Widgets Pair Interactivity with Superior Design"; Flexibits,
  Fantastical for iOS help, "Calendar Views".
- Blake Crosley, design guides "Notion Calendar: Swiss Precision Meets Workspace Integration"
  and "Amie: Joyful Productivity Through Warm Minimalism" (measured values).
- 9to5google, "Google Calendar website rolling out Material You redesign and dark theme"
  (2024-10-23) and "Google Calendar Material 3 Expressive redesign starts rolling out"
  (2025-08-07).
- vimcal.com and reviews (efficient.app): colour coding by keyword, dark mode.
- Screenshots measured in section 1.1 (reference-only, never committed): Intego, "How to Use
  Desktop Widgets on macOS Sonoma" (`widgets3.png`, `widgets4.png`, `widgets5.png`); 512 Pixels,
  Aqua screenshot library, macOS 14 and 15 (`14-Sonoma-Clock.png`,
  `15-Sequoia-Notification-Center.png`); iDownloadBlog, "How to see AirPods battery percentage on
  any device" (`airpods-and-case-battery-in-widget.jpg`); Macworld, "How to add widgets to the
  macOS Sonoma Desktop"; Apple Newsroom, "macOS Sonoma brings new capabilities" (2023-06-05)
  desktop images.
- Sibling docs: `03-COLOR.md` 8, 17; `04-COMPONENTS.md` 39, 40; `07-LOOKS.md` 3, 11.1;
  `08-ICONS.md` 2.5, 2.9, 2.10; `20-SURFACES.md` 1.12, 1.14; `CHECKLIST.md` (contrast).

## 8. Reference survey

### 8.1 The reference desktop's widgets (the Sonoma-era widget style)

| # | Fact | Value | Conf. | Source |
| --- | --- | --- | --- | --- |
| W1 | Contexts | The same small, medium, large (and extra large) system widgets appear on the Mac desktop and in Notification Center | H | HIG Widgets, "System family widgets" |
| W2 | Margin | "the standard margin width for widgets — 16 points for most widgets"; "margins of 11 points" for grouped content; "widgets use smaller margins on the desktop on Mac" | H | HIG Widgets, "Choosing margins and padding" |
| W3 | Corner | content corners are coordinated with the widget's own corner by a container (`ContainerRelativeShape`), not chosen per widget; no number is published | H (rule) / UNKNOWN (number) | HIG Widgets |
| W4 | Size on screen | iOS small 158-170 pt square, medium 338-364 x 158-170; visionOS small 158, medium 338 x 158, large 338 x 354 (the Mac's are not published; they read the same as the phone's at 1x) | H (iOS, visionOS) / L (Mac) | HIG Widgets, "Specifications" |
| W5 | Type | "Prefer using the system font"; "a custom font for the large text in a widget and SF Pro for the smaller text"; "display text using fonts at 11 points or larger" | H | HIG Widgets, "Displaying text" |
| W6 | Density | "Balance information density. Sparse layouts can make the widget seem unnecessary, while overly dense layouts are less glanceable" | H | HIG Widgets, "Best practices" |
| W7 | Rendering on the desktop | full colour while the desktop is in front; the *vibrant* rendering mode when windows cover the desktop: content desaturated, "white or light gray for the most prominent content and darker grayscale values for secondary elements"; user choice Automatic / Monochrome / Full-color | H | HIG Widgets, "Rendering modes"; Macworld, Intego (Sonoma desktop widgets) |
| W8 | Colour | "Convey meaning without relying on specific colors"; "Use color to enhance a widget's appearance without competing with its content" | H | HIG Widgets, "Using color" |
| W9 | Card | an opaque card per scheme (white in light, near-black `#1c1c1e`-like in dark), a large continuous corner (about 22 pt at small size), a soft wide drop on the desktop; no visible hairline in light | L | screenshots of macOS 14-15 desktop widgets |
| W10 | Hierarchy | one hero value per small widget in a heavy rounded or display cut (the temperature, the date, the percentage) at roughly 2.5-3x the label size; the label above it in a small bold caption, often coloured (Calendar's red weekday) | L | screenshots |
| W11 | Batteries | Small: one ring per device (up to four in a 2 x 2) with the device's glyph in the ring, green, no numbers; Medium: the rings in a row with the percentage under each; Large: a list, a horizontal bar per device with glyph, name and percentage. Charging shows a bolt at the ring's top; low turns the ring red | M | Apple Support Communities 251854825 (small = rings, medium = rings + percentage); L for the rest |
| W12 | Batteries ring | a thick round-capped arc (.093 of the ring) on a track that is the plate darkened, not a hue; the glyph is the device's filled symbol in the ring's middle; measured in section 1.1 (M5-M12) | M | section 1.1 |
| W13 | World clock | Analog faces, one per city in the Medium widget (four across), the city under each; the whole face white by day and black by night | M | timeanddate/Apple discussions: "white face during the daytime and a black one at night"; the switch time is not published (discussion 251886962 observes it off sunrise and sunset) |
| W14 | Clock dial | twelve heavy numerals; sixty ticks on the small widget's large dial, none on the medium's; orange seconds hand with a hub ring; black hands with a thin neck near the hub; the day/night switch recolours face, numerals and hands together; measured in section 1.1 (M16-M22) | M | section 1.1 |
| W15 | Calendar | Small: the weekday in red caps, the date in a large display weight, then the next event with a coloured left bar; Medium: that on the left and the month grid or events on the right; Large: the month grid with today on a red disc, event dots | L | screenshots |

### 8.2 The phone platforms' widget galleries

| # | Fact | Value | Conf. | Source |
| --- | --- | --- | --- | --- |
| P1 | Appearances | light, dark, clear and tinted; in clear "the system desaturates the widget and adds translucency, highlights" | H | HIG Widgets, "Appearances" |
| P2 | visionOS | "Elevated style" casts "a soft shadow that helps it feel grounded"; "Recessed style" sets "content set back into the surface, creating a depth effect that gives the illusion of a cutout" | H | HIG Widgets, "Mounting styles" |
| P3 | Paper vs glass treatment | paper "a more grounded, print-like style that feels solid"; glass "adds depth and visual separation between foreground and background elements" | H | HIG Widgets, "Treatment styles" |
| P4 | Android (Material 3) | widgets take the dynamic colour of the wallpaper; the 2025 calendar redesign puts each day in "its own rounded rectangle" | M | 9to5google, Android Police (Calendar Material 3 Expressive) |

P2 and P3 were the second pass's basis (an object with thickness, gauges cut into it); the user
rejected that pass, and the desktop widgets measured in section 1.1 are flat.

### 8.3 Windows 11 widgets

| # | Fact | Value | Conf. | Source |
| --- | --- | --- | --- | --- |
| X1 | Margin | "Each widget has a 16px margin around it and a 48px Attribution area" | H | Microsoft Learn, widgets design fundamentals |
| X2 | Grid | 4 px gutters; "Multiples of Four Px" | H | same |
| X3 | Type | Segoe UI: Caption 12/16, Body 14/20, Body Strong 14/20 bold, Body Large 18/24, Title 28/36 | H | same |
| X4 | Background | "solid light/dark background, gradient tint, or image background" | H | same |
| X5 | Corner | Fluent default 4 px on small shapes; board cards about 8 | H (4) / L (8) | Fluent 2 "Shapes" |

The Windows board reads as a feed of flat cards; it is the counter-example for this pass.

### 8.4 Calendar apps (for section 5 and the future Calendar app)

| # | App | What makes it read as modern | Values | Conf. | Source |
| --- | --- | --- | --- | --- | --- |
| C1 | Fantastical | The medium widget pairs a month grid (left) with the day's events (right); days carry "a heat map or dots to show how much you have scheduled"; month view "separates months and highlights the selected one"; past events can be dimmed | heat map or dots, user's choice | M | MacStories review; Flexibits help |
| C2 | Notion Calendar (Cron) | Keyboard-first; hierarchy from size and weight juxtaposition rather than colour; soft colour fills with darker text; hairline grid | grid `rgba(0,0,0,.09)` 1 px; event radius 4; time gutter 11 medium tabular; labels 12/500 upper tracked .125; eight event colours; ink at .9/.54/.35 | M | Blake Crosley design guide (measured) |
| C3 | Amie | Colour is the navigation: 15 scales of 9 steps; warm neutrals; "elevation through color differences rather than drop shadows"; barely-there shadows | row 60; Inter; body 16/1.75; inner shadow `.04`, outer `.12`; 4 px grid | M | Blake Crosley design guide |
| C4 | Google Calendar | Material 3: rounded containers, per-day rounded rectangles on Android; today on a filled accent disc; event chips as filled pills with white text | not measured | M / L | 9to5google 2024-10-23, 2025-08-07 |
| C5 | Vimcal, Morgen | Dense dark UIs; auto colour-coding by keyword; the time-zone column | not measured | L | vimcal.com; reviews |

Common to all five: **(a)** an event is a pill of its calendar's colour at low alpha with a
3 px solid left bar of the full colour and the title in the colour's dark shade (C2, C4, W15);
**(b)** today is a filled accent disc behind the number, never a ring; **(c)** the month grid
shows load, not titles, at widget size: dots (one per event, max three) or a heat tint; bars
belong to the app's month view; **(d)** hierarchy from size and weight (the date large and
heavy, labels small caps) with one accent colour; **(e)** hairline grids or none, never boxes.
