//! The gates every accent clears, and the steps the derivation takes toward them.
//!
//! Text is WCAG 2's 4.5:1 (the HIG's floor up to 17 pt; button labels, the today disc's number
//! and menu text are all under it, so the 3:1 large-text floor never applies to the ink). The
//! focus ring is non-text, 3:1 against its ground (WCAG 1.4.11). The wash is a fill behind
//! text: the card's ink must read on it at 4.5:1, and it must show at all, 1.15:1 off its
//! ground (a selection nobody can see is not one).

use crate::tokens::hex::Alpha;

/// Ink on the solid fill, the text accent on every card ground, the card's ink on the wash.
pub const TEXT: f64 = 4.5;

/// White ink on a Mac system fill: 3:1, the large-text floor (button labels are semibold);
/// system blue and red hold it with white, orange, green and teal take a dark ink.
pub const INK_ON_SYSTEM_FILL: f64 = 3.0;

/// The focus ring against every card ground.
pub const RING: f64 = 3.0;

/// How far the wash must stand off the ground it lies on.
pub const WASH_SHOWS: f64 = 1.15;

/// A lightness step: mailo's `ACCENT_STEP`.
pub const LIGHTNESS_STEP: f64 = 0.01;

/// The fill never steps darker than this, nor lighter than [`LIGHTEST`].
pub const DARKEST: f64 = 0.30;

/// See [`DARKEST`].
pub const LIGHTEST: f64 = 0.95;

/// How far the wash's alpha rises per step, and the most it may reach.
pub const WASH_STEP: Alpha = Alpha(10);

/// See [`WASH_STEP`].
pub const WASH_MOST: Alpha = Alpha(600);

/// How far the ring's alpha rises per step (it stops at opaque).
pub const RING_STEP: Alpha = Alpha(50);
