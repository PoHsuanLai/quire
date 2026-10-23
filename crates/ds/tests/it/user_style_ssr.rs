//! The look and the person's stylesheets on the `Ds` root: one `<style data-ds-style>` each, after
//! the design-system sheet and in cascade order (look, desktop-wide, app), redrawn when the signal
//! changes, absent when blank, and all absent when "Use my styles" is off.

use dioxus::core::VirtualDom;
use dioxus::prelude::*;
use ds::assembly::ds::Inject;
use ds::prelude::*;
use ds_style::kit::{CustomStyles, LookStyle, Styles, UserStyle};

use std::cell::Cell;
use std::rc::Rc;

/// The signal the root under test reads, handed out so a test can change it.
#[derive(Clone, Default)]
struct Handle(Rc<Cell<Option<Signal<Styles>>>>);

impl PartialEq for Handle {
    fn eq(&self, other: &Self) -> bool {
        Rc::ptr_eq(&self.0, &other.0)
    }
}

#[derive(Props, Clone, PartialEq)]
struct Setup {
    text: String,
    stylesheet: Inject,
    custom: CustomStyles,
    handle: Handle,
}

#[allow(non_snake_case)]
fn Root(setup: Setup) -> Element {
    let styles = use_signal(|| Styles {
        desktop: UserStyle(setup.text.clone()),
        ..Styles::default()
    });
    setup.handle.0.set(Some(styles));
    rsx! {
        Ds {
            appearance: Appearance::default(),
            material: Material::Popover,
            stylesheet: setup.stylesheet,
            styles,
            custom_styles: setup.custom,
            div { class: "probe" }
        }
    }
}

fn dom(text: &str, stylesheet: Inject) -> (VirtualDom, Handle) {
    dom_with(text, stylesheet, CustomStyles::Use)
}

fn dom_with(text: &str, stylesheet: Inject, custom: CustomStyles) -> (VirtualDom, Handle) {
    let handle = Handle::default();
    let setup = Setup {
        text: text.to_owned(),
        stylesheet,
        custom,
        handle: handle.clone(),
    };
    let mut vdom = VirtualDom::new_with_props(Root, setup);
    vdom.rebuild_in_place();
    (vdom, handle)
}

fn html(text: &str, stylesheet: Inject) -> String {
    dioxus_ssr::render(&dom(text, stylesheet).0)
}

#[test]
fn the_user_sheet_comes_after_the_design_system_sheet() {
    let html = html(".ds { --accent: red }", Inject::Inline);
    let ours = html
        .find("== tokens ==")
        .unwrap_or_else(|| panic!("{html}"));
    let theirs = html
        .find("<style data-ds-style=\"desktop\">.ds { --accent: red }</style>")
        .unwrap_or_else(|| panic!("{html}"));
    assert!(ours < theirs, "the person's rules must come last: {html}");
    let probe = html
        .find("class=\"probe\"")
        .unwrap_or_else(|| panic!("{html}"));
    assert!(theirs < probe, "and before the content: {html}");
    assert_eq!(html.matches("data-ds-style").count(), 1, "{html}");
}

#[test]
fn the_user_sheet_follows_a_host_injected_sheet_too() {
    let html = html(".a{}", Inject::Host);
    assert!(
        html.contains("<style data-ds-style=\"desktop\">.a{}</style>"),
        "{html}"
    );
    assert!(!html.contains("== tokens =="), "{html}");
}

#[test]
fn an_empty_style_draws_no_element() {
    for text in ["", "  \n\t"] {
        let html = html(text, Inject::Inline);
        assert!(!html.contains("data-ds-style"), "{text:?}: {html}");
    }
}

