//! The pure transition of the orb's turn.

use super::model::Turn;
use std::time::Duration;

/// Where the glows are `elapsed` after they stood at `from`, turning once per `period`. A
/// period of nothing holds them where they are.
pub(crate) fn turn_after(from: Turn, elapsed: Duration, period: Duration) -> Turn {
    if period.is_zero() {
        return from;
    }
    let through = elapsed.as_nanos() % period.as_nanos();
    let advance = through * u128::from(Turn::FULL) / period.as_nanos();
    // `advance` is below `Turn::FULL`, so it fits.
    let advance = u32::try_from(advance).unwrap_or_default();
    Turn((from.0 % Turn::FULL + advance) % Turn::FULL)
}
