# 08 Icons

Two icon systems, one rule: glyphs are line drawings in one weight and one colour; app icons
are objects on one plate shape. Nothing third-party is adopted wholesale.

Status words: **settled** = decided with the user or copied from the prototypes or plan;
**proposed** = this doc's suggestion, open until the user signs it off.

Source keys: S = `~/mailo-design/mailo-spaces.html`, C = `~/mailo-design/mailo-charm.html`,
PLAN = `~/.claude/plans/vast-toasting-peach.md` (section "Icons", settled 2026-09-23, third round),
SPEC = "Rust Desktop Shell — Technical Spec" (Claude Doc, rev 31).

## 1. Glyph system

### 1.1 Sets and licences (settled)

| Role | Set | Licence | Where it is used |
| --- | --- | --- | --- |
| Primary | Lucide (lucide.dev) | ISC | every glyph that Lucide has |
| Gap fill | Tabler Icons (tabler.io/icons) | MIT | only when Lucide has no glyph for the concept |
| Our own | drawn in-house | our licence | only when neither set has it |

Why Lucide: it is the design's own spec. C:1121-1129 card 1, "One weight, one colour: Lucide
24px grid, 2px stroke, round caps and joins, no illustration, no fills." Tabler uses the same
24 grid, 2 px stroke and round caps, so its glyphs sit next to Lucide without a visible seam.

Rejected (settled, PLAN "Icons"): WhiteSur, Yaru, Tela, Papirus, elementary, Colloid, Candy
icon themes. Phosphor (named in SPEC "Visual design system") is not used: Lucide is already the
prototypes' set.

### 1.2 Drawing rules (settled; applies to Tabler imports and our own glyphs)

1. Canvas `viewBox="0 0 24 24"`.
2. Stroke width 2, `stroke-linecap="round"`, `stroke-linejoin="round"`, `fill="none"`.
3. Keep 1 px padding: no geometry outside 1..23 (Lucide's own rule).
4. No fills. One exception: the star's on-state fills with its stroke colour
   (S:302-303: `.star .ic` stroke `--ink-faint`; on: stroke and fill `--warn`).
5. No text, no gradients, no second colour, no opacity inside the glyph.
6. A Tabler glyph is imported unchanged except for removing Tabler's invisible
   `<path stroke="none" d="M0 0h24v24H0z" fill="none"/>` bounding path.
7. A glyph we draw is reviewed at 16 px next to its three nearest Lucide neighbours in the
   gallery before it lands.

### 1.3 Data model (settled: moves verbatim from mailo, then made `pub`)

Glyphs are data, not markup strings. Source:
`~/mailo/crates/mail-app/src/ui/icon/mod.rs` and `geometry.rs`.

- `enum Icon` (mod.rs:21-54): one variant per glyph, `Copy + Eq`, no serde (an icon is
  never stored; mod.rs:12-13).
- `Icon::ALL` (mod.rs:60-93) lists every variant; a test locks the count
  (mod.rs:271-293).
- `Icon::shapes(self) -> &'static [Shape]` (mod.rs:96-131).
- `enum Shape` (mod.rs:142-161): `Path(&'static str)`, `Circle { cx, cy, r }`,
  `Rect { x, y, width, height, rx }`; numbers are kept as the source's decimal text, so
  `.6` stays `.6` (mod.rs:137-139). A missing `rx` is `"0"`.
- `geometry.rs`: one `const NAME: &[Shape]` per glyph, transcribed exactly from Lucide
  (for example `STAR`, geometry.rs:13-15).

Changes for quire (settled in PLAN "Design: `<ds>`"):

| mailo today | quire |
| --- | --- |
| `pub(super)` | `pub` |
| stroke from CSS `.ic` (mod.rs:186) | stroke as SVG attributes, colour `currentColor`, because usvg in Blitz may not resolve CSS on SVG (spike S6) |
| `Glyph { icon, class }` | `Glyph { icon, size: IconSize }`, renders `svg.ds-ic` |
| 32 glyphs, one `geometry.rs` | `icon/geometry.rs` (mailo set) + `icon/geometry_shell.rs` (shell set, 1.6) |
| S6 fails | fallback: `mask-image` with a `data:` SVG and `background: currentColor` (spike S7) |

Rendered markup (proposed until spike S6 reports):

```html
<svg class="ds-ic" data-size="16" viewBox="0 0 24 24" aria-hidden="true"
     stroke="currentColor" stroke-width="2" stroke-linecap="round"
     stroke-linejoin="round" fill="none"> <path d="…"/> … </svg>
```

The stroke width stays 2 in the 24 grid at every size, so the drawn stroke scales with the
glyph (Lucide behaviour). No `absoluteStrokeWidth`.

### 1.4 Sizes

`enum IconSize` (the `size` prop; px are logical, CSS `width/height`):

| Variant | px | Status | Where (source) |
| --- | --- | --- | --- |
| `Micro` | 11 | settled | clip in row, stop remove, pchip x (S:264, 544, 596) |
| `Tiny` | 12 | settled | request head (S:771), today x (mailo shell.css:119) |
| `Small` | 13 | settled | today x, toast tab, link pill (S:143, 370, 437) |
| `Compact` | 14 | settled | mini, star, strip, btn, prop key, warn (S:173, 302, 322, 391, 588, 697) |
| `Nav` | 15 | settled | sidebar item, foot button, search (mailo shell.css:106, 140; list.css:6) |
| `Base` (default) | 16 | settled | `.ic` base (S:58) |
| `Tile` | 17 | settled | rich menu tile (S:673) |
| `Large` | 18 | settled | picked image, all-accounts tile (S:493, 550, 642) |
| `Bar` | 22 | proposed | bar status items and control-center tiles; given in the orchestrator brief, no prototype line uses it |

Consumers pick a variant; no consumer writes an icon `width` in CSS (lint `RawFontSize` /
`BlitzUnsupported` do not cover this, so `lint::markup` rejects an `.ds-ic` with an inline size;
proposed).

#### 1.4.1 Stroke at a fractional scale (settled 2026-09-25)

At a whole scale the stroke is the design's 2 in the 24 grid. At 1.25, 1.5 and 1.75 `Glyph`
reads the root's scale and rounds the stroke's width in device pixels to the nearest *even*
count (halves up, never below one), writing it back in grid units (`icon/stroke.rs`): a 16 px
glyph draws 2 device pixels at all three (`stroke-width` 2.4, 2, 1.7143), a 24 px one 2, 4, 4.
Even, because Lucide draws on whole grid lines: an even stroke centred on a grid line that falls
on a device pixel boundary has both edges on boundaries; an odd one sits half a pixel off. At
16 px and 1.5 one grid unit is exactly one device pixel, so every whole-coordinate stroke is
crisp; at 1.25 and 1.75 only the grid lines that land on device boundaries are (grid 0, 6, 12,
18, 24 at 16 px), and diagonals and curves are antialiased as they always are. The glyph's box
and a plate's rect are placed on the device grid by the layout snap (01-LAYOUT §2.1); a plate's
rim and highlight are `--hair`.

### 1.5 Colour

- Settled: stroke is `currentColor`; the glyph takes the text colour of its parent. The
  only colours a glyph ever gets are the parent's token colour (ink, ink-soft, ink-faint,
  `--f-ink*` on the frame, accent where the design says so, e.g. S:424, 790).
