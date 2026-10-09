//! `use_window_title`: the window's title is set from the app after launch and follows the
//! app's own value, through the host seam.

use crate::window_frame_controls::{Stub, VIEW, centre, log, ms};
use dioxus::prelude::*;
use ds::prelude::*;
use ds::window::host::{use_window_host, use_window_host_provider, use_window_title};
use ds::window::icon::WindowIcon;
use ds::window::vocab::Maximized;
use ds_harness::{Clock, Driver, Harness, HarnessConfig, Input, WindowHosting};
use std::rc::Rc;

#[allow(non_snake_case)]
fn Titled() -> Element {
    let log = use_signal(Vec::<String>::new);
    let mut opened = use_signal(|| 1u32);
    use_window_host_provider(move || {
        Rc::new(Stub {
            log,
            maximized: Maximized::Off,
        })
    });
    use_window_title(format!("Draft {opened}"));
    rsx! {
        Ds { appearance: Appearance::default(), material: Material::Window,
            button { class: "next", onclick: move |_| opened += 1, "Next" }
            p { class: "log", {log().join(",")} }
        }
    }
}

#[test]
fn the_title_is_set_at_start_and_again_when_the_app_changes_it() {
    let mut harness = Harness::new(Titled, HarnessConfig::new(VIEW).with_clock(Clock::Virtual));
    harness.advance(ms(50));
    assert_eq!(log(&harness), "title:Draft 1");
    let next = centre(&harness, ".next");
    harness.send(Input::click(next));
    harness.advance(ms(50));
    assert_eq!(log(&harness), "title:Draft 1,title:Draft 2");
}

#[allow(non_snake_case)]
fn Hosted() -> Element {
    let mut opened = use_signal(|| 1u32);
    use_window_title(format!("Draft {opened}"));
    let host = use_window_host();
    rsx! {
        Ds { appearance: Appearance::default(), material: Material::Window,
            button { class: "next", onclick: move |_| opened += 1, "Next" }
            button { class: "icon",
                onclick: move |_| {
                    if let (Some(host), Ok(icon)) = (host.as_ref(), WindowIcon::new(vec![255; 4], 1, 1)) {
                        host.set_icon(&icon);
                    }
                },
                "Icon"
            }
        }
    }
}

#[test]
fn a_recording_window_host_keeps_the_titles_and_the_icon_the_app_sets() {
    let config = HarnessConfig::new(VIEW)
        .with_clock(Clock::Virtual)
        .with_window_hosting(WindowHosting::Recorded);
    let mut harness = Harness::new(Hosted, config);
    harness.advance(ms(50));
    assert_eq!(harness.window_titles(), ["Draft 1"]);
    assert_eq!(harness.window_icon(), None);
    for selector in [".next", ".icon"] {
        let at = centre(&harness, selector);
        harness.send(Input::click(at));
        harness.advance(ms(50));
    }
    assert_eq!(harness.window_title().as_deref(), Some("Draft 2"));
    assert_eq!(harness.window_icon().map(|icon| icon.width()), Some(1));
}
