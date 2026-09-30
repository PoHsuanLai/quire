# 30 Catalogue

The settled inventory of the library: every component, motion primitive, interaction primitive,
state enum and Look value. Later agents build it literally; where 04, 05, 06, 07, 21, 26, 27 or
29 disagree, this file wins (each names the sections). HIG page names are cited, never quoted;
design/27 wins over the current HIG text; Liquid Glass is never adopted.

Status: **settled** (decided with the user, 2026-09-29) except where a cell says "conf L": a
macOS number that could not be confirmed, shipped as the default and tuned against a real Mac
(Deferred item 5).

## 0. The rules

| # | Rule |
| --- | --- |
| R1 | Target: macOS Sonoma / Sequoia, pre-Liquid-Glass. Every component's look, sizes, states and behaviour follow the macOS default; the contract names its AppKit counterpart. Every open question takes the macOS default. |
| R2 | One motion model: the macOS one. quire's own flourishes are gone: press squish, hover lift, overshoot and spring pops, tilt, stagger, first-show sweeps, mailo-era keyframes, infinite loops (the spinner is the one exception, and the voice orb's turn while it is `Active`). Motion levels are an accessibility preference only. |
| R3 | One implementation per concept. Every MERGE names one survivor; the absorbed names are deleted, not aliased. |
| R4 | There is one Look (Part 3), the Mac values; Arc's ideas are features (3.3), not values. |
| R5 | Kept although macOS lacks them: skeleton, toast, busy state on buttons and rows, hover card, key-cap. They move the macOS way (section 2.9 and 2.5). |
| R6 | Arc-style features (pinned tiles, Today tabs, edge-peek sidebar, link pill, grouped launcher commands, Ctrl+1-9 Space switching) exist in both Looks (section 2.11). |
| R7 | No `bool` props; no `busy`/`open`/`muted` invented per component: the shared enums of section 1.5. |
| R8 | Every component takes the `Common` props (`id`, `data`, `extra_class`, `aria_label`, `mounted`) and names its public parts (section 1.7). |

Status legend (Part 2): **KEEP** unchanged contract; **MERGE** the survivor of a merge (or a kept
component that changes to absorb others); **ADD** new. Dropped items are in Part 4. Priority:
P1 needed by the shell or mailo migration now; P2 needed by the next apps; mail = mail-only.

---

## 1. Foundations

### 1.1 Motion levels (accessibility preference, not a style)

| Level | Meaning | Effect |
| --- | --- | --- |
| Standard | macOS default | tables 1.2 to 1.4 as written |
| Reduced | macOS "Reduce motion" (with "Prefer cross-fade transitions" always on) | slides, scales and springs become a cross-fade at `--t-quick`; springs are critically damped with no overshoot; drags track 1:1; the spinner keeps turning (macOS does); delays (hover, long press) do not change |

`Calm` and `Extra` are deleted; `--data-motion` has two values; the system preference resolves to
Reduced (`ds-style::appearance`). A Look never sets the level.

### 1.2 Timing tokens

Durations (`DurationToken`); every duration in CSS and Rust comes from this table (lint
`RawDuration`; Rust `Duration::from_millis` outside `ds-style::tokens` is a lint too).

| Token | ms | macOS reference | Used for | Change |
| --- | --- | --- | --- | --- |
| Quick `--t-quick` | 150 | control state fades, menu close, focus ring, cross-fade (conf M) | colour, opacity, glyph swap, close of menu/popover/tooltip, hover-card out | was 170 |
| Move `--t-move` | 250 | `NSAnimationContext` default | collapse, list insert/remove, pane slide, banner in, progress value change | kept |
| Big `--t-big` | 400 | window / sheet class (conf L) | sheet and side-panel in when not spring-driven, Space colour cross-fade | was 420; absorbs Scene 380 |
| Shake `--t-shake` | 420 | login shake (conf M) | the one shake | kept; absorbs ShakeLong |
| SpinStep `--t-spin-step` | 83 | 12-spoke spinner, one turn per second (conf M) | spinner spoke step | replaces PendingStep |
| Hold tokens | | | | |
| IdleDim | 2000 | display dim before sleep | idle overlay | kept |
| Awake | 20000 | | emoji awake window | kept |
| ToastHold | 5000 | notification banner | toast and banner hold | was 5200; absorbs SentHold |

Deleted durations: Tap, Ambient, BigHeavy, CrumpleHeavy, Spark, Curl, CurlHeavy, Send, Float, Sail,
BoatReturn, Spin, Nudge, Park, HcOut (Quick), Scene (Big), Fill, Sweep, PendingStep, SendRing
(moves to `app`, mail-owned), Flash, CountStep. `GRAMMAR_DURATIONS` is the surviving set.

Delays (`DelayToken`, never scaled by any level, each with a settings override of the same name):

| Token | ms | Meaning | Change |
| --- | --- | --- | --- |
| HoverOpen / HoverClose per profile | Tip 1000 / 0; Card 500 / 150; Label 100 / 0 | the three `HoverIntent` profiles (1.4) | replaces HoverOpen/Close/Warm/Fly |
| HoverWarm | 400 | after one hover UI closes, the next opens with delay 0 within this window | kept |
| LongPress | 500, slop 4 px | hold without moving | ADD; replaces the dock 600 and the titlebar 500 |
| SubmenuOpen / TriangleTimeout | 200 / 300 | menu tracking (safe triangle) | ADD as tokens |
| MultiClick | 500 (system double-click time), slop 4 px | click run | ADD; replaces `RUN_GAP` |
| ReleaseWindow | 100 | window for release velocity | ADD; replaces `THROW_WINDOW` and `FLING_WINDOW` |
| SwipeQuiet | 120 | end of a wheel swipe | kept |
| TypeaheadReset | 1000 | type-to-select buffer | ADD |
| Infrastructure (not motion): ReadReflow 260, AutosaveDebounce 700, FocusAfterMount 60, SpellDebounce 300 | | | kept in `app`/`edit` |
| Mail-owned, move to `app`: SendCountdown 5000, SendTick 1000 | | | moved |

Deleted delays: Fly, HealStep, FlashHold, PendingGrace, PendingCap, SettleHold. Frame pacing is
one `FRAME_TICK` 16 (`ds-core::time`); `glide::FRAME` and `Motor::FRAME` are deleted.

