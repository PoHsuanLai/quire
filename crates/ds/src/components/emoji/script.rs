//! What an animated emoji does after a wake, as data (design/25-EMOJI.md section 5): a list
//! of frames to show and waits between them, computed here without a clock and played by
//! `life`. Keeping it pure is what lets the idle rule be tested as arithmetic: every script
//! ends on a rest frame and its waits add up to no more than the awake window.

use super::id::EmojiId;
use super::sheet::durations;
use crate::components::user_picture::Mood;
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

/// The emoji a mood swaps in once, if any: a glance with the eyes when the user starts typing,
/// the confounded face for a wrong password, the partying face on unlock.
pub(crate) fn reaction(mood: Mood) -> Option<EmojiId> {
    match mood {
        Mood::Attentive => Some(EmojiId::ATTENTIVE),
        Mood::Wince => Some(EmojiId::WRONG),
        Mood::Happy => Some(EmojiId::UNLOCKED),
        Mood::Idle | Mood::Asleep => None,
    }
}

/// How the user's own emoji plays after any reaction, inside the awake window.
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub(crate) enum Pace {
    /// Loop after loop: watching the field.
    Steady,
    /// One loop, then [`IDLE_REST`] on the rest frame, and again: at rest, but alive.
    Slow,
    /// Not at all: asleep.
    Still,
}

/// The rest between an idle picture's loops.
pub(crate) const IDLE_REST: Duration = Duration::from_secs(4);

/// The pace a mood plays at.
pub(crate) fn pace(mood: Mood) -> Pace {
    match mood {
        Mood::Attentive => Pace::Steady,
        Mood::Idle | Mood::Wince | Mood::Happy => Pace::Slow,
        Mood::Asleep => Pace::Still,
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
/// emoji at the mood's [`Pace`] for whole loops while they fit in `window`, then at rest. Asleep
/// shows the sleeping face's rest frame and nothing moves.
pub(crate) fn script(
    user: EmojiId,
    mood: Mood,
    change: MoodChange,
    playing: Playing,
    window: Duration,
) -> Vec<Step> {
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
    if playing == Playing::Frames {
        steps.extend(loops(user, pace(mood), window.saturating_sub(spent)));
    }
    steps.push(Step::Show(resting(user, mood)));
    steps
}

/// `user`'s whole loops at `pace` inside `room`, ending wherever the last loop ends.
fn loops(user: EmojiId, pace: Pace, room: Duration) -> Vec<Step> {
    let each = length(user);
    let gap = match pace {
        Pace::Steady => Duration::ZERO,
        Pace::Slow => IDLE_REST,
        Pace::Still => return Vec::new(),
    };
    if each.is_zero() {
        return Vec::new();
    }
    let mut steps = Vec::new();
    let mut spent = Duration::ZERO;
    while spent + each <= room {
        if spent > Duration::ZERO && !gap.is_zero() {
            if spent + gap + each > room {
                break;
            }
            steps.extend([Step::Show(Shown::rest(user)), Step::Wait(gap)]);
            spent += gap;
        }
        steps.extend(once(user));
        spent += each;
    }
    steps
}
