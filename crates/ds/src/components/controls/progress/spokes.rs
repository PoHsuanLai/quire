//! The spoke spinner's arithmetic (design/30 section 1.3, Pending loop): twelve spokes, the lit
//! one going round one step every `--t-spin-step`, the ones behind it fading.

use ds_motion::detail::pending::{
    PendingFrame, PendingLayers, PendingSpec, PendingStyle, SPIN_STEPS,
};

/// The loop an indicator plays: one ring, a twelfth of a turn a step.
pub(crate) const SPIN: PendingSpec = PendingSpec {
    style: PendingStyle::Spin,
    layers: PendingLayers(1),
};

/// How many spokes behind the head `spoke` stands in `frame`: 0 is the lit head, 11 the faintest.
/// An idle loop has no head; every spoke is then at its resting age.
pub(crate) fn age(frame: PendingFrame, spoke: u8) -> u8 {
    match frame {
        PendingFrame::Idle => SPIN_STEPS - 1,
        PendingFrame::Step(step) => (step + SPIN_STEPS - spoke % SPIN_STEPS) % SPIN_STEPS,
    }
}

/// The step written as `--step`, for a barber pole that slides one step at a time.
pub(crate) fn step_of(frame: PendingFrame) -> Option<u8> {
    match frame {
        PendingFrame::Idle => None,
        PendingFrame::Step(step) => Some(step),
    }
}

#[cfg(test)]
mod tests {
    use super::{age, step_of};
    use ds_motion::detail::pending::PendingFrame;

    #[test]
    fn the_head_leads_and_the_tail_fades_behind_it() {
        // (frame, spoke, age): the head is the spoke the step names, the one before it is 1.
        const CASES: &[(PendingFrame, u8, u8)] = &[
            (PendingFrame::Step(0), 0, 0),
            (PendingFrame::Step(0), 1, 11),
            (PendingFrame::Step(0), 11, 1),
            (PendingFrame::Step(5), 5, 0),
            (PendingFrame::Step(5), 4, 1),
            (PendingFrame::Step(5), 6, 11),
            (PendingFrame::Idle, 3, 11),
        ];
        for &(frame, spoke, want) in CASES {
            assert_eq!(age(frame, spoke), want, "{frame:?} spoke {spoke}");
        }
    }

    #[test]
    fn only_a_running_loop_has_a_step() {
        assert_eq!(step_of(PendingFrame::Idle), None);
        assert_eq!(step_of(PendingFrame::Step(7)), Some(7));
    }
}
