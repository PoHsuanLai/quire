//! The green light's hold, as a pure machine: held for the press delay or rested on for the
//! hover delay, it opens the tiling menu; a click that ends a hold which opened the menu does not
//! also zoom. Each wait carries the generation it was started in, so a release, a leave or a
//! newer wait makes an older one stale instead of cancelling a task.

/// What started the wait.
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub(crate) enum Waiting {
    /// Nothing is pending.
    Nothing,
    /// The button is held on the light.
    Press,
    /// The pointer rests on the light.
    Hover,
}

/// What the click that ends a press does.
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub(crate) enum Click {
    /// Zoom, as a click on the green light does.
    Zoom,
    /// Nothing: the press opened the menu.
    Swallow,
}

/// A wait's ticket: the generation it was started in.
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub(crate) struct Ticket(u64);

/// The light's hold.
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub(crate) struct Hold {
    generation: u64,
    waiting: Waiting,
    click: Click,
}

impl Default for Hold {
    fn default() -> Self {
        Hold {
            generation: 0,
            waiting: Waiting::Nothing,
            click: Click::Zoom,
        }
    }
}

impl Hold {
    /// Start waiting for `what`; the returned ticket fires it.
    pub(crate) fn wait(self, what: Waiting) -> (Self, Ticket) {
        let generation = self.generation + 1;
        let click = match what {
            Waiting::Press => Click::Zoom,
            Waiting::Hover | Waiting::Nothing => self.click,
        };
        (
            Hold {
                generation,
                waiting: what,
                click,
            },
            Ticket(generation),
        )
    }

    /// Stop waiting: the button came up or the pointer left.
    pub(crate) fn cancel(self) -> Self {
        Hold {
            generation: self.generation + 1,
            waiting: Waiting::Nothing,
            ..self
        }
    }

    /// The wait for `ticket` is over: whether the menu opens now. A press that opens it
    /// swallows the click that will end it.
    pub(crate) fn fire(self, ticket: Ticket) -> (Self, Opens) {
        if ticket.0 != self.generation || self.waiting == Waiting::Nothing {
            return (self, Opens::No);
        }
        let click = match self.waiting {
            Waiting::Press => Click::Swallow,
            Waiting::Hover | Waiting::Nothing => self.click,
        };
        (
            Hold {
                waiting: Waiting::Nothing,
                click,
                ..self
            },
            Opens::Yes,
        )
    }

    /// A click on the light: what it does, and the hold after it.
    pub(crate) fn click(self) -> (Self, Click) {
        (
            Hold {
                click: Click::Zoom,
                ..self
            },
            self.click,
        )
    }
}

/// Whether a finished wait opens the menu.
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub(crate) enum Opens {
    /// It does.
    Yes,
    /// It was stale.
    No,
}

#[cfg(test)]
mod tests {
    use super::{Click, Hold, Opens, Waiting};

    #[test]
    fn a_held_press_opens_and_swallows_its_click() {
        let (hold, ticket) = Hold::default().wait(Waiting::Press);
        let (hold, opens) = hold.fire(ticket);
        assert_eq!(opens, Opens::Yes);
        let (hold, click) = hold.click();
        assert_eq!(click, Click::Swallow);
        assert_eq!(hold.click().1, Click::Zoom, "only that one click");
    }

    #[test]
    fn a_short_press_is_a_zoom() {
        let (hold, ticket) = Hold::default().wait(Waiting::Press);
        let hold = hold.cancel();
        assert_eq!(hold.fire(ticket).1, Opens::No);
        assert_eq!(hold.click().1, Click::Zoom);
    }

    #[test]
    fn a_rest_opens_and_a_leave_or_a_newer_wait_makes_it_stale() {
        let (hold, ticket) = Hold::default().wait(Waiting::Hover);
        assert_eq!(hold.fire(ticket).1, Opens::Yes);
        assert_eq!(hold.cancel().fire(ticket).1, Opens::No);
        let (newer, _) = hold.wait(Waiting::Press);
        assert_eq!(newer.fire(ticket).1, Opens::No);
    }

    #[test]
    fn a_rest_that_opened_the_menu_leaves_the_click_a_zoom() {
        let (hold, ticket) = Hold::default().wait(Waiting::Hover);
        let (hold, _) = hold.fire(ticket);
        assert_eq!(hold.click().1, Click::Zoom);
    }
}
