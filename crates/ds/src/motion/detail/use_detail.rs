//! Tracking a component's state and naming the moment its latest change is (design/26 4.1).

use super::cue::Cue;
use super::detailed::Detailed;
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

/// Track `state`: the first frame is `S::first`, and each change after is `S::moment(old, new)`,
/// caused by `touch`. The same state rendered again is the same cue, so nothing replays (R1).
/// Moments never queue: the newest supersedes (R11).
pub fn use_detail<S: Detailed>(state: S, touch: Touch) -> Detail<S> {
    // The last state seen lives in a plain value, not a signal: the change is noticed while
    // rendering, and a signal written during render would schedule a second render for nothing.
    let mut seen = use_hook(|| CopyValue::new(None::<(S, Cue)>));
    let before = seen.peek().clone();
    let cue = match &before {
        Some((was, cue)) if *was == state => *cue,
        Some((was, cue)) => Cue::new(S::moment(was, &state), touch, cue.serial() + 1),
        None => Cue::new(S::first(&state), touch, 1),
    };
    if before.map(|(_, seen_cue)| seen_cue) != Some(cue) {
        seen.set(Some((state.clone(), cue)));
    }
    Detail { state, cue }
}
