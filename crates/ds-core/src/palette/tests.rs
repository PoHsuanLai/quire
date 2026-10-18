//! The palette machine, over a list of rows it only counts.

use super::model::{PaletteIn, PaletteIndex, PaletteMove, PaletteOut, PaletteParams, PaletteState};
use crate::machine::Machine;
use crate::text::typed::TypedText;
use crate::time::stamp::Stamp;
use crate::vocab::ShortcutKey;

type Palette = PaletteState;

/// A list of three rows.
const ROWS: usize = 3;

const fn open(query: &'static str, row: usize) -> Palette {
    Palette::open(TypedText::from_static(query), PaletteIndex(row))
}

/// Name, ranked rows, state before, input, state after, outputs.
type Case = (
    &'static str,
    usize,
    Palette,
    PaletteIn,
    Palette,
    &'static [PaletteOut],
);

const CASES: &[Case] = &[
    (
        "open shows an empty query on the first row",
        ROWS,
        Palette::Closed,
        PaletteIn::Open,
        open("", 0),
        &[PaletteOut::Opened],
    ),
    (
        "typing replaces the query and returns to the first row",
        ROWS,
        open("ex", 2),
        PaletteIn::Typed(TypedText::from_static("exp")),
        open("exp", 0),
        &[],
    ),
    (
        "down moves one row",
        ROWS,
        open("", 0),
        PaletteIn::Move(PaletteMove::Down),
        open("", 1),
        &[],
    ),
    (
        "down stops at the last row",
        ROWS,
        open("", 2),
        PaletteIn::Move(PaletteMove::Down),
        open("", 2),
        &[],
    ),
    (
        "up stops at the first row",
        ROWS,
        open("", 0),
        PaletteIn::Move(PaletteMove::Up),
        open("", 0),
        &[],
    ),
    (
        "last jumps to the end",
        ROWS,
        open("", 0),
        PaletteIn::Move(PaletteMove::Last),
        open("", 2),
        &[],
    ),
    (
        "first jumps to the start",
        ROWS,
        open("", 2),
        PaletteIn::Move(PaletteMove::First),
        open("", 0),
        &[],
    ),
    (
        "moving over no rows stays on row zero",
        0,
        open("zzz", 0),
        PaletteIn::Move(PaletteMove::Down),
        open("zzz", 0),
        &[],
    ),
    (
        "a shorter list pulls a stale selection back in range",
        1,
        open("e", 2),
        PaletteIn::Move(PaletteMove::Up),
        open("e", 0),
        &[],
    ),
    (
        "enter runs the highlighted row and closes",
        ROWS,
        open("", 1),
        PaletteIn::Enter,
        Palette::Closed,
        &[PaletteOut::Run(PaletteIndex(1)), PaletteOut::Closed],
    ),
    (
        "enter with no rows stays open",
        0,
        open("zzz", 0),
        PaletteIn::Enter,
        open("zzz", 0),
        &[],
    ),
    (
        "a click runs the row it landed on",
        ROWS,
        open("", 0),
        PaletteIn::Pick(PaletteIndex(2)),
        Palette::Closed,
        &[PaletteOut::Run(PaletteIndex(2)), PaletteOut::Closed],
    ),
    (
        "a click past the rows is nothing",
        ROWS,
        open("", 0),
        PaletteIn::Pick(PaletteIndex(9)),
        open("", 0),
        &[],
    ),
    (
        "escape closes",
        ROWS,
        open("ex", 1),
        PaletteIn::Close,
        Palette::Closed,
        &[PaletteOut::Closed],
    ),
    (
        "opening again keeps the query",
        ROWS,
        open("ex", 1),
        PaletteIn::Open,
        open("ex", 1),
        &[],
    ),
    (
        "a closed palette ignores enter",
        ROWS,
        Palette::Closed,
        PaletteIn::Enter,
        Palette::Closed,
        &[],
    ),
];

