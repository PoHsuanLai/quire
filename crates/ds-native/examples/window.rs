//! A quire window on the portable Blitz shell: a `Ds` root with a Button that opens a Menu and
//! raises the undo Toast. It closes itself after three seconds, or on Escape.
//!
//! `cargo run -p ds-native --example window`

use dioxus::prelude::*;
use dioxus_native::use_window_event;
use dioxus_native::winit::event::{ElementState, WindowEvent};
use dioxus_native::winit::keyboard::{Key, NamedKey};
use ds::{
    Anchor, Appearance, Button, ButtonVariant, Ds, Material, Menu, MenuEntry, MenuKind, Point, Px,
    Switch, Trail, use_toast_hub,
};
use ds_native::{AppConfig, launch};
use std::time::Duration;

/// How long the window stays up on its own.
const LIFETIME: Duration = Duration::from_secs(3);

fn main() {
    launch(
        App,
        AppConfig {
            title: "quire: ds-native window".into(),
            width: 480,
            height: 320,
        },
    );
}

#[allow(non_snake_case)]
fn App() -> Element {
    use_hook(|| {
        spawn(async {
            ds::sleep(LIFETIME).await;
            std::process::exit(0);
        })
    });
    use_window_event(|event, event_loop| {
        if let WindowEvent::KeyboardInput { event, .. } = event
            && event.state == ElementState::Pressed
            && event.logical_key == Key::Named(NamedKey::Escape)
        {
            event_loop.exit();
        }
    });
    rsx! {
        Ds { appearance: Appearance::default(), material: Material::Window, Demo {} }
    }
}

/// The controls, inside the root so they can reach its toast hub.
#[allow(non_snake_case)]
fn Demo() -> Element {
    let mut open = use_signal(|| Switch::Off);
    let toasts = use_toast_hub();
    rsx! {
        Button {
            variant: ButtonVariant::Primary,
            label: "Snooze…",
            pressed: Some(open()),
            onclick: move |_| {
                open.set(Switch::On);
                toasts.push("Snoozed until tomorrow".into(), None);
            },
        }
        if open() == Switch::On {
            Menu {
                kind: MenuKind::Slim,
                anchor: Anchor::Point(Point { x: Px(24.0), y: Px(72.0) }),
                entries: entries(),
                onpick: move |_: u8| open.set(Switch::Off),
                onclose: move |_| open.set(Switch::Off),
            }
        }
    }
}

fn entries() -> Vec<MenuEntry<u8>> {
    ["Later today", "Tomorrow", "Next week"]
        .into_iter()
        .zip(0..)
        .map(|(title, value)| MenuEntry::Item {
            value,
            title: title.into(),
            detail: None,
            tile: None,
            trail: Trail::None,
            check: None,
        })
        .collect()
}
