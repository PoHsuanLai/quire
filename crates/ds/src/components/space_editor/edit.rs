//! The editor's edits as pure functions: a look in, the edited look out
//! (design/04-COMPONENTS.md section 32 behaviour, design/06-INTERACTIONS.md section 2.8).

use crate::components::vocab::Fraction;
use crate::space::{Dot, PRESETS, SpaceLook};
use dioxus::prelude::Key;

/// A Space holds at most three dots (`S:1425`).
pub(super) const MAX_DOTS: usize = 3;
/// A key moves the hue this many degrees (`S:1455`).
const HUE_STEP: f32 = 5.0;
/// A key moves the chroma this much (`S:1456`).
const CHROMA_STEP: f32 = 0.05;
/// A new dot starts this far round from the last (`S:1463`).
const NEW_DOT_TURN: f32 = 48.0;

/// What an arrow key does to a handle's dot.
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub(super) enum Nudge {
    /// Left: hue -5 degrees, wrapping.
    HueDown,
    /// Right: hue +5 degrees, wrapping.
    HueUp,
    /// Up: chroma +.05, clamped.
    ChromaUp,
    /// Down: chroma -.05, clamped.
    ChromaDown,
}

impl Nudge {
    /// The nudge an arrow key asks for; other keys ask for none.
    pub(super) fn of(key: &Key) -> Option<Nudge> {
        match key {
            Key::ArrowLeft => Some(Nudge::HueDown),
            Key::ArrowRight => Some(Nudge::HueUp),
            Key::ArrowUp => Some(Nudge::ChromaUp),
            Key::ArrowDown => Some(Nudge::ChromaDown),
            _ => None,
        }
    }

    fn apply(self, dot: Dot) -> Dot {
        match self {
            Nudge::HueDown => Dot {
                hue: (dot.hue + 360.0 - HUE_STEP).rem_euclid(360.0),
                ..dot
            },
            Nudge::HueUp => Dot {
                hue: (dot.hue + HUE_STEP).rem_euclid(360.0),
                ..dot
            },
            Nudge::ChromaUp => Dot {
                chroma: (dot.chroma + CHROMA_STEP).clamp(0.0, 1.0),
                ..dot
            },
            Nudge::ChromaDown => Dot {
                chroma: (dot.chroma - CHROMA_STEP).clamp(0.0, 1.0),
                ..dot
            },
        }
    }
}

/// `look` with dot `index` replaced by `dot`; an index past the end changes nothing.
pub(super) fn moved(look: &SpaceLook, index: usize, dot: Dot) -> SpaceLook {
    let dots = look
        .dots
        .iter()
        .enumerate()
        .map(|(at, old)| if at == index { dot } else { *old })
        .collect();
    SpaceLook {
        dots,
        ..look.clone()
    }
}

/// `look` with dot `index` nudged by an arrow key.
pub(super) fn nudged(look: &SpaceLook, index: usize, nudge: Nudge) -> SpaceLook {
    match look.dots.get(index) {
        Some(dot) => moved(look, index, nudge.apply(*dot)),
        None => look.clone(),
    }
}

/// `look` with one more dot, 48 degrees round from the last at its chroma; unchanged at three.
pub(super) fn added(look: &SpaceLook) -> SpaceLook {
    let (Some(last), true) = (look.dots.last(), look.dots.len() < MAX_DOTS) else {
        return look.clone();
    };
    let mut dots = look.dots.clone();
    dots.push(Dot {
        hue: (last.hue + NEW_DOT_TURN).rem_euclid(360.0),
        chroma: last.chroma,
    });
    SpaceLook {
        dots,
        ..look.clone()
    }
}

/// `look` without dot `index`; the last dot is never removed.
pub(super) fn removed(look: &SpaceLook, index: usize) -> SpaceLook {
    if look.dots.len() <= 1 {
        return look.clone();
    }
    let dots = look
        .dots
        .iter()
        .enumerate()
        .filter(|&(at, _)| at != index)
        .map(|(_, dot)| *dot)
        .collect();
    SpaceLook {
        dots,
        ..look.clone()
    }
}

