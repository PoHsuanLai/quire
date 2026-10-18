//! The words a service uses for the peer that called it and for its answer.

/// The unique bus name of the peer that made a call (`:1.42`).
#[derive(Debug, Clone, PartialEq, Eq, Hash)]
pub struct SenderName(pub String);

/// A service's answer to "may this caller do this".
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum Verdict {
    Allow,
    /// The caller may not do this. The call fails with a refusal that names no reason.
    Refuse,
}
