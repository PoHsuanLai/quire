//! An event's identity as the service that saw it reports it (design/26-DETAILS.md R6).

/// The same stamp is the same event: a state carrying a failure's stamp that is set again does
/// not change, so the failure never replays (R6); a new stamp is a new event and replays the
/// identical reaction.
#[derive(Debug, Clone, Copy, PartialEq, Eq, Hash, PartialOrd, Ord)]
pub struct EventStamp(pub u32);
