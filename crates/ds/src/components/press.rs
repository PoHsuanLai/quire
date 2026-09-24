//! Press: what a Button or IconButton hands its `onclick`: which pointer button activated it,
//! the modifiers held and where it happened (sill FINDINGS F32, quire gaps Q8 and bar gaps). A
//! tray icon's right-click has to reach the app as a secondary press, its middle click as a
//! middle one, and SNI's `ContextMenu(x, y)` and `Activate(x, y)` want the point.

use crate::geometry::{Point, Px};
use dioxus::core::SuperFrom;
use dioxus::html::input_data::MouseButton;
use dioxus::prelude::*;

/// Which button pressed.
#[derive(Debug, Clone, Copy, PartialEq, Eq, Hash)]
pub enum PointerButton {
    /// The primary button, or the keyboard (Enter or Space on the focused control).
    Primary,
    /// The secondary button: a right-click, which the page reads as a context-menu request.
    Secondary,
    /// The middle button.
    Middle,
}

/// One activation of a control. `PartialEq` without `Eq`: the point is fractional pixels.
#[derive(Debug, Clone, Copy, PartialEq)]
pub struct Press {
    /// Which button.
    pub button: PointerButton,
    /// The modifiers held when it happened.
    pub modifiers: Modifiers,
    /// Where, in the surface's own coordinates (the event's client point: on a shell surface,
    /// surface-local logical pixels). A keyboard activation reports the point its event
    /// carries, which on Blitz is the origin.
    pub at: Point,
}

impl Press {
    /// A primary press with no modifiers at the origin: what a keyboard activation or a test
    /// reports.
    pub fn primary() -> Self {
        Press {
            button: PointerButton::Primary,
            modifiers: Modifiers::empty(),
            at: Point::default(),
        }
    }

    /// The press a mouse event describes, as `button`.
    pub(crate) fn of(event: &MouseEvent, button: PointerButton) -> Self {
        let at = event.client_coordinates();
        Press {
            button,
            modifiers: event.modifiers(),
            at: Point {
                x: Px(at.x as f32),
                y: Px(at.y as f32),
            },
        }
    }
}

/// The pointer button a mouse event's trigger names. No trigger is a keyboard activation,
/// which reports primary; the back and forward buttons (and unknown ones) are not presses.
pub(crate) fn button_of(trigger: Option<MouseButton>) -> Option<PointerButton> {
    match trigger {
        None | Some(MouseButton::Primary) => Some(PointerButton::Primary),
        Some(MouseButton::Secondary) => Some(PointerButton::Secondary),
        Some(MouseButton::Auxiliary) => Some(PointerButton::Middle),
        Some(MouseButton::Fourth | MouseButton::Fifth | MouseButton::Unknown) => None,
    }
}

/// The three listeners a pressable control puts on its element, all reporting through `press`:
/// `click` (primary, and keyboard activation), `contextmenu` (secondary: Blitz and browsers
/// send a right-click as that and never as a click; its default is prevented), and `mouseup`
/// for the middle button, which neither fires as a click on Blitz.
#[derive(Clone, Copy)]
pub(crate) struct PressListeners {
    press: EventHandler<Press>,
}

impl PressListeners {
    /// Listeners that report to `press`.
    pub(crate) fn new(press: EventHandler<Press>) -> Self {
        PressListeners { press }
    }

    /// A `click`: primary, or whatever button the event names.
    pub(crate) fn click(&self, event: &MouseEvent) {
        if let Some(button) = button_of(event.trigger_button()) {
            self.press.call(Press::of(event, button));
        }
    }

    /// A `contextmenu`: a secondary press. The page's own menu is the app's to open.
    pub(crate) fn context_menu(&self, event: &MouseEvent) {
        event.prevent_default();
        self.press.call(Press::of(event, PointerButton::Secondary));
    }

    /// A `mouseup`: only the middle button counts (the primary one arrives as `click`).
    pub(crate) fn mouse_up(&self, event: &MouseEvent) {
        if event.trigger_button() == Some(MouseButton::Auxiliary) {
            self.press.call(Press::of(event, PointerButton::Middle));
        }
    }
}

/// Marks the conversion below, so it does not collide with dioxus's own.
#[doc(hidden)]
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub struct IgnorePress;

/// The compatibility path: an `EventHandler<()>` the caller already holds (a prop passed
/// through, a stored handler) still works as an `onclick`, ignoring which button pressed. A
/// closure written `move |_| …` needs nothing: it takes the `Press` and drops it.
impl SuperFrom<EventHandler<()>, IgnorePress> for EventHandler<Press> {
    fn super_from(handler: EventHandler<()>) -> Self {
        EventHandler::new(move |_: Press| handler.call(()))
    }
}

#[cfg(test)]
mod tests {
    use super::{PointerButton, button_of};
    use dioxus::html::input_data::MouseButton;

    #[test]
    fn a_trigger_names_its_button() {
        const CASES: &[(Option<MouseButton>, Option<PointerButton>)] = &[
            (None, Some(PointerButton::Primary)),
            (Some(MouseButton::Primary), Some(PointerButton::Primary)),
            (Some(MouseButton::Secondary), Some(PointerButton::Secondary)),
            (Some(MouseButton::Auxiliary), Some(PointerButton::Middle)),
            (Some(MouseButton::Fourth), None),
            (Some(MouseButton::Fifth), None),
            (Some(MouseButton::Unknown), None),
        ];
        for (trigger, want) in CASES {
            assert_eq!(button_of(*trigger), *want, "{trigger:?}");
        }
    }
}
