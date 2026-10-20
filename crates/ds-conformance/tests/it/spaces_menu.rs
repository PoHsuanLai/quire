//! The Space's menu through the kit on a real Blitz document: the rows in order, Edit Space's
//! sheet keeping what Save keeps and putting back what Cancel drops, New Space opening its name
//! at once, Delete asking first, and Delete missing for the last Space.

use crate::support::spaces_page::{One, Three, WithAccounts};
use dioxus::prelude::Element;
use ds::base::press::PointerButton;
use ds::prelude::*;
use ds_harness::{Clock, FocusState, Harness, HarnessConfig, Input, Viewport};
use std::time::Duration;

const VIEW: Viewport = Viewport {
    width: 420,
    height: 420,
    scale_percent: 100,
};

fn ms(n: u64) -> Duration {
    Duration::from_millis(n)
}

/// Tall enough for the whole Edit Space sheet (name, colour field, stops, grain, presets and its
/// buttons) to stand on screen without scrolling.
const TALL: Viewport = Viewport {
    width: 480,
    height: 900,
    scale_percent: 100,
};

fn harness_in(page: fn() -> Element, view: Viewport) -> Harness {
    let mut harness = Harness::new(page, HarnessConfig::new(view).with_clock(Clock::Virtual));
    harness.advance(ms(50));
    harness
}

