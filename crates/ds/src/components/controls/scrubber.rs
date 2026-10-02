//! Scrubber: the progress bar of a recording (design/30 section 2.1a): a Slider with the loaded
//! stretches drawn behind the played fill and a tooltip that reads the time under the pointer.
//!
//! A press reports `onscrubstart` at the place, every move while it is held `onscrub`, and the
//! release `onscrubend`; a plain click is a start and an end at one place. The press captures the
//! pointer (`use_pointer_capture`), so a drag that leaves the bar goes on reading positions along
//! it. Escape during a drag is `onscrubcancel`. A key asks for a place with `onseek`: an arrow
//! steps a fiftieth of the length, Shift a tenth, Home and End go to the ends. The caller owns
//! the position: nothing moves until it passes the new one back. The machine is
//! `scrubber_machine.rs`; the drawing `scrubber_face.rs`.

use crate::components::controls::scrubber_face::ScrubberFace;
use crate::components::controls::scrubber_machine::{Jump, Pointer, ScrubInput, ScrubOut, step};
use crate::components::controls::scrubber_model::BufferedRange;
use crate::host::captured::{CapturedPointer, PointerPhase};
use crate::host::measure::client_rect;
use crate::host::pointer_capture::use_pointer_capture;
use crate::root::common::Common;
use dioxus::prelude::*;
use ds_core::geometry::units::{Px, Rect};
use ds_core::vocab::{Availability, Fraction};
use ds_motion::spring::Millis;
use ds_style::tokens::control_size::ControlSize;
use std::rc::Rc;

/// The key's jump: Left and Right step, Shift makes them far, Home and End go to the ends.
fn jump(key: &Key, modifiers: Modifiers) -> Option<Jump> {
    let far = modifiers.contains(Modifiers::SHIFT);
    match key {
        Key::ArrowLeft | Key::ArrowDown => Some(if far { Jump::BackFar } else { Jump::Back }),
        Key::ArrowRight | Key::ArrowUp => Some(if far { Jump::ForwardFar } else { Jump::Forward }),
        Key::Home => Some(Jump::Start),
        Key::End => Some(Jump::End),
        _ => None,
    }
}

/// The pointer's place along the page, in logical pixels.
fn at_x(event: &PointerEvent) -> Px {
    Px(event.client_coordinates().x as f32)
}

/// A recording's progress bar: `position` of the way through `length`, with `buffered` stretches
/// loaded. `Disabled` and `Busy` take no press, drag or key, and `Disabled` leaves the tab order.
#[component]
pub fn Scrubber(
    label: String,
    position: Fraction,
    length: Millis,
    #[props(default)] buffered: Vec<BufferedRange>,
    #[props(default)] size: ControlSize,
    #[props(default)] availability: Availability,
    #[props(default)] onscrubstart: EventHandler<Fraction>,
    #[props(default)] onscrub: EventHandler<Fraction>,
    #[props(default)] onscrubend: EventHandler<Fraction>,
    #[props(default)] onscrubcancel: EventHandler<()>,
    #[props(default)] onseek: EventHandler<Fraction>,
    #[props(default)] common: Common,
) -> Element {
    let mut pointer = use_signal(Pointer::default);
    let mut track = use_signal(|| None::<Rect>);
    let mut root = use_signal(|| None::<Rc<MountedData>>);
    let mut current = use_hook(|| CopyValue::new(position));
    current.set(position);
    let takes_input = availability == Availability::Enabled;
    let apply = use_callback(move |input: ScrubInput| {
        let (next, out) = step(*pointer.peek(), *current.peek(), input);
        pointer.set(next);
        match out {
            Some(ScrubOut::Start(at)) => onscrubstart.call(at),
            Some(ScrubOut::Scrub(at)) => onscrub.call(at),
            Some(ScrubOut::End(at)) => onscrubend.call(at),
            Some(ScrubOut::Cancel) => onscrubcancel.call(()),
            Some(ScrubOut::Seek(at)) => onseek.call(at),
            None => {}
        }
    });
    let hold = use_pointer_capture(move |captured: CapturedPointer| {
        let x = Px(captured.at.x.0);
        match captured.phase {
            PointerPhase::Drag => apply.call(ScrubInput::Dragged { x }),
            PointerPhase::Release => apply.call(ScrubInput::Up { x }),
            PointerPhase::Press => {}
        }
    });
    let (pose, at) = pointer().pose();
    rsx! {
        ScrubberFace {
            label,
            position,
            length,
            buffered,
            pose,
            pointer: at,
            size,
            availability,
            common,
            onmounted: move |event: MountedEvent| {
                hold.on_mounted(event.clone());
                root.set(Some(event.data()));
            },
            onpointerenter: move |event: PointerEvent| {
                if !takes_input {
                    return;
                }
                let x = at_x(&event);
                if let Some(mounted) = root() {
                    spawn(async move {
                        if let Some(measured) = client_rect(&mounted).await {
                            track.set(Some(measured));
                            apply.call(ScrubInput::Moved { track: measured, x });
                        }
                    });
                }
            },
            onpointermove: move |event: PointerEvent| {
                if let (true, Some(measured)) = (takes_input, track()) {
                    apply.call(ScrubInput::Moved { track: measured, x: at_x(&event) });
                }
            },
            onpointerdown: move |event: PointerEvent| {
                if !takes_input {
                    return;
                }
                let x = at_x(&event);
                let _held = hold.begin();
                if let Some(mounted) = root() {
                    let focused = Rc::clone(&mounted);
                    spawn(async move {
                        // Focus is best-effort: Blitz focuses only text inputs on click (spike S12).
                        let _ = crate::focus::soon::focus_element(&focused).await;
                    });
                    spawn(async move {
                        if let Some(measured) = client_rect(&mounted).await {
                            track.set(Some(measured));
                            apply.call(ScrubInput::Down { track: measured, x });
                        }
                    });
                }
            },
            onpointerup: move |event: PointerEvent| apply.call(ScrubInput::Up { x: at_x(&event) }),
            onpointerleave: move |_| apply.call(ScrubInput::Left),
            onpointercancel: move |_| apply.call(ScrubInput::Cancel),
            onkeydown: move |event: KeyboardEvent| {
                if !takes_input {
                    return;
                }
                if event.key() == Key::Escape {
                    apply.call(ScrubInput::Cancel);
                } else if let Some(jump) = jump(&event.key(), event.modifiers()) {
                    apply.call(ScrubInput::Key(jump));
                }
            },
        }
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn keys_jump_and_shift_makes_it_far() {
        // (key, shift, jump)
        let cases: Vec<(Key, Modifiers, Option<Jump>)> = vec![
            (Key::ArrowLeft, Modifiers::empty(), Some(Jump::Back)),
            (Key::ArrowRight, Modifiers::empty(), Some(Jump::Forward)),
            (Key::ArrowLeft, Modifiers::SHIFT, Some(Jump::BackFar)),
            (Key::ArrowRight, Modifiers::SHIFT, Some(Jump::ForwardFar)),
            (Key::Home, Modifiers::empty(), Some(Jump::Start)),
            (Key::End, Modifiers::empty(), Some(Jump::End)),
            (Key::Enter, Modifiers::empty(), None),
        ];
        for (key, modifiers, want) in cases {
            assert_eq!(jump(&key, modifiers), want, "{key:?}");
        }
    }
}
