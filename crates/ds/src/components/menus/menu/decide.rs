//! What a key means to one menu panel, given its choices, its selection and whether its own
//! submenu is open: pure, so the whole keyboard contract is one table (design/06-INTERACTIONS.md
//! section 2.4; design/13-BEHAVIOUR-menus-windows.md sections 13.3.2-13.3.4).

use crate::components::menus::item::item::AfterPick;
use crate::components::menus::menu::choices::{Act, Choice, Slot};
use crate::components::menus::menu::keys::KeyAct;
use crate::stack::roving::{Step, Wrap, edge_live, moved_live};
use ds_core::vocab::Availability;

/// Which panel: the menu itself, or a submenu (whose Escape and Left go back one level).
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub(crate) enum Level {
    /// The menu.
    Root,
    /// A submenu.
    Sub,
}

/// What the panel does.
#[derive(Debug, Clone, PartialEq)]
pub(crate) enum Decision<T> {
    /// Move the selection here.
    Select(usize),
    /// Yield this value, then close the whole menu or leave it open, as the item says.
    Pick(T, AfterPick),
    /// Open this choice's submenu now, by the keyboard.
    Expand(usize),
    /// Close this panel's open submenu.
    CloseSub,
    /// Close this panel (a submenu) and give the focus back to its parent.
    Back,
    /// Close the menu (Escape at the root).
    CloseMenu,
    /// Nothing happens.
    Nothing,
}

/// The choices Up and Down can stop on: a row of swatches is one stop, at its first dot.
fn stops<T>(choices: &[Choice<T>]) -> Vec<Availability> {
    choices
        .iter()
        .map(|choice| match choice.slot {
            Slot::Dot { pos, .. } if pos > 0 => Availability::Disabled,
            Slot::Dot { .. } | Slot::Alone => choice.availability,
        })
        .collect()
}

/// Where a vertical move in direction `step` starts from: the near end of the swatch row the
/// selection is on, so the move leaves the row and does not walk its dots.
fn leaving<T>(choices: &[Choice<T>], selected: usize, step: Step) -> usize {
    match (choices.get(selected).map(|choice| choice.slot), step) {
        (Some(Slot::Dot { pos, .. }), Step::Up) => selected - pos,
        (Some(Slot::Dot { pos, len }), Step::Down) => selected + (len - 1 - pos),
        _ => selected,
    }
}

/// The dot beside `selected` in its row, forward (`Step::Down`) or back, when the selection is on
/// a dot and there is one that way.
fn beside<T>(choices: &[Choice<T>], selected: usize, step: Step) -> Option<usize> {
    let Slot::Dot { pos, len } = choices.get(selected)?.slot else {
        return None;
    };
    match step {
        Step::Down => (pos + 1 < len).then_some(selected + 1),
        Step::Up => (pos > 0).then(|| selected - 1),
    }
}

/// `act` on a panel at `level` whose selection is `selected` among `choices`.
pub(crate) fn decide<T: Clone>(
    act: &KeyAct,
    selected: usize,
    choices: &[Choice<T>],
    child_open: bool,
    level: Level,
) -> Decision<T> {
    let across = match act {
        KeyAct::Open => beside(choices, selected, Step::Down),
        KeyAct::Back => beside(choices, selected, Step::Up),
        _ => None,
    };
    if let Some(at) = across {
        return Decision::Select(at);
    }
    let live = choices
        .get(selected)
        .filter(|choice| choice.availability == Availability::Enabled);
    match act {
        KeyAct::Move(step) => Decision::Select(moved_live(
            Wrap::Wraps,
            leaving(choices, selected, *step),
            &stops(choices),
            *step,
        )),
        KeyAct::Edge(edge) => Decision::Select(edge_live(*edge, selected, &stops(choices))),
        KeyAct::Pick => match live.map(|choice| &choice.act) {
            Some(Act::Pick(value, after)) => Decision::Pick(value.clone(), *after),
            Some(Act::Open(_)) => Decision::Expand(selected),
            None => Decision::Nothing,
        },
        KeyAct::Open => match live.map(|choice| &choice.act) {
            Some(Act::Open(_)) => Decision::Expand(selected),
            Some(Act::Pick(..)) | None => Decision::Nothing,
        },
        KeyAct::Back | KeyAct::Close if child_open => Decision::CloseSub,
        KeyAct::Back | KeyAct::Close if level == Level::Sub => Decision::Back,
        KeyAct::Back => Decision::Nothing,
        KeyAct::Close => Decision::CloseMenu,
        KeyAct::Jump(_) => Decision::Nothing,
    }
}

