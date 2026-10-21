//! What a `VirtualTable` decides, pure: which row a key moves the cursor to.

use crate::components::lists::list::keys::{ListKey, list_key};
use crate::components::lists::virtual_list::model::roved;
use crate::stack::roving::{Rove, Step};
use chordkit::Platform;
use dioxus::prelude::{Key, Modifiers};
use ds_core::command::holds_primary;
use ds_core::selection::PickHow;

/// What a key asks of a virtual table.
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub(crate) enum TableKey {
    /// The cursor moves by a row, or to an end.
    Move(Rove),
    /// The cursor moves by a page.
    Page(Step),
    /// The row under the cursor is picked (Return or Space).
    Pick,
    /// The primary modifier with Down (Command-Down, Finder's Open): the row under the cursor is
    /// opened.
    Open,
}

/// Whether `modifiers` hold a modifier that makes a key a command and not a movement.
fn chorded(modifiers: Modifiers) -> bool {
    modifiers.intersects(Modifiers::CONTROL | Modifiers::ALT | Modifiers::SUPER | Modifiers::META)
}

/// The table's reading of `key` on `platform`; `None` for a key it leaves alone. Letters are not
/// read: the rows are not at hand to match a label against. Shift alone still moves (and extends);
/// any other modifier makes the key the app's, so Command-Up (Enclosing Folder) goes on to the
/// app's keymap. The one exception is the primary modifier with Down, Finder's Open.
pub(crate) fn table_key(key: &Key, modifiers: Modifiers, platform: Platform) -> Option<TableKey> {
    if chorded(modifiers) {
        let primary_alone = holds_primary(platform, modifiers)
            && !modifiers.intersects(Modifiers::SHIFT | Modifiers::ALT);
        return (primary_alone && *key == Key::ArrowDown).then_some(TableKey::Open);
    }
    match key {
        Key::PageDown => Some(TableKey::Page(Step::Down)),
        Key::PageUp => Some(TableKey::Page(Step::Up)),
        _ => match list_key(key, modifiers)? {
            ListKey::Move(rove) => Some(TableKey::Move(rove)),
            ListKey::Pick => Some(TableKey::Pick),
            ListKey::Jump(_) => None,
        },
    }
}

/// Where the cursor goes from `at` among `len` rows for `key`, a page being `page` rows; `None`
/// for an empty table or a key that does not move it. The cursor stops at the ends. A page with
/// no cursor starts from the first row going down and the last going up.
pub(crate) fn target(at: Option<usize>, len: usize, key: TableKey, page: usize) -> Option<usize> {
    let last = len.checked_sub(1)?;
    let page = page.max(1);
    match key {
        TableKey::Move(rove) => roved(at, len, rove),
        TableKey::Page(Step::Down) => Some(at.map_or(0, |at| (at + page).min(last)).min(last)),
        TableKey::Page(Step::Up) => Some(at.map_or(last, |at| at.saturating_sub(page)).min(last)),
        TableKey::Pick | TableKey::Open => None,
    }
}

/// How many of `total` columns are really held in place when `asked` are: at least one column
/// is left to scroll, so a table of one column holds none.
pub(crate) fn held_columns(asked: usize, total: usize) -> usize {
    asked.min(total.saturating_sub(1))
}

/// How an arrow, Home, End, Page Up or Page Down reaches its row: with Shift it extends the
/// selection from its anchor, otherwise it replaces the selection.
pub(crate) fn key_how(modifiers: Modifiers) -> PickHow {
    match modifiers.contains(Modifiers::SHIFT) {
        true => PickHow::Extend,
        false => PickHow::Replace,
    }
}

/// How many whole rows of `pitch` the viewport `height` shows, at least one.
pub(crate) fn page_rows(height: f32, pitch: f32) -> usize {
    match pitch > 0.0 && height.is_finite() {
        true => ((height / pitch).floor() as usize).max(1),
        false => 1,
    }
}

#[cfg(test)]
mod tests {
    use super::{TableKey, held_columns, key_how, page_rows, table_key, target};
    use crate::stack::roving::{Edge, Rove, Step};
    use chordkit::Platform;
    use dioxus::prelude::{Key, Modifiers};
    use ds_core::selection::PickHow;

