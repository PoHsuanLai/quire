# 29 Sizing: control heights, spacing and radii from rules

Status: **draft, proposed** (2026-09-28). Nothing here is settled and nothing has been built:
this is a research and proposal pass. The audit material is in `audit/` (section 2). Confidence
legend as in `23-WIDGETS.md`: **H** primary source (Apple's own design templates, HIG or
Support pages), **M** reliable secondary source or a careful measurement, **L** estimate. The
reference is the pre-Liquid-Glass Mac (macOS 14 Sonoma and 15 Sequoia;
`27-HIG-PARITY.md#01-which-hig`). As in 28, the reference platform is named here only as a
source. Code, class names and tokens never name it.

## 1. What this governs

The user's feedback (2026-09-28): "the bar and control center: the capsule size is very
disproportional ... basically the size and spacing probably needs a principled way to handle.
and some widgets need care, like the toggle." And: the Monochrome and Muted icon styles in dark
mode are "too black, unusable as UI".

This document:

1. lists every size, spacing and radius token, and every hard-coded size in quire's component
   sheets, with what sill's bar and control center add on top (section 3);
2. ranks the inconsistencies (section 4);
3. collects the reference numbers with a source and a confidence each (section 5);
4. states the rules every option obeys (section 6), then three systems, A, B and C, each with a
   token table and before/after mockups (sections 7-9), and a recommendation (section 10);
5. gives three fixes for the dark Monochrome and Muted plates (section 11);
6. sketches the order of work and lists the open decisions (sections 12-13).

## 2. How the audit was made

Everything is under `audit/` in this worktree (branch `sizing-audit`).

| What | Where | How |
| --- | --- | --- |
| Every literal `px` size in quire's component sheets | `audit/hardcoded-px.tsv` (250 rows: file, selector, declaration) | a CSS parse of `crates/ds/src/components/*.css`, comments stripped, shadows and transforms left out |
| Measured boxes of the shell controls | `audit/current/measured.txt` | `cargo run --release -p ds-native --example sizing_audit -- audit/current`: the laid-out rect of each part (`Harness::rect`) at 1x |
| Specimen sheets of the controls as they stand, light and dark, 1x and 2x | `audit/current/specimens-{light,dark}-{1,2}x.png` | the same example: a 32 px bar, toggle, slider, segmented, buttons, module tiles and panel, tinted plates |
| Gallery crops at 1x and 2x | `audit/gallery/gallery-{buttons,status-items,matrix,control-center}-{light,dark}-{1,2}x.png` | `ds-gallery --page P --snapshot DIR --scale 100\|200`, cropped |
| sill's bar and control center, live | `audit/current/{bar,bar-cosmic,bar-cosmic-menu,control-center-cosmic,control-center-wifi-cosmic}.png` | copied from sill's `dev/out/` (the 2026-09-27 acceptance runs on nested cosmic-comp at scale 1). sill has no stand-alone headless PNG path: `dev/shot.sh --surface NAME` goes through a running daemon (`sill debug capture`), and its render tests check HTML only |
| Dock icons in each style | `audit/icons/dock-icons-*.png`, `audit/icons/dock-strip.png`, `audit/icons/shipped-styles-light-dark.png` | sill's `dev/out/` captures; the shipped PNG sets composited on light and dark grounds |
| Mockups of the options | `audit/mockups/{bar,cc,controls}-{light,dark}-{1,2}x.png`, `audit/mockups/plates-dark-{1,2}x.png` | `python3 audit/mockups/build.py && audit/mockups/render.sh` (headless Chrome, Inter). Every number drawn comes from the `OPTIONS` table in `build.py` |

Gotcha found on the way: `ds-gallery --snapshot DIR` at scale 100 also writes into
`tools/progress/shots/gallery/` (`snapshot::progress_dir`), overwriting tracked PNGs. They were
restored with `git checkout`. Worth a flag (`--no-progress`) or a note in the gallery's usage.

## 3. Inventory

### 3.1 Tokens

