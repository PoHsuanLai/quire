//! SyncHalo: a ring around an account avatar, breathing while idle and spinning while syncing
//! (design/04-COMPONENTS.md section 35).

use crate::components::avatar::{Avatar, AvatarSize, AvatarTone};
use crate::components::spinner::{Spinner, SpinnerKind};
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

    /// The ring it wears: Breathe while idle, Spin while busy (`C:288-292`).
    fn ring(self) -> SpinnerKind {
        match self {
            SyncState::Idle => SpinnerKind::Breathe,
            SyncState::Busy => SpinnerKind::Spin,
        }
    }
}

/// An avatar with its sync halo. Both rings loop; the shell keeps Breathe off by default and
/// shows Spin only while busy (TODO(O-9), O-25), which is the consumer's choice of `state`.
#[component]
pub fn SyncHalo(initial: char, tone: AvatarTone, state: SyncState) -> Element {
    rsx! {
        div { class: "ds-account-ring", "data-sync": state.slug(),
            Spinner { kind: state.ring() }
            Avatar { initial, size: AvatarSize::Size30, tone }
        }
    }
}
