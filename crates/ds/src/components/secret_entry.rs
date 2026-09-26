//! The password a lock screen or a polkit prompt asks for, as the prompt holds it
//! (design/04-COMPONENTS.md section 42): a `Secret` field that never writes its text into the
//! markup, emptied by remounting it under a new key; Enter hands the text to `onsubmit`, Escape
//! empties it, and a `Wrong` arrival plays `shake-x` once and empties it when the shake settles
//! ("Errors shake once and hold still", design/05 principle 7).

use crate::appearance::MotionLevel;
use crate::components::lock_vocab::PromptState;
use crate::components::vocab::{PulseKey, StaggerIndex};
use crate::motion::{Anim, settle};
use crate::root::env::Env;
use crate::time::sleep;
use dioxus::prelude::*;

/// Whether the field holds anything: the enter button shows only once it does.
#[derive(Debug, Clone, Copy, PartialEq, Eq, Hash, Default)]
pub(crate) enum Filled {
    /// Nothing typed.
    #[default]
    Empty,
    /// Something typed.
    Typed,
}

impl Filled {
    /// The `data-filled` word.
    pub(crate) fn slug(self) -> &'static str {
        match self {
            Filled::Empty => "empty",
            Filled::Typed => "typed",
        }
    }

    fn of(text: &str) -> Self {
        if text.is_empty() {
            Filled::Empty
        } else {
            Filled::Typed
        }
    }
}

/// Whether the prompt was in its failed state when last rendered.
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
enum Failed {
    Wrong,
    Other,
}

/// What the prompt last rendered: whether it was wrong, the shake it fired, and which firing.
#[derive(Debug, Clone, Copy, PartialEq)]
struct Shaken {
    wrong: Failed,
    pulse: PulseKey,
    round: u32,
}

/// The prompt's hold on its secret.
#[derive(Clone, Copy)]
pub(crate) struct SecretEntry {
    /// The text typed, kept out of the markup and out of any signal a render reads.
    typed: CopyValue<String>,
    /// The field's key: a new one remounts the `Secret` field empty.
    generation: Signal<u32>,
    filled: Signal<Filled>,
    /// The shake, at rest once it has played.
    pub(crate) pulse: PulseKey,
    oninput: EventHandler<String>,
}

impl SecretEntry {
    /// The key to mount the field under.
    pub(crate) fn key(&self) -> u32 {
        (self.generation)()
    }

    /// Whether anything is typed.
    pub(crate) fn filled(&self) -> Filled {
        (self.filled)()
    }

    /// The field reported `text`.
    pub(crate) fn input(&self, text: String) {
        let mut filled = self.filled;
        let now = Filled::of(&text);
        if *filled.peek() != now {
            filled.set(now);
        }
        let mut typed = self.typed;
        typed.set(text.clone());
        self.oninput.call(text);
    }

    /// Empty the field: remount it, forget the text, and tell the caller.
    pub(crate) fn clear(&self) {
        clear(self.typed, self.generation, self.filled, self.oninput);
    }

    /// Hand the text to `onsubmit`, when there is any.
    pub(crate) fn submit(&self, onsubmit: EventHandler<String>) {
        let text = self.typed.peek().clone();
        if !text.is_empty() {
            onsubmit.call(text);
        }
    }
}

fn clear(
    typed: CopyValue<String>,
    generation: Signal<u32>,
    filled: Signal<Filled>,
    oninput: EventHandler<String>,
) {
    let mut typed = typed;
    typed.set(String::new());
    let _ = crate::task::try_set(filled, Filled::Empty);
    if let Ok(round) = crate::task::try_get(generation) {
        let _ = crate::task::try_set(generation, round.wrapping_add(1));
    }
    oninput.call(String::new());
}

/// The secret for a prompt in `state`. Each time `state` becomes `Wrong` the shake fires (on
/// the other alias if it is already playing) and, `settle(ShakeX)` later, the field empties and
/// the shake is at rest again. Mounted in `Wrong`, it shakes as it arrives.
pub(crate) fn use_secret_entry(state: &PromptState, oninput: EventHandler<String>) -> SecretEntry {
    let env = use_hook(try_consume_context::<Signal<Env>>);
    let typed = use_hook(|| CopyValue::new(String::new()));
    let generation = use_signal(|| 0u32);
    let filled = use_signal(Filled::default);
    let settled = use_signal(|| 0u32);
    let mut shaken = use_hook(|| {
        CopyValue::new(Shaken {
            wrong: Failed::Other,
            pulse: PulseKey::rest(Anim::ShakeX),
            round: 0,
        })
    });
    let wrong = if state.is_wrong() {
        Failed::Wrong
    } else {
        Failed::Other
    };
    let seen = *shaken.peek();
    if seen.wrong != wrong {
        let next = match wrong {
            Failed::Wrong => Shaken {
                wrong,
                pulse: seen.pulse.fired(),
                round: seen.round.wrapping_add(1),
            },
            Failed::Other => Shaken { wrong, ..seen },
        };
        shaken.set(next);
        if wrong == Failed::Wrong {
            let level = env
                .map(|env| env.peek().resolved.motion)
                .unwrap_or(MotionLevel::Standard);
            let empty = move || clear(typed, generation, filled, oninput);
            settle_later(settled, level, next.round, empty);
        }
    }
    let now = *shaken.peek();
    let pulse = if settled() == now.round {
        PulseKey::rest(Anim::ShakeX)
    } else {
        now.pulse
    };
    SecretEntry {
        typed,
        generation,
        filled,
        pulse,
        oninput,
    }
}

/// Once the shake has played, mark firing `round` finished and empty the field.
fn settle_later(
    settled: Signal<u32>,
    level: MotionLevel,
    round: u32,
    empty: impl FnOnce() + 'static,
) {
    spawn(async move {
        sleep(settle(Anim::ShakeX, level, StaggerIndex::new(0))).await;
        // Only forward: an older firing's timer must not cut a newer shake short. A prompt that
        // unmounted meanwhile has nothing left to empty.
        if crate::task::try_get(settled).is_ok_and(|done| done < round) {
            let _ = crate::task::try_set(settled, round);
            empty();
        }
    });
}

#[cfg(test)]
mod tests {
    use super::Filled;

    #[test]
    fn filled_follows_emptiness_only() {
        assert_eq!(Filled::of(""), Filled::Empty);
        assert_eq!(Filled::of("x"), Filled::Typed);
        assert_eq!(Filled::Typed.slug(), "typed");
    }
}
