//! mailo gaps 5, places that can take a drag, on a real Blitz document: `DropState::Accepts`
//! writes `data-drop="accepts"` and draws its dashed hairline inside the item's own box, so an
//! item keeps its size and its label stays where it was as a drag starts.

use dioxus::prelude::*;
use ds::components::vocab::{DropState, Here, PulseKey};
use ds::{Anim, Appearance, Ds, Icon, ItemKind, Material, PlaceId, Presence, SidebarItem};
use ds_native::{Harness, Viewport};
use std::time::Duration;

const VIEW: Viewport = Viewport {
    width: 480,
    height: 240,
    scale_percent: 100,
};

const PLACES: [(&str, Icon, DropState); 3] = [
    ("Inbox", Icon::Inbox, DropState::Idle),
    ("Archive", Icon::Archive, DropState::Accepts),
    ("Trash", Icon::Trash, DropState::Target),
];

#[allow(non_snake_case)]
fn Page() -> Element {
    rsx! {
        Ds { appearance: Appearance::default(), material: Material::Window,
            div { style: "width:200px; padding:20px",
                for (label , icon , drop) in PLACES {
                    SidebarItem {
                        key: "{label}",
                        kind: ItemKind::Place { icon },
                        label,
                        here: Here::Elsewhere,
                        count: None,
                        presence: Presence::Present,
                        preview: None,
                        pulse: PulseKey::rest(Anim::Gulp),
                        onclick: |_| {},
                        onclose: None,
                        drop,
                        place: PlaceId(label.to_lowercase()),
                    }
                }
            }
        }
    }
}

#[test]
fn an_accepting_place_is_marked_and_keeps_its_box_and_label() {
    let mut harness = Harness::new(Page, VIEW);
    harness.advance(Duration::from_millis(50));
    assert_eq!(harness.count(".ds-sidebar-item[*|data-drop=accepts]"), 1);
    assert_eq!(
        harness
            .attr(".ds-sidebar-item[*|data-place=archive]", "data-drop")
            .as_deref(),
        Some("accepts")
    );
    let idle = harness
        .rect(".ds-sidebar-item[*|data-place=inbox]")
        .expect("inbox");
    let accepts = harness
        .rect(".ds-sidebar-item[*|data-place=archive]")
        .expect("archive");
    assert_eq!(idle.size, accepts.size, "the hairline is inside the box");
    let idle_text = harness
        .rect(".ds-sidebar-item[*|data-place=inbox] .ds-sidebar-item-text")
        .expect("inbox label");
    let accepts_text = harness
        .rect(".ds-sidebar-item[*|data-place=archive] .ds-sidebar-item-text")
        .expect("archive label");
    assert_eq!(
        accepts_text.origin.x, idle_text.origin.x,
        "the label does not move"
    );
    assert_eq!(
        accepts_text.origin.y - accepts.origin.y,
        idle_text.origin.y - idle.origin.y
    );
}
