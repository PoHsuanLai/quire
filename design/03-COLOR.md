# 03 Color

## 1. What this governs

This file fixes every colour: the paper tokens (light and dark), the status colours, the Space's
flat tint, the eight accents, contrast gates, grain, the four shadows, the precomputed washes,
identity colours, which token goes on which element, and the materials for shell surfaces. A
colour that is not in this file does not exist; add it here first. Layout is `01-LAYOUT.md`, type
is `02-TYPE.md`, the one Look is `30-CATALOGUE.md` section 3 (which wins), the Space model is
`21-SPACES.md`. The Arc-era sections (frame tokens, card accent, Candy, Post) are
`archive/03-COLOR-arc.md`. Source keys `S`, `C`, `P` (defined in `00-PRINCIPLES.md` section 1) are
the Arc/Post-era prototypes: their numbers are history unless this file repeats them.

## 2. Two grounds

Colour lives on two grounds: the sidebar ground carries the Space's flat tint, and everything else
(content, controls, apps) is paper. The old frame/card zones (`--f-*`, gradient layers) are
`archive/03-COLOR-arc.md`.

| Ground | Tokens | Who sets them | Rule |
| --- | --- | --- | --- |
| Sidebar | the Space tint (section 4), under the Sidebar material | derived from the Space's hue, chroma factor and theme (21 section 2) | Text on it is the neutral ink tokens. Nothing else is tinted. |
| Everything else | `--paper`, `--surface`, `--surface-2`, `--raise`, `--ink*`, `--line*`, `--accent*`, `--ok`, `--warn`, `--danger`, `--shadow-*`, `--scrim` | fixed palette in the scheme (section 3); accent per section 5 | The Space never enters it, so every contrast rule we test still holds. |

Shell chrome (bar, dock, launcher, control centre) uses the materials of section 17 and takes no
Space tint (21 section 3).

## 3. Paper tokens

The Mac values (30 section 3.2), as the code has them (`crates/ds-style/src/tokens/colour.rs`):

| Token | Light | Dark | Role |
| --- | --- | --- | --- |
| `--paper` | `#ECECEC` | `#1E1E1E` | window ground; inverse text on ink pills |
| `--surface` | `#FFFFFF` | `#2A2A2A` | controls, content, rows |
| `--surface-2` | `#F5F5F5` | `#242424` | reader, composer, peek, mini, kbd |
| `--raise` | `#FFFFFF` | `#323232` | popovers, hover card, menus |
| `--ink` | `#202020` | `#E8E8E8` | primary text; inverse pill ground |
| `--ink-soft` | `#5C5C5C` | `#A3A3A3` | secondary text; headings (11 px) |
| `--ink-faint` | `#858585` | `#858585` | metadata |
| `--line` | `#D9D9D9` | `#444444` | borders, dividers |
| `--line-soft` | `#E6E6E6` | `#383838` | inner dividers |
| `--accent`, `--accent-ink`, `--accent-soft` | section 5, section 20 | | selection, focus, default button |
| `--scrim` | `rgba(0,0,0,.22)` | same | behind sheets and the command menu |

The prototype's greenish Post palette (`#E9ECE6` and friends) is history. `--seal` is deleted
(unused). Status colours:

| Token | Target (Mac system colour), light / dark | Code today |
| --- | --- | --- |
| `--ok` | green `#34C759` / `#30D158` | `#2C7A57` / `#5EB489` |
| `--warn` | orange `#FF9500` / `#FF9F0A` | `#A5761A` / `#D2A249` |
| `--danger` | red `#FF3B30` / `#FF453A` | `#B03A2A` / `#E0705A` |

The target column is **target (clean-up phase)** (settled 2026-10-02: status colours are the Mac
system green, orange and red). Each status colour keeps its `X-ink` (`--ok-ink`, `--warn-ink`,
`--danger-ink`) for text on it, chosen so every `X` / `X-ink` pair clears 4.5:1 in both schemes
(`crates/ds/tests/legibility.rs::every_ink_on_its_colour_is_legible_in_both_schemes`); the inks
are re-derived when the colours change. Where a system colour is too light for text on paper, the
text variant is a deeper step of the same hue, not a new hue.

## 4. The Space tint

A Space is one flat tint, drawn only as the sidebar ground (21-SPACES). It is `oklch(.936, k·.022,
h)` light and `oklch(.215, k·.018, h)` dark (about 40 % of the prototype's chroma, settled
2026-10-01 and kept), gamut-fitted, then lowered in chroma until neutral ink reads 4.5:1 and faint
ink 3:1 on it. There are no frame inks, no gradient and no per-stop derivation: the old section 4
(frame tokens, the verbatim derivation and the colour arithmetic) is in `archive/03-COLOR-arc.md`.
Status: **target (clean-up phase)** for the code, which still derives `--f-*` and a gradient.

## 5. Accent

The pickable accents are macOS's own eight, in its order: Blue (the default, systemBlue `#007AFF`
/ `#0A84FF`), Purple, Pink, Red, Orange, Yellow, Green, Graphite (the Mac's neutral grey accent,
`#8E8E93` / `#98989D`), each the system colour, ink white where white reads at 3:1 on the fill and
the deep ink of the hue otherwise (yellow, orange, green). Settled 2026-09-30. The accent is the
desktop's, set in Settings (the control centre has no accent picker, 30 section 3.4); a Space never
changes it. Postmark is removed; the old Candy-hue accents (Amber, Violet, teal Blue) are gone, and
stored `postmark`, `amber`, `violet` are not migrated. All eight are generated from one band
(section 20). The Postmark and Space-mode accent derivations and the contrast-gate table of the
prototype are in `archive/03-COLOR-arc.md`.

## 6. Contrast gates

Checks that run as tests (never shown in an editor); each has a floor.

| Check | Measured | Floor |
| --- | --- | --- |
| Sidebar text on the Space tint | ratio(`--ink`, tint); ratio(`--ink-soft`, tint) | 4.5 |
| Faint text on the Space tint | ratio(`--ink-faint`, tint) | 3.0 |
| Accent on the surface | ratio(accent, `--surface`) | 3.0 (the band aims for 4.5) |
| Ink on the accent tint | ratio(`--ink`, `--accent-soft`) | 4.5 |

The design system's `every-pair-legible` test covers every scheme x every accent (8) and every
Space preset on the sidebar ground (21 section 7).

## 6. Contrast gates

Four checks run for the current Space and theme; each has a floor.

| Check | Measured | Floor | Source |
| --- | --- | --- | --- |
| Sidebar text on the colour | min over stops of ratio(`--f-ink`, stop) | 4.5 | `S:1214` |
| Faint text on the colour | min over stops of ratio(`--f-ink-faint`, stop) | 3.0 | `S:1215` |
| Accent on the card | ratio(accent, `--surface`) | 3.0 | `S:1216` |
| Ink on the accent tint | ratio(`--ink`, `--accent-soft`) | 4.5 | `S:1217` |

