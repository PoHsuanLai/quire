//! Places that can take a drag, on a real Blitz document: `DropState::Accepts`
//! writes `data-drop="accepts"` and draws its hairline inside the row's own box, so a
//! item keeps its size and its label stays where it was as a drag starts.

use dioxus::prelude::*;
use ds::base::vocab::RowState;
use ds::prelude::*;
use ds::root::common::Common;
use ds::root::pass_through::{DataAttr, DataName};
use ds_harness::{Clock, Driver, Harness, HarnessConfig, Query, Viewport};
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

/// `data-place="<name>"`.
fn place(name: &str) -> Vec<DataAttr> {
    DataName::parse("place")
        .map(|attribute| vec![DataAttr::new(attribute, name)])
        .unwrap_or_default()
}

#[allow(non_snake_case)]
fn Page() -> Element {
    rsx! {
        Ds { appearance: Appearance::default(), material: Material::Window,
            div { style: "width:200px; padding:20px",
                for (label , icon , drop) in PLACES {
                    Row {
                        state: RowState { selection: Selection::Unselected, drop, ..RowState::default() },
                        key: "{label}",
                        title: label,
                        leading: RowLeading::Icon(icon),
                        common: Common { data: place(&label.to_lowercase()), ..Common::default() },
                    }
                }
            }
        }
    }
}

#[test]
fn an_accepting_place_is_marked_and_keeps_its_box_and_label() {
    let mut harness = Harness::new(Page, HarnessConfig::new(VIEW).with_clock(Clock::Virtual));
    harness.advance(Duration::from_millis(50));
    assert_eq!(harness.count(".ds-row[*|data-drop=accepts]"), 1);
    assert_eq!(
        harness
            .attr(".ds-row[*|data-place=archive]", "data-drop")
            .as_deref(),
        Some("accepts")
    );
    let idle = harness.rect(".ds-row[*|data-place=inbox]").expect("inbox");
    let accepts = harness
        .rect(".ds-row[*|data-place=archive]")
        .expect("archive");
    assert_eq!(idle.size, accepts.size, "the hairline is inside the box");
    let idle_text = harness
        .rect(".ds-row[*|data-place=inbox] .ds-row-title")
        .expect("inbox label");
    let accepts_text = harness
        .rect(".ds-row[*|data-place=archive] .ds-row-title")
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
