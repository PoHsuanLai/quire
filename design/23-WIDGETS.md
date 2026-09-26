# 23 Widgets: Neumorphism & Soft UI, the battery, the world clock, the calendar

Status legend as in `13-BEHAVIOUR-menus-windows.md`: **settled** = decided with the user;
**proposed** = chosen here, rendered in the gallery's "Widget looks" page, the user judges.
Confidence of a reference value: **H** primary source (the vendor's own guidelines or docs),
**M** a reliable secondary source (a review, a teardown, a design write-up that measured the
product), **L** observed from screenshots and memory of the shipping product, not measured,
**UNKNOWN** nothing published. The reference platforms are named here as sources only; code,
class names and asset names never name them.

## 1. What this governs

The small desktop widgets' look: the style every widget is drawn in (section 2, "Neumorphism &
Soft UI"), how our widgets apply it (section 3), the battery, the world clock and the card
(section 4), and the research and specification for the calendar widget (section 5, queued:
this pass does not restyle `MonthGrid`). The frame's footprint and padding stay
`04-COMPONENTS.md` section 40; the surface row stays `20-SURFACES.md` section 1.14; the
material stays `03-COLOR.md` section 17. The reference survey behind all of it is section 8.

**Why this file exists.** The user's verdicts, in order (2026-09-26):
1. First widgets (a flat stroked ring, a plain paper dial): "too ugly"; "this battery circle
   thing looks nothing different to other old looking linux distros".
2. Second pass (depth: the ring in a well round a boss, a glossy cell, bezel and sky dials, a lit
   card): "all widgets looks bad. i want modern abstract design. i think for battery
   specifically drop the circle, use a battery icon and a number to show, like mac".
3. Direction for the third pass: the widget style is **Neumorphism & Soft UI** (section 2).

So: not crude, not skeuomorphic; no ring, no gauge, no bezel, no gloss, no well with a lip. The
depth candidates of pass two (`BatteryLook`, `DialLook`, `FrameFinish` and their `--widget-*`
paints) are removed.

## 2. Neumorphism & Soft UI

**Neumorphism & Soft UI** is the widget style's name; every widget and every agent working on
one uses it. Definition (proposed 2026-09-26, the user's direction):

1. **Monochrome with the material.** A shape is the plate itself, not a second material: it
   has the plate's own colour (no fill of its own, or the plate's tint), so a widget reads as one
   object moulded out of one sheet.
2. **Extruded or inset, by a pair of soft shadows.** A shape is either pushed out of the plate
   (**extruded**, raised) or pressed into it (**inset**). Both are drawn only by two shadows of
   the plate's own light: a lit one and a shaded one, low-contrast, with a wide blur and a small
   offset. Nothing else makes depth: no outlines, no bevel lines, no lips.
3. **One light, from the top left.** An extruded shape throws `--soft-light` up and to the left
   and `--soft-dark` down and to the right; an inset shape takes `--soft-inset-dark` inside its
   upper left edge and `--soft-inset-light` inside its lower right. The direction never turns
   with the shape (a clock hand keeps its light at the top left whichever way it points).
4. **Rounded everything.** Every corner is rounded (the plate's 20, the battery body's 6, the
   channel's 3, pills and discs); every stroke has round caps.
5. **Matte.** No gloss, no specular line, no sheen, no glass; no gradient except, where a shape
   needs it, a very faint one of the plate's own colour for the extrusion. No bezels.
6. **Abstract geometric glyphs.** A battery is a rounded rectangle with a cap, a clock a disc with
   two bars, the sun a disc, the moon a crescent of two circles; no illustration, no realism
   (`08-ICONS.md`, the app icons are the benchmark for the level of abstraction).
7. **Legible type and glyphs stay on the ink tokens.** Neumorphism's known weakness is contrast:
   shapes the colour of their ground cannot carry meaning. So every value that must be read (the
   hero number, labels, hands, the battery's fill, the bolt) is drawn in `--ink`, `--ink-soft`,
   `--ink-faint` or a status colour (`--ok`, `--warn`, `--danger`, `--accent`), never in a soft
   tone, and the text pairs are `03-COLOR.md` section 6's with the `CHECKLIST.md` contrast gates.
   The soft shadows only say which shapes stand out and which are pressed in.
8. **The card is not extruded.** The widget card is the `Widget` material's plate as it is: its
   own tint, hairline, contact and ambient drop (`03-COLOR.md` 17) are its only separation from
   the wallpaper. The soft pairs apply to the shapes on the card.

### 2.1 Tokens (`ds::tokens::SoftPaint`; proposed 2026-09-26)

| Token | Light | Dark | Role |
| --- | --- | --- | --- |
| `--soft-tone-light` | `rgba(255,255,255,.95)` | `rgba(255,255,255,.07)` | the lit tone |
| `--soft-tone-dark` | `rgba(26,30,26,.26)` | `rgba(0,0,0,.62)` | the shaded tone |
| `--soft-light` | `-2px -2px 5px var(--soft-tone-light)` | same, dark tone | an extruded shape's lit side |
| `--soft-dark` | `2px 2px 6px var(--soft-tone-dark)` | same | an extruded shape's shaded side |
| `--soft-inset-light` | `inset -2px -2px 4px var(--soft-tone-light)` | same | an inset shape's lit inner edge |
| `--soft-inset-dark` | `inset 2px 2px 5px var(--soft-tone-dark)` | same | an inset shape's shaded inner edge |
| `--soft-night` | `#1a1e1a` | `#0a0c0a` | a night dial's ground, darker than the plate in either scheme |

An extruded box is `box-shadow: var(--soft-light), var(--soft-dark)`; an inset box
`box-shadow: var(--soft-inset-dark), var(--soft-inset-light)`. An SVG shape cannot take a
`box-shadow` (and Blitz paints no filter): it is drawn three times, a copy in
`--soft-tone-dark` shifted down and right and a little wider, a copy in `--soft-tone-light`
shifted up and left, then the shape; the shift is applied before any turn, so the light stays
at the top left. On the light Widget plate (near white) the lit tone barely shows, so the
extrusion is carried by the shaded side; on the dark plate both show.

## 3. Our widget language

### 3.1 The rule

**Abstract, matte, big type; no gauges, no bezels, no gloss.** A widget is one soft plate with
at most a few soft shapes on it and one hero value in large display type. The app icons
(`assets/icons/apps`) are the benchmark for abstraction: if a shape would not sit in that set's
language, it does not go on a widget. A stroked ring on a track, a dial with a minute track or
numerals, a glossy liquid, a well with a lip, a bezel, a sky gradient: each fails the rule.

### 3.2 Type

The hero value (a battery's percentage, a small clock's time, the date face) is the display
face (`--font-display`, Bricolage Grotesque) at `--fs-widget-hero` (44 px, raised from 34 so the
small widget has no dead middle band) weight 700, tracked -.02em, tabular numerals; its unit
(`%`) at `--fs-subject` 600. Names (a device, a city) are the UI face (`--font-ui`) at
`--fs-control`/`--fs-help` 600 `--ink`; secondary data (a zone offset, a device under the hero,
a row's percentage) the data face (`--font-data`) at `--fs-caption`/`--fs-small` `--ink-soft`
or `--ink-faint`, tabular. The card's title row is a quiet eyebrow (4.3).

### 3.3 Composition

- **Small**: the eyebrow at the top; the content anchored to the bottom left (hero, then its
  label); nothing floats in the middle band.
- **Medium**: the eyebrow, then either rows (the battery: glyph, name, number) spread over the
  body's height, or four faces spread across it (the world clock), each with its label under it.
- Padding 16 (the material's); nothing touches the plate's edge nearer than that.

## 4. Specification per widget

### 4.1 Battery (`BatteryLevel`; `LevelRing` is its old name)

`BatteryLevel { level: Fraction, mark: RingMark::{Plain, Charging}, label: Text, children }`,
renamed from `LevelRing` (kept as `pub use BatteryLevel as LevelRing`, so a caller of the old
name compiles unchanged); every prop kept, the `look` prop removed. Markup
`div.ds-battery[data-tone][data-mark][role=progressbar][aria-valuenow]` holding, when
`children` is given, `span.ds-battery-device` (a device glyph, `--ink-soft`, 6 before the
battery), then `span.ds-battery-glyph` (32 x 16; 40 x 20 in a Small frame, beside the hero):

| Part | Drawing |
| --- | --- |
| Body | a rounded rectangle (corner `--r-small` 6) of the plate's own colour, **extruded**: `--soft-light`, `--soft-dark`; 3 short of the right edge for the cap |
| Channel | 3 inside the body, corner `--r-micro` 3, **inset**: `--soft-inset-dark`, `--soft-inset-light` |
| Fill | 1.5 inside the channel, a solid matte bar as long as the level (`--f`, never shorter than 3, so 1 % is a sliver), corner 3; `--ink` by default, `--warn` at a fifth or less, `--danger` at a tenth or less, `--ok` while charging (a charging battery is never low) |
| Cap | a 2 x 36 % terminal at the right, `--ink-faint` |
| Bolt | while charging, a bolt in `--ink` over the body, cut out of the body and fill by a 3-unit outline in the plate's colour (`--m-tint-solid`) |

**Composition** (the shell composes, quire provides the parts):
- **Small**: the eyebrow ("Battery"); at the bottom left the hero percentage (display face,
  `--fs-widget-hero`, `%` at `--fs-subject`) with the glyph (40 x 20) beside it on the baseline,
  10 apart; under it the device name in the data face `--fs-caption` `--ink-faint`.
- **Medium**: the eyebrow ("Batteries"); up to four rows spread over the body: the glyph, the
  name (UI face `--fs-control` 600, `--ink`, ellipsised), the percentage right-aligned (data face
  `--fs-small`, tabular, `--ink-soft`), 10 apart.

Motion: the fill bumps once on a new percentage (`use_bump_on`); its length does not animate
(O-20).

### 4.2 World clock (`ClockFace`)

`ClockFace { time, phase: DayPhase, look: ClockLook::{Analog, Digital}, label }`: every prop
kept, the `dial` prop removed.

| Part | Analog |
| --- | --- |
| Dial | a 56 px disc **inset** in the plate (`--soft-inset-dark`, `--soft-inset-light`): by day the plate's own colour; by night `--soft-night`, keeping only its inset shade (a lit inner edge on a dark ground reads as gloss) |
| Marks | four quarter marks, 2 px round bars 3.4 long, at .4 of the hands' colour; no tick ring, no numerals |
| Hands | thick round bars from the centre: the hour hand 3.9 px to .55 of the radius, the minute hand 2.8 px to .8; the hub 6 px; **extruded** as SVG (a `--soft-tone-dark` copy shifted down and right, a `--soft-tone-light` copy up and left; by night the light copy is dropped); `--ink` by day, `--paper` by night (`--ink-soft` on a dark desktop, whose ink is already pale) |
| Second hand | 1 px, `--accent`, from 10 behind the hub to .84 of the radius, with a 3 px `--accent` dot; flat |

The dial is drawn in the desktop's scheme (pass two forced a light scope by day and a dark one
by night; a soft shape must share its plate's colour, so the dial now follows the plate).

**Digital**: the time in the display face, 700, tabular, tracked -.02em, at `--fs-widget-hero` in
a Small frame and `--fs-subject` anywhere else (four fit a Medium tile); under it the city in the
UI face `--fs-help` 600 with the phase's mark before it: the **sun** a 12 px disc in `--warn`,
the **moon** a crescent (a disc less a second disc) in `--ink-faint`. It bumps on each new minute.

**Composition**:
- **Small**: the eyebrow ("Clock"); at the bottom left the time, the city with its mark, then
  the offset ("Today", "+1 h") in the data face `--fs-caption` `--ink-faint`.
- **Medium**: the eyebrow ("World Clock"); four dials spread across the body, each with the city
  (UI face `--fs-help` 600 `--ink`) and the offset (data face) under it.
- **Day and night**: the phase the shell computes (sunrise and sunset where it knows the zone,
  else 06:00-18:00; W13 publishes none).

### 4.3 The frame (`WidgetFrame`)

`WidgetFrame` keeps every prop but `finish` (removed with the `Lit` card). The card is the
`Widget` material's plate as it is (section 2, item 8). The title row is a **quiet eyebrow**: the
glyph at 12 and the name in the data face at `--fs-micro`, upper, tracked .08em, weight 400,
both `--ink-faint`, 5 apart, so the widget's own value leads. Padding 16 on the desktop, 12 as a
tile (unchanged).

## 5. Calendar widget (research and specification only; queued)

The calendar widget becomes the future Calendar app's widget (the app will connect to cloud
calendar providers, as the mail app does). `MonthGrid` is not restyled in this pass; this is the
brief for the pass that does it.

| Size | Content | Reference | Our spec (proposed) |
| --- | --- | --- | --- |
| Small | **date face**: the weekday in the data face, upper, `--accent` (W15's red weekday in our accent), the date in the display face at `--fs-widget-hero` x 1.6 (54) weight 700; under it the next event as a pill (below) or "No more events today" in `--ink-faint` | W15, C1 | the date face sits on the plate; the compact month grid moves to Medium |
| Medium | the date face and next two events on the left half; the month on the right half as the compact grid with **load dots** (one per event, max three, 3 px, the calendar's colour) and today on the accent disc (in the pass that restyles it, an extruded soft disc per section 2) | C1 (grid + events), W15 | `MonthGrid` compact at Medium gains `Eventful::Count(n)` dots; heat map an open decision |
| Large | the month on top (regular density), the day's events under it as **event pills** | C1-C4 | pill: `--r-chip` corner, the calendar colour at .14 over the plate, a 3 px left bar in the full colour, the title in the colour's `-deep` (dark scheme: `-soft`) `--fs-help` 600, the time in the data face `--fs-nano` `--ink-soft`; 26 high, 4 apart |

Neighbour days `--ink-faint`; weekends optionally tinted (C1); past days dimmed to `--ink-faint`
(C1). The grid keeps no boxes (common trait e); the month title in the display face, not the
data face's caps (the current `--accent` caps title reads as a form label). Motion unchanged
(`slide-l`/`slide-r`).

## 6. Open decisions (for the user; the gallery's "Widget looks" page renders the proposal)

1. **Battery fill colour:** `--ink` by default (proposed: the reference's menu-bar battery fills
   in the ink and turns green, amber, red by state) or the accent?
2. **Second hand:** keep the thin accent second hand when a zone shows seconds (proposed), or
   never draw one on a widget?
3. **Quarter marks:** four (proposed) or none (a bare disc with hands)?
4. **Small clock:** the digital time with the sun or moon (proposed, rendered) or one large soft
   dial?
5. **Night dial on a dark desktop:** `--soft-night` (near black, proposed) keeps day and night
   apart only by the ground's depth and the hands' tone; or a pale day dial on dark desktops too,
   as the reference does (breaks section 2 item 1)?
6. **Hero size:** `--fs-widget-hero` 44 (proposed; 34 left a dead band in the Small card) or 40?
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
| W12 | Batteries ring | a thick round-capped arc on a darker track of the same hue; the glyph is the device's SF Symbol in the ring's middle; the ring's track is not a separate grey but the tone at low alpha | L | screenshots |
| W13 | World clock | Analog faces, one per city in the Medium widget (four across), the city under each; the whole face white by day and black by night | M | timeanddate/Apple discussions: "white face during the daytime and a black one at night"; the switch time is not published (discussion 251886962 observes it off sunrise and sunset) |
| W14 | Clock dial | twelve numerals on the paper face (no minute track at widget size), orange second hand with a hub ring, black hour and minute hands that are thin near the hub and thicken towards the tip; the day/night switch recolours face, numerals and hands together | L | screenshots |
| W15 | Calendar | Small: the weekday in red caps, the date in a large display weight, then the next event with a coloured left bar; Medium: that on the left and the month grid or events on the right; Large: the month grid with today on a red disc, event dots | L | screenshots |

### 8.2 The phone platforms' widget galleries

| # | Fact | Value | Conf. | Source |
| --- | --- | --- | --- | --- |
| P1 | Appearances | light, dark, clear and tinted; in clear "the system desaturates the widget and adds translucency, highlights" | H | HIG Widgets, "Appearances" |
| P2 | visionOS | "Elevated style" casts "a soft shadow that helps it feel grounded"; "Recessed style" sets "content set back into the surface, creating a depth effect that gives the illusion of a cutout" | H | HIG Widgets, "Mounting styles" |
| P3 | Paper vs glass treatment | paper "a more grounded, print-like style that feels solid"; glass "adds depth and visual separation between foreground and background elements" | H | HIG Widgets, "Treatment styles" |
| P4 | Android (Material 3) | widgets take the dynamic colour of the wallpaper; the 2025 calendar redesign puts each day in "its own rounded rectangle" | M | 9to5google, Android Police (Calendar Material 3 Expressive) |

P2 and P3 are the reference vendor's own words for what the user asks of us: a widget is an
object with thickness (elevated) and its gauges are cut into it (recessed).

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
