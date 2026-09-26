# 23 Widgets: depth, the battery, the world clock, the calendar

Status legend as in `13-BEHAVIOUR-menus-windows.md`: **settled** = decided with the user;
**proposed** = chosen here, rendered in the gallery's "Widget looks" page, the user picks.
Confidence of a reference value: **H** primary source (the vendor's own guidelines or docs),
**M** a reliable secondary source (a review, a teardown, a design write-up that measured the
product), **L** observed from screenshots and memory of the shipping product, not measured,
**UNKNOWN** nothing published. The reference platforms are named here as sources only; code,
class names and asset names never name them.

## 1. What this governs

The small desktop widgets' look: how a widget card and the faces inside it get depth in our
system (section 3), the battery gauge and the world clock (section 4, rendered as candidates
behind `data-look`/`data-dial`/`data-finish`), and the research and specification for the
calendar widget (section 5, queued: this pass does not restyle `MonthGrid`). The frame's
footprint, padding and title row stay `04-COMPONENTS.md` section 40; the surface row stays
`20-SURFACES.md` section 1.14; the material stays `03-COLOR.md` section 17.

**Why this file exists.** The user's verdict on the first widgets (2026-09-26): the battery
ring, the clock face and the month grid are "too ugly" and flat; "this battery circle thing
looks nothing different to other old looking linux distros"; "there has to be thought put into
it and made sure it matches the entire modern design". Flat widget chrome is rejected.

## 2. Reference survey

### 2.1 The reference desktop's widgets (the Sonoma-era widget style)

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
| W12 | Batteries ring | a thick round-capped arc on a darker track of the same hue; the glyph is the device's SF Symbol in the ring's middle; the ring's track is not a separate grey but the tone at low alpha | L | screenshots |
| W13 | World clock | Analog faces, one per city in the Medium widget (four across), the city under each; the whole face white by day and black by night | M | timeanddate/Apple discussions: "white face during the daytime and a black one at night"; the switch time is not published (discussion 251886962 observes it off sunrise and sunset) |
| W14 | Clock dial | twelve numerals on the paper face (no minute track at widget size), orange second hand with a hub ring, black hour and minute hands that are thin near the hub and thicken towards the tip; the day/night switch recolours face, numerals and hands together | L | screenshots |
| W15 | Calendar | Small: the weekday in red caps, the date in a large display weight, then the next event with a coloured left bar; Medium: that on the left and the month grid or events on the right; Large: the month grid with today on a red disc, event dots | L | screenshots |

### 2.2 The phone platforms' widget galleries

| # | Fact | Value | Conf. | Source |
| --- | --- | --- | --- | --- |
| P1 | Appearances | light, dark, clear and tinted; in clear "the system desaturates the widget and adds translucency, highlights" | H | HIG Widgets, "Appearances" |
| P2 | visionOS | "Elevated style" casts "a soft shadow that helps it feel grounded"; "Recessed style" sets "content set back into the surface, creating a depth effect that gives the illusion of a cutout" | H | HIG Widgets, "Mounting styles" |
| P3 | Paper vs glass treatment | paper "a more grounded, print-like style that feels solid"; glass "adds depth and visual separation between foreground and background elements" | H | HIG Widgets, "Treatment styles" |
| P4 | Android (Material 3) | widgets take the dynamic colour of the wallpaper; the 2025 calendar redesign puts each day in "its own rounded rectangle" | M | 9to5google, Android Police (Calendar Material 3 Expressive) |

P2 and P3 are the reference vendor's own words for what the user asks of us: a widget is an
object with thickness (elevated) and its gauges are cut into it (recessed).

### 2.3 Windows 11 widgets

| # | Fact | Value | Conf. | Source |
| --- | --- | --- | --- | --- |
| X1 | Margin | "Each widget has a 16px margin around it and a 48px Attribution area" | H | Microsoft Learn, widgets design fundamentals |
| X2 | Grid | 4 px gutters; "Multiples of Four Px" | H | same |
| X3 | Type | Segoe UI: Caption 12/16, Body 14/20, Body Strong 14/20 bold, Body Large 18/24, Title 28/36 | H | same |
| X4 | Background | "solid light/dark background, gradient tint, or image background" | H | same |
| X5 | Corner | Fluent default 4 px on small shapes; board cards about 8 | H (4) / L (8) | Fluent 2 "Shapes" |

The Windows board reads as a feed of flat cards; it is the counter-example for this pass.

### 2.4 Calendar apps (for section 5 and the future Calendar app)

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

## 3. Our widget language

### 3.1 The rule

**Nothing flat, nothing that could pass as a stock distro widget.** Every visible part of a
widget carries at least one **material cue** (an edge highlight, a recessed well, a gradient
with a reason: light from above, a sky, a liquid's meniscus) and one **typographic hierarchy
cue** (a hero value in the display face against labels in the data face, or a size step of at
least 2x). A plain stroked circle on a flat card, a single-weight label row, or a gauge drawn
as a hairline outline fails the rule.

Depth comes from the same light the icons use (`08-ICONS.md` section 2.9): **one light from
above**, so every raised edge is lit on top and shaded below, every recessed edge the reverse.
No skeuomorphic texture (no metal, no glass refraction, no leather), no frosted glass (the user,
icons round three), no illustration: the depth is bevel, well and gloss, as the icon plate's is.

### 3.2 The vocabulary

| Cue | What it is | Value (proposed; tokens in 3.3) | Borrowed from |
| --- | --- | --- | --- |
| Plate | the widget card: the `Widget` material (tint, hairline, contact and ambient drop, 20 corner) | `03-COLOR.md` 17.2, 17.4 | the material |
| Plate bevel (`Lit` finish) | a warmer, brighter wash; a bright top highlight line and a faint inner rim; a sheen fading down the top third; a shade rising from the foot over half the card (the icons' light from above); a contact edge and a deeper drop | `--widget-lift`, `--widget-sheen` over the top 34 %, `--widget-foot` up to 55 %, `--widget-bevel` (3.3) | icon plate bevel and matte diffusion (08 2.9) |
| Well | a gauge's bed, pressed into the plate: darker than the plate, a two-stop shade under its top edge, light along its inner foot and a lit lip outside it | `--widget-well`, `--widget-well-shade` (3.3) | icon `recessed` relief (08 2.9), the P2 "recessed" mounting |
| Gloss | the level's fill catches the light: a lighter band on its top half, a shaded foot, a specular line along its top | `--widget-gloss` to 0 at 52 %, `--widget-liquid-shade` up to 45 %, `--widget-spec` 1 px | the pill's inner highlight (07 3.2, `--shadow-pill-inset`) |
| Boss | a raised disc or cap (the ring's centre, the bolt, the bezel, the cell's terminal): lit top, shaded foot, a contact shadow and a soft drop | `--widget-boss` (3.3) | `--shadow-1` |
| Bezel | a raised ring round a dial: the rim's gradient lit on top and dark below, seated by `--widget-boss`, then the face recessed 5 px inside it | `--widget-rim-top` → `--widget-rim-bottom` | icon plate thickness (08 2.9) |
| Sky face | the dial's ground by phase, across the whole face: deeper at the zenith, paler at the horizon, with a glass highlight fading down its top 42 % | day `#a9c8e2` → `#f7fafc`; night `#060a13` → `#2c3a54`; `--widget-glass` | the W13 day/night faces, kept in our palette (slate 265) |
| Hand shadow | each hand drawn twice: a copy offset 3 px down and 1.5 px right, ink at .30 by day, black at .65 by night; indices pressed in, with a `--widget-spec` copy 1 px below each | SVG attribute paint (spike S6), no filter (Blitz paints none) | the icons' "seat" and emboss (08 2.9) |
| Grain | the plate keeps the Space grain it already takes as a material | `03-COLOR.md` 8 | the frame |

**Type.** The hero value (a battery's percentage, a small clock's time, the date face) is the
display face (`--font-display`, Bricolage Grotesque) at `--fs-widget-hero` (34 px, new token)
weight 700, tracked -.02em, tabular numerals; its unit (`%`) at half size, 600, `--ink-soft`,
baseline-aligned. Labels (the device, the city, the zone offset) are the data face
(`--font-data`) at `--fs-nano`/`--fs-caption`, upper, tracked .08em, `--ink-soft`. The ratio
hero : label is 34 : 10, above 3:1, the size juxtaposition C2 names.

### 3.3 Tokens (proposed; all in `ds::tokens::WidgetPaint`, light / dark; round two 2026-09-26)

| Token | Light | Dark | Used by |
| --- | --- | --- | --- |
| `--widget-well` | `rgba(26,30,26,.10)` | `rgba(0,0,0,.34)` | wells (ring groove, cell bed) |
| `--widget-well-shade` | `inset 0 2px 4px rgba(26,30,26,.30),inset 0 1px 1px rgba(26,30,26,.20),inset 0 -1px 0 rgba(255,255,255,.85),0 1px 0 rgba(255,255,255,.7)` | `inset 0 2px 5px rgba(0,0,0,.75),inset 0 1px 1px rgba(0,0,0,.5),inset 0 -1px 0 rgba(255,255,255,.10),0 1px 0 rgba(255,255,255,.08)` | wells, the dial face, the sky disc |
| `--widget-gloss` | `rgba(255,255,255,.60)` | `rgba(255,255,255,.38)` | a fill's gloss band, the ring's gloss arc, the lit bolt, the nub |
| `--widget-spec` | `rgba(255,255,255,.85)` | `rgba(255,255,255,.45)` | a fill's specular line, the light under pressed indices, the sky dial's lip |
| `--widget-sheen` | `rgba(255,255,255,.55)` | `rgba(255,255,255,.09)` | the `Lit` plate's top sheen, a boss's face |
| `--widget-bevel` | `inset 0 1px 0 rgba(255,255,255,.95),inset 0 0 0 1px rgba(255,255,255,.22),inset 0 -2px 3px -1px rgba(26,30,26,.14),0 1px 0 rgba(26,30,26,.10),0 12px 28px -12px rgba(26,30,26,.40)` | `inset 0 1px 0 rgba(255,255,255,.22),inset 0 0 0 1px rgba(255,255,255,.04),inset 0 -2px 3px -1px rgba(0,0,0,.45),0 14px 30px -12px rgba(0,0,0,.7)` | the `Lit` plate (1 px is `var(--hair)`) |
| `--widget-lift` | `rgba(255,251,242,.30)` | `rgba(255,250,240,.035)` | the `Lit` plate's warmer, brighter wash |
| `--widget-foot` | `rgba(26,30,26,.12)` | `rgba(0,0,0,.25)` | the `Lit` plate's shade from the foot; the cell bed's seat |
| `--widget-rim-top` / `--widget-rim-bottom` | `#ffffff` / `#a9b2a6` | `#5a6376` / `#0a0d13` | the bezel |
| `--widget-sky-top` / `--widget-sky-bottom` | `#a9c8e2` / `#f7fafc` | `#060a13` / `#2c3a54` | the sky face (the dial scope's scheme is its phase) |
| `--widget-glass` | `rgba(255,255,255,.55)` | `rgba(255,255,255,.14)` | the glass highlight across a face's top |
| `--widget-hand-shadow` | `rgba(26,30,26,.30)` | `rgba(0,0,0,.65)` | the hands' shadow copy, the liquid's seat, the hub's |
| `--widget-liquid-shade` | `rgba(0,0,0,.18)` | `rgba(0,0,0,.30)` | a liquid's shaded foot |
| `--widget-boss` | `inset 0 1px 0 #fff,inset 0 -1px 1px rgba(26,30,26,.08),0 1px 2px rgba(26,30,26,.25),0 4px 10px -3px rgba(26,30,26,.35)` | `inset 0 1px 0 rgba(255,255,255,.16),inset 0 -1px 1px rgba(0,0,0,.4),0 1px 2px rgba(0,0,0,.7),0 4px 10px -3px rgba(0,0,0,.7)` | bosses, the bolt, the bezel, the nub |
| `--fs-widget-hero` | 34 px | same | the hero value |

Round two (2026-09-26, after the first 1x sheet): the wells took a two-stop inset and a lit
lip, so a ring sits in the plate; the boss a real drop; the `Lit` plate a warmer wash, a
stronger sheen, a shade over the lower half, a contact edge and a deeper drop; the sky a
dial-wide gradient strong enough to read at 72 px, and a glass highlight; the indices pressed
in; the hand shadow twice as far; the cell's gloss and foot stronger, a seat hairline on the
bed and a glossed terminal. Judged at 2x over a calm two-stop wallpaper
(`ds-gallery --snapshot DIR --page widget-looks --scale 200`), never the test pattern.

A dial is drawn in its phase's scheme (`04-COMPONENTS.md` section 40), so the dark column of a
dial token is its night value on any desktop.

## 4. Specification per widget

### 4.1 Battery (`LevelRing`, `look: BatteryLook`)

`LevelRing` keeps every prop and gains `look: BatteryLook::{Ring, Well, Cell}` (default
`Ring`, the current flat ring, until the user picks; `data-look` is written only for the
other two, so every existing golden stays). The tone rule (`RingTone`) is unchanged and applies
to every look: `--ok`, `--warn` at a fifth or less, `--danger` at a tenth or less, a charging
battery never low. **Low and critical are the fill's colour, never a flat red circle**: the
well, gloss and bevel stay; only the liquid changes hue.

| Look | Drawing | Size |
| --- | --- | --- |
| `Ring` (current) | a 3.5-unit stroke on a .2 track; bolt on a paper disc | 52 |
| `Well` (candidate) | the ring lies in a **groove**: a disc of `--widget-well` with `--widget-well-shade`, a raised **boss** in the middle (`--raise` under `--widget-sheen`, `--widget-boss`, inset 19 %) holding the glyph, so the groove is the ring between them. The level arc (stroke 4.4 at .9 of the radius, round caps) is set into the groove in the tone, with a narrower `--widget-gloss` arc (stroke 1.5 at .96) riding its outer half, so the liquid reads rounded. Charging: the bolt sits on a small boss at twelve, filled with the tone under `--widget-gloss` | 60 |
| `Cell` (candidate) | a horizontal **cell**: a recessed capsule bed (`--widget-well`, `--widget-well-shade`, `--r-pill`) 3 px inset round a **liquid** capsule whose width is the level (min one height, so 1 % still reads as a bead), filled with the tone under a `--widget-gloss` band on its top half and a 1 px `--widget-spec` line; a terminal nub at the right, a glossed `--ink-faint` cap with `--widget-boss`. Charging: a bolt in the bed's middle, `--ink` on a `--raise` boss; the tone never goes low | fills its parent's width, 28 high (Small widget: full width) |

**Composition** (the shell composes, quire provides the parts):
- **Small**: the title row ("Battery"); the hero percentage in the display face with `%` at
  half size; under it the gauge (`Cell` full width, or `Well` beside the number); the device
  name in the data face.
- **Medium**: up to four devices in a row, each a `Well` ring (glyph in the boss) with the
  percentage under it in the data face at `--fs-help` 700; or four `Cell` rows (glyph, name,
  cell, percentage) as the reference Large does (W11).
- **Charging**: a lit bolt (4.1 table), and the percentage keeps its colour; no pulsing.

Motion: unchanged (`bump` on a new percentage; the arc and the liquid do not animate, O-20).

### 4.2 World clock (`ClockFace`, `dial: DialLook`)

`ClockFace` keeps every prop and gains `dial: DialLook::{Paper, Bezel, Sky}` (default
`Paper`, the current flat dial; `data-dial` written only for the other two). The dial is still
drawn in its phase's scheme.

| Look | Face | Ticks | Hands |
| --- | --- | --- | --- |
| `Paper` (current) | flat `--paper` disc, `--shadow-1` | twelve, quarters heavier | ink bars, accent second |
| `Bezel` (candidate) | a raised **bezel** 5 px wide (`--widget-rim-top` → `--widget-rim-bottom` down, a contact shadow under it) round a **recessed** face in the sky gradient (`--widget-sky-top` → `--widget-sky-bottom`) with `--widget-well-shade` under the bezel's inner edge | a 60-minute track (hairlines at .25) and twelve hour bars (quarters longer and heavier) | tapered: the hour and minute hands are round-capped bars that thicken from the hub (2 units) to 5 and 3.5 at their ends as two strokes; each drawn over its **shadow copy**; the hub a boss in the accent with a paper centre; the second hand in `--accent` with a counterweight |
| `Sky` (candidate) | no bezel: the whole dial is a **well** in the plate, its ground the sky gradient, the well shade inside its edge, a hairline seat and a 1 px lit lip at its foot | twelve round **dots** (quarters as short bars), no minute track: calmer at 72 px | as `Bezel` |

- **Day and night**: day is the sky's pale pair with ink hands; night the deep slate pair with
  paper hands and ticks; the accent second hand in both. The switch is the phase the shell
  computes (sunrise and sunset where it knows the zone, else 06:00-18:00; W13 publishes none).
- **Zone label and offset**: the city in the UI face `--fs-help` 600 `--ink`, the offset
  ("+8", "-7 h", "Today"/"Yesterday") in the data face `--fs-nano` upper `--ink-faint` under it.
  The offset is text the caller passes (it is not a `ClockFace` prop today; sill composes it).
- **Small size**: the digital row, left-aligned: the time in the display face at
  `--fs-widget-hero` with a 16 px disc of the phase's sky beside it (the day/night cue), the city
  and offset under it, so the small clock is not a bare line of digits. (A 40 px `Sky` dial in
  place of the disc is open decision 9.)

### 4.3 The frame (`WidgetFrame`, `finish: FrameFinish`)

`WidgetFrame` keeps every prop and gains `finish: FrameFinish::{Plain, Lit}` (default `Plain`,
the current card; `data-finish=lit` otherwise). `Lit` is the plate bevel (3.2): over the
material's tint, `--widget-lift` (a warmer, brighter wash), `--widget-sheen` down the top 34 %
and `--widget-foot` rising over the lower 55 %; inside the material's own edge and drop,
`--widget-bevel` (the top highlight line, a faint inner rim, a soft inner foot shade, a contact
edge and a deeper drop). The title row steps back to `--ink-faint` at 500 so the hero value
leads. It changes nothing on a `Tile` (the notification center's tiles sit on a Popover, which
already carries the edge).

## 5. Calendar widget (research and specification only; queued)

The calendar widget becomes the future Calendar app's widget (the app will connect to cloud
calendar providers, as the mail app does). `MonthGrid` is not restyled in this pass; this is the
brief for the pass that does it.

| Size | Content | Reference | Our spec (proposed) |
| --- | --- | --- | --- |
| Small | **date face**: the weekday in the data face, upper, `--accent` (W15's red weekday in our accent), the date in the display face at `--fs-widget-hero` x 1.6 (54) weight 700; under it the next event as a pill (below) or "No more events today" in `--ink-faint` | W15, C1 | the date face sits on the plate; the compact month grid moves to Medium |
| Medium | the date face and next two events on the left half; the month on the right half as the compact grid with **load dots** (one per event, max three, 3 px, the calendar's colour) and today on the accent **disc with a boss** (lit top, `--shadow-1`) | C1 (grid + events), W15 | `MonthGrid` compact at Medium gains `Eventful::Count(n)` dots; heat map an open decision |
| Large | the month on top (regular density), the day's events under it as **event pills** | C1-C4 | pill: `--r-chip` corner, the calendar colour at .14 over the plate, a 3 px left bar in the full colour, the title in the colour's `-deep` (dark scheme: `-soft`) `--fs-help` 600, the time in the data face `--fs-nano` `--ink-soft`; 26 high, 4 apart |

Neighbour days `--ink-faint`; weekends optionally tinted (C1); past days dimmed to `--ink-faint`
(C1). The grid keeps no boxes (common trait e); the month title in the display face, not the
data face's caps (the current `--accent` caps title reads as a form label). Motion unchanged
(`slide-l`/`slide-r`).

## 6. Open decisions (picks for the user)

1. **Battery gauge:** `Well` (the ring in a groove round a raised boss) or `Cell` (a recessed
   capsule with a glossy liquid)? Or both: `Cell` for the Small widget's single battery, `Well`
   for the Medium's row of devices (proposed).
2. **Clock dial:** `Bezel` (raised rim, minute track) or `Sky` (a well in the plate, dot
   indices)? Proposed: `Sky` for the medium's four small dials, `Bezel` for the large.
3. **Frame finish:** `Lit` (bevel highlight, sheen, foot) as the default for every desktop
   widget, or keep `Plain`? Proposed: `Lit`.
4. **Hero size:** `--fs-widget-hero` 34 (proposed) or 40 (closer to the reference's small widget
   temperature)?
5. **Day/night boundary:** sunrise/sunset where the zone is known (proposed), or the fixed
   06:00-18:00 the reference appears to ignore too (W13)?
6. **Calendar month load:** dots (one per event, max three; proposed) or a heat tint (C1)?
7. **Calendar Small:** the date face with the next event (proposed, W15) or the compact month
   grid it shows today?
8. **Numerals on the dial:** none at 72 px (proposed; Blitz draws SVG text through fonts it may
   not have, unproven) or 12/3/6/9 in the data face as HTML over the dial?
9. **Small clock's day/night cue:** the 16 px sky disc beside the time (rendered), or a 40 px
   `Sky` dial in the corner?

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
- Sibling docs: `03-COLOR.md` 8, 17; `04-COMPONENTS.md` 39, 40; `07-LOOKS.md` 3, 11.1;
  `08-ICONS.md` 2.5, 2.9, 2.10; `20-SURFACES.md` 1.12, 1.14.