#[test]
fn a_changed_signal_redraws_the_text_and_emptying_it_removes_the_element() {
    let (mut vdom, handle) = dom(".a{}", Inject::Host);
    assert!(dioxus_ssr::render(&vdom).contains(">.a{}</style>"));
    let mut style = handle
        .0
        .get()
        .unwrap_or_else(|| panic!("the root never ran"));
    for (text, present) in [(".b{}", true), ("", false)] {
        vdom.in_scope(ScopeId::ROOT, || {
            style.set(Styles {
                desktop: UserStyle(text.to_owned()),
                ..Styles::default()
            })
        });
        vdom.render_immediate(&mut dioxus::core::NoOpMutations);
        let html = dioxus_ssr::render(&vdom);
        assert_eq!(html.contains("data-ds-style"), present, "{text:?}: {html}");
        assert!(!html.contains(".a{}"), "{html}");
        if present {
            assert!(html.contains(">.b{}</style>"), "{html}");
        }
    }
}

#[allow(non_snake_case)]
fn Named() -> Element {
    rsx! {
        Ds { appearance: Appearance::default(), material: Material::Popover, surface: Some("bar-menu") }
    }
}

#[test]
fn a_named_root_carries_data_surface_and_an_unnamed_one_does_not() {
    let mut vdom = VirtualDom::new(Named);
    vdom.rebuild_in_place();
    let named = dioxus_ssr::render(&vdom);
    assert!(named.contains("data-surface=\"bar-menu\""), "{named}");
    assert!(!html("", Inject::Host).contains("data-surface"));
}

#[allow(non_snake_case)]
fn ThreeSheets() -> Element {
    let styles = use_signal(|| Styles {
        look: LookStyle(".look{}".to_owned()),
        desktop: UserStyle(".desktop{}".to_owned()),
        app: UserStyle(".app{}".to_owned()),
        ..Styles::default()
    });
    rsx! {
        Ds {
            appearance: Appearance::default(),
            material: Material::Popover,
            stylesheet: Inject::Host,
            styles,
            div { class: "probe" }
        }
    }
}

#[test]
fn the_look_the_desktop_sheet_and_the_app_sheet_come_in_that_order_before_the_content() {
    let mut vdom = VirtualDom::new(ThreeSheets);
    vdom.rebuild_in_place();
    let html = dioxus_ssr::render(&vdom);
    let at = |needle: &str| {
        html.find(needle)
            .unwrap_or_else(|| panic!("{needle}: {html}"))
    };
    let look = at("data-ds-style=\"look\"");
    let desktop = at("data-ds-style=\"desktop\"");
    let app = at("data-ds-style=\"app\"");
    assert!(look < desktop && desktop < app, "{html}");
    assert!(app < at("class=\"probe\""), "{html}");
    // The look is the only layered one, and it opens with the statement that orders the layers.
    assert!(
        html.contains("@layer ds, app, look;\n@layer look {\n.look{}\n}"),
        "{html}"
    );
    assert!(html.contains(">.desktop{}</style>"), "{html}");
    assert!(html.contains(">.app{}</style>"), "{html}");
}

#[test]
fn use_my_styles_off_draws_none_of_the_three() {
    let (vdom, _) = dom_with(".a{}", Inject::Host, CustomStyles::Off);
    let html = dioxus_ssr::render(&vdom);
    assert!(!html.contains("data-ds-style"), "{html}");
    assert!(!html.contains(".a{}"), "{html}");
}

#[allow(non_snake_case)]
fn Notice() -> Element {
    rsx! {
        Ds { appearance: Appearance::default(), material: Material::Popover, stylesheet: Inject::Host,
            StylesOffNotice { program: "Mail", onturnon: |_| {} }
        }
    }
}

#[test]
fn the_notice_names_the_program_and_offers_turn_on_again() {
    let mut vdom = VirtualDom::new(Notice);
    vdom.rebuild_in_place();
    let html = dioxus_ssr::render(&vdom);
    assert!(
        html.contains("Your styles were turned off because Mail could not start with them."),
        "{html}"
    );
    assert!(html.contains("Turn On Again"), "{html}");
    assert!(html.contains("ds-inline-banner"), "{html}");
}
