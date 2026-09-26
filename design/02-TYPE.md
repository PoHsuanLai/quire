# 02 Type

## 1. What this governs

This file fixes the two typefaces (System and Editorial) and the faces each maps its jobs to,
which job each element uses, every size, weight, line-height and tracking pair, the text colours that go with them, numerals, and the truncation
rules. It covers the mail prototype's elements exhaustively so a shell surface can pick the
matching role instead of inventing a size. Colour tokens are named here and defined in
`03-COLOR.md`. Source keys `S`, `C`, `P` are defined in `00-PRINCIPLES.md` section 1; `S` wins
every conflict with `C`.

## 2. The faces

Five jobs, each with one token: display for names and headings, UI for reading and controls,
data for anything machine-shaped, code where a fixed pitch carries meaning, serif for a message a
person writes in a serif. Which face does each job is the root's **typeface**
(`appearance.typeface`, `22-SETTINGS.md` section 3.1; `Ds { typeface }`, written as
`data-typeface` on every `.ds`, like the theme).

The user's decision (2026-09-26): "make this desktop use mostly inter". The reference desktop's
system font (SF Pro) is licensed for its own platform only; Inter (SIL OFL 1.1, rsms/inter) is
the open equivalent, and has a Display optical cut for large sizes. Inter is the system face of
the whole desktop; the monospace face stays only where monospace carries meaning (code, `Kbd`,
aligned logs); mail's editorial faces remain an opt-in voice for an app, not the default.

| Token | System (default) | Editorial | Job |
| --- | --- | --- | --- |
| `--font-display` | Inter Display (`opsz` 32), `wght` 500..800 | Bricolage Grotesque | headings, names, initials in avatars, Space name, big numbers, the lock clock |
| `--font-ui` | Inter (`opsz` 14), `wght` 400..700; italic 400 | Karla | body text and every control |
| `--font-data` | Inter, always tabular (`font-variant-numeric:tabular-nums` on every data rule) | Space Mono | eyebrows, section headers, counts, times, chips, shortcuts, tokens, table heads |
| `--font-code` | Space Mono | Space Mono | code, `Kbd`, aligned logs |
| `--font-serif` | Noto Serif | Noto Serif | a message written in a serif, and the control that offers it |

```css
/* .ds (System) */
--font-display:"Inter Display","Inter",system-ui,sans-serif;
--font-ui:"Inter",system-ui,sans-serif;
--font-data:"Inter",system-ui,sans-serif;
--font-code:"Space Mono",ui-monospace,"SFMono-Regular",Menlo,monospace;
/* .ds[data-typeface=editorial]: exactly the prototype's (S:20-22, identical in C:44-46) */
--font-display:"Bricolage Grotesque","Trebuchet MS",system-ui,sans-serif;
--font-ui:"Karla","Segoe UI",system-ui,sans-serif;
--font-data:"Space Mono",ui-monospace,"SFMono-Regular",Menlo,monospace;
```

Editorial's faces, as the prototype loads them:

| Face | Axes and weights loaded by `S` | Loaded by `C` |
| --- | --- | --- |
| Bricolage Grotesque | variable, `opsz` 12..96, `wght` 500..800 | static instances at opsz 12..96: 500, 700, 800 |
| Karla | roman `wght` 400..700; italic 400 | roman 400, 500, 600, 700; italic 400 |
| Space Mono | 400, 700 | 400, 700 |