The derivation guarantees the first two by capping and targets 4.5 for the third (stricter than
its 3.0 floor). The plan turns this into a sweep test: "every 5° of hue and every chroma step,
both themes, naming the exact colour that fails" (`S:924-925`), and the design system's
`every-pair-legible` test covers "2 schemes x 8 accents + every Space preset" (was 6 accents and the frame) (`P:366`).

## 7. Presets and default Spaces

Moved to `21-SPACES.md` section 4 (eight one-hue presets, grain 0). The prototype's dot lists are
in `archive/03-COLOR-arc.md`.

## 8. Grain

Grain is a 128 px grey-noise tile over the sidebar tint, its strength set per Space (0..100), and
**the default is none (0)** for new Spaces and every preset. The slider stays in the Space editor.
Opacity is `grain/100 × 0.10` light and `× 0.08` dark, drawn above the tint and below the sidebar's
content; never on a control or the content. The design system uses a pre-rendered 128x128 alpha
noise PNG from a Park-Miller PRNG with seed 7, multiplier 16807 and modulus 2147483647 (the
prototype's generator and overlay-blend notes are in `archive/03-COLOR-arc.md`, which Blitz cannot
do). The `--grain` token of `C` is unused and deleted. Status: the default is current (code default 0); the
sidebar-only placement stays **target (clean-up phase)**.

## 9. The Space editor field

Gone: the editor is preset swatches and one hue slider (21 section 6). The 2-D field's drawing
code is in `archive/03-COLOR-arc.md`.

## 10. Shadows

Four shadow tokens, and nothing else. Each is tied to one kind of surface; the rest of the old set
(`--shadow-1`, `--shadow-2`, the card, side-peek, peek, command menu, hover card, selection bubble,
floating composer, frame item, provider-mark and handle one-offs) is deleted or absorbed. The old
table is in `archive/03-COLOR-arc.md`. Settled 2026-10-02; the values below are the starting
values (window per 30 section 3.2, the others the nearest of the old set; tune in the gallery).

| Token | Used by | Light | Dark |
| --- | --- | --- | --- |
| `--shadow-window` | app windows | `0 10px 30px -10px rgba(0,0,0,.35)` | `0 10px 30px -10px rgba(0,0,0,.6)` |
| `--shadow-popover` | menus, popovers, hover cards, tooltips, the command menu, toasts: a hairline `0 0 0 1px rgba(0,0,0,.06)` plus `0 18px 40px -16px rgba(0,0,0,.45)` | as shown | hairline `rgba(255,255,255,.08)`, drop `.6` |
| `--shadow-sheet` | sheets, alerts, the side panel: `0 24px 50px -18px rgba(0,0,0,.55)` | as shown | `.7` |
| `--shadow-drag` | an item being dragged: `0 18px 30px -12px rgba(0,0,0,.40)` | as shown | `0 22px 34px -14px rgba(0,0,0,.8)` |

There is no card shadow (the window is flush, 30 section 3.4). Controls carry no shadow token: the
Mac push-button bezel (a 1 px inset highlight and a hairline) is the only one and is part of the
button, not a token. Material shadows in section 17 are these four. `--shadow-popover` is renamed
`--shadow-popover`; `--shadow-sheet` stays. Status: **target (clean-up phase)**; the code still
emits about twelve.

## 11. Washes

`S` mixes five tints at runtime with `color-mix`; the design system precomputes them because Blitz
lacks `color-mix` (`P:293-294`, `P:426`).

| Use | Formula | Design-system token | Source |
| --- | --- | --- | --- |
| OK pill | `color-mix(in oklab, var(--ok) 16%, transparent)` | `--ok-wash` | `S:277` |
| Failing pill | `color-mix(in oklab, var(--danger) 16%, transparent)` | `--danger-wash` (see open decisions) | `S:278` |
| Spoof flag | `color-mix(in oklab, var(--danger) 12%, var(--raise))` | `--danger-wash` (see open decisions) | `S:421` |
| Attachment warning | `color-mix(in oklab, var(--warn) 14%, var(--surface))` | `--warn-wash` | `S:696` |
| Destination ring (end) | `0 0 0 3px color-mix(in oklab, var(--accent) 35%, transparent) inset` | `--accent-ring` | `S:448` |

Precomputed values for these tokens are not specified.

## 12. Fixed literal colours

A few colours are literals in `S`; each needs a token in the port.