| Family | Values | Where | Notes |
| --- | --- | --- | --- |
| Spacing `--s-*` | 1, 1.5, 2, 3, 4, 5, 6, 7, 8, 9, 10, 11, 12, 13, 14, 15, 16, 18, 22, 26, 36 | `tokens/spacing.rs` | Named by value, taken from the web prototype ("dense and not a geometric ramp", `01-LAYOUT.md#2-units-and-the-spacing-scale`). Uses across the sheets: s-6 63, s-8 56, s-2 31, s-4 29, s-10 27, s-12 27, s-3 26, s-5 25, s-16 24, s-7 15, s-14 14, s-9 11, s-11 10, s-1 9, s-22 4, s-18 3, s-26 3, s-1-5 2, s-13 2, s-36 2, s-15 1 |
| Radius `--r-*` | panel 14, card 12/12/12/4, btn 9, chip 6/6/6/2, pill 999, field 10, menu 12, item 9, tile 12, window 18, menu-item 8, bubble-button 7, small 6, kbd 5, tiny 4, micro 3, media 10/10/10/3 | `tokens/shape.rs` | 17 names, 11 distinct numbers. No rule ties a radius to a height. Uses: pill 56, tile 9, panel 9, menu-item 7 |
| Material radius `--m-radius` | window/bar 0, dock 22, popover 14, sheet and OSD 18, toast 16, widget 20 | `material/recipe.rs:274` | The control center is a Sheet (18) |
| Shell scale | bar font 13/500, bar item 24, bar pad 10, bar item radius 4, menu font 13, menu row 22, separator margin 5, highlight radius 6, launcher field 22/500, field glyph 20, row 14/12, tip 12 | `tokens/shell.rs` | Written by sill from its settings (`shell_style.rs`) |
| Status metrics | box 22, glyph 16 | `IconButton { Status }` fallbacks | From `bar.status_icon_box_px`, `bar.status_glyph_px` |
| Type | control 13, small 12, help 11.5, body 13.5, base 15; inherited `line-height:1.55` | `tokens/type_scale.rs`, `css/reset.css:10` | The 1.55 line is the reading line; controls inherit it |

### 3.2 Control geometry, measured (1x, light, `audit/current/measured.txt`)

| Part | Box (w x h) | Where the height comes from |
| --- | --- | --- |
| Toggle track | 38 x 26 | 30 x 18 content + 3 padding + 1 hairline border, `toggle.css` |
| Toggle knob | 18 x 18 | literal; travel a fixed 12 (O-4 candidate, not signed off) |
| Slider | 160 x 22; track 4; thumb 22 with a 3 px ring | literal, `slider.css` (O-5 candidate) |
| Segmented, Small | 196 x **29.8** | 11.5 x 1.55 line + 3+3 padding + 2+2 inset + hairlines |
| Segmented, Regular | 210 x **36.6** | 12 x 1.55 line + 5+5 padding + 3+3 inset + hairlines |
| Button, Mini | 47 x 24 | 12 x 1 line + 5+5 padding + hairlines |
| Button, Primary/Regular | 79 x **38.2** | 13 x 1.55 line + 8+8 padding + hairlines |
| Level capsule (control center) | 270 x 26 | `--level-h:26px`, `level.css` |
| Module tile | 144 x 52 | `min-height:52px`; padding 8 4 8 10 |
| Module disc | 28 x 28 | literal |
| Module panel with a level | 296 x **73.8** | padding 10/12 + 16.25 head + 8 gap + 26 + hairlines |
| Bar item (title) | 49 x 24 | `--shell-bar-item` |
| Status item | 22 x 22 | `--bar-status-box` |
| Icon buttons | tool 28 x 26, foot 24, strip 26, status 22 | literals in `icon_button.css` |
| Text input, boxed | about 37 | `calc(1.55em + 16px)` at 13.5 |
| Settings row | min 44 | literal |
| Menu row, slim/dropdown | 22 | `--shell-menu-row` |

The literal sizes in `audit/hardcoded-px.tsv` fall into these groups: avatar sizes 16, 18, 20,
22, 26, 28, 30, 34, 48, 64; glyph boxes 11, 13, 14, 16, 17, 18, 20, 22; control boxes 22, 24,
26, 28, 34, 38; widths of floating surfaces 220, 226, 250, 260, 280, 296, 300, 540.

### 3.3 How sill's bar and control center use them

sill reads its sizes from `sill-settings` and writes some of them into quire's tuned tokens;
others are its own constants, so there are two copies of several sizes.

