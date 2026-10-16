//! How long a call to a peer may take, by kind of wait.

use std::time::Duration;

/// How long a call to a peer may take before it counts as not answering.
#[derive(Debug, Clone, Copy, PartialEq, Eq, PartialOrd, Ord, Hash)]
pub struct Deadline(Duration);

impl Deadline {
    /// A read from a peer that is running: a property, a settings value, a name's owner. A
    /// healthy peer answers in milliseconds; this is slack for a busy machine.
    pub const READ: Deadline = Deadline(Duration::from_secs(10));
    /// A read the app waits on before it can draw (the first probe of what is installed, the
    /// first read of the desktop's appearance). Short, so a wedged peer costs a start a moment
    /// and no more; what arrives later is picked up by the follow.
    pub const START: Deadline = Deadline(Duration::from_secs(3));
    /// A call that changes something and returns when it is done: setting a value, creating a
    /// transaction, asking a service to start an action.
    pub const ACTION: Deadline = Deadline(Duration::from_secs(30));
    /// A wait on the person: a password prompt, a print dialog. Long, because they may be
    /// reading it, but not for ever; a peer that vanished is noticed sooner by watching its name.
    pub const PROMPT: Deadline = Deadline(Duration::from_secs(600));

    /// A deadline of `duration`, for a test that must not wait the real one out.
    pub const fn of(duration: Duration) -> Deadline {
        Deadline(duration)
    }

    /// How long it is.
    pub const fn duration(self) -> Duration {
        self.0
    }
}

#[cfg(test)]
mod tests {
    use super::Deadline;

    #[test]
    fn a_prompt_outlasts_an_action_and_an_action_outlasts_a_read() {
        assert!(Deadline::START < Deadline::READ);
        assert!(Deadline::READ < Deadline::ACTION);
        assert!(Deadline::ACTION < Deadline::PROMPT);
    }
}
