//! What a `ColumnBrowser` decides, pure: what a key asks, which column the keys are in and which
//! row they move to.

use crate::components::lists::column_browser::item::{
    BrowserColumn, BrowserRowKind, ColumnPick,
};
use crate::components::lists::list::keys::{ListKey, list_key};
use crate::components::lists::virtual_list::model::roved;
use crate::stack::roving::Rove;
use chordkit::Platform;
use dioxus::prelude::{Key, Modifiers};
use ds_core::command::holds_primary;
use ds_core::geometry::units::Px;
use ds_core::selection::{Pick, PickHow};

/// What a key asks of the columns.
#[derive(Debug, Clone, PartialEq, Eq)]
pub(crate) enum BrowserKey {
    /// The cursor moves within its column.
    Move(Rove),
    /// Right: into the selected row's column.
    Enter,
    /// Left: back to the parent column.
    Leave,
    /// The primary modifier with Down (Command-Down, Finder's Open).
    Open,
    /// Letters typed: the cursor jumps to the next name that starts with them.
    Jump(String),
}

/// The browser's reading of `key` on `platform`; `None` for a key it leaves alone. As in
/// `CollectionView`, Shift alone still moves, any other modifier makes the key the app's, and the
/// one exception is the primary modifier with Down. Return and Space are the app's (Finder's
/// Rename and Quick Look).
pub(crate) fn browser_key(
    key: &Key,
    modifiers: Modifiers,
    platform: Platform,
) -> Option<BrowserKey> {
    let chorded = modifiers
        .intersects(Modifiers::CONTROL | Modifiers::ALT | Modifiers::SUPER | Modifiers::META);
    if chorded {
        let primary_alone = holds_primary(platform, modifiers)
            && !modifiers.intersects(Modifiers::SHIFT | Modifiers::ALT);
        return (primary_alone && *key == Key::ArrowDown).then_some(BrowserKey::Open);
    }
    match key {
        Key::ArrowRight => Some(BrowserKey::Enter),
        Key::ArrowLeft => Some(BrowserKey::Leave),
        _ => match list_key(key, modifiers)? {
            ListKey::Move(rove) => Some(BrowserKey::Move(rove)),
            ListKey::Pick => None,
            ListKey::Jump(text) => Some(BrowserKey::Jump(text)),
        },
    }
}

/// The column the keys are in: the last one with a selection, else the first.
pub(crate) fn active<K>(columns: &[BrowserColumn<K>]) -> usize {
    columns
        .iter()
        .rposition(|column| !column.selection.is_empty())
        .unwrap_or(0)
}

/// The row the keys rest on in the active column.
pub(crate) fn cursor<K>(columns: &[BrowserColumn<K>]) -> Option<usize> {
    let column = columns.get(active(columns))?;
    column
        .selection
        .cursor()
        .filter(|row| *row < column.rows.len())
}

/// Where `key` takes the keys, with `how` a move is reached (Shift extends): `None` for a key
/// that goes nowhere (Left in the first column, Right on a leaf). Left re-picks the parent's
/// selected row, which is how the app learns to drop the columns after it.
pub(crate) fn target<K>(
    columns: &[BrowserColumn<K>],
    key: &BrowserKey,
    how: PickHow,
) -> Option<ColumnPick> {
    let here = active(columns);
    let column = columns.get(here)?;
    let at = cursor(columns);
    let reached = |column: usize, row: usize, how: PickHow| ColumnPick {
        column,
        pick: Pick { row, how },
    };
    match key {
        BrowserKey::Move(rove) => {
            roved(at, column.rows.len(), *rove).map(|row| reached(here, row, how))
        }
        BrowserKey::Enter => {
            let into = columns.get(here + 1)?;
            let branch = at.and_then(|row| column.rows.get(row))?.kind == BrowserRowKind::Branch;
            (branch && !into.rows.is_empty()).then(|| reached(here + 1, 0, PickHow::Replace))
        }
        BrowserKey::Leave => {
            let parent = here.checked_sub(1)?;
            let row = columns.get(parent)?.selection.cursor()?;
            Some(reached(parent, row, PickHow::Replace))
        }
        BrowserKey::Open => at.map(|row| reached(here, row, PickHow::Replace)),
        BrowserKey::Jump(_) => None,
    }
}

/// Whether the last column's selection is a row with nothing under it: the preview shows.
pub(crate) fn leaf_selected<K>(columns: &[BrowserColumn<K>]) -> bool {
    columns.last().is_some_and(|column| {
        column
            .selection
            .cursor()
            .and_then(|row| column.rows.get(row))
            .is_some_and(|row| row.kind == BrowserRowKind::Leaf)
    })
}

/// The room a name cell has in a column `width` wide: the row's padding, its gap and the
/// chevron taken out.
pub(crate) fn room(width: Px) -> Px {
    Px(width.0 - ROW_SPACE)
}

/// What a row spends besides its name cell: padding on both sides (`--s-8`), the gap before the
/// chevron and the chevron itself.
const ROW_SPACE: f32 = 40.0;