- Settled: "Colour is a claim" (C:1121-1129 card 4). A glyph is greyscale unless the hue
  states a fact (starred, destructive, which label).
- Symbolic recolouring (settled rule, proposed mechanics) for tray icons and any third-party
  symbolic icon:
  1. Freedesktop `*-symbolic.svg`: rasterise at the target size, keep only alpha, paint the
     alpha with the parent's `currentColor` (on the bar: `--f-ink-soft`, hover `--f-ink`).
  2. SNI `IconPixmap` (ARGB) or a non-symbolic themed icon: if every opaque pixel has OKLCH
     chroma < 0.04 (proposed threshold), treat it as monochrome and recolour as in (1);
     otherwise show it as-is (a coloured tray icon states a fact about its app).
  3. `NeedsAttention` status uses `--warn` for the recoloured mask (proposed).
- Blitz: the recoloured tray icon is a `mask-image` data URI with `background:
  currentColor`, the same path as the S6 fallback.
- Settled mechanics (tray gaps, quire Q6): `ds::IconSource::{Glyph(Icon), Symbolic(ExternalIcon),
  Image(ExternalIcon)}`, `ExternalIcon { url: IconUrl, size: IconSize }`. `IconUrl` is a `data:`
  or `file:` URL only (`png`, `svg`, `file`, `parse` constructors). `Symbolic` renders
  `span.ds-ext-icon[data-kind=symbolic]` with an inline `mask-image:url(...)` over
  `background-color:currentColor`, `mask-size:100% 100%`; `Image` renders
  `[data-kind=image]` with the URL as `background-image`, `background-size:100% 100%`. Both are
  squares of `--ic-size` (the `ExternalIcon`'s size). Proved on Blitz with a PNG mask (the
  opaque pixels take `--ink`, the transparent ones show the ground) and an RGB PNG (keeps its
  red), `crates/ds-native/tests/tray_gaps.rs`. Step 3 (`--warn` for `NeedsAttention`) stays
  the caller's: it sets the colour of the icon's parent.