Loading URLs: `S:4`, `C:4`. The design system ships the faces as subset TTFs (latin plus
latin-ext) registered with the renderer, not as CSS `@font-face` (`P:289`, `P:306`, `P:324`).
Inter is cut from the official Inter 4.1 release (`crates/ds/scripts/cut-inter.sh` records the
zip's SHA-256): the renderer sets no optical size from the font size, so the variable font's
`opsz` axis is pinned into two families, Inter at 14 and Inter Display at 32, each keeping its
`wght` range; the layout features kept are `kern`, `mark`, `mkmk`, `ccmp`, `locl`, `calt`,
`case`, `tnum`, `pnum` and `zero`.

### 2.1 The voice tokens

Sizes stay per role in both typefaces. What Inter wants differently is tracking (its own dynamic
metrics: tight at display sizes, zero at body, far less than a monospace face in caps) and the
weight of a caps label (a monospace label reads at 400; Inter caps at 9.5-10 px need 600 to hold
the line). Those values are tokens, declared per typeface on `.ds`:

| Token | System | Editorial | Used by |
| --- | --- | --- | --- |
| `--tracking-heading` | -0.02em | -0.015em | `h1`-`h3` |
| `--tracking-lock-clock` | -0.02em | -0.035em | the lock screen's time |
| `--tracking-lock-date` | -0.01em | 0.01em | the lock screen's date |
| `--tracking-caps` | 0.06em | 0.14em | eyebrow; section header (frame, field, menu); calendar weekday heads; checks heading |
| `--tracking-caps-narrow` | 0.06em | 0.12em | group header; section header (group); calendar month |
| `--fw-caps` | 600 | 400 | the same caps labels (the calendar month stays 700) |
| `--fs-caps` | `--fs-micro` (9.5) | `--fs-eyebrow` (11) | eyebrow |
| `--tracking-mono` | 0 | -0.02em | `.ds-mono` |
| `--fs-mono` | .86em | .78em | `.ds-mono` |

The eyebrow under System is Inter SemiBold small-caps-like: uppercase, 0.06em apart, at
`--fs-micro`.

## 3. Base text

Every element inherits one base: UI face, 15 px, line-height 1.55, antialiased. The block below
is `S`'s; in the design system the tracking and the eyebrow's size and weight are the voice
tokens of section 2.1, which under Editorial carry exactly these values.

```css
body{ margin:0; background:var(--paper); color:var(--ink); font-family:var(--font-ui);
  font-size:15px; line-height:1.55; -webkit-font-smoothing:antialiased; }
h1,h2,h3{ font-family:var(--font-display); font-weight:700; margin:0; letter-spacing:-.015em; text-wrap:balance; }
button{ font:inherit; color:inherit; }
.mono{ font-family:var(--font-data); font-size:.78em; letter-spacing:-.02em; }
.eyebrow{ font-family:var(--font-data); font-size:11px; letter-spacing:.14em; text-transform:uppercase; color:var(--ink-faint); }
kbd{ font-family:var(--font-data); font-size:10.5px; border:1px solid var(--line); border-bottom-width:2px;
  border-radius:5px; padding:1px 5px; color:var(--ink-soft); background:var(--surface-2); }
```

`S:50-57`, `S:67-68`

Rules that follow from the base:

- Headings `h1`, `h2`, `h3` are always display 700, tracking -0.015em, balanced wrapping
  (`S:52`). `h4` and `h5` in `S` are set to the display face per rule and take the browser's
  default bold (700); no rule sets their weight explicitly (`S:284`, `S:405`, `S:516`).
- Controls inherit the UI face and size (`S:54`); inputs use `font:inherit` (`S:236`, `S:381`,
  `S:599`, `S:781`).
- `.mono` is relative: 0.78 of the parent size with -0.02em tracking (`S:56`).

## 4. The size ramp

### 4.1 Under System (Inter)

Every size in 4.2 stays, role for role; the faces and the pairs move as follows.

| Band | Face | Weight | Tracking | Notes |
| --- | --- | --- | --- | --- |
| 140 (lock clock) | Inter Display | 700 | -0.02em | tabular |
| 20-47 (subject, headings, amount, day, display, widget figures) | Inter Display | as in 4.2 (700; 800 for the day number) | -0.02em on `h1`-`h3`; the rest as in 4.2 | Inter's dynamic metrics give -0.017em at 20 and -0.02em at 26 and up; the Display cut is already spaced for it |
| 22 (lock date) | Inter Display | 600 | -0.01em | |
| 13-16.5 (display: titles, names, tile letters) | Inter Display | as in 4.2 | 0 | |
| 12.5-16 (UI: body, controls, rows) | Inter | as in 4.2 (400/500/600/700) | 0 | Inter's metrics give -0.004em at 13 and -0.011em at 16; the stylesheet keeps body at 0 |
| 9-12 (UI: help, small, tooltips) | Inter | as in 4.2 | 0 | Inter's metrics give +0.005em at 11 |
| 9.5-11.5 (data: times, counts, chips, shortcuts, cells) | Inter, tabular | 400 (700 where 4.2 says so) | 0, or as in section 5 for the non-caps rows | tabular figures everywhere the data face is used |
| 9.5-11 (data caps: eyebrow, section headers, group heads, weekday heads, checks heading) | Inter, tabular | 600 (`--fw-caps`) | 0.06em (`--tracking-caps`, `--tracking-caps-narrow`) | the eyebrow drops from 11 to `--fs-micro` 9.5 |
| 7.5-12.5 (display 800: initials, marks) | Inter Display | 800 | 0 | |
| 9.5-10.5 (code: `Kbd`) | Space Mono (`--font-code`) | 400 | 0 | unchanged |

Five sizes follow the typeface, because they were fitted to measured cap heights (the widgets
against `23-WIDGETS.md` section 1.1 with the display face's cap at .66 em; the lock clock with
them): under System each is the Editorial size x .66 / .7275 (Inter Display's cap height),
rounded to .5 px, so the drawn caps keep the measured heights. Settled, open decision 8.

| Token | Editorial | System | Cap drawn (System) |
| --- | --- | --- | --- |
| `--fs-dial` | 9 | 8 | 5.82 (5.94 measured) |
| `--fs-dial-large` | 18 | 16.5 | 12.00 (11.88) |
| `--fs-widget-figure` | 20 | 18 | 13.10 (13.20) |
| `--fs-widget-hero` | 47 | 42.5 | 30.92 (31.02) |
| `--fs-lock-clock` | 140 | 127 | 92.39 (92.40) |

### 4.2 Under Editorial (the prototype's pairs)

`S` uses 22 distinct sizes between 7.5 and 26 px in app surfaces. This is the full ramp with the
role each size carries; every row cites its rules.

| px | Face | Roles | Source |
| --- | --- | --- | --- |
| 26 | display 700, lh 1.15, ls -.015em | composer subject (`.c-title`) | `S:581` |
| 24 | display 800, lh 1.3 | event day number | `S:514` |
| 22 | display, lh 1 | receipt amount | `S:500` |
| 21 | display (h3 700), lh 1.15 | parsed-body heading `.b-h1` | `S:466` |
| 21 | display (h2 700), lh 1.25 | composer body `h2` | `S:731` |
| 20 | display 700, lh 1.15 | reader subject `h2` | `S:207` |
| 16.5 | display (h3 700) | composer body `h3` | `S:732` |
| 16 | display 700 | list bar title `h2`; reader-empty title | `S:163`, `S:217` |
| 16 | UI | command menu input | `S:236`, `S:795` |
| 15.5 | display (h4 bold) | parsed-body subheading `.b-h2` | `S:467` |
| 15 | display | event title `h4`; hover-card stat number | `S:516`, `S:419` |
| 15 | UI 400, lh 1.55 | body base | `S:51` |
| 14.5 | UI, lh 1.65 | composer body | `S:608`, `S:728` |
| 14 | display, lh 1.2 | hover-card title `h5` | `S:405` |
| 14 | display 800 | `zZ` floater | `S:334` |
| 14 | UI, lh 1.65 | reader body | `S:212` |
| 13.5 | display 700 | Space name | `S:146` |
| 13.5 | UI 700 / 500 | sender name, unread / read | `S:190-191` |
| 13.5 | UI 600 / 400, lh 1.35 | subject, unread / read | `S:193-194` |
| 13.5 | UI 600 (label `.t` 500) | sidebar item | `S:128`, `S:137` |
| 13.5 | UI | command menu item; input `.inp` | `S:239`, `S:781` |
| 13 | UI | command pill; hover card; menu item name (600); bubble button; composer inputs; prop values | `S:95`, `S:400`, `S:674`, `S:685`, `S:381`, `S:603` |
| 13 | UI 700 | reader sender (`.meta .who`); button `.btn` | `S:211`, `S:387` |
| 13 | display 800 | menu tile letter | `S:672` |
| 12.5 | UI 600 | toast; send pill | `S:362`, `S:707` |
| 12.5 | UI | snippet; composer chip; warning line; tables; key-value lists; signature | `S:195`, `S:593`, `S:695`, `S:474`, `S:503`, `S:487` |
| 12.5 | display 800 | account tile avatar initial (`.pin.acct .av`) | `S:548` |
| 12 | UI 600, lh 1 | mini button; segmented control | `S:168`, `S:266` |
| 12 | UI 700 | toast pull tab; send-pill Undo | `S:367`, `S:714` |
| 12 | UI, lh 1.4 | hover-card message (2-line clamp); hover-card flag; tooltip | `S:412`, `S:421`, `S:426` |
| 12 | display 800 | tile avatar initial; reader avatar initial | `S:116`, `S:210` |
| 12 | data, lh 1.6 | code block; plain-text preview | `S:470`, `S:702` |
| 11.5 | UI | menu item help (`small`, lh 1.3); command-menu snippet; attachment name (600); view switch (600) | `S:675`, `S:799`, `S:533`, `S:538` |
| 11.5 | data | numeric table cell | `S:479` |
| 11 | data | eyebrow (ls .14em upper); link pill; prop key; list-title account address | `S:57`, `S:432`, `S:587`, `S:1275` |
| 11 | data 700, ls .08em | attachment kind badge | `S:532` |
| 10.5 | data | kbd; count; hover-card sub; state line; quote author; fold button; blocked image; note; hint line; command token | `S:67`, `S:136`, `S:406`, `S:576`, `S:481`, `S:484`, `S:491`, `S:507`, `S:661`, `S:798` |
| 10 | data | row time; `.k` shortcut; fly preview; section header; menu shortcut; toast hint; hover-card foot; foot keys; key-value key | `S:197`, `S:100`, `S:443`, `S:121`, `S:676`, `S:365`, `S:414`, `S:699`, `S:504` |
| 10 | display 800 | hover-card avatar initial | `S:410` |
| 9.5 | data | chip; via; attachment clip; group header (menu `.g`, command `.cmdk-g`); table head; code language; receipt label; calendar month and weekday; kbd inside hints | `S:198`, `S:192`, `S:543`, `S:668`, `S:238`, `S:475`, `S:472`, `S:501`, `S:513`, `S:515`, `S:662` |
| 9.5 | display 800 | provider mark on tile; event attendee initial | `S:554`, `S:519-520` |
| 9 | data | pin count `.n`; signature label | `S:117`, `S:488` |
| 9 | display 800 | favicon letter in sidebar item; provider mark; composer chip avatar; slim-menu round tile | `S:139`, `S:553`, `S:594`, `S:788` |
| 7.5 | display 800 | in-row provider mark | `S:556` |

The design system's type tokens span `--fs-micro 9.5 .. --fs-display 26` (`P:296`); names for
the steps between are not specified.

Outside that span, the desktop widgets carry four sizes of their own, each set so its cap
height matches the reference widget's measured cap (`23-WIDGETS.md` section 1.1; the display
face's cap height is .66 em): `--fs-dial` 9 (a world clock row's dial numerals), `--fs-dial-large`
18 (a small clock's dial numerals), `--fs-widget-figure` 20 (a battery's percentage under its
ring) and `--fs-widget-hero` 47 (a small battery's percentage). These are Editorial's; System's
are in section 4.1.

Above them sits `--fs-lock-clock` 140 (127 under System), the lock screen's time in the display face at 700
(design/04 section 42; proposed, M11 2026-09-26: the user asked for a very large, heavy clock).

## 5. Tracking

Letter-spacing is fixed per role; uppercase data text is always tracked wide. The table is
Editorial's; under System the rows that are voice tokens (section 2.1) take Inter's values, and
the others (0.02em, 0.06em, 0.08em, 0.1em) are kept, since they are small enough to suit Inter.

| Tracking | Where | Source |
| --- | --- | --- |
| -0.02em | `.mono` | `S:56` |
| -0.015em | every `h1`, `h2`, `h3`; composer subject | `S:52`, `S:581` |
| 0.02em | editor label value `.ed-label .r` (resets the label's tracking) | `S:251` |
| 0.06em | toast hint; key-value key | `S:365`, `S:504` |
| 0.08em | section-header button (no uppercase); attachment kind | `S:123`, `S:532` |
| 0.1em | table head (upper); composer table first row (upper) | `S:475`, `S:766` |
| 0.12em | code language; receipt label; calendar month (all upper) | `S:472`, `S:501`, `S:513` |
| 0.14em | eyebrow; section header `.s-h`; menu group `.g`; command group `.cmdk-g`; editor label; plain-text caption; signature label (all upper) | `S:57`, `S:121`, `S:668`, `S:238`, `S:250`, `S:703`, `S:488` |

`C` adds: segmented control 0.01em (`C:232`), chip 0.02em (`C:398`), toolbar menu `h6` 0.13em
upper (`C:1011-1012`), group header 0.12em upper (`C:1028`), seg label 0.14em upper
(`C:239-240`). `S` wins where the element exists in both.

## 6. Weights

UI text uses four weights and each one means something.

| Weight | Meaning | Examples | Source |
| --- | --- | --- | --- |
| 400 | read, resting | read subject, body, snippet | `S:51`, `S:193` |
| 500 | read name; sidebar label text | read sender, `.item .t`, pinned item | `S:191`, `S:137`, `S:558` |
| 600 | control label; unread subject | sidebar item, mini, segmented, toast, menu item name | `S:128`, `S:168`, `S:194`, `S:266`, `S:362`, `S:674` |
| 700 | unread name; primary action; heading | unread sender, `.btn`, toast tab, reader sender, headings | `S:190`, `S:387`, `S:367`, `S:211`, `S:52` |
| 800 (display) | initials and marks in small coloured shapes; matched characters | avatars, favicons, provider marks, `mark` in menus | `S:116`, `S:139`, `S:553`, `S:791` |

Search highlights are weight, not background: `mark{ background:transparent; color:var(--accent);
font-weight:800; }` (`S:791`).

## 7. Line heights

| Line-height | Where | Source |
| --- | --- | --- |
| 1 | mini button; receipt amount; provider mark | `S:166`, `S:500`, `S:553` |
| 1.15 | reader subject; composer subject; `.b-h1` | `S:207`, `S:581`, `S:466` |
| 1.2 | hover-card title | `S:405` |
| 1.25 | composer `h2` | `S:731` |
| 1.3 | calendar day; menu help line | `S:514`, `S:675` |
| 1.35 | row subject | `S:193` |
| 1.4 | hover-card message and flag | `S:412`, `S:421` |
| 1.45 | cap note | `S:279` |
| 1.5 | notes | `S:285` |
| 1.55 | body base | `S:51` |
| 1.6 | code block; plain-text preview | `S:470`, `S:702` |
| 1.65 | reader body; composer body | `S:212`, `S:608` |

## 8. Text colours by role

Type uses four ink levels inside the card and three on the frame; see `03-COLOR.md` for values.

| Token | Role | Examples | Source |
| --- | --- | --- | --- |
| `--ink` | primary text; parsed-body headings; selected menu item | `.b-h1`, `.chip`, selected `.cmdk` item | `S:199`, `S:240`, `S:466` |
| `--ink-soft` | secondary text; read sender; reader paragraphs; mini and tool labels at rest | `.b-p`, `.mini`, read `.nm` | `S:168`, `S:191`, `S:464` |
| `--ink-faint` | metadata: times, snippets, via, counts, hints, eyebrows | `.row-time`, `.row-snip`, `.eyebrow` | `S:57`, `S:195`, `S:197` |
| `--accent` | links; matched characters; floater | `a`, `mark`, `.floater` | `S:334`, `S:438`, `S:791` |
| `--paper` on `--ink` | inverse pills | toast, send pill, link pill, fly | `S:361`, `S:432`, `S:443`, `S:707` |
| `--f-ink` | frame text, hovered or current | Space name, current item, hovered pill | `S:98`, `S:132-134`, `S:146` |
| `--f-ink-soft` | frame text at rest | sidebar item, command pill | `S:95`, `S:129` |
| `--f-ink-faint` | frame metadata | section header, count, `.k` | `S:100`, `S:121`, `S:136` |
| `#fff` | initials on coloured avatars and favicons | tile avatar, favicon | `S:116`, `S:139` |

Placeholders are `--ink-faint` (`S:600`, `S:783`); empty-state placeholders additionally at
opacity 0.6 or 0.7 (`S:582`, `S:622`, `S:647`, `S:761`).

## 9. Numerals

Times, counts that change in place, and numeric cells use tabular figures.

| Element | Rule | Source |
| --- | --- | --- |
| Row time | `font-variant-numeric:tabular-nums` | `S:197` |
| Editor check value | tabular | `S:275` |
| Table cells | tabular | `S:477` |
| Sidebar count `.item .count` | data 10.5, not tabular in `S` | `S:136` |

`C`'s sidebar count and group count are tabular (`C:308-310`, `C:1032`).

## 10. Truncation and wrapping

One line, ellipsis, never a wrap, for everything in a list or on the frame; a fixed measure for
reading.

| Rule | Elements | Source |
| --- | --- | --- |
| Single line with ellipsis (`white-space:nowrap; overflow:hidden; text-overflow:ellipsis`) | sender `.nm`; subject `.row-sub`; snippet `.row-snip`; sidebar label `.item .t`; command pill label (inline style); attachment name `.att .nm2`; link pill; command-menu snippet (`max-width:380px`) | `S:190`, `S:193`, `S:195`, `S:137`, `S:841`, `S:533`, `S:431-433`, `S:799` |
| Never shrinks (`flex:none`) | via mark; `.k` shortcut; favicon | `S:192`, `S:100`, `S:138` |
| No wrap, no ellipsis | mini; chip; fly preview; toast; send pill; `.k` | `S:166`, `S:199`, `S:442`, `S:362`, `S:708`, `S:100` |
| Two-line clamp | hover-card message (`-webkit-line-clamp:2`) | `S:412` |
| Balanced wrapping | `h1`, `h2`, `h3` (`text-wrap:balance`) | `S:52` |
| Pre-wrap, break words | composer block content | `S:620` |
| Measure | reader blocks and composer body `max-width:66ch`; focus mode `70ch` | `S:462`, `S:608`, `S:728`, `S:574` |
| Direction | a paragraph with `dir` isolates (`unicode-bidi:isolate`); chrome stays left to right | `S:465`, `S:1112` |

`C` also truncates the account name and host (`C:294-297`) and the drag ghost's subject
(`C:585-586`).

The design system cannot rely on `text-overflow:ellipsis` or line clamp in Blitz: it plans a
`.ds-truncate` mask fade for ellipsis and a `clip_chars` helper for line clamp, and bans
`line-clamp` in consumer CSS (`P:420-422`).

## 11. S versus C

`S` wins each row.

| Element | `S` | `C` | Source |
| --- | --- | --- | --- |
| Command menu input | 16 | 14.5 | `S:795`, `C:1071` |
| Segmented control | 12 / 600, padding 5/11 | 12.5 / 600, ls .01em, padding 6/13 | `S:266`, `C:231-232` |
| Floater `zZ` | 14 | 15 | `S:334`, `C:747` |
| Row dot offset | padding-top 6 | 5 | `S:185`, `C:1036` |
| Star icon | 14, stroke 2 | 15, stroke 1.8 | `S:302`, `C:411` |
| Sidebar icon stroke | 2 (`.ic`) | 1.7 | `S:58`, `C:307` |
| Strip icon | 14, stroke 2 | 15, stroke 1.8 | `S:322`, `C:443` |
| Reader-empty caption | mono (`.mono`, 0.78em) "pick a thread" | mono 11 px "pick a thread · or drag one onto a place" | `S:1388`, `C:1207` |
| Hero / intro heading | `clamp(26px,3.6vw,36px)` / 1.02 | `clamp(30px,5.4vw,46px)` / 0.98 | `S:63`, `C:205` |
| Tabs | none | display 700 16, padding `10px 14px 12px` | `C:244-247` |

## Open decisions

1. **Display axes.** Inter's `opsz` axis is pinned per family, Inter at 14 and Inter Display at
   32 (section 2). For Bricolage: `S` loads it as a variable font (`wght 500..800`), `C` as three
   static instances (500, 700, 800) (`S:4`, `C:4`). Which the design system ships, and whether the
   `opsz` axis is set automatically by size or pinned, is not specified.
2. **`h4` and `h5` weight** comes from browser defaults in `S`, not a rule. A native renderer may
   default differently; the intended weight (700 is the inference) is not specified.
3. **Type token names.** Only `--fs-micro 9.5` and `--fs-display 26` are named (`P:296`); the
   mapping of the 23 sizes in section 4 to tokens is not specified, nor whether sizes are merged.
4. **Stroke width exceptions.** `C` uses 1.7 and 1.8 px strokes on some glyphs (`C:307`,
   `C:411`, `C:443`) against its own "2px stroke" rule (`C:1121`). `S` uses 2 everywhere. `S`
   wins; recorded because the plan cites `C`'s rule.
5. **Tabular count.** `S`'s sidebar count is not tabular; `C`'s is. Not specified which the
   design system uses; the plan's extraction lists count as "tabular" (`P:978`).
6. **Shell type.** Bar clock, bar labels, dock hover labels, notification title and body, OSD
   value, control-center labels and widget text have no assigned role. macOS uses a 13 pt menu
   font (`P:1471`); no size is settled for any shell surface.
7. **Ellipsis in Blitz.** Whether the mask-fade replacement is acceptable visually for every row in
   section 10 is not specified (`P:420-422`).
8. **Widget sizes under Inter.** Settled (2026-09-26): the five cap-fitted sizes follow the
   typeface, section 4.1.

## Sources

- `S` `~/mailo-design/mailo-spaces.html`: font loading `S:4`; tokens `S:20-22`; base `S:48-68`;
  sidebar `S:92-153`; card, rows, reader `S:155-218`; command menu `S:229-241`; editor
  `S:243-279`; toast, composer, button `S:359-393`; hover card and link pill `S:396-448`; parsed
  bodies `S:456-560`; composer page `S:564-775`; menus and inputs `S:777-799`; markup
  `S:841`, `S:1275`, `S:1388`.
- `C` `~/mailo-design/mailo-charm.html`: font loading `C:4`; tokens `C:44-46`; base
  `C:178-192`; hero `C:205`; segmented and tabs `C:226-257`; sidebar `C:293-311`; rows
  `C:384-401`; star and strip `C:405-443`; menus and command menu `C:1003-1085`.
- `P` `~/.claude/plans/vast-toasting-peach.md`: fonts in the design system `P:289`, `P:306`,
  `P:324`; type tokens `P:296`; Blitz text risks `P:420-422`; Appendix A2 `P:960-982`; macOS
  menu font `P:1471`.