/// `look` with preset `index`'s dots; grain, theme and accent stay (`S:1471`).
pub(super) fn preset(look: &SpaceLook, index: usize) -> SpaceLook {
    match PRESETS.get(index) {
        Some(preset) => SpaceLook {
            dots: preset.dots.to_vec(),
            ..look.clone()
        },
        None => look.clone(),
    }
}

/// The grain a slider fraction stands for: thousandths to 0..=100, rounded.
pub(super) fn grain_of(fraction: Fraction) -> u8 {
    // At most 1000 after the clamp, so the quotient is at most 100.
    u8::try_from((fraction.clamped().0 + 5) / 10).unwrap_or(100)
}

#[cfg(test)]
mod tests {
    use super::{Nudge, added, grain_of, nudged, preset, removed};
    use crate::components::vocab::Fraction;
    use crate::space::{Dot, PRESETS, SpaceLook};

    fn look(dots: &[(f32, f32)]) -> SpaceLook {
        SpaceLook {
            dots: dots
                .iter()
                .map(|&(hue, chroma)| Dot { hue, chroma })
                .collect(),
            ..SpaceLook::default()
        }
    }

    fn pairs(look: &SpaceLook) -> Vec<(f32, f32)> {
        look.dots
            .iter()
            .map(|dot| {
                (
                    (dot.hue * 100.0).round() / 100.0,
                    (dot.chroma * 100.0).round() / 100.0,
                )
            })
            .collect()
    }

    #[test]
    fn arrows_step_hue_round_and_chroma_up_to_the_ends() {
        const CASES: &[(f32, f32, Nudge, (f32, f32))] = &[
            (2.0, 0.5, Nudge::HueDown, (357.0, 0.5)),
            (358.0, 0.5, Nudge::HueUp, (3.0, 0.5)),
            (100.0, 0.98, Nudge::ChromaUp, (100.0, 1.0)),
            (100.0, 0.02, Nudge::ChromaDown, (100.0, 0.0)),
            (100.0, 0.5, Nudge::ChromaUp, (100.0, 0.55)),
        ];
        for &(hue, chroma, nudge, want) in CASES {
            let got = nudged(&look(&[(hue, chroma)]), 0, nudge);
            assert_eq!(pairs(&got), vec![want], "{hue} {chroma} {nudge:?}");
        }
    }

    #[test]
    fn a_new_dot_turns_48_degrees_and_stops_at_three() {
        let one = look(&[(340.0, 0.6)]);
        let two = added(&one);
        assert_eq!(pairs(&two), vec![(340.0, 0.6), (28.0, 0.6)]);
        let three = added(&two);
        assert_eq!(three.dots.len(), 3);
        assert_eq!(added(&three), three, "no fourth dot");
    }

    #[test]
    fn the_last_dot_stays() {
        let two = look(&[(10.0, 0.5), (20.0, 0.5)]);
        assert_eq!(pairs(&removed(&two, 0)), vec![(20.0, 0.5)]);
        let one = look(&[(10.0, 0.5)]);
        assert_eq!(removed(&one, 0), one);
    }

    #[test]
    fn a_preset_replaces_only_the_dots() {
        let before = SpaceLook {
            grain: crate::space::Grain(80),
            ..look(&[(10.0, 0.5)])
        };
        let after = preset(&before, 1);
        assert_eq!(after.dots, PRESETS[1].dots);
        assert_eq!(after.grain, before.grain);
    }

    #[test]
    fn grain_rounds_from_thousandths() {
        const CASES: &[(u16, u8)] = &[
            (0, 0),
            (350, 35),
            (354, 35),
            (355, 36),
            (1000, 100),
            (4000, 100),
        ];
        for &(permille, want) in CASES {
            assert_eq!(grain_of(Fraction(permille)), want, "{permille}");
        }
    }
}
