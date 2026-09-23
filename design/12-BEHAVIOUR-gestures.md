# 12 BEHAVIOUR: Magic Mouse gestures (`palmrest`)

Status legend: **settled** = decided with the user (plan, 2026-09-23); **proposed** = chosen
here, tune on the device or in the gallery, the user may override. Confidence of the basis:
**H** primary source or code, **M** reliable secondary source or imitation code, **L**
observed or inferred, **UNKNOWN** nothing published. "Plan" = `~/.claude/plans/
vast-toasting-peach.md`; "B" / "C-C" = its Appendix B / Appendix C section C. "Python daemon"
= `~/.local/lib/magic-mouse-kde/magic_mouse_gestures.py` v0.3.0-kde, the service running on
this machine today; "kernel" = `drivers/hid/hid-magicmouse.c` (master).

## 12.1 What this governs

`palmrest`, the Rust Magic Mouse 2 daemon that replaces the Python daemon at M8 (settled): the
hidraw report decoder, per-touch tracking, palm and rest rejection, the gesture set and its
detection thresholds, the configurable gesture-to-action table, the live-tracking model for
later, the outputs (a socket for our surfaces, a virtual touchpad and a virtual keyboard for
everyone else) and the exact hand-off to `11-BEHAVIOUR-scroll.md`. Pointer motion and physical
clicks stay in the kernel driver.

## 12.2 Reference behaviour (macOS)

| # | Fact | Value | Conf. | Source |
| --- | --- | --- | --- | --- |
| R1 | Secondary click | click on the right side | H | C-C (Apple 102482) |
| R2 | Scroll | one finger | H | C-C |
| R3 | Smart zoom | one-finger double-tap | H | C-C |
| R4 | Mission Control | two-finger double-tap | H | C-C |
| R5 | Swipe between pages | one finger left/right | H | C-C |
| R6 | Swipe between full-screen apps / Spaces | two fingers left/right | H | C-C |
| R7 | Clicks vs taps | clicks are physical; taps are used only for the double-taps | H | C-C |
| R8 | Live swipe API | `trackSwipeEventWithOptions`: handler receives fractional `gestureAmount` in pages + phase (Ended = commit, Cancelled = snap back); AppKit animates to -1/0/1 then `isComplete`; beyond min/max pre-dampened (0.5 -> ~0.125); WebKit Mac starts a swipe after 15 px horizontal | H | C-C |
| R9 | Spaces swipe thresholds | not published | UNKNOWN | C-C |
| R10 | Copyable thresholds (WebKit GTK) | progress = sum(dx)/400; cancel if `|p| <= 0.5` and `v*d < 0.4`; past 0.5 cancel only on a reverse flick; finish duration `|dp|/v * 3` clamped 100..400 ms, ease-out-cubic, base v 0.002/ms | H | C-C |
| R11 | Copyable thresholds (libadwaita swipe tracker) | base distances 400 / 300, velocity threshold 0.6, deceleration 0.997, drag threshold 16 px, 100..400 ms | H | C-C |
| R12 | Enabling multitouch | feature report `F1 02 01` | H | C-C, kernel |
| R13 | Report 0x12 | size 8 or `14 + 8N` (N <= 15); Bluetooth may wrap two in 0xF7 | H | C-C, kernel |
| R14 | Header | `data[1]` click bits (1 = left, 2 = right); dx s16 at [2..3], dy s16 at [4..5]; timestamp at [11..13], `ts = d11>>6 | d12<<2 | d13<<10`, unused by the kernel | H | C-C, B |
| R15 | Per touch (8 bytes `t0..t7` at `14 + 8i`) | `x = (t1<<28 | t0<<20) >> 20`; `y = -((t2<<24 | t1<<16) >> 20)`; major `t3`; minor `t4`; size `t5 & 0x3f`; id `((t6<<2) | (t5>>6)) & 0xf`; orientation `(t6>>2) - 32`; state `t7 & 0xf0` (0x30 start, 0x40 drag) | H | C-C, kernel |
| R16 | Surface | x -1100..1258, y -1589..2047 on 90.56 x 51.52 mm (~26 u/mm x, ~70.6 u/mm y) | H (kernel constants) | C-C |
| R17 | Firm touch | size >= 8 | H | C-C (kernel) |
| R18 | Button zones | x < -350 left, > 350 right, else middle (with `emulate_3button`) | H | C-C (kernel) |
| R19 | Palm / rest rejection | not published | UNKNOWN | C-C |
| R20 | MTTouch states | NotTracking, StartInRange, HoverInRange, MakeTouch, Touching, BreakTouch, LingerInRange, OutOfRange | M | C-C (OpenMultitouchSupport) |
| R21 | Kernel today | multitouch always reported to evdev (16 ABS_MT slots); hidraw and evdev coexist; no phase, no end | H | B |
| R22 | libinput | tags MM2 `ID_INPUT_MOUSE` only; MT data not turned into gestures; wheel source, `axis_value120`, no `axis_stop` | H | B |

