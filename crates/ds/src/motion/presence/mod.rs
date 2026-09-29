//! Where an item is in its life: hidden, entering, present, or leaving by some exit
//! (design/04-COMPONENTS.md "Motion states", design/05-MOTION.md section 8).

pub(crate) mod exit;

pub use exit::Exit;

/// An item's motion state: `data-presence`.
#[derive(Debug, Clone, Copy, PartialEq, Eq, Hash)]
pub enum Presence {
    /// Not drawn: before its first showing, and after its exit has settled.
    Hidden,
    /// Playing its entrance.
    Entering,
    /// At rest.
    Present,
    /// Playing an exit; dropped when it settles.
    Leaving(Exit),
}

impl Presence {
    /// The `data-presence` value.
    pub fn slug(self) -> &'static str {
        match self {
            Presence::Hidden => "hidden",
            Presence::Entering => "entering",
            Presence::Present => "present",
            Presence::Leaving(_) => "leaving",
        }
    }
}
