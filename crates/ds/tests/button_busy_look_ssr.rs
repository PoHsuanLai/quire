//! A busy button's two looks as markup: the spinner in the leading slot (the default), or no
//! spinner and the button's own icon turning (`BusyLook::TurnIcon`, a Get Mail arrow).

use dioxus::core::NoOpMutations;
use dioxus::prelude::*;
use ds::components::controls::button_model::{Bezel, BusyLook, ImagePosition};
use ds::prelude::*;
use ds_style::icon::Icon;

fn render(app: fn() -> Element) -> String {
    let mut dom = VirtualDom::new(app);
    dom.rebuild_in_place();
    dom.render_immediate(&mut NoOpMutations);
    let page = dioxus_ssr::render(&dom);
    // The button alone: the page also carries the stylesheet, which names every class.
    let from = page.find("<button").expect("a button");
    let to = page[from..].find("</button>").expect("its end");
    page[from..from + to].to_owned()
}

fn page(busy: BusyLook, availability: Availability, motion: Motion) -> Element {
    rsx! {
        Ds { appearance: Appearance { motion, ..Appearance::default() }, material: Material::Window,
            Button {
                label: "Get Mail",
                bezel: Bezel::Toolbar,
                image: ImagePosition::Only,
                icon: Icon::Refresh,
                availability,
                busy,
                onclick: |_| {},
            }
        }
    }
}

#[allow(non_snake_case)]
fn Turning() -> Element {
    page(BusyLook::TurnIcon, Availability::Busy, Motion::Standard)
}
#[allow(non_snake_case)]
fn TurningReduced() -> Element {
    page(BusyLook::TurnIcon, Availability::Busy, Motion::Reduced)
}
#[allow(non_snake_case)]
fn Spinning() -> Element {
    page(BusyLook::Spinner, Availability::Busy, Motion::Standard)
}
#[allow(non_snake_case)]
fn Idle() -> Element {
    page(BusyLook::TurnIcon, Availability::Enabled, Motion::Standard)
}

#[test]
fn a_busy_button_that_turns_has_no_spinner_and_its_icon_animates() {
    let html = render(Turning);
    assert!(!html.contains("ds-button-lead"), "{html}");
    assert!(!html.contains("ds-progress-indicator"), "{html}");
    assert!(html.contains("ds-button-icon a-turn"), "{html}");
    assert!(html.contains("data-pulse=\"a\""), "{html}");
    assert!(html.contains("data-busy=\"turn-icon\""), "{html}");
    assert!(html.contains("aria-busy=\"true\""), "{html}");
    assert!(
        html.contains("data-availability=\"busy\""),
        "busy as ever: {html}"
    );
}

#[test]
fn under_reduced_motion_the_same_markup_is_held_still_by_the_stylesheet() {
    assert_eq!(render(TurningReduced), render(Turning));
}

#[test]
fn the_default_look_is_the_spinner_and_an_idle_button_does_not_turn() {
    let spinning = render(Spinning);
    assert!(spinning.contains("ds-button-lead"), "{spinning}");
    assert!(spinning.contains("ds-progress-indicator"), "{spinning}");
    assert!(!spinning.contains("a-turn"), "{spinning}");
    assert!(!spinning.contains("data-busy"), "{spinning}");
    let idle = render(Idle);
    assert!(!idle.contains("a-turn"), "{idle}");
    assert!(!idle.contains("data-busy"), "{idle}");
}
