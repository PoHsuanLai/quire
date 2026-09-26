//! SyncHalo: a ring around an account avatar, breathing while idle and spinning while syncing
//! (design/04-COMPONENTS.md section 35).
//!
//! Mail's (mailo pins quire by tag), and unchanged by design/26's first wave: its two rings still
//! loop for as long as they show (design/05 section 12 item 4 proposes removing the idle breathe;
//! mailo decides). It draws the Spinner's ring markup itself, because the `Spinner` component is
//! now a bounded pending loop that needs an `Operation`; the loops live in `sync_halo.css`, the
//! one stylesheet the details lint lets loop for this reason.

use crate::components::avatar::{Avatar, AvatarSize, AvatarTone};
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

impl SyncState {
    /// The `data-sync` word.
    fn slug(self) -> &'static str {
        match self {
            SyncState::Idle => "idle",
            SyncState::Busy => "busy",
        }
    }

    /// The ring it wears, as the Spinner's `data-kind`: Breathe while idle, Spin while busy
    /// (`C:288-292`).
    fn ring(self) -> &'static str {
        match self {
            SyncState::Idle => "breathe",
            SyncState::Busy => "spin",
        }
    }
}

/// An avatar with its sync halo. Both rings loop; the shell keeps Breathe off by default and
/// shows Spin only while busy (TODO(O-9), O-25), which is the consumer's choice of `state`.
#[component]
pub fn SyncHalo(initial: char, tone: AvatarTone, state: SyncState) -> Element {
    rsx! {
        div { class: "ds-account-ring", "data-sync": state.slug(),
            span { class: "ds-spinner", "data-kind": state.ring(), "aria-hidden": "true" }
            Avatar { initial, size: AvatarSize::Size30, tone }
        }
    }
}
