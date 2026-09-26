# 26 Details: the grammar of small state changes

Status: D0 built, 2026-09-27, branch `details-d0b` (quire): `ds::detail` holds the grammar as
types and the primitives (section 4.1 is now the sketch they grew from; the shipped API is in
CONSUMING.md "Details"), the tokens of 3.4, `Spinner` on `use_pending`, the harness's
`assert_settles_to_zero_frames`, the lint rules `InfiniteLoop` and `OffGrammarTiming`, and the
gallery's Details page. D1-D7 are not started. The catalogue (section 5) still describes the
state before D0. Status legend as in `13-BEHAVIOUR-menus-windows.md`:
**settled** = decided with the user or already built; **proposed** = chosen here, the user
judges. Reference confidence: **H** the vendor's own guidelines or documentation, **M** a
reliable secondary source (a review, a developer write-up, a support thread that describes the
behaviour), **L** observed from the shipping product and memory, not measured, **UNKNOWN**
nothing found. The reference desktop is named here as a source only; code, classes, tokens and
assets never name it.

**Why this file exists.** The user, 2026-09-26, after asking for the battery ring's fill-in then
count-up and for animated emoji reactions ("its these small details that makes the ux
standout"): "can this kind of small details work for all the items in a principled way? because
apple has them everywhere for example wifi also has subtle details". The answer is yes, and it is
the same answer the reference itself gives: its small details are not hand-made per item; they
are a small fixed vocabulary of symbol and content effects (section 2) that every system element
draws from. This file is that vocabulary for us, fitted to our rules (design/00 §3-4, design/05
§2): a grammar of *moments* every stateful element passes through (section 3), seven primitives
quire implements once (section 4), and a catalogue that assigns a detail to every moment of every
stateful element of the shell and its parts (section 5), with the gap against what is built today.

## 1. What this governs

- The **moments** a stateful element can be in and what each one means (3.1).
- The **motion family** each moment plays, with its tokens (3.2, 3.4), and the **rules** that
  bound it (3.3).
- The **primitives** in quire that play them (section 4) and how a component **declares** its
  moments so every component uses the same machinery (4.2).
- The **catalogue**: every stateful element, its states, its moment-to-detail table, what the
  reference does, what quire and sill do today, and the gap (section 5).
- The **order** in which the gaps close (section 7).

It does not change a keyframe, token or recipe in `05-MOTION.md`; it cites them. New tokens it
needs are listed in 3.4 as proposed, to be added to 05 and the token table in the first wave (the
lint rules `RawDuration`/`RawEasing` apply as always). Where a catalogue row here and a surface row
in `20-SURFACES.md` disagree, the 20 row stands until the user settles the difference; each such
case is listed in section 8.

## 2. What the reference does, as a system

The reference's details are one vocabulary applied everywhere. Its guidelines list the effects and
say what each is for; its system elements (the status menus, Control Center, the Dock, widgets)
use them. That is the principle the user is asking about.

| # | Finding | Conf. | Source |
| --- | --- | --- | --- |
| A1 | A fixed set of symbol animations, "expressive, configurable", that "help communicate ideas, provide feedback in response to people's actions, and signal changes in status or ongoing activities": Appear, Disappear, Bounce, Scale, Pulse, Variable Color, Replace, Magic Replace, Wiggle, Breathe, Rotate, Draw On / Draw Off. Playback runs "from start to finish, or run indefinitely, repeating its effect until a condition is met". | H | HIG, SF Symbols, "Animations" [HIG-SYM] |
| A2 | Bounce "plays once by default and can help communicate that an action occurred or needs to take place". Scale "persists until you set a new scale", for a selected item. | H | [HIG-SYM] |
| A3 | Pulse and Breathe communicate ongoing activity "continuously until a condition is met"; Rotate "when a task is in progress … confirms that it's working". | H | [HIG-SYM] |
| A4 | Variable Color "incrementally varies the opacity of layers … cumulative or iterative … to communicate progress or ongoing activity, such as playback, connecting, or broadcasting"; it can autoreverse and "hide inactive layers". The Wi-Fi symbol is the canonical example: **iterative** (one bar at a time) for searching, **cumulative** (bars fill and stay) for turning on. | H / M | [HIG-SYM]; WWDC23 "What's new in SF Symbols 5" and "Animate symbols in your app" [WWDC23] |
| A5 | Replace has three directions with meanings: down-up ("a change in state"), up-up ("a sense of forward progression"), off-up ("emphasizes the next available state"); Magic Replace draws a slash on and off and adds or removes a badge while the base symbol stays. | H | [HIG-SYM] |
| A6 | Draw On/Off (2025) draws a symbol along its path "to convey progress, as with a download"; Variable Draw fills a symbol's stroke by a value "to convey strength or progress". A symbol takes Variable Color or Variable Draw, never both. | M | WWDC25 "What's new in SF Symbols 7" [WWDC25] |
| A7 | Numbers change by a content transition (`numericText`): the digits roll to the new value instead of snapping; widgets animate between timeline entries since the 2023 release. | M | numericText docs and write-ups [NUM] |
| A8 | "Add motion purposefully … Don't add motion for the sake of adding motion." "Make motion optional … avoid using it as the only way to communicate important information." "Aim for brevity and precision in feedback animations." "Generally avoid adding motion to UI interactions that occur frequently." "Let people cancel motion." | H | HIG, Motion [HIG-MOT] |
| A9 | "When possible, use a determinate progress indicator"; "switch a progress bar from indeterminate to determinate" when the duration becomes known; "Don't switch from the circular style to the bar style"; "If a process stalls … provide feedback that helps people understand the problem." Circular indicators fill clockwise. | H | HIG, Progress indicators [HIG-PROG] |
| A10 | Apply symbol animations "judiciously … too many animations can overwhelm an interface"; each "has a discrete movement that communicates a certain type of action". | H | [HIG-SYM] |

**What we take and what we refuse.** We take the vocabulary and its meanings (A1-A7), the
brevity and cancelability (A8) and the determinate-first rule (A9). We refuse the indefinite
playback where it runs without a bound: the reference animates the Wi-Fi bars "like searching"
even while connected, as it roams (A4; [WIFI-BLINK] M), and escalates its lock-screen
character's annoyance (design/24 §2). Our rules (design/00 §4, design/05 §2 principle 7, the
idle-frame rule) keep a loop only while a real operation runs, and bound even that (3.3 R4).

## 3. The grammar

### 3.1 Moments

Every stateful element is, at any time, at **Rest** or playing one **moment**. A moment is what a
state change *means* to the person looking; the component decides it from its own state machine
(4.2), never from a timer or a render.

