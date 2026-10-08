//! The palette machine's transitions.

use super::model::{PaletteIn, PaletteIndex, PaletteMove, PaletteOut, PaletteParams, PaletteState};
use ds_core::machine::Machine;
use ds_core::text::typed::TypedText;
use ds_core::time::stamp::Stamp;

type Step<T> = (PaletteState<T>, Vec<PaletteOut<T>>);

impl<T: Clone + PartialEq + 'static> Machine for PaletteState<T> {
    type In = PaletteIn;
    type Out = PaletteOut<T>;
    type Params = PaletteParams<T>;
    type Ctx = ();

    fn step(self, input: PaletteIn, _at: Stamp, params: &PaletteParams<T>, _cx: &()) -> Step<T> {
        match self {
            PaletteState::Closed => closed(input),
            PaletteState::Open {
                query, selection, ..
            } => open(query, selection, input, params),
        }
    }

    fn wake(&self) -> Option<Stamp> {
        None
    }
}

fn closed<T>(input: PaletteIn) -> Step<T> {
    match input {
        PaletteIn::Open => {
            let state = PaletteState::open(TypedText::EMPTY, PaletteIndex(0));
            (state, vec![PaletteOut::Opened])
        }
        PaletteIn::Typed(_)
        | PaletteIn::Move(_)
        | PaletteIn::Pick(_)
        | PaletteIn::Enter
        | PaletteIn::Close
        | PaletteIn::Refreshed
        | PaletteIn::Elapsed => (PaletteState::Closed, vec![]),
    }
}

fn open<T: Clone>(
    query: TypedText,
    selection: PaletteIndex,
    input: PaletteIn,
    params: &PaletteParams<T>,
) -> Step<T> {
    let this = PaletteState::open(query.clone(), selection);
    match input {
        PaletteIn::Typed(text) => (PaletteState::open(text, PaletteIndex(0)), vec![]),
        PaletteIn::Move(movement) => {
            let selection = moved(selection, movement, params.rows.len());
            (PaletteState::open(query, selection), vec![])
        }
        PaletteIn::Refreshed => {
            let selection = in_range(selection, params.rows.len());
            (PaletteState::open(query, selection), vec![])
        }
        PaletteIn::Enter => run(this, selection, params),
        PaletteIn::Pick(row) => run(this, row, params),
        PaletteIn::Close => (PaletteState::Closed, vec![PaletteOut::Closed]),
        PaletteIn::Open | PaletteIn::Elapsed => (this, vec![]),
    }
}

/// `row`'s value, with the palette closed after it; nothing when there is no such row.
fn run<T: Clone>(this: PaletteState<T>, row: PaletteIndex, params: &PaletteParams<T>) -> Step<T> {
    match params.rows.get(row.0) {
        Some(value) => (
            PaletteState::Closed,
            vec![PaletteOut::Run(value.clone()), PaletteOut::Closed],
        ),
        None => (this, vec![]),
    }
}

/// The highlight after `movement` over `rows` rows, clamped to the first and last.
fn moved(selection: PaletteIndex, movement: PaletteMove, rows: usize) -> PaletteIndex {
    let last = rows.saturating_sub(1);
    let target = match movement {
        PaletteMove::Up => selection.0.saturating_sub(1),
        PaletteMove::Down => selection.0.saturating_add(1),
        PaletteMove::First => 0,
        PaletteMove::Last => last,
    };
    PaletteIndex(target.min(last))
}

/// `selection`, or the last row when it is past it.
fn in_range(selection: PaletteIndex, rows: usize) -> PaletteIndex {
    PaletteIndex(selection.0.min(rows.saturating_sub(1)))
}
