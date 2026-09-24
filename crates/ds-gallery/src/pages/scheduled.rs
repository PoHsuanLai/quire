//! A scheduled Today row for the lists page's SidebarItem section (mailo gaps 2): its time and
//! its cancel, which hides it here until the section brings it back.

use dioxus::prelude::*;
use ds::{
    Anim, AvatarFace, AvatarShape, AvatarSize, AvatarTone, Here, ItemKind, PersonHue, Presence,
    PulseKey, SidebarItem, Switch, TodayTrailing,
};

/// A draft waiting for Monday morning, cancelled with its trailing button.
#[component]
pub fn Scheduled() -> Element {
    let mut shown = use_signal(|| Switch::On);
    let avatar = AvatarFace {
        initial: 'Q',
        size: AvatarSize::Size18,
        tone: AvatarTone::Person(PersonHue::of("Q3 notes")),
        shape: AvatarShape::Round,
    };
    rsx! {
        if shown() == Switch::On {
            SidebarItem {
                kind: ItemKind::Today { avatar },
                label: "Q3 notes",
                here: Here::Elsewhere,
                count: None,
                presence: Presence::Present,
                preview: None,
                pulse: PulseKey::rest(Anim::Gulp),
                onclick: |_| {},
                onclose: None,
                trailing: TodayTrailing {
                    time: "Mon 9:00".to_string(),
                    cancel: "Cancel sending Q3 notes".to_string(),
                    on_cancel: EventHandler::new(move |()| shown.set(Switch::Off)),
                },
            }
        }
    }
}
