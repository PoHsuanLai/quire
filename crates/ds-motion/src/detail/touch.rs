//! Who caused a moment (design/26-DETAILS.md R5): only a person's own contact may spend an
//! overshoot. A [`Contact`] is proof of that contact. With feature `dioxus` the only public way
//! to get one is from the event a pointer or key handler receives (`touch_event`), so a remote
//! change cannot claim it.

use crate::velocity::Velocity;

/// The event payloads a handler may hand to [`Contact::from_event`] and [`Touch::from_event`]; the
/// trait lives with them in `touch_event`, and stays reachable here for callers of this path.
#[cfg(feature = "dioxus")]
pub use super::touch_event::Handled;

/// Proof that the person touched the element in this moment. It has no public constructor but
/// `Contact::from_event` (feature `dioxus`), which takes the event a handler was given, so a
/// contact cannot be written by hand: its fields are private.
///
/// A contact also carries the velocity the hand had when it let go (design/27 section 3.12):
/// zero for a click or a key, the drag's release velocity for a throw
/// ([`Contact::with_velocity`]). A spring moved by it starts at that speed
/// (`ds::motion::spring_spec::SpringSpec::for_touch`).
#[derive(Debug, Clone, Copy, PartialEq, Eq, Hash)]
pub struct Contact {
    proof: (),
    velocity: Velocity,
}

impl Contact {
    /// The contact an event handler proves, carrying no velocity. Visible to the details
    /// module only, where the event conversions that may prove contact live.
    pub(super) fn proven() -> Contact {
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
    /// `onclick` (which hands a [`crate::Press`], not the event): a press reaches it only from the
    /// person's click or key on that button. Crate-private, so a caller outside quire still
    /// proves contact with an event.
    pub fn pressed(press: &ds_core::press::Press) -> Contact {
        let _ = press;
        Contact::proven()
    }

    /// A contact for a unit test that has no event to hand.
    #[cfg(test)]
    pub fn for_tests() -> Contact {
        Contact::proven()
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
    /// The release velocity a contact carries; zero for a remote change.
    pub fn velocity(self) -> Velocity {
        match self {
            Touch::Contact(contact) => contact.velocity(),
            Touch::Remote => Velocity::ZERO,
        }
    }
}
