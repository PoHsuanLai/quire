//! A wheel's detents eased over frames for the listeners that asked for it
//! (`ds::host::gesture::WheelDelivery::Eased`): one smooth target per axis
//! (`blitz_kit::scroll::target`). Pure.

use blitz_kit::scroll::engine::Motion;
use blitz_kit::scroll::target::Target;
use blitz_kit::scroll::time::Elapsed;

/// The distance still to deliver on each axis, in winit's sign.
#[derive(Debug, Clone, Copy, PartialEq, Default)]
pub(super) struct Eased {
    x: Target,
    y: Target,
}

impl Eased {
    /// Add `(x, y)` px to the targets at `now`.
    pub(super) fn push(self, x: f64, y: f64, now: Elapsed) -> Eased {
        Eased {
            x: push_axis(self.x, x, now),
            y: push_axis(self.y, y, now),
        }
    }

    /// The distance to deliver at `now` on each axis, and what is left.
    pub(super) fn advance(self, now: Elapsed) -> (Eased, (f64, f64)) {
        let (x, dx) = self.x.advance(now);
        let (y, dy) = self.y.advance(now);
        (Eased { x, y }, (dx, dy))
    }

    /// Whether a frame is wanted.
    pub(super) fn motion(&self) -> Motion {
        match (self.x.motion(), self.y.motion()) {
            (Motion::Still, Motion::Still) => Motion::Still,
            _ => Motion::Animating,
        }
    }
}

/// `target` with `by` added, unchanged for no motion.
fn push_axis(target: Target, by: f64, now: Elapsed) -> Target {
    match by == 0.0 {
        true => target,
        false => target.push(by, now),
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use std::time::Duration;

    fn at(ms: u64) -> Elapsed {
        Elapsed(Duration::from_millis(ms))
    }

    #[test]
    fn an_axis_that_was_not_pushed_stays_put() {
        let eased = Eased::default().push(0.0, 60.0, at(0));
        let (_, (dx, dy)) = eased.advance(at(30));
        assert_eq!(dx, 0.0);
        assert!(dy > 0.0 && dy < 60.0, "{dy}");
    }

    #[test]
    fn the_frames_sum_to_the_push_and_then_stop() {
        let mut eased = Eased::default().push(-60.0, 60.0, at(0));
        let mut sum = (0.0, 0.0);
        for ms in (8..=100).step_by(8) {
            let (next, (dx, dy)) = eased.advance(at(ms));
            eased = next;
            sum = (sum.0 + dx, sum.1 + dy);
        }
        assert!(
            (sum.0 + 60.0).abs() < 1e-9 && (sum.1 - 60.0).abs() < 1e-9,
            "{sum:?}"
        );
        assert_eq!(eased.motion(), Motion::Still);
    }
}
