# 10 BEHAVIOUR: Dock

Status legend used in every table: **settled** = decided with the user (plan, "UX decisions
settled with the user", 2026-09-23); **proposed** = chosen here, tune in the gallery, the user
may override. Confidence of the basis: **H** primary source or code, **M** reliable secondary
source or imitation code, **L** observed or inferred, **UNKNOWN** nothing published (we then
name the imitation we copy). "Plan" means `~/.claude/plans/vast-toasting-peach.md`; "C-A" means
its Appendix C section A.

## 10.1 What this governs

The dock surface of `sill` (`surfaces/dock/**`): its geometry at rest, the magnification curve
and how it follows the pointer, bounce, hover label, running indicator, badge and progress,
click, right-click, drag-to-reorder, drag-out-to-remove, file drops with spring-loading, the
minimize target, auto-hide, exclusive zone, input region and blur region. It does not govern
the icons themselves (`08-ICONS.md`), the menu component (`13-BEHAVIOUR-menus-windows.md` §13.3.2-13.3.3
and `04-COMPONENTS.md`) or the launch mechanics (plan, "Design: sill", Launch).

## 10.2 Reference behaviour (macOS)

| # | Fact | Value | Conf. | Source |
| --- | --- | --- | --- | --- |
| R1 | Default tile size | 48 pt, range 16..128 (`tilesize`) | H | C-A |
| R2 | Magnified size | slider up to 128 (`largesize`); magnification OFF by default | M | C-A |
| R3 | Orientation | left, bottom, right; never top | H | C-A |
| R4 | Usable area | excludes the dock | H | C-A |
| R5 | Magnification curve | not published (patent US 7,434,177 describes, no formula) | UNKNOWN | C-A |
| R6 | Best public imitation | Plank parabola: `offset = min(abs(cursor-center), zoomIconSize); p = offset/zoomIconSize; zoom = 1 + (1-p^2)(zoomPercent*progress - 1)`, influence +-1 magnified icon width, progress 0->1 on enter/exit | M | C-A |
| R7 | Follow | icon under the cursor tracks every mouse move with no easing; neighbours slide apart; dock widens; icons stick up above the background; baseline fixed; short grow on enter, ~0.2 s shrink on leave | L | C-A |
| R8 | Bounce kinds | `NSInformationalRequest` bounces 1 s; `NSCriticalRequest` bounces until the app is active; launch bounce repeats until launched | H | C-A |
| R9 | Bounce shape | amplitude ~1 icon height, decelerating up, accelerating down | L | C-A |
| R10 | Bounce switches | `launchanim`, `no-bouncing` defaults keys | H | C-A |
| R11 | Hover label | immediately above the icon, rounded vibrancy pill, system font; delay and size not published | L / UNKNOWN | C-A |
| R12 | Auto-hide | `autohide-delay` 0.2 s; slide ~0.5 s scaled by `autohide-time-modifier`; shows when the pointer touches the edge and stays for the delay; a ~4 px strip remains the trigger | M / L | C-A |
| R13 | Other keys | `static-only`, `show-process-indicators`, `showhidden` false | H | C-A |
| R14 | Minimize | genie default, scale, hidden `suck`; `minimize-to-application`; ~0.5 s; `slow-motion-allowed` | H / L | C-A |
| R15 | Click | click opens or activates | H | C-A (Apple support) |
| R16 | Cmd-click | reveals in Finder | H | C-A |
| R17 | Option-click | switches and hides the current app | H | C-A |
| R18 | Option-Cmd-click | switches and hides all others | H | C-A |
| R19 | File drag onto icon | opens the file with that app | H | C-A |
| R20 | Drag out | remove by dragging out "until Remove is shown"; running apps stay | H | C-A |
| R21 | Red badge | action needed | H | C-A |
| R22 | Right-click menu order | windows / recent documents; Options > (Keep in Dock, Open at Login, Show in Finder); Show All Windows; Hide; Quit (Force Quit with Option) | M | C-A |
| R23 | Click on running app with no windows | re-opens / unminimizes (which window: most recent) | L | C-A |
| R24 | Spring-loading delay | ~0.5 s (`com.apple.springing.delay`) | M | C-A |
| R25 | Stacks animations | not published | UNKNOWN | C-A |
| R26 | Geometry | running dot ~4-5 pt below the icon; red capsule badge top-right; thin vertical separator; rest not published (measure the Figma Big Sur Dock kit) | L / UNKNOWN | C-A |

## 10.3 Our behaviour (specification)

### 10.3.1 Geometry at rest (bottom dock, logical px)

| Value | Symbol | Number | Status | Basis |
| --- | --- | --- | --- | --- |
| Tile size | `T` | 48 | settled | R1, plan settled decisions |
| Magnified tile size | `S` | 96 (so max zoom `M = S/T = 2.0`) | settled | plan settled decisions |
| Gap between tiles | `g` | 4 | proposed | L; icon plates carry their own inset (08-ICONS), so the visible gap is `g + 2 x plate inset` |
| Pill padding (all four sides) | `pad` | 8 | settled | brief: equals the design card inset 8 (A1 `.win` padding 8) |
| Gutter, pill bottom to screen edge | `gut` | 8 | settled | same card-inset rule |
| Pill height | `H_p` | `T + 2 pad` = 64 | derived | |
| Pill width | `W_0` | `n T + (n-1) g + 2 pad` (+ separator block, 10.3.9) | derived | |
| Pill corner radius | `--m-radius` of `Material::Dock` | 22 | proposed (L) | `03-COLOR.md` materials table (Dock radius 22); the plan names the token only |
| Pill material | `Material::Dock` | tint + edge + shadow tokens from `03-COLOR.md`; compositor blur behind | settled | plan "Design: quire", Material enum |
| Pill tint with Spaces | `--f-*` frame tokens of the current workspace's SpaceLook, cross-fade 380 ms on workspace switch | settled | plan settled decisions (Spaces on the desktop) |
| Surface height | `H_s` | 160 = `gut + pad + S + T` (room for a magnified tile plus one bounce) | proposed | derived; transparent area outside the input region |
| Exclusive zone | | `Reserve(gut + H_p)` = 72 when auto-hide is off; `0` when on | settled | plan: `Reserve(base+margin)`; R4 |
| Horizontal placement | | pill centred on the output; clamped so it keeps 8 px from both output edges | proposed | L |
| Overflow at rest | | if `W_0 > output_w - 16`, `T` shrinks to the largest integer that fits (min 24); `S` keeps its ratio to `T` | proposed (M) | macOS shrinks the dock as it fills (commonly observed) |

### 10.3.2 Running indicator, badge, progress, separator

| Element | Specification | Status |
| --- | --- | --- |
| Running dot | circle 4 px diameter, centred under the tile, centre 4 px below the tile bottom edge (inside the 8 px bottom padding); colour `--f-ink` of the dock scope; shown iff the app has at least one toplevel (any workspace); fades in/out over `--t-quick` | proposed (R26 "4-5 pt below", M) |
| Badge capsule | anchored to the tile's top-right: right edge at tile right + 2, top edge at tile top - 2; height 18, min width 18, horizontal padding 5, radius 9; fill `--danger`, text `--accent-ink`-equivalent white token, ui font 11/700 tabular; text = count; `> 999` renders `999+`; scales with the tile (it is part of the tile's transform); appears with `pop-in --t-move --e-spring`, count changes use `bump` | proposed (R21 H for meaning, geometry L) |
| Badge source | `com.canonical.Unity.LauncherEntry.Update`: `count` + `count-visible` | settled (plan COSMIC findings) |
| Progress ring | circle 20 px outer diameter centred at (tile right - 8, tile bottom - 8), on a 22 px `--raise` disc; stroke 3, track `--ink` at the design's faint alpha, arc `--accent`, starts at 12 o'clock, clockwise, `progress` 0..1 from LauncherEntry `progress` + `progress-visible`; arc length changes are not animated faster than `--t-quick` | proposed (L; macOS draws a bar, the plan names a ring) |
| Urgent | LauncherEntry `urgent = true` starts a Critical bounce (10.3.5) | proposed (M) |
| Separator | 1 px wide, 36 px tall (0.75 T), `--f-line`, 6 px margin each side; present only when the Trash tile is present (10.3.9) | proposed (R26 L) |
| Pressed tile | a `--scrim` overlay clipped to the plate while the primary button is down | proposed (L: macOS darkens the pressed icon) |

