//! A surface's presence as a pure machine: what a change of `shown` or a settled animation does
//! to it, and what `use_presence` must do about it. The hook owns the timers and runs the
//! effects; everything that decides is here, so the rules are a table.

use super::{Exit, Presence};
use ds_core::vocab::Shown;

/// What happened to the surface.
#[derive(Debug, Clone, Copy, PartialEq, Eq, Hash)]
pub(crate) enum PresenceInput {
    /// The caller shows it.
    Show,
    /// The caller hides it.
    Hide,
    /// The entrance settled.
    InSettled,
    /// The exit settled.
    OutSettled,
}

/// What the hook does after a step.
#[derive(Debug, Clone, Copy, PartialEq, Eq, Hash)]
pub(crate) enum PresenceEffect {
    /// Nothing.
    None,
    /// Play the entrance from its first frame (swap the keyframe alias) and start its timer.
    PlayIn,
    /// Start the exit's timer.
    PlayOut,
    /// Stop the exit's timer: the hide was taken back.
    CancelOut,
    /// Tell the caller the surface has gone (`on_hidden`), so it can unmap it.
    Gone,
}

/// The input a render's `shown` makes, from `last` (`None` before the first render). A surface
/// mounted shown plays its entrance; one mounted hidden stays hidden and says nothing.
pub(crate) fn input(last: Option<Shown>, now: Shown) -> Option<PresenceInput> {
    match (last, now) {
        (None | Some(Shown::Hidden), Shown::Visible) => Some(PresenceInput::Show),
        (Some(Shown::Visible), Shown::Hidden) => Some(PresenceInput::Hide),
        (Some(Shown::Visible), Shown::Visible) | (None | Some(Shown::Hidden), Shown::Hidden) => {
            None
        }
    }
}

/// One step; a hide leaves by `exit`. A show while the surface fades out takes the hide back:
/// it is present again at once (no second entrance: it never left) and `on_hidden` never comes
/// for that hide. A settle that arrives after its motion was overtaken changes nothing.
pub(crate) fn step(
    presence: Presence,
    input: PresenceInput,
    exit: Exit,
) -> (Presence, PresenceEffect) {
    use Presence as P;
    use PresenceEffect as E;
    use PresenceInput as I;
    match (presence, input) {
        (P::Hidden, I::Show) => (P::Entering, E::PlayIn),
        (P::Entering | P::Present, I::Hide) => (P::Leaving(exit), E::PlayOut),
        (P::Leaving(_), I::Show) => (P::Present, E::CancelOut),
        (P::Entering, I::InSettled) => (P::Present, E::None),
        (P::Leaving(_), I::OutSettled) => (P::Hidden, E::Gone),
        (P::Entering | P::Present, I::Show)
        | (P::Hidden | P::Leaving(_), I::Hide)
        | (P::Hidden | P::Present | P::Leaving(_), I::InSettled)
        | (P::Hidden | P::Entering | P::Present, I::OutSettled) => (presence, E::None),
    }
}

#[cfg(test)]
mod tests {
    use super::{Presence as P, PresenceEffect as E, PresenceInput as I, input, step};
    use crate::motion::presence::Exit;
    use ds_core::vocab::Shown;

    const OUT: P = P::Leaving(Exit::OsdOut);

    #[test]
    fn each_input_moves_the_surface() {
        #[rustfmt::skip]
        let cases = [
            (P::Hidden, I::Show, P::Entering, E::PlayIn),
            (P::Entering, I::InSettled, P::Present, E::None),
            (P::Present, I::Hide, OUT, E::PlayOut),
            (OUT, I::OutSettled, P::Hidden, E::Gone),
            // Hidden while still coming in: it goes from where it is.
            (P::Entering, I::Hide, OUT, E::PlayOut),
            // Shown again while fading out: the hide is taken back.
            (OUT, I::Show, P::Present, E::CancelOut),
            // Late settles and repeats change nothing.
            (OUT, I::InSettled, OUT, E::None),
            (P::Present, I::OutSettled, P::Present, E::None),
            (P::Present, I::Show, P::Present, E::None),
            (P::Hidden, I::Hide, P::Hidden, E::None),
            (P::Hidden, I::OutSettled, P::Hidden, E::None),
        ];
        for (from, happened, to, effect) in cases {
            assert_eq!(
                step(from, happened, Exit::OsdOut),
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
    fn only_a_drawn_surface_has_a_presence() {
        assert_eq!(P::Hidden.drawn_slug(), None);
        assert_eq!(P::Hidden.shown(), Shown::Hidden);
        assert_eq!(OUT.drawn_slug(), Some("leaving"));
        assert_eq!(P::Entering.shown(), Shown::Visible);
    }
}
