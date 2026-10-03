//! The event conversions that prove a [`Contact`]: the only public way to get one, so a remote
//! change cannot claim it.

use super::touch::{Contact, Touch};
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

impl Contact {
    /// The contact a handler's event is: call it inside `onclick`, `onpointerdown` or
    /// `onkeydown` and keep it with the state change that event caused. It carries no velocity.
    ///
    /// ```
    /// use dioxus::prelude::{Event, MouseData};
    /// use ds::motion::detail::touch::{Contact, Touch};
    ///
    /// fn touched(event: &Event<MouseData>) -> Touch {
    ///     Touch::Contact(Contact::from_event(event))
    /// }
    /// ```
    ///
    /// (Error codes in these blocks are documentation: stable rustdoc checks only that each fails.)
    ///
    /// ```compile_fail,E0451
    /// // A contact cannot be written by hand: its fields are private.
    /// let forged = ds::motion::detail::touch::Contact { proof: () };
    /// ```
    pub fn from_event<T: Handled>(event: &Event<T>) -> Contact {
        let _ = event;
        Contact::proven()
    }
}

impl Touch {
    /// `Touch::Contact` from a handler's event.
    pub fn from_event<T: Handled>(event: &Event<T>) -> Touch {
        Touch::Contact(Contact::from_event(event))
    }
}
