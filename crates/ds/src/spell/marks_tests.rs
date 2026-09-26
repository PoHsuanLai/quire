use super::{Edit, Misspelt, Typing, marks_for, reconcile, shown, typing_after};
use crate::edit::position::{EditNode, TextPosition};
use crate::spell::words::{Span, words};
use std::collections::HashSet;

fn node() -> EditNode {
    EditNode("p0".to_owned())
}

fn mark(start: usize, word: &str) -> Misspelt {
    Misspelt {
        node: node(),
        span: Span::new(start, start + word.len()),
        word: word.to_owned(),
    }
}

#[test]
fn a_paragraphs_marks_are_its_wrong_words() {
    let text = "teh cat teh";
    let wrong: HashSet<String> = ["teh".to_owned()].into();
    let marks = marks_for(&node(), text, &words(text, &[]), &wrong);
    assert_eq!(marks, vec![mark(0, "teh"), mark(8, "teh")]);
}

/// Marks as (start, word).
type At = &'static [(usize, &'static str)];

#[test]
fn marks_follow_an_edit() {
    // (before, after, marks before, marks after)
    let cases: &[(&str, &str, At, At)] = &[
        ("teh cat", "teh cat!", &[(0, "teh")], &[(0, "teh")]),
        ("a teh", "an teh", &[(2, "teh")], &[(3, "teh")]),
        ("a teh", "teh", &[(2, "teh")], &[(0, "teh")]),
        ("teh cat", "tehx cat", &[(0, "teh")], &[]),
        ("teh cat", "the cat", &[(0, "teh")], &[]),
        ("cat teh", "", &[(4, "teh")], &[]),
        (
            "x teh y teh",
            "xx teh y teh",
            &[(2, "teh"), (8, "teh")],
            &[(3, "teh"), (9, "teh")],
        ),
    ];
    for (before, after, from, to) in cases {
        let marks = from.iter().map(|(at, word)| mark(*at, word)).collect();
        let expected: Vec<Misspelt> = to.iter().map(|(at, word)| mark(*at, word)).collect();
        assert_eq!(
            reconcile(marks, before, after),
            expected,
            "{before:?} -> {after:?}"
        );
    }
}

#[test]
fn the_word_being_typed_is_held_until_the_caret_leaves_it() {
    let caret = |offset| TextPosition::new("p0", offset);
    let typing = |start, end| Typing {
        node: node(),
        span: Span::new(start, end),
    };
    // Typed "teh": the caret is at its end, so it is being typed.
    let held = typing_after(None, Some(&caret(3)), Some("teh"), Edit::Changed);
    assert_eq!(held, Some(typing(0, 3)));
    // The checker's read changes nothing: still held.
    let held = typing_after(held, Some(&caret(3)), Some("teh"), Edit::Same);
    assert_eq!(held, Some(typing(0, 3)));
    // A space: the caret no longer touches it.
    assert_eq!(
        typing_after(held.clone(), Some(&caret(4)), Some("teh "), Edit::Changed),
        None
    );
    // An arrow key away from it, the text unchanged.
    assert_eq!(
        typing_after(held.clone(), Some(&caret(0)), Some("teh cat"), Edit::Same),
        Some(typing(0, 3)),
        "still at its start"
    );
    assert_eq!(
        typing_after(held, Some(&caret(5)), Some("teh cat"), Edit::Same),
        None
    );
    // Clicking into a word already marked, without typing, holds nothing.
    assert_eq!(
        typing_after(None, Some(&caret(1)), Some("teh cat"), Edit::Same),
        None
    );
    // No caret: nothing is being typed.
    assert_eq!(typing_after(None, None, Some("teh"), Edit::Changed), None);
}

#[test]
fn every_mark_but_the_typed_word_is_shown() {
    let marks = vec![mark(0, "teh"), mark(4, "sat")];
    let held = Typing {
        node: node(),
        span: Span::new(4, 7),
    };
    let drawn: Vec<&Misspelt> = shown(&marks, Some(&held)).collect();
    assert_eq!(drawn, vec![&marks[0]]);
    assert_eq!(shown(&marks, None).count(), 2);
}

#[test]
fn a_mark_holds_the_positions_on_and_around_its_word() {
    let teh = mark(4, "teh");
    for (offset, held) in [(3, false), (4, true), (6, true), (7, true), (8, false)] {
        assert_eq!(
            teh.holds(&TextPosition::new("p0", offset)),
            held,
            "{offset}"
        );
    }
    assert!(!teh.holds(&TextPosition::new("p1", 5)), "another paragraph");
}
