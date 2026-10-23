//! The titlebar's leading slot on a real Blitz document: it is drawn after the lights and before
//! the title, a click on it reaches the app's own control and a press that travels over it begins
//! no window move, and it keeps the title where each placement puts it (centred in the bar, or in
//! the room between the leading and trailing controls).

use crate::window_frame_controls::{BAR, Stub, VIEW, at, centre, drag, log, ms, start};
use dioxus::prelude::*;
use ds::components::chrome::window_frame::{TitleRoom, WindowTitlebar};
use ds::prelude::*;
use ds::window::host::use_window_host_provider;
use ds::window::vocab::Maximized;
use ds_harness::Input;
use std::rc::Rc;

#[allow(non_snake_case)]
fn Led() -> Element {
    let log = use_signal(Vec::<String>::new);
    let mut picked = use_signal(|| 0u32);
    use_window_host_provider(move || {
        Rc::new(Stub {
            log,
            maximized: Maximized::Off,
        })
    });
    let leading = rsx! {
        button { class: "side", onclick: move |_| picked += 1, "Sidebar" }
    };
    rsx! {
        Ds {
            appearance: Appearance::default(),
            material: Material::Window,
            window: WindowFrame::titlebar("Inbox", TrafficLights::Shown).with_leading(leading),
            p { class: "log", {log().join(",")} }
            p { class: "picked", "{picked}" }
        }
    }
}

/// A bare titlebar with a wide leading control and a trailing one, the title centred.
#[allow(non_snake_case)]
fn BarCentred() -> Element {
    bar(TitleRoom::Centred)
}

/// The same bar with the title kept to the room between the two controls.
#[allow(non_snake_case)]
fn BarBeside() -> Element {
    bar(TitleRoom::Beside)
}

fn bar(title_room: TitleRoom) -> Element {
    rsx! {
        Ds {
            appearance: Appearance::default(),
            material: Material::Window,
            WindowTitlebar {
                title: "Inbox",
                title_room,
                leading: rsx! { button { class: "side", style: "width:120px", "Sidebar" } },
                trailing: rsx! { button { class: "mode", style: "width:40px", "Mode" } },
            }
        }
    }
}

#[test]
fn the_leading_slot_is_drawn_after_the_lights_and_before_the_title() {
    let harness = start(Led);
    let html = harness.html();
    let lights = html.find("class=\"ds-lights").expect("lights");
    let slot = html.find("class=\"ds-titlebar-leading").expect("leading slot");
    let title = html.find("class=\"ds-titlebar-title").expect("title area");
    assert!(lights < slot && slot < title, "{html}");
    assert!(centre(&harness, ".ds-titlebar-leading .side").x.0 < VIEW.width as f32 / 4.0);
}

#[test]
fn a_click_on_the_leading_control_reaches_it_and_moves_nothing() {
    let mut harness = start(Led);
    let side = centre(&harness, ".side");
    harness.send(Input::click(side));
    harness.advance(ms(60));
    assert_eq!(harness.text_of(".picked").as_deref(), Some("1"));
    assert_eq!(log(&harness), "");
}

#[test]
fn a_press_on_the_leading_slot_does_not_begin_a_move() {
    let mut harness = start(Led);
    let from = centre(&harness, ".side");
    let path = [
        at(from.x.0 + 8.0, from.y.0),
        at(from.x.0 + 40.0, from.y.0 + 3.0),
    ];
    drag(&mut harness, from, &path);
    assert_eq!(log(&harness), "", "{}", harness.html());
    drag(&mut harness, BAR, &[at(300.0, 16.0), at(330.0, 20.0)]);
    assert_eq!(
        log(&harness),
        "move",
        "the empty bar still moves the window"
    );
}

#[test]
fn a_centred_title_stays_in_the_middle_of_the_bar_whatever_leads() {
    let harness = start(BarCentred);
    let title = centre(&harness, ".ds-titlebar-title").x.0;
    assert!((title - VIEW.width as f32 / 2.0).abs() < 1.5, "{title}");
}

#[test]
fn a_beside_title_is_centred_between_the_leading_and_trailing_controls() {
    let harness = start(BarBeside);
    let side = centre(&harness, ".ds-titlebar-leading .side").x.0;
    let mode = centre(&harness, ".ds-titlebar-trailing .mode").x.0;
    let title = centre(&harness, ".ds-titlebar-title").x.0;
    assert!(side < title && title < mode, "{side} {title} {mode}");
    // The leading control is wider than the trailing one, so the room's middle lies right of the
    // bar's middle by half the difference of their widths (80 px) and the title follows it.
    assert!(title > VIEW.width as f32 / 2.0, "{title}");
}
