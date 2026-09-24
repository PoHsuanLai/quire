# 21 Spaces on the desktop

"A Space colours the frame; the mail stays on paper." (S:816). On the desktop, a Space is a
COSMIC workspace's look: its colour tints the shell chrome around the apps, never the apps.

Status: **settled** = decided with the user (PLAN "UX decisions settled", 2026-09-23, second
round) or copied from the prototypes/code; **proposed** = open until signed off.
S = `~/mailo-design/mailo-spaces.html`; palette.rs = `~/mailo/crates/mail-app/src/palette.rs`
(moves verbatim into `quire/crates/ds/src/space/palette.rs`).

## 1. The model (settled)

- Each COSMIC workspace owns one `SpaceLook`.
- Bar, dock, launcher and control-center chrome use the `--f-*` frame tokens derived from
  the active workspace's `SpaceLook`, drawn over compositor blur.
- Apps stay on paper (Post surface tokens: paper, surface, surface-2, raise, ink).
- Switching workspace cross-fades the tint over 380 ms.

```rust
pub struct SpaceLook { dots: Vec<Dot>, grain: Grain(u8), theme: Theme, card_accent: CardAccent }
pub struct Dot { hue: f32, chroma: f32 }        // palette.rs:16-23; chroma is 0..1 of the frame max
impl FrameVars { pub fn of(look: &SpaceLook, scheme: Scheme) -> Self; pub fn style_attr(&self) -> String }
```

| Field | Meaning | Range |
| --- | --- | --- |
| `dots` | 1-3 colours, left to right across the gradient (S editor: up to 3 dots) | hue 0-360, chroma 0-1 |
| `grain` | noise strength | 0-100 |
| `theme` | this Space's appearance: System / Light / Dark (S editor "Appearance" segment) | enum |
| `card_accent` | Postmark blue, or the Space's hue (S editor "Accent" segment) | enum |

An empty `dots` list reads as the neutral dot `{hue 250, chroma .06}` (palette.rs:31-35,
121-125), the same grey as preset 8.

## 2. Palette derivation (settled; quoted from Appendix A3 and palette.rs)

Inputs: `k = dots[0].chroma`, `h0 = dots[0].hue`, `dark` from the resolved scheme.

| Quantity | Light | Dark | palette.rs |
| --- | --- | --- | --- |
| Frame L, step per stop, max C | L .936, step -.012, C .052 | L .215, step +.014, C .042 | 83-94 |
| ink (`--f-ink`) | `oklch(.22, .06k, h0)` | `oklch(.93, .018k, h0)` | 132-136 |
| soft (`--f-ink-soft`) | `oklch(.40, .05k, h0)` | `oklch(.80, .03k, h0)` | 137-141 |
| faint (`--f-ink-faint`) | `oklch(.52, .05k, h0)` | `oklch(.66, .035k, h0)` | 142-146 |
| stop[i] | `oklch(L + i·step, dot.c · C, dot.h)`; while `ratio(ink, stop) < 4.5` or `ratio(faint, stop) < 3.0`: `C -= .003`, `capped = true` | same | 149-164, 105 |
| hover (`--f-pill-hover`) | `oklch(.885, .026k + .004, h0)` | `rgba(255,255,255,.06)` | 166-170, 109 |
| pill (`--f-pill`) | `rgba(255,255,255,.72)` | `rgba(255,255,255,.10)` | 171-175, 110-111 |
| accent (space mode) | `oklch(aL, .045 + .035k, h0)`, aL from .45 stepping -.01 until `ratio(accent, surface) ≥ 4.5` | aL from .77 stepping +.01 | 177-184, 107 |
| accent soft | `oklch(.935, .018, h0)` | `oklch(.29, .03, h0)` | 185-189 |
| accent ink | `#FFFFFF` | `oklch(.2, .02, h0)` | 190-194 |
| handle / picked swatch | `oklch(.74, c · .15, h)` | same | 96-100, 195-198 |
| gamut fit | lower C by .002 until in sRGB | same | 103, 270-276 |
| gradient | `linear-gradient(135deg, stops…)`, one stop repeated; positions `round(i/last·100)%` | same | 219-238 |

