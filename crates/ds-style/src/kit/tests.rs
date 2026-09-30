//! Kits order the cascade by rank and place each family by its slot, whatever order they are
//! handed in.

use super::{Kit, KitRank, Kits, Section, Vocabulary};
use crate::tokens::{
    set::{Place, TokenSet},
    spacing::SpacingToken,
    timing::DurationToken,
};
use std::borrow::Cow;

fn first(_: &Kits) -> Cow<'static, str> {
    Cow::Borrowed(".a{}")
}

fn second(_: &Kits) -> Cow<'static, str> {
    Cow::Borrowed(".b{}")
}

fn tokens(kits: &Kits) -> Cow<'static, str> {
    Cow::Owned(kits.token_blocks())
}

static EARLY: Kit = Kit {
    rank: KitRank::Style,
    tokens: &[TokenSet::of::<DurationToken>().at(Place::Motion)],
    sections: &[
        Section {
            name: "one",
            css: first,
        },
        Section {
            name: "tokens",
            css: tokens,
        },
    ],
    sheets: &[],
    vocabulary: Vocabulary::NONE,
};

static LATE: Kit = Kit {
    rank: KitRank::Shell,
    tokens: &[TokenSet::of::<SpacingToken>().at(Place::Scale)],
    sections: &[Section {
        name: "two",
        css: second,
    }],
    sheets: &[],
    vocabulary: Vocabulary::NONE,
};

/// The text positions of `needles` in `haystack`, in the order asked.
fn positions(haystack: &str, needles: &[&str]) -> Vec<usize> {
    needles
        .iter()
        .map(|needle| {
            haystack
                .find(needle)
                .unwrap_or_else(|| panic!("{needle} is missing from {haystack}"))
        })
        .collect()
}

#[test]
fn the_cascade_follows_rank_not_argument_order() {
    for kits in [Kits::of(&[&EARLY, &LATE]), Kits::of(&[&LATE, &EARLY])] {
        let sheet = kits.stylesheet();
        let at = positions(&sheet, &["== one ==", "== tokens ==", "== two =="]);
        assert!(at.windows(2).all(|pair| pair[0] < pair[1]), "{sheet}");
    }
}

#[test]
fn a_family_sits_at_its_place_whichever_kit_declares_it() {
    let sheet = Kits::of(&[&LATE, &EARLY]).stylesheet();
    let at = positions(&sheet, &["--s-1:1px;", "--t-quick:150ms;"]);
    assert!(at[0] < at[1], "{sheet}");
}

fn person(_: &Kits) -> Cow<'static, str> {
    Cow::Borrowed(".person{}")
}

static PERSON: Kit = Kit {
    rank: KitRank::User,
    tokens: &[],
    sections: &[Section {
        name: "person",
        css: person,
    }],
    sheets: &[],
    vocabulary: Vocabulary::NONE,
};

#[test]
fn a_user_kit_is_last_whatever_the_argument_order() {
    for kits in [
        Kits::of(&[&PERSON, &EARLY, &LATE]),
        Kits::of(&[&LATE, &PERSON, &EARLY]),
    ] {
        let sheet = kits.stylesheet();
        let at = positions(&sheet, &["== one ==", "== two ==", "== person =="]);
        assert!(at.windows(2).all(|pair| pair[0] < pair[1]), "{sheet}");
    }
}

#[test]
fn the_user_rank_sorts_after_every_other_rank() {
    use ds_core::word::Word;
    let others = KitRank::ALL.iter().filter(|rank| **rank != KitRank::User);
    for rank in others {
        assert!(*rank < KitRank::User, "{}", rank.slug());
    }
}
