//! A pulse's render key: which animation a `use_pulse` plays and which of its two identical
//! keyframe aliases (`X` or `X--b`) is playing, so a restyle restarts it.

use crate::motion::anim::Anim;

/// Which of the two identical keyframe aliases a pulse is playing (`X` or `X--b`).
#[derive(Debug, Clone, Copy, PartialEq, Eq, Hash, Default)]
pub enum PulsePhase {
    /// Not playing.
    #[default]
    Rest,
    /// Playing the `X` alias: `data-pulse="a"`.
    A,
    /// Playing the `X--b` alias: `data-pulse="b"`.
    B,
}

/// The render-side half of a `use_pulse`: which animation, and which alias is playing.
/// Renders as `class="a-<anim>"` plus `data-pulse="a|b"`.
#[derive(Debug, Clone, Copy, PartialEq, Eq, Hash)]
pub struct PulseKey {
    anim: Anim,
    phase: PulsePhase,
}

impl PulseKey {
    /// A pulse of `anim` that is not playing.
    pub fn rest(anim: Anim) -> Self {
        PulseKey {
            anim,
            phase: PulsePhase::Rest,
        }
    }

    /// The animation this key plays.
    pub fn anim(self) -> Anim {
        self.anim
    }

    /// Which alias is playing.
    pub fn phase(self) -> PulsePhase {
        self.phase
    }

    /// The same animation with the other alias playing: what `Pulse::fire` stores.
    pub fn fired(self) -> Self {
        let phase = match self.phase {
            PulsePhase::Rest | PulsePhase::B => PulsePhase::A,
            PulsePhase::A => PulsePhase::B,
        };
        PulseKey { phase, ..self }
    }

    /// `base` with this pulse's class added while it plays, and its `data-pulse` alias.
    pub fn wear(self, base: &str) -> (String, Option<&'static str>) {
        match self.attrs() {
            Some((anim, alias)) => (format!("{base} {anim}"), Some(alias)),
            None => (base.to_string(), None),
        }
    }

    /// The class and `data-pulse` value to render, or `None` while at rest.
    pub fn attrs(self) -> Option<(String, &'static str)> {
        let alias = match self.phase {
            PulsePhase::Rest => return None,
            PulsePhase::A => "a",
            PulsePhase::B => "b",
        };
        Some((self.anim.class().to_string(), alias))
    }
}
