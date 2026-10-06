# 35 - Symbol effects

An icon's own motion, modelled on SF Symbols' symbol effects (WWDC23 "Animate symbols in your
app": Bounce, Pulse, Variable Color, Scale, Appear and Disappear, Replace; macOS 15: Rotate,
Wiggle, Breathe; macOS 26: Draw On and Draw Off). The geometry is Lucide's (ISC); the motion is
ours.

## 1. Rules

- **Opt-in.** No component plays a symbol effect it was not asked for. design/30 section 1.3
  deleted the default emphasis motions (bump, gulp, pulse, nudge); a symbol effect is the caller
  saying "this icon is the signal". `Button`'s busy turn is the one built-in use.
- **Brief and purposeful** (design/34 section 7): a `Once` effect lasts `--t-big` (400 ms) at
  most; a loop's period is `--t-turn` (1 s). A bounce is a response to something that happened
  (a trigger), never decoration.
- **Reduced motion is central.** A whole-icon effect has a `ReducedForm` in `ds-motion/src/reduced.rs`:
  what moves holds (`hold`) or fades; what only dims (`Pulse`) plays as it is. A part effect
  plays nothing under Reduced (the pose stays at rest), and `Appear`, `Disappear`, `Draw On` and
  `Replace` snap or cross-fade. No state is carried by motion alone.
- **One home.** `ds-motion::symbol` owns the vocabulary, the routing and the drivers; the keyframes
  are in `motion.css`, one `Anim` per whole-icon motion and one `recipe_symbol.rs` row each. Part
  annotations are data beside the geometry: `ds-style::icon::parts`.

## 2. The vocabulary

```rust
Symbol { icon, size, effect: SymbolEffect::Once(OnceEffect::Bounce, trigger) }
Symbol { icon, size, effect: SymbolEffect::While(LoopEffect::Rotate, Activity::Active) }
Symbol { icon, size, effect: SymbolEffect::Transition(TransitionEffect::Appear, Shown::Visible) }
```

A run mode is part of the value, so an effect cannot be given a mode it does not have.

| Mode | Fires | Effects |
| --- | --- | --- |
| `Once(effect, Trigger)` | each time the trigger value changes; never on the first render, never on a re-render with the same value | Bounce, Pulse, Wiggle, Breathe, Rotate, VariableColor, Part |
| `While(effect, Activity)` | for as long as the activity is `Active`; stopping plays `hold` so the engine drops the last frame | Bounce, Pulse, ScaleUp, ScaleDown, Wiggle, Breathe, Rotate, VariableColor, Part |
| `Transition(effect, Shown)` | when the visibility (or, for Replace, the icon) changes; the first render stands at the given state | Appear, Disappear, DrawOn (and Draw Off, its `Hidden`), Replace |

`Part` plays the icon's own annotated gesture and `VariableColor` lights its layers in order; on an
icon without them they fall back to Bounce and Pulse (`route`).

### 2.1 Whole-icon effects (the stylesheet)

Played on the `span.ds-symbol` wrapper by an `a-<anim>` class, restarted by the `data-pulse`
alias swap (design/05 section 9 rule 2). Blitz's stylesheet does not reach inside an SVG, so only
what moves the whole icon is CSS.

| Effect | `Anim` | Keyframes | Duration, easing | Reduced |
| --- | --- | --- | --- | --- |
| Bounce | `Bounce`, `BounceLoop` | `symbol-bounce` | `--t-big` `--e-out` | hold |
| Pulse | `Pulse`; `PulseLoop` | `symbol-pulse` | `--t-big` `--e-in-out`; loop `--t-turn` | as is (opacity only) |
| Wiggle | `Wiggle`; `WiggleLoop` | `symbol-wiggle` | `--t-big` `--e-out`; loop `--t-turn` `--e-in-out` | hold |
| Breathe | `Breathe`; `BreatheLoop` | `symbol-breathe` | `--t-big` `--e-in-out`; loop `--t-turn` | hold |
| Rotate | `RotateOnce`; `Turn` (the constant-speed loop, existing) | `turn` | `--t-big` `--e-in-out`; loop `--t-turn` `--e-linear` | hold |
| Scale up, down | `ScaleUp`, `ScaleDown` | `symbol-scale-up`, `-down` | `--t-move` `--e-out`, held (`forwards`) | hold |
| Appear | `Appear` | `morph-in` (shared with MorphGlyph) | `--t-move` `--e-out` | fade in |
| Disappear | `Disappear` | `morph-out` (shared) | `--t-quick` `--e-exit`, held | fade out |

