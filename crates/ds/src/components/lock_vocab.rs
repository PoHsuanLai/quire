//! What the lock screen and the polkit prompt say about a password being asked for
//! (design/20-SURFACES.md sections 1.9 and 1.10; design/04-COMPONENTS.md section 42): whose it
//! is, how the asking is going, whether caps lock is on, and where the Space's colour reaches.

use crate::components::avatar::AvatarFace;
use crate::components::vocab::Availability;

/// Where the current Space's colour reaches on the lock screen. The reference lock screen is
/// white type and a white glass field over the wallpaper; Arc's colour may take the field.
#[derive(Debug, Clone, Copy, PartialEq, Eq, Hash, Default)]
pub enum LockLook {
    /// White type, a flat white glass field (`--lock-ink`, `--lock-glass`).
    #[default]
    Clear,
    /// The Space's colour on the small surfaces: the date line sits in a pill and the password
    /// field is painted with the Space gradient, in the Space's frame ink (`--f-grad`,
    /// `--f-ink`). The time stays white.
    Space,
}

impl LockLook {
    /// The `data-look` word.
    pub(crate) fn slug(self) -> &'static str {
        match self {
            LockLook::Clear => "clear",
            LockLook::Space => "space",
        }
    }
}

/// How the asking is going. The caller moves it: `Checking` while the password is tried,
/// `Wrong` when it failed (the field shakes once and clears), `LockedOut` after too many tries.
#[derive(Debug, Clone, PartialEq, Eq, Hash, Default)]
pub enum PromptState {
    /// Waiting for a password.
    #[default]
    Idle,
    /// The password is being tried: the field holds still and the enter button spins.
    Checking,
    /// It was wrong: the field plays `shake-x` once (420 ms, `--e-shake`), then empties itself.
    Wrong,
    /// Too many tries: the field is closed until `until`, the caller's own wording of the time
    /// ("9:52"), which the hint line shows as "Try again at 9:52".
    LockedOut {
        /// When it opens again, as the caller writes the time.
        until: String,
    },
}

impl PromptState {
    /// The `data-state` word.
    pub(crate) fn slug(&self) -> &'static str {
        match self {
            PromptState::Idle => "idle",
            PromptState::Checking => "checking",
            PromptState::Wrong => "wrong",
            PromptState::LockedOut { .. } => "locked-out",
        }
    }

    /// Whether the field takes typing: not while the password is tried or the prompt is shut.
    pub(crate) fn availability(&self) -> Availability {
        match self {
            PromptState::Idle | PromptState::Wrong => Availability::Enabled,
            PromptState::Checking | PromptState::LockedOut { .. } => Availability::Disabled,
        }
    }

    /// Whether this is the failed state, the one that shakes.
    pub(crate) fn is_wrong(&self) -> bool {
        matches!(self, PromptState::Wrong)
    }
}

/// Whether caps lock is on, which the field marks beside its button.
#[derive(Debug, Clone, Copy, PartialEq, Eq, Hash, Default)]
pub enum CapsLock {
    /// On: the caps-lock arrow shows in the field.
    On,
    /// Off.
    #[default]
    Off,
}

/// The person the password belongs to.
#[derive(Debug, Clone, PartialEq, Eq, Hash)]
pub struct LockUser {
    /// Their name, as the session shows it.
    pub name: String,
    /// Their avatar; the prompt draws it at its own size (64 at the lock screen, 48 in a
    /// polkit prompt), whatever size it carries.
    pub avatar: AvatarFace,
}

#[cfg(test)]
mod tests {
    use super::PromptState;
    use crate::components::vocab::Availability;

    #[test]
    fn only_idle_and_wrong_take_typing() {
        let cases = [
            (PromptState::Idle, Availability::Enabled, "idle"),
            (PromptState::Wrong, Availability::Enabled, "wrong"),
            (PromptState::Checking, Availability::Disabled, "checking"),
            (
                PromptState::LockedOut {
                    until: "9:52".into(),
                },
                Availability::Disabled,
                "locked-out",
            ),
        ];
        for (state, availability, slug) in cases {
            assert_eq!(state.availability(), availability, "{state:?}");
            assert_eq!(state.slug(), slug, "{state:?}");
        }
    }
}
