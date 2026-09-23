//! What the editor measures: the Space's text and accent against what they are drawn on.
//!
//! Every number here is `contrast::ratio` of a colour [`derive`] produced, or of the Post
//! card's own literals. The editor shows these and computes nothing of its own.
//!
//! Moved from mailo (`mail-app/src/palette/readout.rs`); `Space` became [`SpaceLook`] and the
//! `bool` predicate became a [`Verdict`] (design/03-COLOR.md section 6).

use super::{card, derive};
use crate::appearance::Scheme;
use crate::space::contrast::{Verdict, ratio};
use crate::space::look::{CardAccent, SpaceLook};

/// One measured pair, and the floor it has to clear.
#[derive(Debug, Clone, PartialEq)]
pub struct ContrastCheck {
    /// What was measured, in the editor's words.
    pub label: &'static str,
    /// The contrast ratio. The worst stop, where a pair is drawn on several.
    pub measured: f64,
    /// The ratio it has to reach.
    pub need: f64,
}

impl ContrastCheck {
    /// Whether the pair clears its floor.
    pub fn verdict(&self) -> Verdict {
        if self.measured >= self.need {
            Verdict::Pass
        } else {
            Verdict::Fail
        }
    }
}

/// The four pairs the plan's sweep holds every pick to, for `space` in one scheme.
///
/// Sidebar ink and faint text on every stop of the frame, the card's accent on the card, and
/// the card's ink on the accent's tint. With [`CardAccent::Postmark`] the last two measure
/// Postmark, since that is what the card then wears. A colour that is not a hex pair
/// measures 0, which fails, rather than being left out.
pub fn readout(space: &SpaceLook, scheme: Scheme) -> Vec<ContrastCheck> {
    let palette = derive(&space.dots, scheme);
    let post = card(scheme);
    let worst = |fore: &str| {
        palette
            .stops
            .iter()
            .map(|stop| ratio(fore, stop).unwrap_or(0.0))
            .fold(f64::INFINITY, f64::min)
    };
    let (accent, accent_soft) = match space.card_accent {
        CardAccent::SpaceHue => (palette.accent.as_str(), palette.accent_soft.as_str()),
        CardAccent::Postmark => (post.accent, post.accent_soft),
    };
    vec![
        ContrastCheck {
            label: "Sidebar text on the colour",
            measured: worst(&palette.ink),
            need: 4.5,
        },
        ContrastCheck {
            label: "Faint text on the colour",
            measured: worst(&palette.faint),
            need: 3.0,
        },
        ContrastCheck {
            label: "Accent on the card",
            measured: ratio(accent, post.surface).unwrap_or(0.0),
            need: 4.5,
        },
        ContrastCheck {
            label: "Ink on the accent tint",
            measured: ratio(post.ink, accent_soft).unwrap_or(0.0),
            need: 4.5,
        },
    ]
}
