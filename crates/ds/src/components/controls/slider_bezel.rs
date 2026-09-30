//! The capsule sliders (`SliderLook::Capsule` and `SliderLook::CapsuleKnob`): a level with a
//! glyph that follows it, the control center's volume and brightness (the user's brief of
//! 2026-09-25; design/30 section 2.1).
//!
//! Motion: while the pointer holds it, the fill follows the pointer with no easing; a level set
//! from outside (a key, the service) slides over `--t-quick --e-out`. Keys step by a sixteenth,
//! Shift by a sixty-fourth. The machine is `slider_machine.rs`; the drawing `level_draw.rs`.

use crate::components::content::level_glyph::vocab::LevelSource;
use crate::components::controls::level_draw::{Drawn, GlyphAt, SLIDER, Shape, body, lead};
use crate::components::controls::slider_machine::{Hold, KeyStep, LevelInput, Nudge, step};
use crate::components::controls::slider_model::SliderLook;
use crate::host::measure::client_rect;
use crate::root::common::Common;
use dioxus::prelude::*;
use ds_core::geometry::units::{Point, Px, Rect, Size};
use ds_core::vocab::{Availability, Fraction};
use ds_core::word::Word;
use ds_style::tokens::control_size::ControlSize;
use std::rc::Rc;

/// The shape a capsule look draws.
fn shape_of(look: SliderLook) -> Shape {
    match look {
        SliderLook::Linear | SliderLook::Capsule => Shape::Capsule,
        SliderLook::CapsuleKnob => Shape::CapsuleKnob,
    }
}

/// A level: the glyph that follows it and the capsule that shows it.
/// `Availability::Disabled` draws it plainly unavailable: the capsule and glyph at the disabled
/// .35, no knob, the not-allowed cursor, no press, drag or key, and out of the tab order.
#[component]
pub(crate) fn BezelSlider(
    label: String,
    value: Fraction,
    look: SliderLook,
    glyph: Option<LevelSource>,
    size: ControlSize,
    availability: Availability,
    onchange: EventHandler<Fraction>,
    common: Common,
) -> Element {
    let value = value.clamped();
    let mut state = use_signal(|| Hold::Idle);
    let mut rail = use_signal(|| None::<Rc<MountedData>>);
    let mut root = use_signal(|| None::<Rc<MountedData>>);
    let takes_input = availability == Availability::Enabled;
    let mut apply = move |input: LevelInput, at: Fraction| {
        let stepped = step(*state.peek(), at, input);
        state.set(stepped.state);
        if let Some(next) = stepped.value {
            onchange.call(next);
        }
    };
    let shape = shape_of(look);
    let drawn = Drawn {
        shape,
        parts: SLIDER,
        value,
        glyph: glyph.map(|source| {
            let (glyph, level) = source.drawn(value);
            GlyphAt { glyph, level }
        }),
    };
    let leading = lead(&drawn);
    let pressed = state().phase().attr();
    let fill = value.css();
    let class = common.class("ds-slider");
    let data = common.data_attributes();
    rsx! {
        div {
            id: common.id.clone(),
            class,
            "data-look": look.slug(),
            "data-size": size.slug(),
            "data-availability": availability.slug(),
            "data-pressed": pressed,
            role: "slider",
            // A disabled level is not a stop in the tab order: it takes no key and no press.
            tabindex: if takes_input { Some("0") } else { None },
            "aria-label": common.aria_label.clone().unwrap_or_else(|| label.clone()),
            "aria-valuemin": "0",
            "aria-valuemax": "100",
            "aria-valuenow": "{percent(value)}",
            "aria-disabled": availability.aria_disabled(),
            "aria-busy": availability.aria_busy(),
            style: "--f:{fill}",
            onmounted: move |event| {
                root.set(Some(event.data()));
                common.mounted(event);
            },
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
                        let _ = crate::focus::soon::focus_element(&focus).await;
                    });
                }
                if let Some(mounted) = rail() {
                    spawn(async move {
                        if let Some(measured) = client_rect(&mounted).await {
                            apply(LevelInput::Measured { x, track: travel(measured, shape) }, value);
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
            ..data,
            {leading}
            div {
                class: "ds-slider-rail",
                onmounted: move |event| rail.set(Some(event.data())),
                {body(drawn)}
                if takes_input {
                    div { class: "ds-slider-hit" }
                }
            }
        }
    }
}

/// The span the pointer travels for `shape`: the whole rail, except under a knob, whose centre
/// travels from half a knob in at each end.
fn travel(rail: Rect, shape: Shape) -> Rect {
    match shape {
        Shape::Capsule | Shape::Segments => rail,
        Shape::CapsuleKnob => {
            let inset = rail.size.height.0 / 2.0;
            Rect {
                origin: Point {
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
    value.whole_percent()
}
