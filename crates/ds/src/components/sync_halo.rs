//! SyncHalo: a ring around an account avatar, breathing while idle and spinning while syncing
//! (design/04-COMPONENTS.md section 35).

use crate::components::avatar::AvatarTone;
use dioxus::prelude::*;

/// Whether the account is syncing.
#[derive(Debug, Clone, Copy, PartialEq, Eq, Hash, Default)]
pub enum SyncState {
    /// Live and listening.
    #[default]
    Idle,
    /// Syncing.
    Busy,
}

/// An avatar with its sync halo.
#[component]
pub fn SyncHalo(initial: char, tone: AvatarTone, state: SyncState) -> Element {
    todo!()
}
