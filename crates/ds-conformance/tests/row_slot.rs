//! A scheduled Today row on a real Blitz document, a `Row` with a trailing slot. Its cancel button
//! cancels without opening the row, the row still opens from its title, and its close button says
//! whose it is.

use dioxus::prelude::*;
use ds::{
    Accessory, Appearance, AvatarFace, AvatarShape, AvatarSize, AvatarTone, Ds, Icon, Material,
    PersonHue, Propagation, Row, RowLeading,
};
use ds::{Bezel, Button, ImagePosition};
use ds_harness::{Driver, Harness, Input, Query, Viewport};
use std::time::Duration;

const VIEW: Viewport = Viewport {
    width: 320,
    height: 120,
    scale_percent: 100,
};

const FACE: AvatarFace = AvatarFace {
    initial: 'Q',
    size: AvatarSize::Size18,
    tone: AvatarTone::Person(PersonHue(212)),
    shape: AvatarShape::Round,
};

/// One scheduled row with a close button too, each press logged.
#[allow(non_snake_case)]
fn Scheduled() -> Element {
    let mut log = use_signal(Vec::<&str>::new);
    rsx! {
        Ds { appearance: Appearance::default(), material: Material::Window,
            div { style: "width:260px; padding:8px",
                Row {
                    title: "Q3 notes",
                    leading: RowLeading::Avatar(FACE),
                    accessory: Accessory::Slot(rsx! {
                        span { class: "when", "Mon 9:00" }
                        Button {
                            bezel: Bezel::Toolbar, image: ImagePosition::Only,
                            icon: Icon::X,
                            label: "Cancel sending Q3 notes",
                            propagation: Propagation::Stop,
                            onclick: move |_| log.with_mut(|log| log.push("cancel")),
                        }
                        Button {
                            bezel: Bezel::Toolbar, image: ImagePosition::Only,
                            icon: Icon::X,
                            label: "Close Q3 notes",
                            propagation: Propagation::Stop,
                            onclick: move |_| log.with_mut(|log| log.push("close")),
                        }
                    }),
                    onclick: move |_| log.with_mut(|log| log.push("open")),
                }
            }
            p { class: "log", {log().join(",")} }
        }
    }
}

fn log(harness: &Harness) -> String {
    harness.text_of(".log").unwrap_or_default()
}

#[test]
fn cancel_cancels_without_opening_and_the_label_still_opens() {
    let mut harness = Harness::new(Scheduled, VIEW);
    assert_eq!(log(&harness), "");
    harness.send(Input::click(
        harness
            .centre(".ds-button[*|aria-label=\"Cancel sending Q3 notes\"]")
            .expect("the cancel"),
    ));
    harness.advance(Duration::from_millis(30));
    assert_eq!(log(&harness), "cancel");
    harness.send(Input::click(
        harness.centre(".ds-row-title").expect("the title"),
    ));
    harness.advance(Duration::from_millis(30));
    assert_eq!(log(&harness), "cancel,open");
    assert_eq!(harness.text_of(".when").as_deref(), Some("Mon 9:00"));
}

#[test]
fn the_close_button_names_its_row() {
    let harness = Harness::new(Scheduled, VIEW);
    assert_eq!(
        harness
            .attr(".ds-button[*|aria-label=\"Close Q3 notes\"]", "aria-label")
            .as_deref(),
        Some("Close Q3 notes")
    );
}
