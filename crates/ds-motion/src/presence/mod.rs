//! Where an item is in its life: hidden, entering, present, or leaving by some exit
//! (design/04-COMPONENTS.md "Motion states", design/05-MOTION.md section 8). The rows of a
//! roster carry one; a surface its caller shows and hides gets one from [`use_presence`].

pub(crate) mod exit;
pub mod spec;
#[cfg(feature = "dioxus")]
pub mod spring;
#[cfg(feature = "dioxus")]
pub(crate) mod step;
#[cfg(feature = "dioxus")]
pub mod use_presence;

pub use exit::Exit;

use ds_core::vocab::Shown;

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

    /// The `data-presence` value of something drawn, or `None` while it is hidden.
    pub fn drawn_slug(self) -> Option<&'static str> {
        match self {
            Presence::Hidden => None,
            Presence::Entering | Presence::Present | Presence::Leaving(_) => Some(self.slug()),
        }
    }

    /// The `data-shown` value: whether anything is drawn.
    pub fn shown(self) -> Shown {
        match self {
            Presence::Hidden => Shown::Hidden,
            Presence::Entering | Presence::Present | Presence::Leaving(_) => Shown::Visible,
        }
    }
}
