//! What an animated emoji does after a wake, as data (design/25-EMOJI.md section 5): a list
//! of frames to show and waits between them, computed here without a clock and played by
//! `life`. Keeping it pure is what lets the idle rule be tested as arithmetic: every script
//! ends on a rest frame and its waits add up to no more than the awake window.

use super::id::EmojiId;
use super::sheet::durations;
use crate::components::persona::Mood;
use std::time::Duration;

/// What the picture shows: which emoji, at which frame of its loop.
#[derive(Debug, Clone, Copy, PartialEq, Eq, Hash)]
pub(crate) struct Shown {
    pub(crate) emoji: EmojiId,
    pub(crate) frame: u16,
}

impl Shown {
    /// `emoji` at its rest pose, frame 0.
    pub(crate) fn rest(emoji: EmojiId) -> Shown {
        Shown { emoji, frame: 0 }
    }
}

/// One step of a script.
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub(crate) enum Step {
    /// Show this frame now.
    Show(Shown),
    /// Hold what is shown this long.
    Wait(Duration),
}

/// Whether frames advance: not under Reduced motion.
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub(crate) enum Playing {
    /// Loops play inside the awake window.
    Frames,
    /// Still frames only; a reaction is still shown for as long as it would have played.
    Stills,
}

/// Whether this wake came with a new mood, which plays the mood's reaction.
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub(crate) enum MoodChange {
    /// The mood is new.
    Changed,
    /// Mounting, a new wake stamp, or a new pick with the same mood.
    Same,
}

/// The emoji a mood swaps in once, if any.
pub(crate) fn reaction(mood: Mood) -> Option<EmojiId> {
    match mood {
        Mood::Wince => Some(EmojiId::WRONG),
        Mood::Happy => Some(EmojiId::UNLOCKED),
        Mood::Idle | Mood::Attentive | Mood::Asleep => None,
    }
}

/// What a picture shows at rest in `mood`: the sleeping face asleep, the user's otherwise.
pub(crate) fn resting(user: EmojiId, mood: Mood) -> Shown {
    match mood {
        Mood::Asleep => Shown::rest(EmojiId::ASLEEP),
        Mood::Idle | Mood::Attentive | Mood::Wince | Mood::Happy => Shown::rest(user),
    }
}

/// One pass through `emoji`'s loop: each frame shown, then held for its duration.
fn once(emoji: EmojiId) -> Vec<Step> {
    durations(emoji)
        .into_iter()
        .zip(0u16..)
        .flat_map(|(hold, frame)| [Step::Show(Shown { emoji, frame }), Step::Wait(hold)])
        .collect()
}

fn length(emoji: EmojiId) -> Duration {
    durations(emoji).into_iter().sum()
}

/// The script for a wake: the reaction once if the mood just changed to one, then the user's
/// emoji looping for whole loops while they fit in `window`, then at rest. Asleep shows the
/// sleeping face's rest frame and nothing moves.
pub(crate) fn script(
    user: EmojiId,
    mood: Mood,
    change: MoodChange,
    playing: Playing,
    window: Duration,
) -> Vec<Step> {
    let rest = resting(user, mood);
    let swap = match change {
        MoodChange::Changed => reaction(mood),
        MoodChange::Same => None,
    };
    let mut steps = Vec::new();
    let mut spent = Duration::ZERO;
    if let Some(swapped) = swap {
        match playing {
            Playing::Frames => steps.extend(once(swapped)),
            Playing::Stills => steps.extend([
                Step::Show(Shown::rest(swapped)),
                Step::Wait(length(swapped)),
            ]),
        }
        spent += length(swapped);
    }
    let loops = mood != Mood::Asleep && playing == Playing::Frames;
    let each = length(user);
    if loops && !each.is_zero() {
        while spent + each <= window {
            steps.extend(once(user));
            spent += each;
        }
    }
    steps.push(Step::Show(rest));
    steps
}
