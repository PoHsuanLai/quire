# 07 Looks

Status: draft for review, 2026-09-23. `S` = `~/mailo-design/mailo-spaces.html` (newest; wins
every conflict), `C` = `~/mailo-design/mailo-charm.html`, `B` =
`~/mailo-design/mailo-charm.bak.html`. `S:123` means line 123 of S. "The plan" means
`~/.claude/plans/vast-toasting-peach.md`.

## 1. What this governs

A *look* is one set of values for colour, radius, shadow and motion curves that the whole
interface is drawn with: "One shell, three motion dialects. Tokens are the whole trick: a 'look'
swaps colour + radius + shadow + easing, nothing else" (C:6-10). This file records the four looks
C defines (Post, Riso, Tide, Candy) with every light and dark token, their radii, shadows,
motion overrides and exit animations; the warmth axis C layers on Candy; how S relates to Post;
B's Mochi as history; and which look the desktop ships. Markup, layout and behaviour never change
between looks; only these values do. How a Space colours the frame around a look is
`03-COLOR.md` and `21-SPACES.md`; motion tokens are defined in full in `05-MOTION.md`.

## 2. The look model

| Axis | Values | Where it is set | Source |
| --- | --- | --- | --- |
| Look | `post` (the `:root` default), `riso`, `tide`, `candy` | `body[data-look]` | C:13, C:50, C:67, C:777 |
| Theme | auto (follows the system), light, dark | `:root[data-theme]`; dark tokens also under `@media (prefers-color-scheme: dark) :root:not([data-theme="light"])` | C:88-151, C:1482-1486 |
| Warmth | `cool` (the Candy base), `neutral`, `warm`, `paper`; Candy only | `body[data-warm]` | C:828-924 |
| Motion level | `calm`, `standard`, `extra` | `body[data-motion]` | C:172-176 |

C boots Candy, warm, and Standard (Calm when the system asks for reduced motion) (C:1465-1467).
The reference screenshots `reference-post-light.png` / `reference-post-dark.png` show Post with
C's red Postmark accent.