- Settled (bar gaps): step 2's test is `ds::icon::classify(png_bytes) -> Result<IconKind::{Symbolic,
  Image}, DsError>`: symbolic when every pixel with alpha >= 128 has OKLCH chroma below the
  threshold (0.04, still proposed; `classify_with(png, ChromaLimit(thousandths))` takes another).
  The settings key is `icons.symbolic_chroma_max` (design/22-SETTINGS.md section 3.3, settled
  2026-09-24: default 0.04, range 0.0..=0.2, Advanced). `ChromaLimit` gains a `Default` (the same
  0.04) and `TryFrom<f32>`, which takes the key's plain chroma value and refuses anything outside
  the range (`DsError::ChromaLimitRange`); `sill` still needs to register the key in its own
  settings crate and pass the parsed value into `classify_with` (FINDINGS "Tune wave"). The
  caller maps `IconKind` to `IconSource`.

### 1.6 Initial shell glyph list

Lucide names from lucide.dev; Tabler names from tabler.io/icons. "Check" = confirm the name
exists at the pinned Lucide version when `geometry_shell.rs` is written.

| `Icon` variant | Lucide | Tabler (only if Lucide lacks it) | Used by |
| --- | --- | --- | --- |
| `Wifi`, `WifiLow`, `WifiHigh`, `WifiOff` | `wifi`, `wifi-low`, `wifi-high`, `wifi-off` | | bar, control center |
| `Ethernet` (settled, bar gaps) | `ethernet-port` | | bar (wired network) |
| `Battery`, `BatteryLow`, `BatteryMedium`, `BatteryFull`, `BatteryCharging`, `BatteryWarning` | `battery`, `battery-low`, `battery-medium`, `battery-full`, `battery-charging`, `battery-warning` | | bar |
| `Bluetooth`, `BluetoothConnected`, `BluetoothOff` | `bluetooth`, `bluetooth-connected`, `bluetooth-off` | | control center |
| `Volume`, `Volume1`, `Volume2`, `VolumeX` | `volume`, `volume-1`, `volume-2`, `volume-x` | | bar, OSD |
| `Mic`, `MicOff` | `mic`, `mic-off` | | control center |
| `Sun`, `Moon`, `SunMoon` | `sun`, `moon`, `sun-moon` | | brightness OSD, appearance, DND |
| `Power` | `power` | | power menu |
| `Lock` | `lock` | | lock, power menu |
| `Bell`, `BellOff` | `bell`, `bell-off` | | notifications, DND |
| `Grid` | `layout-grid` | | launcher apps, overview |
| `Window` | `app-window` | | windows provider, dock menu |
| `Monitor` | `monitor` | | displays |
| `Keyboard` | `keyboard` | | input settings |
| `ChevronLeft/Right/Up/Down` | `chevron-left`, `chevron-right`, `chevron-up`, `chevron-down` | | menus, disclosure |
| `Folder` | `folder` | | files, dock stack |
| `File` | `file` | | files, launcher |
| `Image` | `image` | | files, viewer |
| `Terminal` | `square-terminal` (check; older name `terminal-square`) | | launcher, terminal |
| `StickyNote` | `sticky-note` | | quick note |
| `Camera` | `camera` | | screenshot UI |
| `Download`, `Upload` | `download`, `upload` | | dock stack, progress |
| `Copy` | `copy` | | launcher actions |
| `Link` | `link` | | share, link pill |
| `Sparkles` | `sparkles` | | launcher AI/quick actions |
| `Gauge` (power profile) | `gauge` | | control center |
| `Brightness` (alt) | none | `brightness-half` (candidate) | OSD, only if `sun` reads wrong at 22 px |

Plus the 32 mailo glyphs (mod.rs:21-54), unchanged.

## 2. App icon template

The model draws only the object. The plate, gradient, highlight, mask and sizes are applied
by code (settled, PLAN "Icons" item 1).

### 2.1 Plate shape

- Settled: continuous-curvature squircle, a superellipse, not a CSS `border-radius`.
- Proposed: Lamé superellipse with exponent **n = 5**:
  `|x / a|^5 + |y / a|^5 = 1`, `a` = half the plate side, centred on the canvas.
  Its area is 95.0 % of the bounding square (computed: `4·Γ(1+1/n)² / Γ(1+2/n)` divided
  by 4). A plain rounded square with iOS's commonly quoted 22.37 % corner radius covers
  95.7 %; so n = 5 reads as the same weight of corner without copying Apple's curve.
  The perceived corner radius is about 22 % of the side (proposed statement, to be checked
  in the gallery next to a 22.37 % CSS radius).
- Path (proposed): parametric, `t` in `[0, 2π)`:
  `x(t) = cx + a · sgn(cos t) · |cos t|^(2/n)`,
  `y(t) = cy + a · sgn(sin t) · |sin t|^(2/n)`.
  Sample 1024 points (256 per quadrant) evenly in `t`, then fit cubic Béziers
  (4 segments per quadrant, max error 0.25 px at 1024) for the SVG form. The raster mask
  is computed analytically per pixel with 4x4 supersampling for anti-aliasing.
- One function owns the shape: `quire::icon::plate::path(size) -> Path` and
  `mask(size) -> AlphaImage` (proposed names). Nothing else draws a plate.

### 2.2 Canvas and plate grid (proposed)

| Export px | Plate side | Margin per side | Shadow baked |
| --- | --- | --- | --- |
| 1024 (master) | 824 | 100 | yes |
| 512, 256, 128 | 80.5 % of canvas, rounded to whole px | rest | yes |
| 64, 48 | 80.5 % → 52, 39 | rest | yes, 1 px blur floor |
| 32 | 30 | 1 | no |
| 24 | 22 | 1 | no |
| 16 | 15 | 0/1 (plate at 0,0) | no |

824 / 1024 is the published macOS app-icon grid (Apple Human Interface Guidelines, macOS app
icon template); we use the proportion only. Small sizes drop the margin because a 13 px
plate at 16 px is unreadable.

### 2.3 Plate gradient (settled source, proposed mapping)

Two stops, `base` → `deep`, at **135deg** (the same angle as the Space frame gradient,
palette.rs:223, 237), from the Candy shelf (C, PLAN Appendix A3):

| Family | base | deep | soft | Glyph colour on plate (proposed) |
| --- | --- | --- | --- | --- |
| red | `#E8483C` | `#B7352B` | `#FBE3E1` | white |
| amber | `#F0A81E` | `#8E5A05` | `#FAEBD2` | ink `#16171A` |
| green | `#28B24A` | `#1A7A33` | `#DCF1E1` | ink `#16171A` |
| blue | `#2B7CFF` | `#0B5FE0` | `#E2EBFF` | white |
| violet | `#8B5CF0` | `#6B3FCC` | `#ECE4FB` | white |
| paper (neutral, proposed) | `#FFFFFF` | `#F1F3EE` | | ink `#1A1E1A` |