### 10.3.3 Magnification

| Value | Number | Status | Basis |
| --- | --- | --- | --- |
| Default | ON | settled | plan settled decisions |
| Curve | Plank parabola, 10.5.1 | settled | plan settled decisions, R6 |
| Max zoom `M` | `S / T` = 2.0 | settled | |
| Influence radius `R` (distance from pointer to tile centre, in the un-magnified layout, at which zoom reaches 1) | 96 px = `S` (one magnified width) | settled for the curve, value proposed for tuning | R6 (M) |
| Which coordinate | pointer x only; pointer y inside the hit region has no effect | proposed (L, R7) |
| Follow while tracking | no easing, no smoothing, no interpolation: frame `k` uses the latest pointer x received before frame `k` | settled | R7 (L) |
| Enter | `progress` 0 -> 1 over 120 ms, curve `--e-out` | proposed (R7 "short grow", L) |
| Leave | `progress` 1 -> 0 over 200 ms, curve `--e-out`; pointer x frozen at the last value inside the hit region | settled (~200 ms), curve proposed |
| Re-enter while leaving | `progress` rises from its current value; remaining enter time = `(1 - progress) x 120 ms` | proposed |
| Baseline | tile bottoms stay on the line `gut + pad` above the output bottom; tiles grow upward; the pill background stays `H_p` tall and widens horizontally only | settled (R7) |
| Neighbours | slide apart; layout in 10.5.2 keeps the tile under the pointer under the pointer | settled (R7), rule proposed |
| Frozen | while a dock context menu is open, while the auto-hide slide runs, and while a drag ghost is outside the dock, zoom values freeze at their last frame; tracking resumes on the next pointer motion inside the hit region | proposed |
| Magnification OFF | zoom is 1 everywhere; hover still shows the label | settled |
| Rendering | icon raster chosen as the smallest cached size `>= displayed px x output scale` (sizes 48, 64, 96, 128, 192, 256) so magnified tiles never upsample | proposed |