| sill value | Default | Where | Goes to |
| --- | --- | --- | --- |
| `bar.height_px` | 32 | `sill-settings/src/bar.rs:161` | the layer's height and exclusive zone |
| `bar.title_hit_height_px` | 24 | same | the clock's hit box (`bar/clock.rs:25`) |
| `bar.open_title_pill_height_px` | 24 | same | `--shell-bar-item` (`shell_style.rs:17`) |
| `bar.title_padding_px` | 10 | same | `--shell-bar-pad` |
| `bar.status_icon_box_px` / `status_glyph_px` | 22 / 16 | same | `--bar-status-box` / `--bar-status-glyph` |
| `bar.status_gap_px` | 4 | same | the right group's gap |
| `bar.item_radius_px` | 4 | same | `--r-shell-bar-item` (title pill, status pill, workspace pills) |
| bar padding | `var(--s-8)` | `style/bar.css` | |
| `control_center.width_px` / `grid_gap_px` / `grid_padding_px` | 320 / 8 / 12 | `sill-settings/src/control_center.rs:198` | `ModuleGrid` |
| module heights for the popup size | tile 56, level 76, now playing 112, appearance 226, battery 204, detail head 48, list row 34, detail foot 82 | `control_center/geometry.rs` | the popup's size estimate before layout |
| popup chrome | margin 8, side 28, top 8, bottom 40; menu item 30, header 28, separator 9, chrome 12 | `bar/geometry.rs` | the popup's size and anchor |
| the level look | `LevelLook::Capsule` (no knob) | `control_center/level.rs:9` | Display, Keyboard, Sound |

## 4. Inconsistencies, ranked

Ranked by how much of the "disproportional" look each explains.

| # | What | Evidence | Why it matters |
| --- | --- | --- | --- |
| 1 | **No height ladder.** Control heights fall out of padding plus the reading line (1.55), so neighbours never agree: 22, 24, 26, 28, 29.8, 36.6, 38.2 | 3.2 | In the control center a 28 disc, a 26 capsule, a 29.8 segmented and 20 swatches sit together; nothing lines up |
| 2 | **Everything is about 1.2-1.7x the reference.** Regular button 38 against 22; segmented 30-37 against 22; toggle 26 tall against 22; bar 32 against 24 | 3.2, 5.3 | The web prototype's density (15 px body; `27-HIG-PARITY.md` row 8) was carried into the shell |
| 3 | **Fractional heights** 29.8, 36.6, 38.2, 73.8 | measured | Breaks the pixel-snapping rule (`01-LAYOUT.md#21-pixel-snapping-settled-2026-09-25`): edges land between device pixels at 1.25-1.75 |
| 4 | **Toggle proportions.** 38 x 26 track, 18 knob: the knob is .69 of the height and sits 4 px in (3 padding + 1 border); the width is 1.46x the height | `toggle.css`; specimens | The reference's switch knob is the height less 2 (1 px inset) and the track about 1.73x as wide as it is tall (26 x 15, knob 13). Ours reads as a thick frame round a small ball, which is what "the toggle needs care" points at |
| 5 | **The inner radius is larger than the outer one.** The level capsule (radius 13) sits 10-12 px inside a module panel of radius 12; tiles (radius 12) sit 12 px inside the 18 px Sheet; workspace pills (radius 4) sit 2 px inside a radius 4 container | `level.css`, `module_panel.css`, `recipe.rs`, `workspace_pills.css` | Concentric corners need inner = outer - inset. Here the inner corner is rounder than the frame round it, so the gap between the curves is uneven and the corners bulge |
| 6 | **Two kinds of capsule, two sizes.** The Slider is a 4 px track under a 22 thumb; the LevelControl is a 26 px filled capsule with no knob. The control center uses the second at 26, next to 28 discs | `slider.css`, `level.css`, `control_center/level.rs` | The reference's control center slider is about 22 tall with a round knob filling it (M). Our capsule is taller than every other control in its row, the "very disproportional capsule" |
| 7 | **Radii are not tied to height.** `--r-btn` 9 is used on a 38 button (.24 h) and on a 24 Mini (.38 h); the title pill and the status pill are radius 4 on 24 and 22 | `button.css`, `shell.rs` | The reference is 5 on 22 (.23 h) for rounded rectangles and h/2 for capsules |
| 8 | **The bar mixes three item heights**: the title pill 24, the status box 22, the workspace box 24 with 20 pills, all in a 32 bar | sill settings, `workspace_pills.css` | Pills of different heights in one row read as unequal capsules; 4-5 px above and below each |
| 9 | **The status item is a square.** 22 x 22 with a 16 glyph leaves 3 px either side | `icon_button.css` | The reference's status slot is about 30-32 wide by 24 tall with the glyph 7 in from each side (H); a square box makes the pressed pill look pinched |
| 10 | **The spacing scale has 21 steps and 13 of them are off a 4 px grid** (1, 1.5, 3, 5, 7, 9, 11, 13, 14, 15, 18, 22, 26) | 3.1 | Off-grid steps (s-3, s-5, s-7, s-9, s-11) are used about 90 times. Asymmetric paddings follow (tile 8 4 8 10; panel 10/12; button 8/14; Mini 5/10) |
| 11 | **sill keeps a second copy of the sizes.** TILE_PX 56 against the tile's 52; LEVEL_PX 76 against 73.8; LIST_ROW_PX 34 against the settings row's 44; menu ITEM_PX 30 against the 22 row | `control_center/geometry.rs`, `bar/geometry.rs` | Any sizing change must be made twice or the popup's first size is wrong |
| 12 | **The segmented control is a capsule** (radius 999 in and out), where the reference's is a rounded rectangle (well radius 6, segment 5, 1 px inset, H) | `segmented.css` | Capsule segments next to rounded-rectangle buttons: two shape languages in one panel |
| 13 | **Icon-button sizes** 28 x 26 (not square), 24, 26, 22; glyph boxes 11, 13, 14, 16, 17, 18, 20, 22 | `icon_button.css`, `01-LAYOUT.md#11-fixed-sizes` | No glyph size is tied to a control size |
| 14 | **Container radii are unrelated to their padding.** Popover 14, Sheet 18, Toast 16, Widget 20, Dock 22, Menu 12 | `recipe.rs`, `shape.rs` | Reference: window, popover and sheet 10, menu 8 (inset 5), dock 16, notification 16 (H). The rule in 6 derives them |
| 15 | **Slider thumb has a 3 px ring and a tint**; the reference's is a plain white disc of 20 with a shadow on a 4 px track (H) | `slider.css` | Reads heavier than the track it rides |
| 16 | **Disabled and hover are unspecified** for toggle, slider and segmented (O-1) | component headers | Not a size issue, noted because the toggle pass will touch it |
| 17 | **Dark Monochrome and Muted plates** are about 1:1 against the dark dock | section 11 | Unusable, as the user says |