| Literal | Where | Source |
| --- | --- | --- |
| `#FFFFFF` | original-message frame ground (the sender's page); design-system name `--foreign-ground` (`P:293`) | `S:540-541` |
| `#fff` | text on coloured avatars and favicons; lying link pill text; editor handle border | `S:116`, `S:139`, `S:255`, `S:436` |
| `#FFFFFF` | provider mark ground | `S:552`, `S:559` |
| `#666` | fallback favicon colour for Today items when a Space has no pins | `S:1255` |
| `#f3f4f1` / `#1b1d1a` | editor field ground, light / dark | `S:1399` |

## 13. Identity colours

People, accounts and providers get a fixed colour each; these are data, not theme.

| Kind | Rule or values | Source |
| --- | --- | --- |
| Person (no stored colour) | `h = (h × 31 + charCode) mod 360` over the address, then `hsl(h, 38%, 42%)` | `S:1879` |
| Accounts (sample) | acme `#5B4FC4`, corp `#2F7F6E`, lists `#B0662E`, gm `#3C8A5B`, ic `#7A4A9E` | `S:1028-1030`, `S:1087-1088` |
| Pins (sample) | `#5B4FC4`, `#2F6FAE`, `#7A4A9E`, `#3E7F6C`; `#3C8A5B`, `#C0782E`, `#6D7A3A`, `#2E7F8C` | `S:1031`, `S:1089` |
| Provider letter colour | google `#1A73E8`, microsoft `#0F6CBD`, fastmail `#2A5DB0`, icloud `#3A82F7`, yahoo `#6001D2`, imap `#5D6660` | `S:1131-1136` |
| Provider mark | letter in the provider colour on a white chip, "never the provider's logo"; the icons mode shows the provider's own favicon, letters are the fallback | `S:1128-1129`, `S:1139-1147` |
| Draft favicon | `--ink` | `S:724` |
| Unselected account tile | avatar at `saturate(.55)` and opacity 0.85 | `S:551` |

Both backends paint `saturate()` now (FINDINGS "CSS `filter`"), so the unselected tile's `saturate(.55)` needs no precomputed stand-in beyond the opacity it already has (`P:75`, `P:421`).

**As tokens (settled, 2026-09-24).** The person hash is `ds::person_hue(address) -> PersonHue`
(painted by `AvatarTone::Person`, or `PersonHue::colour()`). The eight stored-colour swatches a
consumer hands out in order are `--c-person-1..8` (`ds::style::tokens::person::PersonSwatch`), mailo's `AVATAR` order:
`#5B4FC4`, `#2F7F6E`, `#B0662E`, `#3C8A5B`, `#7A4A9E`, `#C0782E`, `#2E7F8C`, `#6D7A3A` (the
accounts and pins above). They are data, declared once and the same in both schemes. A
consumer keeps no hex constants for people.

## 14. Usage map

Which token paints which element. The sidebar's rows are 14.1; everything else is on paper.

### 14.1 Sidebar

Moved to `archive/03-COLOR-arc.md` (the frame usage map). The sidebar is a Mac source list: its rows read `--ink`, `--ink-soft`, `--ink-faint`, selection `--sel-bg` or `--sel-bg-quiet` (30 section 3.2), on the Space tint.

### 14.2 Card

| Element | Colour | Source |
| --- | --- | --- |
| Page ground | `--paper` | `S:50` |
| Card, list, rows | `--surface` | `S:158`, `S:177` |
| Reader, composer page, peek, mini, kbd | `--surface-2` | `S:167`, `S:201`, `S:223`, `S:567` |
| Row hover | lift, `--shadow-2`, border `--line`; no background change | `S:183` |
| Row selected | bg `--raise`, border `--accent`, `--shadow-1` | `S:184` |
| Unread dot | `--accent` | `S:186` |
| Mini hover | text `--ink`, bg `--raise`, `--shadow-1` | `S:171` |
| Tool hover | text `--ink`, bg `--surface` | `S:206` |
| Strip button hover | bg `--accent-soft`, text `--ink` | `S:320` |
| Segmented / view switch pressed | bg `--ink`, text `--paper` | `S:268`, `S:539` |
| Menu and command item selected | bg `--accent-soft` (command also text `--ink`) | `S:240`, `S:670` |
| Menu check glyph | `--accent` | `S:790` |
| Search match | text `--accent`, weight 800 | `S:791` |
| Focus ring | `outline:2.5px solid var(--accent); outline-offset:2px; border-radius:4px` | `S:55` |
| Input focus | border `--accent` + `0 0 0 3px var(--accent-soft)` | `S:384`, `S:784` |
| Primary button | bg `--accent`, text `--accent-ink`, `--shadow-1` | `S:387` |
| Chip, command token, mention, selected block | bg `--accent-soft`, text `--ink` | `S:198-199`, `S:613`, `S:658`, `S:798` |
| Drop line | `--accent` | `S:610-611`, `S:752` |
| Links | `--accent` | `S:438`, `S:660`, `S:743` |
| Toast, send pill, link pill, fly | bg `--ink`, text `--paper` | `S:361`, `S:432`, `S:443`, `S:707` |
| Toast tab | bg `--paper`, text `--ink`; armed bg `--accent`, text `--accent-ink` | `S:366`, `S:369` |
| Lying link pill | bg `--danger`, text `#fff` | `S:436` |
| Star | off stroke `--ink-faint`; on stroke and fill `--warn`; sparks `--warn` | `S:302-306` |
| Draft state pip | saved `--ok`, dirty `--warn` | `S:577-578` |
| Live dot | `--ok` | `S:561` |
| Spoof flag | bg danger wash, glyph `--danger`; info flag bg `--accent-soft`, glyph `--accent` | `S:420-424` |
| Attachment warning | bg warn wash, glyph `--warn` | `S:696-697` |
| Avatars (reader, hover card) | bg `--ink`, text `--paper` | `S:210`, `S:409` |
| Event attendees | bg `--ink-soft`, text `--paper`, border `--surface` | `S:520` |
| Event month band, pressed RSVP | bg `--accent`, text `--accent-ink` | `S:513`, `S:523` |
| Quote rule | 2 px `--line`; composer blockquote 3 px `--ink` | `S:215`, `S:733` |
| Blocked image | `repeating-linear-gradient(135deg, var(--surface) 0 10px, var(--surface-2) 10px 20px)` | `S:492` |
| Floater `zZ` | `--accent` | `S:335` |

## 15. Candy hues

Moved to `archive/03-COLOR-arc.md` (history). The app-icon template's per-app gradient families are in `08-ICONS.md`.

## 16. S versus C

Moved to `archive/03-COLOR-arc.md` (history: Post versus Charm).

## 17. Materials

Shell surfaces are drawn in one of eight materials; a material is tint, edge, shadow and radius
over optional compositor blur. The plan fixes the type and the mechanism; it gives no starting
values.

### 17.1 The type and the mechanism

```rust
pub enum Material { Window, Bar, Dock, Popover, Sheet, Toast, Osd, Widget }  // slug(), blur(), ALL
pub enum BlurState { Available, Unavailable }  // data-blur=on|off -> --m-tint vs --m-tint-solid (alpha>=.94)
```

`P:315-316`

| Item | Rule | Source |
| --- | --- | --- |
| Tokens | `--m-tint`, `--m-tint-solid`, `--m-edge`, `--m-shadow`, `--m-radius` | `P:304` |
| Root | `div.ds[data-theme][data-accent][data-motion][data-material][data-blur]` (the Space tint on the sidebar ground only; the `--f-*` inline family is deleted, target (clean-up phase)) | `P:320` |
| Nested scope | `Surface(material, theme)` sets a material for a subtree without a stylesheet | `P:323` |
| Blur on | the surface paints `--m-tint` (translucent) and the compositor blurs behind it | `P:316` |
| Blur off | the surface paints `--m-tint-solid`, alpha at least 0.94 | `P:316` |
| Blur source | `ext-background-effect-v1` (cosmic-comp since 1.3.0; KWin 6.7.5 advertises it); needs an alpha buffer and a non-empty region; strength is the COSMIC theme's `frosted` setting | `P:152-155` |
| Blur region | owned by the host: rounded 1 px strips per corner row, capped at the radius; the design system exposes only `Material::blur()` intent | `P:514-516` |
| Fallback | opaque tinted material behind a flag if blur fails the spike | `P:527` |
| Test | `material-legible-over-black-and-white` | `P:369` |
| Gallery | Material (8) x Blur toolbar axes | `P:269` |

### 17.2 Starting values (proposed)

The plan file names the tokens; the values below are the design-system planning agent's
starting point (2026-09-23), to be tuned in the gallery. Every value here is **proposed**,
except the six tint alphas marked **settled (2026-09-24)** below, which the user fixed as the
sensible-default legibility floor over blur (FINDINGS "Bar gaps" and the tune wave). `hairline`
= 0.5 px `rgba(0,0,0,.08)` light / `rgba(255,255,255,.09)` dark (the `--f-line` values, section
4); `highlight` = `inset 0 1px 0 rgba(255,255,255,.6)` light / `rgba(255,255,255,.05)` dark (the
`--shadow-1` inset, section 9). Tint solid = the same tint at alpha .94 or above. Text on every
material must pass the `material-legible-over-black-and-white` test (`P:369`). Wave 1 measured
the translucent tints against it (the card's `--ink` at 4.5:1 over pure black and pure white,
`crates/ds/tests/legibility.rs`) and raised the four that fell short over an *opaque* ground by
the smallest .02 steps that pass: dark Bar .58 to .66 (was 3.63:1 over white), dark Dock .50 to
.66 (2.81:1), light Widget .50 to .54 (4.04:1 over black), dark Widget .45 to .65 (2.43:1).

Open decision 11 asked how those gates hold over *compositor blur*, where the tint is the whole
background rather than a wash over an opaque surface. Measured at wave 1's alphas
(`crates/ds/tests/legibility.rs`), six material/scheme pairs fell short over a pure black or
white backdrop, worst the light Widget at 3.91:1 over black. Settled (2026-09-24), the smallest
further .02 raises that clear 4.5:1 over blur for every pair: dark Bar .66 to .68, light Dock
.55 to .59, dark Dock .66 to .68, dark Osd .66 to .68, light Widget .54 to .60, dark Widget .65
to .67 (Bar light, Popover and Osd light already cleared it). This is a sensible default the
user asked to be tunable later, not a final measurement; `crates/ds/tests/legibility.rs`'s
`the_tinted_chrome_holds_its_ink_over_blur` pins the floor so a retune that drops back below 4.5
fails it.

**The contrast-relax pass (2026-09-26): `Material::Widget` alone gives up the 4.5:1 floor.** The
user's decision, "relax the contrast then": the light widget card may be as see-through as the
reference (design/23-WIDGETS.md section 1.1 M26-M30, measured alpha .48 of a near-white
`rgb(247,248,248)`, fitted over a blurred wallpaper), at the cost of the 4.5:1 small-text rule
over the worst-case black backdrop — accepted because the widget's own text is large or bold
(the hero and figure numerals, bold city names), where the accessibility guideline for large
text is 3:1, not 4.5:1. `crates/ds/tests/legibility.rs` now gates `Material::Widget` (and the
Space-tinted widget card, `CardTint::Space`) at 3:1 instead of 4.5:1, over black and white,
solid and blurred; every other material keeps the 4.5:1 floor. The light tint drops from .60 to
.48 (3.86:1 over black, 16.70:1 over white). The dark tint was never measured against the
reference, so it is raised only as far as the relaxed floor needs across every gate the tests
run — the flat tint alone would clear 3:1 at .53, but the Space-tinted card's darkest preset
stop needs .55 (worst 3.12:1 over white); dark moves from .67 to .55. Some widget text sits
below the WCAG large-text size even by this reading (design/23-WIDGETS.md section 4.3 lists it);
their sizes are unchanged, so they may be hard to read over very dark or very bright wallpapers.

| Material | Tint light | Tint dark | Edge | Shadow | Radius | Blur | Nearest precedent in `S` |
| --- | --- | --- | --- | --- | --- | --- | --- |
| Window | `--f-grad` + `.ds-layer` + `.ds-grain` | same | none | none | 0 (the card inside keeps 12/12/12/4) | none | `.win` (`S:77-87`) |
| Bar | `rgba(248,249,246,.70)` | `rgba(21,24,20,.68)` (settled 2026-09-24; .58 to .66 wave 1, to .68 over blur) | `inset 0 -0.5px 0 hairline` | none | 0 | behind | frame zone (section 4) |
| Dock | `rgba(248,249,246,.59)` (settled 2026-09-24; was .55) | `rgba(21,24,20,.68)` (settled 2026-09-24; .50 to .66 wave 1, to .68 over blur) | `inset 0 0 0 .5px rgba(255,255,255,.55)` + highlight | `0 10px 30px -10px rgba(0,0,0,.35)` | 22 | behind | pinned tiles radius 12 (`S:109`) |
| Popover | `rgba(255,255,255,.78)` | `rgba(42,47,40,.78)` | hairline + highlight | `--shadow-popover` = `0 18px 40px -16px rgba(0,0,0,.45)` | `--r-panel` 14 | behind | `.fmenu` (`S:665-666`) |
| Sheet | `rgba(248,249,246,.82)` | `rgba(21,24,20,.78)` | hairline + highlight | `--shadow-sheet` = `0 24px 50px -18px rgba(0,0,0,.55)` light / `0 30px 60px -20px rgba(0,0,0,.7)` dark | 18 | behind | `.peek` (`S:221-224`) |
| Toast | `rgba(248,249,246,.80)` | `rgba(21,24,20,.74)` | hairline + highlight | `--shadow-popover` | 16 | behind | `.toast` is inverse ink/paper in mail (`S:360-362`); the shell banner is a material, see open decision 12 |
| Osd | `rgba(248,249,246,.72)` | `rgba(21,24,20,.68)` (settled 2026-09-24; was .66) | hairline + highlight | `--shadow-popover` | 18 | behind | none |
| Widget | `rgba(247,248,248,.48)` (contrast-relax pass 2026-09-26, proposed: the reference's own measured near-white at its own measured alpha, design/23 section 1.1 M27-M28; was `rgba(224,224,224,.60)`, the vibrancy pass 2026-09-26; .50 to .54 wave 1, to .60 over blur; gated at 3:1 not 4.5:1, see above) + grain | `rgba(21,24,20,.55)` (contrast-relax pass 2026-09-26: raised only as far as the relaxed 3:1 floor needs; was .67, settled 2026-09-24; .45 to .65 wave 1) + grain | light: a 1 px white rim at .10 inside, a 1 px outer hairline (M32, M33); dark: hairline + highlight | light `0 2px 8px rgba(0,0,0,.12)` (M34); dark `0 8px 20px -8px rgba(0,0,0,.50)` | 20 | behind (sigma 22 recommended, design/23 section 4.3) | `.editor` / `.note` (`S:246`, `S:283`) |

**Target (clean-up phase).** Shell chrome (bar, dock, launcher, control center) is the material
alone: it no longer carries the workspace's `--f-*` frame tokens, and the Space tint reaches only a
sidebar's ground (`21-SPACES.md` section 3). The notes in this section about tinted roots, the
Space gradient and `appearance.material_tint_alpha` describe the code that goes.

### 17.3 Material per surface

| Surface | Material | Status |
| --- | --- | --- |
| App windows | Window | settled by name (`P:315`) |
| Bar | Bar | settled by name |
| Dock | Dock | settled by name |
| Bar and dock menus, tray menus | Popover | settled by name |
| Launcher panel, control center | Sheet | proposed (`P:` sill design: launcher is a centred fixed-size panel) |
| Notification banners | Toast | proposed |
| Notification center | Sheet | proposed |
| OSD | Osd | settled by name |
| Widgets, quick note | Widget | proposed |

### 17.3.1 Modal scrim (sheet and modal parts, 2026-09-25)

`--scrim` (black .22) pushes a peek or a palette back; behind a dialog that asks for a decision
(the power menu) it is too light. `--scrim-modal` is black at .40 in light and .55 in dark
(`ColourToken::ScrimModal`, `Scrim { strength: ScrimStrength::Modal }`, `Sheet { scrim }`).
Gated in `tests/legibility.rs`: the Sheet material's solid tint stands 3.05:1 off the modally
dimmed paper in light (1.89 under `--scrim`), more than under `--scrim` in dark (1.09 against
1.04, where the sheet's hairline and shadow carry the edge), and `--ink` reads 15.6 and 15.0:1 on
the sheet in both schemes.

### 17.3.2 Lock screen colours (M11, 2026-09-26; proposed)

The lock screen draws on the wallpaper, not on a card, so its colours are their own and the same
in both schemes: `--lock-ink` white (the time, the name, the field's dots), `--lock-ink-soft`
white .78 (the date, the hint, the placeholder), `--lock-glass` white .24 light and .18 dark (the
password pill, flat; no blur is assumed), `--lock-glass-strong` white .38 and .30 (the enter
button, a hovered pill), and `--lock-veil` black .12 light and .28 dark over the wallpaper so the
white type reads on a pale picture. The lock screen takes no Space tint (21 section 3); the `LockLook::Space` variant is history, target (clean-up phase).

### 17.3.3 Idle dim scrim (Q447, 2026-09-27)

sill's own idle service dims the screen with a full-screen overlay before it goes off, never
real brightness (design/22-SETTINGS.md section 3.24 `idle.dim_level_pct` 10..90,
`idle.dim_s`; sill FINDINGS "sill idle (Q420 B)"). `--scrim-idle` (`ColourToken::ScrimIdle`) is
that overlay's colour: opaque black, the same in both schemes, not `--scrim` (§17) or
`--scrim-modal` (§17.3.1), whose alphas are the fixed strengths of a menu backdrop and a modal
dialog. This one's strength is the person's own setting, not a fixed alpha the token can carry —
`IdleDim` (design/04 section 4.14, `ds::detail::use_idle_dim`) sets the overlay element's own
`opacity` from `idle.dim_level_pct` each frame it moves, so `--scrim-idle` stays a plain colour
and the level lives where the rest of this system keeps a per-instance value, in the component,
not the token table.

### 17.4 Material stack v2 (settled 2026-09-24, the macOS polish pass)

macOS stacks layers on every chrome material that the section 17.2 recipe lacked, and the shell
looked flat beside it. Stack v2 adds them to every card-like material (Dock, Popover, Sheet,
Toast, Osd, Widget) and a bottom hairline to the Bar; the Window is unchanged. The tints' alphas
(and the legibility gates on them) do not move. Each layer is a `--m-*` variable a material block
declares and `--m-box` lists outside in; each alpha reads an input the root writes from a
settings key (`ds::style::material::stack::MaterialStack`, `Ds { stack }`), with the default below behind it.

| Layer | Variable | Light | Dark | Key (proposed name, design/22) |
| --- | --- | --- | --- | --- |
| Vibrancy | baked into `--m-tint`, `--m-tint-solid` | OKLab chroma x1.4, lightness +.012 (surface `#f8f9f6` becomes `#fcfdf9`) | chroma x1.4, lightness kept (`#151814` becomes `#141813`, the raise `#2a2f28` becomes `#293026`) | `appearance.material_vibrancy`, percent, 100 (0 = section 17.2's flat colour, through `color-mix` on `--m-vibrancy`) |
| Outer hairline | `--m-hairline` | `0 0 0 .5px rgba(0,0,0,.14)` (the widget's 1 px, M32) | `0 0 0 .5px rgba(0,0,0,.60)` | `appearance.material_hairline_light` 14, `_dark` 60 |
| Contact shadow | `--m-shadow-contact` | `0 1px 2px rgba(0,0,0,.10)` | `0 1px 2px rgba(0,0,0,.30)` | `appearance.material_shadow_strength`, percent, 100 (scales both shadows) |
| Ambient shadow | `--m-shadow-ambient` (= `--m-shadow`) | menus, toast, OSD `0 12px 40px -12px rgba(0,0,0,.28)`; sheet `0 24px 60px -18px .40`; dock `0 10px 30px -10px .35`; widget `0 2px 8px .12` (M34, 2026-09-26; was `0 6px 16px -6px rgba(26,30,26,.3)`) | menus, toast, OSD `.55`; sheet `0 30px 70px -20px .65`; dock `.50`; widget `0 8px 20px -8px .50` | as above |
| Inner top highlight | `--m-highlight` | `inset 0 1px 0 rgba(255,255,255,.30)` | `inset 0 1px 0 rgba(255,255,255,.12)` | `appearance.material_highlight_light` 30, `_dark` 12 |
| Inner edge | `--m-edge` | `inset 0 0 0 .5px rgba(0,0,0,.08)` (the dock's `rgba(255,255,255,.55)`; the widget's `inset 0 0 0 1px rgba(255,255,255,.10)`, M33) | `rgba(255,255,255,.09)` | none (the `--f-line` values) |
| Bar | `--m-hairline`, `--m-edge` | `0 .5px 0 rgba(0,0,0,.14)` under `inset 0 -.5px 0 rgba(0,0,0,.08)` | `0 .5px 0 rgba(0,0,0,.60)` | the hairline keys |

**Pixel snapping (2026-09-25).** The .5 px widths in this table are `var(--hairline)` and the
1 px highlight `var(--hair)` in the generated sheet (01-LAYOUT §2.1): identical at 1x and 2x, one
device pixel at 1.25, 1.5 and 1.75, where .5 px would be a 0.6-0.9 device pixel smear.

Why the tint carries the boost: Blitz neither blurs nor saturates what is behind a surface
(FINDINGS S15, S16), so macOS's vibrancy cannot be reproduced; brightening and saturating the
tint itself is the part that can. A tinted root (bar, dock, popover panel, OSD, widget) paints the
Space gradient instead of the tint, which carries its own hue; its frame redraws the inner edge
and highlight (`--m-inner`) over the gradient. The gates hold: `tests/legibility.rs` measures the
boosted tints, over blur and solid, black and white, and all pass.

A squircle corner (`Corner::Squircle(r)`, design/08 section 2.1's `n = 5` superellipse
reaching `2 r` along each edge) is drawn with a `mask-image`, and a mask clips the element's own
`box-shadow`; so a squircle material paints its tint on a masked `::before` and keeps the stack on
its own unmasked box at `--r-squircle`, the circle (about .884 r) the squircle touches at 45
degrees and never leaves by more than .03 r.

## 18. SpaceLook per workspace

Moved to `21-SPACES.md` (the model, now one hue, a chroma factor, grain and a theme).

## 20. Accent band (settled: B, Airy, 2026-09-27)

**Status: settled (B, 2026-09-27).** The user picked band B, Airy, from the three candidates
below. Every accent in quire (now the eight Mac ones, section 5) is generated by the band; Postmark's
navy quad and mailo's `oklch(aL, .045 + .035k, h0)` Space accent derivation are retired from quire
(`archive/03-COLOR-arc.md`). The user's note that started it: Postmark
`#23508F` (OKLCH L .434, C .115) is "too heavy, need to be more light and translucent … make
some constraints on the accent color. mailo already has this."

### 20.1 The constraint

Every accent, built-in or lent by a Space, comes from one function,
`ds::tokens::accent_band::accent_roles(&BAND, AccentPick { hue, weight }, scheme) -> AccentRoles`
(`ds::accent_of(accent, scheme)` for a built-in accent; `ds::derive(dots, scheme).accent_roles`
for a Space's),
the way mailo derives the Space accent (section 4.3): lightness is the band's, the person picks
only the hue (and a Space lends its dot's chroma as the weight, so a grey Space lends a
grey-blue). A band fixes per scheme: the fill's starting lightness, the chroma span (quiet at
weight 0, the ceiling at full weight), the ink rule, the text accent's starting lightness, and
the wash's and ring's starting alphas. The function then steps, as mailo's accent loop does:

| Role | Paints | Rule | Gate (`accent_band::floors`) |
| --- | --- | --- | --- |
| `fill` (solid) | primary button, today disc, toggle on, tile disc, slider fill, armed toast tab | OKLCH (band L, chroma at weight, hue), gamut-fitted; L steps .01 toward the ink until the ink reads | ink on fill >= 4.5 |
| `ink` | text and glyphs on the fill | `White`: `#FFFFFF`, the fill steps darker; `Deep`: OKLCH (.22, .03, hue), the fill steps lighter | (as above) |
| `text` | links, the month title and busy dots, menu check and marks, search match, caret, unread dot, selected-row border, drop line, spinner ring, input focus border, tab indicator | same hue and chroma; L steps away from the card (darker in light, lighter in dark) | >= 4.5 on `--paper`, `--surface`, `--surface-2`, `--raise` and on the wash over each (20.6) |
| `text_material` | the same marks on a translucent Popover, Sheet or Toast (a root of one points `--accent-text` at it) | as `text` | >= 4.5 on every `text` ground, and on those three materials over a black and a white backdrop and the wash over them (20.6) |
| `wash` (translucent) | selected row, menu and launcher highlight, chip, tile on, input focus halo | the fill at an alpha over whatever lies beneath (the Mac's selection); the alpha rises .01 only until it shows | `--ink` on the wash >= 4.5 over every ground; wash >= 1.15:1 off its ground; alpha <= .60 |
| `ring` | keyboard focus ring (3 px, 1 px gap, design/27 6.4) | the card's text accent at an alpha; rises .05 until it stands off | >= 3:1 against every card ground (WCAG 1.4.11) |

No text on the accent qualifies for the 3:1 large-text floor (button labels, the disc's number
and menu text are all under 18 pt / 14 pt bold), so the ink gate is 4.5 everywhere. The fill's
own contrast against the ground is reported, not gated: a toggle's knob and a disc's shape also
show the state (20.4). The sweep `tokens::accent_band::tests::every_hue_in_the_band_is_legible`
runs every 5 degrees of hue x weights 0 to 1 in tenths x both schemes (1,584 cases; 4,752 while
three candidates were swept) and names the colour that fails; mailo's `every_pick_is_legible` is
its model, and it now composites the wash over the card before measuring the ink on it.

A consequence worth knowing: white ink at 4.5:1 pins a white-ink fill to OKLCH L .55-.59 at any
hue (the Mac's `#007AFF`, L .60, carries white at only 4.02:1). A lighter solid needs dark ink
(band B, and C in dark). The lightness the user sees therefore comes mostly from the chroma and
the wash: the big areas (selected rows, menu and launcher highlight, tiles, chips) become a
translucent tint instead of an opaque one.

### 20.2 Candidates (B picked)

The built-in accents take the Candy hues (Red 28, Amber 76, Green 147, Violet 294), Postmark 257
and Blue 215 (20.5). Numbers below are for hue 257 at full weight, then the range over the whole
sweep. A and C are kept as the record of what was not picked; only B is in the code
(`accent_band::BAND`).

| | Postmark (today) | A · System | B · Airy | C · Calm |
| --- | --- | --- | --- | --- |
| Idea | navy, opaque tint | the Mac's system-blue family: vivid mid-light fill, light washes | pastel fills with a deep ink, dusty text accent | calmer chroma; light as A, dark as B |
| Light: band | fixed quad | fill L .62, C .04-.19, White, text L .58, wash .16 | fill L .80, C .03-.10, Deep, text L .54, wash .18 | fill L .64, C .04-.14, White, text L .56, wash .18 |
| Light: fill / ink | `#23508F` / `#F4F8FF` 8.0:1 | `#1673E4` / white 4.56 | `#94C0FE` / `#111B28` 9.3 | `#3C77C8` / white 4.50 |
| Light: text | `#23508F` | `#0067D5` | `#426AA2` | `#2D68B8` |
| Light: wash | `#DCE5F3` solid | fill at .16 | fill at .32 | fill at .18 |
| Light: ring | `#23508F` solid 2.5 px | text at .75 | text at .80 | text at .75 |
| Light: fill vs ground | 7.1 | 3.8 | 1.6 | 3.8 |
| Dark: band | fixed quad | fill L .62, C .04-.17, White, text L .72, wash .26 | fill L .78, C .03-.10, Deep, text L .76, wash .20 | fill L .74, C .04-.12, Deep, text L .74, wash .22 |
| Dark: fill / ink | `#7FA6E6` / `#0B142A` | `#2975D9` / white 4.53 | `#8EBAF7` / `#111B28` 8.7 | `#79ADF6` / `#111B28` 7.5 |
| Dark: text | `#7FA6E6` | `#66A6FE` | `#88B3F0` | `#79ADF6` |
| Dark: wash / ring | `#1E2A44` solid / solid | .26 / .65 | .20 / .60 | .22 / .60 |
| Sweep: fill L light / dark | | .55-.59 / .55-.59 | .80 / .78 | .55-.58 / .74 |
| Sweep: wash alpha light / dark | | .16 / .26 | .29-.36 / .20 | .18 / .22 |
| Sweep: ring alpha light / dark | | .70-.80 / .60-.70 | .75-.80 / .55-.60 | .75-.80 / .60-.65 |
| Sweep: fill vs ground light / dark | | 3.77-3.97 / 2.89-3.04 | 1.50-1.64 / 6.5-7.1 | 3.77-3.97 / 5.6-6.3 |

Every candidate clears every gate at every hue and weight (least: ink on fill 4.50, text 4.50,
ink on wash 7.3, ring 3.00). B's light fill stands only 1.5:1 off white: the button and the
toggle read by their shape and knob, not their colour, which WCAG 1.4.11 allows only as long as
the state is not shown by colour alone.

The candidate sheets (proposal commit): `tools/progress/shots/accent/accent-candidates-{light,dark}.png`
and one crop per column. The settled sheet (`ds-gallery --accent-sheet DIR`,
`crates/ds-gallery/src/accent_sheet.rs`) draws B alone, per scheme, over the calm wallpaper and a Popover, Widget and Sheet material:
the primary button, a toggle on, the segmented control (it selects in `--ink`, unchanged by any
band), a chip, a link, the focus ring, a menu highlight, a selected row, the control center's
tiles, the compact calendar (month title, today disc, dots), the launcher's selected row, and the
six built-in hues: `tools/progress/shots/accent/accent-b-final-{light,dark}.png`.

### 20.3 What changed with the switch (2026-09-27)

| Before | After |
| --- | --- |
| `tokens::AccentQuad { accent, ink, soft, seal }`, `ds::quad(accent, scheme)` | `ds::AccentRoles { fill, ink, text, wash: Alpha, ring: Alpha }`, `ds::accent_of(accent, scheme)`; `wash_colour()` and `ring_colour()` give the CSS colours |
| `--accent` the navy, also used as text | `--accent` the fill only |
| `--accent-soft` an opaque hex tint | `--accent-soft` = `rgba(fill, wash)`, translucent |
| `--accent-ring` = `color-mix(in srgb, var(--accent) 35%, transparent)` in the root | `--accent-ring` = `rgba(text, ring)`, written per accent block and per Space |
| (no token) | `--accent-text`, `ColourToken::AccentText` |
| `data-accent` blocks set 4 properties | they set 6: `--accent`, `--accent-ink`, `--accent-soft`, `--accent-text`, `--accent-ring`, `--seal` |
| focus outline `solid var(--accent)` | `solid var(--accent-ring)` |
| `Palette { accent, accent_soft, accent_ink }` from `oklch(aL, .045 + .035k, h0)` | the same three strings from the band, plus `accent_text`, `accent_ring` and `accent_roles: AccentRoles`; `accent_soft` is `rgba(...)` |
| `FrameVars.accent: Option<[String; 3]>` | `Option<AccentRoles>`; the root writes the six properties |
| `Card { accent, accent_soft, accent_ink }` literals on `POST_LIGHT`/`POST_DARK` | removed: the card keeps `surface` and `ink`; Postmark is `accent_of(Accent::Postmark, scheme)` |
| editor readout "Accent on the card" measured `--accent`, "Ink on the accent tint" the opaque tint | measures `--accent-text` (4.5), and the ink on the wash laid over `--surface` |

Every text or thin-mark use of `--accent` inside quire moved to `--accent-text`: `.ds-menu mark`,
`.ds-menu-filter-caret`, `.ds-menu-trail .ds-ic`, the dropdown's checked `.ds-ic`,
`.ds-hovercard-flag[data-tone=info] .ds-ic`, `.ds-drop-place[data-drop=accepts]` border,
`.ds-polkit-details-word`, `.ds-settings-row-trail[data-mark=check]`,
`.ds-bubble-button[aria-pressed=true]`, `.ds-run-link` underline and hover, `.ds-month-title`,
`.ds-month-dot`, `.ds-row[aria-selected=true]` border, the row's unread dot, `.ds-drop-line`,
the sidebar item's drop ring, the spinner's ring, `.ds-input[data-variant=boxed]:focus` border,
the bare input's caret (`caret-color` and `.ds-input-caret::before`), the tab indicator, and the
`dest` keyframe's start ring. Fills keep `--accent`: the primary and pressed mini button, the
toggle, the today disc, the module tile's disc, the slider fill, the armed toast tab.

### 20.4 State the fill alone cannot carry

B's light fill stands 1.5-1.6:1 off white, so no state may rest on the fill's colour alone.

| Where | Carried by | Status |
| --- | --- | --- |
| Today disc (`MonthGrid`) | the disc's shape (no other day has one) and the number in `--accent-ink`, weight 700 | holds |
| Toggle on | the knob's position (12 px right); the off track also has a `--line` border the on track drops | holds |
| Primary button | its label in `--accent-ink` (9:1), its `--shadow-1` edge and shape; primary is a role, not a state | holds |
| Selected segment (`SegmentedControl`) | `--ink` with `--paper` text; does not use the accent | holds |
| Mini button pressed | was fill only (border transparent) | fixed: the pressed border is `--accent-text` (at least 4.5:1) |
| Module tile on | was the disc's fill and the plate's wash | fixed: the on disc gets an inset `--hair` ring in `--accent-text`; the status line ("Home", "Off") also says it |
| Toast tab armed | was fill only, `--paper` to `--accent` | fixed: an inset `--hair` ring in `--accent-text` |
| Slider fill | the fill's length and the knob | holds |
| Selected list row (mailo's card) | `--raise`, `--shadow-1` and a border | the border moved to `--accent-text` |
| Menu and launcher highlight, chip, selected row wash | a wash behind `--ink` text; the highlight is also the keyboard position | colour only by nature (the Mac's too); the wash is gated to show at 1.15:1 |
| Primary button, disabled (the power menu's default Shut Down) | was the fill at .35 only | fixed: an `--accent-text` edge (20.6) |
| sill's dock tile progress ring (`conic-gradient(var(--accent) …)`) | the arc's length | sill's; its arc stands 1.5:1 off a light track, see sill's list |

### 20.5 Blue moves to 215

Inside the band, Candy's Blue (260) came out 0.005 (OKLab) from Postmark (257): the same swatch
twice in the picker. Blue now generates from 215, a sky blue (`#69CEE6` light, `#62C8DF`
dark). It stands 0.07 from Postmark and 0.11 from Green, at least the picker's narrowest
existing gap (Violet to Postmark, 0.064); `accent_band::tests::every_built_in_swatch_is_distinct`
holds every pair at 0.04 or more in both schemes (2026-09-30: the eight macOS accents; Apple's own Red and Pink stand 0.045 apart in light). Both slugs stay, so stored settings load
unchanged; a stored `blue` now shows the sky blue. The hue strip at the foot of
`accent-b-final-{light,dark}.png` shows the six.

### 20.6 Text on the wash and on the materials (2026-09-27, sill Q410, Q411)

The first cut gated the text accent on the card's four opaque grounds only. Two places it is
read were not among them, and both fell under 4.5 in sill's captures:

- **The wash.** A menu's or the launcher's match highlight (`.ds-menu mark`) sits on the
  selected row's `--accent-soft`. `#426aa2` on the wash over `--paper` (`#d5e3f5`) is 4.23:1
  (Q410).
- **The materials.** PolkitPrompt's "Details" sits on a Sheet, 82 % tint over whatever lies
  behind. `#426aa2` is 4.28:1 on the on-screen composite (`#e5e3db`) and 3.53:1 over black
  (`#cfcfcc`, sill's surface capture) (Q411).

The gate now composites every ground a screen can show before measuring
(`accent_band::text_grounds`): the card's four grounds; the wash over each; the three materials
that carry accent text (Popover, Sheet, Toast: `accent_band::TEXT_MATERIALS`) at their default
tints over the two reference backdrops, black and white (the same two `tests/legibility.rs`
holds their own ink over); and the wash over those. The text steps (.01) until it reads on all
of them, every 5 degrees of hue, weights 0 to 1, both schemes.

**Two text accents, not one.** No single value clears both kinds of ground and stays band B:

| Postmark (h 257) | Card grounds + wash | + materials over black and white, + wash |
| --- | --- | --- |
| Light | `#396198` (L .50; was `#426aa2`) | `#295086` (L .43), Postmark's old navy weight (`#23508F`, L .434) that the band was picked to leave |
| Dark | `#8ebaf7` (L .78; was `#88b3f0`) | `#d5e6fe` (L .92), nearly the ink |

So the band returns both. `text` (`--accent-text`) is gated on the card's grounds and the wash
over them; it paints windows and cards, where the ground is opaque and known. `text_material`
(`--accent-text-material`) is gated on every ground above; a `Ds` root or `Surface` of a
Popover, Sheet or Toast writes `--accent-text:var(--accent-text-material)` inline, so every
component inside it (menu marks and checks, the launcher's match, PolkitPrompt's "Details", a
banner's link) takes it with no change of its own. A separate `--accent-text-on-wash` was
considered and rejected: on the card it would spare the text only .04 of lightness in light and
.01 in dark, and on the materials it is the backdrop, not the wash, that moves the text most
(.54 to .43 in light, .76 to .89 in dark, before the wash's last .01-.04). The fills and washes
are unchanged.

Sweep ranges (every hue and weight):

| | `text` L | `text_material` L | ring alpha | least ratios (text on card / on wash; material text on card / on material / on washed material) |
| --- | --- | --- | --- | --- |
| Light | .47-.50 (was .54 start) | .42-.44 | .70-.75 | 5.18 / 4.50; 5.69 / 4.62 / 4.50 |
| Dark | .77-.79 (was .76 start) | .91-.93 | .55 | 6.74 / 4.50; 7.11 / 5.20 / 4.50 |

(L is the OKLab lightness of the gamut-fitted colour.)

One ground is left out, and the test pins it: the dark Popover's wash over a white backdrop,
where the card's own `--ink` reaches only 4.14:1. The accent cannot be asked to read where the
ink does not; the shortfall is the dark Popover tint's (.78) over a white backdrop, recorded
for the material pass. The ring stays gated on the card's grounds (3:1), now from the card's
text: light .70-.75 (was .75-.80), dark .55 (was .55-.60).

The measured grounds, before and after:

| Q | Where | Ground | Before (`#426aa2`) | After |
| --- | --- | --- | --- | --- |
| Q410 | launcher match on the selected row (a Sheet) | `#d5e3f5` | 4.23 | 6.24 (`#295086`) |
| Q410 | a menu match on a selected row in a window card | wash over `--paper` | 4.23 | 4.83 (`#396198`) |
| Q411 | PolkitPrompt "Details", on screen | `#e5e3db` | 4.28 | 6.32 (`#295086`) |
| Q411 | PolkitPrompt "Details", over black | `#cfcfcc` | 3.53 | 5.20 (`#295086`) |

**A disabled default button** (sill Q413). The Mac keeps a disabled default button's shape and
edge at the reduced opacity. Band B's pastel fill at .35 was barely apart from the disabled
Secondary beside it, the state carried by the fill's colour alone (20.4). A disabled Primary
now keeps an `--accent-text` edge (`button.css`), as the pressed Mini and the armed toast tab
do.

## Open decisions

The frame-era items (static frame fallback, grain blend, the six accents, `--seal`, the editor
handle in dark, frame over blur, default preset per workspace, per-Space theme, the destination
token) are settled or void under the flat-tint decision and are in `archive/03-COLOR-arc.md`.
Still open:

1. **Danger wash.** The prototype has two danger mixes, 16 % over transparent and 12 % over
   `--raise`; one `--danger-wash` is planned. Not resolved.
2. **Washes' precomputed values** for both schemes are not specified.
3. **Every material value** in section 17.2 (tint, tint solid, edge, shadow, radius for all eight
   materials) is proposed, not measured; tune in the gallery. Whether the shell's notification
   banner is a translucent Toast material or mail's inverse ink-on-paper toast is open. The
   material tints still carry the greenish values landed before the Look was one (30 section
   3.2); re-deriving them from the grey neutrals is open.
4. **Status colours** (section 3): the exact `X-ink` and text variants for the Mac green, orange
   and red are derived when the code lands, by the 4.5:1 gates.
5. **Identity colour source.** Settled (2026-09-24): a stored colour is one of the eight
   `--c-person-*` swatches, handed out in order; a person with none takes the person hash
   (section 13).

## Sources

- `S` `~/mailo-design/mailo-spaces.html`: paper tokens `S:7-46`; frame fallback `S:77-79`;
  frame usage `S:89-153`, `S:344-347`, `S:447-454`, `S:548-556`; card usage `S:155-799`;
  notes on Arc's numbers and lightness `S:907-926`; colour arithmetic `S:962-984`; derivation
  `S:986-1021`; Spaces and presets `S:1026-1119`; providers `S:1128-1147`; grain `S:1161-1169`;
  paint and checks `S:1171-1225`; editor `S:1392-1472`; person hash `S:1879`.
- `C` `~/mailo-design/mailo-charm.html`: tokens and candy shelf `C:13-47`; dark blocks
  `C:87-170`; seal `C:315-319`; Candy look and warmth `C:776-966`; scrim and command wrap
  `C:1054-1060`.
- `P` `~/.claude/plans/vast-toasting-peach.md`: context `P:18-19`; mailo accents `P:59`; filter
  `P:75`; COSMIC blur `P:152-155`; gallery `P:269`; grain PNG `P:288`; token families
  `P:291-304`; SpaceLook and Material `P:313-323`; legibility tests `P:366-369`; Blitz risks
  `P:418-426`; blur regions `P:514-516`; spike blur `P:527`; desktop Spaces `P:792-796`; app
  icons `P:810-825`; Appendix A3 `P:984-1026`; A8 `P:1486-1491`.