### 10.3.4 Hover label

| Value | Number | Status | Basis |
| --- | --- | --- | --- |
| Component | ds `Tooltip{Fly}` (ink on paper, data font 10, padding 3/7, radius 6) | settled (brief: fly from the design) | A4 fly |
| Position | horizontally centred on the tile, bottom edge 7 px above the tile's current (magnified, not bouncing) top | proposed | A4 fly: `bottom: calc(100% + 7px)` |
| First delay | `--d-fly` = 350 ms from the pointer settling on a tile | settled (brief) | A4 fly, plan delay tokens |
| Warm | after a label has shown, moving to another tile shows its label with 0 ms delay; warm lasts until the pointer has been outside the hit region for 400 ms (`HoverWarm`) | proposed | A6 hover card warm rule |
| Follow | the label moves with its tile every frame (no easing), like the tile | proposed |
| Hidden | on primary press, while a context menu is open, while dragging, while auto-hide is hidden or sliding | proposed |
| Text | desktop entry `Name` (localised), single line, truncated per `02-TYPE.md` | proposed |

### 10.3.5 Bounce

| Kind | Trigger | Duration | Status |
| --- | --- | --- | --- |
| Launch | primary click on a tile whose app has no toplevel and was launched by us | repeats until `match_app` sees the app's first toplevel, the launched scope exits, or 10 s pass | settled (R8 H), 10 s cap proposed |
| Informational | `sill` IPC `Dock{Attention{app, Informational}}` (proposed IPC addition) | 1 s = 2 periods | R8 (H) |
| Critical | LauncherEntry `urgent = true`, or IPC `Dock{Attention{app, Critical}}` | repeats until the app has an activated toplevel or `urgent` clears | R8 (H) |

| Value | Number | Status | Basis |
| --- | --- | --- | --- |
| Amplitude `A` | one tile height at rest = `T` (48) | settled (brief), R9 (L) |
| Period `P` (up + down) | 500 ms | proposed (L) |
| Shape | ballistic: 10.5.3 (decelerates up, accelerates down, no squash) | R9 (L) |
| Settings | `dock.bounce` ON/OFF gates Informational and Critical; `dock.launch_animation` ON/OFF gates Launch | R10 (H) |
| Frames | a bouncing tile requests frames; all other tiles do not change | proposed |

### 10.3.6 Clicks

"Cmd" below means the physical Cmd key. Toshy remaps it for apps; on layer surfaces without
an app_id the arriving modifier is unverified (plan shell risks), so the dock accepts
**Super or Ctrl** as Cmd and **Alt** as Option.

| Input | Condition | Action | Status |
| --- | --- | --- | --- |
| Primary click (press and release on the same tile, no drag) | app not running | launch (activation token, DBusActivatable, `systemd-run` scope; plan "Launch") + Launch bounce | settled (R15) |
| | running, no toplevel on any workspace but process alive (DBusActivatable) | `org.freedesktop.Application.Activate` | proposed (R23 L) |
| | running, all toplevels minimized | unminimize and activate the most recently active toplevel | proposed (R23 L) |
| | running, not the active app | activate the most recently active toplevel of the app (COSMIC activates one toplevel; others are not raised) | proposed (R15 H; limit) |
| | running and active | cycle: activate the next toplevel of the app in MRU order on the current workspace; with one toplevel, nothing | settled (plan "click launch/activate/cycle") |
| Cmd-click | any | Show in Files: `org.freedesktop.FileManager1.ShowItems([desktop file uri])` | proposed (R16 H meaning) |
| Option-click | any | activate as primary click, then minimize every toplevel of the previously active app | proposed (R17 H) |
| Option-Cmd-click | any | activate as primary click, then minimize every toplevel of every other app on the current workspace | proposed (R18 H) |
| Middle click | any | launch a new instance / the desktop action `new-window` if present | proposed (Linux convention; macOS has none) |
| Secondary press, or primary press held 600 ms without moving 8 px | any | open the context menu (10.3.7) | proposed (press-and-hold duration L) |
| Scroll over dock | any | ignored (no rubber band on the dock, `11-BEHAVIOUR-scroll.md` §11.3.7) | settled (C-B H/M) |