#[test]
fn every_row_of_the_table_steps_as_written() {
    for (name, rows, from, input, state, outs) in CASES {
        let params = PaletteParams { rows: *rows };
        let (next, out) = from.clone().step(input.clone(), Stamp(0), &params, &());
        assert_eq!(next, *state, "{name}: state");
        assert_eq!(out.as_slice(), *outs, "{name}: outputs");
        assert_eq!(next.wake(), None, "{name}: a palette keeps no timer");
    }
}

#[test]
fn keys_mean_moves_enter_and_escape() {
    // name, keys, input
    const KEYS: &[(&str, &[ShortcutKey], Option<PaletteIn>)] = &[
        (
            "up",
            &[ShortcutKey::Up],
            Some(PaletteIn::Move(PaletteMove::Up)),
        ),
        (
            "down",
            &[ShortcutKey::Down],
            Some(PaletteIn::Move(PaletteMove::Down)),
        ),
        ("enter", &[ShortcutKey::Enter], Some(PaletteIn::Enter)),
        ("escape", &[ShortcutKey::Escape], Some(PaletteIn::Close)),
        (
            "command k closes",
            &[ShortcutKey::Super, ShortcutKey::Char('k')],
            Some(PaletteIn::Close),
        ),
        (
            "a letter is typed, not a key",
            &[ShortcutKey::Char('a')],
            None,
        ),
    ];
    for (name, keys, want) in KEYS {
        assert_eq!(PaletteIn::from_key(keys), *want, "{name}");
    }
}

#[test]
fn a_refresh_pulls_a_stale_selection_into_range_and_leaves_a_good_one() {
    // name, rows now, state before, state after
    let cases: &[(&str, usize, Palette, Palette)] = &[
        ("past the end", 1, open("e", 2), open("e", 0)),
        ("onto the last row", 2, open("e", 5), open("e", 1)),
        ("in range stays", 3, open("e", 1), open("e", 1)),
        ("no rows", 0, open("e", 3), open("e", 0)),
        ("closed stays closed", 1, Palette::Closed, Palette::Closed),
    ];
    for (name, rows, from, want) in cases {
        let params = PaletteParams { rows: *rows };
        let (next, out) = from
            .clone()
            .step(PaletteIn::Refreshed, Stamp(0), &params, &());
        assert_eq!(next, *want, "{name}");
        assert!(out.is_empty(), "{name}: a refresh runs nothing");
    }
}

#[test]
fn rows_that_change_while_open_apply_from_the_next_step() {
    let first = PaletteParams { rows: 1 };
    let later = PaletteParams { rows: 2 };
    let state = open("q", 0);
    let (state, _) = state.step(PaletteIn::Move(PaletteMove::Down), Stamp(0), &first, &());
    assert_eq!(state, open("q", 0), "one row: down stays put");
    let (state, _) = state.step(PaletteIn::Move(PaletteMove::Down), Stamp(1), &later, &());
    assert_eq!(
        state,
        open("q", 1),
        "the provider's second row can be reached"
    );
    let (state, out) = state.step(PaletteIn::Enter, Stamp(2), &later, &());
    assert_eq!(state, Palette::Closed);
    assert_eq!(
        out,
        vec![PaletteOut::Run(PaletteIndex(1)), PaletteOut::Closed]
    );
}

#[test]
fn the_highlight_walks_the_rows_in_order() {
    let params = PaletteParams { rows: ROWS };
    let mut state = open("", 0);
    let mut seen = vec![];
    for step in 0..3 {
        let (picked, out) = state.clone().step(PaletteIn::Enter, Stamp(0), &params, &());
        assert_eq!(picked, Palette::Closed);
        seen.push(out[0]);
        let (next, _) = state.step(
            PaletteIn::Move(PaletteMove::Down),
            Stamp(step),
            &params,
            &(),
        );
        state = next;
    }
    assert_eq!(
        seen,
        vec![
            PaletteOut::Run(PaletteIndex(0)),
            PaletteOut::Run(PaletteIndex(1)),
            PaletteOut::Run(PaletteIndex(2))
        ]
    );
}
