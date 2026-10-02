# 21 Spaces on the desktop

A Space is a COSMIC workspace's colour: one flat, quiet tint on the **sidebar ground** of the
windows that have a sidebar, plus an optional grain. It never tints the content, a control, the
bar, the dock or any other shell chrome. This is the quiet Dia/Mac direction (`30-CATALOGUE.md`
section 3.4, which wins over this file); the gradient, frame-token, card-accent and A/B-layer model
is `archive/21-SPACES-arc.md`.

Status: **settled** = decided with the user (PLAN "UX decisions settled", 2026-09-23; the
2026-10-02 decisions in 30 section 3.4) or copied from code; **target (clean-up phase)** = settled
here, not yet in the code (`ds-style/space/palette.rs`, `SpaceEditor`, `SpaceStore`).
`palette.rs` = `crates/ds-style/src/space/palette.rs`.

## 1. The model (settled)

- Each COSMIC workspace owns one `SpaceLook`.
- The Space tint is one flat colour drawn as the sidebar's ground (under the Sidebar material's
  vibrancy). Nothing else changes colour with the Space: the content stays on paper, so every
  contrast rule holds, and shell chrome (bar, dock, launcher, control centre, OSD) takes no tint.
- Switching workspace cross-fades the tint over `--t-big`: no slide, no A/B layers.
- The workspace indicator in the bar shows the Space's dot (its tint) and name.

```rust
// target (clean-up phase)
pub struct SpaceLook { hue: f32, chroma: f32, grain: Grain(u8), theme: Theme }
```

| Field | Meaning | Range |
| --- | --- | --- |
| `hue` | the tint's hue, set by the preset swatch or the one hue slider | 0-360 |
| `chroma` | strength factor on the quiet tint: 1 for every colour preset, .06 for the neutral preset; not shown in the editor | 0-1 |
| `grain` | noise strength; **default 0 (none)** for new Spaces and every preset | 0-100 |
| `theme` | this Space's appearance: System / Light / Dark | enum |

There are no dots list, no gradient and no `card_accent`: the accent is the desktop's, set in
Settings (30 section 3.4), and a Space never changes it. Current code still has `dots`,
`card_accent` and `FrameVars`; they go.

**Motion is not part of a Space (settled, 2026-09-24).** How much a surface moves is global, the
`Appearance`'s `motion` resolved with the system's preferences into the root's `data-motion`.

## 2. The tint (settled values; the code's quiet tint)

