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
/// A key under one of these is hands-off however it is annotated: [`Schema::from_toml`]
/// rejects a schema that marks one settable, and `#[derive(SettingsSchema)]` fails to compile
/// for a field that does (both through [`is_never_settable`]).
///
/// [`Schema::from_toml`]: super::Schema::from_toml
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

/// Whether `path` is under one of the [`AGENT_NEVER_SETTABLE`] prefixes. `const` so the derive
/// can turn a violation into a compile error (`const _: () = assert!(...)`).
///
/// The derive refuses a never-settable key marked `agent_settable` (the doctest pins the
/// error code, so it cannot pass for an unrelated mistake):
///
/// ```compile_fail,E0080
/// use ds_settings::SettingsSchema;
/// use ds_settings::schema::Page;
/// #[derive(Default, serde::Serialize, SettingsSchema)]
/// #[settings(file = "p/s.toml", domain = "cua", page = Page::Dock)]
/// struct S {
///     #[settings(label = "L", agent_settable)]
///     enabled: String,
/// }
/// ```
///
/// The same struct under an ordinary domain compiles:
///
/// ```
/// use ds_settings::SettingsSchema;
/// use ds_settings::schema::Page;
/// #[derive(Default, serde::Serialize, SettingsSchema)]
/// #[settings(file = "p/s.toml", domain = "dock", page = Page::Dock)]
/// struct S {
///     #[settings(label = "L", agent_settable)]
///     enabled: String,
/// }
/// ```
pub const fn is_never_settable(path: &str) -> bool {
    let mut at = 0;
    while at < AGENT_NEVER_SETTABLE.len() {
        if starts_with(path.as_bytes(), AGENT_NEVER_SETTABLE[at].as_bytes()) {
            return true;
        }
        at += 1;
    }
    false
}

const fn starts_with(text: &[u8], prefix: &[u8]) -> bool {
    if prefix.len() > text.len() {
        return false;
    }
    let mut at = 0;
    while at < prefix.len() {
        if text[at] != prefix[at] {
            return false;
        }
        at += 1;
    }
    true
}

#[cfg(test)]
mod tests {
    use super::{AGENT_NEVER_SETTABLE, AGENT_SETTABLE_PROPOSED, is_never_settable};

    #[test]
    fn a_key_under_a_never_prefix_is_never_settable() {
        const CASES: &[(&str, bool)] = &[
            ("cua.enabled", true),
            ("agent.budget", true),
            ("ai.model", true),
            ("memory.on", true),
            ("companion.mood", true),
            ("session.lock_grace_s", true),
            ("idle.dim_after_s", true),
            ("accounts.signed_in", true),
            ("appearance.theme", false),
            ("cuarto.x", false),
            ("cua", false),
            ("", false),
        ];
        for (path, want) in CASES {
            assert_eq!(is_never_settable(path), *want, "{path}");
            let by_list = AGENT_NEVER_SETTABLE.iter().any(|p| path.starts_with(p));
            assert_eq!(is_never_settable(path), by_list, "{path}");
        }
    }

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