Tokens that every look sets: the neutrals `--paper --surface --surface-2 --raise --ink
--ink-soft --ink-faint --line --line-soft`; the accent quad `--accent --accent-ink --accent-soft`
plus `--accent-2` (declared by every look, **used by no rule** in C or S) and `--seal` (the
current-place dot, C:317); the status colours `--ok --warn --danger`; the shadows `--shadow-1
--shadow-2 --shadow-drag`; `--grain` (declared, **used by no rule**, A8 #8); radii; motion.

## 3. Post

"Post" is the paper-and-ink look: sage neutrals, a stamp-coloured accent, the flap corner on
cards (`12px 12px 12px 4px`), an inset white highlight over a soft drop shadow.

### 3.1 Colour (C's Post)

| Token | Light | Dark |
| --- | --- | --- |
| `--paper` | #E9ECE6 | #151814 |
| `--surface` | #F8F9F6 | #1D211B |
| `--surface-2` | #F1F3EE | #232722 |
| `--raise` | #FFFFFF | #2A2F28 |
| `--ink` | #1A1E1A | #E7EBE3 |
| `--ink-soft` | #586057 | #A0A79B |
| `--ink-faint` | #676E65 | #8A9284 |
| `--line` | #D6DBD0 | #333A30 |
| `--line-soft` | #E3E7DE | #282E26 |
| `--accent` | #C0402A | #E9684B |
| `--accent-ink` | #FFF6F3 | #1A0F0B |
| `--accent-soft` | #F2DCD5 | #3A201A |
| `--accent-2` | #2E5AA6 | #7FA6E6 |
| `--ok` | #2C7A57 | #5EB489 |
| `--warn` | #A5761A | #D2A249 |
| `--danger` | #B03A2A | #E0705A |
| `--seal` | #C0402A | #E9684B |
| `--wash` | not declared (the hero falls back to `--accent-soft`, C:201) | not declared |
| `--grain` | .035 | .05 |

Sources: light C:14-19, C:34; dark C:91-99 and C:123-131 (identical blocks, one per theme
mechanism).

### 3.2 Shape and shadow

| Token | Value | Source |
| --- | --- | --- |
| `--r-card` | `12px 12px 12px 4px` | C:30, S:16 |
| `--r-btn` | `9px` | same |
| `--r-pill` | `999px` | same |
| `--r-panel` | `14px` | same |
| `--r-chip` | `6px 6px 6px 2px` | same |
| `--shadow-1` light | `0 1px 0 rgba(255,255,255,.7) inset, 0 1px 2px rgba(26,30,26,.10)` | C:31, S:13 |
| `--shadow-2` light | `0 1px 0 rgba(255,255,255,.7) inset, 0 6px 16px -6px rgba(26,30,26,.30)` | C:32, S:14 |
| `--shadow-drag` light | `0 18px 30px -12px rgba(26,30,26,.40)` | C:33 |
| `--shadow-1` dark | `0 1px 0 rgba(255,255,255,.05) inset, 0 2px 4px rgba(0,0,0,.45)` | C:96, S:32 |
| `--shadow-2` dark | `0 1px 0 rgba(255,255,255,.05) inset, 0 10px 22px -8px rgba(0,0,0,.7)` | C:97, S:33 |
| `--shadow-drag` dark | `0 22px 34px -14px rgba(0,0,0,.8)` | C:98 |

### 3.3 Motion and exits

The base motion set (`05-MOTION.md#31-base-tokens-per-look-motion-level-standard`): t-tap 90,
t-quick 170, t-move 250, t-big 420, t-ambient 5s; `--e-out (.22,.9,.30,1)`, `--e-spring
(.34,1.42,.52,1)`, `--e-exit (.55,0,.75,.2)`; overshoot 1.04, squish .955, lift -2px, tilt
2.2deg, stagger 26ms (C:37-42). Exits: archive `fold`, trash `crumple`, snooze `curl`
(C:712-715).

## 4. Riso

Named for risograph print: lavender paper, one saturated violet-blue ink, hard offset shadows
with no blur, big radii, the fastest and springiest motion.

### 4.1 Colour

| Token | Light | Dark |
| --- | --- | --- |
| `--paper` | #F1EDFA | #150F28 |
| `--surface` | #FFFFFF | #1E1636 |
| `--surface-2` | #F8F4FF | #251C42 |
| `--raise` | #FFFFFF | #2C2250 |
| `--ink` | #231B3C | #F1EBFF |
| `--ink-soft` | #5B5083 | #B4A9D8 |
| `--ink-faint` | #655C90 | #9089B0 |
| `--line` | #DCD2F2 | #352B57 |
| `--line-soft` | #EBE4FA | #2A2148 |
| `--accent` | #4A37D9 | #7C6BFF |
| `--accent-ink` | #F2F0FF | #0E0A2A |
| `--accent-soft` | #E0DBFF | #2A2360 |
| `--accent-2` | #00A094 | #3AD8C6 |
| `--ok` | #00897E | #3AD8C6 |
| `--warn` | #A66300 | #F0B03C |
| `--danger` | #C0392E | #FF8A80 |
| `--seal` | #5B3FA8 | #9E7BFF |
| `--grain` | 0 | not declared: Post's dark `.05` wins (section 8) |

Sources: light C:51-56, C:60; dark C:102-107, C:134-139.

### 4.2 Shape and shadow

| Token | Light | Dark | Source |
| --- | --- | --- | --- |
| `--r-card` | 18px | same | C:57 |
| `--r-btn` | 14px | same | C:57 |
| `--r-panel` | 22px | same | C:57 |
| `--r-chip` | 999px | same | C:57 |
| `--r-pill` | 999px (inherited) | same | C:30 |
| `--shadow-1` | `2px 2px 0 #5B3FA8` | `2px 2px 0 #9E7BFF` | C:58, C:107 |
| `--shadow-2` | `4px 4px 0 #5B3FA8` | `4px 4px 0 #9E7BFF` | C:58, C:107 |
| `--shadow-drag` | `8px 10px 0 rgba(91,63,168,.5)` | `8px 10px 0 rgba(158,123,255,.55)` | C:59, C:108 |

### 4.3 Motion and exits

Overrides (C:61-63): t-tap 80, t-quick 150, t-move 230, t-big 460; `--e-spring
(.34,1.8,.5,1)`; overshoot 1.09, squish .90, lift -4px, tilt 3.6deg. Inherited from Post:
t-ambient 5s, `--e-out`, `--e-exit`, stagger 26ms. Exits: archive **`yeet`** (`--t-big`
`--e-exit`, C:738-743), trash `crumple`, snooze `curl`.

## 5. Tide

Sea-green neutrals and ink, long soft shadows, the slowest motion, no spring and no tilt, and one
exit for everything.

### 5.1 Colour

| Token | Light | Dark |
| --- | --- | --- |
| `--paper` | #E4EBE9 | #0B1415 |
| `--surface` | #F3F8F7 | #111F21 |
| `--surface-2` | #EAF1EF | #152629 |
| `--raise` | #FBFDFC | #1B2F32 |
| `--ink` | #122220 | #E1ECEA |
| `--ink-soft` | #4C5E5A | #91A5A1 |
| `--ink-faint` | #5F716C | #7E918D |
| `--line` | #CAD8D4 | #1D3235 |
| `--line-soft` | #DCE7E4 | #17282B |
| `--accent` | #1B7466 | #4FBBA4 |
| `--accent-ink` | #F0FBF8 | #05201B |
| `--accent-soft` | #D2E7E1 | #123531 |
| `--accent-2` | #C4832A | #E0A559 |
| `--ok` | #1B7466 | #4FBBA4 |
| `--warn` | #C4832A | #E0A559 |
| `--danger` | #A9462F | #D2705A |
| `--seal` | #1B7466 | #4FBBA4 |
| `--grain` | 0 | not declared: Post's dark `.05` wins (section 8) |

Sources: light C:68-73, C:78; dark C:111-115, C:143-147. `--ok` equals `--accent` in Tide.

### 5.2 Shape and shadow

| Token | Light | Dark | Source |
| --- | --- | --- | --- |
| `--r-card` | 14px | same | C:74 |
| `--r-btn` | 11px | same | C:74 |
| `--r-panel` | 18px | same | C:74 |
| `--r-chip` | 999px | same | C:74 |
| `--shadow-1` | `0 2px 10px -6px rgba(18,34,32,.5)` | `0 2px 12px -6px rgba(0,0,0,.8)` | C:75, C:116 |
| `--shadow-2` | `0 14px 34px -18px rgba(18,34,32,.65)` | `0 16px 38px -18px rgba(0,0,0,.9)` | C:76, C:117 |
| `--shadow-drag` | `0 26px 48px -20px rgba(18,34,32,.55)` | `0 28px 52px -20px rgba(0,0,0,.85)` | C:77, C:118 |

### 5.3 Motion and exits

Overrides (C:79-84): t-tap 140, t-quick 260, t-move 420, t-big 720, t-ambient 7s; `--e-out` and
`--e-spring` both `(.25,.7,.25,1)` (no overshoot anywhere); `--e-exit (.4,0,.6,1)`; overshoot
1.0, squish .985, lift -1px, tilt 0deg, stagger 44ms. Exits: archive, trash and snooze all
**`dissolve`** 620ms `--e-exit` forwards (C:731-737), which blurs (`filter: blur(5px)`) and
therefore cannot be painted by Blitz on the default backend (`05-MOTION.md#11-excluded-keyframes`).

## 6. Candy

"greyscale chrome, candy only where colour carries state" (C:776). Cool grey neutrals, one blue
for "you, here, now", five candy hues spent only on facts, the biggest radii, bottom-heavy
shadows, wobbly contact springs.

### 6.1 Colour (the `cool` warmth step)

| Token | Light | Dark |
| --- | --- | --- |
| `--paper` | #E8E9EC | #131417 |
| `--surface` | #FBFBFC | #1B1C20 |
| `--surface-2` | #F2F3F5 | #222327 |
| `--raise` | #FFFFFF | #2A2B30 |
| `--ink` | #16171A | #ECEDEF |
| `--ink-soft` | #55585E | #A3A6AD |
| `--ink-faint` | #63666C | #959AA2 |
| `--line` | #DCDEE3 | #33353B |
| `--line-soft` | #E9EAEE | #27292E |
| `--accent` | #0B5FE0 | #6FB0FF |
| `--accent-ink` | #FFFFFF | #06142B |
| `--accent-soft` | #E2EBFF | #162943 |
| `--accent-2` | #16171A ("deliberately just ink") | #ECEDEF |
| `--wash` | #F1F4FA | #1C2029 |
| `--ok` | #1A7A33 | #6EDB86 |
| `--warn` | #8E5A05 | #F0B84A |
| `--danger` | #B7352B | #FF8A80 |
| `--seal` | #0B5FE0 | #6FB0FF |
| `--grain` | 0 | not declared: Post's dark `.05` wins (section 8) |

Sources: light C:780-788, C:797; dark C:806-811, C:818-823. "one candy in the chrome, and it is
the one that means 'you, here, now': selection, focus, the unread mark" (C:783-784).

### 6.2 The candy shelf (declared for every look, used by Candy)

"Five hues, defined once for every look and theme, spent only where colour *means* something:
unread, starred, destructive, a named label. `-deep` is the text-safe member of each family,
`-soft` its tint" (C:21-23).

| Hue | Light base / deep / soft | Dark base / deep / soft |
| --- | --- | --- |
| red | #E8483C / #B7352B / #FBE3E1 | #FF6B60 / #FF8A80 / #3A1E1C |
| amber | #F0A81E / #8E5A05 / #FAEBD2 | #F5BC4E / #F0B84A / #3A2C12 |
| green | #28B24A / #1A7A33 / #DCF1E1 | #4FD06A / #6EDB86 / #153520 |
| blue | #2B7CFF / #0B5FE0 / #E2EBFF | #4C9BFF / #6FB0FF / #14263F |
| violet | #8B5CF0 / #6B3FCC / #ECE4FB | #A98BFF / #B69BFF / #271C42 |

Sources: light C:24-28; dark C:157-161, C:165-169.

### 6.3 Where the candy goes (C:935-966)

| Element | Rule |
| --- | --- |
| label chip | `-soft` background, `-deep` text; one hue per label for the life of the label (prototype: spec blue, rust amber, work violet, family green) |
| sidebar label item | stays grey; only its icon and its seal take `--tag`: blue and violet use the base hue, amber and green the `-deep` member ("amber and green have to darken to clear 3:1 against the sidebar") |
| starred | stroke `--c-amber-deep`, fill `--c-amber`; sparks `--c-amber` |
| Trash, Delete forever strip buttons | red (`-soft` background, `-deep` text) on hover only ("the red is a warning and not a label") |
| Restore strip button | green on hover |
| sidebar counts | `--ink-faint` ("nothing demands"; C:932-933; a no-op in C because counts are already `--ink-faint`, C:309) |

### 6.4 Shape, shadow, density

| Token | Light | Dark | Source |
| --- | --- | --- | --- |
| `--r-card` | 20px | same | C:790 |
| `--r-btn` | 14px | same | C:790 |
| `--r-panel` | 24px | same | C:790 |
| `--r-chip` | 999px | same | C:790 |
| `--shadow-1` | `0 3px 8px -4px rgba(16,17,20,.22)` | `0 3px 10px -4px rgba(0,0,0,.62)` | C:794, C:812 |
| `--shadow-2` | `0 10px 22px -10px rgba(16,17,20,.30)` | `0 12px 26px -10px rgba(0,0,0,.74)` | C:795, C:813 |
| `--shadow-drag` | `0 20px 34px -14px rgba(16,17,20,.36)` | `0 22px 38px -14px rgba(0,0,0,.82)` | C:796, C:814 |

"radius-to-size is what reads as cute: 20 on a 74px row, pill on everything small" (C:789).
Shadows are "bottom-heavy: the offset is always downward and the blur always wider than the
offset", and their black is tinted per warmth step (C:791-793). Chubbier padding "so the big
radii have something to sit on" (C:926-931): row `13px 13px`, chip `2px 9px`, view `9px 11px`,
mini `6px 13px`, unread dot 8x8.

### 6.5 Motion and exits

Overrides (C:798-802): t-tap 110, t-quick 190, t-move 300, t-big 520, t-ambient 7s; `--e-out
(.3,.8,.3,1)` ("ambient: no overshoot at all"); `--e-spring (.34,1.5,.5,1)` ("contact:
underdamped, visibly wobbly"); `--e-exit` inherited from Post; overshoot 1.07, squish .925, lift
-3px, tilt 2.6deg, stagger 30ms. Exits as Post: `fold`, `crumple`, `curl`.

## 7. Warmth (Candy only)

"The only thing these change is the hue of the neutrals (and the shadow's black, which was blue
and had nothing to push against). Radii, timings, easings, candy semantics and every animation
are untouched" (C:828-832). `cool` is the Candy base (section 6.1); `warm` is C's default.
Values not listed keep the Candy base.

### 7.1 Light

| Token | cool | neutral | warm | paper |
| --- | --- | --- | --- | --- |
| `--paper` | #E8E9EC | #E9E9E9 | #ECE9E3 | #E7E0D3 |
| `--surface` | #FBFBFC | #FBFBFB | #FCFBF8 | #FBF8F1 |
| `--surface-2` | #F2F3F5 | #F3F3F3 | #F4F1EB | #F3EDE1 |
| `--raise` | #FFFFFF | #FFFFFF | #FFFFFF | #FFFDF8 |
| `--ink` | #16171A | #171717 | #1A1814 | #1E1A12 |
| `--ink-soft` | #55585E | #575757 | #5A554C | #5E5648 |
| `--ink-faint` | #63666C | #656565 | #68635A | #6B6254 |
| `--line` | #DCDEE3 | #DDDDDD | #E0DBD1 | #DCD3C3 |
| `--line-soft` | #E9EAEE | #EAEAEA | #EDE9E2 | #EAE4D8 |
| `--wash` | #F1F4FA | #F4F6FA | #F5F1E8 | #F5EFE2 |
| `--accent` | #0B5FE0 | = | = | #0A55C8 |
| `--accent-soft` | #E2EBFF | = | #E4EAF7 | #DEE6F1 |
| `--seal` | #0B5FE0 | = | = | #0A55C8 |
| `--ok` / `--warn` / `--danger` | #1A7A33 / #8E5A05 / #B7352B | = | = | #17682C / #835206 / #AC3128 |
| `--c-blue-deep` / `-soft` | #0B5FE0 / #E2EBFF | = | = / #E4EAF7 | #0A55C8 / #DEE6F1 |
| `--c-amber-deep` / `-soft` | #8E5A05 / #FAEBD2 | = | = / #FAEAD0 | #835206 / #F6E4C4 |
| `--c-green-deep` / `-soft` | #1A7A33 / #DCF1E1 | = | = / #DEEFDC | #17682C / #DBEAD4 |
| `--c-violet-deep` / `-soft` | #6B3FCC / #ECE4FB | = | = / #EDE4F4 | #6B3FCC / #E8DFEC |
| `--c-red-deep` / `-soft` | #B7352B / #FBE3E1 | = | = / #FBE2DC | #AC3128 / #F7DCD3 |
| shadow black (all three shadows) | rgba(16,17,20,…) | rgba(18,18,18,…) | rgba(46,36,24,…) | rgba(62,46,26,…) |
| `--shadow-1` | `0 3px 8px -4px rgba(16,17,20,.22)` | `0 3px 8px -4px rgba(18,18,18,.22)` | `0 3px 8px -4px rgba(46,36,24,.24)` | `0 3px 8px -4px rgba(62,46,26,.26)` |
| `--shadow-2` | `0 10px 22px -10px rgba(16,17,20,.30)` | `0 10px 22px -10px rgba(18,18,18,.30)` | `0 10px 22px -10px rgba(46,36,24,.32)` | `0 10px 22px -10px rgba(62,46,26,.34)` |
| `--shadow-drag` | `0 20px 34px -14px rgba(16,17,20,.36)` | `0 20px 34px -14px rgba(18,18,18,.36)` | `0 20px 34px -14px rgba(46,36,24,.38)` | `0 20px 34px -14px rgba(62,46,26,.40)` |

Sources: cool C:780-796; neutral C:833-840; warm C:841-851; paper C:852-868. Paper: "a tinted
ground eats about half a point of contrast, so four values go one step deeper" (accent, ok, warn,
danger; C:856-859); the candy `-deep` members follow them, except violet-deep, which stays
#6B3FCC (C:863).

