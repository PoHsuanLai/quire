//! Who moves a menu's highlight: the menu itself, or a field beside it. The
//! composer's `/` and `@` menus and a people input keep the keyboard in their own field while a
//! menu lists the matches, so the field says which choice is highlighted and the menu only
//! reports what the keys and the pointer asked for. Pure, beside `menu` and `menu_panel`.

use crate::components::menus::menu_lines::KeyAct;
use crate::stack::roving::{Step, settled};
use ds_core::vocab::Availability;

/// Whose highlight a menu shows.
#[derive(Debug, Clone, Copy, PartialEq, Eq, Hash, Default)]
pub enum MenuCursor {
    /// The menu's own: Up and Down, the pointer and a submenu's rest move it, and the menu takes
    /// the keyboard when it opens.
    #[default]
    Auto,
    /// The caller's: this choice (clamped; `None` highlights nothing), numbered as `expanded`
    /// numbers them. The menu leaves the keyboard where it is, and a key or the pointer only
    /// asks for a move through `on_active`.
    Controlled(Option<usize>),
}

/// The highlighted choice among `live`: the menu's own `selection` settled onto an enabled
/// choice, or the caller's, clamped to the last choice. `None` when nothing is highlighted.
pub(crate) fn highlighted(
    cursor: MenuCursor,
    selection: usize,
    live: &[Availability],
) -> Option<usize> {
    let last = live.len().checked_sub(1)?;
    match cursor {
        MenuCursor::Auto => Some(settled(selection, live)),
        MenuCursor::Controlled(index) => index.map(|index| index.min(last)),
    }
}

/// Where a key starts from when nothing is highlighted: Down lands on the first choice (from the
/// last, wrapping), Up on the last (from the first); anything else has nothing to act on.
pub(crate) fn seed(act: &KeyAct, count: usize) -> Option<usize> {
    let last = count.checked_sub(1)?;
    match act {
        KeyAct::Move(Step::Down) => Some(last),
        KeyAct::Move(Step::Up) | KeyAct::Edge(_) => Some(0),
        KeyAct::Pick
        | KeyAct::Open
        | KeyAct::Back
        | KeyAct::Close
        | KeyAct::Type(_)
        | KeyAct::Jump(_)
        | KeyAct::Erase => None,
    }
}

impl MenuCursor {
    /// Whether the menu takes the keyboard as it opens: not while a field beside it drives it.
    pub(crate) fn takes_focus(self) -> bool {
        matches!(self, MenuCursor::Auto)
    }
}

#[cfg(test)]
mod tests {
    use super::{MenuCursor, highlighted, seed};
    use crate::components::menus::menu_lines::KeyAct;
    use crate::stack::roving::Step;
    use ds_core::vocab::Availability::{Disabled as D, Enabled as E};

    #[test]
    fn the_highlight_is_the_menus_own_or_the_callers() {
        // (cursor, own selection, live, want)
        #[rustfmt::skip]
        let cases: &[(MenuCursor, usize, &[_], Option<usize>)] = &[
            (MenuCursor::Auto, 0, &[D, E], Some(1)),
            (MenuCursor::Auto, 1, &[E, E], Some(1)),
            (MenuCursor::Controlled(Some(0)), 1, &[E, E], Some(0)),
            // The caller's is shown as asked, disabled or not, clamped to the list.
            (MenuCursor::Controlled(Some(0)), 1, &[D, E], Some(0)),
            (MenuCursor::Controlled(Some(9)), 0, &[E, E, E], Some(2)),
            (MenuCursor::Controlled(None), 1, &[E, E], None),
            (MenuCursor::Auto, 0, &[], None),
            (MenuCursor::Controlled(Some(0)), 0, &[], None),
        ];
        for &(cursor, selection, live, want) in cases {
            assert_eq!(
                highlighted(cursor, selection, live),
                want,
                "{cursor:?} {selection} {live:?}"
            );
        }
    }

    #[test]
    fn a_move_from_nothing_starts_at_an_end() {
        assert_eq!(seed(&KeyAct::Move(Step::Down), 3), Some(2));
        assert_eq!(seed(&KeyAct::Move(Step::Up), 3), Some(0));
        assert_eq!(seed(&KeyAct::Pick, 3), None);
        assert_eq!(seed(&KeyAct::Move(Step::Down), 0), None);
        assert!(MenuCursor::Auto.takes_focus());
        assert!(!MenuCursor::Controlled(None).takes_focus());
    }
}