#[cfg(test)]
mod tests {
    use super::{BrowserKey, active, browser_key, cursor, leaf_selected, target};
    use crate::components::content::icon_source::IconSource;
    use crate::components::lists::column_browser::item::{BrowserColumn, BrowserRow};
    use crate::stack::roving::{Edge, Rove, Step};
    use chordkit::Platform;
    use dioxus::prelude::{Key, Modifiers};
    use ds_core::selection::{PickHow, RowSelection};
    use ds_style::icon::Icon;

    fn rows(names: &[(&str, bool)]) -> Vec<BrowserRow<String>> {
        let icon = IconSource::Glyph(Icon::File);
        names
            .iter()
            .map(|(name, branch)| match branch {
                true => BrowserRow::branch(name.to_string(), *name, icon.clone()),
                false => BrowserRow::leaf(name.to_string(), *name, icon.clone()),
            })
            .collect()
    }

    /// Home > (Docs, Music, notes.txt); Docs selected > (a.pdf, b.pdf) with b selected.
    fn two() -> Vec<BrowserColumn<String>> {
        vec![
            BrowserColumn::new(rows(&[("Docs", true), ("Music", true), ("notes.txt", false)]))
                .selected(RowSelection::single(0)),
            BrowserColumn::new(rows(&[("a.pdf", false), ("b.pdf", false)]))
                .selected(RowSelection::single(1)),
        ]
    }

    #[test]
    fn keys_are_read_as_moves_enter_leave_open_and_letters_and_the_rest_are_left_alone() {
        let plain = Modifiers::empty();
        let cases: &[(Key, Modifiers, Option<BrowserKey>)] = &[
            (Key::ArrowRight, plain, Some(BrowserKey::Enter)),
            (Key::ArrowLeft, plain, Some(BrowserKey::Leave)),
            (
                Key::ArrowDown,
                plain,
                Some(BrowserKey::Move(Rove::Step(Step::Down))),
            ),
            (
                Key::ArrowUp,
                Modifiers::SHIFT,
                Some(BrowserKey::Move(Rove::Step(Step::Up))),
            ),
            (Key::End, plain, Some(BrowserKey::Move(Rove::Edge(Edge::Last)))),
            (Key::Enter, plain, None),
            (Key::ArrowRight, Modifiers::ALT, None),
            (Key::ArrowDown, Modifiers::SUPER, Some(BrowserKey::Open)),
            (
                Key::Character("m".to_owned()),
                plain,
                Some(BrowserKey::Jump("m".to_owned())),
            ),
        ];
        for (key, modifiers, want) in cases {
            assert_eq!(
                browser_key(key, *modifiers, Platform::Mac),
                *want,
                "{key:?} {modifiers:?}"
            );
        }
    }

    #[test]
    fn the_keys_are_in_the_last_column_with_a_selection() {
        let mut columns = two();
        assert_eq!((active(&columns), cursor(&columns)), (1, Some(1)));
        columns[1].selection = RowSelection::none();
        assert_eq!((active(&columns), cursor(&columns)), (0, Some(0)));
        columns[0].selection = RowSelection::none();
        assert_eq!((active(&columns), cursor(&columns)), (0, None));
    }

    #[test]
    fn up_and_down_move_in_the_active_column_and_stop_at_its_ends() {
        let columns = two();
        let down = BrowserKey::Move(Rove::Step(Step::Down));
        let up = BrowserKey::Move(Rove::Step(Step::Up));
        let pick = target(&columns, &down, PickHow::Replace).expect("a move");
        assert_eq!((pick.column, pick.pick.row), (1, 1), "the last row stays");
        let pick = target(&columns, &up, PickHow::Extend).expect("a move");
        assert_eq!((pick.column, pick.pick.row), (1, 0));
        assert_eq!(pick.pick.how, PickHow::Extend);
    }

    #[test]
    fn right_enters_the_child_column_only_from_a_branch_with_rows_and_left_re_picks_the_parent() {
        let mut columns = two();
        columns[1].selection = RowSelection::none();
        let pick = target(&columns, &BrowserKey::Enter, PickHow::Replace).expect("enter");
        assert_eq!((pick.column, pick.pick.row), (1, 0));
        columns[0].selection = RowSelection::single(2);
        assert_eq!(
            target(&columns, &BrowserKey::Enter, PickHow::Replace),
            None,
            "a leaf has no column"
        );
        let columns = two();
        let pick = target(&columns, &BrowserKey::Leave, PickHow::Replace).expect("leave");
        assert_eq!((pick.column, pick.pick.row), (0, 0));
        let first = vec![two().remove(0)];
        assert_eq!(target(&first, &BrowserKey::Leave, PickHow::Replace), None);
    }

    #[test]
    fn open_names_the_cursor_row_and_a_leaf_selection_shows_the_preview() {
        let columns = two();
        let pick = target(&columns, &BrowserKey::Open, PickHow::Replace).expect("open");
        assert_eq!((pick.column, pick.pick.row), (1, 1));
        assert!(leaf_selected(&columns));
        assert!(!leaf_selected(&columns[..1]), "a folder is selected");
        assert!(!leaf_selected(&[]));
    }
}