Inputs: `h = hue`, `k = chroma`, `dark` from the resolved scheme. The chroma is the quiet tint
that has been in `palette.rs` since 2026-10-01 (about 40 % of the prototype's); the gradient and
extra stops of that derivation are dropped.

| Quantity | Light | Dark |
| --- | --- | --- |
| Tint (the sidebar ground) | `oklch(.936, k · .022, h)` | `oklch(.215, k · .018, h)` |
| Contrast loop | while ink on the tint is under 4.5 or faint ink under 3.0: `C -= .003` | same |
| Gamut fit | lower C by .002 until in sRGB | same |
| Grain opacity | `grain/100 × .10` | `grain/100 × .08` |
| Swatch / handle colour | `oklch(.74, k · .15, h)` | same |

Text on the tint is the neutral ink tokens (`--ink`, `--ink-soft`, `--ink-faint`), not Space-tinted
frame inks; the `--f-*` family is deleted. Hover and selection on the sidebar use `--sel-bg-quiet`
and the Sidebar material, as for any Mac source list.

## 3. Where the tint applies

| Surface | Tint? |
| --- | --- |
| A window's sidebar ground (mailo, files, settings, any `Sidebar`) | yes |
| Workspace indicator dot in the bar, Space dots in the editor and the sidebar foot | yes (a dot) |
| Bar, dock, launcher, control centre, OSD, notifications, power menu, lock, polkit | no |
| Content, list, reader, controls, apps' paper | no |

Status: **target (clean-up phase)**. The code still tints the bar, dock, launcher, control centre
and OSD through `data-frame="tinted"` and the `.ds-frame` group; that goes with the frame model.

## 4. Presets and defaults per workspace index

Eight presets, each one hue (settled hues from the former first dots; settled names, mailo's
`space/presets.rs::PRESET_NAMES`):

| # | Hue | Chroma | Name |
| --- | --- | --- | --- |
| 1 | 268 | 1 | Dusk |
| 2 | 152 | 1 | Orchard |
| 3 | 220 | 1 | Harbour |
| 4 | 20 | 1 | Ember |
| 5 | 190 | 1 | Lagoon |
| 6 | 340 | 1 | Heather |
| 7 | 95 | 1 | Moss |
| 8 | 250 | .06 | Stone (neutral) |

Default `SpaceLook` for a workspace with no stored look: the preset at the workspace's 0-based
position on its output modulo 8, **grain 0**, `theme = System`. Every preset has grain 0; the old
per-preset grains (35, 55, 40) are gone.

## 5. Workspace switch (settled model)

1. The sidebar ground's colour transitions from the old tint to the new over `--t-big`
   (`--e-in-out`); the grain opacity transitions with it.
2. Text does not change colour (it is neutral ink), so there is nothing else to fade.
3. No-op when the target is the current workspace.
4. Per output: each output's workspace indicator follows that output's active workspace.
5. Reduced motion keeps the cross-fade (30 section 1.1).

Trigger: cctk `WorkspaceState` activation events on the `cosmic_wl` thread, then `use_workspaces`,
then each sidebar's `Ds { look }`. The shortcut is ⌃1-9 (30 section 3.4); ⌘1-9 stay free for apps.

## 6. Editor (settled placement; target (clean-up phase) pieces)

Lives in two places, one component (`SpaceEditor` in quire): Settings, Spaces page (one row per
workspace, full editor), and the bar's workspace menu ("Edit Space…" opens it in a `Popover`).

| Piece | Spec |
| --- | --- |
| presets | the eight preset swatches as round swatches, border `--line`; picking one sets hue and chroma |
| hue slider | one `Slider` 0-360 with the hue ramp as its track; keys left/right hue 5 degrees; the swatch row follows |
| grain | `Slider` 0-100, default 0 |
| appearance | `SegmentedControl`: System / Light / Dark |
| preview | the Space's dot and name |

Gone: the 2-D hue-by-chroma field and its handle, the dot chips (up to three dots), the Accent
segment, the contrast pills and the cap note. Contrast is guaranteed by the tests of section 7,
never shown. Every edit writes immediately (no Save button) and the tint updates live with the §5
cross-fade.

## 7. Contrast guarantees (settled floors; tests only)

| Pair | Floor | Test (quire `space/palette` tests) |
| --- | --- | --- |
| `--ink` on the tint | ≥ 4.5 | `ink_on_tint` |
| `--ink-faint` on the tint | ≥ 3.0 | `faint_on_tint` |
| `--ink-soft` on the tint | ≥ 4.5 | `soft_on_tint` |

Run over the 8 presets x {light, dark}, plus a sweep of hue 0-355 step 5 at chroma {.06, 1}. The
sidebar pairs are also measured with the Sidebar material composited over pure black and pure white
backdrops. The accent contrast tests (accent on surface, ink on accent tint) belong to 03 section 20
and no longer depend on a Space.

## 8. Wallpaper (proposed)

The wallpaper is independent of the Space: it does not change on workspace switch, and the tint
never reaches it.

## 9. Apps opting in

- A window that has a `Sidebar` takes the Space tint on its ground; mailo does too. Nothing else
  in an app changes with the Space.
- API (proposed): `Ds { look: Some(look), .. }` on the app root; a running app learns the
  workspace's look through `ds-settings` (`use_environment`), keyed by the workspace its toplevel
  is on.

## 10. Storage (settled path; target (clean-up phase) schema)

`$XDG_CONFIG_HOME/quire/spaces.json`, atomic write (the `ds-settings` writer), watched with
`notify` (rename replaces inode; debounce 30 ms). `ds::style::space::store::SpaceStore` is the
schema (`crates/ds-style/src/space/store.rs`), read and written through `ds-settings`'s
`Settings<SpaceStore>` with `Format::Json`. A `null` in `by_index` means "no look stored at this
position".

```json
{
  "version": 2,
  "by_id": {
    "<cosmic workspace id>": { "hue": 268, "chroma": 1, "grain": 0, "theme": "system" }
  },
  "by_index": [ { "...": "SpaceLook for workspace 0 on its output" } ]
}
```

**Migration (version 1 to 2, settled 2026-10-02).** An existing Space keeps its **first dot's hue**
(and that dot's chroma factor, which is 1 for every preset but Stone); the extra dots and
`card_accent` are dropped; a `grain` value the user set is kept as it is (a stored 35 stays 35;
only Spaces with no stored look default to 0). `theme` is kept. A version-1 file is read, converted
and rewritten once.

Lookup (`SpaceStore::look_for_workspace`): `by_id[workspace id]` if the compositor gives a stable
id (ext-workspace `id` event), else `by_index[position]`, else the §4 default. Writes go to
`by_id` when an id exists and always also to `by_index`.

## 11. Open decisions

1. Does mailo's sidebar follow its own Space, the desktop workspace's, or its own unless unset?
   Proposed: its own, falling back to the workspace's.
2. Per-output workspaces: which look does a dock spanning one output use when the focused window is
   on another? Resolved by 3: the dock takes no tint, so none.
3. Whether COSMIC workspace ids are stable across sessions; decides whether `by_id` is useful.
4. Migration keeps the first dot's chroma factor as well as its hue (the decision names the hue
   only); confirm, or force chroma 1.

## 12. Sources

- PLAN "UX decisions settled with the user (2026-09-23, second round)", "Spaces on the desktop";
  `30-CATALOGUE.md` section 3.4.
- `crates/ds-style/src/space/palette.rs` (the quiet tint's constants).
- SPEC "Workspaces" (cctk, pinning, naming, reorder).