Why the glyph colour column: measured WCAG ratios against the lighter `base` stop
(worst case): white on red 3.87, blue 3.87, violet 4.30, amber 2.03, green 2.78; ink on
amber 8.81, green 6.45. White fails 3.0 on amber and green, so those plates take ink.
Soft is for the symbolic variant's badge background and for third-party neutral plates,
not for the gradient.

Dark mode: icons do not change with the theme (proposed; matches freedesktop, where an app
icon has one appearance). Open decision 2.

### 2.4 Object placement (proposed)

- Safe area: a centred square of **80 %** of the plate side. No object pixel with alpha
  > 0.1 lies outside it.
- Optical centre: the object's alpha-weighted centroid sits at the plate centre, shifted up
  by 2 % of the plate side (objects with a heavy base read low otherwise).
- Size: the object's bounding box fills 64-80 % of the plate side on its longer axis.
  A single glyph (not an object) on a plate: 56 % (proposed; Lucide glyph at stroke 2 in a
  24 grid, scaled).
- Object shadow: the model's own soft studio shadow is kept only if it stays in the safe
  area; otherwise it is cut at the safe area edge with a 4 % feather.

### 2.5 Edge highlight and shadow (proposed values from design tokens)

Scaled from the dock tile size of 48 px (10-BEHAVIOUR-dock base size) to the export size
with factor `s = size / 48`:

