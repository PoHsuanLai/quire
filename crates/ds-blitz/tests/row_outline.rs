//! A folder tree of `Row`s on a real Blitz document. A branch row's triangle asks the app for
//! the other state through `on_toggle` and its body opens and closes by `use_collapse`; a press on
//! the row selects without toggling; a press on the trailing ⋯ is the button's alone; the drop
//! states are written and painted by the row's own rules; and a rename field drawn as the row's
//! `content` takes the title's place and moves nothing.

use dioxus::prelude::*;
use ds::{
    Accessory, Appearance, Button, Common, DataAttr, DataName, DropState, Ds, FieldBezel,
    FieldFocus, Icon, Material, Outline, Point, Propagation, Px, Row, RowLeading, RowState,
    ShortcutKey, Shown, TextField, use_focus_request,
};
use ds::{Bezel, ImagePosition};
use ds_harness::harness::settle_until;
use ds_harness::{Clock, Harness, HarnessConfig, Viewport};
use std::time::Duration;

const VIEW: Viewport = Viewport {
    width: 360,
    height: 320,
    scale_percent: 100,
};

fn ms(n: u64) -> Duration {
    Duration::from_millis(n)
}

/// `data-place="<place>"`.
fn place(path: &str) -> Common {
    Common {
        data: DataName::parse("place")
            .map(|name| vec![DataAttr::new(name, path)])
            .unwrap_or_default(),
        ..Common::default()
    }
}

/// Two branches, each with a leaf; Projects accepts the drag, Archive is its target.
#[allow(non_snake_case)]
fn Page() -> Element {
    let mut log = use_signal(Vec::<String>::new);
    let mut note = move |entry: String| log.with_mut(|log| log.push(entry));
    let mut projects = use_signal(|| Shown::Visible);
    let mut archive = use_signal(|| Shown::Hidden);
    let more = move |path: &'static str| {
        Accessory::Slot(rsx! {
            Button {
                bezel: Bezel::Toolbar, image: ImagePosition::Only,
                icon: Icon::Ellipsis,
                label: "Actions for {path}",
                propagation: Propagation::Stop,
                onclick: move |_| note(format!("more:{path}")),
            }
        })
    };
    rsx! {
        Ds { appearance: Appearance::default(), material: Material::Window,
            div { class: "tree", style: "width:240px; padding:20px",
                Row {
                    state: RowState { drop: DropState::Accepts, ..RowState::default() },
                    title: "Projects",
                    leading: RowLeading::Icon(Icon::Folder),
                    outline: Outline::Branch(projects()),
                    on_toggle: move |to: Shown| {
                        note(format!("toggle:projects:{to:?}"));
                        projects.set(to);
                    },
                    accessory: more("projects"),
                    common: place("projects"),
                    onclick: move |_| note("select:projects".to_string()),
                    Row { title: "Quire", outline: Outline::Leaf, common: place("quire") }
                }
                Row {
                    state: RowState { drop: DropState::Target, ..RowState::default() },
                    title: "Archive",
                    leading: RowLeading::Icon(Icon::Archive),
                    outline: Outline::Branch(archive()),
                    on_toggle: move |to: Shown| {
                        note(format!("toggle:archive:{to:?}"));
                        archive.set(to);
                    },
                    accessory: more("archive"),
                    common: place("archive"),
                    Row { title: "2025", outline: Outline::Leaf, common: place("2025") }
                }
                Row { title: "Receipts", leading: RowLeading::Icon(Icon::Folder), outline: Outline::Leaf, common: place("receipts") }
            }
            p { class: "log", {log().join(",")} }
        }
    }
}

fn settled() -> Harness {
    let mut harness =
        Harness::with_config(Page, HarnessConfig::new(VIEW).with_clock(Clock::Virtual));
    harness.advance(ms(50));
    harness
}

fn centre(harness: &Harness, selector: &str) -> Point {
    harness
        .centre(selector)
        .unwrap_or_else(|| panic!("{selector} is not laid out:\n{}", harness.html()))
}

fn press(harness: &mut Harness, at: Point) {
    harness.click(at);
    harness.advance(ms(400));
}

fn log(harness: &Harness) -> String {
    harness.text_of(".log").unwrap_or_default()
}

const PROJECTS: &str = ".ds-row[*|data-place=projects]";
const ARCHIVE: &str = ".ds-row[*|data-place=archive]";

/// How tall a branch's body is drawn: the first branch's or the second's.
fn body(harness: &Harness, place: &str) -> f32 {
    let nth = if place == PROJECTS { 1 } else { 2 };
    harness
        .rect(&format!(
            ".tree > .ds-row-branch:nth-child({nth}) > .ds-disclosure-body"
        ))
        .map_or(0.0, |rect| rect.size.height.0)
}