This machine (verified 2026-09-23): `/etc/modprobe.d/99-magic-mouse-kde.conf` =
`scroll_acceleration=0 scroll_speed=32 emulate_scroll_wheel=0 emulate_3button=1`;
`/etc/udev/rules.d/70-magic-mouse-kde.rules` gives `uaccess` + mode 0660 to
`hidraw` with `KERNELS=="0005:004C:0269.*"` (Bluetooth MM2 only; the USB-C product id is
unverified, B).

## 12.3 Our behaviour (specification)

### 12.3.1 Report decoding

`decode(bytes: &[u8]) -> Result<Vec<Report>, DecodeError>` (pure):

| Rule | Specification | Status |
| --- | --- | --- |
| Report id 0x12 | accept iff `len == 8` (no touches, header only) or `len >= 14 && (len - 14) % 8 == 0 && (len - 14)/8 <= 15`; else `DecodeError::Length` (counted, dropped) | H (R13) |
| Report id 0xF7 | `first_len = data[1]`; decode `data[2 .. 2 + first_len]` and `data[2 + first_len ..]` as two reports, in that order | H (kernel) |
| Other ids | ignored (counted) | H |
| Buttons | `data[1] & 1` left, `& 2` right: used only for click suppression (12.3.4); the kernel emits the clicks | H |
| Mouse motion | `dx = i16::from_le_bytes(data[2..4])`, `dy = data[4..6]`: used only for rejection rule P5 | H |
| Device timestamp | 18 bits `(d11 >> 6) | (d12 << 2) | (d13 << 10)`, wraps at 2^18; unit UNKNOWN; calibrated at runtime (below) | H layout, unit UNKNOWN |
| Touch x | signed 12-bit: `((t1 & 0x0f) << 8 | t0)` sign-extended (kernel form) | H |
| Touch y | signed 12-bit `(t2 << 4 | t1 >> 4)` sign-extended, then negated (kernel form) | H |
| Other fields | major `t3`, minor `t4`, size `t5 & 0x3f`, id 0..15, orientation `(t6 >> 2) - 32`, state nibble `t7 >> 4` | H |
| Canonical frame | `+x` = rightwards across the mouse, `+y` = toward the user, both in raw units; a fixture test pins the sign against a recorded stroke | proposed |
| Python difference | the Python daemon reads x and y as **unsigned** 12-bit and fixes wraps with `wrap_delta` (modulo 4096), and does not negate y. Deltas are identical after sign handling; palmrest uses the kernel form and needs no wrap fix | H (both sources) |

Timestamp calibration (proposed): for the first 10 s after connect, fit `mono_ns = k * ticks +
c` by least squares over (arrival time, unwrapped device ticks); if the residual p95 < 1 ms,
events carry `t = k * ticks + c` (device time, jitter-free); else arrival `CLOCK_MONOTONIC`.
Re-fit every 60 s. Report rate is logged (unverified, B).

Multitouch enable: send feature report `F1 02 01` via `HIDIOCSFEATURE` once after open and
after every reconnect (idempotent; the kernel sends it at probe and resume). Needs the hidraw
node opened read-write (the udev rule grants 0660 + uaccess). Proposed (R12 H).

### 12.3.2 Touch tracking