    #[test]
    fn the_cursor_moves_by_a_row_an_end_or_a_page_and_stops_at_the_ends() {
        let down = TableKey::Move(Rove::Step(Step::Down));
        let up = TableKey::Move(Rove::Step(Step::Up));
        let end = TableKey::Move(Rove::Edge(Edge::Last));
        let page_down = TableKey::Page(Step::Down);
        let page_up = TableKey::Page(Step::Up);
        // (cursor, rows, key, page, wanted)
        type Case = (Option<usize>, usize, TableKey, usize, Option<usize>);
        let cases: &[Case] = &[
            (None, 100, down, 10, Some(0)),
            (Some(5), 100, down, 10, Some(6)),
            (Some(99), 100, down, 10, Some(99)),
            (Some(5), 100, up, 10, Some(4)),
            (Some(5), 100, end, 10, Some(99)),
            (Some(5), 100, page_down, 10, Some(15)),
            (Some(95), 100, page_down, 10, Some(99)),
            (Some(5), 100, page_up, 10, Some(0)),
            (Some(50), 100, page_up, 10, Some(40)),
            (None, 100, page_down, 10, Some(0)),
            (None, 100, page_up, 10, Some(99)),
            (Some(5), 100, page_down, 0, Some(6)),
            (Some(150), 100, page_down, 10, Some(99)),
            (Some(0), 0, page_down, 10, None),
            (Some(3), 100, TableKey::Pick, 10, None),
        ];
        for &(at, len, key, page, want) in cases {
            assert_eq!(target(at, len, key, page), want, "{at:?} of {len} {key:?}");
        }
    }

    #[test]
    fn a_key_is_read_as_a_move_a_page_or_a_pick_and_letters_are_left_alone() {
        let cases: &[(Key, Option<TableKey>)] = &[
            (Key::PageDown, Some(TableKey::Page(Step::Down))),
            (Key::PageUp, Some(TableKey::Page(Step::Up))),
            (Key::ArrowDown, Some(TableKey::Move(Rove::Step(Step::Down)))),
            (Key::Home, Some(TableKey::Move(Rove::Edge(Edge::First)))),
            (Key::Enter, Some(TableKey::Pick)),
            (Key::Character(" ".to_owned()), Some(TableKey::Pick)),
            (Key::Character("a".to_owned()), None),
            (Key::Tab, None),
        ];
        for (key, want) in cases {
            let read = table_key(key, Modifiers::empty(), Platform::MacOs);
            assert_eq!(read, *want, "{key:?}");
        }
    }

    #[test]
    fn a_modified_arrow_is_the_apps_except_shift_and_the_primary_modifier_with_down() {
        let mac = Platform::MacOs;
        let down = Key::ArrowDown;
        let up = Key::ArrowUp;
        let (home, page_down, enter) = (Key::Home, Key::PageDown, Key::Enter);
        let step_down = Some(TableKey::Move(Rove::Step(Step::Down)));
        // (key, modifiers, wanted)
        let cases: &[(&Key, Modifiers, Option<TableKey>)] = &[
            (&down, Modifiers::SHIFT, step_down),
            (&down, Modifiers::SUPER, Some(TableKey::Open)),
            (&up, Modifiers::SUPER, None),
            (&down, Modifiers::SUPER | Modifiers::SHIFT, None),
            (&down, Modifiers::ALT, None),
            (&down, Modifiers::CONTROL, None),
            (&home, Modifiers::SUPER, None),
            (&page_down, Modifiers::ALT, None),
            (&enter, Modifiers::SUPER, None),
        ];
        for &(key, modifiers, want) in cases {
            assert_eq!(
                table_key(key, modifiers, mac),
                want,
                "{key:?} {modifiers:?}"
            );
        }
    }

    #[test]
    fn a_page_is_the_whole_rows_the_viewport_shows() {
        // (viewport height, pitch, wanted)
        let cases = [
            (168.0, 32.0, 5),
            (160.0, 32.0, 5),
            (10.0, 32.0, 1),
            (0.0, 32.0, 1),
            (100.0, 0.0, 1),
            (f32::NAN, 32.0, 1),
        ];
        for (height, pitch, want) in cases {
            assert_eq!(page_rows(height, pitch), want, "{height} over {pitch}");
        }
    }

    #[test]
    fn at_least_one_column_is_left_to_scroll_and_shift_extends() {
        // (asked, columns, held)
        let cases = [
            (0, 5, 0),
            (2, 5, 2),
            (5, 5, 4),
            (9, 5, 4),
            (2, 1, 0),
            (2, 0, 0),
        ];
        for (asked, total, held) in cases {
            assert_eq!(held_columns(asked, total), held, "{asked} of {total}");
        }
        assert_eq!(key_how(Modifiers::SHIFT), PickHow::Extend);
        assert_eq!(key_how(Modifiers::SHIFT | Modifiers::ALT), PickHow::Extend);
        assert_eq!(key_how(Modifiers::empty()), PickHow::Replace);
        assert_eq!(key_how(Modifiers::CONTROL), PickHow::Replace);
    }
}