A loop is the same keyframes at `Iteration::Infinite`; the details lint's loop rule is excepted
for the motion sheet with that reason (`ds-shell/tests/details_lint.rs`), and Reduced runs every
animation once (`css.rs`).

### 2.2 Part effects and Draw On (Rust)

What moves a piece of the SVG is written as attributes frame by frame, like MorphGlyph's slash:
`PosedGlyph` (`ds-style::icon::posed`) draws each annotated part in a `g` carrying its pose
(`transform` about the part's pivot, `opacity`, `fill`). The pose is `symbol::pose`, pure
arithmetic over stops that a table pins; the clock is `timeline::cycle::Cycle` under
`use_playback`, which asks for frames only while a cycle runs. Durations are the same tokens:
a `Once` cycle is `--t-big`, a loop's period `--t-turn` (`symbol::timing`). A Rust-driven motion
has no keyframes, so it has a row in `timing.rs` instead of `recipe_symbol.rs`.

Draw On is `use_tween` over the strokes' dash (`stroke-dasharray`, `stroke-dashoffset`), at
`--t-move` `--e-out`; the dash length is `ds-style::icon::length`, a bound that is never shorter
than the stroke (control polygon of a Bezier, an arc's own radius and chord). A fully hidden
stroke is `opacity` 0, since a round cap would leave a dot.

## 3. Part annotations

`Icon::parts()` (`ds-style/src/icon/parts.rs`): which of the icon's Lucide shapes form the moving
part, and the pivot (tenths of a grid unit). The test checks every named shape exists and no shape
is in two parts. Layers are listed in the order they step.

| Icon | Gesture | Part (shape indices) | Pivot |
| --- | --- | --- | --- |
| Trash | Lift | lid: rule and handle (0, 2) | left end of the rule (5, 6) |
| Bell | Ring | whole bell (0, 1), swings | top (12, 3) |
| Mail | Flap | the V (1) flips to half open height | the top edge (12, 7) |
| Folder | Open | outline (0) leans open | bottom-left corner (2, 20) |
| Lock | Lift | shackle (1) | right foot (17, 11) |
| Star, Heart | Pop | outline (0) swells and flashes its fill | centre |
| Refresh | Turn | both arrows (0-3) turn once | centre |
| Wifi | Layers | dot, small, middle, large arc (0, 3, 2, 1) | none |
| Volume2 | Layers | small wave, large wave (1, 2); the speaker stays | none |
| BatteryFull | Layers | cells left to right (3, 0, 1) | none |

Layers (Variable Color): all layers dim for the first tenth of the cycle, then each comes up in
its turn and stays; the cycle ends with every layer lit, which is rest.

`Icon::Heart` (Lucide `heart`) was added for this.

## 4. Credits

The motion values are quire's own. The shapes of the gestures (a lid that tips about its hinge, a
bell that swings from its top and settles, a flap that opens about its edge, a shackle that
lifts) follow the idea of pqoqubbw's animated Lucide icons (lucide-animated, MIT); no code or
timing table is copied, and the durations are the tokens above. Lucide's drawings are ISC
(`crates/ds-style/assets/icons/LICENSE-lucide.txt`).

## 5. Tests

- `ds-motion`: `symbol::attrs` (when each mode fires, as a table), `symbol::route`, `symbol::pose`
  (rest at both ends, the stops, layers in order), `timeline::cycle`, `css::tests` (Reduced forms and
  loops in the generated sheet), `reduced::tests` (every moving keyframe has a still form).
- `ds-style`: `icon::parts` (every part names existing shapes), `icon::posed` (groups, pivots,
  dashes), `icon::length`.
- `ds/tests/motion_drift.rs`: the CSS and the recipe table agree. `ds/tests/symbol_ssr.rs`: markup
  per effect. `ds/tests/button_busy_look_ssr.rs`: the Button wrapper's markup contract.
- `ds-conformance/tests/symbol_effects.rs`: `Clock::Virtual`; Once fires on change and not on
  re-render, While runs and stops, Reduced plays no part motion, each ends at 0 frames.
- Gallery page `symbols`: every effect, a button per Once and a toggle per While.
