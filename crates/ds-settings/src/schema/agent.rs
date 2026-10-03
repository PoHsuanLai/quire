//! Which keys are proposed as agent-settable (design/22-SETTINGS.md section 9.7).
//!
//! The mark itself lives in each program's own schema (`#[settings(agent_settable)]`, written as
//! `agent = "settable"`); this file is the one reviewed list of what quire proposes the programs
//! mark, so quire's, sill's and detent's tests can check their schemas against the same words.
//! The list is a proposal for the person to accept or trim, not a grant.

/// The keys proposed as agent-settable: appearance, dock size and magnification, the
/// launcher's web search, and Do Not Disturb. Each is cosmetic or a convenience the person undoes
/// in one click; none reaches data, accounts, the network beyond a search link, or a lock.
pub const AGENT_SETTABLE_PROPOSED: &[&str] = &[
    "appearance.theme",
    "appearance.accent",
    "dock.magnification",
    "dock.tile_size_px",
    "dock.magnified_size_px",
    "launcher.web_search",
    "notifications.dnd",
];

/// Key-path prefixes no schema may mark agent-settable: the companion's own limits, the memory
/// and computer-use switches, model and spend choices, and anything that locks or signs in.
/// A key under one of these is hands-off however it is annotated.
pub const AGENT_NEVER_SETTABLE: &[&str] = &[
    "agent.",
    "ai.",
    "cua.",
    "memory.",
    "companion.",
    "session.",
    "idle.",
    "accounts.",
];

#[cfg(test)]
mod tests {
    use super::{AGENT_NEVER_SETTABLE, AGENT_SETTABLE_PROPOSED};

    #[test]
    fn nothing_proposed_is_security_relevant() {
        for key in AGENT_SETTABLE_PROPOSED {
            assert!(
                !AGENT_NEVER_SETTABLE
                    .iter()
                    .any(|prefix| key.starts_with(prefix)),
                "{key} is under a never-settable prefix"
            );
        }
    }

    #[test]
    fn the_proposal_has_no_duplicates() {
        let mut sorted = AGENT_SETTABLE_PROPOSED.to_vec();
        sorted.sort_unstable();
        sorted.dedup();
        assert_eq!(sorted.len(), AGENT_SETTABLE_PROPOSED.len());
    }
}
