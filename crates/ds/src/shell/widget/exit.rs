//! A widget card's exit (design/05 principle 3 and section 8, "acting gets an exit"):
//! a desktop widget the person removes in Edit Widgets used to vanish between one frame and the
//! next. Its host now says the card is leaving (`CardPresence::Leaving`) and keeps it drawn;
//! the card shrinks and fades with `widget-out` (`--t-move --e-exit`, a fade alone under
//! Reduced), holds its last, transparent frame, and calls the host's `on_gone` once at
//! `settle(WidgetOut)`, when the host drops it. Taken back before then (`Placed` again), it stops
//! and plays `hold`, so the restyle that drops the exit leaves no half-faded value behind
//!. Independent of any list: a card on the desktop is not a row.

use ds_motion::anim::Anim;
use ds_motion::timer::use_motion_timer;
use dioxus::prelude::*;

/// Whether a placed widget's card stays or is on its way out.
#[derive(Debug, Clone, Copy, PartialEq, Eq, Hash, Default)]
pub enum CardPresence {
    /// Placed: drawn at rest.
    #[default]
    Placed,
    /// Removed by its host, which keeps drawing it until `on_gone`: it plays its exit.
    Leaving,
}

/// What the card plays this render.
#[derive(Debug, Clone, Copy, PartialEq, Eq, Hash)]
pub(crate) enum CardMotion {
    /// Nothing: at rest.
    Rest,
    /// Its exit (`a-widget-out`).
    Leaving,
    /// A leave taken back: `hold`, which moves nothing, once.
    Back,
}

impl CardMotion {
    /// The pulse class and `data-pulse` it renders, if any.
    pub(crate) fn pulse(self) -> Option<&'static str> {
        match self {
            CardMotion::Rest => None,
            CardMotion::Leaving => Some(Anim::WidgetOut.class()),
            CardMotion::Back => Some(Anim::Hold.class()),
        }
    }

    /// `data-presence`: `leaving` while it goes.
    pub(crate) fn presence(self) -> Option<&'static str> {
        match self {
            CardMotion::Leaving => Some("leaving"),
            CardMotion::Rest | CardMotion::Back => None,
        }
    }
}

/// The card's motion for `presence`: its exit starts the render its host first says `Leaving`
/// (even a card mounted leaving) and `on_gone` runs once at `settle(WidgetOut)`; `Placed` again
/// before then stops the timer, so `on_gone` never runs for a card that stayed.
pub(crate) fn use_card_exit(
    presence: CardPresence,
    on_gone: Option<EventHandler<()>>,
) -> CardMotion {
    let timer = use_motion_timer(Anim::WidgetOut);
    let mut seen = use_hook(|| CopyValue::new((CardPresence::Placed, CardMotion::Rest)));
    let (was, motion) = *seen.peek();
    if was == presence {
        return motion;
    }
    let next = match presence {
        CardPresence::Leaving => {
            let gone = on_gone.unwrap_or_default();
            timer.start(EventHandler::new(move |()| gone.call(())));
            CardMotion::Leaving
        }
        CardPresence::Placed => {
            timer.cancel();
            CardMotion::Back
        }
    };
    seen.set((presence, next));
    next
}

#[cfg(test)]
mod tests {
    use super::CardMotion;

    #[test]
    fn only_a_leaving_card_says_so_and_a_taken_back_one_holds() {
        const CASES: &[(CardMotion, Option<&str>, Option<&str>)] = &[
            (CardMotion::Rest, None, None),
            (CardMotion::Leaving, Some("a-widget-out"), Some("leaving")),
            (CardMotion::Back, Some("a-hold"), None),
        ];
        for &(motion, pulse, presence) in CASES {
            assert_eq!(motion.pulse(), pulse, "{motion:?}");
            assert_eq!(motion.presence(), presence, "{motion:?}");
        }
    }
}
