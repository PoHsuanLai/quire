//! The green light's hold, as a pure [`Machine`]: held for the press delay or rested on for the
//! hover delay, it opens the tiling menu; a click that ends a hold which opened the menu does not
//! also zoom. The wait's end is in the state, so a release, a leave or a newer wait replaces it
//! and nothing is left to cancel.

use ds_core::machine::{Elapsed, Machine};
use ds_core::time::stamp::Stamp;
use std::time::Duration;

/// What the light is waiting for, and until when.
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub(crate) enum Waiting {
    /// Nothing is pending.
    Nothing,
    /// The button is held on the light; the menu opens at this time.
    Press(Stamp),
    /// The pointer rests on the light; the menu opens at this time.
    Hover(Stamp),
}

/// What the click that ends a press does.
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub(crate) enum Click {
    /// Zoom, as a click on the green light does.
    Zoom,
    /// Nothing: the press opened the menu.
    Swallow,
}

/// What starts a wait.
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub(crate) enum Wait {
    /// The button went down on the light.
    Press,
    /// The pointer came to rest on the light.
    Hover,
}

/// The light's hold.
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub(crate) struct Hold {
    waiting: Waiting,
    click: Click,
}

impl Default for Hold {
    fn default() -> Self {
        Hold {
            waiting: Waiting::Nothing,
            click: Click::Zoom,
        }
    }
}

/// How long each wait lasts (`window.tile_menu_press_ms`, `window.tile_menu_hover_ms`).
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub(crate) struct HoldTiming {
    pub(crate) press: Duration,
    pub(crate) hover: Duration,
}

/// What moves the hold.
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub(crate) enum HoldIn {
    /// Start waiting; a wait still running is replaced.
    Wait(Wait),
    /// Stop waiting: the button came up or the pointer left.
    Cancel,
    /// A click on the light.
    Click,
    /// The wait's end came.
    Elapsed,
}

impl From<Elapsed> for HoldIn {
    fn from(_: Elapsed) -> Self {
        HoldIn::Elapsed
    }
}

/// What the light's owner does.
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub(crate) enum HoldOut {
    /// Open the tiling menu.
    OpenMenu,
    /// Toggle the window's zoom: a click that did not follow a hold which opened the menu.
    ToggleZoom,
}

impl Machine for Hold {
    type In = HoldIn;
    type Out = HoldOut;
    type Params = HoldTiming;
    type Ctx = ();

    /// A press that ends its wait opens the menu and swallows the click that will end it; a rest
    /// that ends its wait opens it and leaves the click a zoom. A wake before the wait's end, or
    /// after it was replaced, changes nothing.
    fn step(self, input: HoldIn, at: Stamp, timing: &HoldTiming, _: &()) -> (Hold, Vec<HoldOut>) {
        match (input, self.waiting) {
            (HoldIn::Wait(Wait::Press), _) => (
                Hold {
                    waiting: Waiting::Press(at.after_span(timing.press)),
                    click: Click::Zoom,
                },
                Vec::new(),
            ),
            (HoldIn::Wait(Wait::Hover), _) => (
                Hold {
                    waiting: Waiting::Hover(at.after_span(timing.hover)),
                    ..self
                },
                Vec::new(),
            ),
            (HoldIn::Cancel, _) => (
                Hold {
                    waiting: Waiting::Nothing,
                    ..self
                },
                Vec::new(),
            ),
            (HoldIn::Click, _) => (
                Hold {
                    click: Click::Zoom,
                    ..self
                },
                match self.click {
                    Click::Zoom => vec![HoldOut::ToggleZoom],
                    Click::Swallow => Vec::new(),
                },
            ),
            (HoldIn::Elapsed, Waiting::Press(until)) if at >= until => (
                Hold {
                    waiting: Waiting::Nothing,
                    click: Click::Swallow,
                },
                vec![HoldOut::OpenMenu],
            ),
            (HoldIn::Elapsed, Waiting::Hover(until)) if at >= until => (
                Hold {
                    waiting: Waiting::Nothing,
                    ..self
                },
                vec![HoldOut::OpenMenu],
            ),
            (HoldIn::Elapsed, Waiting::Press(_) | Waiting::Hover(_) | Waiting::Nothing) => {
                (self, Vec::new())
            }
        }
    }

    fn wake(&self) -> Option<Stamp> {
        match self.waiting {
            Waiting::Nothing => None,
            Waiting::Press(until) | Waiting::Hover(until) => Some(until),
        }
    }
}