Easings (`EasingToken`, evaluated only by `CubicBezier::at`; sill's own `ease` is deleted):

| Token | Value | Use |
| --- | --- | --- |
| `--e-in-out` | (.42,0,.58,1) | the default: macOS ease-in-out for every state change and slide |
| `--e-out` | (.22,.9,.30,1) | entrances that decelerate; Rust tweens |
| `--e-exit` | (.55,0,.75,.2) | exits accelerate |
| `--e-linear` | linear | progress value interpolation, spinner |
| `--e-shake` | (.36,.07,.19,.97) | the shake |

`--e-spring` is deleted (springs are `Spring` only, 1.3). Scalars (`ScalarToken`: overshoot, squish,
lift, tilt, stagger, pickup) are deleted with their users; there is no `ScalarToken`.

### 1.3 Motion primitives (one survivor each)

Rule: a motion a hand can touch or interrupt is a Rust spring or tween; an arrival nobody touches
is a CSS transition or one keyframe. Every primitive declares its Reduced form. Settle timers stay
crate-private (Blitz fires no `animationend`).

| Primitive | Survivor | Contract | Reduced | Absorbs (deleted) |
| --- | --- | --- | --- | --- |
| Presence | `ds-motion::presence::{Presence, use_presence}`; `Presence {Entering, Present, Leaving}` | one lifecycle for anything shown: enter with the chosen `Enter`, stay, leave with the chosen `Exit`, unmount after the exit ends, resume if re-shown mid-exit. `Enter {Instant, Fade, Slide(Side), Spring}`; `Exit {Instant, Fade, Slide(Side)}` | Fade | `ListPresence`, `ShownPhase`, `OsdPhase`, `spring_presence`, `CardPresence`, `palette_shown`, `PaneSlide`, sill `Reveal`, `use_entrance`, the popover/menu `data-presence` timers, `MenuEntrance`, `PaletteEntrance`, `FirstShow`, `Motion{Play,Still}` |
| Roster | `ds-motion::roster::{Roster, use_roster}`; `LeaveBy {Action, Delist}` | list rows: insert fades and slides down `--t-move`; removal fades and slides up `--t-quick` accelerating, rows below close the gap over `--t-move` (macOS `NSTableView` `effectFade` + `slideUp`) | fade only, gap closes at once | `batch_roster`, `roster_exits`, `use_leaving`, `LeavingList`, `AnimatedList`, `Exit::{Fold,Curl,Crumple,TabOut,BannerOut}`, `heal`, the `max-height` tab hack |
| Spring | `ds-motion::spring::Spring {damping, response}`, `SpringSpec::for_touch`, `use_spring`, `use_spring_point`, `Throw` | integrator, retargets mid-flight keeping position and velocity; damping 1.0 (tap, key), 0.8 when released with momentum toward the target; response Quick 300 ms, Move 450 ms (conf L); throw lands on the endpoint nearest `p + v*r/(1-r)`, r = .998/ms | critically damped, no throw | `--e-spring`, `use_spring_motion`, `use_spring_point_motion`, `SpringResponse` duplicates, overshoot keyframes |
| Cross-fade | `use_cross_fade` (over `MorphGlyph` layers) | a glyph, colour or content swap fades `--t-quick`; never pulses | same | `LayerGlyph`, `MorphGlyph` keyframes, `morph-*`, `fade-in` duplicates |
| Collapse | `use_collapse(Shown)` | height (and content opacity) animate `--t-move` `--e-in-out` from the measured height; the disclosure triangle turns `--t-quick` | instant, triangle cross-fades | `tab-in/out max-height`, per-component chevron rotations |
| Progress sweep | none (deleted) | a determinate value that changes tweens linearly over `--t-move` to the new value, retargeting; nothing sweeps in on first show | value jumps | `use_sweep`, `use_level_run`, `Fill`, `Sweep`, `RunTail`, `use_tween` callers that only counted |
| Number change | none (deleted) | text changes instantly, as in every AppKit label | same | `RollDigits`, `use_count_up`, `CountUp`, `Bumped`, `use_bump_on`, `--t-count` |
| Emphasis | none (deleted) | no bump, gulp, seal, chip flash, pulse, nudge, accept beat or level tick. The two macOS attention moves are kept where they belong: the menu-item blink on activation (`Menu`, 2 flashes of 70 ms) and the Dock icon bounce (sill) | | `use_pulse`, `PulseKey`, `use_nudge`, `use_once` (public), `Anim::{Bump,Gulp,SealPop,ChipFlash,PictureAccept,LevelTick,NudgeUp}`, the `X--b` alias trick |
| Shake | `use_shake(cue)` | one damped side-to-side `--t-shake` `--e-shake` on a failed secure entry or failed authentication (login window) | still | `shake` (C), Wi-Fi/row shakes on non-secure failures |
| Drag settle | `use_drag_return` (2-D spring home) | a drop that is refused springs back to the source with the release velocity; an accepted drop just lands | jumps | `DragReturnFrame`, `--tilt`, `--pickup` |
| Rubber band | `ds-motion::rubber::resist` (extracted from `Level`) | past a limit, displacement = .55 of the overshoot (conf M), spring back on release; used by swipe resistance and host scroll; not by sliders | 1:1, hard stop | `Rubber{On,Off}`, the level control's `--rb` |
| Magnification | stays in sill `dock` (Plank parabola), evaluated with `CubicBezier::at` | 120 ms in / 200 ms out (design/10 R5-R6) | off | sill's own evaluator |
| Pending loop | `use_pending(Operation)` | a spinner exists only while an `Operation` runs; it spins at once (no grace, no cap: the macOS default) in 12 steps of `--t-spin-step` | keeps turning | `PendingGrace`, `PendingCap`, `Spinner::Breathe`, `breathe`, `spin`, `dest` loops, `SyncHalo` |
| Detail grammar | `Detailed`, `Moment`, `Cue`, `Touch` | state to moment to primitive; the `Moment` set is pruned to moments a survivor plays (Appear, Dismiss, Failure, Pending) | inside | moments whose primitive is deleted |
| Sequence/timeline | none | no general timeline | | |

Motion of the macOS surfaces, all built from the rows above:

| Surface | Enter | Exit |
| --- | --- | --- |
| Menu, first bar menu and after | Instant | Fade `--t-quick`; hover-switch between bar menus Instant both ways |
| Popover | Fade `--t-quick` | Fade `--t-quick` |
| Tooltip, hover card | Fade `--t-quick` | Fade `--t-quick` |
| Sheet, alert | Slide(Top) with `Spring` response Move | Slide(Top) `--t-move` `--e-exit` |
| Side panel, notification banner, toast | Slide(Right) `--t-move` `--e-out` | Slide(Right) `--t-quick` `--e-exit`; swipe right dismisses with the release velocity |
| OSD | Fade `--t-quick` | Fade `--t-move` after its hold |
| Pane switch | Spring on `--pane-p` | reverses mid-slide |
| Space switch | colour cross-fade `--t-big` | |

### 1.4 Interaction primitives

| Primitive | Survivor | Contract |
| --- | --- | --- |
| Press | `Press {button, modifiers, at}`; state `PressPhase` (1.5); CSS `[data-pressed]` written for pointer down or key down | pressed = the pressed appearance of the control (darker fill, no scale); a key activation (Space, Return) shows it for one frame at least; release inside activates |
| Multi-click | `ClickRun` (`MultiClick`) | 1..3, shared by the text surface and the titlebar zoom |
| Long-press | `LongPress {delay: LongPress, slop}` pure machine | fires `PressPhase::Held`; dock menu, green-button menu, toolbar pull-down |
| Hover intent | `HoverIntent` with profile `Tip`, `Card`, `Label` (1.2); sill dock label reuses it | open after the profile delay, close after its close delay, warm window skips the delay |
| Hover style | CSS `:hover` only where macOS shows a hover (toolbar/icon bezel, sidebar disclosure, segmented, link cursor, table header); no `Hover` enum except in machines | list rows, push buttons, menu rows-by-pointer: menu rows highlight by tracking, not hover |
| Focus ring | `FocusStyle {Ring, Highlight}`; `data-modality`; `--focus-ring` | Ring: translucent accent (.55 alpha) 3 px, gap 1, radius = element radius + gap, fades in `--t-quick` (design/27 6.4); Highlight (list rows, menu and launcher rows): accent fill with accent ink when the list is focused in the active window, grey `--sel-bg-quiet` otherwise. Keyboard modality only for Ring. `appearance.keyboard_navigation` `All` default, `TextAndLists` as the macOS setting (settled 27 §8.3) |
| Roving focus | `Roving<K>` machine | one item focused; arrows move; Home/End; wraps only in menus; used by menu, segmented, radio group, list, tabs, emoji grid (`GridMove`), palette |
| Typeahead | `Typeahead` (buffer, `TypeaheadReset`) | letters jump to the next item whose label starts with the buffer; menus, lists, pop-up buttons |
| Layer stack | `LayerStack`; `Dismiss` (1.5) | Escape closes the top layer; outside click per `Dismiss` |
| Drag and drop | `use_drag`, `DragGhost`, `DropState`, `file_drop` | threshold 3 px Euclidean (content), 4 px (window drag); Option copies; the cursor shows copy / not-allowed / disappearing; multi-item drag shows a count badge; failed drop returns (1.3); auto-scroll near an edge |
| Swipe | `use_swipe` with `SwipeMetrics` | dismiss at 80 px or 600 px/s, quarter resistance the other way (`rubber::resist`), wheel quiet `SwipeQuiet` |
| Scroll physics | host (design/11) | momentum, rubber band, axis lock stay in the host scroll daemon; ds only reads `rubber::resist` |
| Cursor | CSS | arrow on controls; pointing hand only on links and the link pill; I-beam on editable text; grab/grabbing on drag handles; resize on dividers and window edges (27 §6.3; lint `PointerCursor`) |
| Keyboard | `StandardAction`, `Shortcut` | bindings written in Mac terms, modifiers rendered in the order Control, Option, Shift, Command; reserved combinations are refused (27 §6.2) |

### 1.5 The shared state vocabulary (one enum per concept)

| Concept | Survivor (`ds-core::vocab`) | Merged in (deleted) |
| --- | --- | --- |
| Open / closed / shown | `Shown {Visible, Hidden}` (maps to `aria-expanded`, `hidden`) | `Expanded`, `Disclosure`, tooltip `Shown`, `data-shown`, `MenuBarItem.open: Switch`, `Button.expanded`, `IconButton.expanded` |
| On / off / mixed | `Check {Off, On, Mixed}` (= `NSControl.StateValue`; `Toggle` refuses Mixed) | `Switch`, `Check {Checked, Unchecked}`, `Button.pressed: Switch` (a toggle button's value) |
| Availability | `Availability {Enabled, Disabled, Busy}`; Busy = `aria-busy`, not interactive, the busy accessory shows (1.6) | per-component busy flags, `RowPhase`, `RowWork`, `SendPhase`, `StatusState`, `ModuleState::Busy`, `Tick` |
| Selection | `Selection {Selected, Unselected}` | `Here {Here, Elsewhere}`, `data-selected`, `Retain` |
| Row state | `RowState {selection, emphasis, availability, drop}` | the six row vocabularies (`Selection`, `Emphasis`, `Presence`, `DropState`, `Here`, `Switch`) as row props |
| Emphasis | `Emphasis {Strong, Plain}` | `AlertEmphasis` stays a separate `AlertStyle` (contract, not emphasis); `ScrimStrength` deleted with Scrim |
| Muted | `Muting {Audible, Muted}` | `AvatarMuting` |
| Press phase | `PressPhase {Idle, Pressed, Held}` (Pressed also from a key; Held = long press elapsed) | three `Held` types, `Hold {Idle, Pressing, Held}` |
| Dismiss policy | `Dismiss {Transient, Semitransient, Manual}` (= `NSPopover.Behavior`; Transient = Esc and outside, Semitransient = Esc only, Manual = neither) | `Dismiss {EscAndOutside, EscOnly, None}`, `PickDismiss`, `Swipe::Dismiss` as a policy; `Dismissal {Close, PassThrough}` stays as the layer-stack outcome |
| Presence | `Presence {Entering, Present, Leaving}` | `ListPresence`, `CardPresence`, `TimerPhase`, `PulsePhase`, `Stage`, `Reveal` |
| Async work | `Operation` (+ `Availability::Busy`) | `SyncState`, `SpinnerKind`, `RowPhase`, `SendPhase` |
| Window activity | `Activity {Active, Inactive}` (ADD: inactive window dims controls and greys selection) | none (gap) |
| Focus | `FocusStyle {Ring, Highlight}`, `InputModality {Pointer, Keyboard}` | none (gap) |
| Size | `ControlSize {Mini, Small, Regular, Large}` (1.6); `SidebarSize {Small, Medium, Large}` | `ButtonSize`, `ButtonVariant::Mini`, `SegSize`, `KbdSize`, `ControlSize {S,M,L}` old, `MarkSize`, `PickerLayout` |
| Placement | `Side`, `Align`, `PanelEdge`, `GridEdge`, `Grow` | `SheetPlacement` becomes `Attach {Window, Centre, Bottom}` (Bottom is a documented addition, below) |

### 1.6 The size ladder

`ControlSize` = `NSControl.ControlSize`. One `SizeScale` (design/29) computes everything; sizes are
not props of a Look. Heights in pt at 13 pt body.

| Rung | Height | Label pt | Radius (push, field) | Switch (w x h) | Slider knob / track | Checkbox, radio | Segmented well / segment | Spinner | Progress bar |
| --- | --- | --- | --- | --- | --- | --- | --- | --- | --- |
| Mini | 16 | 9 | 4 | 26 x 15 | 12 / 3 | 10 | 16 / 14 | 10 | 4 |
| Small | 19 | 11 | 5 | 32 x 18 | 14 / 4 | 12 | 19 / 17 | 16 | 6 |
| Regular | 22 | 13 | 5 | 38 x 22 | 20 / 4 | 14 | 22 / 20 | 32 | 6 |
| Large | 28 | 15 | 5 | 38 x 22 | 20 / 4 | 14 | 28 / 26 | 32 | 6 |

The old `Small` (16) is Mini; the macOS small rung 19 is added (conf M; the rest of the row is
templates, conf H, except switch small/regular, spinner, mini progress: conf L). Controls without
a Large rung repeat Regular. Text sizes are the type scale, not the ladder. Icon glyph sizes
`IconSize` 12, 14, 16, 20, 24 (follow the rung), plate sizes 32, 48, 64, 128; no `Px`.
Avatar sizes 16, 24, 32, 48, 64. Sidebar row: Small 24, Medium 28, Large 32, chosen by
`appearance.sidebar_size` (default Medium); table and list rows 24 (compact) / 44 (settings).

### 1.7 Anatomy and public parts

| Rule | Value |
| --- | --- |
| Root class | `.ds-<component>`; parts `.ds-<component>-<part>`; listed per component in Part 2 and in `ds::selectors` |
| Part names | `icon`, `label`, `detail`, `trailing`, `badge`, `indicator`, `track`, `fill`, `thumb`, `title`, `body`, `header`, `footer` |
| Axes | `data-variant` (role), `data-size` (`ControlSize` slug), `data-state` (`Check`), `data-selected`, `data-busy`, `data-availability`, `data-focus`, `data-activity`, `aria-*`; `data-kind` is never used for a variant |
| Row anatomy | leading, title, detail, trailing accessory, state: one anatomy for menu, list, palette, sidebar and settings rows |
| Callbacks | `onclick: EventHandler<Press>`, `onchange`, `onpick`, `on_hover` unified to `on<event>` |
| Labels | `label: Text`; `aria_label` only when the visible label is absent; `tooltip` is a `Tooltip` child, not a prop |
| Option lists | one shape: `Vec<Choice<T>> {value, label, icon, availability}` for segmented, radio, pop-up |

### 1.8 Settings keys this catalogue needs

Each is a `22-SETTINGS.md` row before it is read (ARCHITECTURE section 9).

| Key | Values | Default |
| --- | --- | --- |
| `appearance.motion` | Standard, Reduced (system preference resolves to Reduced) | Standard |
| `appearance.keyboard_navigation` | All, TextAndLists | All |
| `appearance.sidebar_size` | Small, Medium, Large | Medium |
| `hover.tip_open_ms`, `hover.card_open_ms`, `hover.label_open_ms` | ms | 1000, 500, 100 |
| `input.long_press_ms`, `input.multi_click_ms` | ms | 500, system value |
| `dock.magnify_enter_ms`, `dock.magnify_leave_ms`, `dock.hold_to_menu_ms` | ms | 120, 200, `input.long_press_ms` |
| `notifications.toast_hold_ms` | ms | 5000 |

Settings override a `DelayToken` default; the token table stays the one home of the default.

---

## 2. Components

Codes. **Z4** = Mini/Small/Regular/Large; **Z3** = Mini/Small/Regular; **Z-** = fixed macOS metric.
States: **CTL** = normal, pressed, focused (Ring), disabled, busy, inactive-window; **+H** adds a
hover appearance. Every entry inherits `Common` (R8).

### 2.1 Controls

| Name | AppKit | Contract | Sizes | States | Public parts | Absorbs | St | Pri |
| --- | --- | --- | --- | --- | --- | --- | --- | --- |
| Button | `NSButton` push, toolbar/inline/help bezels | bezel `Push` (default button takes the accent and answers Return; Escape answers Cancel), `Toolbar` (glyph, hover bezel), `Inline`, `Help`; `role Normal/Destructive`; a toggle button holds `Check`; Busy shows a spinner in the leading slot, crossfaded, and blocks input | Z4 | CTL +H(Toolbar) | icon, label, badge | `IconButton`, `PlayPauseButton`, `ButtonVariant`, `ButtonSize`, `Trailing::Caret`, `nudge`, `first` | MERGE | P1 |
| PopUpButton | `NSPopUpButton` | `Kind {PopUp, PullDown}`: pop-up shows the chosen item with a check in its menu, pull-down a fixed title; owns the chevrons; opens `Menu{Popup}`; type-to-select | Z4 | CTL, open (`Shown`) | label, indicator | hand-built Button + `Menu{Dropdown}` | ADD | P1 |
| Slider | `NSSlider` | linear; `Ticks {None, Every(n)}` with snap; knob on release springs to the tick; control-center bezel with leading glyph and 22 pt capsule | Z4 | CTL, dragging | track, fill, thumb, icon | `LevelControl` (interactive), `LevelLook`, `Rubber`, press swell | MERGE | P1 |
| Stepper | `NSStepper` | up/down pair bound to a `TextField` value; hold repeats | Z3 | CTL | up, down | none | ADD | P2 |
| Disclosure | `NSButton` disclosure triangle | `Shown`; triangle turns `--t-quick`; owned by outline rows and grouped panes; drives `use_collapse` | Z3 | CTL | indicator | `Disclosure` enum, `TreeItem` chevron, `ModuleTile` chevron | ADD | P1 |
| KeyEquivalent | menu key-equivalent text and the key-cap | `Style {Text, Cap}`: Text is the menu trailing text; Cap draws each key as a key-cap (the kept Kbd); symbols in the order Control, Option, Shift, Command then the key; `+ label` composition is `KeyEquivalent` beside a `Label`, not a component | Z3 | none; no press animation | key | `Chord`, `Kbd`, `KbdSize`, `KeyHint`, sill `hint()` | MERGE | P1 |
| Chip | `NSTokenField` token / mail label | `Kind {Neutral, Accent, Status, Person}`; `Removal {Removable, Fixed}` (the token pill); fade in/out only | Z3 | CTL, selected | label, icon, remove | `ChipVariant::Token`, pulse, `ChipVariant` duplicates | MERGE | mail |

### 2.2 Fields

| Name | AppKit | Contract | Sizes | States | Public parts | Absorbs | St | Pri |
| --- | --- | --- | --- | --- | --- | --- | --- | --- |
| TextField | `NSTextField`, `NSSecureTextField`, `NSSearchField` | `Kind {Plain, Secure, Search}`; `Bezel {Bezeled, Plain}`; Search has magnifier, clear button, cancel, tokens, suggestions list; prefix/suffix; help text is a `Label` below; invalid shows a `--danger` secondary `Label` below and a shake only for Secure; edit menu (Undo, Cut, Copy, Paste, Select All) on every field; expansion tooltip on truncation | Z4 | CTL, focused ring, editing | label, field, icon, clear, help | `TextInput`, `SearchField`, `SecretEntry`, `InputVariant` Boxed/Inline/Bare, `TextInputKind` | MERGE | P1 |
| TextView | `NSTextView` | rich multi-line, IME, spell, undo, find; the surface of mail compose | Z- | CTL | body | `EditSurface`, `SpellMarks` | KEEP | mail |
| TokenField | `NSTokenField` | recipients: text plus `Chip{Removable}`, menu on token | Z3 | CTL | token | free-form chips in `SearchField` | ADD | mail |
| FieldRow | `NSGridView` form row | label column, control column, help; System Settings form rows 36-37 | Z- | Availability | label, control, detail | ad-hoc form layouts | ADD | P2 |

### 2.3 Selection

| Name | AppKit | Contract | Sizes | States | Public parts | Absorbs | St | Pri |
| --- | --- | --- | --- | --- | --- | --- | --- | --- |
| Toggle | `NSSwitch` | `Check` On/Off; knob moves by spring, track tint fades `--t-quick`; label in the row, not the switch | Z3 | CTL | track, indicator | `Switch`, `ControlSize` S/M/L | KEEP | P1 |
| Checkbox | `NSButton` checkbox | `Check {Off, On, Mixed}`; label; Space toggles | Z3 | CTL | indicator, label | menu `Check` drawing | ADD | P1 |
| RadioGroup<T> | `NSButton` radio group | one of N; arrows rove and select; image labels for the Appearance choice | Z3 | CTL per item | item, indicator, label | the appearance picker, `PickerLayout` | ADD | P1 |
| SegmentedControl<T> | `NSSegmentedControl` | `Tracking {SelectOne, SelectAny, Momentary}`; text or image segments; per-segment availability; the indicator slides by spring (crate-private `SelectionTrack`, also under `WorkspacePills`) | Z4 | CTL +H, selected | segment, indicator, label, icon | `Tabs`, `SegSize`, the `TabView` strip | MERGE | P1 |
| DatePicker | `NSDatePicker` | `Style {Textual, Graphical}`; graphical is `MonthGrid` plus a time field | Z3 | CTL | field, calendar | none | ADD | P2 |

### 2.4 Menus

| Name | AppKit | Contract | Sizes | States | Public parts | Absorbs | St | Pri |
| --- | --- | --- | --- | --- | --- | --- | --- | --- |
| Menu<T> | `NSMenu` | `Placement {Bar, Popup, Context}` (bar menu, pop-up / pull-down / Dock / status-item menu, context menu); one density (row 22, separator 9, inset 5, radius 8); safe-triangle tracking; type-to-select; picking blinks the row twice then closes; context menus hide unavailable items and show no shortcuts (27) | Z- | open, tracking | item, separator, header | `MenuKind {Rich, Slim, Dropdown, Context}`, `PickDismiss`, `Filter` on non-palette menus, `MenuEntrance`, file/clip row shapes (to `List`) | MERGE | P1 |
| MenuItem | `NSMenuItem` | title, image, `KeyEquivalent`, `Check` state, submenu arrow, separator, section header, alternate (Option), availability; highlight is a `Highlight` fill that snaps (no fade) | Z- | highlighted, disabled | icon, label, trailing | `menu_rows`, `row_action`, `row_chord`, `MenuEntry::{Item,Submenu,Header,Info}` | MERGE | P1 |
| MenuBar | `NSMenu` mainMenu | App, File, Edit, View, Window, Help with the standard items and Help search; Cmd-? opens Help | Z- | open | title | none (model only) | ADD | P1 |
| CommandPalette<T> | Spotlight panel | search `TextField{Search}` + results `List` + preview; grouped commands (2.11) | Z- | selected, empty (`EmptyState`) | field, list, preview | palette row components (rows are `Row`), `PaletteEntrance`, `Corner` | KEEP | P1 |

### 2.5 Overlays

| Name | AppKit | Contract | Sizes | States | Public parts | Absorbs | St | Pri |
| --- | --- | --- | --- | --- | --- | --- | --- | --- |
| Popover | `NSPopover` | `Dismiss` policy; `Arrow {Arrow, None}` by anchor: an app popover anchored to a control has the 34 x 8 arrow, shell popovers hung from the bar or Control Center have none (27 5.6); one at a time | Z- | `Shown` | body | `Elevation {Pop, Bubble, Sheet}`, `Dismiss` old | MERGE | P1 |
| Sheet | `NSWindow` sheet | `Attach {Window, Centre, Bottom}`: window-attached slides from the titlebar, centre is the shell's app-modal dialog, bottom stands 8 above the bottom edge under half the root and rises from below (added: Edit Widgets' gallery must leave the desktop's top rows, where a new widget lands, in view); width Regular / Narrow / Wide (1040, added for the same gallery); dims nothing on macOS (no scrim) | Z- | `Shown` | body | `Scrim`, `ScrimStrength`, `SheetPlacement`, `PanelScrim`, sill click catchers L10 (shell-host) | MERGE | P1 |
| Alert | `NSAlert` | `Style {Informational, Warning, Critical}`; icon, message, informative text, 1 to 3+ buttons (first is default, never destructive by default), optional suppression `Checkbox`, help button; floating on `Sheet` or inline in a popover | Z- | `Shown` | icon, title, body, footer | `AlertEmphasis`, `Flow` duplicates, the power-menu body (L4) | KEEP | P1 |
| SidePanel | none (Notification Center side sheet) | edge Right; slides `--t-move`; shell only | Z- | `Shown` | header, body | `Panel`, `PanelEdge::Bottom` | MERGE | P1 |
| Tooltip | `NSToolTip` | one line, `HoverIntent` Tip; anchors to an element id or a rect; truncation expansion | Z- | `Shown` | body | `TooltipKind`, `sub` line, `HoverKind::Tip` | MERGE | P1 |
| HoverCard | Safari link preview / data-detector popup | kept: a card with rich content after `HoverIntent` Card, Fade in and out `--t-quick`, no spring; kinds are content, not variants | Z- | `Shown` | body | `HoverKind` variants (content), `hc-in` spring | KEEP | P2 |
| DragImage | `NSDraggingItem` | `DragGhost` follows 1:1, count badge for many; `DropLine` is a `List` part | Z- | dragging | ghost | `Grip`, `DragReturnFrame`, tilt | KEEP | P2 |

### 2.6 Lists and tables

| Name | AppKit | Contract | Sizes | States | Public parts | Absorbs | St | Pri |
| --- | --- | --- | --- | --- | --- | --- | --- | --- |
| List | `NSTableView` / `NSOutlineView` (view-based) | `Style {Plain, Inset, SourceList}`; single/multi selection; `Highlight` emphasized when focused in the active window, grey otherwise; Roving + Typeahead; outline through `Disclosure`; insert/remove by `Roster`; drop indicator line | rows 24 / 44; SourceList `SidebarSize` | selection, focus, drop | row, header, separator, drop-line | `AnimatedList`, `LeavingList`, `RowList`, `SettingsGroup`, tree containers | MERGE | P1 |
| Row | `NSTableCellView` | leading image, title, detail, trailing accessory `Accessory {None, Check, Toggle, Chevron, Text, Glyph, Battery, Spinner, Badge}`; `RowState`; Busy shows the small spinner accessory then a check on completion; mail row, sidebar row, tree row, settings row, palette result and module tile are `Row` with content, not types | Z- | `RowState` | leading, title, detail, trailing | `SettingsRow`, `SidebarItem`, `TreeItem`, `ListRow` layout, palette rows, `RowTrailing`, `RowDisc`, `RowBattery` | MERGE | P1 |
| SectionHeader | table group row / source-list header | one look; collapsible in a SourceList | Z- | collapsed | title, trailing | `HeaderKind` Frame/Group/Field/Menu | KEEP | P1 |
| Table | `NSTableView` multi-column | columns, sort indicator, resize, alternating rows off | rows 24 | selection, sorted | header, column, cell | none | ADD | P2 |

### 2.7 Navigation and chrome

| Name | AppKit | Contract | Sizes | States | Public parts | Absorbs | St | Pri |
| --- | --- | --- | --- | --- | --- | --- | --- | --- |
| WindowTitlebar | `NSWindow` titlebar | traffic lights (Option = zoom, hold = tiling menu), title, subtitle, proxy icon, edited dot; drag and double-click zoom | 28 | `Activity` | title, controls | none | KEEP | P2 |
| Toolbar | `NSToolbar` | leading, title, trailing, overflow chevron; `Button{Toolbar}` items, 52 tall region | Z- | overflow | leading, title, trailing | ad-hoc toolbars | ADD | P2 |
| SplitView | `NSSplitView` | hairline divider, 6 px drag zone, min widths, collapse by spring, double-click resets | Z- | dragging, collapsed | divider, pane | fixed layouts | ADD | P2 |
| Sidebar | `NSSplitViewItem` sidebar | `SplitView` pane with a `List{SourceList}`; material Sidebar | `SidebarSize` | collapsed | body, header | none | ADD | P2 |
| TabView | `NSTabView` | container, at most six tabs, strip is `SegmentedControl` | Z3 | selected | strip, body | `Tabs` container role | ADD | P2 |
| PaneSwitcher | Control Center module detail | Root / Detail slide on a spring; `PaneHeader {back, title, trailing}`, `PaneFooter` | Z- | mid-slide | header, body, footer | sill `sill-cc-detail-*` (L5) | MERGE | P1 |

### 2.8 Content and media

| Name | AppKit | Contract | Sizes | States | Public parts | Absorbs | St | Pri |
| --- | --- | --- | --- | --- | --- | --- | --- | --- |
| Label | `NSTextField` label | `Role {Primary, Secondary, Tertiary, Quaternary}` plus the type-scale style; the only way to draw plain text | type scale | Availability | text | raw spans, `StatusLine`, `.sill-*-title`, `HeaderKind::Field` | ADD | P1 |
| IconView | `NSImageView` | glyph or app-icon plate; `IconSize` ladder; plate family by Look (abstract, embossed) | 12..128 | none | icon | `Px` sizes, plate families beyond the app plate | KEEP | P1 |
| Avatar | contact photo / monogram | round photo or monogram; sizes 16..64; no moods | 5 | none | image | `AvatarShape` Square, `UserPortrait`, `AvatarMuting`, portrait moods | MERGE | P1 |
| StatusItem glyphs | `NSStatusItem` images | Wi-Fi (bars animate while joining), Bluetooth, volume, battery (`BatteryGlyph`, outline) | icon | pending (Wi-Fi only) | glyph | `LayerGlyph`, Bluetooth/other pending loops | KEEP | P1 |
| PreviewPane | Spotlight preview | app, emoji, facts, image, PDF, text, web; shell launcher only | Z- | busy | body | none | KEEP | P1 |
| PdfThumb | Quick Look thumbnail | PDF first page, spinner while rendering | 3 | busy | image | `PDF_THUMB_GRACE` | KEEP | P1 |
| EmojiGrid<T> | Character Viewer | grid with `GridMove` roving | Z- | selected | cell | none | KEEP | P1 |

### 2.9 Feedback and loading

| Name | AppKit | Contract | Sizes | States | Public parts | Absorbs | St | Pri |
| --- | --- | --- | --- | --- | --- | --- | --- | --- |
| ProgressIndicator | `NSProgressIndicator` | `Style {Bar, Spinner, Ring}`; `value: Option<Fraction>`: Some = determinate (bar fill, pie/ring arc, updates tween by 1.3), None = indeterminate (barber-pole bar, spoke spinner) and requires an `Operation`; one arc geometry; `Ring` carries an optional centre glyph | Z3 | none | track, fill, indicator | `Spinner`, battery ring, `SendPill` ring, dock progress ring (the Dock shows a bar: `dock.progress_style = Ring` is removed), `TrackPosition` bar, `SyncHalo`, `Breathe` | MERGE | P1 |
| LevelIndicator | `NSLevelIndicator` | `Style {Continuous, Discrete}` with warning and critical bands (volume/brightness OSD bar, battery/storage rows) | Z3 | none | track, fill | read-only `LevelControl`, `LevelLook::Segments` | MERGE | P1 |
| EmptyState | `ContentUnavailableView` | glyph, title, description, optional action; forms Empty, No Results, Failure (with Retry); one shake per new failure stamp only for Secure entry (1.3), otherwise static | Z- | none | icon, title, body, action | `.sill-cc-empty`, `.sill-nc-empty`, palette `empty`, `ErrorState`, `RowPhase::Failed` | ADD | P1 |
| Skeleton | SwiftUI `redacted` placeholder | kept: grey placeholder shapes (`Shape {Line, Block, Circle}`) with a static fill, no shimmer; cross-fades `--t-quick` to content; shown while an `Operation` without a value runs | Z- | none | shape | none | ADD | P1 |
| VoiceOrb | none public (the system voice orb is private) | a round field of drifting colour that shows a voice assistant is listening: six conic glows behind a dot grid, sized by `size: Px` (the whole look follows from it: `OrbMetrics::of`, a pure table-tested function), coloured by `OrbColours` (the `--orb-bg`, `--orb-c1..3` tokens unless the caller brings its own), `period: Duration` (default 20 s), and `activity: Activity {Active, Inactive}` (1.5). The glows turn once per period only while `Active`: a Rust frame timer (`FRAME_TICK`) writes `--orb-turn` inline, exists only while `Active`, and stops where it stands on `Inactive` (an idle orb wakes nothing). It is the second infinite loop after the spinner and no other. Blitz cannot animate a registered custom property, so the angle is not a CSS `@property`; layer-by-layer transform keyframes are not used because each gradient turns about its own centre, not the box's, and the lint bans an infinite CSS animation. The dot layer is a reduced-opacity plain layer under the mask where a browser overlay-blends and backdrop-blurs it (Blitz has neither); `filter: blur() contrast()` paints on both renderers. Decorative unless `aria_label` names it. Becomes a `Timeline` implementor when `use_timeline` lands (step 4) | any `Px` | `Activity` | glow, dots | none | ADD | P2 |
| Toast | notification banner | kept: `ToastHost`, one at a time, `ToastHold`; Slide(Right) in, Slide(Right) out; swipe right dismisses; hover pauses the hold; an optional action button (Undo); the pull tab is deleted | Z- | `Shown` | body, action | `ToastHost` pull tab, `pill-up`, `SentHold` | MERGE | P1 |
| Badge | Dock / app badge | `Tone {Alert, Quiet}`; red capsule with `999+` rule (Alert), neutral capsule (Quiet, list trailing counts); dot form when no number; no bump | Z3 | none | label | `Count`, `CountPlace`, sill `.sill-dock-badge` (L1) | MERGE | P1 |

Busy is not a component. `Availability::Busy` on a Button, Row or Tile blocks input, sets
`aria-busy`, and shows `ProgressIndicator{Spinner, Small}` in the leading slot (Button) or the
trailing accessory (Row, Tile), cross-faded `--t-quick`. When the operation ends the control
returns to Enabled with no success flourish; a Row shows the `Check` accessory until its next
change.

### 2.10 Shell-only

Each entry keeps its contract from 20-SURFACES; the status is KEEP unless stated. Motion follows
section 1.3; sizes follow 1.6; rows and controls are the survivors above.

| Name | AppKit or macOS surface | Contract | States | Public parts | Absorbs | St | Pri |
| --- | --- | --- | --- | --- | --- | --- | --- |
| MenuBarItem | `NSStatusItem` button / menu title | title or glyph; open highlight; `Shown` | CTL | title, icon | `open: Switch` | KEEP | P1 |
| WorkspacePills | Mission Control Spaces bar | one pill per COSMIC workspace; `Selection` | selected | pill | `Here` | KEEP | P1 |
| ModuleGrid, ModuleTile, ModulePanel, ModuleDisc | Control Center modules | 1x1, 2x1, 2x2, full-width; tile is a `Row`-like tile with `Availability` | CTL, on/off (`Check`) | tile, disc, panel | `ModuleState`, `DiscMotion`, `Chevron` | MERGE | P1 |
| Osd | volume / brightness HUD | `LevelIndicator` in a HUD material, `Shown` | `Shown` | body | `OsdPhase` | KEEP | P1 |
| NotificationCard, GroupHeader, BannerStack | banner, Notification Center | swipe to dismiss, action buttons, inline reply field (P2) | swiped | card, header | `banner-*` keyframes | KEEP | P1 |
| AppSwitcher | Cmd-Tab | fit, leaving by `Roster` | selected | tile | own leaving | KEEP | P1 |
| ShotThumbnail, ShotGhost | screenshot thumbnail | corner thumbnail, drag, swipe away | dragging | image | `shot-*` | KEEP | P2 |
| LockScreen, LockClock, LockPrompt, PolkitPrompt | login window, authentication dialog | user picture, `TextField{Secure}`, shake on failure | CTL | picture, field | `SecretEntry`, moods | MERGE | P1 |
| UserPicturePicker | Users & Groups picture chooser | grid of choices | selected | cell | accept beat | KEEP | P2 |
| WidgetFrame, WidgetCard, WidgetGallery, WidgetSlotGuide | desktop widgets | small/medium/large | placed, leaving | card | `CardPresence` | KEEP | P1 |
| ClockFace, MonthGrid, NowPlayingTrack | Clock, Calendar, Now Playing widgets | MonthGrid also under `DatePicker` | none | face, grid | `RollDigits` clock digits | KEEP | P1 |
| Battery widgets | Batteries widget | `BatteryGlyph` + `ProgressIndicator{Ring}` + `Label`, one `BatteryState`, one arc geometry | none | glyph, ring | `BatteryLevel`, `BatteryFigure`, `DeviceBattery`, `RowBattery`, `DeviceGlyph` duplicates | MERGE | P1 |
| IdleDim | display dim | `IdleDim` token | none | overlay | none | KEEP | P2 |
| Dock: DockTile, DockLabel, RunningDot, DockFloor | Dock | `DockTile` = plate + `Badge` + `ProgressIndicator{Bar}` + running dot; `DockLabel` uses `HoverIntent` Label; magnification and bounce in sill | hover (magnify) | tile, label, dot | sill `Tile`, `.sill-dock-label` (L14), `TooltipKind::Fly` | ADD | P1 |
| SpaceEditor, SpaceDot | none (user-settled, 21-SPACES) | dots, theme | none | dot | none | KEEP | P2 |
| AnimatedEmoji | Messages reactions (design/25) | the asset's own animation; no quire loop | none | image | none | KEEP | P2 |

### 2.11 App features and mail-only

Features available on every surface (R6). They are `Row`, `List`, `Popover` and `CommandPalette`
configurations plus the pieces below; their motion uses 1.3 only.

| Feature | Contract | Built from | Where | St | Pri |
| --- | --- | --- | --- | --- | --- |
| Pinned tiles | a grid of square tiles above the sidebar list; drag to reorder; add tile | `PinTile` (from `AccountTile`, `AddAccountTile`) with `Badge`, `List` drop line | apps | MERGE | P1 |
| Today tabs | temporary tabs under the pinned tiles that expire; enter and leave by `Roster` | `Row{Today}` in a SourceList | apps | MERGE | P1 |
| Edge-peek sidebar | a hidden sidebar reveals on pointer at the window edge (`HoverIntent`, Slide(Left) by spring), pins on click | `EdgePeek` (from `EdgeStrip` and the sidebar's peek; `Peek`, mail's reader panel, and `HoverStrip`, the thread row's action strip, stay) | apps | MERGE | P1 |
| Link pill | rounded pill showing the current link; hover-intent expands, click copies | `LinkPill` | apps | KEEP | P2 |
| Grouped launcher commands | palette results grouped by kind with `SectionHeader`; group order per Space | `CommandPalette`, `List` | shell, apps | KEEP | P1 |
| Space switching | Ctrl+1..9 (Cmd+1..9 left to apps; 27 §8.5); Space colour cross-fades `--t-big` | `StandardAction`, `Shortcut` | shell | KEEP | P1 |

Mail-only (`ds::app`; mailo keeps its own look for now):

| Name | AppKit | Contract | Absorbs | St | Pri |
| --- | --- | --- | --- | --- | --- |
| ThreadRow content | `NSTableCellView` | mail content inside `Row` (name, subject, snippet, time, tags, star, hover strip) | `ListRow` component | MERGE | mail |
| SendPill | none | send countdown with `ProgressIndicator{Ring}` | ring, `SendPhase` | KEEP | mail |
| CommandPill | none | compose command pill | none | KEEP | mail |
| ProviderMark | none | provider glyph tile on the ladder | `MarkSize` | KEEP | mail |
| RichText, TextRuns | none | inline runs, links | none | KEEP | mail |

### 2.12 Size adoption and build order

| Follows `ControlSize` | Fixed by its macOS metric | Follows the type scale |
| --- | --- | --- |
| Button, PopUpButton, Slider, Stepper, Toggle, Checkbox, RadioGroup, SegmentedControl, TextField, DatePicker, Chip, Badge, KeyEquivalent, ProgressIndicator, LevelIndicator, Disclosure, TabView strip | Menu (row 22), Popover, Sheet, Alert, SidePanel, WindowTitlebar (28), Toolbar (52), Row (24 / 44), Sidebar (`SidebarSize`), Avatar and IconView (their ladders) | Label, EmptyState, Skeleton, Tooltip, HoverCard |

Build order: 1 tokens and vocabulary; 2 motion and interaction
primitives; 3 `Label`, `Button` (+ `IconButton` merge), `Toggle`, `Checkbox`, `RadioGroup`,
`SegmentedControl`, `Slider`, `TextField`; 4 `ProgressIndicator`, `LevelIndicator`, `Badge`,
`KeyEquivalent`, `VoiceOrb`; 5 `Menu`, `MenuItem`, `PopUpButton`, `Disclosure`; 6 `List`, `Row`, `SectionHeader`;
7 overlays (`Popover`, `Sheet`, `Alert`, `SidePanel`, `Tooltip`, `HoverCard`, `Toast`,
`DockLabel`), `EmptyState`, `Skeleton`; 8 shell-only and app features; 9 P2 (Stepper, DatePicker,
Table, Toolbar, SplitView, Sidebar, TabView, FieldRow, MenuBar model); 10 sill switch-over
(L1-L16 of the component inventory); Arc's ideas arrive as features (section 3.3).

---

## 3. The Look

There is one Look: the Mac values below. It is one value set, not a switch: the library has no
Look type, no `data-look` and no per-Look stylesheet, and a stored `appearance.look` is an unknown
key. What the design takes from the Arc prototype are features (3.3), not values.

### 3.1 What the Look supplies

Colour tokens (light and dark), radii, font families, the material recipes, shadows, the icon plate
and the orb colours, all in `ds-style::tokens`; every component reads them and none carries a value
of its own. The Look never sets a size (`SizeScale`), a duration, easing or motion level, a
behaviour or a default keyboard binding. User styles (ARCHITECTURE section 10) apply after it.

### 3.2 The values

| Value | The Look |
| --- | --- |
| Font UI / display / mono | Inter / Inter / Space Mono (Inter has no monospace; the bundled mono stays) |
| Accent (light / dark) | system blue #007AFF / #0A84FF (conf M); the picker offers macOS's eight (Blue, Purple, Pink, Red, Orange, Yellow, Green, Graphite) |
| Paper (window) | #ECECEC / #1E1E1E (conf M) |
| Surface (control / content) | #FFFFFF / #2A2A2A (conf M) |
| Ink / soft / faint | 85 %, 55 %, 25 % black or white (label, secondary, tertiary) |
| Selection | `--sel-bg` accent, `--sel-bg-quiet` neutral grey `rgba(128,128,128,.25)` |
| Radii ctl / field / seg well / seg / menu / popover / sheet / notification | 5 / 5 / 6 / 5 / 8 / 10 / 10 / 16 |
| Card radius | 10 (group box 6) |
| Window / panel | window 10, Control Center panel 18, module 8 |
| Grain | none: no grain is painted anywhere |
| Shadows | window `0 10px 30px -10px rgba(0,0,0,.35)`; menu and popover a hairline plus soft drop |
| Materials | translucent vibrancy tints; Menu, Popover, Sheet, Sidebar, Bar, Dock, Osd, Toast, Widget, Window (Window = flat `--paper`) |
| Icon plate | abstract embossed plate, matte, per-app gradient, tone band in dark |
| Orb colours (`--orb-bg --orb-c1 --orb-c2 --orb-c3`, light / dark) | oklch(95% .02 264.695), (75% .15 350), (80% .12 200), (78% .14 280) / the same hues at 24%, 68%, 72%, 70% lightness (dark conf L) |
| Backdrop | the Space colour tints the chrome (its gradient over the wallpaper's blur, frame inks from the Space); apps stay on paper |

Materials (`Material`): Window, Bar, Dock, Menu, Popover, Sheet, Sidebar, Toast, Osd, Widget
(Menu and Sidebar are added). Material tints and shadows still carry the values landed before the
Look was one; re-deriving them from the grey neutrals is open, and needs the accent band's grounds
re-measured with them.

Selection is three tokens, `--sel-bg`, `--sel-bg-quiet` and `--sel-ink`, read by `Row`: an active
window's selected row is the accent under its ink, an inactive window's (and a list that does not
hold the keyboard, except a source list, which follows its window) the quiet grey.

### 3.3 What the design takes from Arc

Features, available to every surface (R6), not values:

- the Space colour on the chrome: the bar, dock, launcher and control center tint from the Space,
  with frame inks (`--f-*`) derived from its dots (21 section 2);
- the command pill, pinned tiles, Today tabs, the edge-peek sidebar, the link pill, grouped
  launcher commands and Ctrl+1-9 Space switching (section 2.11);
- the sidebar on colour: a source list draws on the Space's frame.

---

## 4. Drop list

Each row is a DROP: name, then the reason.

| Dropped | Reason |
| --- | --- |
| Press squish, `--squish`, strip `.88`, level swell | macOS buttons do not scale |
| Hover lift `--lift`, module tile lift | not macOS |
| Overshoot / spring pop keyframes, `--overshoot`, `--e-spring`, `menu-pop`, `pop-in`, `chip-in`, `chip-land`, `star-pop`, `seal-pop`, `cmdk-in`, `hc-in`, `osd-in` | R2: springs only on contact; menus open instantly |
| `--tilt`, `--pickup`, `--stagger`, `Reveal`, `use_rise_on`, `StaggerIndex` | no macOS counterpart |
| Motion levels Calm and Extra, per-Look motion tables (Post, Riso, Tide, Candy) | one model; levels are accessibility only |
| Looks Post, Riso, Tide, Candy, warmth, Mochi | Mac and Arc are the two Looks; Arc values come from Post/S |
| Curl, Crumple, Fold, TabOut, BannerOut exits; `*Heavy` durations | Roster fade and slide |
| `use_pulse`, `PulseKey`, `use_nudge`, `use_bump_on`, `Bumped`, `use_once` public | emphasis flourishes |
| `RollDigits`, `use_count_up`, `CountUp` | numbers change instantly |
| `use_sweep`, `use_level_run`, `RunTail`, `Fill`, `Sweep` | no first-show sweeps |
| `Spinner::Breathe`, `breathe`, `spin`, `dest` infinite loops, `Ambient` | one loop: the spinner |
| `SyncHalo` | infinite loop, no counterpart |
| Mailo-era tokens Spark, Send, Float, Sail, BoatReturn, Spin, Nudge, Park, ShakeLong | orphaned |
| `PendingGrace`, `PendingCap` | macOS spinners spin at once |
| Eight presence machines, three roster hooks, `Anim` variants not named in 1.3 | one `Presence`, one `Roster` |
| `Tabs`, the appearance picker, `PickerLayout` | `SegmentedControl` / `RadioGroup` |
| `IconButton`, `PlayPauseButton`, `ButtonVariant`, `ButtonSize`, `ButtonFace`, `Pin`/`Frame`/`Strip` variants | `Button` bezels and `ControlSize` |
| `Spinner`, `RowBattery`, `BatteryLevel`, `BatteryFigure`, `DeviceBattery`, `TrackPosition` bar, dock ring | `ProgressIndicator`, `BatteryGlyph`, `Label` |
| `Count`, `CountPlace` | `Badge` |
| `Chord`, `Kbd`, `KeyHint` as separate items | `KeyEquivalent` |
| `TextInput`, `SearchField`, `SecretEntry` | `TextField` |
| `SelectionBubble` | macOS uses the context menu |
| `MenuKind`, `MenuEntrance`, `PickDismiss`, palette rows, menu row files | `Menu` placement, `MenuItem`, `Row` |
| `SettingsRow`, `SidebarItem`, `TreeItem`, `ListRow` (component), `RowPhase`, `RowWork`, `Succeeded/Failed` flourishes | `Row` and `List` |
| `HeaderKind` Frame/Field/Menu | `Label`, `MenuItem` header |
| `Scrim` (public), `ScrimStrength`, `PanelScrim` | a sheet dims nothing; click-outside is a shell-host concern |
| `Panel` (as a Bottom edge), `SheetPlacement` | `SidePanel`, `Attach` |
| `Elevation`, `Alert Flow` duplicates | one Popover / Alert look |
| `TooltipKind`, `HoverKind::Tip`, `.sill-dock-label`, hover-card `sub` line | `Tooltip`, `DockLabel` |
| `Grip`, `DragReturnFrame` | `DragImage` and `use_drag_return` |
| Toast pull tab, `pill-up` | Toast slides like a banner |
| Portrait moods and life animations, `UserPortrait` component | `Avatar` |
| `Switch`, old `Check`, `Expanded`, `Disclosure`, `ListPresence`, `AvatarMuting`, three `Held`, `Here`, `FirstShow`, `Motion{Play,Still}`, `OsdPhase`, `ShownPhase` | section 1.5 survivors |
| sill `easing.rs::ease`, sill's `sill-dock/src/dock/machine/label.rs` machine, `PDF_THUMB_GRACE`, `THROW_WINDOW`, `FLING_WINDOW`, `glide::FRAME`, `RUN_GAP` | one evaluator, one `HoverIntent`, shared tokens |
| Not built, no consumer: `ComboBox`, `ColorWell`, `ImageWell`, `PathControl`, `RatingIndicator`, column views, charts, iOS patterns | macOS has them; no shell or mail need |

---

## 5. Deferred

| # | Item | Why it waits |
| --- | --- | --- |
| 1 | App layout: frameless Space-coloured window versus toolbar window, per app | decided as each app is built; not designed here |
| 2 | Menu-bar transport for quire apps (push the `MenuBar` model to the shell over IPC, or draw in-window) | COSMIC has no global-menu protocol; 27 §8.6 covers foreign apps only |
| 3 | mailo's move onto a Look and the density step it keeps (29 decision 9) | mailo keeps its current look for now |
| 4 | HUD / inspector panel (`NSPanel`) | no app needs one yet |
| 5 | The measured-values pass: every "conf L" value (spring responses 300/450, Big 400, switch small/regular, mini progress, Look colours) tuned against a real Mac | needs hardware; the defaults ship meanwhile |

---

## 6. Sources

design/27 (HIG parity, wins over the current HIG), design/29 (sizes: templates and HIG 2024
snapshots), design/05 §14 (spring), design/13 (menus), design/10 (dock), design/11 (scroll).
HIG pages cited by name only: Motion, Buttons, Pop-up buttons, Pull-down buttons, Toggles,
Segmented controls, Sliders, Steppers, Text fields, Search fields, Progress indicators, Level
indicators, Disclosure controls, Menus, Context menus, The menu bar, Popovers, Sheets, Alerts,
Tooltips, Notifications, Sidebars, Split views, Toolbars, Tab views, Tables, Focus and
selection, Pointing devices, Keyboards. AppKit names are the contract's reference, not a
dependency.
