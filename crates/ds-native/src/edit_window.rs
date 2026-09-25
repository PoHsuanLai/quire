//! The window's side of an edit surface's pointer capture (FINDINGS "Edit surface 2"): the
//! window hook hears every winit pointer event before the document, wherever the pointer is,
//! so a move or the primary release goes to the surface a press captured it for even outside
//! the surface's box. Blitz dispatches a move only to the element under the pointer.

use dioxus_native::winit::event::{ButtonSource, ElementState, MouseButton, WindowEvent};
use dioxus_native::winit::keyboard::ModifiersState;
use ds::{CapturedPointer, Point, PointerPhase, Px};
use keyboard_types::Modifiers;

/// The capture event a winit event makes, at display scale `scale`, with `held` modifiers.
pub(crate) fn captured_of(
    event: &WindowEvent,
    scale: f64,
    held: Modifiers,
) -> Option<CapturedPointer> {
    let (phase, position) = match event {
        WindowEvent::PointerMoved { position, .. } => (PointerPhase::Drag, position),
        WindowEvent::PointerButton {
            state: ElementState::Released,
            position,
            button,
            ..
        } if primary(button) => (PointerPhase::Release, position),
        _ => return None,
    };
    let scale = scale.max(f64::EPSILON);
    Some(CapturedPointer {
        phase,
        at: Point {
            x: Px((position.x / scale) as f32),
            y: Px((position.y / scale) as f32),
        },
        modifiers: held,
    })
}

/// The primary button, as a mouse, a finger or a pen reports it.
fn primary(button: &ButtonSource) -> bool {
    button.clone().mouse_button() == Some(MouseButton::Left)
}

/// winit's modifier state as the keyboard vocabulary's.
pub(crate) fn modifiers_of(state: ModifiersState) -> Modifiers {
    [
        (state.shift_key(), Modifiers::SHIFT),
        (state.control_key(), Modifiers::CONTROL),
        (state.alt_key(), Modifiers::ALT),
        (state.meta_key(), Modifiers::META),
    ]
    .into_iter()
    .filter(|(down, _)| *down)
    .fold(Modifiers::empty(), |all, (_, flag)| all | flag)
}
