//! The follow hook: a timeline its caller recomputes each render, run from now whenever it
//! changes.

use super::Timeline;
use super::playback::use_playback;
use dioxus::core::queue_effect;
use dioxus::prelude::*;

/// The frame of `timeline`: a first or a changed `timeline` starts a run from now, frames tick
/// until it settles, and nothing asks for one at rest. Reduced motion is the timeline's own business (a
/// timeline built for it is already at its end), since some timelines (a pending loop) still
/// wait out a grace under it.
pub(crate) fn use_timeline<T: Timeline>(timeline: T) -> T::Frame {
    let playback = use_playback(timeline.clone());
    let mut seen = use_hook(|| CopyValue::new(None::<T>));
    if seen.peek().as_ref() != Some(&timeline) {
        seen.set(Some(timeline.clone()));
        queue_effect(move || playback.play(timeline));
    }
    playback.frame()
}
