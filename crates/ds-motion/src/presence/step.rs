//! A surface's life as a pure machine: what a change of `shown` or a settled animation does to it.
//! The settle deadlines are in the state, so the machine's wake is the one timer and every rule
//! is a table.

use super::Presence;
use super::model::{Life, PresenceIn, PresenceOut, PresenceParams};
use crate::settle::settle;
use ds_core::machine::Machine;
use ds_core::time::stamp::Stamp;
use ds_core::vocab::Shown;

/// The input a render's `shown` makes, from `last` (`None` before the first render). A surface
/// mounted shown plays its entrance; one mounted hidden stays hidden and says nothing.
pub fn change(last: Option<Shown>, now: Shown) -> Option<PresenceIn> {
    match (last, now) {
        (None | Some(Shown::Hidden), Shown::Visible) => Some(PresenceIn::Show),
        (Some(Shown::Visible), Shown::Hidden) => Some(PresenceIn::Hide),
        (Some(Shown::Visible), Shown::Visible) | (None | Some(Shown::Hidden), Shown::Hidden) => {
            None
        }
    }
}

impl Machine for Life {
    type In = PresenceIn;
    type Out = PresenceOut;
    type Params = PresenceParams;
    type Ctx = ();

    /// One step; a hide leaves by `params.exit`. A show while the surface fades out takes the
    /// hide back: it is present again at once (no second entrance: it never left) and `Gone`
    /// never comes for that hide. A wake that arrives after its motion was overtaken, or before
    /// it is due, changes nothing.
    fn step(
        self,
        input: PresenceIn,
        at: Stamp,
        params: &PresenceParams,
        _: &(),
    ) -> (Life, Vec<PresenceOut>) {
        use Presence as P;
        use PresenceIn as I;
        let settles = |anim| Some(at.after_span(settle(anim, params.motion)));
        let quiet = |life| (life, Vec::new());
        match (self.presence, input) {
            (P::Hidden, I::Show) => quiet(Life {
                presence: P::Entering,
                alias: self.alias.flipped(),
                until: settles(params.enter),
            }),
            (P::Entering | P::Present, I::Hide) => quiet(Life {
                presence: P::Leaving(params.exit),
                until: settles(params.exit.anim()),
                ..self
            }),
            (P::Leaving(_), I::Show) => quiet(Life {
                presence: P::Present,
                alias: super::EntranceAlias::Held,
                until: None,
            }),
            (P::Entering, I::Elapsed) if due(self.until, at) => quiet(Life {
                presence: P::Present,
                until: None,
                ..self
            }),
            (P::Leaving(_), I::Elapsed) if due(self.until, at) => (
                Life {
                    presence: P::Hidden,
                    until: None,
                    ..self
                },
                vec![PresenceOut::Gone],
            ),
            (P::Entering | P::Present, I::Show)
            | (P::Hidden | P::Leaving(_), I::Hide)
            | (P::Hidden | P::Present | P::Entering | P::Leaving(_), I::Elapsed) => quiet(self),
        }
    }

    fn wake(&self) -> Option<Stamp> {
        self.until
    }
}

/// Whether the deadline `until` has come at `at`.
fn due(until: Option<Stamp>, at: Stamp) -> bool {
    until.is_some_and(|until| at >= until)
}

#[cfg(test)]
mod tests {
    use super::{Life, Presence as P, PresenceIn as I, PresenceOut as O, PresenceParams, change};
    use crate::anim::Anim;
    use crate::presence::{EntranceAlias as A, Exit};
    use ds_core::machine::Machine;
    use ds_core::time::stamp::Stamp;
    use ds_core::vocab::Shown;
    use ds_style::appearance::motion::MotionLevel;

    const OUT: P = P::Leaving(Exit::OsdOut);
    const PARAMS: PresenceParams = PresenceParams {
        enter: Anim::PaletteFade,
        exit: Exit::OsdOut,
        motion: MotionLevel::Standard,
    };

    fn life(presence: P, alias: A, until: Option<u64>) -> Life {
        Life {
            presence,
            alias,
            until: until.map(Stamp),
        }
    }

    /// The settle of the OSD's entrance and exit at Standard, in ms.
    fn settles() -> (u64, u64) {
        let ms = |anim| crate::settle::settle(anim, MotionLevel::Standard).as_millis() as u64;
        (ms(Anim::PaletteFade), ms(Anim::OsdOut))
    }

    /// Name, state before, input, time, state after, outputs.
    type Case = (&'static str, Life, I, u64, Life, Vec<O>);

    #[test]
    fn each_input_moves_the_surface() {
        let (enter, exit) = settles();
        let entering = |from| life(P::Entering, A::B, Some(from + enter));
        let leaving = |alias, from| life(OUT, alias, Some(from + exit));
        #[rustfmt::skip]
        let cases: Vec<Case> = vec![
            ("show a hidden surface", Life::hidden(), I::Show, 100, entering(100), vec![]),
            ("its entrance settles", entering(100), I::Elapsed, 100 + enter, life(P::Present, A::B, None), vec![]),
            ("hide a present surface", life(P::Present, A::B, None), I::Hide, 500, leaving(A::B, 500), vec![]),
            ("its exit settles: gone", leaving(A::B, 500), I::Elapsed, 500 + exit, life(P::Hidden, A::B, None), vec![O::Gone]),
            // Hidden while still coming in: it goes from where it is.
            ("hide while entering", entering(100), I::Hide, 150, leaving(A::B, 150), vec![]),
            // Shown again while fading out: the hide is taken back, no second entrance.
            ("show while leaving", leaving(A::B, 500), I::Show, 520, life(P::Present, A::Held, None), vec![]),
            ("the next showing flips the alias", life(P::Hidden, A::Held, None), I::Show, 900, life(P::Entering, A::A, Some(900 + enter)), vec![]),
            // Early wakes, late settles and repeats change nothing.
            ("early wake while entering", entering(100), I::Elapsed, 100 + enter - 1, entering(100), vec![]),
            ("early wake while leaving", leaving(A::B, 500), I::Elapsed, 500 + exit - 1, leaving(A::B, 500), vec![]),
            ("a wake at rest", life(P::Present, A::B, None), I::Elapsed, 900, life(P::Present, A::B, None), vec![]),
            ("show a present surface", life(P::Present, A::B, None), I::Show, 900, life(P::Present, A::B, None), vec![]),
            ("hide a hidden surface", Life::hidden(), I::Hide, 900, Life::hidden(), vec![]),
            ("hide a leaving surface", leaving(A::B, 500), I::Hide, 520, leaving(A::B, 500), vec![]),
        ];
        for (name, from, input, at, to, outs) in cases {
            let (next, out) = from.step(input, Stamp(at), &PARAMS, &());
            assert_eq!((next, out), (to, outs), "{name}");
        }
    }

    #[test]
    fn only_motion_in_flight_wakes_the_machine() {
        let (enter, exit) = settles();
        let step = |life: Life, input, at| life.step(input, Stamp(at), &PARAMS, &()).0;
        let entering = step(Life::hidden(), I::Show, 40);
        assert_eq!(entering.wake(), Some(Stamp(40 + enter)));
        let present = step(entering, I::Elapsed, 40 + enter);
        assert_eq!(present.wake(), None);
        assert_eq!(step(present, I::Hide, 9).wake(), Some(Stamp(9 + exit)));
        assert_eq!(Life::hidden().wake(), None);
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
            assert_eq!(change(last, now), want, "{last:?} -> {now:?}");
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