### 7.2 Dark

| Token | cool | neutral | warm | paper |
| --- | --- | --- | --- | --- |
| `--paper` | #131417 | #141414 | #16140F | #1A160D |
| `--surface` | #1B1C20 | #1C1C1C | #1E1B16 | #231F15 |
| `--surface-2` | #222327 | #232323 | #26231D | #2B271C |
| `--raise` | #2A2B30 | #2B2B2B | #2E2A23 | #332E22 |
| `--ink` | #ECEDEF | #EDEDED | #EFEBE3 | #F3EDE0 |
| `--ink-soft` | #A3A6AD | #A5A5A5 | #A8A296 | #AEA695 |
| `--ink-faint` | #959AA2 | #979797 | #9B9488 | #A19986 |
| `--line` | #33353B | #353535 | #383429 | #3D3729 |
| `--line-soft` | #27292E | #282828 | #2B2820 | #2F2B1F |
| `--wash` | #1C2029 | #1F1F1F | #221E17 | #262117 |
| `--accent` / `--seal` | #6FB0FF | = | #7BB4FF | #84B9FF |
| `--accent-soft` | #162943 | = | #1B2739 | #20293A |
| `--c-blue-deep` / `-soft` | #6FB0FF / #14263F | = | #7BB4FF / #1B2739 | #84B9FF / #20293A |
| `--c-amber-deep` / `-soft` | #F0B84A / #3A2C12 | = | #F2BC52 / #3A2D14 | #F5C25C / #3E3117 |
| `--c-green-deep` / `-soft` | #6EDB86 / #153520 | = | #76DC8C / #193420 | #7FE095 / #1D3823 |
| `--c-violet-deep` / `-soft` | #B69BFF / #271C42 | = | #BBA1FF / #2A2140 | #C2AAFF / #2E2544 |
| `--c-red-deep` / `-soft` | #FF8A80 / #3A1E1C | = | #FF9288 / #3C221C | #FF9B90 / #402620 |
| shadows | Candy dark (6.4) | = | = | = |
| `--ok` / `--warn` / `--danger` | #6EDB86 / #F0B84A / #FF8A80 | = | = | = |

