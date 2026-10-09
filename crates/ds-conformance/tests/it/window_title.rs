//! `use_window_title`: the window's title is set from the app after launch and follows the
//! app's own value, through the host seam.

use crate::window_frame_controls::{Stub, VIEW, centre, log, ms};
use dioxus::prelude::*;
use ds::prelude::*;
use ds::window::host::{use_window_host_provider, use_window_title};
use ds::window::vocab::Maximized;
use ds_harness::{Clock, Driver, Harness, HarnessConfig, Input};
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
