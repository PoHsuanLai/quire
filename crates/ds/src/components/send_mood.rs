//! The send pill's failed moods (design/04-COMPONENTS.md section 31, C's outbox; design/05-MOTION.md
//! sections 4.4.8 and 4.4.9): a one-shot `nudge` or `shake` when a send goes wrong, played
//! through the pulse machinery and cleared at `settle()`, never looped ("Errors shake once and
//! hold still", design/05 principle 7).

use crate::appearance::MotionLevel;
use crate::components::vocab::{PulseKey, StaggerIndex};
use crate::motion::{Anim, settle};
use crate::root::env::Env;
use crate::time::sleep;
use dioxus::prelude::*;

/// How a send is going, as the pill wears it.
#[derive(Debug, Clone, Copy, PartialEq, Eq, Hash, Default)]
pub enum SendMood {
    /// Nothing wrong: counting, sending or sent.
    #[default]
    Calm,
    /// It failed and will try again by itself: the pill nudges up once (`nudge`, 520 ms).
    Nudge,
    /// It needs the person, a sign-in: the pill shakes once (`shake`, 560 ms).
    Shake,
    /// It will not go: the pill shakes once and turns to `--danger`.
    Fatal,
}

impl SendMood {
    /// The one-shot this mood plays as it arrives; `None` for calm.
    pub(crate) fn anim(self) -> Option<Anim> {
        match self {
            SendMood::Calm => None,
            SendMood::Nudge => Some(Anim::Nudge),
            SendMood::Shake | SendMood::Fatal => Some(Anim::Shake),
        }
    }

    /// `data-mood`: written only when something is wrong, so a calm pill's markup is as it was.
    pub(crate) fn slug(self) -> Option<&'static str> {
        match self {
            SendMood::Calm => None,
            SendMood::Nudge => Some("nudge"),
            SendMood::Shake => Some("shake"),
            SendMood::Fatal => Some("fatal"),
        }
    }
}

/// The pulse to play after `playing` when `anim` arrives: the other alias of the same
/// animation, so it restarts, or a fresh one of a different animation.
pub(crate) fn next_pulse(playing: PulseKey, anim: Anim) -> PulseKey {
    if playing.anim() == anim {
        playing.fired()
    } else {
        PulseKey::rest(anim).fired()
    }
}

/// What the pill last rendered: the mood and the pulse it fired, and which firing that was, so
/// a settle timer from an older firing does not cut a newer one short.
#[derive(Debug, Clone, Copy, PartialEq)]
struct Played {
    mood: SendMood,
    pulse: PulseKey,
    round: u32,
}

/// The pulse the pill wears for `mood`: at rest on mount, fired each time the mood changes to a
/// failed one, back at rest `settle(anim)` after each firing. The pill itself stays up.
///
/// The mood is decided while rendering, so it lives in a plain value (as `Count`'s last value
/// does): a signal written during render would schedule a render for nothing. Only the settle
/// timer writes a signal, the round that has finished, which this render reads.
pub(crate) fn use_mood_pulse(mood: SendMood) -> PulseKey {
    let env = use_hook(try_consume_context::<Signal<Env>>);
    let mut played = use_hook(|| {
        CopyValue::new(Played {
            mood,
            pulse: PulseKey::rest(Anim::Nudge),
            round: 0,
        })
    });
    let settled = use_signal(|| 0u32);
    let seen = *played.peek();
    if seen.mood != mood {
        let next = match mood.anim() {
            Some(anim) => Played {
                mood,
                pulse: next_pulse(seen.pulse, anim),
                round: seen.round.wrapping_add(1),
            },
            None => Played { mood, ..seen },
        };
        played.set(next);
        if let Some(anim) = mood.anim() {
            let level = env
                .map(|env| env.peek().resolved.motion)
                .unwrap_or(MotionLevel::Standard);
            settle_later(settled, anim, level, next.round);
        }
    }
    let now = *played.peek();
    if settled() == now.round {
        PulseKey::rest(now.pulse.anim())
    } else {
        now.pulse
    }
}

/// Mark firing `round` finished once `anim` has played.
fn settle_later(settled: Signal<u32>, anim: Anim, level: MotionLevel, round: u32) {
    spawn(async move {
        sleep(settle(anim, level, StaggerIndex::new(0))).await;
        // Only forward: a slower shake from an older firing must not reopen a newer one. A pill
        // that unmounted meanwhile has nothing left to settle.
        if crate::task::try_get(settled).is_ok_and(|done| done < round) {
            let _ = crate::task::try_set(settled, round);
        }
    });
}

#[cfg(test)]
mod tests {
    use super::{SendMood, next_pulse};
    use crate::components::vocab::{PulseKey, PulsePhase};
    use crate::motion::Anim;

    #[test]
    fn each_failed_mood_plays_its_one_shot() {
        const CASES: &[(SendMood, Option<Anim>)] = &[
            (SendMood::Calm, None),
            (SendMood::Nudge, Some(Anim::Nudge)),
            (SendMood::Shake, Some(Anim::Shake)),
            (SendMood::Fatal, Some(Anim::Shake)),
        ];
        for &(mood, want) in CASES {
            assert_eq!(mood.anim(), want, "{mood:?}");
        }
    }

    #[test]
    fn the_same_animation_swaps_its_alias_and_another_starts_fresh() {
        let nudging = PulseKey::rest(Anim::Nudge).fired();
        assert_eq!(nudging.phase(), PulsePhase::A);
        let again = next_pulse(nudging, Anim::Nudge);
        assert_eq!((again.anim(), again.phase()), (Anim::Nudge, PulsePhase::B));
        let shaking = next_pulse(again, Anim::Shake);
        assert_eq!(
            (shaking.anim(), shaking.phase()),
            (Anim::Shake, PulsePhase::A)
        );
    }
}
