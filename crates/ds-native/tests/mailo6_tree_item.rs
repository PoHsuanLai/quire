//! mailo gaps 6, a folder tree on a real Blitz document: `TreeItem` is a `details`/`summary`
//! whose state is the app's. A press on the row asks for the other state through `on_toggle`
//! (and the details follows the app, not the summary's own toggle); a press on the trailing ⋯
//! is the button's alone, with `Propagation::Stop` and without it (the slot fences it); a press
//! on a selectable label selects without toggling; and the drop states are written and painted
//! by the rule set SidebarItem shares.

use dioxus::prelude::*;
use ds::{
    Appearance, DataAttr, DataName, Disclosure, DropState, Ds, Icon, IconButton, IconButtonVariant,
    Material, PlaceId, Point, Propagation, Px, TreeItem, TreeShape,
};
use ds_native::{Harness, Viewport};
use std::time::Duration;

const VIEW: Viewport = Viewport {
    width: 360,
    height: 320,
    scale_percent: 100,
};

/// `data-folder="<path>"` for a ⋯ button.
fn folder(path: &str) -> Vec<DataAttr> {
    DataName::parse("folder")
        .map(|name| vec![DataAttr::new(name, path)])
        .unwrap_or_default()
}

/// Two branches, each with a leaf; Projects accepts the drag, Archive is its target. Projects'
/// ⋯ stops its press; Archive's does not, and relies on the slot.
#[allow(non_snake_case)]
fn Page() -> Element {
    let mut log = use_signal(Vec::<String>::new);
    let mut note = move |entry: String| log.with_mut(|log| log.push(entry));
    let mut projects = use_signal(|| Disclosure::Open);
    let mut archive = use_signal(|| Disclosure::Closed);
    rsx! {
        Ds { appearance: Appearance::default(), material: Material::Window,
            div { class: "tree", style: "width:240px; padding:20px",
                TreeItem {
                    label: "Projects",
                    open: projects(),
                    on_toggle: move |to: Disclosure| {
                        note(format!("toggle:projects:{to:?}"));
                        projects.set(to);
                    },
                    glyph: Icon::Folder,
                    drop: DropState::Accepts,
                    place: PlaceId("projects".to_string()),
                    onselect: move |_| note("select:projects".to_string()),
                    trailing: rsx! {
                        IconButton { variant: IconButtonVariant::Strip, icon: Icon::Ellipsis, label: "Actions for Projects",
                            propagation: Propagation::Stop, data: folder("INBOX/Projects"),
                            onclick: move |_| note("more:projects".to_string()) }
                    },
                    TreeItem { label: "Quire", open: Disclosure::Closed, on_toggle: |_| {}, shape: TreeShape::Leaf, place: PlaceId("quire".to_string()) }
                }
                TreeItem {
                    label: "Archive",
                    open: archive(),
                    on_toggle: move |to: Disclosure| {
                        note(format!("toggle:archive:{to:?}"));
                        archive.set(to);
                    },
                    glyph: Icon::Archive,
                    drop: DropState::Target,
                    place: PlaceId("archive".to_string()),
                    trailing: rsx! {
                        IconButton { variant: IconButtonVariant::Strip, icon: Icon::Ellipsis, label: "Actions for Archive",
                            data: folder("Archive"), onclick: move |_| note("more:archive".to_string()) }
                    },
                    TreeItem { label: "2025", open: Disclosure::Closed, on_toggle: |_| {}, shape: TreeShape::Leaf, place: PlaceId("2025".to_string()) }
                }
                TreeItem { label: "Receipts", open: Disclosure::Closed, on_toggle: |_| {}, shape: TreeShape::Leaf, glyph: Icon::Folder, place: PlaceId("receipts".to_string()) }
            }
            p { class: "log", {log().join(",")} }
        }
    }
}

fn settled() -> Harness {
    let mut harness = Harness::new(Page, VIEW);
    harness.advance(Duration::from_millis(50));
    harness
}

fn centre(harness: &Harness, selector: &str) -> Point {
    harness
        .centre(selector)
        .unwrap_or_else(|| panic!("{selector} is not laid out:\n{}", harness.html()))
}

fn press(harness: &mut Harness, at: Point) {
    harness.click(at);
    harness.advance(Duration::from_millis(50));
}

