//! `LevelControl`: a level with a glyph that follows it, in one of three looks (the user's brief
//! of 2026-09-25). The form [`crate::Slider`] stays for settings rows; this is
//! the shell's volume and brightness control and the OSD's level.
//!
//! Motion: while the pointer holds it, the fill follows the pointer with no easing; a level set
//! from outside (a key, the OSD, the service) slides over `--t-quick --e-out`. A press swells the
//! track (`scaleY(1.08)`, `--e-spring`: contact); dragging past either end stretches the capsule
//! a few pixels (the rubber band, off under Reduced) and it springs back on release. Keys step by
//! a sixteenth, Shift by a sixty-fourth. The machine is `machine.rs`; the drawing `look.rs`.

use super::look::{Drawn, body};
use super::machine::{Hold, KeyStep, LevelInput, LevelState, Nudge, Rubber, step};
use crate::components::content::level_glyph::glyph::LevelGlyphView;
use crate::components::content::level_glyph::vocab::{LevelLook, LevelMode, LevelSource, Tick};
use crate::core::geometry::units::{Px, Rect, Size};
use crate::core::vocab::{Availability, Fraction};
use crate::core::word::Word;
use crate::host::measure::client_rect;
use crate::motion::{anim::Anim, pulse::use_pulse};
use crate::style::appearance::motion::MotionLevel;
use crate::style::icon::render::IconSize;
use crate::style::scope::use_scope;
use dioxus::core::queue_effect;
use dioxus::prelude::*;
use std::rc::Rc;

/// A level: the glyph that follows it and the capsule, knob or segments that show it.
/// `onchange` is never called in `LevelMode::ReadOnly`, so a read-only level may leave it out.
/// `Availability::Disabled` draws it plainly unavailable (a disabled level used to look like
/// an enabled one at 0 %): the capsule and glyph at the disabled .35, no knob, the not-allowed
/// cursor, no press, drag, key or swell, and out of the tab order.
/// `glyph` is a `LevelGlyph` that follows `value`, or a `VolumeState` (both convert): then the
/// speaker draws that state's waves and slash, as the bar's volume item does.
#[component]
pub fn LevelControl(
    label: String,
    value: Fraction,
    #[props(into)] glyph: LevelSource,
    #[props(default)] mode: LevelMode,
    #[props(default)] look: LevelLook,
    #[props(default)] tick: Tick,
    #[props(default)] availability: Availability,
    #[props(default)] onchange: EventHandler<Fraction>,
) -> Element {
    let value = value.clamped();
    let (glyph, glyph_level) = glyph.drawn(value);
    let rubber = match use_scope().resolved.motion {
        MotionLevel::Reduced => Rubber::Off,
        MotionLevel::Standard => Rubber::On,
    };
    let mut state = use_signal(|| LevelState::IDLE);
    let mut rail = use_signal(|| None::<Rc<MountedData>>);
    let mut root = use_signal(|| None::<Rc<MountedData>>);
    let pulse = use_pulse(Anim::LevelTick);
    let before = use_before(value);
    if tick == Tick::Quiet
        && super::machine::crossing(before, value) == super::machine::Crossing::Crossed
    {
        queue_effect(move || pulse.fire());
    }
    let now = state();
    let takes_input = mode == LevelMode::Interactive && availability == Availability::Enabled;
    let mut apply = move |input: LevelInput, at: Fraction| {
        let stepped = step(*state.peek(), at, input, rubber);
        state.set(stepped.state);
        if let Some(next) = stepped.value {
            onchange.call(next);
        }
    };
    let drawn = Drawn {
        look,
        value,
        before,
        glyph,
        glyph_level,
        tick: tick_attrs(tick, pulse.attrs()),
    };
    let lead = match look {
        LevelLook::Capsule => None,
        LevelLook::CapsuleKnob | LevelLook::Segments => Some(glyph),
    };
    let (over, stretch) = now.stretch.attrs().unzip();
    let live = match now.hold {
        Hold::Idle => "idle",
        Hold::Pressing { .. } | Hold::Held { .. } => "live",
    };
    // A disabled level is not a stop in the tab order: it takes no key and no press.
    let (role, tabindex) = match (mode, availability) {
        (LevelMode::Interactive, Availability::Enabled) => ("slider", Some("0")),
        (LevelMode::Interactive, Availability::Disabled | Availability::Busy) => ("slider", None),
        (LevelMode::ReadOnly, _) => ("progressbar", None),
    };
    let fill = value.css();
    let rb = stretch
        .map(|by| format!(";--rb:{by:.2}px"))
        .unwrap_or_default();
    rsx! {
        div {
            class: "ds-level",
            "data-look": look.slug(),
            "data-mode": mode.slug(),
            "data-drag": live,
            "data-over": over,
            role,
            tabindex,
            "aria-label": "{label}",
            "aria-valuemin": "0",
            "aria-valuemax": "100",
            "aria-valuenow": "{percent(value)}",
            "aria-disabled": availability.aria_disabled(),
            "aria-busy": availability.aria_busy(),
            style: "--f:{fill}{rb}",
            onmounted: move |event| root.set(Some(event.data())),
            onpointerdown: move |event| {
                if !takes_input {
                    return;
                }
                let x = Px(event.client_coordinates().x as f32);
                apply(LevelInput::Down { x }, value);
                // The press focuses the control (Blitz focuses only text inputs on click, spike
                // S12) and measures the rail after layout, never in `onmounted` (spike S9); the
                // two are separate tasks, so the measurement never waits for the focus.
                if let Some(focus) = root() {
                    spawn(async move {
                        let _ = crate::focus::host::focus_element(&focus).await;
                    });
                }
                if let Some(mounted) = rail() {
                    spawn(async move {
                        if let Some(measured) = client_rect(&mounted).await {
                            apply(LevelInput::Measured { x, track: travel(measured, look) }, value);
                        }
                    });
                }
            },
            onpointermove: move |event| {
                if takes_input {
                    apply(LevelInput::Move { x: Px(event.client_coordinates().x as f32) }, value);
                }
            },
            onpointerup: move |_| apply(LevelInput::Up, value),
            onpointerleave: move |_| apply(LevelInput::Up, value),
            onkeydown: move |event| {
                let fine = event.modifiers().contains(Modifiers::SHIFT);
                if let (true, Some(nudge)) = (takes_input, nudge(&event.key())) {
                    let step = if fine { KeyStep::Fine } else { KeyStep::Coarse };
                    apply(LevelInput::Key { nudge, step }, value);
                }
            },
            if let Some(glyph) = lead {
                span { class: "ds-level-lead",
                    LevelGlyphView { glyph, value: glyph_level, size: IconSize::Bar }
                }
            }
            div {
                class: "ds-level-rail",
                onmounted: move |event| rail.set(Some(event.data())),
                {body(drawn)}
                if takes_input {
                    div { class: "ds-level-hit" }
                }
            }
        }
    }
}

