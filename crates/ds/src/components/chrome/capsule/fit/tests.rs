use super::*;
use crate::components::chrome::capsule::model::{LevelSlot, ScrubSlot};
use ds_core::vocab::{Availability, Fraction};
use ds_motion::spring::Millis;
use ds_style::icon::Icon;

type Slot = RankedSlot<String>;

fn button(rank: Option<u8>, name: &str) -> Slot {
    let slot = CapsuleSlot::button(name.to_owned(), name, Icon::Play);
    match rank {
        Some(rank) => slot.droppable(rank),
        None => slot.essential(),
    }
}

fn divider() -> Slot {
    CapsuleSlot::Divider.essential()
}

fn readout(rank: u8, text: &str) -> Slot {
    CapsuleSlot::Readout(text.to_owned()).droppable(rank)
}

fn bar() -> Slot {
    CapsuleSlot::Scrub(ScrubSlot {
        label: "Position".to_owned(),
        position: Fraction(0),
        length: Millis(1000),
        buffered: Vec::new(),
        availability: Availability::Enabled,
    })
    .essential()
}

fn level(rank: u8) -> Slot {
    CapsuleSlot::Level(LevelSlot {
        label: "Volume".to_owned(),
        value: Fraction(500),
        availability: Availability::Enabled,
    })
    .droppable(rank)
}

/// A player's capsule as a media family ranks it: the stays are the transport and the bar.
fn player() -> Vec<Slot> {
    vec![
        button(None, "back"),
        button(None, "play"),
        button(None, "forward"),
        divider(),
        readout(1, "0:25"),
        bar(),
        readout(3, "1:40"),
        divider(),
        level(2),
        divider(),
        button(Some(4), "slower"),
        readout(4, "1x"),
        button(Some(4), "faster"),
        divider(),
        button(Some(5), "export"),
    ]
}

fn name(slot: &CapsuleSlot<String>) -> String {
    match slot {
        CapsuleSlot::Item(item) => item.label.clone(),
        CapsuleSlot::Readout(text) => format!("[{text}]"),
        CapsuleSlot::Divider => "|".to_owned(),
        CapsuleSlot::Scrub(_) => "bar".to_owned(),
        CapsuleSlot::Level(_) => "level".to_owned(),
    }
}

/// The names of the slots of `slots` that show on `stage`.
fn names(slots: &[Slot], stage: Option<u32>) -> Vec<String> {
    slots
        .iter()
        .zip(fit(slots, stage))
        .filter(|(_, state)| *state == SlotShown::Shown)
        .map(|(one, _)| name(&one.slot))
        .collect()
}

#[test]
fn a_narrower_stage_drops_the_least_important_first() {
    const BASE: &[&str] = &["back", "play", "forward", "|", "[0:25]", "bar"];
    const LEVEL: &[&str] = &["|", "level"];
    const SPEED: &[&str] = &["|", "slower", "[1x]", "faster"];
    const EXPORT: &[&str] = &["|", "export"];
    let join = |parts: &[&[&str]]| -> Vec<String> {
        parts
            .iter()
            .flat_map(|part| part.iter())
            .map(|one| (*one).to_owned())
            .collect()
    };
    let length: &[&str] = &["[1:40]"];
    let cases: Vec<(&str, u32, Vec<String>)> = vec![
        (
            "everything",
            900,
            join(&[BASE, length, LEVEL, SPEED, EXPORT]),
        ),
        (
            "the least stage for everything",
            672,
            join(&[BASE, length, LEVEL, SPEED, EXPORT]),
        ),
        (
            "the export goes first",
            660,
            join(&[BASE, length, LEVEL, SPEED]),
        ),
        (
            "the speed goes whole, not piecemeal",
            600,
            join(&[BASE, length, LEVEL]),
        ),
        (
            "the length goes before the level",
            460,
            join(&[BASE, LEVEL]),
        ),
        ("then the level", 440, join(&[BASE])),
        (
            "then the clock",
            330,
            join(&[&["back", "play", "forward", "|", "bar"]]),
        ),
        (
            "the essentials only",
            300,
            join(&[&["back", "play", "forward", "|", "bar"]]),
        ),
    ];
    for (why, stage, want) in cases {
        assert_eq!(names(&player(), Some(stage)), want, "{why} at {stage}");
    }
}