Per id 0..15 a track follows the MTTouch model (R20). The state nibble is interpreted as the
MTTouch state number (M: consistent with the kernel's 0x30 = 3 MakeTouch start and 0x40 = 4
Touching drag, and with the Python daemon's "1-4 contact, 5-7 lift"):

| Nibble | MTTouch state | Counted as contact | Status |
| --- | --- | --- | --- |
| 0 | NotTracking | no | M |
| 1 | StartInRange | yes (Python parity) | settled by feel |
| 2 | HoverInRange | yes (Python parity) | settled by feel |
| 3 | MakeTouch | yes | H (kernel start) |
| 4 | Touching | yes | H (kernel drag) |
| 5 | BreakTouch | no | settled (Python) |
| 6 | LingerInRange | no | settled (Python) |
| 7 | OutOfRange | no | settled (Python) |

A touch is a **contact** iff its state is 1..4 and `size > 0` (Python `parse_report`). A
contact absent from a report is lifted. The **contact set** is the sorted tuple of contact ids
not rejected by 12.3.3; any change of the set (finger added or lifted) ends the current
gesture and starts a new one (Python `_start_touch`; test "finger count change does not
jump").

### 12.3.3 Palm and rest rejection

Apple's rules are UNKNOWN (R19). Rules applied to each contact; a rejected contact is removed
from the contact set for its whole lifetime.

| Rule | Specification | Status |
| --- | --- | --- |
| P1 Light touch | size < 8: may continue a gesture it is already part of, never starts one or counts toward a new contact set | proposed (floor from R17 H) |
| P2 Resting grip | a contact that lands in the side bands (`x < -900` or `x > 1060`) or the rear band (rear 15 % of the y span, sign pinned by fixture) and moves < 30 units in its first 150 ms is Resting | proposed (L, tune with `palmrest record`) |
| P3 Large contact | `major >= 64` raw is Palm | proposed (L, units unverified) |
| P4 Click in progress | while a physical button bit is set and for 200 ms after release: no gesture starts, taps are void, the running scroll gesture is Cancelled at press | proposed (R7 H: clicks are not taps) |
| P5 Mouse moving | while mouse motion exceeds 200 counts/s over the last 100 ms, the axis-lock threshold doubles (40 -> 80) | proposed (L) |
| Parity switch | `rejection = Off` disables P1-P3 and P5 for a bit-exact Python comparison | proposed |

### 12.3.4 The gesture set

All distances are raw units in the canonical frame (x ~26 u/mm, y ~70.6 u/mm, R16).

| Gesture | Detection | Default action | Status |
| --- | --- | --- | --- |
| G1 One-finger scroll | 1 contact; axis lock: accumulate centroid deltas, lock X when `|x| >= 40 && |x| >= 1.5 |y|` (Y symmetric), drop the accumulated pre-lock motion, unlock on a non-zero delta `>= 0.25 s` after the previous motion; units to px per 12.3.8 | scroll (11) | settled (Python) |
| G2 Two-finger scroll | 2 contacts, centroid of both; same lock; X lock becomes Blocked (never scrolls sideways) | scroll vertical (11) | settled (Python) |
| G3 Three or more contacts | nothing (scroll resets) | none | settled (Python `SCROLL_MAX_FINGERS 2`) |
| G4 One-finger horizontal swipe | 1 contact locked X; at lift: travel `|x| >= 300`, duration from lock to lift `<= 400 ms`, `|x| >= 2 |y|`, last motion `<= 50 ms` before lift | Back (fingers moved right) / Forward (left) | settled mapping (Alt+Left/Right), thresholds proposed |
| G5 Two-finger horizontal swipe | 2 contacts; centroid travel from landing; if `|y| > 150 && |y| > |x|`: re-anchor (reset both); when `|x| >= 200`: fire once in the direction the fingers moved, re-anchor, suppress if `< 0.5 s` since the last fire | Workspace: fingers right -> previous (left) workspace, fingers left -> next | settled (Python `SWIPE_THRESHOLD 200`, `SWIPE_VERTICAL_MAX 150`, `SWIPE_COOLDOWN 0.5`; direction "desktop follows the fingers") |
| G6 Two-finger double-tap | two taps, each: exactly 2 contacts at peak, all contacts down <= 200 ms, each contact moves < 40 units, no click bit; the second tap lands <= 350 ms after the first lifts, centroid within 300 units of the first; fires at the second tap's lift | Workspace overview | settled mapping, thresholds proposed |
| G7 One-finger double-tap | same timing with exactly 1 contact | Smart zoom where supported | settled mapping, thresholds proposed |
| G8 Secondary click | kernel: a click with a touch in the right zone (`x > 350`) is BTN_RIGHT | right click | settled (R1; kernel-owned, palmrest does nothing) |
| G9 Middle zone | kernel with `emulate_3button=1` (this machine): middle zone may give BTN_MIDDLE | middle click | kept as configured (L); macOS has none |

G4 vs scrolling: G4 is evaluated at lift and never suppresses the horizontal scroll that G1
already emitted during the stroke. On our surfaces the host decides instead (12.3.6); palmrest
does not emit G4 keys while a host claims the pointer.

G5 on a long drag keeps firing every 200 units after the 0.5 s cooldown (Python test "long
two-finger drag switches again after cooldown"); a two-finger vertical drag never switches;
one finger never switches (Python tests).

### 12.3.5 Scroll phases emitted

| Phase | When |
| --- | --- |
| MayBegin | a new contact set of 1 or 2 fingers lands |
| Began | the axis locks (first emitted delta is the first post-lock sample; pre-lock motion dropped) |
| Changed | every post-lock sample with a non-zero px delta on the locked axis |
| Ended | all contacts lift, the contact set changes, or the lock times out (then MayBegin, and Began at re-lock) |
| Cancelled | physical click (P4), a contact turns Palm (P3), device disconnect |

Blocked (two fingers locked X) emits no scroll phases; G5 owns those contacts.

### 12.3.6 One-finger swipe on our surfaces (host-decided)

When a host claims the pointer (12.3.10), a G1 stroke locked on X is a **page swipe** instead
of a scroll iff, at Began, the latched target (11 §11.3.3) cannot move in the stroke's
direction and the surface registered a navigation handler (`ds_native::use_navigation`). Then
the host runs the live model 12.3.7 with `D = 1000` raw x units per page (~38 mm) and
`Back/Forward` on commit. Otherwise it scrolls. WebKit's 15 px start distance is subsumed by
the 40-unit lock. Status: proposed (R5 H, R8 H).

### 12.3.7 Live tracking model (for later: workspace swipe with the content following)

| Value | Number | Status | Basis |
| --- | --- | --- | --- |
| Progress unit | pages; `p = sum(dx) / D`, finger-following sign | M (R8, R10) |
| `D` two-finger workspace | 1000 raw x units per page | proposed (L; R10 uses 400 touchpad px) |
| Beyond the first / last page | displayed `p' = bound + 0.25 (p - bound)`, capped at 0.25 page past the bound (0.5 -> 0.125) | proposed (R8 H for the 0.5 -> 0.125 pair) |
| Velocity | least-squares slope over the last 80 ms, pages/s | proposed |
| Commit | at lift: if `|p| >= 0.5`: commit unless `v * sign(p) <= -1.5 pages/s` (reverse flick); if `|p| < 0.5`: commit iff `v * sign(p) >= 1.5 pages/s` | proposed (R10/R11: libadwaita 0.6 px/ms over 400 px = 1.5 pages/s) |
| Finish duration | `clamp(|p_target - p| / max(|v|, 2 pages/s) * 3, 100 ms, 400 ms)`, ease-out-cubic | proposed (R10: base 0.002/ms = 2 pages/s) |
| One page per gesture | yes (live mode never fires twice per contact) | proposed (R8) |
| Output on COSMIC | no client protocol for a live workspace offset (ext-workspace has none; "no live workspace tracking yet"); route: synthetic 4-contact horizontal swipe on the palmrest virtual touchpad so the compositor runs its own live gesture | proposed (L: compositor finger count to verify) |
| Default today | Trigger mode (G5), live mode behind `workspace_swipe = Live` | settled ("trigger now, live tracking later") |

### 12.3.8 Units to px (the `feel` module)

Pure, shared by both outputs so foreign clients and our surfaces get the same distance:

```
px = raw * (detent_px / ((64 - speed) * 7)) * g_repeat * g_velocity      detent_px = 60, speed = 22
g_velocity = 1 + min((|raw| / dt) / 2000, 2.0 - 1)   if 0 < dt <= 0.05 s, else 1
g_repeat   = min(2.3, 7 / D); D = 7 at start; new contact set within 0.5 s of the last output: D = max(1, D - 1); else D = 7
```

(Python `_convert_axis_delta`, `_speed_gain`, `_start_touch`; settled feel, `detent_px = 60`
proposed, see 11 §11.3.8.) The Python quantisation to v120 units with remainders is kept only
in the Wheel output (12.3.9); the socket carries f32 px.

### 12.3.9 Outputs

| Output | For | Specification | Status |
| --- | --- | --- | --- |
| O1 Socket | our Blitz surfaces (every shell-host process) | `$XDG_RUNTIME_DIR/palmrest/events.sock`, `SOCK_SEQPACKET`, mode 0600, many subscribers; messages 12.3.10 | settled (plan B), protocol proposed |
| O2 Virtual touchpad | every other client, via the compositor and libinput | uinput device "palmrest touchpad", `BUS_VIRTUAL`, vendor 0x0001 product 0x0002; `INPUT_PROP_POINTER`; `EV_KEY` BTN_LEFT (never pressed), BTN_TOUCH, BTN_TOOL_FINGER / DOUBLETAP / TRIPLETAP / QUADTAP; `EV_ABS` ABS_X/Y, ABS_MT_SLOT 0..4, ABS_MT_TRACKING_ID, ABS_MT_POSITION_X 0..4000 and Y 0..2800 with resolution 40 units/mm (100 x 70 mm) | settled (plan B), fields proposed |
| O2 contact rules | never exactly one contact (a 1-finger touchpad contact would move the pointer); scroll = 2 contacts 20 mm apart moved by the identical delta (no pinch); contacts land at Began (never at MayBegin) and lift at Ended/Cancelled; if the stroke lasted < 200 ms or moved < 2 mm, the lift is delayed to 200 ms after landing (defeats libinput tap-to-click, tap timeout 180 ms); live swipe = 4 contacts | proposed (M: libinput behaviour) |
| O2 scale | virtual mm per px chosen so that a libinput 2-finger scroll in GTK4 moves the same px as the O1 path for the same stroke; start value 0.25 mm/px, calibrated by acceptance test 9 | proposed (L) |
| O2 direction | finger direction (a touchpad reports where the fingers go); the compositor's natural-scroll setting for this device applies (11 §11.3.13) | proposed |
| O3 Virtual keyboard | Keys actions | uinput "palmrest keys", only the keys used by the action table | proposed |
| O4 Wheel (fallback) | foreign clients when `foreign_output = Wheel` | exactly the Python device: name "Magic Mouse Scroll", `BUS_VIRTUAL`, vendor 0x0001 product 0x0001 (identity saved in Plasma `kcminputrc` with NaturalScroll=true), REL_WHEEL/HWHEEL + HI_RES, v120 quantisation with remainders | settled (Python) |
| Suppression | while any subscriber claims the pointer (`PointerOver{Ours}`), O2 and O4 emit nothing and G4 emits no keys | proposed (11 §11.3.14) |
| Kernel param | `emulate_scroll_wheel=0` stays (settled); palmrest refuses to start its outputs only if it cannot read hidraw, and logs loudly; it never changes module parameters | settled |

### 12.3.10 Socket protocol (O1)

```rust
pub enum ToHost {
    Hello   { version: u16, device: DeviceInfo },            // on connect and reconnect
    Scroll  { gesture: GestureId, phase: Phase, dx_px: f32, dy_px: f32,
              fingers: u8, axis: Option<Axis>, t_ns: u64 },   // finger direction, gains applied
    Swipe   { gesture: GestureId, phase: Phase, fingers: u8, dx_units: f32, t_ns: u64 }, // G4/G5 raw, host may run 12.3.7
    Tap     { kind: TapKind /* One | Two */, t_ns: u64 },     // G6/G7 fired
    Touches { active: ActiveTouch },                           // Down | Up: for double-scroll rule
    DeviceGone,
}
pub enum ToDaemon { PointerOver { over: Over /* Ours | NotOurs */ }, Subscribe { filter: Filter } }
```

`t_ns` is `CLOCK_MONOTONIC` (device-time mapped when calibrated). Latency budget: HID read to
socket write <= 2 ms p95.

### 12.3.11 Interaction with 11-BEHAVIOUR-scroll (exact)

| palmrest | shell-host engine (11) |
| --- | --- |
| `Touches{Down}` / MayBegin | stop momentum and rebound (touch to stop); start the double-scroll drop window |
| `Scroll{Began}` | latch (11 §11.3.3) on `axis`; or swipe decision (12.3.6) |
| `Scroll{Changed}` | `offset += natural(dx_px, dy_px)` on the latched axis, rubber band at edges |
| `Scroll{Ended}` | release velocity (11 §11.5.2) from the host's own samples of `t_ns`; momentum or rebound |
| `Scroll{Cancelled}` | rebound with `v0 = 0`, no momentum |
| `Touches{Up}` + 150 ms | end of the double-scroll drop window (unless host momentum is running) |
| `Tap{One}` | `SmartZoom` event to the focused ds root when the pointer is over our surface |
| `Swipe{fingers: 2}` | ignored by hosts in Trigger mode (the daemon fires the action) |

Momentum and rubber band are never computed in palmrest. Foreign clients get momentum from
their toolkit via O2's `axis_stop`.

### 12.3.12 Daemon architecture

| Part | Specification |
| --- | --- |
| Repo / crate | `magic-gestures` repo (plan repo map, M8), binary `palmrest`; no shell dependency; `unsafe_code = "deny"` except the ioctl module |
| Threads | one reader thread: `poll` on the hidraw fd, `read(64)`, decode, `Tracker::step`, outputs; one socket thread (accept, fan-out); no async runtime |
| Pure core | `decode`, `Tracker::step(tracker, report, now) -> (Tracker, Vec<Out>)`, `feel`, `gestures`, `action_for` |
| Device discovery | scan `/sys/class/hidraw/*/device/uevent` for vendor 004C product 0269 (Python `find_hidraw_device`); reconnect backoff 1 s x2 up to 30 s; 10 consecutive read errors = disconnect (Python) |
| Service | `palmrest.service` (systemd user, `graphical-session.target`, `Restart=on-failure`); installing it disables the Python service; uinput ACL wait loop as Python `wait_for_scroll_output` (1 s x2 up to 5 s) |
| Tools | `palmrest record FILE` (raw reports + monotonic times), `palmrest replay FILE` (drives the pure core; used by tests and tuning), `palmrest --check-uinput` |
| Config | `~/.config/palmrest/config.toml`, watched (debounce 30 ms); the Python env vars (`SCROLL_SPEED`, `SCROLL_LOCK_*`, `SCROLL_VELOCITY_GAIN_MAX`, `SCROLL_ACCEL_MAX`, `SWIPE_*`) still override for parity runs |

## 12.4 State machine

```rust
pub struct Units(pub f32);                       // raw device units
pub enum Lock { Free { pend_x: Units, pend_y: Units }, X, Y, Blocked }
pub enum Contact { Light, Firm, Resting, Palm }

pub enum ScrollG {
    Idle,
    Touching { set: ContactSet, lock: Lock, last_motion: Instant, gesture: GestureId },
}
pub enum SwipeG {                                // G5, Trigger mode
    Idle,
    Pair { set: ContactSet, travel_x: Units, travel_y: Units },
}
pub enum TapG {
    Idle,
    Down  { fingers: u8, since: Instant, moved: Units, origin: Point },
    Gap   { fingers: u8, lifted: Instant, origin: Point },          // first tap done
    Down2 { fingers: u8, since: Instant, moved: Units, origin: Point },
}
pub struct Tracker { scroll: ScrollG, swipe: SwipeG, tap: TapG, one: OneSwipe,
                     repeat: RepeatGain, last_output: Option<Instant>, last_fire: Option<Instant>,
                     click_until: Option<Instant>, claimed: Over }

pub enum Out { Host(ToHost), Pad(PadFrame), Wheel(WheelDelta), Key(KeyCombo), Action(Action) }

pub fn step(t: Tracker, r: &Report, now: Instant, cfg: &Config) -> (Tracker, Vec<Out>);
```

Scroll (per report): contact set changed -> Ended (if Touching) + start (repeat-gain update,
Free) + MayBegin; Free -> X/Y/Blocked per the lock rule (pending dropped) -> Began; X/Y with
a delta `>= 0.25 s` after `last_motion` -> Ended, Free (delta added to pending); X/Y -> Changed;
Blocked -> nothing (still updates `last_motion`); contacts < 1 or > 2 -> Ended, Idle.

Swipe (Trigger): exactly 2 contacts else Idle; new pair -> Pair{0,0}; accumulate centroid;
vertical rule re-anchors; `|travel_x| >= 200` -> re-anchor and fire iff `now - last_fire >=
0.5 s`.

Tap: Idle -> Down on landing (no click, not in P4 window); Down -> Idle if moved >= 40 or
`now - since > 200 ms` or finger count changes; Down -> Gap on full lift; Gap -> Idle at
`lifted + 350 ms`; Gap -> Down2 on landing within 300 units; Down2 -> fire + Idle on lift under
the same limits. Taps never emit scroll (their motion stays under the lock threshold).

## 12.5 Formulas

```
centroid delta   = (1/n) sum_i (p_i(now) - p_i(prev))              n = 1 or 2 contacts, per axis
axis lock        = |a| >= 40 and |a| >= 1.5 |b|                     raw units (80 when P5 active)
px               = 12.3.8
mm (for tuning)  = x / 26.0 ; y / 70.6                              R16
swipe progress   = sum(dx) / D ; displayed p' = 12.3.7
commit           = 12.3.7
finish duration  = clamp(|dp| / max(|v|, 2) * 3 s, 0.1, 0.4) ; ease-out-cubic
virtual pad mm   = px * 0.25 (calibrated)
```

## 12.6 Configuration

`~/.config/palmrest/config.toml`, edited by the Settings app ("Mouse & Gestures" page).

| Key | Default | Range | Status |
| --- | --- | --- | --- |
| `scroll.speed` | 22 | 0..63 | settled (Python) |
| `scroll.lock_threshold` | 40 | 10..200 | settled |
| `scroll.lock_ratio` | 1.5 | 1.0..4.0 | settled |
| `scroll.lock_timeout_ms` | 250 | 0..2000 | settled |
| `scroll.velocity_gain_max` | 2.0 | 1.0..4.0 | settled |
| `scroll.repeat_gain_max` | 2.3 | 1.0..4.0 | settled |
| `scroll.detent_px` | 60 | 20..200 | proposed |
| `swipe.threshold` | 200 | 50..1000 | settled |
| `swipe.vertical_max` | 150 | 20..1000 | settled |
| `swipe.cooldown_ms` | 500 | 0..2000 | settled |
| `swipe.workspace_mode` | `Trigger` | `Trigger | Live` | settled |
| `swipe.repeat` | `AfterCooldown` | `AfterCooldown | OncePerGesture` | settled (Python), see open decisions |
| `tap.max_ms` / `tap.gap_ms` / `tap.max_move` | 200 / 350 / 40 | | proposed |
| `rejection` | `On` | `On | Off` | proposed |
| `foreign_output` | `Touchpad` | `Touchpad | Wheel | Off` | proposed |

Gesture -> action table (every gesture configurable; Apple mapping shipped):

```rust
pub enum Gesture { OneFingerSwipeLeft, OneFingerSwipeRight, TwoFingerSwipeLeft,
                   TwoFingerSwipeRight, TwoFingerDoubleTap, OneFingerDoubleTap }
pub enum Action {
    None, Back, Forward,                          // Keys Alt+Left / Alt+Right on O3
    WorkspacePrev, WorkspaceNext, WorkspaceOverview,
    SmartZoom,                                    // our surfaces only; foreign: None
    Keys(KeyCombo), Command(Vec<String>), ShellIpc(String),
}
```

| Gesture | Default (Apple) | COSMIC backend | KWin backend (today) | Status |
| --- | --- | --- | --- | --- |
| One-finger swipe right | Back | Alt+Left on O3 | Alt+Left on O3 | settled |
| One-finger swipe left | Forward | Alt+Right | Alt+Right | settled |
| Two-finger swipe right (fingers move right) | WorkspacePrev | `sill workspace prev` IPC (cctk `WorkspaceState::activate`); without sill: the COSMIC shortcut keys from the user's shortcuts file | kglobalaccel `Switch One Desktop to the Left` (Python, H) | settled |
| Two-finger swipe left | WorkspaceNext | `sill workspace next` | `Switch One Desktop to the Right` | settled |
| Two-finger double-tap | WorkspaceOverview | COSMIC `WorkspaceOverview` system action (sill IPC or its shortcut) | kglobalaccel `Overview` | settled mapping, backends L |
| One-finger double-tap | SmartZoom | our surfaces: focused ds root; others: none | same | settled ("where supported") |

## 12.7 Integration

| Concern | Owner |
| --- | --- |
| Decoder, tracker, feel, gestures, outputs, service | `magic-gestures` repo, `palmrest` binary |
| Host client, double-scroll rule, swipe decision on our surfaces | `shell-host/src/input/palmrest.rs` + `scroll/**` (11) |
| Workspace / overview actions on COSMIC | `sill-ipc` gains `Workspace{Prev, Next, Overview}` (proposed addition); `sill-services` `cosmic_wl` performs them |
| Settings page | Settings app, writes `config.toml` and the compositor's natural-scroll value for "palmrest touchpad" |

Platform limits:

- No live workspace tracking through a client protocol on cosmic-comp; live mode depends on
  the compositor's own touchpad gesture (finger count to verify).
- Wayland cannot identify the device behind an axis event; hence the suppression rules.
- libinput: tap-to-click, disable-while-typing and palm detection may act on the virtual
  touchpad; the contact rules in 12.3.9 are designed against tap; DWT should not apply to a
  `BUS_VIRTUAL` external device (L, verify).
- Unverified (B): whether a virtual touchpad affects pointer motion (never one contact, so it
  should not), MM2 report rate, USB-C product id (udev rule covers Bluetooth only).
- Only the kernel sees clicks; right-side secondary click and the middle zone are kernel
  behaviour (`emulate_3button`).

## 12.8 Acceptance tests

1. **Decoder goldens**: recorded reports with N = 0, 1, 2, 3 touches, an 8-byte header-only
   report, a 0xF7 double, lengths 13, 15 and 29 (rejected with `DecodeError::Length`): exact field values; x/y agree with
   the kernel form and, after wrap handling, with the Python parser's deltas.
2. **Python parity (scroll)**: replay 5 recorded sessions (slow, fast, repeated flicks,
   diagonal, finger-count changes) through palmrest `feel` with `rejection = Off` and through
   the Python `ScrollAxisLock`: identical lock decisions per report; cumulative v120 on O4
   identical; cumulative px on O1 = v120 x 0.5 within 1 v120 unit.
3. **Python tests ported**: every case in `test_kde_gestures.py` (one finger vertical only,
   horizontal only, two fingers never horizontal, swipe right -> desktop left, long drag
   switches again after cooldown, vertical drag no switch, one finger never switches, two
   fingers scroll vertically, three fingers nothing, finger count change no jump, steps finer
   than a tenth detent) passes against palmrest.
4. **Swipe trigger**: synthetic 2-finger stroke of 200 units in 0.3 s: exactly one
   WorkspacePrev/Next; 199 units: none; 160 units vertical before 160 horizontal: none (re-
   anchored) then a further 200 horizontal fires once; 600 units in 0.4 s: one fire (cooldown).
5. **Double-taps**: two 2-finger taps of 120 ms, 250 ms apart: one Overview; gap 400 ms: none;
   a tap moving 45 units: none; a tap with the click bit set: none; one-finger variant fires
   SmartZoom only when a host claims the pointer.
6. **One-finger swipe**: 350 units in 250 ms with lift: Alt+Left once and horizontal scroll
   still emitted; 350 units in 600 ms: scroll only.
7. **Rejection**: a stationary contact in the side band plus a moving finger scrolls as one
   finger (with `rejection = On`), as two with `Off`.
8. **Virtual touchpad** (`libinput debug-events`, needs the input group): a scroll stroke
   yields `POINTER_SCROLL_FINGER` events and one terminating zero (axis stop), no
   `POINTER_MOTION`, no `POINTER_BUTTON`; 50 random short strokes produce zero taps.
9. **Calibration**: the same recorded stroke scrolls a GTK4 test list via O2 and a ds list via
   O1 by the same px within 10 % (sets the 0.25 mm/px factor).
10. **Suppression**: with a host claiming the pointer, O2/O4 emit nothing and G4 emits no keys.
11. **Latency**: HID read to socket write p95 <= 2 ms, to uinput write p95 <= 2 ms over 10 000
    reports.
12. **Timestamp calibration**: on the device, fit residual p95 < 1 ms is reported with the
    derived tick period; if not, arrival time is used (logged).

## 12.9 Open decisions

1. G5 repeat: Python repeats every 200 units after the cooldown; macOS moves one Space per
   gesture. Keep `AfterCooldown` (current feel) or switch to `OncePerGesture`.
2. G4 on foreign clients emits Alt+Left/Right (settled) while also scrolling horizontally;
   alternative `Native`: send only the horizontal 2-finger scroll on O2 and let browsers do
   their own overscroll navigation (no double effect in horizontally scrollable views).
3. Treat nibble 1-2 (in range) as contact (Python parity) or only 3-4 (kernel "down").
4. Rejection zones and the palm size are guesses; tune with `palmrest record`.
5. The kernel `u/mm` constants make X 2.7x coarser than Y per mm; the Python thresholds are in
   raw units, so horizontal locks need ~2.7x more finger travel. Keep (feel) or normalise to mm.
6. `emulate_3button=1` gives a middle click that macOS does not have; keep or set 0.

## 12.10 Sources

- Plan Appendix B (kernel parameters, report format, libinput, existing Linux projects,
  ownership decision) and Appendix C-C (all R rows); plan settled decisions (gestures,
  palmrest replaces the Python daemon at M8).
- Python daemon v0.3.0-kde and its tests `test_kde_gestures.py`; `/etc/modprobe.d/
  99-magic-mouse-kde.conf`; `/etc/udev/rules.d/70-magic-mouse-kde.rules`.
- Linux `drivers/hid/hid-magicmouse.c`; Apple Support 102482 (Magic Mouse gestures); Apple
  `NSEvent.trackSwipeEvent(options:...)`; WebKit GTK `ViewGestureController`; libadwaita
  `AdwSwipeTracker`; OpenMultitouchSupport (MTTouch states); brenoperucchi/
  magic-mouse-gestures; Salacfrantisek/magic-mouse-wayland-gestures; RicardoEPRodrigues/
  magicmouse-hid (as cited in B and C-C).
