//! Press: what a Button or IconButton hands its `onclick`: which pointer button activated it,
//! the modifiers held and where it happened. A
//! tray icon's right-click has to reach the app as a secondary press, its middle click as a
//! middle one, and SNI's `ContextMenu(x, y)` and `Activate(x, y)` want the point.

use crate::core::geometry::units::{Point, Px};
use crate::core::press::{PointerButton, Press};
use crate::focus::click::kept_click;
use dioxus::html::input_data::MouseButton;
use dioxus::prelude::*;

/// The press a mouse event describes, as `button`.
pub(crate) fn press_of(event: &MouseEvent, button: PointerButton) -> Press {
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

/// The pointer button a mouse event's trigger names. No trigger is a keyboard activation,
/// which reports primary; the back and forward buttons (and unknown ones) are not presses.
pub fn button_of(trigger: Option<MouseButton>) -> Option<PointerButton> {
    match trigger {
        None | Some(MouseButton::Primary) => Some(PointerButton::Primary),
        Some(MouseButton::Secondary) => Some(PointerButton::Secondary),
        Some(MouseButton::Auxiliary) => Some(PointerButton::Middle),
        Some(MouseButton::Fourth | MouseButton::Fifth | MouseButton::Unknown) => None,
    }
}

/// Whether a press goes on to the control's ancestors after the control has heard it (mailo
/// gaps 5). A button inside a `<summary>` (a collapsible section's header action) must keep its
/// press to itself, or the `<details>` around it toggles as well: the caller only receives a
/// [`Press`], never the event, so it cannot stop the event itself.
#[derive(Debug, Clone, Copy, PartialEq, Eq, Hash, Default)]
pub enum Propagation {
    /// The press reaches the ancestors too, as any click does (the default).
    #[default]
    Bubble,
    /// The press ends at the control: its propagation is stopped before `onclick` runs, and
    /// its default action is prevented, since on Blitz a click's default action walks up the
    /// ancestors (it is what toggles a `<details>` from its `<summary>`). A `type=button` has
    /// no default action of its own, so nothing the control does is lost. The keyboard still
    /// moves to the control, as for any press: the control hands the click to the host's
    /// click focus itself (FINDINGS "Native focus").
    Stop,
}

impl Propagation {
    /// Keep `event` at the control when the press stops there.
    fn apply(self, event: &MouseEvent) {
        match self {
            Propagation::Bubble => {}
            Propagation::Stop => {
                event.stop_propagation();
                event.prevent_default();
            }
        }
    }
}

/// The three listeners a pressable control puts on its element, all reporting through `press`:
/// `click` (primary, and keyboard activation), `contextmenu` (secondary: Blitz and browsers
/// send a right-click as that and never as a click; its default is prevented), and `mouseup`
/// for the middle button, which neither fires as a click on Blitz. Under
/// [`Propagation::Stop`] each of them keeps its event at the control before reporting.
#[derive(Clone, Copy)]
pub struct PressListeners {
    press: EventHandler<Press>,
    propagation: Propagation,
}

impl PressListeners {
    /// Listeners that report to `press` and let the event bubble on.
    pub fn new(press: EventHandler<Press>) -> Self {
        PressListeners {
            press,
            propagation: Propagation::Bubble,
        }
    }

    /// The same listeners, keeping or passing on the event as `propagation` says.
    pub fn with_propagation(self, propagation: Propagation) -> Self {
        PressListeners {
            propagation,
            ..self
        }
    }

    /// A `click`: primary, or whatever button the event names. A click kept at the control
    /// ([`Propagation::Stop`]) is handed to the host's click focus last, as the root would have
    /// (`focus::click::kept_click`): the control has the keyboard afterwards.
    pub fn click(&self, event: &MouseEvent) {
        self.propagation.apply(event);
        if let Some(button) = button_of(event.trigger_button()) {
            self.press.call(press_of(event, button));
        }
        if self.propagation == Propagation::Stop {
            kept_click(event);
        }
    }

    /// A `contextmenu`: a secondary press. The page's own menu is the app's to open.
    pub fn context_menu(&self, event: &MouseEvent) {
        event.prevent_default();
        self.propagation.apply(event);
        self.press.call(press_of(event, PointerButton::Secondary));
    }

    /// A `mouseup`: only the middle button counts (the primary one arrives as `click`).
    pub fn mouse_up(&self, event: &MouseEvent) {
        if event.trigger_button() == Some(MouseButton::Auxiliary) {
            self.propagation.apply(event);
            self.press.call(press_of(event, PointerButton::Middle));
        }
    }
}

#[cfg(test)]
mod tests {
    use super::button_of;
    use crate::core::press::PointerButton;
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
