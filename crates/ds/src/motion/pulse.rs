//! Restarting a keyframe without a reflow: the A/B alias swap (design/05-MOTION.md section 9
//! rule 2). The stylesheet emits every keyframe as `X` and `X--b`; firing toggles
//! `data-pulse` so the animation name changes and the engine restarts it.

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
        // A pulse whose owner has unmounted has nothing left to play.
        if let Ok(key) = crate::task::try_get(self.key) {
            let _ = crate::task::try_set(self.key, key.fired());
        }
    }

    /// What to pass to the component that renders it.
    pub fn key(&self) -> PulseKey {
        (self.key)()
    }

    /// The class and `data-pulse` value to render (`a-gulp`, `a`|`b`), or `None` at rest.
    pub fn attrs(&self) -> Option<(String, &'static str)> {
        self.key().attrs()
    }
}

/// A pulse of `anim`, at rest until fired.
pub fn use_pulse(anim: Anim) -> Pulse {
    Pulse {
        key: use_signal(|| PulseKey::rest(anim)),
    }
}
