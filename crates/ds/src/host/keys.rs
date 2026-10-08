//! What a renderer knows about a key press that Dioxus's keyboard event does not carry.

use dioxus::prelude::{Key, KeyboardData};

/// The text a key press types and the key it is without modifiers, as the platform reports them.
/// Each is `None` where the platform gives no answer (a renderer that does not forward them, a
/// key that types nothing).
#[derive(Debug, Clone, PartialEq, Eq, Default)]
pub struct KeyExtras {
    /// The text the press types, with the layout's shift, dead keys and compose applied.
    pub text: Option<String>,
    /// The logical key the physical key makes with no modifier held: `1` for Shift+1, `a` for
    /// Ctrl+A.
    pub unshifted: Option<Key>,
}

/// A renderer that can say more about a key press than the event does.
pub trait KeysHost {
    /// What the platform reported alongside `event`.
    fn extras(&self, event: &KeyboardData) -> KeyExtras;
}
