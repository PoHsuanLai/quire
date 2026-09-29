//! What the editor measures: the Space's text and accent against what they are drawn on.
//!
//! Every number here is `contrast::ratio` of a colour [`derive`] produced, or of the Post
//! card's own literals. The editor shows these and computes nothing of its own.
//!
//! A pass or fail is a [`Verdict`] (design/03-COLOR.md section 6).

use super::{card, derive};
use crate::appearance::{Accent, Scheme};
use crate::space::contrast::{Verdict, ratio};
use crate::space::look::{CardAccent, SpaceLook};
use crate::tokens::accent_band::over;
use crate::tokens::{Hex, accent_of};

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
/// Sidebar ink and faint text on every stop of the frame, the card's accent text on the card,
/// and the card's ink on the accent's wash laid over the card (the wash is translucent, so it
/// is composited before it is measured). With [`CardAccent::Postmark`] the last two measure
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
    let roles = match space.card_accent {
        CardAccent::SpaceHue => palette.accent_roles,
        CardAccent::Postmark => accent_of(Accent::Postmark, scheme),
    };
    let wash = Hex::parse(post.surface)
        .map(|surface| over(roles.fill, roles.wash, surface).css())
        .unwrap_or_default();
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
            measured: ratio(&roles.text.css(), post.surface).unwrap_or(0.0),
            need: 4.5,
        },
        ContrastCheck {
            label: "Ink on the accent tint",
            measured: ratio(post.ink, &wash).unwrap_or(0.0),
            need: 4.5,
        },
    ]
}