Activation happens on **release** inside the same tile. Release outside the tile after a press
without a drag cancels.

### 10.3.7 Context menu

Opens on secondary **press** (press-drag-release selects, §13.3.2). ds `Menu{Context}` in an
`xdg_popup` anchored to the tile's rest rect, placement Top, gap 8, grab Seat. Items in order
(sections separated by `MenuEntry::Separator`):

1. Windows of the app: one item per toplevel (title), current workspace first then others
   under their workspace name as `Header`; checkmark on the active toplevel; minimized
   toplevels prefixed with the ds diamond glyph. Picking activates it. (R22 M)
2. Desktop actions from the entry's `Actions=` (plan). 
3. `Options >` submenu: `Keep in Dock` (check), `Open at Login` (check; writes or removes
   `~/.config/autostart/<id>.desktop`), `Show in Files`. (R22 M)
4. `Hide` (minimize every toplevel of the app), shown when running. (R22; macOS Hide is not
   minimize, this is the nearest COSMIC operation, proposed)
5. `Quit` (close every toplevel via `ToplevelManager::close`), shown when running. With Option
   held while the menu is open the item reads `Force Quit` and sends SIGKILL to the app's
   `app-sill-*` scope; hidden for apps we did not launch (no pid from the toplevel protocol).
   (R22 M)

`Show All Windows` is omitted in v1 (no per-app window overview in COSMIC). Recent documents
are omitted in v1. While the menu is open the dock is Frozen (10.3.3).

### 10.3.8 Drag inside, drag out, drops

| Value | Number | Status | Basis |
| --- | --- | --- | --- |
| Drag start | primary press on a tile, then pointer moves more than 8 px Manhattan | settled for the design (A6 drag "live after 8 px Manhattan") |
| Ghost | the tile image at its current size follows the pointer at the press offset; drawn in a dedicated Overlay layer surface (input region Empty, kept warm with `set_visibility`), because the pointer leaves the dock surface; motion keeps arriving through the implicit pointer grab of the press | proposed |
| Reorder | while the pointer is within the pill rect inflated by 64 px, the insertion index is the slot whose midpoint is nearest; tiles slide apart with `transform --t-move --e-spring`; magnification keeps tracking with the gap counted as a tile | proposed |
| Remove threshold | pointer outside the pill rect inflated by 64 px | proposed (R20 H behaviour, distance UNKNOWN) |
| Remove label | a `Tooltip{Fly}` reading "Remove" 7 px above the ghost, shown while beyond the threshold; no poof animation | settled (brief) |
| Release beyond threshold | pinned: unpin; the ghost fades with `fade --t-quick`; the gap heals `--t-move --e-spring`; UI sound `item-deleted` (§13.3.10). Running app: unpinned, its tile animates back into the running section | R20 (H) |
| Release inside | commit the new order to `~/.config/sill/dock.json` | proposed |
| Esc during drag | cancel, ghost returns to the origin slot `--t-move --e-spring` | proposed |
| File drop onto a tile | accepted iff the drag offers `text/uri-list` and at least one URI's MIME type is in the app's `MimeType=` (directories: `inode/directory`); the tile shows an `--accent` 2 px ring; drop launches via `expand_exec` (`%f %F %u %U`) or D-Bus `Open` | R19 (H) |
| Spring-loading | a DnD pointer resting on a running app's tile for 500 ms activates that app's most recent toplevel; the drag continues | R24 (M) |
| Drop between tiles | a `.desktop` file or a launcher item dropped between tiles pins it at that index | proposed |
| Magnification during DnD | tracks `wl_data_device` motion like pointer motion | proposed |

### 10.3.9 Items and order

`[pinned (dock.json order)] [running, unpinned, in launch order] | [Trash]`. Grouping by
`match_app` (plan). Trash tile: opens `trash:///` in the file manager, accepts file drops
(move to trash), context menu `Empty Trash` (UI sound `trash-empty`). The separator and the
Trash tile are **proposed** (R26); stacks are **deferred** (R25 UNKNOWN).

### 10.3.10 Minimize target

The dock calls `set_rectangle(toplevel, dock_surface, rect)` (cctk `ToplevelManager`) for every
toplevel of every tile with the tile's **rest** rect (not magnified), after each layout change,
debounced 100 ms. The compositor draws the minimize animation into that rect; its style and
duration are cosmic-comp's (R14 not reproducible from a client). Minimized windows do not get
their own tiles (`minimize-to-application` = ON, the only mode in v1). Status: proposed (M, cctk
API as used by cosmic-applets app-list).

### 10.3.11 Auto-hide

