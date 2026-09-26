//! Tracking a component's state and naming the moment its latest change is (design/26 4.1).

use super::cue::Cue;
use super::detailed::Detailed;
use super::first_show::FirstShow;
use super::moment::Moment;
use super::touch::Touch;
use dioxus::prelude::*;

/// A component's state with the moment its latest change is.
#[derive(Debug, Clone, PartialEq)]
pub struct Detail<S> {
    state: S,
    cue: Cue,
}

impl<S> Detail<S> {
    /// The state as it is now.
    pub fn state(&self) -> &S {
        &self.state
    }

    /// The latest change, for the primitives.
    pub fn cue(&self) -> Cue {
        self.cue
    }

    /// What the latest change means.
    pub fn moment(&self) -> Moment {
        self.cue.moment()
    }
}

/// Track `state`: the first frame is `S::first` (its Appear only when `first` is `Animate`), and
/// each change after is `S::moment(old, new)`, caused by `touch`. The same state rendered again
/// is the same cue, so nothing replays (R1). Moments never queue: the newest supersedes (R11).
pub fn use_detail<S: Detailed>(state: S, first: FirstShow, touch: Touch) -> Detail<S> {
    // The last state seen lives in a plain value, not a signal: the change is noticed while
    // rendering, and a signal written during render would schedule a second render for nothing.
    let mut seen = use_hook(|| CopyValue::new(None::<(S, Cue)>));
    let before = seen.peek().clone();
    let cue = match &before {
        Some((was, cue)) if *was == state => *cue,
        Some((was, cue)) => Cue::new(S::moment(was, &state), touch, cue.serial() + 1),
        None => Cue::new(opening(S::first(&state), first), touch, 1),
    };
    if before.map(|(_, seen_cue)| seen_cue) != Some(cue) {
        seen.set(Some((state.clone(), cue)));
    }
    Detail { state, cue }
}

/// A first frame's moment: `Still` holds back an Appear and nothing else.
fn opening(moment: Moment, first: FirstShow) -> Moment {
    match (moment, first) {
        (Moment::Appear, FirstShow::Still) => Moment::Rest,
        (moment, FirstShow::Animate | FirstShow::Still) => moment,
    }
}

#[cfg(test)]
mod tests {
    use super::opening;
    use crate::detail::{FirstShow, Moment};

    #[test]
    fn still_holds_back_only_an_appear() {
        const CASES: &[(Moment, FirstShow, Moment)] = &[
            (Moment::Appear, FirstShow::Animate, Moment::Appear),
            (Moment::Appear, FirstShow::Still, Moment::Rest),
            (Moment::Pending, FirstShow::Still, Moment::Pending),
            (Moment::Rest, FirstShow::Animate, Moment::Rest),
        ];
        for &(moment, first, want) in CASES {
            assert_eq!(opening(moment, first), want, "{moment:?} {first:?}");
        }
    }
}