| Moment | Meaning | Motion family | Duration, easing | Rules (3.3) |
| --- | --- | --- | --- | --- |
| **Rest** | nothing changed | none | 0 frames | R3 |
| **Appear** | a value is shown for the first time on a surface the person just opened | *Sweep* from zero, *Count* in step, *Reveal* for lists; a mark that arrives alone `pop-in` | Sweep `--t-sweep` `--e-out`; Count in step with it; Reveal `rise` `--t-move` `--e-out` + `--stagger`; mark `pop-in` `--t-move` `--e-spring` only on contact, else `--e-out` | R1, R5, R13 |
| **Pending** | an operation someone started is running and its end is unknown | *bounded pending loop*: Iterate (one layer at a time), Cumulate, Breathe (opacity .45-1) or Spin | one step `--t-pending-step`, linear between steps; starts after `PendingGrace`; holds its still frame after `PendingCap` | R4, R8, R9 |
| **Progress** | an operation with a known share done has advanced | *Sweep* from the current share to the new one; a readout *Count* in step | `--t-quick` `--e-out` per report; no tween for a clock-paced report (media position): it steps | R9, R12 |
| **Success** | the operation ended as asked | *Settle*: Fill (a level glyph fills cumulatively once to its true value), Check (a check draws on), LockIn (the element seals: `gulp`'s shape once) | Fill `--t-pending-step` per layer; Check `--t-move` `--e-out`; LockIn `--t-big`, `--e-spring` only on contact | R5, R6 |
| **Failure** | the operation did not happen | *Shake* once, then the still failure state (a glyph morph to the error mark, words) | `shake-x` `--t-shake` `--e-shake`; morph `--t-quick` | R6, R8 |
| **Change** | a value changed without our asking (a signal got weaker, a charge dropped, a count arrived) | *Morph*: value to value (Sweep + Count, `--t-quick`), glyph to glyph (DownUp / OffUp / CrossFade / Slash), colour cross-fade; a count badge `bump` (design/05 principle 5) | `--t-quick` `--e-out`; `bump` `--t-move` `--e-spring` | R1, R2, R12, R15 |
| **Select** | the person moved a choice (a segment, a switcher cell, a tab) | the indicator *slides* from the old place to the new | `--t-quick` `--e-spring` when the person moved it (contact), `--e-out` when something else did | R5 |
| **Attention** | the element needs the person | *Nudge* once (`nudge`, or the dock's bounce), never a loop without a waiting condition | `--t-nudge` `--e-out`; the dock's bounce per design/10 §10.3.5 | R4, R6 |
| **Unavailable** | the element cannot act now | dim to the disabled opacity (.35, 13.2 R2) by a `--t-quick` cross-fade; no motion inside it; a Pending never plays in it | `--t-quick` `--e-out` | R2 |
| **Preview** | the pointer (or focus) is on the element: show what a press would do | marks fade in (the traffic lights' glyphs), a card expands, a label flies; hover intent where it opens anything | `--t-quick` `--e-out`; expansion `--t-move` `--e-out`; intent 450/150/400 (design/06) | R10 |
| **Dismiss** | the element leaves | the exit design/05 §8 and §10 give its kind (`fold`, `banner-out`, `osd-out`, `shot-out` …); the receiver answers | the exit's token, `--e-exit` | design/05 principles 3, 4, 10 |

Moments that are *not* in the list, on purpose: an Idle "alive" moment (only an animated emoji
has one, design/25 §5, and it is bounded; the persona that first had it was dropped), a Celebration (design/05 principle 11: "never celebrate a
retraction"; concessions are dull, design/00 §4 rule 10), and Escalation (R6).

### 3.2 Motion families

| Family | What moves | How it is played on Blitz | Primitive (section 4) |
| --- | --- | --- | --- |
| Sweep | an arc or bar from share a to share b | a Rust tween writes the share each frame; an SVG arc's path is recomputed (`battery_ring::arc_path`), a bar's width is `--f` on an HTML element | `Sweep` |
| Count | an integer from a to b, in step with a Sweep or on its own clock | the same tween; the text repaints at most every `--t-count-step` | `CountUp` |
| Staggered reveal | the children of a list on first show | CSS `rise` with `--i x --stagger`, capped at 12 (design/05 §9 rule 3) | `Reveal` |
| Bounded pending loop | layers of a glyph in turn, or a disc's opacity, or a ring's turn | a Rust step timer sets `data-step` on stacked layers (one `svg` per layer inside HTML wrappers, as `LevelGlyph` does); stops at the operation's end or the cap | `Pending` |
| Success settle | the final value fills, a check draws, a seal pops | cumulative fill = the step timer once; check = the tween drives `stroke-dashoffset` as an attribute (spike S6); seal = a `gulp` pulse | `Settle` |
| Failure shake | the element's box | `shake-x` through `use_pulse` on the HTML wrapper | `Shake` |
| Value morph, glyph morph, cross-fade | one value or glyph to the next | value: Sweep + Count; glyph: two stacked layers, the outgoing one `data-morph=out` and the incoming `in`, CSS transitions on the wrappers (opacity, scale); colour: a `--t-quick` transition on `currentColor`'s owner | `Morph` |
| Nudge | the element's box, once | `nudge` or the dock's own ballistic bounce | `Nudge` (a thin `use_pulse`) |
| Exit | the element | the existing `Presence`/`Roster` exits | (built: `motion::presence`, `use_roster`) |

Why Rust tweens: Blitz's stylesheet does not reach inside an SVG (spike S6; FINDINGS "Glyph
follows the level"; design/24 §6), and a CSS transition on `stroke-dashoffset` does not run
(design/05 §9 rule 8). So a part *inside* a glyph either is its own stacked `svg` whose HTML
wrapper CSS can move (layers: Pending, glyph Morph, Shake), or is recomputed per frame in Rust
(shapes: Sweep, Check). The tween reads the easing tokens through `CubicBezier::at`
(`motion/curve.rs`), so CSS-played and Rust-played motion share one curve table.

### 3.3 Rules

Each rule is testable; the test named is the one the primitive or the component ships with.

- **R1. Motion only on Appear and change.** A render, a poll that returns the same state, a
  surface re-mapped in place, a theme or Space switch: none of these is a moment. Appear plays
  only when the person opened the surface that shows the value (the control center, the launcher,
  a widget's first map, the session's first frame for widgets); always-there chrome (bar items)
  has no Appear at all. *Test:* re-render with the same state; `Harness::is_animating() == false`.
- **R2. No motion for information that did not change.** The moment is computed on what the
  person can see, not on the raw value: a Wi-Fi strength of 67 then 68 is the same three bars
  (no moment); a battery at 80.4 then 80.1 % shows 80 (no moment); a notification restated with
  the same body does not bump. Each component quantises first, then compares
  (`Detailed::moment`, 4.2). *Test:* a moment table per component (4.2).
- **R3. The idle-frame rule.** At Rest an element paints 0 frames; every primitive's clock stops
  when its moment ends (design/CHECKLIST 7, "Idle surface paints 0 frames"). A primitive never
  uses an `infinite` animation. *Test:* each primitive's harness test ends with
  `is_animating() == false` after `settle`.
- **R4. A pending loop only while a real operation is pending, bounded.** Pending starts from an
  operation the person or the system started and the service reports (an `OpStamp`, 4.1), never
  from a background activity (a scan, a roam, a sync the person did not ask for). It shows only
  if the operation is still running after `PendingGrace` (a fast join shows no loop at all). It
  stops the moment the operation ends. It is bounded twice: after `PendingCap` it holds its still
  frame (the glyph dimmed, the words "Connecting…") while the operation continues, so a stuck
  operation costs 0 frames; and the operation's own timeout (the service's) turns it into a
  Failure. Under Reduced it never loops: it shows the still frame. This replaces the unbounded
  `breathe`/`spin` the `Spinner` plays today (design/05 §12 item 4).
- **R5. Springs only on contact** (design/00 §3). An overshoot (`--e-spring`) is spent only on the
  element the person touched, within the moment that touch caused: the knob they flicked, the
  selection they moved, the success of the join they clicked. The same success arriving from
  elsewhere (auto-join at wake) settles with `--e-out`. The primitive takes a `Touch` (4.1).
- **R6. One reaction per failure, never escalating.** A failure shakes once and holds its still
  state (design/05 principle 7: "A looping error animation is something you learn to ignore
  inside a day"). The same failure (same stamp) never replays; a new failure replays the
  identical shake: no growing amplitude, no second effect for the third try (design/24 §5).
  Attention is the same: one nudge per request, except the dock's bounces, which design/10 owns
  and section 8 item 1 bounds.
- **R7. Reduced motion = the final state at once.** Rust tweens (Sweep, Count, Check) jump to the
  target in one frame; Pending shows its still frame; Shake plays nothing (the failure's glyph and
  words carry it, R8); glyph morphs snap; Reveal has no stagger. CSS pulses keep design/05 §3.2's
  60 ms so settle timers still run. Calm keeps every moment but spends no overshoot
  (`--e-spring` = `--e-out`, as today) and sweeps at the Calm `--t-sweep`.
- **R8. Motion is never the only carrier.** Every moment's meaning is also in a still state:
  Pending has its words and still frame, Failure its mark and words, Success its final glyph,
  Attention its badge or dot ([HIG-MOT] "make motion optional").
- **R9. Determinate first.** If a share is known, the element shows Progress (Sweep), not Pending.
  A Pending may become Progress when the share becomes known; a ring never becomes a bar or back
  ([HIG-PROG]).
- **R10. Nothing waits for a detail.** No moment blocks input. A new state interrupts the moment
  playing, from where it is: a Sweep retargets from its current share, a morph from its current
  layer; the old moment's timer is cancelled (`MotionTimer::cancel`).
- **R11. One moment per element at a time.** A newer moment supersedes; moments never queue.
- **R12. Frequent interactions get the smallest detail** ([HIG-MOT]). A held volume key moves the
  fill with no count-up and no bump per step (the `LevelTick` mark and the sound are its detail);
  the switcher's selection slides but nothing else in the panel moves.
- **R13. Stagger on first show only, capped at 12** (design/05 §9 rule 3, principle 8).
- **R14. Budget.** A moment other than Pending ends within `--t-sweep`; a Count never outlasts
  the Sweep it follows; a Success hold (a check left on screen) is `SettleHold`.
- **R15. A colour change is a Change.** When a hue starts or stops stating a fact (the battery
  turning red at low), it cross-fades over `--t-quick`; it never pulses (design/00 §3, colour is a
  claim).

### 3.4 Tokens

Existing tokens are used wherever they fit; five are new.

| Token | Standard | Calm | Extra | Reduced | Kind | Used by | Status |
| --- | --- | --- | --- | --- | --- | --- | --- |
| `--t-sweep` | 700 ms | 500 ms | 900 ms | 0 (the target at once) | duration | Sweep and Count on Appear (the battery ring's fill-in, a module's level) | proposed |
| `--t-count-step` | 33 ms | 33 ms | 33 ms | n/a | repaint floor (Rust-only) | Count repaints the text at most this often (30 Hz), so 0 to 93 in 700 ms is at most 21 text frames | proposed |
| `--t-pending-step` | 300 ms | 360 ms | 300 ms | n/a (no loop) | duration, linear | one step of a pending loop; a four-layer Wi-Fi cycle is 1200 ms | proposed |
| `PendingGrace` | 400 ms | 400 ms | 400 ms | 400 ms | delay (Rust-only, does not follow the level: it measures the operation, not motion) | a pending loop shows only if the operation runs longer | proposed |
| `PendingCap` | 10 s | 10 s | 10 s | 0 (still at once) | delay (Rust-only) | after this, a pending loop holds its still frame | proposed |
| `SettleHold` | 900 ms | 900 ms | 900 ms | 900 ms | hold (Reduced keeps it: it is reading time, as `--t-send-ring`) | how long a Check stays before the element rests or leaves | proposed |
| `--t-quick`, `--t-move`, `--t-big`, `--t-shake`, `--t-nudge`, `--stagger` | as design/05 §3 | | | | | Change, Select, Preview, Reveal, Shake, Nudge | settled |

None of these is a setting: durations are token-table data, not keys (design/22 §3.2). The
operation timeouts are the services' (sill), not motion tokens.

## 4. The primitives

### 4.1 API sketch (quire, `ds::detail`; built in D0 with the changes CONSUMING.md "Details" lists)

Typed, no `bool` (CONVENTIONS §11), small structs, every effect started from a handler or an
effect hook, never from render (design/05 §7.1). Each primitive is a hook returning what the
component renders, plus, where it helps, a thin component. Names avoid the existing `ds::Count`
(the count badge): the counting primitive is `CountUp`.

```rust
// ---- the grammar as types --------------------------------------------------------------

/// What a state change means (design/26 §3.1).
#[derive(Debug, Clone, Copy, PartialEq, Eq, Hash)]
pub enum Moment {
    Rest,
    Appear,
    Pending,
    Progress,
    Success,
    Failure,
    Change,
    Select,
    Attention,
    Unavailable,
    Preview,
    Dismiss,
}

/// Who caused a moment: only `Contact` may spend an overshoot (R5).
#[derive(Debug, Clone, Copy, PartialEq, Eq, Hash, Default)]
pub enum Touch {
    Contact,
    #[default]
    Remote,
}

/// Whether the surface showing an element was just opened (Appear plays) or the element is
/// re-mounted in place (it does not) (R1).
#[derive(Debug, Clone, Copy, PartialEq, Eq, Hash, Default)]
pub enum FirstShow {
    Animate,
    #[default]
    Still,
}

/// An event's identity, from the service that saw it: the same stamp is the same event, so a
/// repeated failure never replays (R6) and a new operation restarts the cap (R4).
#[derive(Debug, Clone, Copy, PartialEq, Eq, Hash)]
pub struct EventStamp(pub u32);

/// An operation as the service reports it.
#[derive(Debug, Clone, Copy, PartialEq, Eq, Hash, Default)]
pub enum Operation {
    #[default]
    Idle,
    Running(EventStamp),
}

// ---- how a component declares its moments (4.2) -----------------------------------------

/// A component's own state, which knows what each of its transitions means.
pub trait Detailed: Clone + PartialEq + 'static {
    /// The moment `from -> to` is, on what the person sees (quantised first: R2).
    fn moment(from: &Self, to: &Self) -> Moment;
    /// The moment on first show when the surface was just opened; `Moment::Rest` for chrome
    /// that is always there.
    fn first(state: &Self) -> Moment;
}

/// The moment a component is in.
#[derive(Debug, Clone, PartialEq)]
pub struct Detail<S> {
    pub state: S,
    pub moment: Moment,
    pub touch: Touch,
}

/// Track `state`; returns the moment its latest change is, until that moment settles (R11).
pub fn use_detail<S: Detailed>(state: S, first: FirstShow, touch: Touch) -> Detail<S>;

// ---- the frame driver under Sweep, CountUp and Settle's check ------------------------------

/// A share driven frame by frame from a Rust timer, for what CSS cannot reach inside an SVG.
/// Requests frames only while it moves (R3); retargets from where it is (R10).
#[derive(Debug, Clone, Copy, PartialEq)]
pub struct Tween { /* now, from, to, started, spec: Signals */ }

#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub struct TweenSpec {
    pub duration: DurationToken,
    pub easing: EasingToken,
}

impl Tween {
    /// The share now (eased), in thousandths.
    pub fn now(&self) -> Fraction;
    /// How far through its duration the tween is (not eased), for things in step with it.
    pub fn progress(&self) -> Fraction;
}

pub fn use_tween(target: Fraction, spec: TweenSpec) -> Tween;

// ---- Sweep: an arc or bar from a to b -------------------------------------------------------

/// An arc or a bar's share. Appear sweeps from zero over `--t-sweep`; Change and Progress from
/// the current share over `--t-quick`; Rest sits at the level with no frames.
#[derive(Debug, Clone, Copy, PartialEq)]
pub struct Sweep {
    tween: Tween,
}

impl Sweep {
    /// The share to draw now: the SVG arc's span (`battery_ring::Span::filled`) or a bar's `--f`.
    pub fn share(&self) -> Fraction;
}

pub fn use_sweep(level: Fraction, moment: Moment) -> Sweep;

// ---- CountUp: an integer in step -----------------------------------------------------------

/// What paces a count.
#[derive(Debug, Clone, Copy, PartialEq)]
pub enum CountPace {
    /// In step with a sweep: the number reads the arc's own progress, so it lands with it.
    InStep(Sweep),
    /// On its own clock (a readout with no arc).
    Own(DurationToken),
}

#[derive(Debug, Clone, Copy, PartialEq)]
pub struct CountUp { /* shown: Signal<i64> */ }

impl CountUp {
    /// The number to print: never past the target, repainted at most every `--t-count-step`.
    pub fn shown(&self) -> i64;
}

pub fn use_count_up(value: i64, moment: Moment, pace: CountPace) -> CountUp;

// ---- Reveal: staggered children, first show only -------------------------------------------

/// Children play `rise` with `--i x --stagger`, the index capped at 12 (R13), only when
/// `first` is `Animate`; a re-render never replays it.
#[component]
pub fn Reveal(first: FirstShow, children: Element) -> Element;

// ---- Pending: a bounded loop ----------------------------------------------------------------

#[derive(Debug, Clone, Copy, PartialEq, Eq, Hash)]
pub enum PendingStyle {
    /// One layer at a time, then none: the Wi-Fi bars searching.
    Iterate,
    /// Layers fill in turn and stay, then clear: a level being found.
    Cumulate,
    /// A disc's opacity between .45 and 1 (the `busy` keyframe's floor), one step per half.
    Breathe,
    /// A ring's dash turns one step per `--t-pending-step`.
    Spin,
}

/// How many layers a style steps through (the Wi-Fi glyph: the dot and three arcs = 4).
#[derive(Debug, Clone, Copy, PartialEq, Eq, Hash)]
pub struct Layers(pub u8);

#[derive(Debug, Clone, Copy, PartialEq, Eq, Hash)]
pub struct PendingSpec {
    pub style: PendingStyle,
    pub layers: Layers,
}

/// What to draw.
#[derive(Debug, Clone, Copy, PartialEq, Eq, Hash)]
pub enum PendingFrame {
    /// No operation, or one younger than `PendingGrace`: draw the state as it is.
    Idle,
    /// Draw step `n` (layer `n` lit for Iterate, layers `..=n` for Cumulate, …).
    Step(u8),
    /// Past `PendingCap`, or Reduced: the still frame (the glyph dimmed); 0 frames.
    Stalled,
}

pub fn use_pending(op: Operation, spec: PendingSpec) -> PendingFrame;

// ---- Settle: success -------------------------------------------------------------------------

#[derive(Debug, Clone, Copy, PartialEq, Eq, Hash)]
pub enum SettleStyle {
    /// A level glyph fills cumulatively once to its true value, one layer per step.
    Fill(Layers),
    /// A check draws on over `--t-move`, holds `SettleHold`.
    Check,
    /// The element seals: `gulp`'s shape once (`--e-spring` only on `Touch::Contact`).
    LockIn,
}

#[derive(Debug, Clone, Copy, PartialEq)]
pub enum Settling {
    Rest,
    /// Fill: layers `..=n` lit.
    Filling(u8),
    /// Check: the stroke drawn so far (write it as `stroke-dashoffset`).
    Drawing(Fraction),
    /// LockIn: the pulse to render.
    Sealing(PulseKey),
}

pub fn use_settle(success: Option<EventStamp>, style: SettleStyle, touch: Touch) -> Settling;

// ---- Shake: failure ----------------------------------------------------------------------------

/// `shake-x` once per new failure stamp; the same stamp never replays; the same amplitude every
/// time (R6); nothing under Reduced (R7).
pub fn use_shake(failure: Option<EventStamp>) -> PulseKey;

// ---- Morph: value to value, glyph to glyph ---------------------------------------------------

#[derive(Debug, Clone, Copy, PartialEq, Eq, Hash)]
pub enum MorphStyle {
    /// The old glyph shrinks to .7 and fades, the new one grows from .7: a change of state.
    DownUp,
    /// The old glyph goes at once, the new one grows in: the next available action (play/pause).
    OffUp,
    /// Opacity only: two glyphs of the same shape family (the Wi-Fi strengths, battery buckets).
    CrossFade,
    /// The base stays; a slash layer draws on or off (mute, Bluetooth off, Wi-Fi off).
    Slash,
}

/// Two stacked layers, each its own `svg` in an HTML wrapper the stylesheet can move.
#[component]
pub fn MorphGlyph(icon: Icon, size: IconSize, style: MorphStyle) -> Element;

/// Digits that roll to a new value (A7), for a readout that is text (section 8 item 3).
#[component]
pub fn RollDigits(value: String) -> Element;

// ---- Nudge: attention ---------------------------------------------------------------------------

/// `nudge` once per new attention stamp (R6).
pub fn use_nudge(attention: Option<EventStamp>) -> PulseKey;
```

Built on what exists: `use_pulse` and the A/B alias swap (Shake, Nudge, LockIn), `MotionTimer`
and `settle()` (every moment's end), `CubicBezier::at` (Tween), `Presence`/`Roster` (Dismiss),
`LevelGlyph`'s stacked layers (MorphGlyph, Pending's layers), `battery_ring::arc_path` (Sweep's
first consumer), `use_bump_on` (Change for a readout, as today).

### 4.2 How a component declares its states

Every stateful component keeps its state as its own enum (as `PromptState`, `ModuleState`,
`SendPhase` already do) and implements `Detailed` for it: the one place that says what each
transition means. The primitives take the `Detail` and nothing else, so a component cannot play a
moment its table does not name. Example, the bar's network item:

```rust
#[derive(Clone, PartialEq)]
pub enum NetGlyph {
    Off,
    Joining(EventStamp),
    Connected { bars: Bars, lock: Secured },
    NoInternet { bars: Bars },
    Failed(EventStamp),
}

impl Detailed for NetGlyph {
    fn moment(from: &Self, to: &Self) -> Moment {
        use NetGlyph::*;
        match (from, to) {
            (a, b) if a == b => Moment::Rest,
            (_, Joining(_)) => Moment::Pending,
            (Joining(_), Connected { .. }) => Moment::Success,
            (_, Failed(_)) => Moment::Failure,
            (Connected { bars: a, .. }, Connected { bars: b, .. }) if a == b => Moment::Rest,
            _ => Moment::Change,
        }
    }
    fn first(_: &Self) -> Moment {
        Moment::Rest // bar chrome is always there (R1)
    }
}
```

Every such table is tested as data (`detail::moment_table::<NetGlyph>(&[(from, to, Moment)])`),
and every component with a `Detailed` state ships one harness test per moment that ends with
`is_animating() == false` (R3). The CHECKLIST block "State details (26)" makes both a gate.

## 5. The catalogue

How to read an entry: **States** are the element's own enum. The table gives, per moment, **our
detail** (primitive and tokens), **the reference** (with confidence and source key), **today**
(quire `crates/ds`, sill `crates/sill-surfaces` at `ee9dd76`), and the **gap** (numbered `G#`,
section 6 counts them). A moment the element does not have is omitted. "none" under Today means
the state is shown with no motion (a snap).

### 5.1 Bar status items

#### 5.1.1 Wi-Fi item
States: Off, Joining, Connected {bars 1-3, secured}, NoInternet, Failed. Today the sill glyph is a
bucket of `WifiLow`/`Wifi`/`WifiHigh`/`WifiOff`, and `Link::Connecting` shows `WifiOff`
(`bar/status.rs`).

| Moment | Our detail | Reference | Today | Gap |
| --- | --- | --- | --- | --- |
| Pending (joining) | `Pending{Iterate, Layers(4)}`: dot, then each arc alone, 1200 ms a cycle, after 400 ms grace, still (dimmed full glyph) after 10 s | the bars cycle "like searching" while it looks for or joins a network (M, [WIFI-BLINK]); iterative Variable Color is the searching effect (H, [WWDC23]); it also cycles while roaming (M) | shows `WifiOff` while connecting | **G1** the layered Wi-Fi glyph and its pending loop; we do not animate roaming (R4) |
| Success (joined) | `Settle{Fill(Layers(4))}`: the layers fill cumulatively once, up to the real bars | cumulative fill is the "turning on" effect (H, [WWDC23]); the menu bar glyph goes solid (M) | snap to the bucket | **G2** |
| Change (bars) | `MorphGlyph{CrossFade}` between strengths, only when the bar count changes | the glyph changes with strength (H, [SUP-WIFI]); animation unknown | snap | **G3** |
| Change (no internet) | `MorphGlyph{DownUp}` to the glyph with its "!" badge; words in the menu | a distinct "No internet" icon (H, [SUP-WIFI]) | shows `WifiOff` (conflates it with disconnected) | **G4** a no-internet glyph |
| Failure (join failed) | `Shake` once on the bar item's glyph, then `WifiOff`; the reason in the module's words | not found (UNKNOWN); the menu shows an alert | none | **G5** |
| Unavailable (radio off) | `MorphGlyph{Slash}` to Off | a distinct "off" glyph (H) | swap | part of **G3** |

#### 5.1.2 Bluetooth item (off by default, `control_center.menu_bar_bluetooth = Hide`)
States: Off, On, Connecting(device), Connected(n devices), Failed.

| Moment | Our detail | Reference | Today | Gap |
| --- | --- | --- | --- | --- |
| Unavailable ↔ On | `MorphGlyph{Slash}` | slashed glyph when off (L) | `Bluetooth`/`BluetoothOff` swap in sill's module | **G6** the bar item itself (none in sill today) |
| Pending (connecting a device) | `Pending{Breathe}` on the glyph | the glyph pulses while connecting (L) | none | **G7** |
| Change (a device connects) | `MorphGlyph{CrossFade}` to `BluetoothConnected` | the connected variant (L) | not shown in the bar | part of **G6** |

#### 5.1.3 Battery item
States: Discharging(level), Charging(level), Full/OnHold (plugged, not charging), Low (≤ 20 %),
Critical (≤ 10 %), Absent.

| Moment | Our detail | Reference | Today | Gap |
| --- | --- | --- | --- | --- |
| Change (level) | the glyph's inner fill is a `Sweep` (a bar inside the battery outline, `--t-quick`), only when the drawn width changes by a pixel; the optional percentage text changes with no count (R12) | the outline is "partially filled" by level (M, [BATT]) | five bucketed glyphs, snap | **G8** a battery glyph with a continuous fill layer |
| Change (plugged / unplugged) | `MorphGlyph{Slash}`-like badge in and out: the bolt layer draws on (`--t-quick`) | "lightning bolt" while charging, a plug when held (M, [BATT]) | `BatteryCharging` swap | **G9** bolt as a layer, and the on-hold plug state |
| Change (low) | the fill cross-fades to `--battery-low` (R15) | the outline "turning red" under about 20 % (M, [BATT]) | `BatteryWarning` at ≤ 10 % | **G10** colour as the claim, at the reference's threshold (a key: `bar.battery_low_percent`, proposed) |
| Attention (critical) | `Nudge` once when crossing into Critical, plus the system notification | a notification at low battery (L) | none | **G11** |

#### 5.1.4 Volume item
States: Muted, Level(0-3 waves), NoDevice. `LevelGlyph::Volume(Muting)` already cross-fades its
waves by thirds and draws a slash (quire `level/glyph.rs`); the bar uses static `Volume*` icons.

| Moment | Our detail | Reference | Today | Gap |
| --- | --- | --- | --- | --- |
| Change (waves) | the bar item uses `LevelGlyph` (waves cross-fade `--t-quick`) | waves follow the level (L) | bucketed `VolumeX/Volume/Volume1/Volume2`, snap | **G12** the bar item on `LevelGlyph` |
| Change (mute) | `MorphGlyph{Slash}`: waves out, slash draws on | Magic Replace draws the slash (H, [HIG-SYM]) | swap to `VolumeX` | part of **G12** |

#### 5.1.5 Clock
States: the minute (and the second if shown), the date.

| Moment | Our detail | Reference | Today | Gap |
| --- | --- | --- | --- | --- |
| Change (minute) | none: the text is replaced (R12, a clock is read, not watched) | none observed (L) | none | none |

#### 5.1.6 Workspace pills and the control center item
| Moment | Our detail | Reference | Today | Gap |
| --- | --- | --- | --- | --- |
| Select (workspace) | the pill's `--f-pill` background slides to the new pill `--t-quick` `--e-spring` when the person switched, `--e-out` when a window moved them | Spaces indicator not a system element (UNKNOWN) | background cross-fade `--t-quick` (`workspace_pills.css`) | **G13** a sliding indicator (shared with Segmented, 5.10.2) |
| Preview (open menu) | the item's pill at once (13.3.1: no transition) | open with no animation (L, 13.2 R3) | as specified | none |

### 5.2 Control center

#### 5.2.1 The panel
| Moment | Our detail | Reference | Today | Gap |
| --- | --- | --- | --- | --- |
| Appear | `menu-pop`; the panel's levels and toggles are shown still (the person opened it to act, not to watch; R12); only the Battery module's rings sweep in (5.2.9) | panel appears with no published numbers (13.2 R9) | `menu-pop` | none |
| Select (detail pane) | `PaneSwitcher` (built) | slides the detail in (L) | built | none |

#### 5.2.2 Wi-Fi module and networks pane
States: Off, On(idle), Scanning, Joining(ssid), Connected(ssid), Failed(ssid, reason); each
network row: Known/New, Secured/Open, bars, Connected/Joining/Idle.

| Moment | Our detail | Reference | Today | Gap |
| --- | --- | --- | --- | --- |
| Change (toggle) | the tile's disc cross-fades Off ↔ On (built); the disc's glyph `Settle{Fill(Layers(4))}` on turning on (`Touch::Contact`) | cumulative fill for enabling (H, [WWDC23]) | colour cross-fade only | **G14** the disc glyph's fill on enable |
| Pending (scanning, joining) | tile disc: `Pending{Iterate}` on its Wi-Fi glyph; the joining row: `Pending{Spin}` in the row's trailing slot, replacing the lock | the joining row shows a small spinner; "Other Networks" scans with a spinner (L) | `ModuleState::Busy` = `Spinner{Breathe}`, **infinite**; row says "Connecting…" | **G15** Busy on the bounded `Pending` (removes an infinite loop); **G16** row spinner |
| Success | the row's disc goes accent (the connected row's filled disc) by `Settle{LockIn}` (`Touch::Contact`); the tile status word changes | connected network shows a filled accent disc (L) | snap | **G17** |
| Failure | the row `Shake`s once; its detail line says why ("Wrong password"); the password sheet reopens for a secured one | a password prompt shakes on a wrong key (M, [SHAKE] pattern) | none | **G18** |
| Change (bars per row) | `MorphGlyph{CrossFade}` only when bars change (R2) | bars per network (L) | snap | part of **G3** |
| Appear (the list) | `Reveal` on the pane's first show | the list appears whole (L) | none | **G19** (small: Reveal on panes) |

#### 5.2.3 Bluetooth module and devices pane
States as Wi-Fi's, per device: Paired, Connecting, Connected {battery}, Failed.

| Moment | Our detail | Reference | Today | Gap |
| --- | --- | --- | --- | --- |
| Pending (connecting) | the device row's glyph disc `Pending{Breathe}` | a spinner on the device row (L) | the row's check dropped while connecting (`RowMark::Nothing`), no motion | **G20** |
| Success | row disc to accent, `Settle{LockIn}`; the device battery `Sweep` in from zero with its `CountUp` (Appear of a value) | connected devices show battery levels (L) | snap | **G21** |
| Failure | `Shake` the row once; words | UNKNOWN | none | part of **G20** |

#### 5.2.4 Sharing (AirDrop-like) and screen mirroring
States: Off, Discoverable, Sending(progress), Received; Mirroring: Off, Searching, Connecting,
Mirroring(display).

| Moment | Our detail | Reference | Today | Gap |
| --- | --- | --- | --- | --- |
| Pending (searching displays) | `Pending{Iterate}` on the display glyph's waves | the list searches with a spinner (L) | no module | **G22** modules not built (sharing, mirroring); details specified here so they are born with them |
| Progress (sending) | a ring `Sweep` round the recipient's avatar | a progress ring round the recipient (L) | none | part of **G22** |
| Success | `Settle{Check}` on the avatar, `SettleHold` | "Sent" under the recipient (L) | none | part of **G22** |

#### 5.2.5 Focus / Do Not Disturb
States: Off, On(mode), Scheduled.

| Moment | Our detail | Reference | Today | Gap |
| --- | --- | --- | --- | --- |
| Change (on) | the moon glyph `MorphGlyph{DownUp}` to its filled variant, `Touch::Contact` spring | a Bounce confirms an action (H, [HIG-SYM]); the Focus tile turns accent (L) | disc colour cross-fade | **G23** |
| Change (the bar) | a Focus mark appears in the bar with `pop-in` (`--e-out`: not contact with the bar) | the Focus glyph appears in the menu bar (L) | none | **G24** |

#### 5.2.6 Display brightness, 5.2.7 Keyboard brightness
States: Level(share), Auto.

| Moment | Our detail | Reference | Today | Gap |
| --- | --- | --- | --- | --- |
| Change (from keys or auto) | fill `Sweep` `--t-quick` (built as the level's transition); the sun's rays scale with it (built) | the slider moves (L) | built (`LevelControl`) | none |
| Change (by drag) | no tween under the pointer (built) | follows the finger (H, [HIG-MOT] "realistic feedback") | built | none |
| Keyboard brightness | same `LevelControl`, a keyboard glyph with the same rays treatment | a keyboard-brightness module (L) | no module | **G25** module (glyph layers) |

#### 5.2.8 Sound with output device
States: Level, Muted, Device(n), Switching(device).

| Moment | Our detail | Reference | Today | Gap |
| --- | --- | --- | --- | --- |
| Change (level, mute) | built (`LevelGlyph`) | as 5.1.4 | built | none |
| Pending (switching output) | the chosen row's glyph `Pending{Breathe}` until the sink reports | a brief spinner on AirPlay targets (L) | none | **G26** |
| Success | row check draws on, `Settle{Check}` | check mark moves to the new device (L) | check snaps | part of **G26** |

#### 5.2.9 Battery module
| Moment | Our detail | Reference | Today | Gap |
| --- | --- | --- | --- | --- |
| Appear | each device ring `Sweep` from zero over `--t-sweep` with its percentage `CountUp::InStep` (the user's ask) | widget content transitions (M, [NUM]) | `BatteryLevel` bumps on change only | **G27** (the battery-fill lane in flight; should consume `Sweep`/`CountUp`) |
| Change | `Sweep` from the current share `--t-quick`, count in step; no count for a one-point step (R12) | numericText rolls the digits (M) | `bump` | part of **G27** |

#### 5.2.10 Now Playing
States: NotPlaying, Playing(track, position), Paused, Buffering, trackchange.

| Moment | Our detail | Reference | Today | Gap |
| --- | --- | --- | --- | --- |
| Change (play ↔ pause) | `MorphGlyph{OffUp}` (the next action grows in), `Touch::Contact` | Replace off-up "emphasizes the next available state or action" (H, [HIG-SYM]) | `Icon::Play`/`Pause` swap | **G28** |
| Progress (position) | the position bar steps once a second, no tween (a clock-paced report; 1 frame/s, only while the panel is open) | a scrubber advancing (L) | no progress bar | **G29** position bar |
| Change (track) | art and titles cross-fade `--t-quick` | cross-fade of artwork (L) | snap | **G30** |
| Pending (buffering) | `Pending{Breathe}` on the art | UNKNOWN | none | part of **G29** |

#### 5.2.11 Power profile (segmented) and Appearance picker
See 5.10.2 (Segmented) and `AppearancePicker` (built; selection ring is its own).

### 5.3 OSD

#### 5.3.1 Volume and brightness OSD
States: Hidden, Shown(level, glyph), Muted.

| Moment | Our detail | Reference | Today | Gap |
| --- | --- | --- | --- | --- |
| Appear | `osd-in` (built); the level shown still (it is the value before the key's step), then the step is a Change | a small panel under the menu bar's matching item (M, [OSD]); 27.2 draws a **segmented** bar "making it clearer how much each key press adjusts" (M, [OSD]) | built; `LevelLook::Segments` exists | **G31** use `Segments` for the OSD by default (the reference's newest look), user to confirm |
| Change (key step) | fill `--t-quick`, `LevelTick` mark, sound (built) | one segment lights per press (M) | built | none |
| Change (mute) | `MorphGlyph{Slash}` on the capsule's glyph (built through `LevelGlyph`) | slash (H) | built | none |
| Dismiss | `osd-out` after `osd.hold_ms` (built) | fades (L) | built | none |

### 5.4 Notifications

| Element / moment | Our detail | Reference | Today | Gap |
| --- | --- | --- | --- | --- |
| Arrival | `banner-in` `--t-move --e-spring` (built) | slides in from the right (M, 13.2 R8) | built | none (the spring here is design/05 §12 item 12's open reading) |
| Grouping (a second from the same app) | the card's content cross-fades `--t-quick`; `GroupCount` chip `bump`; a new offset layer `pop-in` under it | stacked group with layers beneath (M, [NC]) | layers and chip built; content snaps; chip does not bump | **G32** |
| Expand (hover) | height `--t-move --e-out`, actions fade (built) | hover expands (M) | built | none |
| Expand (group in the center) | the group's cards `Reveal` downward from the head, `--stagger`; collapse `heal` back | click the stack to expand; "Show Less" collapses (M, [NC]) | `GroupHeader` exists; no expand motion | **G33** |
| Clear (hover on the group's X) | the X morphs into a "Clear" pill (`--t-quick` width and cross-fade) | "a large X icon … turns to a Clear All button" (M, [NC]) | 18 px close only | **G34** |
| Dismiss | `banner-out` + `heal` (built); center rows `fold` (20 §1.6) | slides out (M) | built | none |
| Action success (Reply sent, Mark as read) | the pressed button `Settle{Check}` `SettleHold`, then the card exits | the banner closes after the action (L) | closes at once | **G35** |
| Action failure | `Shake` the card once, a line of words | UNKNOWN | none | part of **G35** |

### 5.5 Dock

| Element / moment | Our detail | Reference | Today | Gap |
| --- | --- | --- | --- | --- |
| Launch (Pending) | the launch bounce (design/10 §10.3.5; a real pending operation, capped at 10 s) | bounces until launched (H, design/10 R8) | built in sill (`machine/bounce.rs`) | none |
| Running dot (Appear/Dismiss) | `fade` `--t-quick` in (built); out `fade` reversed when the last window closes | the dot appears (L) | in built, out snaps | **G36** (small) |
| Badge (Appear / Change / Dismiss) | `pop-in` on first count (`--e-out`: not contact), `bump` on change (built through `Count`), `menu-out`-like fade on clear | badge appears with the count (H meaning, L motion) | `Count` bumps on change; appears and clears with no motion | **G37** |
| Progress ring (Progress) | `Sweep` from the current share `--t-quick` (10.3.2: no faster); Appear from zero; at 100 % `Settle{LockIn}` then fade | macOS draws a bar under the icon (L); the Downloads stack shows a bar (M, [DL]) | ring drawn at the share, snap | **G38** |
| Attention | informational bounce 1 s; critical repeats (design/10) | as design/10 (H) | built | section 8 item 1 (the critical loop's bound) |
| Download finished (the stack) | the Downloads tile bounces **once** (informational) and its badge bumps | "When a file finishes downloading, the stack notifies you by bouncing" (M, [DL]) | no stack tile | **G39** |
| Drag out to remove | the tile fades and scales out `--t-move --e-exit`; neighbours heal; `item-deleted` sound (13.3.10) | the "poof" (H, 13.2 R13) | reorder built; remove motion unknown | **G40** |
| Quit (from the menu) | the running dot fades (G36); nothing else | the dot disappears (L) | as G36 | part of G36 |

### 5.6 Launcher

| Element / moment | Our detail | Reference | Today | Gap |
| --- | --- | --- | --- | --- |
| Open / close | `peek-in` / `fade` (built) | appears quickly (L) | built | none |
| Results (Appear) | `Reveal` on the first result set after the panel opens; later result sets replace in place with **no** stagger (R1, R12: typing is frequent) | results update in place as you type (L) | no rise | **G41** |
| Calculator result | the result row's value `RollDigits` as the expression grows; an invalid expression shows no row (no error motion) | inline answer as you type (L) | a calculator provider exists? (sill M9 providers) | **G42** |
| No results | a still "No results" line (`fade` `--t-quick`); no shake (typing is not a failure) | a "no results" state (L) | none | **G43** (words and fade) |
| Selection (Select) | moves instantly (13.3.9, settled) | instant (L) | built | none |

### 5.7 Lock screen and polkit

| Element / moment | Our detail | Reference | Today | Gap |
| --- | --- | --- | --- | --- |
| Typing | each dot appears `pop-in` at `--t-tap` `--e-out` (typing is contact, but frequent: no overshoot, R12); the user's picture goes Attentive (built, design/25 §7) | dots appear per key (L) | dots snap | **G44** (small) |
| Checking (Pending) | the enter arrow `Pending{Spin}` bounded (replaces the `Spinner`'s infinite spin) | a spinner while authenticating (L) | `Checking` spins (unbounded) | **G45** |
| Wrong password (Failure) | `Shake` once, empty the field on settle; the picture winces once (built); same every time (R6) | the login window shakes (M, [SHAKE]: about three shakes in 0.3 s); the character escalates (M, design/24) | built | none (we refuse escalation) |
| Unlock (Success) | the picture's accept beat, `picture-accept` (built), then the lock surface `fade` `--t-move --e-exit` | fade to the desktop (L) | built (20 §1.9) | none |
| Caps lock | the mark `pop-in` / fade `--t-quick` | caps-lock glyph in the field (L) | snaps | **G46** (small) |
| Locked out | the field dims (Unavailable), the time in the hint | "try again in" (L) | built (words) | none |
| Polkit | same as the lock prompt (built) | same (M) | built | follows G45 |

### 5.8 App switcher

| Element / moment | Our detail | Reference | Today | Gap |
| --- | --- | --- | --- | --- |
| Appear | after the show delay (13.3.5), `fade` `--t-quick`, no pop (built) | appears after a short hold (M, 13.2 R6) | built | none |
| Select | the selection square slides `--t-quick --e-spring` (built) | the highlight moves (L) | built | none |
| Quit (Q) | the tile `fold`s, the row heals (built in quire; `Anim::Fold`) | the icon leaves the switcher (M, [CMDTAB]) | built | none |
| Hide (H) | the tile dims to .5 once (`--t-quick`) and stays: hidden is a state | hidden apps stay listed (L) | none | **G47** (small) |

### 5.9 Widgets

| Element / moment | Our detail | Reference | Today | Gap |
| --- | --- | --- | --- | --- |
| Battery ring (Appear) | `Sweep` from zero over `--t-sweep`, percentage `CountUp::InStep` (the user's ask), on the widget's first map in a session and when the widget is added | widgets animate between entries (M, [NUM]) | `BatteryLevel` bump on change only | **G27** |
| Battery ring (Change) | `Sweep` `--t-quick` + count in step; one-point steps: no count, the text changes (R12) | digits roll (M) | `bump` | part of G27; section 8 item 3 (bump vs roll) |
| Battery (charging) | the bolt layer draws on in the ring's gap, the gap opens by `Sweep` | a bolt in a gap at twelve (M, design/23 M11) | drawn still | **G48** |
| Clock second hand | **tick**: one frame a second, no tween; only when seconds are shown | the second hand sweeps smoothly (L, [CLOCK]) | ticks (a `Seconds::Shown(u8)` per render) | section 8 item 2 (sweep costs 60 frames a second forever: refused by R3 unless the user overrides) |
| Clock minute | none (R12) | none (L) | none | none |
| Calendar today (day rollover) | the accent disc cross-fades from yesterday to today `--t-quick` at midnight | the date changes (L) | snaps | **G49** (small) |
| World clock day/night | the face cross-fades white ↔ dark `--t-big` at the zone's sunrise and sunset | faces by day and night (M, design/23 M22) | snaps | **G50** (small) |

### 5.10 Controls

#### 5.10.1 Toggle
| Moment | Our detail | Reference | Today | Gap |
| --- | --- | --- | --- | --- |
| Change (by press) | knob `--t-move --e-spring` (built, contact), track colour `--t-quick` | the knob slides (H, platform switch) | built | none |
| Change (from outside) | knob `--e-out` (not contact, R5) | UNKNOWN | spring either way | **G51** (the toggle takes a `Touch`) |
| Pending (the change needs a service) | the knob moves at once (the press is honoured), the module's disc carries the Pending; on failure the knob returns with `--e-out` and the row shakes once | switches flip then revert on failure (L) | none | part of G51 |

#### 5.10.2 Segmented control
| Moment | Our detail | Reference | Today | Gap |
| --- | --- | --- | --- | --- |
| Select | the selected plate slides from the old segment to the new `--t-quick --e-spring` (contact) | the selection slides between segments (L) | background cross-fade per segment | **G13** (one sliding indicator for Segmented, WorkspacePills, Tabs) |

#### 5.10.3 Slider / level
Built: drag with no tween, rubber band, press swell, `LevelTick`. No gap.

#### 5.10.4 Menu check (a picked item in a menu)
| Moment | Our detail | Reference | Today | Gap |
| --- | --- | --- | --- | --- |
| Success (pick) | the row blinks once (`--t-tap` off, `--t-tap` on) before the menu closes | the chosen item flashes after release (M, [MENU-BLINK]) | closes then picks (13.9 item 3 open) | **G52** (decide 13.9 item 3: recommend blink) |

#### 5.10.5 Text field, button
Focus ring and press `--squish` built; a field's error is Failure: `Shake` once plus the error
words (built for the lock and polkit prompts; not a general `TextInput` state). **G53**: a
`FieldState::Invalid(EventStamp)` on `TextInput` so every form shakes the same way.

### 5.11 Menus

| Moment | Our detail | Reference | Today | Gap |
| --- | --- | --- | --- | --- |
| Open | `menu-pop` first open, instant while switching (built) | no open animation (L, 13.2 R3) | built | none (05 §12 item 6 decided the pop) |
| Submenu | no animation (built) | none (L) | built | none |
| Disabled | 35 % opacity, no hover, no motion (Unavailable) | 35 % (H, 13.2 R2) | built | none |
| Close | `menu-out` `--t-quick` (built) | fades on close (L) | built | none |
| Pick | see 5.10.4 | | | G52 |

### 5.12 Window frame

| Moment | Our detail | Reference | Today | Gap |
| --- | --- | --- | --- | --- |
| Preview (hover on the lights) | all three marks fade in `--t-quick` (built) | marks appear on all three when the pointer is over the group (H, platform behaviour) | built | none |
| Change (active ↔ inactive) | lights cross-fade colour ↔ grey `--t-quick` | grey in inactive windows (H) | `background-color --t-quick` (built) | none |
| Zoom | the compositor's (not ours) | zoom animation (L) | not ours | none (limit) |
| Tile menu | `menu-pop` (built) | the tiling menu (M) | built | none |

### 5.13 Screenshot thumbnail

| Moment | Our detail | Reference | Today | Gap |
| --- | --- | --- | --- | --- |
| Appear | `ShotIn` (built) | slides in at the bottom right (H, platform help) | built | none |
| Dismiss (hold ends, swipe) | `ShotOut`, swipe (built) | slides away (H) | built | none |
| Success (copied, saved) | `Settle{Check}` over the picture `SettleHold` then out | UNKNOWN | none | **G54** (small) |

### 5.14 Downloads and progress (any surface)

| Moment | Our detail | Reference | Today | Gap |
| --- | --- | --- | --- | --- |
| Pending → Progress | a ring `Pending{Spin}` until the size is known, then `Sweep` of the same ring (R9: ring stays ring) | determinate when possible, never ring ↔ bar (H, [HIG-PROG]) | no general progress component (`SendRing` only) | **G55** a `ProgressRing` / `ProgressBar` pair on `Sweep` |
| Success | `Settle{Check}` on the ring; the Downloads dock tile bounces once (5.5) | Draw On a check or download symbol (M, [WWDC25]) | none | part of G55 |
| Failure | ring turns `--danger` (R15), `Shake` once | "provide feedback that helps people understand the problem" (H) | none | part of G55 |

### 5.15 Power menu, sheets

Built: `peek-in`, `sheet-out`, scrim. A destructive action pressed (Restart) is Pending while the
session ends: the button's label is replaced by `Pending{Spin}` after `PendingGrace`. **G56**
(small).

## 6. Counts

- Elements catalogued: **51** (bar 7, control center 12, OSD 1, notifications 3 [banner,
  group, center], dock 6, launcher 4, lock and polkit 2, switcher 1, widgets 4, controls 6
  [toggle, segmented, slider, menu check, text field, button], menus 1, window frame 1,
  screenshot 1, progress 1, power menu 1), in **109** moment rows.
- Gaps: **56** (`G1`-`G56`). By size: 14 small (one component's CSS or one pulse: G19, G36,
  G44, G46, G47, G49, G50, G54, G56, G43, G34, G30, G24, G11), 38 medium (a component adopting
  a primitive), 4 large (new components: the layered status glyphs G1/G8, `ProgressRing` G55, the
  unbuilt modules G22).
- Infinite loops the grammar removes: 2 (`Spinner{Breathe}` behind `ModuleState::Busy`, G15;
  `Spinner{Spin}` behind `PromptState::Checking`, G45). Both keep Blitz from ever idling while
  shown today (`spinner.css` TODO O-9).
- Moment rows already right today: **34** (Gap "none"); the other 75 rows carry the 56 gaps.

## 7. Waves

Lanes are agent-sized (one branch, one gate, about a day of agent work). **Q** = quire only;
**Q+S** = a quire lane and a sill lane after it (the sill lane wires services' states and stamps
into the new component states). The two lanes in flight (battery-fill, animated-emoji) should
land on the primitives rather than beside them: battery-fill is the first consumer of `Sweep` and
`CountUp`.

| Wave | Lane | Scope | Where | Closes |
| --- | --- | --- | --- | --- |
| D0 | D0a primitives: time | `Moment`, `Touch`, `FirstShow`, `EventStamp`, `Operation`, `Detailed`, `use_detail`, `moment_table`; `Tween` on `CubicBezier::at` and a frame clock that stops at rest; `Sweep`, `CountUp`, `Reveal`; tokens `--t-sweep`, `--t-count-step`, `SettleHold` into design/05 and the token table; gallery page "Details" with a replay button per primitive; harness tests (idle after settle, Reduced jumps, retarget mid-flight) | Q | foundation. **Built 2026-09-27** (`details-d0b`) |
| D0 | D0b primitives: state | `Pending` (+ `--t-pending-step`, `PendingGrace`, `PendingCap`), `Settle`, `Shake`, `MorphGlyph`, `RollDigits`, `Nudge`; `Spinner` rebuilt on `Pending` (its two infinite loops gone; `ModuleState::Busy` and `PromptState::Checking` inherit the bound; `SyncHalo`'s idle breathe, mailo's, goes as design/05 §12 item 4 proposes, so tell the mailo session before it lands) | Q | G15, G45 (quire side). **Built 2026-09-27** (`details-d0b`), with one change: `SyncHalo` keeps its loops (it is mail's; mailo decides, design/05 §12 item 4), listed in `ds/tests/details_lint.rs` with the three other mail loops |
| D1 | status glyphs | a layered `StatusGlyph` family: Wi-Fi (dot + 3 arcs, the "!" badge), battery (outline, fill layer, bolt, plug), Bluetooth (base, slash, connected dots), volume (on `LevelGlyph`), each with its `Detailed` state and moment table | Q | G1-G4, G8-G10, G12 (quire side) |
| D1 | bar wiring | sill's bar items on `StatusGlyph`: `Link::Connecting` → Joining with an op stamp from the network service, no-internet from connectivity, battery thresholds (`bar.battery_low_percent`), the Bluetooth bar item | S | G1-G12 |
| D2 | control center modules | `ModuleTile` disc glyph on `MorphGlyph`/`Settle`; `SettingsRow` trailing `Pending`/`Settle{Check}`/`Shake`; Now Playing play/pause `MorphGlyph{OffUp}`, position bar, track cross-fade; Battery module rings on `Sweep`/`CountUp`; a keyboard-brightness module on `LevelControl` | Q+S | G14, G16-G18, G20, G21, G23, G25-G30 |
| D3 | OSD | default `LevelLook::Segments` for the OSD (user confirms), nothing else | Q+S (a settings default) | G31 |
| D4 | dock | badge appear and clear, running dot out, progress ring on `Sweep` + `Settle`, remove poof, Downloads stack tile's one bounce | S (quire only if `ProgressRing` from D6 is not in yet) | G36-G40 |
| D5 | notifications | group content cross-fade and chip bump, center group expand (`Reveal`) and collapse (`heal`), the X → Clear morph, action `Settle`/`Shake` | Q+S | G32-G35 |
| D6 | progress and forms | `ProgressRing`/`ProgressBar` on `Sweep` (Pending → Progress), `TextInput` `Invalid` state, sliding selection indicator for `SegmentedControl`/`WorkspacePills`/`Tabs`, `Toggle { touch }` | Q | G13, G51, G53, G55 |
| D7 | small ones | launcher Reveal and no-results, lock dots and caps mark, switcher hide, widgets (day rollover, day/night cross-fade, bolt gap), screenshot check, power-menu pending, menu pick blink (after the 13.9 decision) | Q+S | G19, G24, G41-G44, G46-G50, G52, G54, G56, G22 (details only; the modules are their own milestone) |

Order by visibility after D0: D1 (the bar is always on screen), D2, D3, D4, D5, then D6 and D7.
D0a and D0b can run in parallel; D1's quire lane needs both.

## 8. Open decisions

1. **The dock's critical bounce.** design/10 §10.3.5 repeats it until the app is active (the
   reference). R4 wants every loop bounded. Proposed: bounce for `PendingCap` (10 s), then hold
   the tile lifted by a quarter of its height with the badge, 0 frames, until the app is active.
   The user decides; design/05 §12 item 4 proposed keeping the loop.
2. **Clock second hand: tick or sweep.** A sweep repaints 60 times a second for as long as the
   widget is visible, which R3 forbids; a tick is one frame a second. Proposed: tick (built).
3. **Readouts: bump or roll.** design/20 §1.14 settles "value change `bump`" for widgets; the
   reference rolls digits (A7). Proposed: a count badge bumps (design/05 principle 5, a receiver
   answering); a readout (a percentage, a temperature) rolls with `RollDigits` over `--t-quick`.
4. **OSD look.** The reference's newest OSD is segmented (M, [OSD]); ours defaults to Capsule.
   Proposed: `Segments` for the OSD, Capsule in the control center (as the reference: its
   Control Center sliders "did not change").
5. **Menu pick blink** (13.9 item 3). Proposed: blink once.
6. **Launcher results Reveal.** Proposed: only the first result set after opening staggers.

## 9. Sources

- [HIG-SYM] Apple Human Interface Guidelines, "SF Symbols", section "Animations"
  (developer.apple.com/design/human-interface-guidelines/sf-symbols; read 2026-09-26 through the
  page's JSON). H.
- [HIG-MOT] Apple HIG, "Motion" (…/motion). H.
- [HIG-PROG] Apple HIG, "Progress indicators" (…/progress-indicators) and "Loading". H.
- [WWDC23] "What's new in SF Symbols 5" (WWDC23 10197, WWDC Notes summary) and "Animate symbols
  in your app" (WWDC23 10258); Alexander Logan, "Symbol Animations" (the Wi-Fi iterative example).
  H / M.
- [WWDC25] "What's new in SF Symbols 7" (WWDC25 337, WWDC Notes summary): Draw On/Off, Variable
  Draw. M.
- [NUM] Apple, `ContentTransition.numericText(value:)`; Create with Swift, "Animating numeric
  text in SwiftUI"; widget animations since the 2023 release (Juniper Photon, "Animate widget
  changes"). M.
- [SUP-WIFI] Apple Support, "Wi-Fi menu icons on Mac" (support.apple.com/guide/mac-help/mchlcedc581e):
  seven icons including "No internet". H.
- [WIFI-BLINK] Apple Community, "Blinking WiFi icon in menu bar" (discussions.apple.com/thread/8025003):
  the bars cycle while searching and while roaming. M.
- [BATT] Juicy, "Mac Battery Menu Bar Symbols Explained"; Apple Community "Red Battery icon". M.
- [OSD] MacRumors, "macOS 27.2 Beta Brings New Volume and Brightness Sliders", 2026-09-21; Many
  Tricks, "Hudlum" (the corner popovers since Tahoe). M.
- [SHAKE] Cocoa Is My Girlfriend, "Core Animation Tutorial: Window Shake Effect" (the login
  window's shake; about three shakes over 0.3 s); YouTube "OS X login shake animation". M.
- [MENU-BLINK] macosx.com, "Menu Blinking" (the chosen item flashes after release; the blink
  count was a setting in the classic OS). M.
- [DL] Wikipedia, "Stacks (Mac OS)" (the Downloads stack bounces when a download finishes);
  Apple Community "Progress bar under Downloads in Dock". M.
- [NC] Macworld, "macOS Big Sur … Notification Center"; AppleInsider, "How to make the most of
  notifications in macOS Big Sur" (stacks, Show Less, the X that becomes Clear All). M.
- [CMDTAB] How-To Geek, "Quit and Hide macOS Apps from the Command+Tab Interface". M.
- [CLOCK] MacRumors Forums, "Interesting Clock icon behavior" (the second hand moves smoothly). L.
- Our docs: design/00 §3-4, design/05 (§2, §3, §7, §9, §10, §12), design/10 §10.3.2 and §10.3.5,
  design/13 §13.2-13.3, design/20 §1, design/22 §3.2, design/23 §1.1, design/24 §5.
- Code read: quire `crates/ds/src/motion/*`, `components/{battery_level,battery_ring,count,bump_on,
  spinner,sync_halo,module_tile*,level/*,osd*,lock_*,app_switcher,shot_thumbnail,toggle,segmented,
  traffic_lights,notification_card,banner_stack,menu}*`; sill `crates/sill-surfaces/src/surfaces/
  {bar/status.rs, control_center/**, dock/**}` at `ee9dd76`.