#[test]
fn the_triangle_asks_the_app_the_row_selects_and_the_trailing_button_acts_alone() {
    let mut harness = settled();
    assert!(body(&harness, PROJECTS) > 10.0, "Projects starts open");
    assert_eq!(body(&harness, ARCHIVE), 0.0, "Archive starts closed");
    assert_eq!(
        harness.attr(PROJECTS, "aria-expanded").as_deref(),
        Some("true")
    );

    // The ⋯ acts alone: it neither toggles nor selects.
    let more = centre(&harness, &format!("{PROJECTS} .ds-button"));
    press(&mut harness, more);
    assert_eq!(log(&harness), "more:projects");
    assert!(body(&harness, PROJECTS) > 10.0);

    // The row selects and does not toggle.
    let title = centre(&harness, &format!("{PROJECTS} .ds-row-title"));
    press(&mut harness, title);
    assert_eq!(log(&harness), "more:projects,select:projects");
    assert!(body(&harness, PROJECTS) > 10.0);

    // The triangle is the toggle: the app hears the state asked for.
    let triangle = centre(&harness, &format!("{PROJECTS} .ds-row-disclosure"));
    press(&mut harness, triangle);
    let triangle = centre(&harness, &format!("{ARCHIVE} .ds-row-disclosure"));
    press(&mut harness, triangle);
    assert_eq!(
        log(&harness),
        "more:projects,select:projects,toggle:projects:Hidden,toggle:archive:Visible"
    );
    assert_eq!(body(&harness, PROJECTS), 0.0, "Projects closed");
    assert!(body(&harness, ARCHIVE) > 10.0, "Archive opened");
    assert_eq!(
        harness.attr(PROJECTS, "aria-expanded").as_deref(),
        Some("false")
    );
}

#[test]
fn a_branch_opens_over_the_move_duration_from_its_measured_height() {
    let mut harness = settled();
    let open = body(&harness, PROJECTS);
    let triangle = centre(&harness, &format!("{PROJECTS} .ds-row-disclosure"));
    harness.click(triangle);
    harness.advance(ds::settle(ds::Anim::Heal, ds::MotionLevel::Standard) / 3);
    let mid = body(&harness, PROJECTS);
    assert!(
        mid > 0.0 && mid < open,
        "between open and closed a third of the way through: {mid} of {open}"
    );
    settle_until(&mut harness, |h| body(h, PROJECTS) == 0.0);
}

#[test]
fn the_drop_states_are_written_and_painted_by_the_rows_own_rules() {
    let mut harness = settled();
    let drop = |harness: &Harness, place: &str| {
        harness.attr(&format!(".ds-row[*|data-place={place}]"), "data-drop")
    };
    assert_eq!(drop(&harness, "projects").as_deref(), Some("accepts"));
    assert_eq!(drop(&harness, "archive").as_deref(), Some("target"));
    assert_eq!(drop(&harness, "receipts"), None);

    // Accepts draws its hairline inside the row's box: it is as tall as the target's, whose
    // content is the same, and its title starts at the same place.
    let accepts = harness.rect(PROJECTS).expect("projects");
    let idle = harness
        .rect(".ds-row[*|data-place=receipts]")
        .expect("receipts");
    let lit = harness.rect(ARCHIVE).expect("archive");
    assert_eq!(
        accepts.size.height, lit.size.height,
        "the hairline is inside the box"
    );
    let title = |harness: &Harness, place: &str| {
        harness
            .rect(&format!(".ds-row[*|data-place={place}] .ds-row-title"))
            .expect("title")
    };
    let (projects_title, archive_title) = (title(&harness, "projects"), title(&harness, "archive"));
    assert_eq!(
        projects_title.origin.x - accepts.origin.x,
        archive_title.origin.x - lit.origin.x,
        "the title does not move"
    );

    // The target is lit: its fill differs from an idle row's.
    let target = harness.rect(ARCHIVE).expect("archive");
    let image = harness.render().expect("a frame");
    let sample = |rect: ds::Rect| {
        let x = (rect.origin.x + rect.size.width - Px(40.0)).0 as u32;
        let y = (rect.origin.y + Px(rect.size.height.0 / 2.0)).0 as u32;
        *image.get_pixel(x, y)
    };
    assert_ne!(
        sample(target),
        sample(idle),
        "the target is lit with --accent-soft"
    );
}

// ---- A rename field as the row's content ---------------------------------------------------

/// Whether the page starts with the Projects row being renamed.
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
enum Start {
    /// The title shows.
    Reading,
    /// The rename field shows in the title's place.
    Renaming,
}

