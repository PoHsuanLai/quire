//! Who moves a menu's highlight (mailo gaps 2): the menu itself, or a field beside it. The
//! composer's `/` and `@` menus and a people input keep the keyboard in their own field while a
//! menu lists the matches, so the field says which choice is highlighted and the menu only
//! reports what the keys and the pointer asked for. Pure, beside `menu` and `menu_panel`.

use crate::components::menu_lines::{KeyAct, Step, settled};
use crate::components::vocab::Availability;

/// Whose highlight a menu shows.
#[derive(Debug, Clone, Copy, PartialEq, Eq, Hash, Default)]
pub enum Cursor {
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
    cursor: Cursor,
    selection: usize,
    live: &[Availability],
) -> Option<usize> {
    let last = live.len().checked_sub(1)?;
    match cursor {
        Cursor::Auto => Some(settled(selection, live)),
        Cursor::Controlled(index) => index.map(|index| index.min(last)),
    }
}

/// Where a key starts from when nothing is highlighted: Down lands on the first choice (from the
/// last, wrapping), Up on the last (from the first); anything else has nothing to act on.
pub(crate) fn seed(act: &KeyAct, count: usize) -> Option<usize> {
    let last = count.checked_sub(1)?;
    match act {
        KeyAct::Move(Step::Down) => Some(last),
        KeyAct::Move(Step::Up) => Some(0),
        KeyAct::Pick
        | KeyAct::Open
        | KeyAct::Back
        | KeyAct::Close
        | KeyAct::Type(_)
        | KeyAct::Erase => None,
    }
}

impl Cursor {
    /// Whether the menu takes the keyboard as it opens: not while a field beside it drives it.
    pub(crate) fn takes_focus(self) -> bool {
        matches!(self, Cursor::Auto)
    }
}

#[cfg(test)]
mod tests {
    use super::{Cursor, highlighted, seed};
    use crate::components::menu_lines::{KeyAct, Step};
    use crate::components::vocab::Availability::{Disabled as D, Enabled as E};

    #[test]
    fn the_highlight_is_the_menus_own_or_the_callers() {
        // (cursor, own selection, live, want)
        #[rustfmt::skip]
        let cases: &[(Cursor, usize, &[_], Option<usize>)] = &[
            (Cursor::Auto, 0, &[D, E], Some(1)),
            (Cursor::Auto, 1, &[E, E], Some(1)),
            (Cursor::Controlled(Some(0)), 1, &[E, E], Some(0)),
            // The caller's is shown as asked, disabled or not, clamped to the list.
            (Cursor::Controlled(Some(0)), 1, &[D, E], Some(0)),
            (Cursor::Controlled(Some(9)), 0, &[E, E, E], Some(2)),
            (Cursor::Controlled(None), 1, &[E, E], None),
            (Cursor::Auto, 0, &[], None),
            (Cursor::Controlled(Some(0)), 0, &[], None),
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
        assert!(Cursor::Auto.takes_focus());
        assert!(!Cursor::Controlled(None).takes_focus());
    }
}