Frame tokens that are not in palette.rs (settled in S:1188-1193; `FrameVars` adds them):

| Token | Light | Dark |
| --- | --- | --- |
| `--f-line` | `rgba(0,0,0,.08)` | `rgba(255,255,255,.09)` |
| `--f-solid` | `stops[0]` | `stops[0]` |
| grain opacity | `grain/100 x .20` | `grain/100 x .16` |

Port notes (settled finding, fix in quire): palette.rs:177 hard-codes the card surface as
`#F8F9F6` / `#1D211B`; in quire it reads the `surface` token from the token table.

## 3. Where the tokens apply

| Surface | Frame tokens? | Background | Text |
| --- | --- | --- | --- |
| Bar | yes | Space gradient at material tint alpha over blur | `--f-ink-soft`, current/hover `--f-ink` |
| Dock pill | yes | Space gradient at material tint alpha over blur | `--f-ink` labels |
| Launcher panel chrome (field row, group headers) | yes | Space gradient | `--f-ink*` |
| Launcher result list | no | `--raise` | Post ink |
| Control center chrome | yes | Space gradient | `--f-ink*`; controls inside on `--f-pill` |
| OSD | yes (settled 2026-09-24) | Space gradient at material tint alpha over blur | `--f-ink*` |
| Notifications, power menu, lock, polkit | no (proposed) | their Material over Post tokens | Post ink |
| Apps | no (paper) | Post tokens | Post ink |
| Mail window frame | yes (opt-in, §9) | Space gradient + grain | `--f-ink*` |

Settled: bar, dock, launcher, control center and OSD (PLAN "UX decisions settled", this doc's
brief, and the OSD call below). The "no" row for notifications, power menu, lock and polkit
stays proposed.

Material and blur (settled mechanism, proposed alpha): `Material::{Bar,Dock,Popover}` paint
`--m-tint` = the Space gradient at alpha .80 (proposed) when `data-blur=on`, and
`--m-tint-solid` (alpha ≥ .94, settled) when blur is unavailable. The contrast tests in §7
run against both.

Settled implementation (bar gaps, sill Q9): a `Ds` root in Bar, Dock, Osd or Widget, or a
Popover root that is itself a panel (`chrome: Painted`, the launcher), stamps
`data-frame="tinted"` and draws §5's two layers and the grain as one `.ds-frame` group at
the material's own tint alpha from 03-COLOR §17.2 scaled by `appearance.material_tint_alpha`
(`--m-frame-alpha`: the bar .70 light, .68 dark at the default) with blur, and at .94 without.
This row used to read "no (proposed)" for OSD; the implementation tinted it as the bar-gaps
brief asked (coherence across chrome wins), and that is now the settled call — confirmed in the
goldens (`crates/ds/tests/snapshots/root/chrome/osd.html`) and the Materials gallery sheet,
where every tinted material including the OSD specimen shows the gradient. Over a pure black or
white backdrop, at wave 1's alphas, six material/scheme pairs fell short of 4.5; settled
(2026-09-24), the smallest further .02 raises close all six (03-COLOR §17.2 lists them;
`crates/ds/tests/legibility.rs`'s `the_tinted_chrome_holds_its_ink_over_blur` now holds for
every pair, with and without blur).

## 4. Presets and defaults per workspace index

The 8 presets (settled, S; Appendix A3):

