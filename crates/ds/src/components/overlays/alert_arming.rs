//! An alert's arming delay (`Alert`'s `arm_after`) and the rule that decides what an input does
//! while it waits: an answer already in flight (a keypress or click aimed at the window under a
//! permission question) must not land on a question the person never saw.

use crate::components::overlays::alert_model::{AlertRole, default_button, escape_button};
use ds_core::machine::{Elapsed, Machine};
use ds_core::time::stamp::Stamp;
use ds_core::vocab::Availability;
use std::time::Duration;

/// Whether an alert takes answers yet: `Alert`'s `arm_after` holds them off for a moment after
/// the alert appears, so a keypress or click that was already on its way (the person typing in
/// the window under a permission question) cannot answer it. Without a delay an alert is
/// `Armed` from the start. The deadline is in the state, and the clock alone ends the wait.
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum Arming {
    /// Takes answers: Return, Escape, Space and clicks do what they always do.
    Armed,
    /// Takes none until `until`: its buttons are disabled, and Return, Escape, Space and clicks
    /// do nothing (Escape does not dismiss either).
    Waiting {
        /// When the alert arms.
        until: Stamp,
    },
}

impl Arming {
    /// The state of an alert that appears at `at` with `delay` (none: armed at once).
    pub fn after(at: Stamp, delay: Option<Duration>) -> Arming {
        match delay {
            Some(delay) => Arming::Waiting {
                until: at.after_span(delay),
            },
            None => Arming::Armed,
        }
    }

    /// What the buttons are while this holds: disabled until armed.
    pub fn availability(self) -> Availability {
        match self {
            Arming::Armed => Availability::Enabled,
            Arming::Waiting { .. } => Availability::Disabled,
        }
    }
}

/// An answer the person gives an alert, from the keyboard or the pointer.
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum AlertInput {
    /// Return, wherever the keyboard is.
    Return,
    /// Escape, or the sheet closing another way.
    Escape,
    /// Space or a click on button `n`.
    Press(usize),
}

/// What the caller wants to hear of a dismissal.
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum Hearing {
    /// Only the buttons: Escape presses Cancel.
    Buttons,
    /// A dismissal too (`on_dismiss`): Escape reports it and presses nothing.
    Dismissal,
}

/// What an [`AlertInput`] comes to.
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum Route {
    /// Press button `n`.
    Press(usize),
    /// Report the dismissal.
    Dismiss,
    /// Nothing happens.
    Nothing,
}

/// What `input` comes to for an alert whose buttons have `roles`: nothing until it is armed, then
/// Return presses the default, Escape reports a dismissal the caller listens for or else presses
/// Cancel, and a press is that button's.
pub fn route(arming: Arming, input: AlertInput, roles: &[AlertRole], hearing: Hearing) -> Route {
    let press = |index: Option<usize>| index.map_or(Route::Nothing, Route::Press);
    match (arming, input, hearing) {
        (Arming::Waiting { .. }, _, _) => Route::Nothing,
        (Arming::Armed, AlertInput::Return, _) => press(default_button(roles)),
        (Arming::Armed, AlertInput::Escape, Hearing::Dismissal) => Route::Dismiss,
        (Arming::Armed, AlertInput::Escape, Hearing::Buttons) => press(escape_button(roles)),
        (Arming::Armed, AlertInput::Press(index), _) => {
            press((index < roles.len()).then_some(index))
        }
    }
}

impl Machine for Arming {
    type In = Elapsed;
    type Out = ();
    type Params = ();
    type Ctx = ();

    fn step(self, input: Elapsed, at: Stamp, _: &(), _: &()) -> (Arming, Vec<()>) {
        let next = match (self, input) {
            (Arming::Waiting { until }, Elapsed) if at >= until => Arming::Armed,
            (state, Elapsed) => state,
        };
        (next, Vec::new())
    }

    fn wake(&self) -> Option<Stamp> {
        match self {
            Arming::Waiting { until } => Some(*until),
            Arming::Armed => None,
        }
    }
}

#[cfg(test)]
mod tests {
    use super::{AlertInput, Arming, Hearing, Route, route};
    use crate::components::overlays::alert_model::AlertRole::{Cancel, Normal};
    use ds_core::machine::{Elapsed, Machine};
    use ds_core::time::stamp::Stamp;
    use ds_core::vocab::Availability;
    use std::time::Duration;

    #[test]
    fn arming_waits_for_its_deadline_and_the_clock_alone_ends_it() {
        let waiting = Arming::Waiting { until: Stamp(1500) };
        // (name, state, woken at, state after, wake after, buttons after)
        let cases = [
            (
                "no delay: armed at once",
                Arming::after(Stamp(500), None),
                500,
                Arming::Armed,
                None,
                Availability::Enabled,
            ),
            (
                "a delay: waiting",
                Arming::after(Stamp(500), Some(Duration::from_secs(1))),
                600,
                waiting,
                Some(Stamp(1500)),
                Availability::Disabled,
            ),
            (
                "woken early: still waiting",
                waiting,
                1499,
                waiting,
                Some(Stamp(1500)),
                Availability::Disabled,
            ),
            (
                "the end: armed",
                waiting,
                1500,
                Arming::Armed,
                None,
                Availability::Enabled,
            ),
            (
                "armed stays armed",
                Arming::Armed,
                9000,
                Arming::Armed,
                None,
                Availability::Enabled,
            ),
        ];
        for (name, state, at, next, wake, buttons) in cases {
            let (after, outs) = state.step(Elapsed, Stamp(at), &(), &());
            assert_eq!(
                (after, after.wake(), after.availability()),
                (next, wake, buttons),
                "{name}"
            );
            assert!(outs.is_empty(), "{name}: outputs");
        }
    }

    #[test]
    fn before_arming_no_input_does_anything_and_after_it_they_all_do() {
        let waiting = Arming::Waiting { until: Stamp(1) };
        let roles = [Normal, Cancel];
        // (input, listening, armed route)
        let cases = [
            (AlertInput::Return, Hearing::Buttons, Route::Press(0)),
            (AlertInput::Return, Hearing::Dismissal, Route::Press(0)),
            (AlertInput::Escape, Hearing::Buttons, Route::Press(1)),
            (AlertInput::Escape, Hearing::Dismissal, Route::Dismiss),
            (AlertInput::Press(1), Hearing::Buttons, Route::Press(1)),
            (AlertInput::Press(2), Hearing::Buttons, Route::Nothing),
        ];
        for (input, hearing, armed) in cases {
            assert_eq!(
                route(Arming::Armed, input, &roles, hearing),
                armed,
                "{input:?} {hearing:?}"
            );
            assert_eq!(
                route(waiting, input, &roles, hearing),
                Route::Nothing,
                "{input:?} {hearing:?} unarmed"
            );
        }
        // Escape with no Cancel button and nobody listening presses nothing, armed or not.
        assert_eq!(
            route(
                Arming::Armed,
                AlertInput::Escape,
                &[Normal],
                Hearing::Buttons
            ),
            Route::Nothing
        );
    }
}