#[component]
fn Rename(start: Start) -> Element {
    let mut log = use_signal(Vec::<String>::new);
    let mut note = move |entry: String| log.with_mut(|log| log.push(entry));
    let mut renaming = use_signal(|| start == Start::Renaming);
    let mut name = use_signal(|| "Projects".to_string());
    let request = use_focus_request().with_select_all();
    let editing = renaming().then(|| {
        rsx! {
            // A press in the field is the field's: it does not reach the row.
            span {
                class: "fence",
                onclick: move |event: MouseEvent| event.stop_propagation(),
                TextField { bezel: FieldBezel::Plain, label: "Rename folder", value: name(),
                    focus: FieldFocus::Controlled(request),
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
        }
    });
    rsx! {
        Ds { appearance: Appearance::default(), material: Material::Window,
            div { class: "tree", style: "width:240px; padding:20px",
                Row {
                    title: name(),
                    content: editing,
                    leading: RowLeading::Icon(Icon::Folder),
                    outline: Outline::Branch(Shown::Visible),
                    on_toggle: move |to: Shown| note(format!("toggle:{to:?}")),
                    accessory: Accessory::Badge(3),
                    common: place("projects"),
                    onclick: move |_| note("select".to_string()),
                    Row { title: "Quire", outline: Outline::Leaf, common: place("quire") }
                }
            }
            p { class: "log", {log().join(",")} }
        }
    }
}

fn reading() -> Element {
    rsx! { Rename { start: Start::Reading } }
}

fn renaming() -> Element {
    rsx! { Rename { start: Start::Renaming } }
}

fn started(app: fn() -> Element) -> Harness {
    let mut harness = Harness::new(app, VIEW);
    harness.advance(ms(50));
    harness
}

const FIELD: &str = ".ds-row-words input";

#[test]
fn the_field_takes_the_titles_place_and_moves_nothing() {
    let read = started(reading);
    let edit = started(renaming);
    let (title, slot) = (
        read.rect(&format!("{PROJECTS} .ds-row-words"))
            .expect("the words"),
        edit.rect(&format!("{PROJECTS} .ds-row-words"))
            .expect("the words"),
    );
    assert_eq!(
        slot.origin.x, title.origin.x,
        "the field starts where the title does"
    );
    assert_eq!(
        slot.size.width, title.size.width,
        "and takes the free space"
    );
    assert_eq!(
        edit.rect(PROJECTS).map(|row| row.size.height),
        read.rect(PROJECTS).map(|row| row.size.height),
        "the row keeps its height"
    );
    assert_eq!(
        edit.rect(".ds-badge").map(|count| count.origin),
        read.rect(".ds-badge").map(|count| count.origin),
        "the count stays put"
    );
    assert_eq!(
        edit.rect(".ds-row[*|data-place=quire]")
            .map(|next| next.origin.y),
        read.rect(".ds-row[*|data-place=quire]")
            .map(|next| next.origin.y),
        "the next row stays put"
    );
}

#[test]
fn the_field_has_the_keyboard_with_its_text_selected() {
    let mut edit = started(renaming);
    settle_until(&mut edit, |harness| harness.is_focused(FIELD));
    settle_until(&mut edit, |harness| {
        harness.selected_text(FIELD).as_deref() == Some("Projects")
    });
}

#[test]
fn a_press_and_typing_in_the_field_neither_toggle_nor_select_the_row() {
    let mut edit = started(renaming);
    settle_until(&mut edit, |harness| harness.is_focused(FIELD));
    let at = edit.centre(FIELD).expect("the field is laid out");
    edit.click(at);
    edit.advance(ms(50));
    for c in "Work".chars() {
        edit.key(ShortcutKey::Char(c));
        edit.advance(ms(20));
    }
    assert!(edit.is_focused(FIELD), "the field kept the keyboard");
    assert_eq!(log(&edit), "", "no toggle, no select");
    assert_eq!(
        edit.attr(PROJECTS, "aria-expanded").as_deref(),
        Some("true")
    );
}

#[test]
fn enter_and_escape_reach_the_fields_handler_and_end_the_rename() {
    let mut edit = started(renaming);
    settle_until(&mut edit, |harness| harness.is_focused(FIELD));
    for c in "Work".chars() {
        edit.key(ShortcutKey::Char(c));
        edit.advance(ms(20));
    }
    edit.key(ShortcutKey::Enter);
    settle_until(&mut edit, |harness| harness.count(FIELD) == 0);
    assert_eq!(log(&edit), "enter:Work");
    assert_eq!(
        edit.text_of(&format!("{PROJECTS} .ds-row-title"))
            .as_deref(),
        Some("Work")
    );
    let mut again = started(renaming);
    settle_until(&mut again, |harness| harness.is_focused(FIELD));
    again.key(ShortcutKey::Escape);
    settle_until(&mut again, |harness| harness.count(FIELD) == 0);
    assert_eq!(log(&again), "escape");
}

#[test]
fn a_press_on_the_row_selects_once_the_rename_is_over() {
    let mut read = started(reading);
    let at = read
        .centre(&format!("{PROJECTS} .ds-row-title"))
        .expect("the title");
    read.click(at);
    read.advance(ms(50));
    assert_eq!(log(&read), "select");
}
