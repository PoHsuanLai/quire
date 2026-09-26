//! A wake counter: the caller's way to say "play your entrance again" to a component that
//! otherwise moves only on mount and on a change of its own value (a persona's awake window,
//! design/24-PERSONA.md; a battery ring's fill, design/23-WIDGETS.md section 4.1).

/// A wake counter: pass a different value (`WakeStamp::next`) to wake the component, as a lock
/// screen does when the pointer moves or a key is pressed, or a widget host does when its
/// widgets come into view. Mounting wakes it too.
#[derive(Debug, Clone, Copy, PartialEq, Eq, Hash, Default)]
pub struct WakeStamp(pub u32);

impl WakeStamp {
    /// The next stamp.
    pub fn next(self) -> WakeStamp {
        WakeStamp(self.0.wrapping_add(1))
    }
}
