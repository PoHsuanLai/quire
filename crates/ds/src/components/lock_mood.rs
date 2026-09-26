//! What the lock prompt's persona is doing (design/04-COMPONENTS.md section 42;
//! design/24-PERSONA.md section 4), read from the prompt's own state so the shell sets no mood:
//! attentive while the person types or the password is tried, a wince when it was wrong, happy
//! once it was accepted, idle otherwise; and a [`WakeStamp`] that any key or pointer activity in
//! the prompt advances, so the persona rests again 20 s after the last of it.

use crate::components::lock_vocab::PromptState;
use crate::components::persona::{Mood, WakeStamp};
use crate::components::secret_entry::Filled;
use crate::components::vocab::PulsePhase;
use dioxus::prelude::*;
use std::time::{Duration, Instant};

/// Whether the caret is in the password field.
#[derive(Debug, Clone, Copy, PartialEq, Eq, Hash)]
pub(crate) enum Caret {
    /// The field has the keyboard.
    In,
    /// It does not.
    Out,
}

/// The persona's mood for a prompt in `state` whose field holds `filled` with the caret `caret`,
/// its shake at `shake`. A wrong password winces through the shake and while the field is
/// empty after it; typing again turns it attentive, so the next wrong winces once more (a
/// wince does not escalate).
pub(crate) fn prompt_mood(
    state: &PromptState,
    filled: Filled,
    caret: Caret,
    shake: PulsePhase,
) -> Mood {
    let typing = filled == Filled::Typed && caret == Caret::In;
    match state {
        PromptState::Accepted => Mood::Happy,
        PromptState::Checking => Mood::Attentive,
        PromptState::Wrong if typing && shake == PulsePhase::Rest => Mood::Attentive,
        PromptState::Wrong => Mood::Wince,
        PromptState::Idle if typing => Mood::Attentive,
        PromptState::Idle | PromptState::LockedOut { .. } => Mood::Idle,
    }
}

/// The finest a stream of activity (a moving pointer, held keys) advances the stamp: each
/// advance replays the persona's breath from its start, so a stream wakes it once a second at
/// most, and it rests between 19 and 20 s after the last of it.
const GRAIN: Duration = Duration::from_secs(1);

/// The prompt's own wake stamp, advanced by activity in it.
#[derive(Clone, Copy)]
pub(crate) struct Stir {
    stamp: Signal<WakeStamp>,
    last: CopyValue<Option<Instant>>,
}

impl Stir {
    /// Something happened in the prompt: advance the stamp, unless it advanced under
    /// [`GRAIN`] ago.
    pub(crate) fn stirred(self) {
        let now = Instant::now();
        let mut last = self.last;
        let due = last
            .peek()
            .is_none_or(|then| now.duration_since(then) >= GRAIN);
        if due {
            last.set(Some(now));
            let mut stamp = self.stamp;
            let next = stamp.peek().next();
            stamp.set(next);
        }
    }

    /// The stamp the persona takes: the prompt's own, moved on by the caller's `outside` one,
    /// so a change in either wakes it.
    pub(crate) fn stamp(self, outside: Option<WakeStamp>) -> WakeStamp {
        let own = (self.stamp)();
        WakeStamp(own.0.wrapping_add(outside.map_or(0, |stamp| stamp.0)))
    }
}

/// A prompt's stir, at rest until the first activity.
pub(crate) fn use_stir() -> Stir {
    Stir {
        stamp: use_signal(WakeStamp::default),
        last: use_hook(|| CopyValue::new(None)),
    }
}

#[cfg(test)]
mod tests {
    use super::{Caret, prompt_mood};
    use crate::components::lock_vocab::PromptState;
    use crate::components::persona::Mood;
    use crate::components::secret_entry::Filled;
    use crate::components::vocab::PulsePhase;

    #[test]
    fn the_prompt_s_state_picks_the_mood() {
        let out = PromptState::LockedOut {
            until: "9:52".into(),
        };
        let cases = [
            (
                PromptState::Idle,
                Filled::Empty,
                Caret::In,
                PulsePhase::Rest,
                Mood::Idle,
            ),
            (
                PromptState::Idle,
                Filled::Typed,
                Caret::In,
                PulsePhase::Rest,
                Mood::Attentive,
            ),
            (
                PromptState::Idle,
                Filled::Typed,
                Caret::Out,
                PulsePhase::Rest,
                Mood::Idle,
            ),
            (
                PromptState::Checking,
                Filled::Typed,
                Caret::Out,
                PulsePhase::Rest,
                Mood::Attentive,
            ),
            (
                PromptState::Wrong,
                Filled::Typed,
                Caret::In,
                PulsePhase::A,
                Mood::Wince,
            ),
            (
                PromptState::Wrong,
                Filled::Empty,
                Caret::In,
                PulsePhase::Rest,
                Mood::Wince,
            ),
            (
                PromptState::Wrong,
                Filled::Typed,
                Caret::In,
                PulsePhase::Rest,
                Mood::Attentive,
            ),
            (
                PromptState::Accepted,
                Filled::Typed,
                Caret::In,
                PulsePhase::Rest,
                Mood::Happy,
            ),
            (out, Filled::Typed, Caret::In, PulsePhase::Rest, Mood::Idle),
        ];
        for (state, filled, caret, shake, mood) in cases {
            assert_eq!(
                prompt_mood(&state, filled, caret, shake),
                mood,
                "{state:?} {filled:?} {caret:?} {shake:?}"
            );
        }
    }
}
