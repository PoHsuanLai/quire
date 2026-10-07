//! The answer: each capability, here or absent.

use crate::capability::Capability;
use std::collections::BTreeMap;

/// Whether a capability answers.
#[derive(Debug, Clone, Copy, PartialEq, Eq, Default)]
pub enum Presence {
    /// The service owns its name, or the bus can activate it on demand.
    Here,
    /// Nothing answers.
    #[default]
    Absent,
}

/// Every capability and whether it is here. The default is everything `Absent`.
#[derive(Debug, Clone, PartialEq, Eq, Default)]
pub struct Desktop {
    presence: BTreeMap<Capability, Presence>,
}

impl Desktop {
    /// How `capability` is now.
    pub fn get(&self, capability: Capability) -> Presence {
        self.presence
            .get(&capability)
            .copied()
            .unwrap_or(Presence::Absent)
    }

    /// Whether `capability` is here; the gate an extra's entry point renders under.
    pub fn here(&self, capability: Capability) -> bool {
        self.get(capability) == Presence::Here
    }

    #[cfg(any(feature = "dbus", test))]
    pub(crate) fn set(&mut self, capability: Capability, presence: Presence) {
        self.presence.insert(capability, presence);
    }
}

#[cfg(test)]
mod tests {
    use super::{Capability, Desktop, Presence};

    #[test]
    fn a_new_desktop_has_nothing() {
        let desktop = Desktop::default();
        assert!(Capability::ALL.iter().all(|&c| !desktop.here(c)));
    }

    #[test]
    fn set_then_get() {
        let mut desktop = Desktop::default();
        desktop.set(Capability::Memory, Presence::Here);
        assert!(desktop.here(Capability::Memory));
        assert_eq!(desktop.get(Capability::Intents), Presence::Absent);
    }
}
