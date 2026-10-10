//! The palette machine's states, inputs, outputs and params.

use crate::text::typed::TypedText;
use crate::vocab::ShortcutKey;
use std::marker::PhantomData;

/// A position in the ranked rows, from 0.
#[derive(Debug, Clone, Copy, PartialEq, Eq, Hash, PartialOrd, Ord, Default)]
pub struct PaletteIndex(pub usize);

/// Whether the palette is open, over rows of type `T` (what [`PaletteOut::Run`] hands back).
///
/// The rows are not kept here: they are [`PaletteParams`], given at each step.
#[derive(Debug, Clone, PartialEq, Eq)]
pub enum PaletteState<T> {
    /// Not showing.
    Closed,
    /// Showing, with what was typed and the highlighted row.
    Open {
        query: TypedText,
        selection: PaletteIndex,
        /// Ties the state to its row type; holds nothing. Spell it [`PhantomData`], or build
        /// the state with [`PaletteState::open`].
        rows: PhantomData<fn() -> T>,
    },
}

// Not derived: a derive would ask `T: Default`, and a row type has no such need.
#[allow(clippy::derivable_impls)]
impl<T> Default for PaletteState<T> {
    fn default() -> Self {
        PaletteState::Closed
    }
}

impl<T> PaletteState<T> {
    /// Open on `query` with `selection` highlighted.
    pub const fn open(query: TypedText, selection: PaletteIndex) -> Self {
        PaletteState::Open {
            query,
            selection,
            rows: PhantomData,
        }
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

/// What the palette wants done.
#[derive(Debug, Clone, PartialEq, Eq)]
pub enum PaletteOut<T> {
    /// Show the palette and focus its field.
    Opened,
    /// Hide the palette and give focus back.
    Closed,
    /// Run this row.
    Run(T),
}

/// The rows for the query now in the field, best first. Matching and ranking happen outside,
/// each time the query changes; the palette only walks the list it is given, in the order it is
/// given. A caller with a fixed order (a new-tab bar: open tabs, then history, then search) puts
/// its rows in that order and the palette keeps it; rows that arrive later from a provider are
/// the next params, followed by [`PaletteIn::Refreshed`].
#[derive(Debug, Clone, PartialEq, Eq)]
pub struct PaletteParams<T> {
    /// The ranked rows.
    pub rows: Vec<T>,
}

impl<T> Default for PaletteParams<T> {
    fn default() -> Self {
        PaletteParams { rows: Vec::new() }
    }
}
