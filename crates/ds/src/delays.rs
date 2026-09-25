//! The hover-intent delays as plain constants, for tests (mailo gaps 7).
//!
//! A test that drives hover intent waits "the open delay plus slack" before asserting a card is
//! open, and `DelayToken::HoverOpen.delay(level)` is a method call on a level it has no reason
//! to name. These are the same values the tokens give (the hover delays do not follow the motion
//! level; a test below holds them equal at every level). Test-facing: components read the token,
//! never these.

use std::time::Duration;

/// A hover card opens after the pointer rests this long: `DelayToken::HoverOpen`.
pub const HOVER_OPEN: Duration = Duration::from_millis(450);

/// A hover card closes this long after the pointer leaves: `DelayToken::HoverClose`.
pub const HOVER_CLOSE: Duration = Duration::from_millis(150);

/// Cards and labels stay warm this long after a close: `DelayToken::HoverWarm`.
pub const HOVER_WARM: Duration = Duration::from_millis(400);

#[cfg(test)]
mod tests {
    use super::{HOVER_CLOSE, HOVER_OPEN, HOVER_WARM};
    use crate::appearance::MotionLevel;
    use crate::tokens::DelayToken;
    use std::time::Duration;

    const PAIRS: [(Duration, DelayToken); 3] = [
        (HOVER_OPEN, DelayToken::HoverOpen),
        (HOVER_CLOSE, DelayToken::HoverClose),
        (HOVER_WARM, DelayToken::HoverWarm),
    ];

    #[test]
    fn each_constant_is_its_token_at_every_level() {
        for level in MotionLevel::ALL {
            for (constant, token) in PAIRS {
                assert_eq!(constant, token.delay(level), "{token:?} at {level:?}");
            }
        }
    }
}
