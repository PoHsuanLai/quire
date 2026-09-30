//! The blink a picked item makes before its menu closes (design/30 section 1.3, emphasis): the
//! highlight goes out and comes back twice, each flash `MenuBlink` long, and only then does the
//! menu act on the pick. Pure: the phases and what each one draws.

use ds_style::tokens::delay::DelayToken;
use std::time::Duration;

/// Where a menu's blink stands.
#[derive(Debug, Clone, Copy, PartialEq, Eq, Default)]
pub(crate) enum Blink {
    /// No pick has been made.
    #[default]
    Idle,
    /// The picked item of the panel at `depth` is drawn without its highlight.
    Unlit { depth: u8 },
    /// The picked item of the panel at `depth` is drawn highlighted.
    Lit { depth: u8 },
    /// The blink is over: the menu acts on the pick.
    Done,
}

/// How many flashes a pick makes.
const FLASHES: u8 = 2;

/// How long each half of a flash lasts: a flash is `MenuBlink` long, out and back.
pub(crate) fn half() -> Duration {
    DelayToken::MenuBlink.delay() / 2
}

/// The phases of a pick made in the panel at `depth`, in order: out and back, `FLASHES` times.
pub(crate) fn phases(depth: u8) -> Vec<Blink> {
    (0..FLASHES)
        .flat_map(|_| [Blink::Unlit { depth }, Blink::Lit { depth }])
        .chain(std::iter::once(Blink::Done))
        .collect()
}

/// Whether the highlight of the panel at `depth` is drawn: always, except while its item is out.
pub(crate) fn highlighted(blink: Blink, depth: u8) -> bool {
    !matches!(blink, Blink::Unlit { depth: out } if out == depth)
}

/// Whether a pick has been made, so nothing else is picked.
pub(crate) fn picked(blink: Blink) -> bool {
    !matches!(blink, Blink::Idle)
}

#[cfg(test)]
mod tests {
    use super::{Blink, half, highlighted, phases, picked};
    use ds_style::tokens::delay::DelayToken;

    #[test]
    fn a_pick_flashes_twice_then_acts() {
        assert_eq!(
            phases(1),
            [
                Blink::Unlit { depth: 1 },
                Blink::Lit { depth: 1 },
                Blink::Unlit { depth: 1 },
                Blink::Lit { depth: 1 },
                Blink::Done,
            ]
        );
        assert_eq!(half() * 2, DelayToken::MenuBlink.delay());
    }

    #[test]
    fn only_the_picked_panels_highlight_goes_out() {
        const CASES: &[(Blink, u8, bool)] = &[
            (Blink::Idle, 0, true),
            (Blink::Unlit { depth: 1 }, 1, false),
            (Blink::Unlit { depth: 1 }, 0, true),
            (Blink::Lit { depth: 1 }, 1, true),
            (Blink::Done, 1, true),
        ];
        for &(blink, depth, want) in CASES {
            assert_eq!(highlighted(blink, depth), want, "{blink:?} at {depth}");
        }
        assert!(!picked(Blink::Idle));
        assert!(picked(Blink::Lit { depth: 0 }));
    }
}
