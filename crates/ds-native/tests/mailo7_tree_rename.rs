//! mailo gaps 7, item 3: a folder renamed in place. `TreeItem { editing }` draws the caller's
//! field where the label is, at the label's metrics, so nothing on the row moves; a press or
//! typing in it neither toggles nor selects the row; and Enter and Escape reach the field's own
//! handler.

use dioxus::prelude::*;
use ds::{
    Appearance, Disclosure, Ds, FieldFace, Focus, Icon, Key, Material, PlaceId, TextInput,
    TreeItem, TreeShape, use_focus_request,
};
use ds_native::harness::settle_until;
use ds_native::{Harness, Viewport};
use std::time::Duration;

const VIEW: Viewport = Viewport {
    width: 360,
    height: 240,
    scale_percent: 100,
};

fn ms(n: u64) -> Duration {
    Duration::from_millis(n)
}

/// Whether the page starts with the Projects row being renamed.
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
enum Start {
    /// The label shows.
    Reading,
    /// The rename field shows in the label's place.
    Renaming,
}

#[component]
fn Page(start: Start) -> Element {
    let mut log = use_signal(Vec::<String>::new);
    let mut note = move |entry: String| log.with_mut(|log| log.push(entry));
    let mut open = use_signal(|| Disclosure::Open);
    let mut renaming = use_signal(|| start == Start::Renaming);
    let mut name = use_signal(|| "Projects".to_string());
    let request = use_focus_request().with_select_all();
    let editing = renaming().then(|| {
        rsx! {
            TextInput { variant: FieldFace::Bare, label: "Rename folder", value: name(),
                focus: Focus::Controlled(request),
                oninput: move |next: String| name.set(next),
                onkey: move |event: KeyboardEvent| match event.key() {
                    dioxus::prelude::Key::Enter => {
                        note(format!("enter:{}", name.peek()));
                        renaming.set(false);
                    }
                    dioxus::prelude::Key::Escape => {
                        note("escape".to_string());
                        renaming.set(false);
                    }
                    _ => {}
                },
            }
        }
    });
    rsx! {
        Ds { appearance: Appearance::default(), material: Material::Window,
            div { class: "tree", style: "width:240px; padding:20px",
                TreeItem {
                    label: name(),
                    open: open(),
                    on_toggle: move |to: Disclosure| {
                        note(format!("toggle:{to:?}"));
                        open.set(to);
                    },
                    glyph: Icon::Folder,
                    count: 3,
                    place: PlaceId("projects".to_string()),
                    onselect: move |_| note("select".to_string()),
                    editing,
                    TreeItem { label: "Quire", open: Disclosure::Closed, on_toggle: |_| {}, shape: TreeShape::Leaf, place: PlaceId("quire".to_string()) }
                }
            }
            p { class: "log", {log().join(",")} }
        }
    }
}

fn reading() -> Element {
    rsx! { Page { start: Start::Reading } }
}

fn renaming() -> Element {
    rsx! { Page { start: Start::Renaming } }
}

fn harness(app: fn() -> Element) -> Harness {
    let mut harness = Harness::new(app, VIEW);
    harness.advance(ms(50));
    harness
}

const FIELD: &str = ".ds-tree-item-edit input";
const ROW: &str = "summary[*|data-place=projects]";

fn log(harness: &Harness) -> String {
    harness.text_of(".log").unwrap_or_default()
}

#[test]
fn the_field_takes_the_labels_place_and_moves_nothing() {
    let read = harness(reading);
    let edit = harness(renaming);
    let (label, slot) = (
        read.rect(".ds-tree-item-label").expect("the label"),
        edit.rect(".ds-tree-item-edit").expect("the editing slot"),
    );
    let (row, edited_row) = (
        read.rect(ROW).expect("the row"),
        edit.rect(ROW).expect("the row"),
    );
    assert_eq!(
        slot.origin.x, label.origin.x,
        "the field starts where the label does"
    );
    assert_eq!(
        slot.size.width, label.size.width,
        "and takes the label's free space"
    );
    assert_eq!(
        edited_row.size.height, row.size.height,
        "the row keeps its height"
    );
    assert_eq!(
        edit.rect(".ds-count").map(|count| count.origin),
        read.rect(".ds-count").map(|count| count.origin),
        "the count stays put"
    );
    assert_eq!(
        edit.rect(".ds-tree-item-row[*|data-place=quire]")
            .map(|next| next.origin.y),
        read.rect(".ds-tree-item-row[*|data-place=quire]")
            .map(|next| next.origin.y),
        "the next row stays put"
    );
}

#[test]
fn the_field_has_the_keyboard_with_its_text_selected() {
    let mut edit = harness(renaming);
    settle_until(&mut edit, |harness| harness.is_focused(FIELD));
    settle_until(&mut edit, |harness| {
        harness.selected_text(FIELD).as_deref() == Some("Projects")
    });
}

#[test]
fn a_press_and_typing_in_the_field_neither_toggle_nor_select_the_row() {
    let mut edit = harness(renaming);
    settle_until(&mut edit, |harness| harness.is_focused(FIELD));
    let at = edit.centre(FIELD).expect("the field is laid out");
    edit.click(at);
    edit.advance(ms(50));
    for c in "Work".chars() {
        edit.key(Key::Char(c));
        edit.advance(ms(20));
    }
    edit.key(Key::Char(' '));
    edit.advance(ms(20));
    assert!(edit.is_focused(FIELD), "the field kept the keyboard");
    assert_eq!(log(&edit), "", "no toggle, no select");
    assert_eq!(
        edit.attr(ROW, "aria-expanded").as_deref(),
        Some("true"),
        "the folder is still open"
    );
    // The press put the caret after the text (the selection collapsed), and the keys typed there.
    assert_eq!(edit.attr(FIELD, "value").as_deref(), Some("ProjectsWork "));
}

#[test]
fn enter_reaches_the_fields_handler_and_ends_the_rename() {
    let mut edit = harness(renaming);
    settle_until(&mut edit, |harness| harness.is_focused(FIELD));
    for c in "Work".chars() {
        edit.key(Key::Char(c));
        edit.advance(ms(20));
    }
    edit.key(Key::Enter);
    settle_until(&mut edit, |harness| harness.count(FIELD) == 0);
    assert_eq!(log(&edit), "enter:Work");
    assert_eq!(edit.text_of(".ds-tree-item-label").as_deref(), Some("Work"));
    assert_eq!(edit.attr(ROW, "aria-expanded").as_deref(), Some("true"));
}

#[test]
fn escape_reaches_the_fields_handler_and_ends_the_rename() {
    let mut edit = harness(renaming);
    settle_until(&mut edit, |harness| harness.is_focused(FIELD));
    edit.key(Key::Escape);
    settle_until(&mut edit, |harness| harness.count(FIELD) == 0);
    assert_eq!(log(&edit), "escape");
    assert_eq!(edit.attr(ROW, "aria-expanded").as_deref(), Some("true"));
}

#[test]
fn a_press_on_the_label_still_selects_once_the_rename_is_over() {
    let mut read = harness(reading);
    let at = read.centre(".ds-tree-item-label").expect("the label");
    read.click(at);
    read.advance(ms(50));
    assert_eq!(log(&read), "select");
}
