# 03 Color

## 1. What this governs

This file fixes every colour: the paper tokens inside the card (light and dark), the frame
tokens around it and the arithmetic that derives them from a Space, the card accent, presets,
grain, shadows, the precomputed washes, identity colours, the candy hues from `C`, which token
goes on which element, the materials for shell surfaces, and how a SpaceLook attaches to a
desktop workspace. A colour that is not in this file does not exist; add it here first. Layout
is `01-LAYOUT.md`, type is `02-TYPE.md`, looks other than Post are `07-LOOKS.md`. Source keys
`S`, `C`, `P` are defined in `00-PRINCIPLES.md` section 1; `S` wins every conflict with `C`.

## 2. Two zones

Colour lives in two zones that never mix: the frame carries the Space, the card carries paper.

| Zone | Tokens | Who sets them | Rule |
| --- | --- | --- | --- |
| Frame | `--f-ink`, `--f-ink-soft`, `--f-ink-faint`, `--f-pill`, `--f-pill-hover`, `--f-line`, `--f-solid`, plus the gradient layers and grain | derived from the Space's dots and theme (section 4) | Only frame elements use `--f-*`. |
| Card | `--paper`, `--surface`, `--surface-2`, `--raise`, `--ink*`, `--line*`, `--accent*`, `--seal`, `--ok`, `--warn`, `--danger`, `--shadow-*`, `--scrim` | fixed Post palette in the Space's theme (section 3), accent per section 5 | "The colour fills the window **around** the Post card, never inside it" (`S:818-819`). |

