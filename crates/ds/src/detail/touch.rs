//! Who caused a moment (design/26-DETAILS.md R5): only a person's own contact may spend an
//! overshoot. A [`Contact`] is proof of that contact, and the only public way to get one is from
//! the event a pointer or key handler receives, so a remote change cannot claim it.

use dioxus::prelude::{Event, KeyboardData, MouseData, PointerData};

mod sealed {
    /// Closes [`super::Handled`] to the event payloads dioxus hands a handler.
    pub trait Sealed {}
    impl Sealed for dioxus::prelude::MouseData {}
    impl Sealed for dioxus::prelude::KeyboardData {}
    impl Sealed for dioxus::prelude::PointerData {}
}

/// An event payload a person's own hand produces: a click, a pointer press, a key.
pub trait Handled: sealed::Sealed {}
impl Handled for MouseData {}
impl Handled for KeyboardData {}
impl Handled for PointerData {}

/// Proof that the person touched the element in this moment. It has no public constructor but
/// [`Contact::from_event`], which takes the event a handler was given:
///
/// ```
/// use dioxus::prelude::{Event, MouseData};
/// use ds::detail::{Contact, Touch};
///
/// fn touched(event: &Event<MouseData>) -> Touch {
///     Touch::Contact(Contact::from_event(event))
/// }
/// ```
///
/// (Error codes in these blocks are documentation: stable rustdoc checks only that each fails.)
///
/// ```compile_fail,E0451
/// // A contact cannot be written by hand: its field is private.
/// let forged = ds::detail::Contact { proof: () };
/// ```
#[derive(Debug, Clone, Copy, PartialEq, Eq, Hash)]
pub struct Contact {
    proof: (),
}

impl Contact {
    /// The contact a handler's event is: call it inside `onclick`, `onpointerdown` or
    /// `onkeydown` and keep it with the state change that event caused.
    pub fn from_event<T: Handled>(event: &Event<T>) -> Contact {
        let _ = event;
        Contact { proof: () }
    }
}

/// Who caused a moment: the person's own contact, or anything else (a service, a timer, another
/// window). Only `Contact` may spend an overshoot (R5).
#[derive(Debug, Clone, Copy, PartialEq, Eq, Hash, Default)]
pub enum Touch {
    /// The person touched this element; carries the proof.
    Contact(Contact),
    /// The change came from elsewhere.
    #[default]
    Remote,
}

impl Touch {
    /// `Touch::Contact` from a handler's event.
    pub fn from_event<T: Handled>(event: &Event<T>) -> Touch {
        Touch::Contact(Contact::from_event(event))
    }
}