| Layer | Source token | Value at 48 px | Applied |
| --- | --- | --- | --- |
| Inner top highlight | `--shadow-1` / `--shadow-2` inset part, `0 1px 0 rgba(255,255,255,.7) inset` (S:13-14) | 1 px white at alpha .35 (the cmd pill's weight, A4 `0 1px 0 rgba(255,255,255,.35)`) | baked, all sizes ≥ 24 |
| Inner rim | `--f-line` light `rgba(0,0,0,.08)` | 1 px inside the mask edge, alpha .08 black | baked, all sizes |
| Drop shadow | `--shadow-2` outer part, light `0 6px 16px -6px rgba(26,30,26,.30)` (S:14) | as token | baked only in exported hicolor files ≥ 48 (for other launchers); the dock and launcher load the shadowless `flat` render (2.6) and draw `--shadow-2` in CSS |

The dock and launcher draw their own shadow from the token at render time, so the
theme's shadow (dark `0 10px 22px -8px rgba(0,0,0,.7)`, S:33) applies. Only exported
hicolor files carry a baked shadow.

### 2.6 Exports and naming (proposed paths, settled sizes)

Sizes (settled): freedesktop hicolor 16, 24, 32, 48, 64, 128, 256, 512 + 1024 master, plus
`@2` directories (freedesktop Icon Theme Specification scale directories).

```
icons/
  master/<app-id>.png                         1024, shadow, sRGB, 8-bit RGBA
  master/<app-id>.object.png                  the model's object on transparent, 1024
  master/<app-id>.flat.png                    1024, no shadow (dock/launcher source)
  hicolor/<N>x<N>/apps/<app-id>.png           N in 16 24 32 48 64 128 256 512
  hicolor/<N>x<N>@2/apps/<app-id>.png         N in 16 24 32 48 64 128 256
  hicolor/scalable/apps/<app-id>.svg          only if vtracer output passes 6 (optional)
  hicolor/symbolic/apps/<app-id>-symbolic.svg
  recipes/<app-id>.toml                       see 3.8
```

- `<app-id>` is the app's reverse-DNS id, identical to its `.desktop` file name and its
  Wayland `app_id` (SPEC "App integration": reverse-DNS id, icon named by app id).
  Namespace is open decision 1 (placeholder `<ns>.Mail`, `<ns>.Files`, …).
- Downscaling: from `flat` 1024 with Lanczos3 in linear light, then the plate mask is
  re-applied at the target size (never downscale a masked edge).
- SPEC says apps ship "an SVG icon". Our icons are rasters (1024 master, as macOS icons are);
  the scalable SVG is optional via vtracer. Open decision 3.

### 2.7 Symbolic variant (proposed)

For the bar, tray and any place that shows the app in one colour:
- One Lucide/Tabler glyph that names the app (Mail → `mail`, Files → `folder`, Terminal →
  `square-terminal`, Notes → `sticky-note`, Photos → `image`, Settings → `settings`), drawn
  by 1.2 rules, exported as a freedesktop `-symbolic.svg` with fill/stroke `#bebebe` (the
  freedesktop symbolic convention, so GTK/Qt recolour it too).
- Inside quire it is never loaded from disk: the shell uses the `Icon` variant directly.

### 2.8 The language: abstract, paper and ink (proposed, round two, 2026-09-24)

Round one (docs/icons-bakeoff.md) produced 3D clay objects in front of a plate. The user's
verdict: "too realistic, I want it to be more abstract, and try to make it speak our design
language." What the language says, applied to an app icon:

| Principle (00, 07) | On the plate |
| --- | --- |
| Paper and ink (07 3, Post) | The object is paper cut-outs laid on the plate: Post `--surface` `#F8F9F6` as paper, Post `--ink` `#1A1E1A` as ink. |
| One weight (00 3, C:1121) | Every line is one stroke: 2 units of the 24 glyph grid laid over the plate, round caps and joins. No hairlines, no second weight. |
| Colour is a claim (00 3) | Four colours besides the plate: the family's `deep` and `soft` (or `base` on paper), paper, ink. No other hue. |
| The cuteness is in the corner radius (C:1162) | Paper shapes take Post's card corner, `12 12 12 4` read as `3 3 3 1` (or `2.5 2.5 2.5 1`) grid units, so the small bottom-left corner recurs across the set. |
| Grain (03 8) | The plate carries the frame's grain tile (Park-Miller seed 7, overlay, `grain / 100 x 0.20`) from 48 px up; cut-outs sit above it, like content over the frame. |
| No illustration (C:968) | No lighting, no perspective, no gloss, no scene. One idea per icon, two to four shapes. |

Cut-outs may carry a lift: Post `--shadow-1`'s drop (`0 1px 2px`) read in the grid at the 48 px
dock size, 0.5 unit down and 1 unit soft, ink at alpha .16 (alpha proposed). The plate itself
keeps 2.3-2.5 unchanged (gradient, highlight, rim, baked shadow).

Two routes produce icons in this language; the user picks (docs/icons-bakeoff.md, round two):

- **Procedural** (`tools/icons abstract`): an icon is a TOML spec in `tools/icons/specs/`: the
  family, the grain and a list of layers from a five-shape vocabulary: `rect` (per-corner radii),
  `circle`, `chevron`, `bar` (a round-capped stroke), `fold` (a turned-down corner) plus a
  general `polygon` (rounded by `round`). Each layer has an optional `fill` and `stroke` from
  `base | deep | soft | paper | ink` and `lift = "flat" | "lifted"`. Every export size is drawn
  natively from the shapes (analytic anti-aliasing of signed distances), not downscaled, so
  16 px edges stay crisp. Deterministic; no model.
- **Generated** (3.3's round-two brief): the same language asked of the models, then 3.7.

### 2.9 Round three: pressed into the plate (proposed, 2026-09-24; replaces 2.8's stroke rule for app icons)

The user's reference for round three (no frosted glass; colours may differ per app) asks for a
symbol crafted into the plate rather than drawn on it. For **app icons only**, these rules replace
2.8's "one weight" and "four colours" rows and 2.3's single-family gradient; the small glyph set
(section 1) keeps Lucide's 2 px stroke unchanged.

| Rule | Value (proposed) |
| --- | --- |
| No outlines | No borders, outlines, strokes or edge separation anywhere in the symbol. Chevrons and bars are filled round-ended bands (default width 2 grid units), not strokes. |
| Emboss | Form comes from shallow relief. A `raised` layer has a light inner edge on top (white, alpha .55) and a soft darker inner edge below (ink, .20), both reaching 0.45 grid units in with 0.8 units of softness, plus a faint seat under it (ink, .14); a `recessed` layer the reverse; `flush` has none. Everything is clipped to the shape, so nothing draws outside it but the seat. |
| Part of the plate | The symbol is paper at 0.92 opacity (the ground tints it); details pressed into it are ink at 0.08-0.10, `recessed`. No drop shadow floats it off the plate. |
| Two-hue ground | Each app has its own pair `ground = { start, end }` (a Candy family's base or a hex), 135deg across the plate, mixed in OKLCh along the shorter hue arc so distant hues meet through a saturated middle, softened to 78 % chroma and lifted 0.03 in lightness. |
| Matte diffusion | A gentle light from above: the face is 0.05 L lighter at the top than the middle and as much darker at the bottom. The frame grain (03 8) stays, at 20. No glossy highlight. |
| Plate thickness | The bevel catches light in narrow spots only: a thin highlight arc along the top edge (white, .55, strongest in the middle, fading to nothing at the sides), a soft shade band along the bottom edge (.14), and one small soft specular point near the top-left corner (.28, radius 1.1 x plate/48 px). The 2.5 drop shadow stays for exported hicolor files; no glow. |
| Small sizes | At 16 and 32 px the emboss, the grain, the specular point and the soft bevel are dropped; the flat silhouettes stay, and recessed details get 2.5x their fill opacity so they still show. |

Pairs used for the first five (proposed): Mail blue to violet, Files amber to green, Terminal
violet `#6B3FCC` to amber, Notes green to blue, Photos red to amber.

Routes, both implemented:
- **Procedural**: `tools/icons abstract` renders these rules from the specs in
  `tools/icons/specs/` (fields: `ground`, `grain`, and per layer `kind`, `fill` = `paper | ink |
  start | end | <colour>`, `opacity`, `relief` = `raised | recessed | flush`).
- **Generated**: 3.3's round-three brief, then `tools/icons face`: `--mode ground` takes the
  model's full-bleed embossed surface as the plate face (centre 78 % crop) and adds our bevel;
  `--mode tile` keys a model-drawn tile off its ground and re-masks it with our squircle, keeping
  the model's own bevel.

## 3. Generation pipeline

Settled route: LOCAL FIRST (PLAN "Icons", "Route").

### 3.1 Environment (settled)

- ComfyUI in its own uv venv (`uv init`, `uv add`, `uv run`; never system pip), on the RTX
  5070 Ti (16 GB).
- Run through `limit4g` like every heavy tool (PLAN "Memory and build enforcement" item 5).
  Proposed: the ComfyUI server itself gets a separate `limit16g` cgroup, because model
  weights exceed 4 GB of host RAM while loading; open decision 4.
- Driven from the orchestrating session through ComfyUI's HTTP API by a small Python client
  in `~/quire/tools/icongen/` (uv project, proposed path), so every run is a script.

### 3.2 Models (settled candidates)

| Order | Model | Size | Why | Licence |
| --- | --- | --- | --- | --- |
| 1 | Qwen-Image-2512 (the design's "Qwen-Image 2.0" does not exist under that name; Qwen-Image-2.1 is non-commercial and excluded) | 20B (every release); needs ~13 GB VRAM even at Q2_K | generation and reference edit in one model | Apache-2.0 |
| 2 | FLUX.2 Klein | 4B, ~13 GB VRAM, 4-step | fast iterations | FLUX.2 Klein licence (record exact terms, section 5) |
| 3 | HiDream-O1-Image | | only if 1 or 2 disappoints | MIT |

### 3.3 Style brief (settled rules, proposed text; rewritten in round two)

Written once, used verbatim for every icon; only `{subject}` and `{palette}` change.
Never say "iOS", "macOS", "Apple", "app icon" or "icon" in the prompt (settled).

Round one's brief (a 3D clay object under studio light, `tools/icongen/icongen/brief.py`
`BRIEF`, style `3d`) is kept only to reproduce round one. The brief below replaces it (2.8).

Positive prompt (proposed, `FLAT_BRIEF`, style `flat`):

```
A flat abstract geometric emblem: {subject}. Built from a few simple paper cut-out layers with
crisp clean edges, like shapes cut from coloured card and laid flat on top of each other. Only
four flat colours: {palette}, paper white, and near-black ink. Thick uniform lines with round
caps and rounded joins, one line weight everywhere. Completely flat solid fills, no lighting,
no shading, no gradients, no shadows, no highlights, no gloss, no texture, no depth, no
perspective, straight-on front view. Centred, compact, with generous empty space around it.
Plain flat solid mid grey background. No text, no letters, no numbers, no logo, no frame, no
border, no rounded square behind it, no people, no hands.
```

The ground is mid grey, not light grey: paper white and ink both key cleanly against it (round
one's light ground ate white rims, 3.7).

`{palette}` is the plate family's own two tones in words and hex (the object shares its plate's
family; round one's neighbouring-family palette is dropped): red "tomato red (#E8483C) and deep
brick red (#B7352B)"; amber "warm amber (#F0A81E) and deep ochre brown (#8E5A05)"; green "leaf
green (#28B24A) and deep forest green (#1A7A33)"; blue "cobalt blue (#2B7CFF) and deep
ultramarine (#0B5FE0)"; violet "soft violet (#8B5CF0) and deep indigo-violet (#6B3FCC)".

`{subject}` describes an arrangement of shapes, not an object: Mail "a paper white rectangle
with a folded triangular flap across its top, like a closed envelope reduced to two shapes";
Files "two paper folder shapes with rounded tabs, offset one behind the other"; Terminal "a
prompt chevron and a cursor block: an ink right-pointing chevron like a greater-than sign, then
a solid paper white bar, side by side on one line"; Notes "a square paper sheet with one lifted,
folded-over corner and three short lines"; Photos "a circle sun above a gentle hill shape inside
a rounded frame".

Style-trigger variant (style `flat-trigger`): the same brief prefixed with "Flat vector
illustration, minimalist, Swiss graphic design style."

Negative prompt (models that take one; proposed): `3D render, realistic, photo, glossy,
shading, bevel, depth of field, gradient, shadow, perspective, texture, text, letters,
watermark, frame, border, rounded square, app tile, busy detail, multiple objects, cropped`.
FLUX.2 Klein (distilled, cfg 1) takes no negative prompt; it applies to Qwen-Image only.

Fixed parameters per model are recorded in the recipe (3.8), not in prose.

Round three (2.9; `tools/icongen/icongen/brief.py` `EMBOSS_*`, styles `emboss` and
`emboss-tile`):

```
A single abstract geometric symbol: {subject}. The symbol is crafted into the surface, not drawn
on it: shallow embossed, slightly raised filled shapes in a pale tint of the surface colour, with
a subtle light edge on top and a soft darker edge below, soft internal geometry. No outlines, no
stroke, no edge lines, no borders around the shapes. Matte, smooth diffuse light, gentle, no
sharp highlights, no glow. Straight-on front view, centred, the symbol spans about half the width
with generous empty space around it. No text, no letters, no numbers, no logo, no people.
```

followed by, for `emboss` (the model draws the plate's face): "The whole image is one smooth
matte surface, edge to edge, with a soft two-colour gradient from {start} at the top left to
{end} at the bottom right. No tile, no frame, no border, no vignette."; or for `emboss-tile`
(the model draws the whole tile): "The symbol is crafted into a single rounded square tile with a
subtly beveled edge catching light in narrow spots: a thin highlight arc along the top edge and a
small soft point of light near one corner. The tile face has a soft two-colour gradient from
{start} at the top left to {end} at the bottom right. The tile is centred, straight on, filling
most of the frame, on a plain flat mid grey background. No shadow under the tile, no glow."

Negative: `frosted glass, glossy, outline, stroke, text, realistic, photo, 3D render, glow,
letters, watermark, border lines, drop shadow, busy detail, multiple objects`.

Round-three subjects: Mail "a closed envelope reduced to a rounded rectangle with a V-shaped flap
pressed into it"; Files "two rounded folder shapes with small tabs, one offset behind the other";
Terminal "a right-pointing chevron like a greater-than sign followed by a short horizontal cursor
bar, side by side"; Notes "a rounded square sheet with one folded-down corner and two short
raised bars"; Photos "a rounded frame holding a wide low hill across its bottom and a small sun
circle in its upper right corner, off to the side" (a centred circle over a hill reads as a
person).

### 3.4 Bake-off protocol (settled)

1. Same brief, same 5 subjects, each model. Proposed subjects: Mail (envelope), Files
   (folder), Terminal (small screen with a prompt caret, no letters), Notes (sticky note
   pad), Photos (instant camera or flower).
2. 4 seeds per subject per model (proposed), 1024 x 1024.
3. Every output goes through the Rust post-process (3.7) so the user compares finished
   icons on plates, not raw renders.
4. Contact sheet: rows = subjects, columns = model x seed; also a strip at 16/32/48 px
   on light and dark backgrounds. One PNG, reviewed by the user.
5. The user picks the model. If no model is acceptable, 3.10 applies.

### 3.5 Hero icon (settled)

The user picks one icon (proposed: Mail, the suite's first app) from the bake-off or a
second round on that subject. The hero is the visual DNA reference for every later icon.

### 3.6 The rest of the set (settled)

Every further icon is a reference-conditioned edit with the chosen model, passing the hero
(object layer, `master/<hero>.object.png`) as the reference image, plus the brief with the
new subject. Same model as the hero; no mixing models inside one set.

### 3.7 Post-processing in Rust (settled steps, proposed tool name)

A quire tool (proposed: `cargo run -p quire-icons -- build <app-id>`):

1. Background removal: the flat grey background is keyed out to alpha (the brief asks for a
   plain background so this is a colour-distance key, not a segmentation model; proposed
   threshold ΔE_OK < 0.03 from the sampled corner colour, 2 px feather).
2. Fit the object into the safe area (2.4).
3. Plate: gradient (2.3) masked by the squircle (2.1).
4. Composite object, then highlight and rim (2.5).
5. Export the size set and `@2` set (2.6), master with and without shadow.
6. Optional: vtracer (MIT) to SVG for `scalable/`; kept only if the SVG render differs from
   the 256 px raster by < 2 % mean per-channel error (proposed).
7. Write the acceptance report (6).

The tool is pure Rust (image 0.25; no Python in the post-process).

### 3.8 Reproducibility (settled)

Committed per icon, `icons/recipes/<app-id>.toml` (proposed fields):
`model`, `weights_sha256`, `comfyui_rev`, `workflow_sha256` (the workflow JSON is committed
next to it), `seed`, `steps`, `cfg`, `sampler`, `scheduler`, `width`, `height`, `prompt`,
`negative`, `reference` (path + sha256 of the hero object), `lora` (name + sha256 +
strength), `family`, `glyph_colour`, `postprocess_version`. Rendering the recipe again
must produce the same object PNG on the same GPU and driver (deterministic sampler, proposed
check in CI-less local run).

### 3.9 LoRA (settled)

After the hero set is approved (proposed: at least 8 icons), train a small LoRA on the
approved object layers (not the plated icons, so the model never learns the plate), locally
on the 16 GB GPU. Then the whole set is regenerable from recipes, and new icons use
brief + LoRA (+ hero reference where it helps). Trainer choice is open decision 5.

### 3.10 Fallback to hosted services (settled)

Only if the bake-off shows local quality is too low, and only when the user decides at that
point. Candidates recorded in PLAN: mcp-image over Gemini (Nano Banana) / OpenAI (GPT Image
edit), fal.ai or Replicate for Recraft V3 / FLUX Kontext. Their output terms get recorded in
section 5 before any output ships.

### 3.11 Timing (settled)

- 08-ICONS (template + brief) is Phase 0.
- M3 (dock) needs at least a placeholder set: the plate with the app's symbolic glyph at
  56 % in the plate's glyph colour (proposed placeholder form).
- The full ~20-icon set lands with M12 (app suite).

## 4. Third-party app icons

### 4.1 Plate-mask rule (settled rule, proposed numbers)

Every dock tile and launcher row has the same silhouette. A third-party app's own icon
(resolved from its `.desktop` `Icon=` through the icon theme: COSMIC icon theme, else
Cosmic, else hicolor, PLAN "use_apps") is drawn inside our plate:

- Plate: the neutral "paper" plate of 2.3 (proposed). Dark: `#2A2E28` → `#1D211B`
  (proposed, from the dark surface token `#1D211B` in palette.rs:177).
- Icon inset: the app icon's bounding box scaled to **72 %** of the plate side, centred
  (proposed).
- Source size: the smallest themed size ≥ 2 x the rendered size, else `scalable`, else the
  largest available.
- Cache: rendered once per (app-id, icon file mtime, size, scale, theme) under
  `$XDG_CACHE_HOME/sill/icons/` (proposed).

### 4.2 Icons that are already squircles or rounded squares (proposed)

Detect: the icon's alpha mask, scaled to our plate, has IoU ≥ 0.90 with the plate mask
(or with a rounded square of 15-25 % radius). Then:
- scale the icon so its shape fills the plate (100 %, not 72 %),
- re-mask with our squircle (cuts off its own corners; no double frame),
- skip our plate gradient (its own background is the plate).
Circle icons (IoU ≥ 0.90 with a circle) keep the 72 % rule.

### 4.3 Symbolic fallback (proposed)

No icon found, or the file fails to decode: the neutral plate with the Lucide `app-window`
glyph at 56 % in ink. No letters: icons carry no text (3.3).

## 5. Licensing notes to record

Recorded in `docs/licensing-references.md` (settled location, PLAN "Icons"):

| Item | Licence | What to record |
| --- | --- | --- |
| Lucide | ISC | notice file `assets/icons/LICENSE-lucide.txt` (moves from mailo), version/commit of the geometry |
| Tabler Icons | MIT | notice file `assets/icons/LICENSE-tabler.txt`, list of imported glyph names, version |
| Our glyphs | our licence | author, date |
| Qwen-Image-2512 weights | Apache-2.0 | model card URL, weights sha256, date |
| FLUX.2 Klein 4B weights | FLUX.2 Klein licence | exact licence text and whether outputs may be used commercially; verify before shipping |
| HiDream-O1-Image weights | MIT | as above, if used |
| LoRA | derived from our approved outputs | base model + its licence |
| ComfyUI | GPL-3.0 | tool only, never shipped or linked |
| vtracer | MIT | tool only |
| Hosted services (only if 3.10 happens) | provider terms | output ownership clause, date read |
| App icons (outputs) | our own licence | "generated with <model>, post-processed by quire-icons"; recipes committed |
| Third-party app icons | their own | never redistributed; rendered from the user's installed theme at runtime |

## 6. Acceptance

An app icon passes when all of these hold (checked by `quire-icons check`, proposed):

1. Silhouette: at every export size, the icon's alpha mask differs from `plate::mask(size)`
   by at most 1 px along any edge normal (compare the 0.5 alpha contour).
2. Safe area: no object pixel with alpha > 0.1 outside the 80 % safe square (2.4).
3. Contrast: glyph or object on plate ≥ 3.0 (WCAG ratio). For an object, measured between
   the mean colour of object pixels within 2 px inside its contour and the plate colour
   under them (proposed method); for a glyph, glyph colour against the plate's lighter stop.
4. Legibility at 16 px: the 16 px export is reviewed next to its 48 px export on light and
   dark bars in the gallery; the user can name the app from the 16 px alone (review item,
   not automatic). Proposed automatic proxy: the 16 px object covers ≥ 20 % of plate pixels.
5. Reproducible: the recipe re-renders the object (3.8).
6. Named correctly: files match 2.6 paths and the app id in the `.desktop` file.

A third-party tile passes 1 and 3 (plate vs its icon's mean edge colour) only.

## 7. Open decisions

- 2026-09-24 setup finding: FLUX.2 Klein 4B (Q4_K_M GGUF, 2.6 GB, Apache-2.0, ungated) runs on this machine in 22 s at 512 px with 8 GB VRAM free; Qwen-Image-2512 needs ~13 GB free and waits for the GPU; the smoke render is photographic, so the style brief must ask for a flat illustrated object, not a photo. Setup and workflow: `~/comfy/README.md`.
1. App-id namespace for our apps (placeholder `<ns>.Mail`). Needs a domain or
   `io.github.<user>.*`.
2. Do our app icons get a dark variant (dimmer plate, like macOS 26 tinted/dark icons)? Proposed: no.
3. SPEC "App integration" requires an SVG icon; we propose PNG set + optional vtracer SVG.
4. ComfyUI memory cap: 4 GB `limit4g` cannot load the weights; proposed a separate 16 GB cap.
5. LoRA trainer (candidates: ostris ai-toolkit, kohya musubi-tuner); pick after the bake-off
   picks the base model.
6. Superellipse exponent n = 5 and plate 824/1024: confirm by gallery comparison.
7. Neutral plate for third-party icons vs a plate tinted from the icon's dominant hue.
8. Glyph size 22 (`Bar`) needs a design source or sign-off.
9. Third-party inset 72 %: confirm on 10 real apps (Firefox, Chromium, VS Code, Steam,
   GIMP, Inkscape, LibreOffice, Thunderbird, Signal, Spotify) in the gallery.

## 8. Sources

- PLAN "Icons (settled 2026-09-23, third round)"; "Design: `<ds>`" (Glyph, icon files);
  Appendix A0 (C:1121-1129 principles), A3 (Candy shelf), A4 (icon base `.ic`).
- `~/mailo/crates/mail-app/src/ui/icon/mod.rs` lines 12-13, 21-54, 60-93, 96-131, 137-161,
  183-202, 271-293; `geometry.rs` lines 1-15.
- `~/mailo/crates/mail-app/src/palette.rs` lines 177, 219-238 (135deg gradient, dark surface).
- `~/mailo-design/mailo-spaces.html` lines 13-14, 32-33 (`--shadow-1/2`), 58 (`.ic`), and the
  size lines cited in 1.4.
- `~/mailo/crates/mail-app/src/ui/style/{controls,shell,list}.css` (`.ic` base and sizes).
- SPEC "Visual design system" (Typography and icons), "App integration" (reverse-DNS id, SVG
  icon), "Bundled native apps".
- Lucide, lucide.dev (ISC); Tabler Icons, tabler.io/icons (MIT).
- freedesktop Icon Theme Specification (hicolor, scale directories `@2`) and Icon Naming
  Specification (`-symbolic`).
- Apple Human Interface Guidelines, App icons (macOS 1024 canvas / 824 body grid; used for
  proportion only).
- Contrast numbers in 2.3: WCAG 2.x relative luminance, computed 2026-09-23 from the Candy hexes.
