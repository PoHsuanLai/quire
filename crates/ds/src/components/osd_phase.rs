//! The OSD card's presence as a pure machine (sill FINDINGS Q76): what a change of `shown` or a
//! settled animation does to the card, and what the component must do about it. The component
//! (`osd.rs`) owns the timers and runs the effects; everything that decides is here, so the rules
//! are a table.

use crate::components::tooltip::Shown;

/// Where the card is in its life: `data-presence` while it is drawn, `data-shown="hidden"` once it
/// has gone.
#[derive(Debug, Clone, Copy, PartialEq, Eq, Hash)]
pub(crate) enum OsdPhase {
    /// Not drawn: before its first showing, and after its exit has settled.
    Hidden,
    /// Playing `osd-in`.
    Entering,
    /// At rest, fully shown.
    Present,
    /// Playing `osd-out`; hidden when it settles.
    Leaving,
}

impl OsdPhase {
    /// The `data-presence` value, or `None` while hidden.
    pub(crate) fn presence(self) -> Option<&'static str> {
        match self {
            OsdPhase::Hidden => None,
            OsdPhase::Entering => Some("entering"),
            OsdPhase::Present => Some("present"),
            OsdPhase::Leaving => Some("leaving"),
        }
    }

    /// The `data-shown` value: whether anything is drawn.
    pub(crate) fn shown(self) -> Shown {
        match self {
            OsdPhase::Hidden => Shown::Hidden,
            OsdPhase::Entering | OsdPhase::Present | OsdPhase::Leaving => Shown::Visible,
        }
    }
}

/// What happened to the card.
#[derive(Debug, Clone, Copy, PartialEq, Eq, Hash)]
pub(crate) enum OsdInput {
    /// The caller shows it.
    Show,
    /// The caller hides it.
    Hide,
    /// `osd-in` settled.
    InSettled,
    /// `osd-out` settled.
    OutSettled,
}

/// What the component does after a step.
#[derive(Debug, Clone, Copy, PartialEq, Eq, Hash)]
pub(crate) enum OsdEffect {
    /// Nothing.
    None,
    /// Play `osd-in` from its first frame (swap the keyframe alias) and start its timer.
    PlayIn,
    /// Start `osd-out`'s timer.
    PlayOut,
    /// Stop `osd-out`'s timer: the hide was taken back.
    CancelOut,
    /// Tell the caller the card has gone (`on_hidden`), so it can unmap the surface.
    Gone,
}

/// The input a render's `shown` makes, from `last` (`None` before the first render). A card
/// mounted shown plays its entrance; one mounted hidden stays hidden and says nothing.
pub(crate) fn input(last: Option<Shown>, now: Shown) -> Option<OsdInput> {
    match (last, now) {
        (None | Some(Shown::Hidden), Shown::Visible) => Some(OsdInput::Show),
        (Some(Shown::Visible), Shown::Hidden) => Some(OsdInput::Hide),
        (Some(Shown::Visible), Shown::Visible) | (None | Some(Shown::Hidden), Shown::Hidden) => {
            None
        }
    }
}

/// One step. A show while the card fades out takes the hide back: the card is present again at
/// once (no second entrance: it never left) and `on_hidden` never comes for that hide. A settle
/// that arrives after its motion was overtaken changes nothing.
pub(crate) fn step(phase: OsdPhase, input: OsdInput) -> (OsdPhase, OsdEffect) {
    use OsdEffect as E;
    use OsdInput as I;
    use OsdPhase as P;
    match (phase, input) {
        (P::Hidden, I::Show) => (P::Entering, E::PlayIn),
        (P::Entering | P::Present, I::Hide) => (P::Leaving, E::PlayOut),
        (P::Leaving, I::Show) => (P::Present, E::CancelOut),
        (P::Entering, I::InSettled) => (P::Present, E::None),
        (P::Leaving, I::OutSettled) => (P::Hidden, E::Gone),
        (P::Entering | P::Present, I::Show)
        | (P::Hidden | P::Leaving, I::Hide)
        | (P::Hidden | P::Present | P::Leaving, I::InSettled)
        | (P::Hidden | P::Entering | P::Present, I::OutSettled) => (phase, E::None),
    }
}

#[cfg(test)]
mod tests {
    use super::{OsdEffect as E, OsdInput as I, OsdPhase as P, input, step};
    use crate::components::tooltip::Shown;

    #[test]
    fn each_input_moves_the_card() {
        #[rustfmt::skip]
        let cases = [
            (P::Hidden, I::Show, P::Entering, E::PlayIn),
            (P::Entering, I::InSettled, P::Present, E::None),
            (P::Present, I::Hide, P::Leaving, E::PlayOut),
            (P::Leaving, I::OutSettled, P::Hidden, E::Gone),
            // Hidden while still coming in: it goes from where it is.
            (P::Entering, I::Hide, P::Leaving, E::PlayOut),
            // Shown again while fading out: the hide is taken back.
            (P::Leaving, I::Show, P::Present, E::CancelOut),
            // Late settles and repeats change nothing.
            (P::Leaving, I::InSettled, P::Leaving, E::None),
            (P::Present, I::OutSettled, P::Present, E::None),
            (P::Present, I::Show, P::Present, E::None),
            (P::Hidden, I::Hide, P::Hidden, E::None),
            (P::Hidden, I::OutSettled, P::Hidden, E::None),
        ];
        for (from, happened, to, effect) in cases {
            assert_eq!(
                step(from, happened),
                (to, effect),
                "{from:?} + {happened:?}"
            );
        }
    }

    #[test]
    fn only_a_change_of_shown_is_an_input() {
        #[rustfmt::skip]
        let cases = [
            (None, Shown::Visible, Some(I::Show)),
            (None, Shown::Hidden, None),
            (Some(Shown::Hidden), Shown::Visible, Some(I::Show)),
            (Some(Shown::Visible), Shown::Hidden, Some(I::Hide)),
            (Some(Shown::Visible), Shown::Visible, None),
            (Some(Shown::Hidden), Shown::Hidden, None),
        ];
        for (last, now, want) in cases {
            assert_eq!(input(last, now), want, "{last:?} -> {now:?}");
        }
    }

    #[test]
    fn only_a_drawn_card_has_a_presence() {
        assert_eq!(P::Hidden.presence(), None);
        assert_eq!(P::Hidden.shown(), Shown::Hidden);
        assert_eq!(P::Leaving.presence(), Some("leaving"));
        assert_eq!(P::Entering.shown(), Shown::Visible);
    }
}
