//! mailo gaps 4, sidebar places as drop targets, on a real Blitz document: each place names
//! itself in `data-place`, hands the caller its pointer's entry, moves, exit and release, and
//! lights as the target (`data-drop=target`) while the caller says it is (design/06 section
//! 6.1).

use dioxus::prelude::*;
use ds::components::vocab::{DropState, Here, PulseKey};
use ds::{
    Anim, Appearance, Ds, Icon, ItemKind, Material, PlaceId, Point, Presence, Px, SidebarItem,
};
use ds_native::{Harness, Viewport};
use std::time::Duration;

const VIEW: Viewport = Viewport {
    width: 720,
    height: 320,
    scale_percent: 100,
};

const PLACES: [(&str, &str, Icon); 3] = [
    ("inbox", "Inbox", Icon::Inbox),
    ("archive", "Archive", Icon::Archive),
    ("label:7", "Invoices", Icon::Tag),
];

/// The sidebar during a drag the page tracks itself: the place under the pointer is the target.
#[allow(non_snake_case)]
fn Page() -> Element {
    let mut log = use_signal(Vec::<String>::new);
    let mut over = use_signal(|| None::<&'static str>);
    let mut moves = use_signal(|| 0u32);
    rsx! {
        Ds { appearance: Appearance::default(), material: Material::Window,
            div { class: "side", style: "width:200px; padding:20px",
                for (id , label , icon) in PLACES {
                    SidebarItem {
                        key: "{id}",
                        kind: ItemKind::Place { icon },
                        label,
                        here: Here::Elsewhere,
                        count: None,
                        presence: Presence::Present,
                        preview: None,
                        pulse: PulseKey::rest(Anim::Gulp),
                        onclick: |_| {},
                        onclose: None,
                        drop: if over() == Some(id) { DropState::Target } else { DropState::Idle },
                        place: PlaceId(id.to_string()),
                        onpointerenter: move |_| {
                            over.set(Some(id));
                            log.with_mut(|log| log.push(format!("enter:{id}")));
                        },
                        onpointerleave: move |_| {
                            over.set(None);
                            log.with_mut(|log| log.push(format!("leave:{id}")));
                        },
                        onpointermove: move |_| moves += 1,
                        onpointerup: move |_| log.with_mut(|log| log.push(format!("drop:{id}"))),
                    }
                }
            }
            p { class: "log", {log().join(",")} }
            p { class: "moves", "{moves}" }
        }
    }
}

fn centre(harness: &Harness, selector: &str) -> Point {
    harness
        .centre(selector)
        .unwrap_or_else(|| panic!("{selector} is not in the document:\n{}", harness.html()))
}

fn item(id: &str) -> String {
    format!(".ds-sidebar-item[*|data-place=\"{id}\"]")
}

#[test]
fn a_place_names_itself_and_hands_its_pointer_to_the_caller() {
    let mut harness = Harness::new(Page, VIEW);
    harness.advance(Duration::from_millis(50));
    assert_eq!(harness.count(".ds-sidebar-item[*|data-place]"), 3);
    assert_eq!(
        harness
            .attr(".ds-sidebar-item:nth-child(3)", "data-place")
            .as_deref(),
        Some("label:7")
    );
    let archive = centre(&harness, &item("archive"));
    harness.pointer_move(archive);
    harness.pointer_move(Point {
        x: Px(archive.x.0 + 10.0),
        ..archive
    });
    harness.advance(Duration::from_millis(50));
    assert_eq!(
        harness.attr(&item("archive"), "data-drop").as_deref(),
        Some("target"),
        "the caller lit the place under the pointer"
    );
    assert_eq!(harness.attr(&item("inbox"), "data-drop"), None);
    let label = centre(&harness, &item("label:7"));
    harness.pointer_move(label);
    harness.pointer_up(label);
    harness.advance(Duration::from_millis(50));
    assert_eq!(
        harness.text_of(".log").as_deref(),
        Some("enter:archive,leave:archive,enter:label:7,drop:label:7")
    );
    assert_eq!(
        harness.attr(&item("label:7"), "data-drop").as_deref(),
        Some("target")
    );
    let moves: u32 = harness
        .text_of(".moves")
        .and_then(|text| text.parse().ok())
        .unwrap_or_default();
    assert!(moves >= 2, "the moves were heard: {moves}");
}
