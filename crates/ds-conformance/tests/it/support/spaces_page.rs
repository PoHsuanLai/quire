//! A toy notes app on the Spaces kit: three Spaces, a place that is restored on switch, and the
//! head, foot and menu. Shared by `spaces_switch` and `spaces_menu`.

#![allow(dead_code)]

use dioxus::prelude::*;
use ds::components::app::spaces::{
    SlideIn, SpaceHead, SpaceMenu, Spaces, SpacesFoot, SwitchChord, Switched, use_spaces,
};
use ds::components::menus::item::item::{AfterPick, MenuItem};
use ds::prelude::*;

/// The toy payload: the folder a Space opens on.
#[derive(Debug, Clone, PartialEq, Default)]
pub struct Notes {
    pub folder: String,
}

#[derive(Clone, Copy)]
enum Extra {
    None,
    Accounts,
}

fn spaces_of(count: usize) -> Spaces<Notes, String> {
    Spaces::first_run(
        (0..count).map(|n| Notes {
            folder: format!("folder {n}"),
        }),
        Notes::default,
    )
}

fn page_of(count: usize, with_extra: Extra) -> Element {
    let mut place = use_signal(String::new);
    let mut writes = use_signal(|| 0u32);
    let spaces = use_spaces(
        move || spaces_of(count),
        move |_| writes += 1,
        move || place.peek().clone(),
        move |switched: Switched<String>| place.set(switched.restore),
    );
    let new_payload = use_callback(|()| Notes::default());
    let mut picked = use_signal(String::new);
    let extra = match with_extra {
        Extra::None => Vec::new(),
        Extra::Accounts => vec![MenuItem::Submenu {
            title: "Accounts".to_owned(),
            image: None,
            availability: Availability::Enabled,
            children: ["Ada", "Bo"]
                .into_iter()
                .map(|name| {
                    MenuItem::new(name.to_owned(), name)
                        .with_check(Check::Off)
                        .with_after(AfterPick::KeepOpen)
                })
                .collect(),
        }],
    };
    rsx! {
        Ds { appearance: Appearance::default(), material: Material::Window, extent: RootExtent::Viewport, look: spaces.look(),
            div {
                class: "app",
                tabindex: "0",
                style: "padding:24px; width:260px",
                onkeydown: move |event: KeyboardEvent| {
                    spaces.on_key(SwitchChord::Command, &event);
                },
                SpaceHead { handle: spaces }
                p { class: "place", {place()} }
                p { class: "writes", "{writes}" }
                p { class: "slide", {spaces.slide().map_or("", SlideIn::attribute)} }
                p { class: "current", "{spaces.current().0}" }
                button { class: "go", onclick: move |_| place.set("sent".to_owned()), "go" }
                SpacesFoot { handle: spaces, new_payload }
                SpaceMenu::<Notes, String, String> {
                    handle: spaces,
                    new_payload,
                    extra,
                    on_extra: move |(_, name): (ds::components::app::spaces::SpaceId, String)| {
                        picked.with_mut(|log| log.push_str(&name));
                    },
                }
                p { class: "extra-log", {picked()} }
            }
        }
    }
}

#[allow(non_snake_case)]
pub fn Three() -> Element {
    page_of(3, Extra::None)
}

#[allow(non_snake_case)]
pub fn One() -> Element {
    page_of(1, Extra::None)
}

#[allow(non_snake_case)]
pub fn WithAccounts() -> Element {
    page_of(2, Extra::Accounts)
}