Sources: cool C:806-814; neutral C:870-874, C:898-902; warm C:875-885, C:903-913; paper
C:886-896, C:914-924. In the dark, only neutrals, the accent and the candy members move; shadows
stay pure black.

## 8. Cascade facts the port must not reproduce by accident

The prototype's CSS has four interactions that the Rust token table (the source of truth, plan
"Token model") must decide explicitly rather than inherit:

1. **Motion levels do nothing on Candy.** `body[data-motion]` (C:173-176) and `body[data-look]`
   have equal specificity; Candy is declared later (C:777) and wins; Riso and Tide are declared
   earlier and lose. Details and the effective matrix: `05-MOTION.md#33-effective-values-per-look-and-level-as-the-prototype-cascade-computes-them`.
2. **Post's dark block leaks into other looks.** `:root:not([data-theme="light"]) body` (C:89)
   is more specific than `body[data-look="riso"]`, so any token a look's own dark block omits
   falls back to Post dark, not to the look's light value. Only `--grain` is affected (Riso,
   Tide, Candy dark get .05), and `--grain` is unused.
3. **Light warmth blocks under dark.** `body[data-look="candy"][data-warm="…"]` light blocks
   (C:833-868) have the same specificity as Post's dark block and come later, but lose to
   Candy's dark block; the effective dark values are the ones in 7.2.
