//! The titlebar's press, as a pure machine: a press on its empty area becomes a move once the
//! pointer has travelled past the threshold, and asks for it exactly once; a press that never
//! travels is a click (the second of two is the double-click that zooms).

use crate::geometry::{Point, Px};

/// Where a titlebar press is.
#[derive(Debug, Clone, Copy, PartialEq, Default)]
pub(crate) enum Grab {
    /// No press on the titlebar.
    #[default]
    Idle,
    /// Pressed at `from`, not yet moved past the threshold.
    Pressed {
        /// Where the button went down.
        from: Point,
    },
    /// The move has been asked for; the compositor has the pointer until the release.
    Moving,
}

/// What a step asks of the host.
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub(crate) enum GrabEffect {
    /// Nothing.
    None,
    /// Start the interactive move.
    BeginMove,
}

impl Grab {
    /// The primary button went down on the titlebar at `at`.
    pub(crate) fn down(at: Point) -> Self {
        Grab::Pressed { from: at }
    }

    /// The pointer is at `at`: past `threshold` on either axis from the press, the move begins.
    pub(crate) fn moved(self, at: Point, threshold: Px) -> (Self, GrabEffect) {
        match self {
            Grab::Pressed { from } if travelled(from, at) > threshold.0 => {
                (Grab::Moving, GrabEffect::BeginMove)
            }
            other => (other, GrabEffect::None),
        }
    }
}

/// The larger of the two axes' travel.
fn travelled(from: Point, at: Point) -> f32 {
    (at.x.0 - from.x.0).abs().max((at.y.0 - from.y.0).abs())
}

#[cfg(test)]
mod tests {
    use super::{Grab, GrabEffect};
    use crate::geometry::{Point, Px};

    fn at(x: f32, y: f32) -> Point {
        Point { x: Px(x), y: Px(y) }
    }

    #[test]
    fn a_press_moves_once_past_the_threshold() {
        const CASES: &[(&str, &[(f32, f32)], usize)] = &[
            ("still", &[], 0),
            ("within 4", &[(3.0, 0.0), (4.0, 4.0)], 0),
            ("past 4 across", &[(5.0, 0.0)], 1),
            ("past 4 down", &[(0.0, -4.5)], 1),
            (
                "keeps moving",
                &[(2.0, 0.0), (6.0, 0.0), (20.0, 3.0), (40.0, 9.0)],
                1,
            ),
        ];
        for (name, path, moves) in CASES {
            let mut grab = Grab::down(at(0.0, 0.0));
            let mut asked = 0;
            for &(x, y) in *path {
                let (next, effect) = grab.moved(at(x, y), Px(4.0));
                grab = next;
                asked += usize::from(effect == GrabEffect::BeginMove);
            }
            assert_eq!(asked, *moves, "{name}");
        }
    }

    #[test]
    fn no_press_never_moves() {
        let (grab, effect) = Grab::Idle.moved(at(50.0, 50.0), Px(4.0));
        assert_eq!((grab, effect), (Grab::Idle, GrabEffect::None));
    }
}