#[cfg(test)]
mod tests {
    use super::{AfterPick, Decision, Level, decide};
    use crate::components::menus::menu::choices::{Act, Choice, Slot};
    use crate::components::menus::menu::keys::KeyAct;
    use crate::stack::roving::{Edge, Step};
    use ds_core::vocab::Availability;

    fn pick(value: u8) -> Choice<u8> {
        Choice {
            act: Act::Pick(value, AfterPick::Close),
            availability: Availability::Enabled,
            title: String::new(),
            slot: Slot::Alone,
        }
    }

    fn off(value: u8) -> Choice<u8> {
        Choice {
            act: Act::Pick(value, AfterPick::Close),
            availability: Availability::Disabled,
            title: String::new(),
            slot: Slot::Alone,
        }
    }

    fn parent() -> Choice<u8> {
        Choice {
            act: Act::Open(Vec::new()),
            availability: Availability::Enabled,
            title: String::new(),
            slot: Slot::Alone,
        }
    }

    #[test]
    fn keys_act_on_the_selected_choice_and_close_one_level() {
        let choices = vec![pick(10), off(11), parent(), pick(13)];
        let (open, closed) = (true, false);
        let (root, sub) = (Level::Root, Level::Sub);
        // (key, selected, child, level, want)
        #[rustfmt::skip]
        let cases: Vec<(KeyAct, usize, bool, Level, Decision<u8>)> = vec![
            (KeyAct::Move(Step::Down), 0, closed, root, Decision::Select(2)),
            (KeyAct::Move(Step::Up), 2, closed, root, Decision::Select(0)),
            (KeyAct::Pick, 0, closed, root, Decision::Pick(10, AfterPick::Close)),
            (KeyAct::Pick, 1, closed, root, Decision::Nothing),
            (KeyAct::Pick, 2, closed, root, Decision::Expand(2)),
            (KeyAct::Open, 2, closed, sub, Decision::Expand(2)),
            (KeyAct::Open, 0, closed, root, Decision::Nothing),
            (KeyAct::Back, 2, open, root, Decision::CloseSub),
            (KeyAct::Back, 0, closed, sub, Decision::Back),
            (KeyAct::Back, 0, closed, root, Decision::Nothing),
            (KeyAct::Close, 2, open, sub, Decision::CloseSub),
            (KeyAct::Close, 0, closed, sub, Decision::Back),
            (KeyAct::Close, 0, closed, root, Decision::CloseMenu),
            (KeyAct::Jump("a".into()), 0, closed, root, Decision::Nothing),
        ];
        for (act, selected, child_open, level, want) in cases {
            assert_eq!(
                decide(&act, selected, &choices, child_open, level),
                want,
                "{act:?} on {selected} (child open: {child_open}, {level:?})"
            );
        }
    }

    fn dot(value: u8, pos: usize) -> Choice<u8> {
        Choice {
            slot: Slot::Dot { pos, len: 3 },
            ..pick(value)
        }
    }

    #[test]
    fn a_row_of_dots_is_one_stop_for_up_and_down_and_left_and_right_walk_its_dots() {
        let choices = vec![pick(0), dot(1, 0), dot(2, 1), dot(3, 2), pick(4)];
        let (closed, root, sub) = (false, Level::Root, Level::Sub);
        // (key, selected, level, want)
        #[rustfmt::skip]
        let cases: Vec<(KeyAct, usize, Level, Decision<u8>)> = vec![
            (KeyAct::Move(Step::Down), 0, root, Decision::Select(1)),
            (KeyAct::Move(Step::Down), 1, root, Decision::Select(4)),
            (KeyAct::Move(Step::Down), 2, root, Decision::Select(4)),
            (KeyAct::Move(Step::Up), 3, root, Decision::Select(0)),
            (KeyAct::Move(Step::Up), 4, root, Decision::Select(1)),
            (KeyAct::Edge(Edge::Last), 1, root, Decision::Select(4)),
            (KeyAct::Open, 1, root, Decision::Select(2)),
            (KeyAct::Open, 3, root, Decision::Nothing),
            (KeyAct::Back, 3, root, Decision::Select(2)),
            (KeyAct::Back, 1, root, Decision::Nothing),
            (KeyAct::Back, 1, sub, Decision::Back),
            (KeyAct::Pick, 2, root, Decision::Pick(2, AfterPick::Close)),
        ];
        for (act, selected, level, want) in cases {
            assert_eq!(
                decide(&act, selected, &choices, closed, level),
                want,
                "{act:?} on {selected} ({level:?})"
            );
        }
    }
}