| # | dots `[{hue, chroma}]` | Name in S |
| --- | --- | --- |
| 1 | `[{268,.72},{318,.55}]` | Work (grain 35) |
| 2 | `[{152,.62},{62,.55},{28,.5}]` | Home (grain 55) |
| 3 | `[{220,.7}]` | |
| 4 | `[{20,.66},{55,.6}]` | |
| 5 | `[{190,.6},{240,.55}]` | |
| 6 | `[{340,.6},{290,.5}]` | |
| 7 | `[{95,.5}]` | |
| 8 | `[{250,.06}]` (neutral) | |

Default `SpaceLook` for a workspace with no stored look (proposed):
- `dots = PRESETS[index % 8]`, index = the workspace's 0-based position on its output.
- `grain`: 35 for preset 1, 55 for preset 2 (settled values), 40 for presets 3-8 (proposed).
- `theme = System`; `card_accent = Postmark` (proposed).

## 5. Workspace switch (settled model)

The layer A/B model from S (Appendix A6 "Space switch"; S:1180-1187):

1. Each tinted surface has two background layers, `.ds-layer` (front) and `.ds-layer.back`
   (settled for every tinted root, bar gaps: on shell chrome they sit in `.ds-frame`, whose own
   background is the current gradient, so the group is opaque inside through the fade).
2. On switch, the hidden layer gets the new gradient and opacity 1; the front goes to
   opacity 0; the roles swap. Transition `opacity --t-scene (380 ms) --e-out`.
3. Grain opacity transitions over the same 380 ms (proposed).
4. Text colours (`--f-ink*`) change as values on `.ds`: if spike S4 (transition on
   var-driven values) passes, `color` transitions over `--t-scene`; if not, they swap at
   190 ms, the cross-fade's midpoint (proposed).
5. No-op when the target is the current workspace. Counts do not bump.
6. Per output: each output's bar follows that output's active workspace. The dock and
   launcher follow the workspace of the output they are on (proposed).
7. Reduced motion: the cross-fade uses the Reduced level (60 ms).

Trigger: cctk `WorkspaceState` activation events on the `cosmic_wl` thread → `use_workspaces`
→ each surface's `Ds { look }` (PLAN "Design: `<shell>`" hooks).

## 6. Editor (settled placement, pieces from 04)

Lives in two places, one component (`SpaceEditor`, proposed name, in quire):
- Settings → Spaces page: one row per workspace, full editor.
- Bar workspace menu: right-click (or the menu button) on a workspace dot → "Edit Space…"
  opens the editor in a `Popover` (proposed: the compact layout without provider marks).

Pieces (settled, Appendix A4 "Space editor", from 04-COMPONENTS):

| Piece | Spec |
| --- | --- |
| panel | pad 14, gap 14, r-panel, surface, shadow-1 |
| title | h3 15 + 14 px gradient swatch |
| field | h 176, r 12, crosshair; canvas 540 x 352, bg `#f3f4f1` / `#1b1d1a`, dot step 18, r 5.2, colour `oklch(dark .66 : .74, (1 - y/H)·.15, x/W·360)` |
| handle | 22 circle, border 3 white, shadow `0 0 0 1px rgba(0,0,0,.25), 0 3px 8px rgba(0,0,0,.35)`, `role=slider`, `.on` scale 1.15; keys left/right hue 5deg, up/down chroma .05 |
| stops | pill chips pad `3 5 3 4`, 14 disc + degrees + x 11; up to 3 dots |
| grain | range slider 0-100 (`Slider`) |
| segments | Appearance (System/Light/Dark), Accent (Space hue vs Postmark) |
| presets | 8 cols gap 6, round, border line, hover scale 1.1 |
| checks | pills ok/bad at 16 % wash: the four guarantees of §7 |
| capnote | 11.5 px, shown when `capped` |

Every edit writes immediately (no Save button, proposed) and the tint updates live with the
§5 cross-fade.

## 7. Contrast guarantees (settled floors; tests)

