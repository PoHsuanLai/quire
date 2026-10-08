//! A tip writes no `title`, so the control says it as `aria-description` (AXHelp): under `Ds`
//! the title text is the description, inside a `Tooltip` the wrapper's text stands and the
//! button adds none.

use dioxus::core::NoOpMutations;
use dioxus::prelude::*;
use ds::components::controls::button_model::{Bezel, ImagePosition};
use ds::prelude::*;
use ds_style::icon::Icon;

fn button_markup(app: fn() -> Element) -> String {
    let mut dom = VirtualDom::new(app);
    dom.rebuild_in_place();
    dom.render_immediate(&mut NoOpMutations);
    let page = dioxus_ssr::render(&dom);
    let from = page.find("<button").expect("a button");
    let to = page[from..].find("</button>").expect("its end");
    page[from..from + to].to_owned()
}

fn icon_only() -> Element {
    rsx! {
        Button {
            label: "Sidebar",
            title: Some("Hide the sidebar (\u{2303}\u{2318}S)".to_owned()),
            bezel: Bezel::Toolbar,
            image: ImagePosition::Only,
            icon: Icon::Refresh,
            onclick: |_| {},
        }
    }
}

#[allow(non_snake_case)]
fn Titled() -> Element {
    rsx! {
        Ds { appearance: Appearance::default(), material: Material::Window, {icon_only()} }
    }
}

#[allow(non_snake_case)]
fn Wrapped() -> Element {
    rsx! {
        Ds { appearance: Appearance::default(), material: Material::Window,
            Tooltip { text: "Hide the sidebar", {icon_only()} }
        }
    }
}

#[test]
fn a_titled_icon_button_describes_itself() {
    let html = button_markup(Titled);
    assert!(
        html.contains("aria-description=\"Hide the sidebar (\u{2303}\u{2318}S)\""),
        "{html}"
    );
    assert!(!html.contains(" title="), "{html}");
}

#[test]
fn a_button_inside_a_tooltip_adds_no_description() {
    let html = button_markup(Wrapped);
    assert!(!html.contains("aria-description"), "{html}");
}

fn same_words() -> Element {
    rsx! {
        Ds { appearance: Appearance::default(), material: Material::Window,
            Button {
                label: "Zoom out",
                title: Some(" Zoom out ".to_owned()),
                bezel: Bezel::Toolbar,
                image: ImagePosition::Only,
                icon: Icon::Refresh,
                onclick: |_| {},
            }
        }
    }
}

#[test]
fn a_tip_equal_to_the_accessible_name_adds_no_description() {
    let html = button_markup(same_words);
    assert!(html.contains("aria-label=\"Zoom out\""), "{html}");
    assert!(!html.contains("aria-description"), "{html}");
    assert!(html.contains("data-tip=\"Zoom out\""), "{html}");
}

#[test]
fn a_tip_that_differs_from_the_accessible_name_is_kept() {
    let html = button_markup(Titled);
    assert!(html.contains("aria-label=\"Sidebar\""), "{html}");
    assert!(
        html.contains("aria-description=\"Hide the sidebar"),
        "{html}"
    );
    assert!(!html.contains("data-tip"), "{html}");
}

fn untitled() -> Element {
    rsx! {
        Ds { appearance: Appearance::default(), material: Material::Window,
            Button { label: "Zoom out", bezel: Bezel::Toolbar, image: ImagePosition::Only, icon: Icon::Refresh, onclick: |_| {} }
        }
    }
}

#[test]
fn an_untitled_button_carries_no_tip_marker() {
    let html = button_markup(untitled);
    assert!(!html.contains("data-tip"), "{html}");
}
