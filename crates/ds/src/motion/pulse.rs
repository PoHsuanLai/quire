//! Restarting a keyframe without a reflow: the A/B alias swap (design/05-MOTION.md section 9
//! rule 2). The stylesheet emits every keyframe as `X` and `X--b`; firing toggles
//! `data-pulse` so the animation name changes and the engine restarts it.
#![allow(unused_variables)] // Freeze stubs: remove with the last todo!().

use crate::components::vocab::PulseKey;
use crate::motion::anim::Anim;
use dioxus::prelude::*;

/// A restartable animation on one element.
#[derive(Debug, Clone, Copy, PartialEq)]
pub struct Pulse {
    key: Signal<PulseKey>,
}

impl Pulse {
    /// Play the animation again from its first frame. Call from an event handler.
    pub fn fire(&self) {
        todo!()
    }

    /// What to pass to the component that renders it.
    pub fn key(&self) -> PulseKey {
        todo!()
    }

    /// The class and `data-pulse` value to render (`a-gulp`, `a`|`b`), or `None` at rest.
    pub fn attrs(&self) -> Option<(String, &'static str)> {
        todo!()
    }
}

/// A pulse of `anim`, at rest until fired.
pub fn use_pulse(anim: Anim) -> Pulse {
    todo!()
}
