//! The crash guard for styles (customization design section 7.4): the pure decision as a table,
//! and the record file's life in a scratch state directory (tests never touch the real system).

use ds::style::kit::{Fingerprint, LookStyle, Styles, UserStyle};
use ds_blitz::StylesGuard;
use ds_blitz::styles_guard::{Load, Reached, Record, decide, parse, render};
use std::path::PathBuf;

const A: Fingerprint = Fingerprint(0xa);
const B: Fingerprint = Fingerprint(0xb);

const fn start(hash: Fingerprint) -> Record {
    Record {
        hash,
        reached: Reached::Starting,
    }
}

const fn good(hash: Fingerprint) -> Record {
    Record {
        hash,
        reached: Reached::Good,
    }
}

#[test]
fn two_unfinished_starts_with_the_same_sheets_turn_them_off() {
    struct Case {
        name: &'static str,
        records: &'static [Record],
        now: Fingerprint,
        expect: Load,
    }
    const CASES: &[Case] = &[
        Case {
            name: "no history",
            records: &[],
            now: A,
            expect: Load::WithStyles,
        },
        Case {
            name: "one unfinished start",
            records: &[start(A)],
            now: A,
            expect: Load::WithStyles,
        },
        Case {
            name: "two unfinished starts",
            records: &[start(A), start(A)],
            now: A,
            expect: Load::Without,
        },
        Case {
            name: "more than two unfinished starts",
            records: &[start(A), start(A), start(A)],
            now: A,
            expect: Load::Without,
        },
        Case {
            name: "the sheets changed since",
            records: &[start(A), start(A)],
            now: B,
            expect: Load::WithStyles,
        },
        Case {
            name: "the earlier start was a different set",
            records: &[start(B), start(A)],
            now: A,
            expect: Load::WithStyles,
        },
        Case {
            name: "the last start was good",
            records: &[start(A), good(A)],
            now: A,
            expect: Load::WithStyles,
        },
        Case {
            name: "a good start between two unfinished ones",
            records: &[start(A), good(A), start(A)],
            now: A,
            expect: Load::WithStyles,
        },
        Case {
            name: "an old failure under a good run",
            records: &[start(A), start(A), good(A)],
            now: A,
            expect: Load::WithStyles,
        },
    ];
    for case in CASES {
        assert_eq!(decide(case.records, case.now), case.expect, "{}", case.name);
    }
}

#[test]
fn the_record_file_round_trips_and_a_damaged_one_is_no_history() {
    let records = [start(A), good(B), start(Fingerprint(u64::MAX))];
    assert_eq!(parse(&render(&records)), records);
    assert_eq!(parse(""), []);
    assert_eq!(parse("not toml at all\n[[start]]\nhash = \"zz\"\n"), []);
    // A half-written record (no state) is dropped, the whole one before it kept.
    let torn = format!(
        "{}\n[[start]]\nhash = \"000000000000000c\"\n",
        render(&[good(A)])
    );
    assert_eq!(parse(&torn), [good(A)]);
}

fn scratch(name: &str) -> PathBuf {
    let dir = PathBuf::from(env!("CARGO_TARGET_TMPDIR")).join(format!("styles-guard-{name}"));
    let _ = std::fs::remove_dir_all(&dir);
    dir
}

fn styles(app: &str) -> Styles {
    Styles {
        look: LookStyle(".look{}".to_owned()),
        app: UserStyle(app.to_owned()),
        ..Styles::default()
    }
}

#[test]
fn a_start_is_written_as_starting_then_good_in_the_programs_own_file() {
    let state = scratch("good");
    let guard = StylesGuard::begin(&state, "mailo", &styles(".a{}"));
    assert_eq!(guard.load(), Load::WithStyles);
    let file = state.join("styles-guard").join("mailo");
    let text = std::fs::read_to_string(&file).expect("the record is written before the load");
    assert_eq!(parse(&text), [start(guard.fingerprint())], "{text}");
    guard.mark_good();
    let text = std::fs::read_to_string(&file).expect("the record stays");
    assert_eq!(parse(&text), [good(guard.fingerprint())], "{text}");
}

#[test]
fn two_starts_that_never_got_good_make_the_third_start_without_styles() {
    let state = scratch("crash");
    let sheets = styles(".boom{}");
    let first = StylesGuard::begin(&state, "mailo", &sheets);
    let second = StylesGuard::begin(&state, "mailo", &sheets);
    assert_eq!(
        [first.load(), second.load()],
        [Load::WithStyles, Load::WithStyles]
    );
    let third = StylesGuard::begin(&state, "mailo", &sheets);
    assert_eq!(third.load(), Load::Without);
    // It stays off on the fourth start too: nothing was marked good.
    assert_eq!(
        StylesGuard::begin(&state, "mailo", &sheets).load(),
        Load::Without
    );
    // Another program has its own file.
    assert_eq!(
        StylesGuard::begin(&state, "sill", &sheets).load(),
        Load::WithStyles
    );
}

#[test]
fn a_changed_sheet_or_turn_on_again_clears_the_guard() {
    let state = scratch("clear");
    let broken = styles(".boom{}");
    for _ in 0..2 {
        StylesGuard::begin(&state, "mailo", &broken);
    }
    let off = StylesGuard::begin(&state, "mailo", &broken);
    assert_eq!(off.load(), Load::Without);
    // The same sheets: the guard stands.
    assert_eq!(off.changed(&broken), off);
    // A fixed sheet: a fresh try.
    let fixed = off.changed(&styles(".fixed{}"));
    assert_eq!(fixed.load(), Load::WithStyles);
    assert_eq!(
        StylesGuard::begin(&state, "mailo", &broken).load(),
        Load::WithStyles,
        "the failures were forgotten"
    );
    // Turn On Again with the very same sheets also forgets them.
    for _ in 0..2 {
        StylesGuard::begin(&state, "sill", &broken);
    }
    let off = StylesGuard::begin(&state, "sill", &broken);
    assert_eq!(off.load(), Load::Without);
    assert_eq!(off.turn_on_again(&broken).load(), Load::WithStyles);
}
