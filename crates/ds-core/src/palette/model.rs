//! The palette machine's states, inputs, outputs and params.

use crate::text::typed::TypedText;
use crate::vocab::ShortcutKey;

/// A position in the ranked rows, from 0.
#[derive(Debug, Clone, Copy, PartialEq, Eq, Hash, PartialOrd, Ord, Default)]
pub struct PaletteIndex(pub usize);

/// Whether the palette is open.
///
/// The rows are not kept here: only how many there are, in [`PaletteParams`], given at each step.
#[derive(Debug, Clone, PartialEq, Eq, Default)]
pub enum PaletteState {
    /// Not showing.
    #[default]
    Closed,
    /// Showing, with what was typed and the highlighted row.
    Open {
        query: TypedText,
        selection: PaletteIndex,
    },
}

impl PaletteState {
    /// Open on `query` with `selection` highlighted.
    pub const fn open(query: TypedText, selection: PaletteIndex) -> Self {
        PaletteState::Open { query, selection }
    }
}

/// Where the highlight goes.
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum PaletteMove {
    /// One row up.
    Up,
    /// One row down.
    Down,
    /// The first row.
    First,
    /// The last row.
    Last,
}

/// What moves the palette.
#[derive(Debug, Clone, PartialEq, Eq)]
pub enum PaletteIn {
    /// Open with an empty query.
    Open,
    /// The field now holds this text; the rows are re-ranked for it.
    Typed(TypedText),
    /// Move the highlight; it stops at the first and last row.
    Move(PaletteMove),
    /// A row was clicked.
    Pick(PaletteIndex),
    /// Enter: run the highlighted row.
    Enter,
    /// Esc or a click outside.
    Close,
    /// The rows changed under an open palette (a provider answered after the query was typed):
    /// pull a highlight that is now past the last row back onto it. The palette never sees the
    /// rows change by itself, so a caller whose rows can change without a new query sends this.
    Refreshed,
    /// The clock; the palette keeps no timer.
    Elapsed,
}

impl From<crate::machine::Elapsed> for PaletteIn {
    fn from(_: crate::machine::Elapsed) -> Self {
        PaletteIn::Elapsed
    }
}

impl PaletteIn {
    /// What a key means to an open palette: arrows move, Enter runs, Esc and ⌘K close. Anything
    /// else is typed into the field, which reports it as `Typed`.
    pub fn from_key(keys: &[ShortcutKey]) -> Option<PaletteIn> {
        match keys {
            [ShortcutKey::Up] => Some(PaletteIn::Move(PaletteMove::Up)),
            [ShortcutKey::Down] => Some(PaletteIn::Move(PaletteMove::Down)),
            [ShortcutKey::Enter] => Some(PaletteIn::Enter),
            [ShortcutKey::Escape] | [ShortcutKey::Super, ShortcutKey::Char('k')] => {
                Some(PaletteIn::Close)
            }
            _ => None,
        }
    }
}

/// What the palette wants done. A row is named by its index in the list the step ran over: the
/// caller maps it back through the rows it passed as params.
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum PaletteOut {
    /// Show the palette and focus its field.
    Opened,
    /// Hide the palette and give focus back.
    Closed,
    /// Run the row at this index.
    Run(PaletteIndex),
}

/// How many rows the query now in the field lists. Matching and ranking happen outside, each time
/// the query changes; the palette only walks the list, in the order the caller gives it. A caller
/// with a fixed order (a new-tab bar: open tabs, then history, then search) puts its rows in that
/// order and the palette keeps it; rows that arrive later from a provider are the next params,
/// followed by [`PaletteIn::Refreshed`].
#[derive(Debug, Clone, Copy, PartialEq, Eq, Default)]
pub struct PaletteParams {
    /// How many ranked rows there are.
    pub rows: usize,
}
