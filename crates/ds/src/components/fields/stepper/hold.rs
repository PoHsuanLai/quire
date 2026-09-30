//! A held press on a stepper (design/30 section 2.1: hold repeats): pure, so the timer that
//! repeats it can ask whether it is still the current one. A press starts a hold and gets a
//! ticket; each later press or release makes the older tickets stale.

use crate::components::fields::stepper::model::StepDirection;

/// Which press the timer belongs to.
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub(crate) struct Ticket(u32);

/// The half held down, if any, and the ticket the next press will take.
#[derive(Debug, Clone, Copy, PartialEq, Eq, Default)]
pub(crate) struct StepHold {
    held: Option<(StepDirection, Ticket)>,
    issued: u32,
}

impl StepHold {
    /// The half of the pair held down, as the pressed appearance wants it.
    pub(crate) fn direction(self) -> Option<StepDirection> {
        self.held.map(|(direction, _)| direction)
    }

    /// A press on `direction`, and the ticket its repeat timer carries.
    pub(crate) fn press(self, direction: StepDirection) -> (StepHold, Ticket) {
        let ticket = Ticket(self.issued.wrapping_add(1));
        (
            StepHold {
                held: Some((direction, ticket)),
                issued: ticket.0,
            },
            ticket,
        )
    }

    /// The press let go.
    pub(crate) fn release(self) -> StepHold {
        StepHold { held: None, ..self }
    }

    /// What the timer of `ticket` should step: the direction while that press is still the
    /// held one, nothing once it was released or replaced.
    pub(crate) fn due(self, ticket: Ticket) -> Option<StepDirection> {
        self.held
            .filter(|&(_, held)| held == ticket)
            .map(|(direction, _)| direction)
    }
}

#[cfg(test)]
mod tests {
    use super::StepHold;
    use crate::components::fields::stepper::model::StepDirection;

    #[test]
    fn a_timer_repeats_only_while_its_own_press_is_held() {
        let (held, first) = StepHold::default().press(StepDirection::Up);
        assert_eq!(held.due(first), Some(StepDirection::Up));
        let released = held.release();
        assert_eq!(released.due(first), None, "released");
        let (again, second) = released.press(StepDirection::Down);
        assert_eq!(again.due(first), None, "the first timer is stale");
        assert_eq!(again.due(second), Some(StepDirection::Down));
        assert_eq!(again.direction(), Some(StepDirection::Down));
    }
}