| Pair | Floor | Test (quire `space/palette` tests, proposed names) |
| --- | --- | --- |
| sidebar/chrome text `--f-ink` on every stop | ≥ 4.5 | `ink_on_every_stop` |
| faint `--f-ink-faint` on every stop | ≥ 3.0 | `faint_on_every_stop` |
| accent on card surface | ≥ 3.0 (derivation aims for 4.5) | `accent_on_card` |
| ink on accent tint (`accent_soft`) | ≥ 4.5 | `ink_on_accent_soft` |

Run over: 8 presets x {light, dark}, plus a sweep of hue 0-355 step 5 x chroma {0, .25, .5,
.75, 1} for 1-dot Spaces (proposed). Chrome pairs are also measured with the material tint
composited over pure black and pure white backdrops (PLAN test
`material-legible-over-black-and-white`). The existing mailo tests in
`palette.rs` `mod tests` move with the file.

## 8. Wallpaper (proposed)

The wallpaper is independent of the Space: it does not change on workspace switch, and the
tint sits only on shell chrome. Reason: a per-workspace wallpaper swap is a full-screen
repaint on every switch and would compete with the tint as the Space's colour claim.

## 9. Apps opting in

- Default: apps do not show the Space colour (paper only).
- Mail's window frame does (settled design): it renders its own `.ds-layer` A/B and grain
  from a `SpaceLook`.
- Opt-in API (proposed): `Ds { look: Some(look) , material: Window }` on the app root; a
  running app learns the workspace's look through `ds-settings` (`use_environment`), keyed by
  the workspace its toplevel is on.

## 10. Storage (settled path and schema)

`$XDG_CONFIG_HOME/quire/spaces.json`, atomic write (the `ds-settings` writer), watched with
`notify` (rename replaces inode; debounce 30 ms). **Settled** (2026-09-24, sill gap Q3):
`ds::SpaceStore` is this schema (`crates/ds/src/space/store.rs`), and `ds_settings::SPACES`
reads, writes and watches it through the generic settings file API
(`crates/ds-settings/src/spaces.rs`). A `null` in `by_index` means "no look stored at this
position".

```json
{
  "version": 1,
  "by_id": {
    "<cosmic workspace id>": { "dots": [{"hue": 268, "chroma": 0.72}, {"hue": 318, "chroma": 0.55}],
                               "grain": 35, "theme": "system", "card_accent": "postmark" }
  },
  "by_index": [ { "...": "SpaceLook for workspace 0 on its output" } ]
}
```

Lookup (settled; `SpaceStore::look_for_workspace`): `by_id[workspace id]` if the compositor gives a stable id
(ext-workspace `id` event), else `by_index[position]`, else the §4 default. Writes go to
`by_id` when an id exists and always also to `by_index`, so a restart that renumbers ids
still finds a look.

## 11. Open decisions

1. Mail has its own Spaces (Work, Home). Does mail's frame follow its own Space, the desktop
   workspace's, or its own unless unset? Proposed: its own, falling back to the workspace's.
2. Tint alpha over blur (.80 proposed).
3. Grain default 40 for presets 3-8.
4. Do notifications, OSD and power menu take the tint? Proposed no.
5. Per-output workspaces: which look does a dock spanning one output use when the focused
   window is on another? Proposed: the dock's own output.
6. Whether COSMIC workspace ids are stable across sessions; decides whether `by_id` is useful.
7. Wallpaper independence (§8).

## 12. Sources

- PLAN "UX decisions settled with the user (2026-09-23, second round)", "Spaces on the
  desktop"; "Design: `<ds>`" (`SpaceLook`, `FrameVars`, `Ds`, `BlurState`); Appendix A3
  (derivation, presets, grain), A4 (Space editor), A6 (Space switch).
- `~/mailo/crates/mail-app/src/palette.rs` lines cited in §2.
- `~/mailo-design/mailo-spaces.html` lines 816 (headline), 1180-1193 (layer swap, `--f-*`).
- SPEC "Workspaces" (cctk, pinning, naming, reorder).
