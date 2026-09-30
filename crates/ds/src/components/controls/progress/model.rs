//! What a `ProgressIndicator` is and how far along it stands: data only.

use ds_core::vocab::Fraction;
use ds_core::word::Word;
use ds_motion::detail::operation::Operation;

/// How progress is drawn (`NSProgressIndicator` style, design/30 section 2.9).
#[derive(Debug, Clone, Copy, PartialEq, Eq, Hash, Default, Word)]
pub enum ProgressStyle {
    /// A horizontal capsule that fills, or a barber pole while the work has no end in sight.
    #[default]
    Bar,
    /// Twelve spokes, the lit one going round: only for work with no known end.
    Spinner,
    /// A circular arc from twelve o'clock, clockwise.
    Ring,
}

/// How far the work has come: a share, or an operation without one.
///
/// `Unknown` carries the [`Operation`] that drives the loop: a spinner or a barber pole exists
/// only while one runs, and a loop cannot be written without it (design/26-DETAILS.md R4).
#[derive(Debug, Clone, Copy, PartialEq, Eq, Hash)]
pub enum Progress {
    /// Determinate: this share of the work is done. A change tweens linearly over `--t-move`;
    /// nothing sweeps in on first show.
    Known(Fraction),
    /// Indeterminate: turns at once while `Running`, shows nothing while `Idle`.
    Unknown(Operation),
}

impl Progress {
    /// `aria-valuenow` on the 0..=100 scale the markup declares; `None` for an unknown amount.
    pub(crate) fn valuenow(self) -> Option<u16> {
        match self {
            Progress::Known(share) => Some(share.whole_percent()),
            Progress::Unknown(_) => None,
        }
    }
}

#[cfg(test)]
mod tests {
    use super::Progress;
    use ds_core::vocab::Fraction;
    use ds_motion::detail::operation::{Operation, PendingToken};

    #[test]
    fn only_a_known_share_reports_a_value() {
        let cases = [
            (Progress::Known(Fraction(0)), Some(0)),
            (Progress::Known(Fraction(496)), Some(50)),
            (Progress::Known(Fraction(1000)), Some(100)),
            (Progress::Known(Fraction(4000)), Some(100)),
            (Progress::Unknown(Operation::Idle), None),
            (
                Progress::Unknown(Operation::Running(PendingToken::start())),
                None,
            ),
        ];
        for (progress, want) in cases {
            assert_eq!(progress.valuenow(), want, "{progress:?}");
        }
    }
}