fn harness(page: fn() -> Element) -> Harness {
    harness_in(page, VIEW)
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
            "Edit Space\u{2026}",
            "Appearance",
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
fn edit_opens_a_sheet_whose_save_keeps_the_name() {
    let mut harness = harness_in(Three, TALL);
    open_menu(&mut harness);
    pick(&mut harness, 1);
    assert_eq!(harness.count(".ds-space-edit"), 1, "{}", harness.html());
    for letter in "Work".chars() {
        harness.send(Input::key(ShortcutKey::Char(letter)));
    }
    harness.advance(ms(100));
    assert_eq!(
        harness
            .attr(".ds-space-head .ds-label", "aria-label")
            .as_deref(),
        Some("The Work Space"),
        "the name changes as it is typed"
    );
    harness.send(Input::key(ShortcutKey::Enter));
    harness.advance(ms(400));
    assert_eq!(
        harness.count(".ds-space-edit"),
        0,
        "Return saves and closes"
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
    pick(&mut harness, 4);
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
    pick(&mut harness, 5);
    assert_eq!(harness.count(".ds-alert"), 1);
    assert_eq!(dots(&harness), 3, "asking deletes nothing");
    let confirm = harness
        .centre(".ds-alert-slot:nth-child(1) > .ds-button")
        .expect("the Delete button");
    harness.send(Input::click(confirm));
    harness.advance(ms(400));
    assert_eq!(dots(&harness), 2);
    assert_eq!(harness.count(".ds-alert"), 0);
}

#[test]
fn the_app_submenu_keeps_the_menu_open_and_a_kit_row_still_closes_it() {
    let mut harness = harness(WithAccounts);
    open_menu(&mut harness);
    // Rows: Edit Space, Appearance, Accounts, rule, New Space, Delete.
    let accounts = harness
        .centre(".ds-menu-item:nth-child(3) .ds-menu-label")
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
    // The same item again, past the double-click interval: add an account, then take it out.
    harness.advance(ms(600));
    // Where the item is now: the submenu settles to its row once the first pick re-renders it.
    let ada_again = harness
        .centre(".ds-menu[data-depth=\"1\"] .ds-menu-item:nth-child(1) .ds-menu-label")
        .expect("the item is still there");
    harness.send(Input::click(ada_again));
    harness.advance(ms(100));
    assert_eq!(
        harness.text_of(".extra-log").as_deref(),
        Some("AdaAda"),
        "the same keep-open item can be picked twice in a row"
    );
    let bo = harness
        .centre(".ds-menu[data-depth=\"1\"] .ds-menu-item:nth-child(2) .ds-menu-label")
        .expect("the second account");
    harness.send(Input::click(bo));
    harness.advance(ms(100));
    assert_eq!(
        harness.text_of(".extra-log").as_deref(),
        Some("AdaAdaBo"),
        "and another can be picked"
    );
    // A kit row closes it: New Space is row 5 here.
    let new = harness
        .centre(".ds-menu-item:nth-child(5) .ds-menu-label")
        .expect("New Space");
    harness.send(Input::click(new));
    harness.advance(ms(500));
    assert_eq!(harness.count(".ds-menu"), 0, "a kit row closes the menu");
    assert_eq!(harness.count(".ds-spaces-dot-hold"), 3, "and did its work");
}

/// The first frame the submenu is painted, its rect; then its rect once everything has settled.
fn first_painted_and_settled(harness: &mut Harness) -> (Rect, Rect) {
    const SUB: &str = ".ds-menu[data-depth=\"1\"]";
    let accounts = harness
        .centre(".ds-menu-item:nth-child(3) .ds-menu-label")
        .expect("the Accounts row");
    harness.send(Input::click(accounts));
    let mut first = None;
    for _ in 0..60 {
        harness.advance(ms(16));
        let hidden = harness
            .attr(SUB, "style")
            .is_none_or(|style| style.contains("visibility:hidden"));
        if first.is_none() && !hidden {
            first = harness.rect(SUB);
        }
    }
    harness.advance(ms(400));
    // A kept-open pick re-renders the page; the submenu must not move for it.
    let ada = harness
        .centre(&format!("{SUB} .ds-menu-item:nth-child(1) .ds-menu-label"))
        .expect("the first account");
    harness.send(Input::click(ada));
    harness.advance(ms(400));
    let settled = harness.rect(SUB).expect("the submenu");
    (first.expect("the submenu was painted"), settled)
}

#[test]
fn the_submenu_is_painted_where_it_settles() {
    let mut harness = harness(WithAccounts);
    open_menu(&mut harness);
    let (first, settled) = first_painted_and_settled(&mut harness);
    assert_eq!(first, settled, "the first painted frame is the settled one");
}

const OPENER: &str = ".app";

fn key(harness: &mut Harness, key: ShortcutKey) {
    harness.send(Input::key(key));
    harness.advance(ms(400));
}

fn focused(harness: &Harness, selector: &str) -> bool {
    harness.focus_of(selector) == FocusState::Focused
}

/// Open the menu, check it took the keyboard, and pick `row`.
fn open_part(harness: &mut Harness, row: usize) {
    open_menu(harness);
    assert!(focused(harness, ".ds-menu"), "the menu took the keyboard");
    pick(harness, row);
    harness.advance(ms(300));
}

fn head_label(harness: &Harness) -> Option<String> {
    harness.attr(".ds-space-head .ds-label", "aria-label")
}

#[test]
fn rename_takes_the_keyboard_and_escape_restores_the_name() {
    let mut harness = harness(Three);
    open_part(&mut harness, 4);
    assert!(
        focused(&harness, ".ds-space-rename input"),
        "{}",
        harness.html()
    );
    let opened_on = head_label(&harness);
    for letter in "Work".chars() {
        harness.send(Input::key(ShortcutKey::Char(letter)));
    }
    harness.advance(ms(100));
    key(&mut harness, ShortcutKey::Escape);
    assert_eq!(harness.count(".ds-space-rename"), 0, "Escape closes it");
    assert_ne!(opened_on.as_deref(), Some("The Work Space"));
    assert_eq!(
        head_label(&harness),
        opened_on,
        "the old name is back:\n{}",
        harness.html()
    );
    assert!(
        focused(&harness, OPENER),
        "the keyboard went back to the opener"
    );
}

#[test]
fn rename_return_commits_and_gives_the_keyboard_back() {
    let mut harness = harness(Three);
    open_part(&mut harness, 4);
    for letter in "Work".chars() {
        harness.send(Input::key(ShortcutKey::Char(letter)));
    }
    key(&mut harness, ShortcutKey::Enter);
    assert_eq!(harness.count(".ds-space-rename"), 0);
    assert_eq!(
        harness
            .attr(".ds-space-head .ds-label", "aria-label")
            .as_deref(),
        Some("The Work Space")
    );
    assert!(
        focused(&harness, OPENER),
        "the keyboard went back to the opener"
    );
}

#[test]
fn delete_focuses_cancel_and_return_cancels() {
    let mut harness = harness(Three);
    open_part(&mut harness, 5);
    assert!(
        focused(&harness, ".ds-alert-slot:nth-child(2) > .ds-button"),
        "Cancel is the safe default:\n{}",
        harness.html()
    );
    key(&mut harness, ShortcutKey::Enter);
    assert_eq!(
        harness.count(".ds-alert"),
        0,
        "Return pressed Cancel"
    );
    assert_eq!(dots(&harness), 3, "nothing was deleted");
    assert!(
        focused(&harness, OPENER),
        "the keyboard went back to the opener"
    );
}

#[test]
fn delete_escape_cancels_and_gives_the_keyboard_back() {
    let mut harness = harness(Three);
    open_part(&mut harness, 5);
    key(&mut harness, ShortcutKey::Escape);
    assert_eq!(harness.count(".ds-alert"), 0, "Escape closes it");
    assert_eq!(dots(&harness), 3, "nothing was deleted");
    assert!(
        focused(&harness, OPENER),
        "the keyboard went back to the opener"
    );
}

/// The frame's look: the root's style, which the Space's colour writes.
fn frame_style(harness: &Harness) -> String {
    harness.attr(".ds", "style").unwrap_or_default()
}

/// A preset in the open Edit Space sheet that is not the Space's look: the fourth.
fn click_a_preset(harness: &mut Harness) {
    let preset = harness
        .centre(".ds-space-edit .ds-preset:nth-child(4)")
        .unwrap_or_else(|| panic!("no preset:\n{}", harness.html()));
    harness.send(Input::click(preset));
    harness.advance(ms(300));
}

#[test]
fn edit_takes_the_keyboard_and_escape_puts_the_name_back() {
    let mut harness = harness_in(Three, TALL);
    let name = head_label(&harness);
    open_part(&mut harness, 1);
    assert!(
        focused(&harness, ".ds-space-edit input"),
        "the name has the keyboard:\n{}",
        harness.html()
    );
    for letter in "Work".chars() {
        harness.send(Input::key(ShortcutKey::Char(letter)));
    }
    harness.advance(ms(100));
    assert_eq!(head_label(&harness).as_deref(), Some("The Work Space"));
    key(&mut harness, ShortcutKey::Escape);
    assert_eq!(harness.count(".ds-space-edit"), 0, "Escape closes it");
    assert_eq!(head_label(&harness), name, "the name it opened on is back");
}

#[test]
fn edit_cancel_puts_the_colour_back() {
    let mut harness = harness_in(Three, TALL);
    let frame = frame_style(&harness);
    open_part(&mut harness, 1);
    click_a_preset(&mut harness);
    assert_ne!(
        frame_style(&harness),
        frame,
        "the frame repaints as the colour changes"
    );
    let cancel = harness
        .centre(".ds-space-edit-acts .ds-button:nth-child(1)")
        .expect("the Cancel button");
    harness.send(Input::click(cancel));
    harness.advance(ms(400));
    assert_eq!(harness.count(".ds-space-edit"), 0, "Cancel closes it");
    assert_eq!(
        frame_style(&harness),
        frame,
        "the look it opened on is back"
    );
}

#[test]
fn edit_save_keeps_the_name_and_colour() {
    let mut harness = harness_in(Three, TALL);
    let frame = frame_style(&harness);
    open_part(&mut harness, 1);
    for letter in "Work".chars() {
        harness.send(Input::key(ShortcutKey::Char(letter)));
    }
    click_a_preset(&mut harness);
    let painted = frame_style(&harness);
    let save = harness
        .centre(".ds-space-edit-acts .ds-button:nth-child(2)")
        .expect("the Save button");
    harness.send(Input::click(save));
    harness.advance(ms(400));
    assert_eq!(harness.count(".ds-space-edit"), 0, "Save closes it");
    assert_eq!(head_label(&harness).as_deref(), Some("The Work Space"));
    assert_ne!(painted, frame);
    assert_eq!(frame_style(&harness), painted, "the new look is kept");
    assert_eq!(
        harness.text_of(".writes").as_deref(),
        Some("1"),
        "written once, when it closed"
    );
}