#[test]
fn every_slot_shows_when_they_all_fit() {
    let slots = player();
    assert!(
        fit(&slots, Some(2000))
            .iter()
            .all(|state| *state == SlotShown::Shown)
    );
}

#[test]
fn only_the_essentials_stay_however_narrow() {
    for stage in [0, 100, 200] {
        assert_eq!(
            names(&player(), Some(stage)),
            ["back", "play", "forward", "|", "bar"],
            "at {stage}"
        );
    }
}

#[test]
fn only_the_bar_fits_when_it_is_all_that_is_essential() {
    let slots = vec![button(Some(1), "a"), bar(), button(Some(2), "b"), level(3)];
    // The bar alone: padding 16 + its least; the stage adds the margin each side.
    let alone = (S8 * 2 + SCRUB_LEAST) + S16 * 2;
    assert_eq!(names(&slots, Some(alone)), ["bar"]);
    assert_eq!(names(&slots, Some(alone - 1)), ["bar"]);
    assert_eq!(names(&slots, Some(alone + 40)), ["a", "bar"]);
}

#[test]
fn equal_priorities_drop_together() {
    let slots = vec![
        button(Some(1), "a"),
        button(Some(1), "b"),
        button(Some(1), "c"),
        bar(),
        button(Some(2), "d"),
    ];
    let wide = 2 * S16 + 2 * S8 + SCRUB_LEAST;
    let step = 40 + S4;
    assert_eq!(
        names(&slots, Some(wide + 4 * step)),
        ["a", "b", "c", "bar", "d"]
    );
    // One button short of all: the lone rank-2 goes, the group of three stays whole.
    assert_eq!(names(&slots, Some(wide + 3 * step)), ["a", "b", "c", "bar"]);
    // Two short of all: the whole group goes at once, never one or two of three.
    assert_eq!(names(&slots, Some(wide + 2 * step)), ["bar"]);
    assert_eq!(names(&slots, Some(wide)), ["bar"]);
}

#[test]
fn a_stage_not_measured_yet_shows_everything() {
    assert_eq!(names(&player(), None).len(), player().len());
}

#[test]
fn a_wider_stage_only_ever_brings_slots_back() {
    let slots = player();
    let mut before = fit(&slots, Some(0));
    for stage in 1..900 {
        let now = fit(&slots, Some(stage));
        assert_eq!(now.len(), slots.len(), "one entry per slot at {stage}");
        let lost = before
            .iter()
            .zip(&now)
            .any(|(was, is)| *was == SlotShown::Shown && *is == SlotShown::Dropped);
        assert!(!lost, "a slot left as the stage widened to {stage}");
        before = now;
    }
}

#[test]
fn no_divider_is_left_at_an_end_or_beside_another() {
    for stage in (0..1000).step_by(10) {
        let slots = player();
        let shown: Vec<CapsuleSlot<String>> = slots
            .iter()
            .zip(fit(&slots, Some(stage)))
            .filter(|(_, state)| *state == SlotShown::Shown)
            .map(|(one, _)| one.slot.clone())
            .collect();
        let divided = |at: usize| matches!(shown.get(at), Some(CapsuleSlot::Divider));
        assert!(!divided(0), "first at {stage}");
        assert!(!divided(shown.len() - 1), "last at {stage}");
        assert!(
            (1..shown.len()).all(|at| !(divided(at) && divided(at - 1))),
            "doubled at {stage}"
        );
    }
}

#[test]
fn what_is_kept_fits_the_room() {
    let slots = player();
    for stage in (350..1000).step_by(10) {
        let shown = fit(&slots, Some(stage));
        assert!(fits(&slots, &shown, stage), "at {stage}");
    }
}
