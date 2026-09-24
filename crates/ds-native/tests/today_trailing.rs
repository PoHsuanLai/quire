//! mailo gaps 2, step 4: a scheduled Today row on a real Blitz document. Its cancel button
//! cancels without opening the row, the row still opens from its label, and its close button
//! says whose it is.

use dioxus::prelude::*;
use ds::{
    Anim, Appearance, AvatarFace, AvatarShape, AvatarSize, AvatarTone, Ds, Here, ItemKind,
    Material, PersonHue, Presence, PulseKey, SidebarItem, TodayTrailing,
};
use ds_native::{Harness, Viewport};
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
fn Row() -> Element {
    let mut log = use_signal(Vec::<&str>::new);
    rsx! {
        Ds { appearance: Appearance::default(), material: Material::Window,
            div { style: "width:260px; padding:8px",
                SidebarItem {
                    kind: ItemKind::Today { avatar: FACE },
                    label: "Q3 notes",
                    here: Here::Elsewhere,
                    count: None,
                    presence: Presence::Present,
                    preview: None,
                    pulse: PulseKey::rest(Anim::Gulp),
                    onclick: move |()| log.with_mut(|log| log.push("open")),
                    onclose: move |()| log.with_mut(|log| log.push("close")),
                    trailing: TodayTrailing {
                        time: "Mon 9:00".to_string(),
                        cancel: "Cancel sending Q3 notes".to_string(),
                        on_cancel: EventHandler::new(move |()| log.with_mut(|log| log.push("cancel"))),
                    },
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
    let mut harness = Harness::new(Row, VIEW);
    assert_eq!(log(&harness), "");
    harness.click(
        harness
            .centre(".ds-sidebar-item-cancel")
            .expect("the cancel"),
    );
    harness.advance(Duration::from_millis(30));
    assert_eq!(log(&harness), "cancel");
    harness.click(harness.centre(".ds-sidebar-item-text").expect("the label"));
    harness.advance(Duration::from_millis(30));
    assert_eq!(log(&harness), "cancel,open");
    assert_eq!(
        harness.text_of(".ds-sidebar-item-time").as_deref(),
        Some("Mon 9:00")
    );
}

#[test]
fn the_close_button_names_its_row() {
    let harness = Harness::new(Row, VIEW);
    assert_eq!(
        harness
            .attr(".ds-sidebar-item-close", "aria-label")
            .as_deref(),
        Some("Close Q3 notes")
    );
}