/// The level the control showed before this render, kept across renders: what the tick and the
/// segments' stagger compare the new level with.
fn use_before(value: Fraction) -> Fraction {
    let mut last = use_hook(|| CopyValue::new(value));
    let before = *last.peek();
    last.set(value);
    before
}

/// The tick's class and alias when `Tick::Quiet`.
fn tick_attrs(tick: Tick, attrs: Option<(String, &'static str)>) -> Option<(String, &'static str)> {
    match tick {
        Tick::Off => None,
        Tick::Quiet => Some(attrs.unwrap_or_else(|| (String::new(), "rest"))),
    }
}

/// The span the pointer travels for `look`: the whole rail, except under a knob, whose centre
/// travels from half a knob in at each end.
fn travel(rail: Rect, look: LevelLook) -> Rect {
    match look {
        LevelLook::Capsule | LevelLook::Segments => rail,
        LevelLook::CapsuleKnob => {
            let inset = rail.size.height.0 / 2.0;
            Rect {
                origin: crate::core::geometry::units::Point {
                    x: Px(rail.origin.x.0 + inset),
                    y: rail.origin.y,
                },
                size: Size {
                    width: Px((rail.size.width.0 - 2.0 * inset).max(0.0)),
                    height: rail.size.height,
                },
            }
        }
    }
}

/// The key's nudge: Left and Down lower the level, Right and Up raise it.
fn nudge(key: &Key) -> Option<Nudge> {
    match key {
        Key::ArrowLeft | Key::ArrowDown => Some(Nudge::Down),
        Key::ArrowRight | Key::ArrowUp => Some(Nudge::Up),
        _ => None,
    }
}

/// `aria-valuenow`: the level on the 0..=100 scale the markup declares.
fn percent(value: Fraction) -> u16 {
    (value.clamped().0 + 5) / 10
}