## 5. Reference numbers

Measured from Apple's **macOS Sequoia Design Templates** (Sketch; frames and `fixedRadius` read
from the layer JSON, so exact) unless the source column says otherwise. Control Center is not in
the templates: those rows are pixel measurements of Apple's Mac User Guide illustration, scaled
by 1.51 px/pt on the assumption of a 37 pt notched menu bar, which makes the panel 300 pt wide.

### 5.1 Menu bar

| Item | Value (pt) | Source | Conf. |
| --- | --- | --- | --- |
| Height, no notch | 24 (22 up to Yosemite) | templates `Menu Bar/Light` 1440 x 24; bjango.com/articles/designingmenubarextras | H |
| Height, notched | 37 (27-43 by display scaling) | bjango | M |
| Status glyph | 16 x 16 round, 18 x 16 box; at most 22 tall | templates `Item Icon`; bjango | H/M |
| Status slot | 30-32 wide x 24, glyph 7 in from each side, 3 from the top | templates `Right Side` | H |
| Text item | label 7 in from each side | templates `Menu Bar/x/Item` | H |
| Open-item highlight | about 22 tall on the 24 bar; radius unknown (5-6 likely) | Support illustration (30 on the 37 bar) | L |

### 5.2 Control Center (Sonoma/Sequoia)

| Item | Value (pt) | Conf. |
| --- | --- | --- |
| Panel width | about 300 | L-M |
| Outer padding, module gap | about 10, 10 | L-M |
| 1x1 module | about 63 x 63; 2x1 (Focus) 136 x 63; 2x2 (Wi-Fi, Bluetooth, AirDrop) 136 x 136; full-width rows (Display, Sound, Now Playing) 281 x 63 | L-M |
| Module radius | about 10 (continuous curve) | L |
| Panel radius | about 16 | L |
| Display/Sound slider | a capsule about 22 tall with a white knob of about 21 filling it | L-M |
| Round toggle disc (Wi-Fi etc.) | about 25-26 | L |

### 5.3 Controls

