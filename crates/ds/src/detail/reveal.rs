//! Reveal: a list's children rise in turn on first show, and never again (design/26-DETAILS.md
//! R1, R13). [`Reveal`] plays at mount; a part that outlives its lists (the command palette,
//! whose result sets come and go) plays from a [`RevealCue`] instead, each new Appear once.

use super::cue::Cue;
use super::first_show::FirstShow;
use super::moment::Moment;
use crate::components::vocab::StaggerIndex;
use crate::motion::{Anim, TimerPhase, use_motion_timer};
use dioxus::core::queue_effect;
use dioxus::prelude::*;

/// What tells a list that outlives its contents when to rise: its surface's opening
/// (`First(FirstShow::Animate)` rises on each opening, `Still` never), or a [`Cue`] from the
/// caller's own `use_detail`, whose each new Appear rises once and whose every other moment
/// (a Change: a new result set replacing the old in place) replays nothing.
#[derive(Debug, Clone, Copy, PartialEq, Eq, Hash)]
pub enum RevealCue {
    /// Rise on the surface's opening, or never.
    First(FirstShow),
    /// Rise on each new Appear of this cue.
    Cue(Cue),
}

impl Default for RevealCue {
    /// No rise at all: what a list drew before it could be told.
    fn default() -> Self {
        RevealCue::First(FirstShow::Still)
    }
}

impl From<FirstShow> for RevealCue {
    fn from(first: FirstShow) -> Self {
        RevealCue::First(first)
    }
}

impl From<Cue> for RevealCue {
    fn from(cue: Cue) -> Self {
        RevealCue::Cue(cue)
    }
}

impl RevealCue {
    /// The key a rise starts on, given how many times the surface has opened: a new key is a
    /// new rise, `None` none.
    pub(crate) fn key(self, openings: u32) -> Option<u32> {
        match self {
            RevealCue::First(FirstShow::Animate) => Some(openings),
            RevealCue::First(FirstShow::Still) => None,
            RevealCue::Cue(cue) => (cue.moment() == Moment::Appear).then_some(cue.serial()),
        }
    }
}

/// Whether a list's rise is playing.
#[derive(Debug, Clone, Copy, PartialEq, Eq, Hash)]
pub(crate) enum Revealing {
    /// Its children rise (`data-reveal=play`).
    Play,
    /// They sit still.
    Still,
}

impl Revealing {
    /// The `data-reveal` word.
    pub(crate) fn slug(self) -> &'static str {
        match self {
            Revealing::Play => "play",
            Revealing::Still => "still",
        }
    }
}

/// `Play` from the render that first sees a new `key` until the twelfth child's rise settles
/// (`settle(Rise, level, 12)`), `Still` otherwise: each key plays once, and a re-render, or the
/// same key again, replays nothing (R1).
pub(crate) fn use_rise_on(key: Option<u32>) -> Revealing {
    let timer = use_motion_timer(Anim::Rise);
    let mut seen = use_hook(|| CopyValue::new(None::<u32>));
    let fresh = key.is_some() && *seen.peek() != key;
    if fresh {
        seen.set(key);
        queue_effect(move || {
            timer.start_staggered(StaggerIndex::new(usize::from(StaggerIndex::CAP)));
        });
    }
    match (fresh, timer.phase()) {
        (true, _) | (false, TimerPhase::Running) => Revealing::Play,
        (false, TimerPhase::Idle | TimerPhase::Settled) => Revealing::Still,
    }
}

/// Its direct children play `rise` at `--t-move --e-out`, each `--stagger` after the one before,
/// the index capped at 12 (R13), only when `first` is `Animate` and only in the first settle
/// after mounting; a re-render never replays it (R1). Under Reduced the stagger is 0 and the rise
/// 60 ms, as the level's tokens say. The wrapper is a plain block (`div.ds-reveal`): lay the
/// children out inside it.
#[component]
pub fn Reveal(first: FirstShow, children: Element) -> Element {
    let playing = use_rise_on(RevealCue::First(first).key(0));
    rsx! {
        div { class: "ds-reveal", "data-reveal": playing.slug(), {children} }
    }
}

#[cfg(test)]
mod tests {
    use super::RevealCue;
    use crate::detail::{Cue, FirstShow, Moment, Touch};

    #[test]
    fn only_an_appear_or_an_opening_is_a_rise() {
        let cue = |moment| RevealCue::Cue(Cue::new(moment, Touch::Remote, 4));
        let cases: &[(RevealCue, u32, Option<u32>)] = &[
            (RevealCue::First(FirstShow::Animate), 0, Some(0)),
            (RevealCue::First(FirstShow::Animate), 3, Some(3)),
            (RevealCue::First(FirstShow::Still), 3, None),
            (cue(Moment::Appear), 9, Some(4)),
            (cue(Moment::Change), 9, None),
            (cue(Moment::Rest), 9, None),
            (RevealCue::default(), 0, None),
        ];
        for &(reveal, openings, want) in cases {
            assert_eq!(reveal.key(openings), want, "{reveal:?} {openings}");
        }
    }
}