4. **Dead declarations.** `--accent-2` (every look), `--grain` (every look), `--seal` in S
   (declared S:11, S:30; S's seal paints with `--f-ink`, S:346), Candy's count colour (6.3).

## 9. How S relates to Post

S is Post, re-accented and set inside a Space-coloured frame: "A Space colours the frame; the
mail stays on paper" (S:816). "The colour fills the window **around** the Post card, never
inside it, so every contrast rule we already test still holds" (S:817-820). S wins every
conflict with C's Post.

| Aspect | C Post | S |
| --- | --- | --- |
| Accent (light / dark) | red #C0402A / #E9684B | blue "Postmark" #23508F / #7FA6E6 (S:11, S:30) |
| Accent ink | #FFF6F3 / #1A0F0B | #F4F8FF / #0B142A |
| Accent soft | #F2DCD5 / #3A201A | #DCE5F3 / #1E2A44 |
| Seal | = accent, painted with `--seal` | declared = accent, but the seal is painted with `--f-ink` on the frame (S:346) |
| `--accent-2` | #2E5AA6 / #7FA6E6 | absent |
| Accent option | fixed | per Space: "Postmark" or "A hint of the Space" (an OKLCH accent derived from the Space's first dot, contrast-checked against the card; S:1201-1203, `03-COLOR.md`) |
| `--scrim` | none; the scrim is `--ink` at opacity .16 (C:1054) | `rgba(0,0,0,.22)` (S:15) |
| Grain | `--grain` token .035/.05, unused | a real 128x128 noise tile on the frame, overlay blend, opacity = grain/100 x (.16 dark, .20 light) (S:87, S:1162-1169, S:1188) |
| The frame | none: the shell is a bordered panel on paper (C:262-268) | `.win`: Space gradient layers, grain, sidebar drawn on the colour with `--f-*` tokens, the card inset 8 px (S:77-87, S:156-160) |
| Candy shelf, `--wash` | present | absent: label chips are all `--accent-soft`; "Colour is a claim" is not carried into S |
| Motion tokens | full set incl. `--t-ambient`, `--e-exit`, `--tilt`, `--shadow-drag` | only t-tap/quick/move/big, e-out, e-spring, overshoot, squish, lift, stagger (S:17-19); the exit curve is written inline |
| Motion levels, looks, warmth | yes | no |
| Reduced motion | media rule + boot Calm | media rule only (S:807-809) |
| Toast hidden offset | `translateY(140%)` (C:559) | `translateY(160%)` (S:360) |
| Hover strip | gap 4, buttons 28, icons 15 at stroke 1.8, `--i` stagger, also on `:focus-within` | gap 3, buttons 26, icons 14, `--j` stagger, hover only (S:312-322) |
| Star | left 8 bottom 8, icon 15 | left 6 bottom 7, icon 14 (S:299-302) |
| Command menu | `min(420px, 88%)`, `cmdk-in`, padding-top 14%, list max 252 | `min(540px, 88%)` (S:793; S:232 first says 520/86%), `peek-in`, padding-top 11%, max 360 |
| Peek | `inset 34px 10%`, `peek-in` from .97/10px | `inset 36px 12%`, `peek-in` from .95/12px (S:223-226) |
| Reader head padding | `16px 18px 12px` | `14px 18px 12px` (S:202) |
| `.btn` | padding 8/15, hover adds `filter:saturate(1.1)` | padding 8/14, no filter (S:386-390) |
| Menus | `.menu` (menu-in, anchored to trigger) | `.fmenu` (menu-pop, placed in Rust) |
| Trash op | yes | no (A8 #9) |
| Extra radii | none | literal 10px fields, 12px menus, 9px items, 12px tiles, which the plan names `--r-field 10 --r-menu 12 --r-item 9 --r-tile 12` |

## 10. B's Mochi (history)

B (`mailo-charm.bak.html`) is the version of C before Candy. Its default look was **Mochi**
(B:965), "cute form, chill behaviour, springs only on contact" (B:748). Candy is Mochi with the
pink removed and the randomness removed:

| Aspect | Mochi (B:749-800) | Candy |
| --- | --- | --- |
| Neutrals (light) | pink: paper #F4EDEE, surface #FDF8F8, surface-2 #F8F1F2, raise #FFFFFF, ink #40353A, ink-soft #786A6F, ink-faint #6B5E63, line #E7D9DB, line-soft #F0E5E6 | cool grey (6.1) |
| Accent | #C96B80, ink #FFF6F8, soft #F8DDE3, accent-2 #C08B5E, seal #C96B80 | #0B5FE0 |
| Status | ok #5E9478, warn #C08B5E, danger #C96B80 (danger = accent) | ok, warn, danger distinct |
| Dark | paper #1B1518, surface #241D20, surface-2 #2B2226, raise #332A2E, ink #F2E7E9, ink-soft #B9A6AC, ink-faint #A8969C, line #3A2F33, line-soft #2F262A, accent #E9899D (ink #2A1017, soft #452129), accent-2 #D8A87A, ok #7FB79A, warn #D8A87A, danger #E9899D (B:770-790) | 6.1 |
| Radii | 20 / 14 / 24 / 999 | identical |
| Shadows | same shapes, black rgba(78,52,60,…): `.28`, `.34`, `.40` | same shapes, per-warmth black |
| Motion | identical to Candy (t-tap 110 … stagger 30; `--e-out (.3,.8,.3,1)`, `--e-spring (.34,1.5,.5,1)`; "fast on contact, slow in the ambient: waiting is not chill") | identical |
| Density | the same chubbier padding | identical |
| Resting tilt | "handmade wobble: the same seeded hash that varies the fold sets the resting tilt": `rotate: calc(var(--sway,0) * .32deg)`, 0 when selected (B:791-794); B's script never sets `--sway`, so the tilt rendered as 0 | removed: "no hash-seeded variation either" (C:1382) |
| Candy shelf, warmth | none | added |

## 11. Desktop default

| Decision | Status | Source |
| --- | --- | --- |
| The desktop ships **Post as S defines it** (blue Postmark, S's tokens and component values from section 9) for every app and every surface's content | settled (plan: "spaces.html wins"; design seed "mailo's 'Spaces' language … paper-and-ink palette") | plan "Context", "Design: `<ds>`" |
| Shell chrome (bar, dock, launcher) is drawn on the current workspace's Space frame: its `--f-*` tokens over compositor blur; apps stay on paper; a workspace switch cross-fades the tint over 380 ms | settled (user) | plan "UX decisions settled"; `21-SPACES.md` |
| The Candy shelf (5 hues x base/deep/soft, light and dark, 6.2) is the palette for **app icons** (the plate's gradient families) and for **label colours** (`--c-{red,amber,green,blue,violet}{,-deep,-soft}` in the token model) | settled for icons (user, "Icons"); proposed for labels, which reverses S's "all chips accent-soft" and restores "Colour is a claim" | plan "Icons", "Token model" |
| Riso, Tide, Candy (with warmth) are kept in the token table as optional themes, reachable from the AppearancePicker, not the default | proposed | this document |
| Tide as an optional theme needs a filter-free exit (`dissolve` blurs) | proposed | `05-MOTION.md#12-open-decisions` |
| Motion levels Calm / Standard / Extra / Reduced apply on top of whichever look is active | proposed (see section 8.1) | plan "Token model" |

## 12. Open decisions

1. **Which looks ship at all.** Post (S) is the default. Keep Riso, Tide and Candy as optional
   themes, or remove them from the token table to cut the test matrix (each look multiplies the
   legibility tests by 2 schemes x 6 accents x presets)?
2. **Labels get candy hues** on the desktop (section 11), or stay `--accent-soft` as in S?
3. **Warmth outside Candy.** C scopes warmth to Candy. Post has sage neutrals of its own; should
   the warmth axis exist when Post is the default? Proposed: no.
4. **Dead tokens.** Drop `--accent-2` and `--grain` from the table (S already dropped both).
5. **Candy's `--e-exit`** is inherited from Post; Tide has its own. Confirm Candy keeps Post's.
6. **Motion levels vs looks** (8.1): uniform override proposed.
7. **Accent per Space vs Postmark** on the desktop: which is the default for a new workspace?
   Not specified (`21-SPACES.md`).

## 13. Sources

- `~/mailo-design/mailo-charm.html` (C): Post C:13-47, C:88-132; Riso C:50-64, C:101-109,
  C:133-141; Tide C:67-85, C:110-119, C:142-151; candy shelf C:21-28, C:153-170; levels
  C:172-176; exits C:711-743; Candy C:776-827; warmth C:828-924; Candy components C:926-966;
  boot C:1465-1467; notes C:1114-1162, C:1377-1388.
- `~/mailo-design/mailo-spaces.html` (S): tokens S:7-46; frame S:73-87; card S:155-160;
  palette application S:1178-1209; reduced motion S:807-809; intro S:815-820.
- `~/mailo-design/mailo-charm.bak.html` (B): Mochi B:748-800, look picker B:965-968.
- `~/mailo-design/reference-post-light.png`, `reference-post-dark.png`: Post with C's red accent.
- `~/.claude/plans/vast-toasting-peach.md`: "Context", "Design: `<ds>` design-system repo"
  (token model), "UX decisions settled with the user", "Icons", Appendix A7.
