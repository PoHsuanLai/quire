//! How long a client waits on a live module.

use ds_peer::Deadline;

/// How long each call to a live module may take before it is [`LiveError::Timeout`](super::LiveError::Timeout).
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
#[non_exhaustive]
pub struct Timeouts {
    /// `Describe`, `Get`, `Snapshot` and the subscription: the module reads its own state.
    pub read: Deadline,
    /// `Set`, including running an action key: the module may ask a provider, or the person.
    pub action: Deadline,
}

impl Timeouts {
    /// Timeouts of `read` for reads and `action` for a `Set`.
    pub const fn new(read: Deadline, action: Deadline) -> Timeouts {
        Timeouts { read, action }
    }
}

impl Default for Timeouts {
    fn default() -> Timeouts {
        Timeouts::new(Deadline::READ, Deadline::ACTION)
    }
}
