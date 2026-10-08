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