/// Whether the branch at `place` shows its child: the row drawn after the branch (`next`)
/// starts more than one row below the branch's own. On Blitz a hidden child is the user-agent
/// sheet's reading of the `details`' `open`, so this holds only if the attribute is one the
/// sheet sees. (A hidden node keeps its last layout box, so its own rect cannot say.)
fn is_open(harness: &Harness, place: &str, next: &str) -> bool {
    let row = harness
        .rect(&format!("summary[*|data-place={place}]"))
        .unwrap_or_else(|| panic!("{place} is not laid out"));
    let after = harness
        .rect(&format!(".ds-tree-item-row[*|data-place={next}]"))
        .unwrap_or_else(|| panic!("{next} is not laid out"));
    let shown = after.origin.y.0 - row.origin.y.0 > row.size.height.0 * 1.5;
    let expanded = harness.attr(&format!("summary[*|data-place={place}]"), "aria-expanded");
    assert_eq!(
        expanded.as_deref() == Some("true"),
        shown,
        "{place}: aria-expanded {expanded:?} but child shown {shown}"
    );
    shown
}

fn log(harness: &Harness) -> String {
    harness.text_of(".log").unwrap_or_default()
}

#[test]
fn the_trailing_button_keeps_its_press_and_the_row_asks_the_app() {
    let mut harness = settled();
    assert!(is_open(&harness, "projects", "archive") && !is_open(&harness, "archive", "receipts"));
    assert_eq!(
        harness.attr("details.ds-tree-item", "open").as_deref(),
        Some("true")
    );

    // The ⋯ that stops its press, and the one that does not: neither toggles.
    let more = centre(&harness, "summary[*|data-place=projects] .ds-icon-button");
    press(&mut harness, more);
    let more = centre(&harness, "summary[*|data-place=archive] .ds-icon-button");
    press(&mut harness, more);
    assert_eq!(log(&harness), "more:projects,more:archive");
    assert!(is_open(&harness, "projects", "archive") && !is_open(&harness, "archive", "receipts"));

    // The selectable label selects and does not toggle.
    let label = centre(
        &harness,
        "summary[*|data-place=projects] .ds-tree-item-label",
    );
    press(&mut harness, label);
    assert_eq!(log(&harness), "more:projects,more:archive,select:projects");
    assert!(is_open(&harness, "projects", "archive"));

    // The chevron is the row: the app hears the state asked for, and the details follows it.
    let chevron = centre(
        &harness,
        "summary[*|data-place=projects] .ds-tree-item-chevron",
    );
    press(&mut harness, chevron);
    let archive = centre(
        &harness,
        "summary[*|data-place=archive] .ds-tree-item-label",
    );
    press(&mut harness, archive);
    assert_eq!(
        log(&harness),
        "more:projects,more:archive,select:projects,toggle:projects:Closed,toggle:archive:Open"
    );
    assert!(!is_open(&harness, "projects", "archive") && is_open(&harness, "archive", "receipts"));
    assert_eq!(
        harness
            .attr("summary[*|data-place=projects]", "aria-expanded")
            .as_deref(),
        Some("false")
    );
}

#[test]
fn the_drop_states_and_the_consumer_data_render() {
    let harness = settled();
    let drop = |place: &str| {
        harness.attr(
            &format!(".ds-tree-item-row[*|data-place={place}]"),
            "data-drop",
        )
    };
    assert_eq!(drop("projects").as_deref(), Some("accepts"));
    assert_eq!(drop("archive").as_deref(), Some("target"));
    assert_eq!(drop("receipts"), None);
    assert_eq!(harness.count(".ds-drop-place[*|data-drop]"), 2);
    assert_eq!(
        harness
            .attr(
                "summary[*|data-place=projects] .ds-icon-button",
                "data-folder"
            )
            .as_deref(),
        Some("INBOX/Projects")
    );

    // Accepts draws its hairline inside the row's box: the same size as an idle row's.
    let accepts = harness
        .rect(".ds-tree-item-row[*|data-place=projects]")
        .expect("projects");
    let idle = harness
        .rect(".ds-tree-item-row[*|data-place=receipts]")
        .expect("receipts");
    assert_eq!(accepts.size, idle.size, "the hairline is inside the box");
    let label = |place: &str| {
        harness
            .rect(&format!(
                ".ds-tree-item-row[*|data-place={place}] .ds-tree-item-label"
            ))
            .expect("label")
    };
    assert_eq!(
        label("projects").origin.x - accepts.origin.x,
        label("receipts").origin.x - idle.origin.x,
        "the label does not move"
    );
}

#[test]
fn the_target_row_is_painted_by_the_shared_rule() {
    let mut harness = settled();
    let target = harness
        .rect(".ds-tree-item-row[*|data-place=archive]")
        .expect("archive");
    let idle = harness
        .rect(".ds-tree-item-row[*|data-place=receipts]")
        .expect("receipts");
    let image = harness.render().expect("a frame");
    // Just inside each row's right end, clear of the label and of the hidden ⋯.
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