#[cfg(test)]
mod tests {
    use super::{Click, Hold, HoldIn, HoldOut, HoldTiming, Wait, Waiting};
    use ds_core::machine::Machine;
    use ds_core::time::stamp::Stamp;
    use std::time::Duration;

    const TIMING: HoldTiming = HoldTiming {
        press: Duration::from_millis(500),
        hover: Duration::from_millis(800),
    };

    fn hold(waiting: Waiting, click: Click) -> Hold {
        Hold { waiting, click }
    }

    fn press(until: u64) -> Waiting {
        Waiting::Press(Stamp(until))
    }

    fn hover(until: u64) -> Waiting {
        Waiting::Hover(Stamp(until))
    }

    /// Name, state before, input, time, state after, outputs, next wake.
    type Case = (
        &'static str,
        Hold,
        HoldIn,
        u64,
        Hold,
        &'static [HoldOut],
        Option<u64>,
    );

    #[test]
    fn a_hold_or_a_rest_opens_the_menu_and_a_click_zooms_unless_a_hold_opened_it() {
        use Click::{Swallow, Zoom};
        use HoldOut::{OpenMenu, ToggleZoom};
        let rest = Hold::default();
        #[rustfmt::skip]
        let cases: &[Case] = &[
            ("a press starts the press wait", rest, HoldIn::Wait(Wait::Press), 100, hold(press(600), Zoom), &[], Some(600)),
            ("a rest starts the hover wait", rest, HoldIn::Wait(Wait::Hover), 100, hold(hover(900), Zoom), &[], Some(900)),
            ("a press ends: the menu opens, the click is swallowed", hold(press(600), Zoom), HoldIn::Elapsed, 600, hold(Waiting::Nothing, Swallow), &[OpenMenu], None),
            ("a rest ends: the menu opens, the click still zooms", hold(hover(900), Zoom), HoldIn::Elapsed, 900, hold(Waiting::Nothing, Zoom), &[OpenMenu], None),
            ("woken early: nothing", hold(press(600), Zoom), HoldIn::Elapsed, 599, hold(press(600), Zoom), &[], Some(600)),
            ("a wake with nothing waiting: nothing", rest, HoldIn::Elapsed, 900, rest, &[], None),
            ("a short press is cancelled: no menu", hold(press(600), Zoom), HoldIn::Cancel, 200, hold(Waiting::Nothing, Zoom), &[], None),
            ("a leave cancels a rest", hold(hover(900), Zoom), HoldIn::Cancel, 200, hold(Waiting::Nothing, Zoom), &[], None),
            ("a newer wait replaces an older one", hold(hover(900), Zoom), HoldIn::Wait(Wait::Press), 300, hold(press(800), Zoom), &[], Some(800)),
            ("a rest after a swallowing hold keeps the swallow", hold(Waiting::Nothing, Swallow), HoldIn::Wait(Wait::Hover), 700, hold(hover(1500), Swallow), &[], Some(1500)),
            ("a press clears the swallow", hold(Waiting::Nothing, Swallow), HoldIn::Wait(Wait::Press), 700, hold(press(1200), Zoom), &[], Some(1200)),
            ("a click zooms", rest, HoldIn::Click, 10, rest, &[ToggleZoom], None),
            ("the click that ends a hold which opened the menu does nothing", hold(Waiting::Nothing, Swallow), HoldIn::Click, 10, rest, &[], None),
        ];
        for (name, from, input, at, state, outs, wake) in cases {
            let (next, out) = from.step(*input, Stamp(*at), &TIMING, &());
            assert_eq!(next, *state, "{name}: state");
            assert_eq!(out.as_slice(), *outs, "{name}: outputs");
            assert_eq!(next.wake(), wake.map(Stamp), "{name}: wake");
        }
    }

    #[test]
    fn only_the_one_click_after_a_hold_is_swallowed() {
        let (hold, _) = Hold::default().step(HoldIn::Wait(Wait::Press), Stamp(0), &TIMING, &());
        let (hold, opens) = hold.step(HoldIn::Elapsed, Stamp(500), &TIMING, &());
        assert_eq!(opens, vec![HoldOut::OpenMenu]);
        let (hold, first) = hold.step(HoldIn::Click, Stamp(520), &TIMING, &());
        assert_eq!(first, Vec::new(), "swallowed");
        let (_, second) = hold.step(HoldIn::Click, Stamp(900), &TIMING, &());
        assert_eq!(second, vec![HoldOut::ToggleZoom], "the next one zooms");
    }
}
