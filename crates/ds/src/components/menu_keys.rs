//! What a key means to one menu panel, given its choices, its selection and whether its own
//! submenu is open: pure, so the whole keyboard contract is one table (design/06-INTERACTIONS.md
//! section 2.4; design/13-BEHAVIOUR-menus-windows.md sections 13.3.2-13.3.4).

use crate::components::menu_lines::{Act, Choice, KeyAct, Nav, liveness, moved_live};
use crate::components::vocab::Availability;

/// Which panel: the menu itself, or a submenu (whose Escape and Left go back one level).
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub(crate) enum Level {
    /// The menu.
    Root,
    /// A submenu.
    Sub,
}

/// Whether the panel's own submenu is open.
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub(crate) enum Child {
    /// A submenu of this panel is open.
    Open,
    /// None is.
    Closed,
}

/// What the panel does.
#[derive(Debug, Clone, PartialEq)]
pub(crate) enum Decision<T> {
    /// Move the selection here.
    Select(usize),
    /// Close the whole menu, then yield this value.
    Pick(T),
    /// Open this choice's submenu now, by the keyboard.
    Expand(usize),
    /// Close this panel's open submenu.
    CloseSub,
    /// Close this panel (a submenu) and give the focus back to its parent.
    Back,
    /// Close the menu (Escape at the root).
    CloseMenu,
    /// The key edits the query (the root's filter).
    Query,
    /// Nothing happens.
    Nothing,
}

/// `act` on a panel at `level` whose selection is `selected` among `choices`.
pub(crate) fn decide<T: Clone>(
    act: &KeyAct,
    selected: usize,
    choices: &[Choice<T>],
    child: Child,
    level: Level,
) -> Decision<T> {
    let live = choices
        .get(selected)
        .filter(|choice| choice.availability == Availability::Enabled);
    match act {
        KeyAct::Move(step) => {
            Decision::Select(moved_live(Nav::Wrap, selected, &liveness(choices), *step))
        }
        KeyAct::Pick => match live.map(|choice| &choice.act) {
            Some(Act::Pick(value)) => Decision::Pick(value.clone()),
            Some(Act::Open(_)) => Decision::Expand(selected),
            None => Decision::Nothing,
        },
        KeyAct::Open => match live.map(|choice| &choice.act) {
            Some(Act::Open(_)) => Decision::Expand(selected),
            Some(Act::Pick(_)) | None => Decision::Nothing,
        },
        KeyAct::Back | KeyAct::Close if child == Child::Open => Decision::CloseSub,
        KeyAct::Back | KeyAct::Close if level == Level::Sub => Decision::Back,
        KeyAct::Back => Decision::Nothing,
        KeyAct::Close => Decision::CloseMenu,
        KeyAct::Type(_) | KeyAct::Erase => match level {
            Level::Root => Decision::Query,
            Level::Sub => Decision::Nothing,
        },
    }
}

#[cfg(test)]
mod tests {
    use super::{Child, Decision, Level, decide};
    use crate::components::menu_lines::{Act, Choice, KeyAct, Step};
    use crate::components::vocab::Availability;

    fn pick(value: u8) -> Choice<u8> {
        Choice {
            act: Act::Pick(value),
            availability: Availability::Enabled,
        }
    }

    fn off(value: u8) -> Choice<u8> {
        Choice {
            act: Act::Pick(value),
            availability: Availability::Disabled,
        }
    }

    fn parent() -> Choice<u8> {
        Choice {
            act: Act::Open(Vec::new()),
            availability: Availability::Enabled,
        }
    }

    #[test]
    fn keys_act_on_the_selected_choice_and_close_one_level() {
        let choices = vec![pick(10), off(11), parent(), pick(13)];
        let (open, closed) = (Child::Open, Child::Closed);
        let (root, sub) = (Level::Root, Level::Sub);
        // (key, selected, child, level, want)
        #[rustfmt::skip]
        let cases: Vec<(KeyAct, usize, Child, Level, Decision<u8>)> = vec![
            (KeyAct::Move(Step::Down), 0, closed, root, Decision::Select(2)),
            (KeyAct::Move(Step::Up), 2, closed, root, Decision::Select(0)),
            (KeyAct::Pick, 0, closed, root, Decision::Pick(10)),
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
            (KeyAct::Type("a".into()), 0, closed, root, Decision::Query),
            (KeyAct::Erase, 0, closed, sub, Decision::Nothing),
        ];
        for (act, selected, child, level, want) in cases {
            assert_eq!(
                decide(&act, selected, &choices, child, level),
                want,
                "{act:?} on {selected} ({child:?}, {level:?})"
            );
        }
    }
}