`S:73-76` ("Every --f-* token is computed from the Space by the script."), `S:155` ("the card:
Post, untouched by the Space's hue"), `S:1194-1205`.

On the desktop the same split holds: bar, dock and launcher chrome are the frame; app content is
the card (`P:792-793`; `00-PRINCIPLES.md` section 8.2).

## 3. Paper tokens

The card uses the Post palette; `S`'s light and dark values are the canonical set.

```css
:root{
  --paper:#E9ECE6; --surface:#F8F9F6; --surface-2:#F1F3EE; --raise:#FFFFFF;
  --ink:#1A1E1A; --ink-soft:#586057; --ink-faint:#676E65;
  --line:#D6DBD0; --line-soft:#E3E7DE;
  --accent:#23508F; --accent-ink:#F4F8FF; --accent-soft:#DCE5F3; --seal:#23508F;
  --ok:#2C7A57; --warn:#A5761A; --danger:#B03A2A;
  --shadow-1:0 1px 0 rgba(255,255,255,.7) inset, 0 1px 2px rgba(26,30,26,.10);
  --shadow-2:0 1px 0 rgba(255,255,255,.7) inset, 0 6px 16px -6px rgba(26,30,26,.30);
  --scrim:rgba(0,0,0,.22);
  color-scheme:light;
}
:root[data-theme="dark"]{
  --paper:#151814; --surface:#1D211B; --surface-2:#232722; --raise:#2A2F28;
  --ink:#E7EBE3; --ink-soft:#A0A79B; --ink-faint:#8A9284;
  --line:#333A30; --line-soft:#282E26;
  --accent:#7FA6E6; --accent-ink:#0B142A; --accent-soft:#1E2A44; --seal:#7FA6E6;
  --ok:#5EB489; --warn:#D2A249; --danger:#E0705A;
  --shadow-1:0 1px 0 rgba(255,255,255,.05) inset, 0 2px 4px rgba(0,0,0,.45);
  --shadow-2:0 1px 0 rgba(255,255,255,.05) inset, 0 10px 22px -8px rgba(0,0,0,.7);
  color-scheme:dark;
}
```

`S:7-15`, `S:23`, `S:37-46` (non-colour tokens omitted; the dark block is repeated under
`@media (prefers-color-scheme: dark) :root:not([data-theme="light"])` at `S:25-36`)

| Token | Light | Dark | Role |
| --- | --- | --- | --- |
| `--paper` | `#E9ECE6` | `#151814` | page ground; inverse text on ink pills |
| `--surface` | `#F8F9F6` | `#1D211B` | card, list, rows |
| `--surface-2` | `#F1F3EE` | `#232722` | reader, composer, peek, mini, kbd |
| `--raise` | `#FFFFFF` | `#2A2F28` | selected row, popovers, hover card, command menu, menus, bubble, strip |
| `--ink` | `#1A1E1A` | `#E7EBE3` | primary text; inverse pill ground |
| `--ink-soft` | `#586057` | `#A0A79B` | secondary text |
| `--ink-faint` | `#676E65` | `#8A9284` | metadata |
| `--line` | `#D6DBD0` | `#333A30` | borders, dividers |
| `--line-soft` | `#E3E7DE` | `#282E26` | inner dividers |
| `--accent` | `#23508F` | `#7FA6E6` | Postmark accent (section 5) |
| `--accent-ink` | `#F4F8FF` | `#0B142A` | text on accent |
| `--accent-soft` | `#DCE5F3` | `#1E2A44` | accent tint |
| `--seal` | `#23508F` | `#7FA6E6` | defined, unused in `S` (the seal uses `--f-ink`, `S:346`) |
| `--ok` | `#2C7A57` | `#5EB489` | saved, live |
| `--warn` | `#A5761A` | `#D2A249` | star, dirty, attachment warning |
| `--danger` | `#B03A2A` | `#E0705A` | spoof flag, lying link |
| `--scrim` | `rgba(0,0,0,.22)` | same (not redefined) | behind peek and command menu |

**Status inks (settled, 2026-09-24).** Each status colour has an ink for text on it, as
`--accent-ink` is for `--accent`, and every `X` / `X-ink` pair clears 4.5:1 in both schemes
(`crates/ds/tests/legibility.rs::every_ink_on_its_colour_is_legible_in_both_schemes`):

| Token | Light | Dark | Ratio light / dark |
| --- | --- | --- | --- |
| `--ok-ink` | `#FFFFFF` | `#0B1A12` | 5.21 / 7.15 |
| `--warn-ink` | `#140D03` | `#140D03` | 4.78 / 8.27 (white on light `--warn` is 4.03) |
| `--danger-ink` | `#FFFFFF` (`S:436`'s `#fff`) | `#1A0B08` (mailo's) | 6.03 / 6.06 (white on dark `--danger` is 3.17) |

The card always takes the Post tokens of the Space's own theme, not the page's: the script writes
them onto `#card` and `#win` and sets `color-scheme` to match (`S:1196-1205`).

## 4. Frame tokens

The frame is derived, not chosen: a Space is up to three dots on a hue x chroma field plus a
grain and a theme, and every frame colour follows from them.

### 4.1 Static fallback

Before the palette is applied, `.win` carries these defaults:

```css
--f-ink:#1b1b22; --f-ink-soft:#44444f; --f-ink-faint:#5d5d6a;
--f-pill:rgba(255,255,255,.58); --f-pill-hover:rgba(255,255,255,.34); --f-line:rgba(0,0,0,.08);
```

`S:78-79`. `--f-solid` has no default; only the script sets it (`S:1191`).

### 4.2 Derived values

| Token | Light | Dark | Source |
| --- | --- | --- | --- |
| `--f-ink` | `oklch(0.22, 0.06·k, h0)` | `oklch(0.93, 0.018·k, h0)` | `S:997` |
| `--f-ink-soft` | `oklch(0.40, 0.05·k, h0)` | `oklch(0.80, 0.03·k, h0)` | `S:998` |
| `--f-ink-faint` | `oklch(0.52, 0.05·k, h0)` | `oklch(0.66, 0.035·k, h0)` | `S:999` |
| `--f-pill` | `rgba(255,255,255,.72)` | `rgba(255,255,255,.10)` | `S:1009` |
| `--f-pill-hover` | `oklch(0.885, 0.026·k + 0.004, h0)` | `rgba(255,255,255,.06)` | `S:1008` |
| `--f-line` | `rgba(0,0,0,.08)` | `rgba(255,255,255,.09)` | `S:1193` |
| `--f-solid` | first gradient stop | first gradient stop | `S:1191` |
| gradient stop `i` | `oklch(0.936 - 0.012·i, dot_i.c × 0.052, dot_i.h)` | `oklch(0.215 + 0.014·i, dot_i.c × 0.042, dot_i.h)`, each capped (4.3) | `S:990`, `S:1001-1007` |

`k` is the first dot's chroma (0..1 on the field) and `h0` its hue in degrees (`S:996`). Every
oklch value goes through `hex()`: gamut-fit, then sRGB-encode and round to `#rrggbb` (4.4).

The light hover is a solid colour darker than the frame; the dark hover is a white overlay. The
pill is always a white overlay.

### 4.3 The derivation, verbatim

```js
const FRAME = {light:{L:0.936, step:-0.012, C:0.052}, dark:{L:0.215, step:0.014, C:0.042}};
const PICK = {L:0.74, C:0.15};

/* Derive a whole palette from a Space's dots. Pure: dots + theme in, tokens out. */
function derive(space, dark){
  const f = FRAME[dark ? "dark" : "light"];
  const d0 = space.dots[0], hue0 = d0.h, k = d0.c;
  const ink = dark ? hex(0.93, 0.018 * k, hue0) : hex(0.22, 0.06 * k, hue0);
  const soft = dark ? hex(0.80, 0.03 * k, hue0) : hex(0.40, 0.05 * k, hue0);
  const faint = dark ? hex(0.66, 0.035 * k, hue0) : hex(0.52, 0.05 * k, hue0);
  let capped = false;
  const stops = space.dots.map((d, i) => {
    const L = f.L + i * f.step;
    let c = d.c * f.C;
    let hx = hex(L, c, d.h);
    while(c > 0 && (ratio(ink, hx) < 4.5 || ratio(faint, hx) < 3.0)){ c -= 0.003; capped = true; hx = hex(L, c, d.h); }
    return hx;
  });
  const hover = dark ? "rgba(255,255,255,.06)" : hex(0.885, 0.026 * k + 0.004, hue0);
  const pill = dark ? "rgba(255,255,255,.10)" : "rgba(255,255,255,.72)";
  const surface = dark ? "#1D211B" : "#F8F9F6";
  let aL = dark ? 0.77 : 0.45, aC = 0.045 + 0.035 * k;
  let accent = hex(aL, aC, hue0);
  while(ratio(accent, surface) < 4.5 && aL > 0.2 && aL < 0.95){ aL += dark ? 0.01 : -0.01; accent = hex(aL, aC, hue0); }
  const accentSoft = dark ? hex(0.29, 0.03, hue0) : hex(0.935, 0.018, hue0);
  const accentInk = dark ? hex(0.2, 0.02, hue0) : "#FFFFFF";
  const picked = space.dots.map(d => hex(PICK.L, d.c * PICK.C, d.h));
  return {stops, picked, ink, soft, faint, hover, pill, capped, accent, accentSoft, accentInk, surface, dark};
}

function gradient(p){ return p.stops.length === 1
  ? "linear-gradient(135deg," + p.stops[0] + "," + p.stops[0] + ")"
  : "linear-gradient(135deg," + p.stops.map((c, i) => c + " " + Math.round(i / (p.stops.length-1) * 100) + "%").join(",") + ")"; }
```

`S:990-1021` (comment lines 1010-1012 omitted), `S:1174-1176`

Rules the code encodes:

- Lightness is fixed per theme; the user controls hue and chroma only. "Lightness is fixed per
  theme (frame at `L 0.94` light, `0.21` dark, chroma at most `0.05`, all in OKLCH — the range
  Arc's own exported palettes sit in)" (`S:913-916`).
- A stop's chroma drops in steps of 0.003 until frame ink reaches 4.5:1 and frame faint reaches
  3:1 against it; `capped` records that it happened (`S:1005`, `S:1222-1224`).
- The gradient runs at 135 degrees with stops evenly spaced from 0% to 100%; one dot paints a flat
  colour (`S:1174-1176`).
- Frame ink is "a near-black tinted in the Space's hue" (`S:989`).

### 4.4 Colour arithmetic, verbatim

```js
function oklchToRgb(L, C, h){
  const a = C * Math.cos(h * Math.PI / 180), b = C * Math.sin(h * Math.PI / 180);
  const l_ = L + 0.3963377774*a + 0.2158037573*b, m_ = L - 0.1055613458*a - 0.0638541728*b, s_ = L - 0.0894841775*a - 1.2914855480*b;
  const l = l_*l_*l_, m = m_*m_*m_, s = s_*s_*s_;
  return [ 4.0767416621*l - 3.3077115913*m + 0.2309699292*s,
          -1.2684380046*l + 2.6097574011*m - 0.3413193965*s,
          -0.0041960863*l - 0.7034186147*m + 1.7076147010*s ];
}
const inGamut = rgb => rgb.every(v => v >= -0.0005 && v <= 1.0005);
function fit(L, C, h){ let c = C; while(c > 0 && !inGamut(oklchToRgb(L, c, h))) c -= 0.002; return Math.max(c, 0); }
const enc = v => { v = clamp(v, 0, 1); return v <= 0.0031308 ? 12.92*v : 1.055*Math.pow(v, 1/2.4) - 0.055; };
function hex(L, C, h){
  const rgb = oklchToRgb(L, fit(L, C, h), h).map(enc);
  return "#" + rgb.map(v => Math.round(v*255).toString(16).padStart(2, "0")).join("");
}
function lum(hx){
  const c = [1,3,5].map(i => parseInt(hx.slice(i, i+2), 16) / 255).map(v => v <= 0.03928 ? v/12.92 : Math.pow((v+0.055)/1.055, 2.4));
  return 0.2126*c[0] + 0.7152*c[1] + 0.0722*c[2];
}
function ratio(a, b){ const x = lum(a), y = lum(b); return (Math.max(x,y)+0.05) / (Math.min(x,y)+0.05); }
```

`S:965-984`

Gamut fitting lowers chroma in 0.002 steps (`S:974`). Contrast is WCAG 2 relative luminance with
the 0.03928 linearisation threshold (`S:981`). Contrast is measured on the rounded hex, so the
Rust port must round to 8-bit before measuring to reproduce `capped` exactly.

### 4.5 Application

```js
$("#grain").style.opacity = String(s.grain / 100 * (p.dark ? 0.16 : 0.2));
win.style.setProperty("--f-ink", p.ink); win.style.setProperty("--f-ink-soft", p.soft); win.style.setProperty("--f-ink-faint", p.faint);
win.style.setProperty("--f-pill", p.pill);
win.style.setProperty("--f-solid", p.stops[0]);
win.style.setProperty("--f-pill-hover", p.hover);
win.style.setProperty("--f-line", p.dark ? "rgba(255,255,255,.09)" : "rgba(0,0,0,.08)");
```

`S:1188-1193`

A Space switch paints the new gradient on the back layer and swaps opacities (380 ms cross-fade,
`S:86`, `S:1181-1186`); an edit repaints the front layer in place (`S:1187`). In the design system
these values are computed in Rust for the resolved scheme and written inline on the `.ds` root;
the `-l`/`-d` copies are deleted (`P:303-304`).

## 5. Card accent

Inside the card the accent is either Postmark blue or a quiet echo of the Space's hue; each
Space chooses.

| Mode | `--accent` | `--accent-soft` | `--accent-ink` | Source |
| --- | --- | --- | --- | --- |
| Postmark, light | `#23508F` | `#DCE5F3` | `#F4F8FF` | `S:1203` |
| Postmark, dark | `#7FA6E6` | `#1E2A44` | `#0B142A` | `S:1203` |
| Space, light | `oklch(aL, 0.045 + 0.035·k, h0)`, aL from 0.45 stepping -0.01 until 4.5:1 on `#F8F9F6` | `oklch(0.935, 0.018, h0)` | `#FFFFFF` | `S:1014-1018` |
| Space, dark | same, aL from 0.77 stepping +0.01 until 4.5:1 on `#1D211B` | `oklch(0.29, 0.03, h0)` | `oklch(0.2, 0.02, h0)` | `S:1014-1018` |

The accent loop stops at aL 0.2 or 0.95 even if 4.5:1 is not reached (`S:1016`). Why the Space
accent is quiet: it "keeps Postmark's weight and borrows only the Space's hue, at a quarter of the
chroma a free accent would have: a tint you notice on the unread dot and the selected row after a
while, never a second colour scheme" (`S:1010-1012`). Both default Spaces use `accent:"space"`
(`S:1027`, `S:1086`). The accent triple is written on both the card and the window, so frame
elements that use `--accent` (the destination ring) follow it (`S:1204`).

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
`every-pair-legible` test covers "2 schemes x 6 accents + every PRESET frame" (`P:366`).

## 7. Presets and default Spaces

Eight presets, each a list of one to three dots `{h, c}` with h in degrees and c in 0..1.

| # | Dots | Source |
| --- | --- | --- |
| 0 | `{268, .72}`, `{318, .55}` | `S:1118` |
| 1 | `{152, .62}`, `{62, .55}`, `{28, .5}` | `S:1118` |
| 2 | `{220, .7}` | `S:1118` |
| 3 | `{20, .66}`, `{55, .6}` | `S:1118` |
| 4 | `{190, .6}`, `{240, .55}` | `S:1119` |
| 5 | `{340, .6}`, `{290, .5}` | `S:1119` |
| 6 | `{95, .5}` | `S:1119` |
| 7 | `{250, .06}` | `S:1119` |

The two sample Spaces are presets 0 and 1:

| Space | Dots | Grain | Mode | Accent | Source |
| --- | --- | --- | --- | --- | --- |
| Work | `{268, .72}`, `{318, .55}` | 35 | system | space | `S:1027` |
| Home | `{152, .62}`, `{62, .55}`, `{28, .5}` | 55 | system | space | `S:1086` |

A Space's mode is `system`, `light` or `dark`; `system` follows the viewer's scheme
(`S:1157-1159`). A Space holds at most three dots; a new dot starts 48 degrees past the last one
with the same chroma (`S:1425`, `S:1463`). Keyboard nudges on a handle: hue ±5 degrees, chroma
±0.05 (`S:1455-1456`).

## 8. Grain

Grain is a 128 px grey-noise tile over the frame, its strength set per Space.

```js
const grainURL = (() => { try{
  const c = document.createElement("canvas"); c.width = c.height = 128; const x = c.getContext("2d");
  const img = x.createImageData(128, 128);
  let seed = 7; const rnd = () => (seed = (seed * 16807) % 2147483647) / 2147483647;
  for(let i = 0; i < img.data.length; i += 4){ const v = Math.floor(rnd()*255); img.data[i]=img.data[i+1]=img.data[i+2]=v; img.data[i+3]=255; }
  x.putImageData(img, 0, 0); return c.toDataURL();
}catch(e){ return ""; } })();
```

`S:1162-1168`

| Property | Value | Source |
| --- | --- | --- |
| Tile | 128x128, repeated (`background-size:128px 128px`) | `S:87`, `S:1163` |
| Generator | Park-Miller (Lehmer) PRNG, seed 7, multiplier 16807, modulus 2147483647 | `S:1165` |
| Pixel | grey `v = floor(rnd × 255)` in R, G and B; alpha 255 | `S:1166` |
| Blend | `mix-blend-mode:overlay` | `S:87` |
| Opacity | `grain / 100 × 0.20` light, `× 0.16` dark; grain is 0..100 | `S:1188`, `S:875` |
| Z | above the gradient layers (-1), below content | `S:87` |

The design system replaces the overlay blend (unsupported in Blitz) with a pre-rendered PNG built
from the same PRNG and seed as "128x128 alpha noise" (`P:288`, `P:421`).

`C` defines `--grain:.035` light and `.05` dark for Post and 0 for the other looks, but no rule
reads it (`C:34`, `C:99`, `P:1490-1491`).

## 9. The Space editor field

The picking field shows hue across and chroma down at the pick lightness.

```js
x.fillStyle = dark ? "#1b1d1a" : "#f3f4f1"; x.fillRect(0, 0, W, H);
const step = 18;
for(let py = step/2; py < H; py += step) for(let px = step/2; px < W; px += step){
  const h = px / W * 360, c = (1 - py / H) * PICK.C;
  x.fillStyle = hex(dark ? 0.66 : PICK.L, c, h);
  x.beginPath(); x.arc(px, py, 5.2, 0, Math.PI*2); x.fill();
}
```

`S:1399-1405`. Canvas 540x352 (`S:870`); handle position left = h/360, top = 1 - c (`S:1416-1417`);
handle fill = `oklch(0.74, c × 0.15, h)` (`PICK`, `S:991`, `S:1019`, `S:1418`). The handle's
fill does not switch to 0.66 in dark while the field dots do.

## 10. Shadows

Two elevation tokens plus a fixed set of one-off shadows, each tied to one surface.

| Token or surface | Light | Dark | Source |
| --- | --- | --- | --- |
| `--shadow-1` | `0 1px 0 rgba(255,255,255,.7) inset, 0 1px 2px rgba(26,30,26,.10)` | `0 1px 0 rgba(255,255,255,.05) inset, 0 2px 4px rgba(0,0,0,.45)` | `S:13`, `S:32` |
| `--shadow-2` | `0 1px 0 rgba(255,255,255,.7) inset, 0 6px 16px -6px rgba(26,30,26,.30)` | `0 1px 0 rgba(255,255,255,.05) inset, 0 10px 22px -8px rgba(0,0,0,.7)` | `S:14`, `S:33` |
| Window (`--shadow-window`, app windows) | `0 1px 3px rgba(0,0,0,.12), 0 24px 64px -16px rgba(0,0,0,.4)` | `0 1px 3px rgba(0,0,0,.4), 0 24px 64px -16px rgba(0,0,0,.66)` | settled 2026-09-24 (the macOS polish pass: a contact shadow under a wide soft ambient one); was `S:82`'s `0 18px 40px -22px rgba(0,0,0,.45)` |
| Card | `0 0 0 1px rgba(0,0,0,.06), 0 10px 30px -14px rgba(0,0,0,.45)` | same | `S:159` |
| Side peek | `0 20px 40px -16px rgba(0,0,0,.45)` | same | `S:454` |
| Peek | `0 24px 50px -18px rgba(0,0,0,.55)` | same | `S:224` |
| Command menu | `0 30px 60px -20px rgba(0,0,0,.5)` | same | `S:233` |
| Hover card, floating menu | `0 18px 40px -16px rgba(0,0,0,.45)` | same | `S:400`, `S:666` |
| Selection bubble | `0 12px 28px -12px rgba(0,0,0,.45)` | same | `S:684` |
| Floating composer | `0 20px 40px -18px rgba(0,0,0,.5)` | same | `S:374` |
| Frame: current item, pressed tile | `0 1px 0 rgba(255,255,255,.4) inset, 0 2px 6px -3px rgba(0,0,0,.25)` | same | `S:118`, `S:135` |
| Frame: command pill | `0 1px 0 rgba(255,255,255,.35) inset` | same | `S:95` |
| Editor handle | `0 0 0 1px rgba(0,0,0,.25), 0 3px 8px rgba(0,0,0,.35)` | same | `S:256` |
| Provider mark | `0 0 0 1px rgba(0,0,0,.08)` | same | `S:553` |
| Provider mark on tile | `0 0 0 1.5px var(--f-pill), 0 1px 2px rgba(0,0,0,.2)` | same | `S:555` |

The design system adds `--shadow-pop` and `--shadow-sheet` (`P:295`); which one-off shadows they
absorb is not specified.

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

The design system bans `filter`; `saturate()` becomes opacity (`P:75`, `P:421`).

**As tokens (settled, 2026-09-24).** The person hash is `ds::person_hue(address) -> PersonHue`
(painted by `AvatarTone::Person`, or `PersonHue::colour()`). The eight stored-colour swatches a
consumer hands out in order are `--c-person-1..8` (`ds::PersonSwatch`), mailo's `AVATAR` order:
`#5B4FC4`, `#2F7F6E`, `#B0662E`, `#3C8A5B`, `#7A4A9E`, `#C0782E`, `#2E7F8C`, `#6D7A3A` (the
accounts and pins above). They are data, declared once and the same in both schemes. A
consumer keeps no hex constants for people.

## 14. Usage map

Which token paints which element. Frame rows use `--f-*`; everything else is inside the card.

### 14.1 Frame

| Element | Rest | Hover | Current / pressed | Source |
| --- | --- | --- | --- | --- |
| Command pill | bg `--f-pill`, text `--f-ink-soft`, `.k` `--f-ink-faint` | text `--f-ink` | | `S:94-100` |
| Account tile | bg `--f-pill-hover`, text `--f-ink` | bg `--f-pill` | bg `--f-pill` + current shadow | `S:109-118` |
| Tile count `.n` | `--f-ink-soft` | | | `S:117` |
| Section header | text and button `--f-ink-faint`, rule `--f-line` | button `--f-ink` | | `S:120-125` |
| Sidebar item | bg transparent, text `--f-ink-soft`, count and x `--f-ink-faint` | bg `--f-pill-hover`, text `--f-ink` | bg `--f-pill`, text `--f-ink`, current shadow | `S:126-142` |
| Seal | `--f-ink` | | | `S:346` |
| Destination (archive target) | bg `--f-pill-hover` (source says `--f-hover`, undefined), text `--f-ink`, inset ring `--accent` | | | `S:447`, `P:1487` |
| Space name | `--f-ink` | | | `S:146` |
| Space dot | bg = that Space's gradient; border transparent | | border `--f-ink` | `S:147-150`, `S:1208` |
| Foot button | text `--f-ink-soft` | bg `--f-pill-hover`, text `--f-ink` | | `S:151-153` |
| Side peek | bg `--f-solid` | | | `S:454` |
| All-accounts tile icon | `--f-ink` | | | `S:549` |
| Today empty note | `--f-ink-faint` | | | `S:1257` |

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

`C` defines five hues with a text-safe deep member and a soft tint; `S` does not use them (every
chip is `--accent-soft`). The design system keeps them as label hues `--c-{red,amber,green,blue,
violet}{,-deep,-soft}` (`P:294-295`) and the app-icon template uses them as gradient families
(`P:812-813`).

| Hue | Base light | Deep light | Soft light | Base dark | Deep dark | Soft dark |
| --- | --- | --- | --- | --- | --- | --- |
| red | `#E8483C` | `#B7352B` | `#FBE3E1` | `#FF6B60` | `#FF8A80` | `#3A1E1C` |
| amber | `#F0A81E` | `#8E5A05` | `#FAEBD2` | `#F5BC4E` | `#F0B84A` | `#3A2C12` |
| green | `#28B24A` | `#1A7A33` | `#DCF1E1` | `#4FD06A` | `#6EDB86` | `#153520` |
| blue | `#2B7CFF` | `#0B5FE0` | `#E2EBFF` | `#4C9BFF` | `#6FB0FF` | `#14263F` |
| violet | `#8B5CF0` | `#6B3FCC` | `#ECE4FB` | `#A98BFF` | `#B69BFF` | `#271C42` |

`C:24-28`, `C:155-170`

`C`'s mapping in the Candy look: label chip = soft background, deep text; sidebar tag glyph and
its seal = `--tag` (blue, amber-deep, violet, green-deep by label); starred = amber-deep stroke,
amber fill, amber sparks; trash and purge hover = red-soft background, red-deep text; restore hover
= green-soft, green-deep (`C:935-966`). The Warm and Paper warmth steps retune the soft and deep
members (`C:841-924`); see `07-LOOKS.md`.

## 16. S versus C

`S` wins each row.

| Item | `S` | `C` | Source |
| --- | --- | --- | --- |
| Post accent, light | `#23508F`, ink `#F4F8FF`, soft `#DCE5F3` | `#C0402A`, ink `#FFF6F3`, soft `#F2DCD5` | `S:11`, `C:17` |
| Post accent, dark | `#7FA6E6`, ink `#0B142A`, soft `#1E2A44` | `#E9684B`, ink `#1A0F0B`, soft `#3A201A` | `S:30`, `C:94` |
| `--accent-2` | none | light `#2E5AA6`, dark `#7FA6E6` | `C:17`, `C:94` |
| `--seal` | `#23508F` / `#7FA6E6` | `#C0402A` / `#E9684B` | `S:11`, `S:30`, `C:19`, `C:95` |
| Seal colour in use | `--f-ink` (frame) | `--seal` | `S:346`, `C:317` |
| Scrim | `--scrim rgba(0,0,0,.22)` | peek scrim `--ink` at opacity .16; command wrap `rgba(0,0,0,.22)` | `S:15`, `C:1054-1060` |
| Grain | frame, per Space (section 8) | `--grain` token, unused | `S:87`, `C:34` |
| `--shadow-drag` | none | light `0 18px 30px -12px rgba(26,30,26,.40)`, dark `0 22px 34px -14px rgba(0,0,0,.8)` | `C:33`, `C:98` |
| Label chips | all `--accent-soft` | per-label candy in Candy look | `S:198`, `C:941-944` |
| Reference PNGs | show `C`'s red accent | | `P:986-987` |

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
| Root | `div.ds[data-theme][data-accent][data-motion][data-material][data-blur]` with `--f-*` inline | `P:320` |
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

| Material | Tint light | Tint dark | Edge | Shadow | Radius | Blur | Nearest precedent in `S` |
| --- | --- | --- | --- | --- | --- | --- | --- |
| Window | `--f-grad` + `.ds-layer` + `.ds-grain` | same | none | none | 0 (the card inside keeps 12/12/12/4) | none | `.win` (`S:77-87`) |
| Bar | `rgba(248,249,246,.70)` | `rgba(21,24,20,.68)` (settled 2026-09-24; .58 to .66 wave 1, to .68 over blur) | `inset 0 -0.5px 0 hairline` | none | 0 | behind | frame zone (section 4) |
| Dock | `rgba(248,249,246,.59)` (settled 2026-09-24; was .55) | `rgba(21,24,20,.68)` (settled 2026-09-24; .50 to .66 wave 1, to .68 over blur) | `inset 0 0 0 .5px rgba(255,255,255,.55)` + highlight | `0 10px 30px -10px rgba(0,0,0,.35)` | 22 | behind | pinned tiles radius 12 (`S:109`) |
| Popover | `rgba(255,255,255,.78)` | `rgba(42,47,40,.78)` | hairline + highlight | `--shadow-pop` = `0 18px 40px -16px rgba(0,0,0,.45)` | `--r-panel` 14 | behind | `.fmenu` (`S:665-666`) |
| Sheet | `rgba(248,249,246,.82)` | `rgba(21,24,20,.78)` | hairline + highlight | `--shadow-sheet` = `0 24px 50px -18px rgba(0,0,0,.55)` light / `0 30px 60px -20px rgba(0,0,0,.7)` dark | 18 | behind | `.peek` (`S:221-224`) |
| Toast | `rgba(248,249,246,.80)` | `rgba(21,24,20,.74)` | hairline + highlight | `--shadow-pop` | 16 | behind | `.toast` is inverse ink/paper in mail (`S:360-362`); the shell banner is a material, see open decision 12 |
| Osd | `rgba(248,249,246,.72)` | `rgba(21,24,20,.68)` (settled 2026-09-24; was .66) | hairline + highlight | `--shadow-pop` | 18 | behind | none |
| Widget | `rgba(248,249,246,.60)` (settled 2026-09-24; .50 to .54 wave 1, to .60 over blur) + grain | `rgba(21,24,20,.67)` (settled 2026-09-24; .45 to .65 wave 1, to .67 over blur) + grain | hairline + highlight | soft: `0 6px 16px -6px rgba(26,30,26,.30)` (`--shadow-2` drop) | 20 | behind | `.editor` / `.note` (`S:246`, `S:283`) |

Tinted shell chrome (bar, dock, launcher, control center) additionally carries the workspace's
`--f-*` frame tokens over the material (section 18 and `21-SPACES.md`): the material gives the
translucency and edge, the frame gives the hue.

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
white type reads on a pale picture. Under `LockLook::Space` the field and the date pill take the
Space gradient and the frame's ink instead (design/04 section 42).

### 17.4 Material stack v2 (settled 2026-09-24, the macOS polish pass)

macOS stacks layers on every chrome material that the section 17.2 recipe lacked, and the shell
looked flat beside it. Stack v2 adds them to every card-like material (Dock, Popover, Sheet,
Toast, Osd, Widget) and a bottom hairline to the Bar; the Window is unchanged. The tints' alphas
(and the legibility gates on them) do not move. Each layer is a `--m-*` variable a material block
declares and `--m-box` lists outside in; each alpha reads an input the root writes from a
settings key (`ds::MaterialStack`, `Ds { stack }`), with the default below behind it.

| Layer | Variable | Light | Dark | Key (proposed name, design/22) |
| --- | --- | --- | --- | --- |
| Vibrancy | baked into `--m-tint`, `--m-tint-solid` | OKLab chroma x1.4, lightness +.012 (surface `#f8f9f6` becomes `#fcfdf9`) | chroma x1.4, lightness kept (`#151814` becomes `#141813`, the raise `#2a2f28` becomes `#293026`) | `appearance.material_vibrancy`, percent, 100 (0 = section 17.2's flat colour, through `color-mix` on `--m-vibrancy`) |
| Outer hairline | `--m-hairline` | `0 0 0 .5px rgba(0,0,0,.14)` | `0 0 0 .5px rgba(0,0,0,.60)` | `appearance.material_hairline_light` 14, `_dark` 60 |
| Contact shadow | `--m-shadow-contact` | `0 1px 2px rgba(0,0,0,.10)` | `0 1px 2px rgba(0,0,0,.30)` | `appearance.material_shadow_strength`, percent, 100 (scales both shadows) |
| Ambient shadow | `--m-shadow-ambient` (= `--m-shadow`) | menus, toast, OSD `0 12px 40px -12px rgba(0,0,0,.28)`; sheet `0 24px 60px -18px .40`; dock `0 10px 30px -10px .35`; widget `0 6px 16px -6px rgba(26,30,26,.3)` | menus, toast, OSD `.55`; sheet `0 30px 70px -20px .65`; dock `.50`; widget `0 8px 20px -8px .50` | as above |
| Inner top highlight | `--m-highlight` | `inset 0 1px 0 rgba(255,255,255,.30)` | `inset 0 1px 0 rgba(255,255,255,.12)` | `appearance.material_highlight_light` 30, `_dark` 12 |
| Inner edge | `--m-edge` | `inset 0 0 0 .5px rgba(0,0,0,.08)` (the dock's `rgba(255,255,255,.55)`) | `rgba(255,255,255,.09)` | none (the `--f-line` values) |
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

Each desktop workspace owns a SpaceLook, and the shell chrome takes its frame tokens.

```rust
pub struct SpaceLook { dots: Vec<Dot>, grain: Grain(u8), theme: Theme, card_accent: CardAccent }
impl FrameVars { pub fn of(look:&SpaceLook, scheme:Scheme)->Self; pub fn style_attr(&self)->String }
```

`P:313-314`

| Field | Prototype equivalent | Range | Source |
| --- | --- | --- | --- |
| `dots` | `space.dots`, 1 to 3 `{h, c}` | h 0..360, c 0..1 | `S:1027`, `S:1425` |
| `grain` | `space.grain` | 0..100 | `S:875`, `S:1027` |
| `theme` | `space.mode`: system, light, dark | | `S:1027`, `S:1159` |
| `card_accent` | `space.accent`: space, post | | `S:1027`, `S:1201-1203` |

Settled rules:

| Rule | Source |
| --- | --- |
| Each COSMIC workspace has a SpaceLook | `P:792` |
| Bar, dock and launcher chrome use that SpaceLook's `--f-*` over compositor blur | `P:792-793` |
| Apps stay on paper | `P:793` |
| Switching workspace cross-fades the tint over 380 ms | `P:793-794` |
| A workspace-to-SpaceLook store exists, with default presets per workspace index | `P:794-795` |
| The editor lives in Settings and in the bar's workspace menu | `P:796` |
| Detailed spec goes in `design/21-SPACES.md` | `P:794` |

Precedent: `S`'s two Spaces are presets 0 and 1 in order and `Ctrl 1..9` switches Spaces by
index (`S:1027`, `S:1086`, `S:1669`).

## Open decisions

1. **Static frame fallback versus derived values.** `.win` defaults `--f-pill` to
   `rgba(255,255,255,.58)` and `--f-pill-hover` to `rgba(255,255,255,.34)` (`S:79`); the derived
   light values are `.72` and a solid `oklch(0.885, …)` (`S:1008-1009`). Whether the port keeps a
   fallback at all is not specified.
2. **Grain blend.** `S` blends an opaque grey tile with `overlay` (`S:87`, `S:1166`); the design
   system plans "alpha noise" PNG without blending (`P:288`). The alpha mapping that reproduces the
   overlay look is not specified.
3. **Danger wash.** `S` has two danger mixes, 16% over transparent (`S:278`) and 12% over
   `--raise` (`S:421`); the plan names one `--danger-wash` (`P:294`). Not resolved.
4. **Washes' precomputed values** for both schemes are not specified.
5. **`--shadow-pop` and `--shadow-sheet`** values and which one-off shadows in section 10 they
   replace are not specified (`P:295`).
6. **The six accents.** The design system carries mailo's `accents.css` "6 hues x 4 props"
   (`P:59`) and the gallery has an Accent(6) axis (`P:269`); their values are not in either
   prototype. How the six accents relate to Postmark and the Space accent is not specified.
7. **`--seal`** is defined but unused in `S` (the seal is `--f-ink`); whether the token survives is
   not specified.
8. **Editor handle in dark** uses the light pick lightness 0.74 while the field dots use 0.66
   (`S:1403`, `S:1418`). Not resolved.
9. **Every material value** in section 17.2 (tint, tint solid, edge, shadow, radius for all eight
   materials) is not specified.
10. **Material values** in 17.2 are proposed starting values, not measured against the prototypes;
    tune in the gallery. Whether the shell's notification banner is a translucent Toast material
    or mail's inverse ink-on-paper toast is open.
11. **Frame over blur.** The frame tokens were designed over an opaque gradient; over compositor
    blur, whether the bar and dock paint the Space gradient as their tint, a single stop, or
    `--f-solid` at some alpha is not specified (bar gaps' `FrameTint::Tinted`, `21-SPACES.md`
    section 3, answers this with the tint alpha; whether that reading is final is still open).
    The contrast gates in section 6 assume an opaque ground; how they hold over a blurred
    wallpaper is now specified for the tinted chrome materials — settled (2026-09-24), section
    17.2's six raised alphas clear 4.5:1 over both a pure black and a pure white backdrop, as a
    sensible default the user can retune later.
12. **Default preset per workspace index** (for example workspace i takes preset i mod 8) is not
    specified; `S` only shows presets 0 and 1 as the first two Spaces.
13. **Which theme wins** when a workspace SpaceLook says `light` and the system scheme is dark:
    per-Space mode is `S`'s rule (`S:1159`), but whether the shell honours a per-workspace theme
    is not specified.
14. **Destination token bug.** `S:447` uses the undefined `--f-hover`; the plan says to read
    `--f-pill-hover` (`P:1487`). Section 14.1 follows the plan.
15. **Identity colour source.** Settled (2026-09-24): a stored colour is one of the eight
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