| Control | Value (pt) | Source | Conf. |
| --- | --- | --- | --- |
| Push button, regular | 22, radius 5, label inset 7 | templates | H |
| Push button, large | 28, radius 5 | templates | H |
| Pop-up, pull-down, combo box | 22, radius 5; indicator 16, radius 4 | templates | H |
| Text and search field | 22, radius 5 | templates | H |
| Segmented control | well 22, radius 6; selected segment 20, radius 5, 1 inset | templates | H |
| Segmented icon sizes | 17 x 17, 14 x 13, 12 x 11 (regular, small, mini) | HIG Segmented controls (2024 snapshot) | H |
| Switch, mini | 26 x 15 track (capsule), knob 13, inset 1 | templates `Controls/Switch/On`; HIG Toggles (Mar 2024): the mini switch in grouped form rows | H |
| Switch, small / regular | about 32 x 18 / 38 x 22 | not in the templates | L |
| Slider, regular | track 4, radius 2; round knob 20; frame 20; tick-mark knob 8 x 20 | templates | H |
| Checkbox, radio | 14 | templates | H |
| Sidebar row | 28, selection radius 5 | templates | H |
| Menu item | 22; separator 9 | templates | H |
| Title bar, toolbar | 28, 52 (items 38) | templates | H |

Mini and small heights for push buttons (16, 19 are often quoted) were not confirmed. A set of
switch sizes going round (36 x 16 ... 64 x 28) was measured on macOS 26 and does not apply.

### 5.4 Radii, spacing, contrast

