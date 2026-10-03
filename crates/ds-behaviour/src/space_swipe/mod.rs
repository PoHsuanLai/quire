//! The live Space swipe (design/12 §12.3.7; 03 §4.2): the desktop follows the fingers between
//! Spaces, rubber-bands past the first and last, and on lift finishes to the Space the distance
//! and speed commit to, one Space per gesture. A keyboard or intent `Go` slides through the same
//! `Finishing` state, so a swipe can catch a slide in flight and a `Go` redirects one (03 §1.5).
//!
//! Positions are integer thousandths of a page ([`PageMilli`]) along the display's row of Spaces,
//! growing toward later Spaces: Space `k` sits at `k × 1000`. The caller draws the row at each
//! [`SwipeOut::Offset`] and, while [`Swipe::Finishing`], samples the ease-out-cubic finish itself
//! from the state; the machine wakes once, at the finish's end.

mod model;
mod numbers;
mod step;
#[cfg(test)]
mod tests;

pub use model::{
    PageMilli, PagesPerSecMilli, Reduced, Samples, Swipe, SwipeIn, SwipeOut, SwipeParams,
    SwipeSource,
};
pub use numbers::{commit, finish_duration, rubber};
