//! Whether a pressed control takes the keyboard: `FocusOnPress`, and the scope that sets it for
//! every control inside.
//!
//! On macOS a control in a formatting bar (an `NSSegmentedControl` that refuses first
//! responder, the Pages and TextEdit format bars) acts on a press and leaves the keyboard, the
//! caret and the selection in the text view. A browser button takes the keyboard; Blitz clears it
//! on any click nothing took. [`FocusOnPressScope`] puts [`FocusOnPress::Refuses`] around a bar,
//! and each quire control inside (`Button`, `IconButton`, `SegmentedControl`) then prevents its
//! click's default, so Blitz leaves the focus where it was, and hands the click to no host: the
//! text body keeps the keyboard, and a later command-I still reaches it. The scope is a context,
//! not a prop on each control, so one wrapper covers every control in the bar.

use dioxus::prelude::*;
use ds_core::word::Word;

/// Whether pressing a control moves the keyboard to it.
#[derive(Debug, Clone, Copy, PartialEq, Eq, Hash, Default, Word)]
pub enum FocusOnPress {
    /// The control takes the keyboard, as a button does (the default).
    #[default]
    Takes,
    /// The control acts on the press and leaves the keyboard where it was.
    Refuses,
}

/// Put `focus` around every control inside.
#[component]
pub fn FocusOnPressScope(focus: FocusOnPress, children: Element) -> Element {
    use_context_provider(|| focus);
    rsx! { {children} }
}

/// What the nearest [`FocusOnPressScope`] says, or [`FocusOnPress::Takes`] outside one.
pub(crate) fn focus_on_press() -> FocusOnPress {
    try_consume_context::<FocusOnPress>().unwrap_or_default()
}

/// A control's click: under a scope that refuses, keep the renderer from moving the keyboard.
/// Returns what the control then does about it.
pub(crate) fn on_click(event: &MouseEvent) -> FocusOnPress {
    let focus = focus_on_press();
    if focus == FocusOnPress::Refuses {
        event.prevent_default();
    }
    focus
}
