//! Who caused a moment (design/26-DETAILS.md R5): only a person's own contact may spend an
//! overshoot. A [`Contact`] is proof of that contact, and the only public way to get one is from
//! the event a pointer or key handler receives, so a remote change cannot claim it.

use crate::motion::velocity::Velocity;
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
/// // A contact cannot be written by hand: its fields are private.
/// let forged = ds::detail::Contact { proof: () };
/// ```
///
/// A contact also carries the velocity the hand had when it let go (design/27 section 3.12):
/// zero for a click or a key, the drag's release velocity for a throw
/// ([`Contact::with_velocity`]). A spring moved by it starts at that speed
/// (`ds::motion::SpringSpec::for_touch`).
#[derive(Debug, Clone, Copy, PartialEq, Eq, Hash)]
pub struct Contact {
    proof: (),
    velocity: Velocity,
}

impl Contact {
    /// The contact a handler's event is: call it inside `onclick`, `onpointerdown` or
    /// `onkeydown` and keep it with the state change that event caused. It carries no velocity.
    pub fn from_event<T: Handled>(event: &Event<T>) -> Contact {
        let _ = event;
        Contact {
            proof: (),
            velocity: Velocity::ZERO,
        }
    }

    /// The same contact, released at `velocity`: what a drag's `onpointerup` hands on, from its
    /// own measure of the pointer's speed.
    pub fn with_velocity(self, velocity: Velocity) -> Contact {
        Contact { velocity, ..self }
    }

    /// The velocity the hand had when it let go (zero for a tap or a key).
    pub fn velocity(self) -> Velocity {
        self.velocity
    }

    /// The contact a quire button's press is, for a part of quire's own that hears a `Button`'s
    /// `onclick` (which hands a [`crate::components::press::Press`], not the event): a press reaches it only from the
    /// person's click or key on that button. Crate-private, so a caller outside quire still
    /// proves contact with an event.
    pub(crate) fn pressed(press: &crate::components::press::Press) -> Contact {
        let _ = press;
        Contact {
            proof: (),
            velocity: Velocity::ZERO,
        }
    }

    /// A contact for a unit test that has no event to hand.
    #[cfg(test)]
    pub(crate) fn for_tests() -> Contact {
        Contact {
            proof: (),
            velocity: Velocity::ZERO,
        }
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

    /// The release velocity a contact carries; zero for a remote change.
    pub fn velocity(self) -> Velocity {
        match self {
            Touch::Contact(contact) => contact.velocity(),
            Touch::Remote => Velocity::ZERO,
        }
    }
}
