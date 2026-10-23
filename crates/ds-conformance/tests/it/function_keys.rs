//! A function key sent through the harness reaches an app as chordkit's action: F9 is
//! `ToggleSidebar` in the CUA table, and a Mac table (Ctrl+Command+S) leaves it alone.

use chordkit::{Action, Context, Desktop, Platform, StandardAction};
use dioxus::prelude::*;
use ds::keys::{ActionTaken, on_action};
use ds::prelude::*;
use ds_blitz::FocusFallback;
use ds_harness::{Clock, Harness, HarnessConfig, Input, Viewport};
use std::time::Duration;

const VIEW: Viewport = Viewport {
    width: 300,
    height: 120,
    scale_percent: 100,
};

#[allow(non_snake_case)]
fn Page() -> Element {
    rsx! {
        Ds { appearance: Appearance::default(), material: Material::Window, extent: RootExtent::Viewport,
            Pad {}
        }
    }
}

/// Under `Ds`, so `use_keys` reads the root's keymap (the harness's platform), not the default.
#[allow(non_snake_case)]
#[component]
fn Pad() -> Element {
    let keys = ds::keys::use_keys();
    let mut log = use_signal(String::new);
    rsx! {
        div {
            id: "pad",
            tabindex: "0",
            style: "width:200px;height:60px",
            onkeydown: on_action(keys, Context::Normal, move |action| {
                log.with_mut(|log| log.push_str(&format!("{action:?};")));
                ActionTaken::Yes
            }),
            "pad"
        }
        span { id: "log", "{log}" }
    }
}

fn started(platform: Platform) -> Harness {
    let config = HarnessConfig::new(VIEW)
        .with_clock(Clock::Virtual)
        .with_focus_fallback(FocusFallback::BlitzDefault)
        .with_platform(platform);
    let mut harness = Harness::new(Page, config);
    harness.advance(Duration::from_millis(100));
    // Tab gives the pad the keys (a click does not focus a plain element here).
    harness.send(Input::key(ShortcutKey::Tab));
    harness.advance(Duration::from_millis(16));
    harness
}

fn heard(harness: &Harness) -> String {
    harness.text_of("#log").unwrap_or_default()
}

#[test]
fn f9_is_toggle_sidebar_under_the_cua_table() {
    let mut harness = started(Platform::Linux {
        desktop: Desktop::Other,
    });
    harness.send(Input::key(ShortcutKey::F(9)));
    harness.advance(Duration::from_millis(16));
    harness.advance(Duration::from_millis(16));
    let want = format!("{:?};", Action::Standard(StandardAction::ToggleSidebar));
    assert_eq!(heard(&harness), want);
}

#[test]
fn f9_means_nothing_to_the_mac_table() {
    let mut harness = started(Platform::MacOs);
    harness.send(Input::key(ShortcutKey::F(9)));
    harness.advance(Duration::from_millis(16));
    harness.advance(Duration::from_millis(16));
    assert_eq!(heard(&harness), "");
}
