//! The Space's menu through the kit on a real Blitz document: the rows in order, Rename keeping
//! what is typed, New Space opening its name at once, Delete asking first, and Delete missing
//! for the last Space.

use crate::support::spaces_page::{One, Three, WithAccounts};
use dioxus::prelude::Element;
use ds::base::press::PointerButton;
use ds::prelude::*;
use ds_harness::{Clock, Driver, Harness, HarnessConfig, Input, Query, Viewport};
use std::time::Duration;

const VIEW: Viewport = Viewport {
    width: 420,
    height: 420,
    scale_percent: 100,
};

fn ms(n: u64) -> Duration {
    Duration::from_millis(n)
}

fn harness(page: fn() -> Element) -> Harness {
    let mut harness = Harness::new(page, HarnessConfig::new(VIEW).with_clock(Clock::Virtual));
    harness.advance(ms(50));
    harness
}

/// Right click on the head: the Space's menu opens at the pointer.
fn open_menu(harness: &mut Harness) {
    let head = harness.centre(".ds-space-head").expect("the head");
    harness.send(Input::press(head, PointerButton::Secondary));
    harness.advance(ms(100));
}

fn pick(harness: &mut Harness, row: usize) {
    let at = harness
        .centre(&format!(".ds-menu-item:nth-child({row}) .ds-menu-label"))
        .unwrap_or_else(|| panic!("no row {row}:\n{}", harness.html()));
    harness.send(Input::click(at));
    harness.advance(ms(500));
}

fn labels(harness: &Harness) -> Vec<String> {
    (1..=9)
        .filter_map(|n| harness.text_of(&format!(".ds-menu-item:nth-child({n}) .ds-menu-label")))
        .collect()
}

fn dots(harness: &Harness) -> usize {
    harness.count(".ds-spaces-dot-hold")
}

#[test]
fn the_menu_lists_its_rows_in_order() {
    let mut three = harness(Three);
    open_menu(&mut three);
    assert_eq!(
        labels(&three),
        [
            "Rename\u{2026}",
            "Colour\u{2026}",
            "Appearance",
            "Accent Inside the Card",
            "New Space",
            "Delete Space\u{2026}"
        ]
    );
}

#[test]
fn the_last_space_cannot_be_deleted() {
    let mut one = harness(One);
    open_menu(&mut one);
    let rows = labels(&one);
    assert!(rows.contains(&"New Space".to_owned()), "{rows:?}");
    assert!(
        !rows.iter().any(|row| row.starts_with("Delete")),
        "{rows:?}"
    );
}

#[test]
fn rename_opens_a_field_that_keeps_the_name_when_it_closes() {
    let mut harness = harness(Three);
    open_menu(&mut harness);
    pick(&mut harness, 1);
    assert_eq!(harness.count(".ds-space-rename"), 1);
    for letter in "Work".chars() {
        harness.send(Input::key(ShortcutKey::Char(letter)));
    }
    harness.advance(ms(100));
    harness.send(Input::key(ShortcutKey::Enter));
    harness.advance(ms(400));
    assert_eq!(
        harness.count(".ds-space-rename"),
        0,
        "Return closes the field"
    );
    assert_eq!(
        harness
            .attr(".ds-space-head .ds-label", "aria-label")
            .as_deref(),
        Some("The Work Space"),
        "{}",
        harness.html()
    );
    assert_eq!(
        harness.text_of(".writes").as_deref(),
        Some("1"),
        "written once, when it closed"
    );
}

#[test]
fn new_space_makes_one_and_opens_its_name() {
    let mut harness = harness(Three);
    open_menu(&mut harness);
    pick(&mut harness, 6);
    assert_eq!(dots(&harness), 4);
    assert_eq!(
        harness.text_of(".current").as_deref(),
        Some("3"),
        "the new Space is on screen"
    );
    assert_eq!(harness.count(".ds-space-rename"), 1, "its name is open");
}

#[test]
fn delete_asks_and_only_its_button_deletes() {
    let mut harness = harness(Three);
    open_menu(&mut harness);
    pick(&mut harness, 7);
    assert_eq!(harness.count(".ds-space-delete"), 1);
    assert_eq!(dots(&harness), 3, "asking deletes nothing");
    let confirm = harness
        .centre(".ds-space-part-acts .ds-button:nth-child(2)")
        .expect("the Delete Space button");
    harness.send(Input::click(confirm));
    harness.advance(ms(400));
    assert_eq!(dots(&harness), 2);
    assert_eq!(harness.count(".ds-space-delete"), 0);
}

#[test]
fn the_app_submenu_keeps_the_menu_open_and_a_kit_row_still_closes_it() {
    let mut harness = harness(WithAccounts);
    open_menu(&mut harness);
    // Rows: Rename, Colour, Appearance, Accent, Accounts, rule, New Space, Delete.
    let accounts = harness
        .centre(".ds-menu-item:nth-child(5) .ds-menu-label")
        .expect("the Accounts row");
    harness.send(Input::click(accounts));
    harness.advance(ms(400));
    let ada = harness
        .centre(".ds-menu[data-depth=\"1\"] .ds-menu-item:nth-child(1) .ds-menu-label")
        .unwrap_or_else(|| panic!("no submenu:\n{}", harness.html()));
    harness.send(Input::click(ada));
    harness.advance(ms(100));
    assert_eq!(
        harness.text_of(".extra-log").as_deref(),
        Some("Ada"),
        "on_extra heard the item"
    );
    assert!(harness.count(".ds-menu") > 0, "the menu stays open");
    let bo = harness
        .centre(".ds-menu[data-depth=\"1\"] .ds-menu-item:nth-child(2) .ds-menu-label")
        .expect("the second account");
    harness.send(Input::click(bo));
    harness.advance(ms(100));
    assert_eq!(
        harness.text_of(".extra-log").as_deref(),
        Some("AdaBo"),
        "and another can be picked"
    );
    // A kit row closes it: New Space is row 7 here.
    let new = harness
        .centre(".ds-menu-item:nth-child(7) .ds-menu-label")
        .expect("New Space");
    harness.send(Input::click(new));
    harness.advance(ms(500));
    assert_eq!(harness.count(".ds-menu"), 0, "a kit row closes the menu");
    assert_eq!(harness.count(".ds-spaces-dot-hold"), 3, "and did its work");
}
