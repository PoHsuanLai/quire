//! Lists, mailo gaps 4: a strip whose press acts before any measurement.

use super::Section;
use dioxus::prelude::*;
use ds::{
    ActionId, Anim, Emphasis, HoverStrip, Icon, ListRow, Presence, Selection, Shown, StaggerIndex,
    StripAction,
};

/// Archive and snooze, each doing nothing on its measured click: the press says what happened.
fn actions() -> Vec<StripAction> {
    [
        (
            "archive",
            Icon::Archive,
            "Archive",
            "Archive → out of Inbox",
        ),
        ("snooze", Icon::Clock, "Snooze", "Snooze until…"),
    ]
    .into_iter()
    .map(|(id, icon, label, fly)| StripAction {
        id: ActionId(id.to_string()),
        icon,
        label: label.to_string(),
        fly: fly.to_string(),
        onhover: None,
        onclick: EventHandler::new(|_| {}),
    })
    .collect()
}

/// A row whose strip's press is heard at once.
#[component]
pub fn StripPress() -> Element {
    let mut pressed = use_signal(|| None::<ActionId>);
    let said = pressed().map_or("nothing yet".to_string(), |id| id.0);
    rsx! {
        Section {
            title: "A strip that acts before it measures",
            note: "on_press hears the button inside the click, before the rect read; the measured onclick follows only where layout answers.",
            ul { class: "g-list g-stage-pad",
                ListRow {
                    selection: Selection::Selected,
                    emphasis: Emphasis::Strong,
                    index: StaggerIndex::new(0),
                    presence: Presence::Present,
                    name: "Dana Okafor",
                    via: None,
                    subject: "Re: UIDL stability",
                    snippet: None,
                    time: "09:41",
                    tags: rsx! {},
                    star: None,
                    star_pulse: ds::PulseKey::rest(Anim::StarPop),
                    strip: rsx! {
                        HoverStrip {
                            actions: actions(),
                            shown: Shown::Visible,
                            on_press: move |id| pressed.set(Some(id)),
                        }
                    },
                    onclick: |_| {},
                }
            }
            p { class: "g-note", "Pressed: {said}" }
        }
    }
}