| Value | Number | Status | Basis |
| --- | --- | --- | --- |
| Default | OFF | settled |
| Show delay | 200 ms of the pointer inside the trigger strip | settled (R12 M) |
| Trigger strip | bottom 4 px of the output across the rest pill width (input region while hidden) | proposed (R12 L) |
| Slide | 500 ms, show with `--e-out`, hide with `--e-exit`; distance `gut + H_p + 8` (shadow clearance) | settled (~0.5 s), curves proposed |
| Hide trigger | pointer leaves the hit region: hide starts after 0 ms | proposed (L) |
| Stays shown | while a dock menu is open, while dragging, while a DnD hovers the strip | proposed |
| Exclusive zone | 0 | R4 inverse, proposed |

### 10.3.12 Input region and blur region

| Region | At rest (not hovered) | While hovered / magnified | Status |
| --- | --- | --- | --- |
| Input (`RegionSpec::Element("dock-hit")`) | pill rect extended down to the output edge (includes the gutter, so the screen edge hits the dock) | union of the above with the bounding box of all magnified tiles | settled (plan, brief) |
| Blur (`BlurSpec::Region` from `Element("dock-pill")`) | pill rect with `--m-radius` corners (`rounded_strips`) | the widened pill only; magnified tiles above the pill are not blurred | settled (plan) |
| Update rate | recomputed on every frame whose pill rect changed; committed with that frame's buffer | settled (plan shell-host `RegionSpec::Element`) |
| Auto-hide hidden | trigger strip only; blur region Empty | proposed |

## 10.4 State machine

Pure, in `sill-surfaces/src/surfaces/dock/machine.rs`; the component feeds events and applies
effects. `now` is always an argument; no clocks inside.

```rust
pub struct Px(pub f32);                       // logical pixels
pub struct TileId(pub u32);
pub struct Progress(pub f32);                 // 0.0..=1.0

pub enum Hover {
    Rest,
    Entering { from: Progress, since: Instant, x: Px },
    Tracking { x: Px },
    Leaving  { from: Progress, since: Instant, x: Px },
    Frozen   { progress: Progress, x: Px, cause: FreezeCause },
}
pub enum FreezeCause { Menu, Drag, Slide }

pub enum Label {
    Idle,
    Pending { tile: TileId, since: Instant },
    Shown   { tile: TileId },
    Warm    { until: Instant },               // 0 ms delay for the next tile
}

pub enum Bounce { Still, Bouncing { kind: BounceKind, since: Instant } }
pub enum BounceKind { Launch { deadline: Instant }, Informational, Critical }

pub enum Press {
    Up,
    Down { tile: TileId, at: Point, button: Button, since: Instant },
    Dragging { tile: TileId, offset: Point, over: DropZone },
}
pub enum DropZone { Slot(u32), Remove }

pub enum Reveal {                                  // only when auto-hide is ON
    Hidden,
    Armed   { since: Instant },
    Showing { since: Instant },
    Shown,
    Hiding  { since: Instant },
}

pub struct Dock { hover: Hover, label: Label, press: Press, reveal: Reveal,
                  bounces: Vec<(TileId, Bounce)>, spring: Option<(TileId, Instant)> }

pub enum DockEvent {
    PointerEnter { x: Px }, PointerMove { x: Px, y: Px }, PointerLeave,
    Press { button: Button, x: Px, y: Px }, Release { button: Button, x: Px, y: Px },
    Key(DockKey), DndMotion { x: Px }, DndLeave, Drop,
    MenuClosed, AppState { tile: TileId, change: AppChange }, Attention { tile: TileId, kind: BounceKind },
    Frame,                                          // wl_surface.frame callback
}

pub enum DockEffect {
    Redraw(Layout), RequestFrames, StopFrames,
    SetInput(RegionSpec), SetBlur(BlurSpec), SetExclusive(ExclusiveZone),
    ShowLabel { tile: TileId }, HideLabel,
    OpenMenu { tile: TileId }, Launch(TileId), Activate(TileId, ActivateMode),
    Reorder { tile: TileId, to: u32 }, Unpin(TileId), SpringOpen(TileId),
    Sound(SoundName), SetMinimizeRects,
}

pub fn step(dock: Dock, ev: DockEvent, now: Instant, p: &DockParams) -> (Dock, Vec<DockEffect>);
pub fn magnify(centers: &[Px], pointer_x: Px, progress: Progress, p: &MagnifyParams) -> Layout;
```

Hover transitions (P = progress):

