//! A minimal page proving quire's coherence rules from outside the quire workspace
//! (`../../CONSUMING.md` "The consumer example"): one `Ds` root, a page built only from quire
//! components (`Button`, `TextField`, `Menu`, `Toast`), a stylesheet of its own, and motion
//! driven only by `ds::motion` timers, never an ad-hoc sleep.
//!
//! `tests/coherence.rs` is the point of this crate: it runs the four coherence rules from
//! `CONSUMING.md` section 5 against `App`'s own output, the way a real consumer's tests would.

use ds::{TextField};
use ds::{Answers};
use dioxus::prelude::*;
use ds::{Anchor, Anim, Availability, Button, Common, Ds, FieldFocus, Icon, Material, Menu, MenuEntry, MenuKind, MenuTile, MenuTrail, MountedRef, use_focus_request, use_motion_timer, use_toasts};
use ds_native::TokioSpawner;
use ds_settings::{AppName, ConfigRoot, Store, SystemPrefsSource, use_environment};
use std::sync::Arc;

/// This example's own stylesheet, quire tokens only (`tests/coherence.rs::our_stylesheet_lints_clean`).
pub const STYLE: &str = include_str!("style.css");

/// The example's one page, wrapped in the root every quire surface draws inside.
///
/// Reads its settings through `ds_settings::use_environment` (`../../CONSUMING.md` "Reading
/// appearance"), live file and portal watches included. Its watches run on the Tokio runtime
/// that `ds_native::launch` enters, through `ds_native::TokioSpawner`.
#[component]
pub fn App() -> Element {
    let store = Store::new(ConfigRoot::Xdg, AppName("consumer"));
    let env = use_environment(store, SystemPrefsSource::Portal, Arc::new(TokioSpawner::current()));
    let now = env();
    rsx! {
        Ds {
            appearance: now.settings.appearance.appearance(),
            system: now.system,
            material: Material::Window,
            style { {STYLE} }
            Page {}
        }
    }
}

/// What picking a "More" menu entry does.
#[derive(Debug, Clone, Copy, PartialEq)]
enum Action {
    Duplicate,
    Discard,
}

fn entries() -> Vec<MenuEntry<Action>> {
    vec![
        MenuEntry::Item {
            availability: Availability::Enabled,
            value: Action::Duplicate,
            title: "Duplicate".to_owned(),
            detail: None,
            tile: Some(MenuTile::Icon(Icon::Mail)),
            trail: MenuTrail::None,
            check: None,
        },
        MenuEntry::Item {
            availability: Availability::Enabled,
            value: Action::Discard,
            title: "Discard".to_owned(),
            detail: None,
            tile: Some(MenuTile::Icon(Icon::Trash)),
            trail: MenuTrail::None,
            check: None,
        },
    ]
}

/// A subject field, a `Send` button whose "Sent" confirmation is timed by
/// `ds::use_motion_timer` rather than a sleep, and a "More" button that opens a quire `Menu`
/// anchored to the button itself: `Button`'s `common.mounted` hands over its element and the menu
/// takes it as `Anchor::Mounted`, measured when placing (`CONSUMING.md` §4, "Overlays"). No
/// wrapper element is measured in its place. The subject field has the keyboard as the page
/// mounts and gets it back whenever the menu closes (`FieldFocus::Controlled`, `CONSUMING.md` §6).
#[component]
fn Page() -> Element {
    let mut subject = use_signal(String::new);
    let mut menu_open = use_signal(|| false);
    let mut more = use_signal(|| None::<MountedRef>);
    let toasts = use_toasts();
    let badge = use_motion_timer(Anim::Fade);
    let mut sent = use_signal(|| false);
    let field = use_focus_request();

    rsx! {
        div { class: "page",
            TextField {
                label: "Subject".to_owned(),
                value: subject(),
                oninput: move |value| subject.set(value),
                focus: FieldFocus::Controlled(field),
            }
            div { class: "actions",
                Button {
                    answers: Answers::Return,
                    label: "Send".to_owned(),
                    icon: Some(Icon::Send),
                    onclick: move |_| {
                        toasts.push(format!("Sent: {}", subject()), None);
                        sent.set(true);
                        badge.start(EventHandler::new(move |()| sent.set(false)));
                    },
                }
                Button {
                    label: "More".to_owned(),
                    onclick: move |_| menu_open.set(true),
                    common: Common {
                        mounted: Some(EventHandler::new(move |event: MountedEvent| {
                            more.set(Some(MountedRef(event.data())));
                        })),
                        ..Common::default()
                    },
                }
            }
            span {
                class: "sent-badge",
                "data-shown": if sent() { "shown" } else { "hidden" },
                "Sent"
            }
            if let (true, Some(button)) = (menu_open(), more()) {
                Menu {
                    kind: MenuKind::Rich,
                    anchor: Anchor::Mounted(button),
                    entries: entries(),
                    onpick: move |action: Action| {
                        menu_open.set(false);
                        field.request();
                        match action {
                            Action::Duplicate => subject.set(format!("{} (copy)", subject())),
                            Action::Discard => subject.set(String::new()),
                        }
                    },
                    onclose: move |()| {
                        menu_open.set(false);
                        field.request();
                    },
                }
            }
        }
    }
}
