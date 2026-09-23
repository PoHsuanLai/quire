//! A minimal page proving quire's coherence rules from outside the quire workspace
//! (`../../CONSUMING.md` "The consumer example"): one `Ds` root, a page built only from quire
//! components (`Button`, `TextInput`, `Menu`, `Toast`), a stylesheet of its own, and motion
//! driven only by `ds::motion` timers, never an ad-hoc sleep.
//!
//! `tests/coherence.rs` is the point of this crate: it runs the four coherence rules from
//! `ORCHESTRATION.md` against `App`'s own output, the way a real consumer's tests would.

use dioxus::prelude::*;
use ds::{
    Anim, Appearance, Button, ButtonVariant, Ds, Icon, InputVariant, Material, Menu, MenuEntry,
    MenuKind, TextInput, Tile, Trail, use_motion_timer, use_rect, use_toasts,
};

/// This example's own stylesheet, quire tokens only (`tests/coherence.rs::our_stylesheet_lints_clean`).
pub const STYLE: &str = include_str!("style.css");

/// The example's one page, wrapped in the root every quire surface draws inside.
///
/// Reads `appearance()` rather than `ds_settings::use_environment` (`../../CONSUMING.md`
/// "Reading appearance" has the reason: `use_environment`'s live watches, portal and file
/// alike, need an entered Tokio runtime, which neither `ds_native::launch` nor
/// `ds_native::Harness` provides — a real gap, not worked around here). `ds_native::launch`
/// takes a plain `fn() -> Element` with no captures, so there is nowhere to hand `App` a
/// pre-loaded value from `main`; it loads its own settings synchronously instead, the same
/// pattern `ds_settings::environment::load_initial` uses inside `use_environment` itself.
#[component]
pub fn App() -> Element {
    rsx! {
        Ds {
            appearance: appearance(),
            material: Material::Window,
            style { {STYLE} }
            Page {}
        }
    }
}

/// `quire/appearance.toml`-style settings for this example's own config directory
/// (`$XDG_CONFIG_HOME/consumer`), read once per render with no live reload
/// (`ds_settings::load`, a synchronous `std::fs::read` — no Tokio runtime needed). A missing or
/// unreadable file is `Appearance::default()`, never an error: first run looks like every run
/// after it until someone saves a preference.
fn appearance() -> Appearance {
    ds_settings::config_dir(ds_settings::AppName("consumer"))
        .map(|dir| ds_settings::load(&dir).appearance.appearance())
        .unwrap_or_default()
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
            value: Action::Duplicate,
            title: "Duplicate".to_owned(),
            detail: None,
            tile: Some(Tile::Icon(Icon::Mail)),
            trail: Trail::None,
            check: None,
        },
        MenuEntry::Item {
            value: Action::Discard,
            title: "Discard".to_owned(),
            detail: None,
            tile: Some(Tile::Icon(Icon::Trash)),
            trail: Trail::None,
            check: None,
        },
    ]
}

/// A subject field, a `Send` button whose "Sent" confirmation is timed by
/// `ds::use_motion_timer` rather than a sleep, and a "More" button that opens a quire `Menu`
/// anchored to itself through `ds::use_rect` — the pattern `CONSUMING.md`'s "Component
/// catalogue" Overlays example shows, and, until wave 2 integration's `ds::HostMeasure`/
/// `Measured::Busy` fix, one that panicked under `ds_native::Harness`
/// (`CONSUMING.md` §9 records what was found and that it is now fixed; this page no longer
/// needs to route around it).
#[component]
fn Page() -> Element {
    let mut subject = use_signal(String::new);
    let mut menu_open = use_signal(|| false);
    let anchor = use_rect();
    let toasts = use_toasts();
    let badge = use_motion_timer(Anim::Fade);
    let mut sent = use_signal(|| false);

    rsx! {
        div { class: "page",
            TextInput {
                variant: InputVariant::Boxed,
                label: "Subject".to_owned(),
                value: subject(),
                oninput: move |value| subject.set(value),
            }
            div { class: "actions",
                Button {
                    variant: ButtonVariant::Primary,
                    label: "Send".to_owned(),
                    icon: Some(Icon::Send),
                    onclick: move |_| {
                        toasts.push(format!("Sent: {}", subject()), None);
                        sent.set(true);
                        badge.start(EventHandler::new(move |()| sent.set(false)));
                    },
                }
                span {
                    onmounted: move |event| anchor.on_mounted(event),
                    Button {
                        variant: ButtonVariant::Secondary,
                        label: "More".to_owned(),
                        onclick: move |_| menu_open.set(true),
                    }
                }
            }
            span {
                class: "sent-badge",
                "data-shown": if sent() { "shown" } else { "hidden" },
                "Sent"
            }
            if let (true, Some(at)) = (menu_open(), anchor.anchor()) {
                Menu {
                    kind: MenuKind::Rich,
                    anchor: at,
                    entries: entries(),
                    onpick: move |action: Action| {
                        menu_open.set(false);
                        match action {
                            Action::Duplicate => subject.set(format!("{} (copy)", subject())),
                            Action::Discard => subject.set(String::new()),
                        }
                    },
                    onclose: move |()| menu_open.set(false),
                }
            }
        }
    }
}