| From | Event | To | Effects |
| --- | --- | --- | --- |
| Rest | PointerEnter / first PointerMove (cosmic-comp #2230: a still pointer gets no enter) | Entering{from: 0, since: now} | RequestFrames |
| Entering | Frame, `now - since >= (1-from) x 120 ms` | Tracking | Redraw |
| Entering / Tracking | PointerMove | same, x updated | Redraw on next Frame |
| Tracking | PointerMove with no change in x | Tracking | none (no frame) |
| Tracking | Frame with no pending move | Tracking | StopFrames |
| Entering / Tracking | PointerLeave | Leaving{from: P(now), since: now} | RequestFrames |
| Leaving | PointerEnter | Entering{from: P(now)} | |
| Leaving | Frame, P reaches 0 | Rest | Redraw, SetInput(rest), SetBlur(rest), StopFrames |
| any | OpenMenu / drag leaves the dock / Reveal sliding | Frozen{cause} | |
| Frozen | MenuClosed, drag ends, slide ends | Tracking if the pointer is inside, else Leaving | |

Label: Idle -> Pending on hover of a tile (`d_fly` 350 ms) -> Shown on Frame after the delay;
Shown -> Shown{other} immediately on tile change; Shown -> Warm{until: now + 400 ms} on leave;
Warm -> Shown{tile} with 0 ms on hover; Warm -> Idle when `now >= until`. Press, drag, menu:
-> Idle.

Press: Up -> Down on Press over a tile; Down -> Dragging when moved > 8 px Manhattan (primary
only); Down -> Up on Release inside the same tile (Launch/Activate by 10.3.6); Down -> Up +
OpenMenu when held 600 ms (checked on Frame or a timer effect); Dragging -> Up on Release (Unpin
or Reorder), on Esc (cancel).

Reveal (auto-hide ON): Hidden -> Armed on PointerEnter of the strip; Armed -> Showing when
`now - since >= 200 ms`; Armed -> Hidden on PointerLeave; Showing -> Shown after 500 ms; Shown ->
Hiding on PointerLeave unless a menu, drag or DnD is active; Hiding -> Showing on PointerEnter
(reverse from the current offset).

Timers are expressed as "earliest `now` at which a transition fires" and returned as a
`RequestFrames` or a one-shot timer effect by the component; the machine never sleeps.

## 10.5 Formulas

### 10.5.1 Zoom

For tile `i` with un-magnified centre `c_i` and pointer x `x`:

```
d_i  = |x - c_i|                               px, measured in the un-magnified layout
q_i  = min(d_i, R) / R                         0..1
zoom_i = 1 + (M - 1) * (1 - q_i^2) * P         P = displayed progress 0..1
size_i = T * zoom_i                            px
```

Constants: `T = 48`, `M = 2.0`, `R = 96`, `P` from 10.5.4. Note: the appendix transcribes
Plank as `1 + (1-q^2)(M*P - 1)`, which equals the form above at `P = 1` but yields `q^2 < 1`
(shrinking) at `P = 0`; we use the form above so that `P = 0` gives exactly 1.

Example (`P = 1`, pointer on a tile centre, pitch `T + g = 52`): centre tile 96 px; neighbours
at `d = 52`: `q = 0.5417`, `zoom = 1.7066`, 81.9 px; next at `d = 104 > R`: 48 px. Extra width
`E = sum((zoom_i - 1) T)` = 115.8 px.

### 10.5.2 Layout (pointer-anchored)

Slots tile the pill interior continuously: slot `k` spans `[a_k, b_k)` with
`a_k = c_k - (T + g)/2`, `b_k = c_k + (T + g)/2` (the outer slots extend to the pill padding
edge). Magnified slot width `w'_k = size_k + g`.

```
k   = slot containing x (clamp to first/last)
phi = (x - a_k) / (b_k - a_k)                  0..1
a'_k = x - phi * w'_k                          the slot under the pointer keeps the pointer at the same fraction
a'_{j+1} = a'_j + w'_j                         for j >= k (rightwards)
a'_{j-1} = a'_j - w'_{j-1}                     for j <= k (leftwards)
tile_left_j = a'_j + g/2 ; tile_bottom_j = gut + pad (baseline)
pill_left  = a'_0 - pad + g/2 ; pill_right = a'_{n-1} + w'_{n-1} + pad - g/2
if pill_left < 8:            shift everything right by 8 - pill_left
if pill_right > out_w - 8:   shift everything left by pill_right - (out_w - 8)
```

Invariant (property-tested): for every x inside the hit region and no clamp active, the tile
whose rest slot contains x also contains x after magnification, and the mapping is continuous
in x (slot boundaries map to the same point from both sides).

### 10.5.3 Bounce

```
u = ((now - since) mod P) / P                  0..1, P = 500 ms
lift(u) = A * 4 u (1 - u)                      px, A = T = 48; peak A at u = 0.5
```

Vertical velocity `A(4 - 8u)/P` falls linearly on the way up and rises on the way down (R9).
Informational stops at `now - since = 2P`; Launch and Critical stop at the end of the period
in which their stop condition becomes true (never mid-air).

### 10.5.4 Progress

```
enter:  P_lin = from + (now - since) / 120 ms           clamp 0..1
leave:  P_lin = from - (now - since) / 200 ms * from     clamp 0..1  (always 200 ms to 0)
P = bezier(.22, .9, .30, 1)(P_lin) on enter;  P = 1 - bezier(.22,.9,.30,1)(1 - P_lin) on leave
```

### 10.5.5 Auto-hide offset

```
showing: y_off(t) = D * (1 - e_out(t / 500 ms))       D = gut + H_p + 8 = 80 px
hiding:  y_off(t) = D * e_exit(t / 500 ms)
```

## 10.6 Configuration

Settings app keys, stored in `~/.config/sill/settings.json` under `dock` (pinned items stay in
`~/.config/sill/dock.json`, plan). Two-state values are enums, not bools (CONVENTIONS §0).

| Key | Type | Default | Range | Status |
| --- | --- | --- | --- | --- |
| `dock.magnification` | `On | Off` | `On` | | settled |
| `dock.tile_size` | px | 48 | 32..80 | settled default, range proposed |
| `dock.magnified_size` | px | 96 | `tile_size`..128 | settled default, range proposed |
| `dock.autohide` | `On | Off` | `Off` | | settled |
| `dock.autohide_delay_ms` | ms | 200 | 0..1000 | settled default |
| `dock.autohide_slide_ms` | ms | 500 | 0..1500 | settled default |
| `dock.position` | `Bottom | Left | Right` | `Bottom` | never Top (R3) | settled; Left/Right deferred |
| `dock.indicators` | `On | Off` | `On` | | proposed (R13) |
| `dock.bounce` | `On | Off` | `On` | | proposed (R10) |
| `dock.launch_animation` | `On | Off` | `On` | | proposed (R10) |
| `dock.click_active_app` | `Cycle | Nothing` | `Cycle` | | settled (plan), `Nothing` = macOS |
| `dock.trash` | `On | Off` | `On` | | proposed |

Gallery-only knobs (not in Settings): `R` (96), `g` (4), `--m-radius` (22, owned by 03-COLOR), enter 120 ms,
leave 200 ms, bounce `P` 500 ms, hold-to-menu 600 ms, remove threshold 64 px.

## 10.7 Integration

| Concern | Owner | Notes |
| --- | --- | --- |
| Dock surface, machine, `magnify` | `sill-surfaces/src/surfaces/dock/**`, pure helpers table-tested (plan names `magnify(centers, pointer_x, Params)`) | Top layer, anchor BOTTOM, `KeyboardMode::None` (OnDemand while a menu is open, shell-host rule) |
| Layer surface, regions, blur, popups, frame pacing | `shell-host` (`layer`, `region::rounded_strips`, `blur`, `popup`, `frame`) | input and blur regions via `RegionSpec::Element` resolved after layout each frame |
| Toplevels, activation, minimize rects, workspaces | `sill-services` `cosmic_wl` thread (cctk `ToplevelInfoState`, `ToplevelManagerState`) | second Wayland connection (plan: orphan rule) |
| Items, `match_app`, launch | `sill-services` apps + launch services | plan "Design: sill" |
| Badges, progress, urgent | `use_launcher_entry` (zbus match on LauncherEntry) | |
| Menu component | ds `Menu{Context}` | §13.3.2-13.3.4 behaviour |
| Label | ds `Tooltip{Fly}` + ds delay tokens | |
| Tint | `21-SPACES.md` SpaceLook of the active workspace | cross-fade 380 ms |
| Sounds | §13.3.10 | |

Platform limits:

- cosmic-comp #2230: a layer surface under a still pointer gets no enter; magnification and
  the label start on the first motion (the host synthesises PointerMove before a press).
- cosmic-comp #2777: delayed frame callbacks for fullscreen layer surfaces; the dock is never
  fullscreen, but the watchdog applies (plan frame machine).
- No urgency on toplevels in cctk: attention comes only from LauncherEntry `urgent` and our IPC.
- KWin (development host): no foreign-toplevel or workspace protocol, so no running dots,
  no window list, click = launch or D-Bus activate only, no minimize target; badges work.
  "Dock degrades quietly on KWin" (plan).
- Fullscreen windows cover the Top layer; the dock is not revealed over fullscreen in v1.
- Toshy modifier mapping on app_id-less layer surfaces is unverified; hence Super or Ctrl =
  Cmd in 10.3.6.
- Blur cost (cosmic-comp #2613/#2795): one blur region, only while the dock is visible.

## 10.8 Acceptance tests

Automatable through `sill debug {pointer,leave,stats,capture}` in nested cosmic-comp, the
headless renderer and pure table tests. Frame times refer to the output refresh `f`.

1. **Curve table** (pure): `magnify` for `P = 1`, pointer on tile 3 of 7: sizes
   `[48, 48, 81.9, 96, 81.9, 48, 48]` within 0.1 px; `P = 0` gives all 48 exactly.
2. **Anchor invariant** (property, 10 000 random pointer x per dock of 1..30 tiles, no clamp):
   the tile containing x at rest contains x magnified; `|layout(x + e) - layout(x)| -> 0` as
   `e -> 0` at every slot boundary.
3. **Hover sweep**: pointer driven across the dock at 800 px/s at pill mid-height for 1 s:
   every captured frame's tile sizes match `magnify(x_frame)` within 1 px, where `x_frame` is
   the last pointer x delivered before that frame; frames >= 0.9 x callbacks; p95 frame time
   < 1000/f ms (plan M0 c).
4. **No easing**: while Tracking, teleport the pointer 100 px; the next frame equals
   `magnify(new x)` exactly (0 px error).
5. **Leave**: after `leave`, every tile returns to 48 px within 200 ms + 1 frame; the pill
   returns to `W_0`; input and blur regions equal the rest rects in the same frame.
6. **Idle**: 5 s with the pointer still inside, and 5 s with it outside: 0 frames, 0 commits,
   <= 5 context switches (plan M0 c).
7. **Label**: hover a tile and hold still: label appears at 350 ms +- 1 frame, not before;
   move to the neighbour: its label appears within 1 frame; leave for 300 ms and return: 0 ms
   delay; leave for 500 ms and return: 350 ms.
8. **Bounce**: Informational: the tile's lift follows 10.5.3 within 1 px per frame and ends at
   1000 ms +- 1 frame; Critical: continues until a toplevel of that app is activated, then
   finishes the current period; Launch with a test app that maps a window at 1.3 s: stops at
   the end of the period containing 1.3 s (1.5 s).
9. **Exclusive zone**: a maximised foot window has height `output_h - bar_h - 72`
   (auto-hide Off) and `output_h - bar_h` (auto-hide On).
10. **Clicks** (scripted with a test app): launch; activate; cycle with two windows alternates
    them; all-minimized click restores the most recent one; release outside the tile does
    nothing.
11. **Drag out**: drag a pinned tile 100 px above the pill: "Remove" label visible; release:
    `dock.json` no longer contains it; running app variant keeps its tile in the running
    section; drag 40 px and release: order changed or unchanged per insertion index, nothing
    removed.
12. **Spring-loading**: a DnD (test source offering `text/uri-list`) resting 500 ms +- 1 frame
    on a running app's tile activates it; 400 ms and leave: not activated.
13. **Auto-hide**: pointer in the 4 px strip for 190 ms: stays hidden; 200 ms: slide starts and
    completes in 500 ms +- 1 frame; leaving the dock starts the hide immediately.
14. **Regions**: during a sweep, input region always contains every magnified tile's rect;
    blur region never extends above the pill's top edge.
15. **Headless snapshots**: rest, magnified at 3 pointer positions, badge 7 / 1234, progress
    0.4, label shown, dragging with Remove, in light and dark and two SpaceLooks.

## 10.9 Open decisions

1. Influence radius `R`: 96 (Plank, one magnified width) affects only one neighbour each side
   at 48/96; macOS appears to affect about two. Alternative: 144. Tune in the gallery.
2. Progress indicator: ring (plan) vs bar under the icon (macOS). Ring specified.
3. Trash tile and separator in v1 (proposed) or deferred with stacks.
4. Hover label font: the design's Fly uses Space Mono 10; macOS labels are ~13 pt. Keep Fly
   (coherence) or add a `size=dock` variant to ds.
5. `click_active_app`: Cycle (plan) vs Nothing (macOS).
6. Informational attention source on Linux: only our IPC today; map app notifications to
   Informational bounce or not.
7. Bounce period 500 ms is a guess (UNKNOWN); measure a macOS screen recording.

## 10.10 Sources

- Plan Appendix C-A (all R rows), "UX decisions settled with the user", "Design: sill" (Dock,
  Launch), "Design: shell-host" (`RegionSpec::Element`, `rounded_strips`, frame machine),
  "Findings: COSMIC" (cctk APIs, LauncherEntry, cosmic-comp #2230, #2777, #2613, #2795),
  M0 spike item c.
- Plan Appendix A: A1 (card inset 8), A4 (fly), A5 (tokens), A6 (hover warm, drag threshold),
  A7. Sibling docs: `03-COLOR.md` materials table (Dock tint, edge, shadow, radius 22),
  `01-LAYOUT.md` §13.2 open decisions 3-4 (answered here), `20-SURFACES.md` §1.2.
- Plank dock source (parabola), Apple Support "Use the Dock on Mac", Apple
  `NSApplication.RequestUserAttentionType` documentation, patent US 7,434,177 (as cited in
  C-A).