| Item | Value | Source | Conf. |
| --- | --- | --- | --- |
| Window, sheet, alert | 10 | templates | H |
| Popover | 10, arrow 34 x 8 | templates | H |
| Menu | 8, content inset 5 (item highlight 5: not concentric) | templates | H |
| Notification | 346 x 75, radius 16 | templates | H |
| Dock | 16 | templates | H |
| Group box, form group | 6 | templates | H |
| Concentricity | the segmented control follows inner = outer - inset (5 = 6 - 1); menus do not. Apple published concentricity as a rule only with Liquid Glass (WWDC25), after the target | templates; WWDC25 | H (the numbers), L (as an older rule) |
| Window content margin | 20 | templates | H |
| Form rows | 36-37 (51 with a subtitle); inset 20; section gap 10 | templates | H |
| Sibling spacing | 8 (Interface Builder's standard) | NSLayoutConstraint | M |
| Contrast | text up to 17 pt 4.5:1, 18 pt or bold 3:1; custom colours at least 4.5:1, aim 7:1 for small text | HIG Accessibility, Dark Mode (2024) | H |
| Dark and tinted app icons | not on macOS 14/15 (iOS 18 only; the Mac got them in 26). The iOS 18 recipe: a greyscale icon on a system gradient; dark = transparent art on a dark system gradient; no published luminance numbers | HIG App icons (2024 changelog, 10 Jun 2024) | H |

## 6. Rules every option obeys

These are the principles; the options differ only in the numbers fed to them.

- **R1. Heights come from a ladder, never from padding.** A control sets `height` from a size
  token and centres its label with `line-height:1`; padding sets only the horizontal inset.
  Every control height is a whole number of pixels.
- **R2. Capsules are h/2.** Toggle track, slider track, level capsule, pills and badges:
  `radius = height / 2`.
- **R3. Knobs fill their track less a fixed inset.** `knob = height - 2 x inset`. The inset is a
  token (1 in A, 2 in B and C); a knob never shrinks by padding plus border.
- **R4. Switch width is a fixed ratio of its height:** `width = round_even(height x k)`. The
  reference's k is 1.73 (26/15 and 38/22 both); A uses it, B and C round to 1.6-1.7.
- **R5. Rounded rectangles take their radius from the ladder** (the control's size), not from a
  per-component token.
- **R6. Concentric nesting.** An element inset `p` inside a frame of radius `R` has radius
  `max(R - p, r_min)`; equivalently, a container's radius is its child's radius plus its padding.
  A child is never rounder than `R - p`. A capsule child forces its container to be a capsule or
  to have `R >= h/2 + p`.
- **R7. Spacing is on a 4 px grid with a 2 px half step**: 2, 4, 6, 8, 10, 12, 16, 20, 24, 32. 1 is
  kept for offsets only (the hairline is its own token).
- **R8. Glyph size follows control size** (a table per option), so a glyph never sets a box.
- **R9. One source of truth.** sill's size estimates (`control_center/geometry.rs`,
  `bar/geometry.rs`) read quire's size tokens (a `ds::ControlSize` / module-height API) instead
  of their own constants.

R1-R6 are checkable: a unit test over the token tables (every capsule radius is h/2, every knob
is h - 2i, every declared nesting obeys R6) and a lint (`Rule::RawControlHeight`: a literal
height on a control in consumer CSS). Per the standing rule, a new Strict lint breaks sill master
at once, so the sill session is warned before it lands.

## 7. Option A: the reference ladder (S 16 / M 22 / L 28)

Take the reference's numbers literally for the shell. Three control sizes; everything else is
derived by the rules in section 6 with a 1 px knob inset.

| Token (proposed) | S | M | L | Rule |
| --- | --- | --- | --- | --- |
| `--ctl-h` control height | 16 | **22** | 28 | ladder |
| rounded-rectangle radius | 4 | 5 | 6 | ladder (about h/4.5) |
| capsule radius | 8 | 11 | 14 | R2 |
| knob inset | 1 | 1 | 1 | R3 |
| switch | 26 x 15, knob 13 | 38 x 22, knob 20 | 48 x 28, knob 26 | R3, R4 (S is the reference's mini exactly) |
| slider | track 4, knob 16 | track 4, knob 20 | track 6, knob 26 | knob = h - 2 |
| level capsule | 16, knob 14 | 22, knob 20 | 28, knob 26 | R2, R3 |
| segmented | well 16 r4, segment 14 r3 | well 22 r6, segment 20 r5 | well 28 r7, segment 26 r6 | R6 |
| label | 11/600 | 13/500 | 13/600 | |
| glyph | 12 | 16 | 18 | R8 |
| horizontal inset | 6 | 10 | 12 | R7 |

Shell composition:

| Surface | Before | A |
| --- | --- | --- |
| Bar | 32 tall; title pill 24 r4; status 22 x 22 r4; gap 4 | **24** tall (37 on a tall-bar setting, items unchanged); title pill 22 r5, inset 8; status slot 30 x 22 r5, glyph 16, gap 0; workspace box 22 r5, pills 18 r3 |
| Control center | 320 wide, Sheet r18, pad 12, gap 8, modules r12 | 320 wide, pad 10, gap 10, modules r8, **panel r18** (8 + 10, R6) |
| Module tile | 52, disc 28, padding 8 4 8 10 | 56 (2 x L), disc 28 (L), padding 0 10 |
| Level module | 73.8; capsule 26, no knob | 64: pad 10, head 16, gap 6, capsule 22 with a 20 knob |
| Appearance picker | segmented 29.8 capsule | segmented 22 r6 / 20 r5 |
| Menu | row 22, highlight 6, menu 12 | row 22, highlight 5, menu 10 (5 + 5, R6) |
| Toggle in a settings row | 38 x 26 | S: 26 x 15 (the reference's mini in form rows) |
| Buttons | Regular 38 r9, Mini 24 r9 | M 22 r5, S 16 r4, L 28 r6 |

Mockups: `audit/mockups/bar-{light,dark}-{1,2}x.png` (row A),
`audit/mockups/cc-{light,dark}-{1,2}x.png` (column A), `audit/mockups/controls-{light,dark}-{1,2}x.png`.

For: exact parity with the target (27's aim), the smallest and densest shell, every number
cited. Against: a big step down from today (a 24 bar is 8 px shorter; buttons lose 16 px), which
also moves mailo if the app components share the ladder; 16 px S controls are small on a 1x
panel.

## 8. Option B: one unit per density (u = 20 / 24 / 28)

The user's suggestion made parametric. Each surface picks a density; one unit `u` (the regular
control height) drives every size through `calc()` (Blitz already resolves `calc()` with
variables throughout the sheets). A 2 px knob inset.

| Quantity | Formula | Compact u=20 | **Regular u=24** | Comfortable u=28 |
| --- | --- | --- | --- | --- |
| control height | u | 20 | 24 | 28 |
| small / large control | u - 4 / u + 4 | 16 / 24 | 20 / 28 | 24 / 32 |
| rounded-rectangle radius | u / 4 | 5 | 6 | 7 |
| capsule radius | u / 2 | 10 | 12 | 14 |
| knob | u - 4 | 16 | 20 | 24 |
| switch | round_even(1.6u) x u | 32 x 20 | 38 x 24 | 44 x 28 |
| slider | track 4, knob u - 4 | 4 / 16 | 4 / 20 | 4 / 24 |
| segmented | well u + 4, segment u, inset 2 | 24 / 20 | 28 / 24 | 32 / 28 |
| glyph | u - 8 | 12 | 16 | 20 |
| horizontal inset | u / 2 | 10 | 12 | 14 |
| gap | u / 3, to even | 6 | 8 | 10 |
| bar height | u + 8 | 28 | 32 | 36 |
| module tile | 2u + 8 | 48 | 56 | 64 |
| module padding / panel padding | u / 2 | 10 | 12 | 14 |
| module radius / panel radius | u / 2 / module + padding | 10 / 20 | 12 / 24 | 14 / 28 |

For: one setting (`appearance.density`) replaces a dozen px keys in sill's settings; the bar
keeps its 32; the densities give the app side (mailo) its own scale without a second system;
the rules are formulas, so a test can sweep every density. Against: Regular is not the
reference (24 against 22 controls, 32 against 24 bar); the 2 px inset and 1.6 switch ratio are
ours, not cited; `calc()` everywhere makes the sheets harder to read and every fractional `u`
has to be forbidden (u stays a multiple of 4).

Mockups: row/column B in the same files (drawn at u = 24).

## 9. Option C: repair in place

Keep today's sizes and token names; fix only the ratios and the snapping.

| Fix | Before | After |
| --- | --- | --- |
| Explicit heights, `line-height:1` in controls | 29.8, 36.6, 38.2, 73.8 | 30, 36, 36, 72 |
| Toggle | 38 x 26, knob 18 | 44 x 26, knob 22 (h - 4, 1.7x) |
| Level capsule | 26 inside a r12 panel | 24, panel r12 and pad 12 kept, capsule h/2 = 12 = R - 0 (accepted as the one exception) |
| Control center panel | r18, pad 12, tiles r12 | r24 (12 + 12) |
| Status item | 22 x 22 | 24 x 24, level with the title pill |
| Workspace pills | r4 in r4, pad 2 | r2 in r4 |
| sill constants | own copies | read from quire (R9) |
| Spacing scale | 21 steps | unchanged; new code uses R7 steps only (lint warns) |

For: least churn, no settled value moves, can land in a day. Against: it leaves the
disproportion the user named (every control stays 1.2-1.7x the reference), so the complaint
survives; the spacing scale stays a 21-step list.

## 10. Recommendation

**A's numbers, built with B's mechanism.** Take the reference ladder (S 16 / M 22 / L 28, 1 px
knob inset, concentric containers, 4 px grid) as the shell's scale, because the whole programme
aims at parity with the pre-Liquid-Glass Mac (27) and A is the only option whose every number is
cited. Build it the way B proposes: a `ds::ControlSize { Small, Regular, Large }` whose sizes are
computed from a height and the section 6 rules in Rust (one `SizeScale` struct, one test that
sweeps R1-R6), written as tokens (`--ctl-h-*`, `--ctl-r-*`, `--knob-*`, `--seg-*`), so a
density can be added later without a second system. The shell ships Compact (A's numbers); the
app side (mailo) can sit one step larger until it is audited separately. C's two cheap fixes
(sill reading quire's sizes, the gallery's progress-dir gotcha) go first whichever option is
taken.

Why not B alone: its Regular keeps the shell a step larger than the reference, the thing the
user objected to. Why not C: it leaves the proportions as they are.

The bar height is the one number worth a separate decision (section 13): A says 24, but sill's
32 may be deliberate for a non-notched external display; A's items (22) work in either, centred.

## 11. Dark icon plates: Monochrome and Muted

**What happens.** A tinted plate keeps each stop's OKLCh lightness and replaces hue and chroma
(`icon/retint.rs::recolour`, `icon/plate_tint.rs`). In dark, the neutral plate is `#2A2E28` to
`#1D211B` (L .30 and .24, `icon/family.rs:74`), so a Monochrome neutral plate is a dark navy
about `#222C42`: **1.00:1** against a dark dock of `#2A2C30` (`audit/mockups/plates-dark.txt`).
A third-party raster keeps its own lightness too, so Chrome's ring or a black logo stays near
black on it. The shipped quire icon sets are fine (their Monochrome sets sit at L .55-.64,
`audit/icons/shipped-styles-light-dark.png`); the trouble is the neutral plate, and the
third-party raster on it (`audit/icons/dock-strip.png`, bottom row). Muted keeps the same dark
neutral plate, so it fails the same way.

Three fixes (`audit/mockups/plates-dark-{1,2}x.png`):

| Fix | Rule | Plate on the dark dock | Glyph on plate | Raster's dark ink |
| --- | --- | --- | --- | --- |
| Before | lightness kept | #222C42, 1.00:1 | 11.5:1 | #1B212D (lost) |
| **F1** lightness floor | in dark, a tinted plate's stops are raised to L >= .50 (base) and .44 (deep); chroma kept at the tint's | #5B6375, 2.32:1 | 4.95:1 | #1B212D (lost) |
| F2 light plate in both schemes | tinted plates always use the light scheme's neutral stops (`#FFFFFF`/`#F1F3EE`, dark ink) | white, 14:1 | 17:1 | #1B212D (reads) |
| **F3** tone band | F1 for the plate (L mapped into .46-.56), and in dark the raster and glyph lightness remapped into a band, `L' = .62 + .34 L`, so no part of an icon is darker than the plate | #6B7486, 2.98:1 | 3.9:1 | #98A0B0 (reads) |

**Recommendation: F3.** F1 alone fixes the plate but not the raster (the user's screenshot shows
both). F2 is legible but a row of white plates glares on a dark dock and drops the tone-on-tone
look Monochrome exists for. F3 keeps the look (one hue, tone on tone), clears 3:1 for the plate
against the dock (the non-text floor) and lifts the raster's darks. It is one more branch in
`recolour` (a `Scheme` argument and the band) and the same rule for `PlateTint::tinted`, so plate
and raster still go through one implementation. Muted takes the same floor on its plate (its
chroma rule is unchanged). The numbers (.46, .56, .62, .34) are proposals to tune on the
progress page beside Colour.

## 12. Order of work (sketch)

1. C's two cheap fixes: sill's size constants read quire (R9); the gallery's snapshot stops
   overwriting `tools/progress/shots/gallery/` unless asked.
2. `ds::SizeScale` and `ControlSize` with R1-R6 as a unit test; tokens emitted; no component
   moved yet.
3. Toggle, Slider, SegmentedControl, LevelControl (add the knob look to the control center),
   Button sizes, IconButton Status onto the scale; specimens re-rendered at 1x, 1.5x and 2x.
4. ModuleTile, ModulePanel, ModuleGrid and the Sheet radius (R6); sill's module heights follow.
5. The bar: status slot width, one item height, workspace pills; the bar height decided (13).
6. Spacing: R7 steps in new code (lint warns), then a sweep of the off-grid uses.
7. F3 for dark plates.

Every step lands on the progress page at a gate, with a sill session warned before any Strict
lint (`Rule::RawControlHeight`).

## 13. Decisions (settled with the user 2026-09-28)

| # | Decision | Settled |
| --- | --- | --- |
| 1 | Which system | A's numbers built on B's mechanism: one `SizeScale` in Rust computes every size from R1-R6 (user picked A) |
| 2 | Bar height | 24, items 22, status slots 30 x 22 (user) |
| 3 | Knob inset | 1 (reference) |
| 4 | Toggle in settings rows | S (26 x 15) |
| 5 | Segmented shape | rounded rectangle (well r6, segment r5) |
| 6 | Control center radius | panel 18 = module 8 + padding 10 (R6) |
| 7 | Control center level look | capsule with a knob |
| 8 | Dark plate fix | F3 tone band (user) |
| 9 | Does mailo move with the shell | not yet: one density step larger until audited |

## 14. Sources

- Apple, macOS Sequoia Design Templates (Sketch):
  `https://devimages-cdn.apple.com/design/resources/download/macOS-Sequoia-Design-Templates-Sketch.dmg`,
  layer frames and `fixedRadius` read from the document JSON (2026-09-28).
- Apple Mac User Guide, "Quickly change settings" (Control Center illustration):
  `https://support.apple.com/guide/mac-help/quickly-change-settings-mchl50f94f8f/15.0/mac/15.0`.
- Apple HIG, archived 2024 snapshots (`web.archive.org/web/20240901id_/…`): Toggles,
  Segmented controls, Sliders, App icons (changelog 10 June 2024), Accessibility, Dark Mode.
- bjango, "Designing menu bar extras": `https://bjango.com/articles/designingmenubarextras/`.
- WWDC25 (Liquid Glass) for concentricity as a named rule; not in the target period.
- This repo: `crates/ds/src/tokens/{spacing,shape,shell,type_scale}.rs`,
  `crates/ds/src/components/*.css`, `crates/ds/src/material/recipe.rs`,
  `crates/ds/src/icon/{family,plate_tint,retint}.rs`; sill (read only):
  `crates/sill-settings/src/{bar,control_center}.rs`,
  `crates/sill-surfaces/src/surfaces/{bar,control_center}/`, `crates/sill-surfaces/src/style/`.
