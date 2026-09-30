//! The person's stylesheet on the `Ds` root: `<style data-ds-user>` after the design-system sheet
//! (so a user rule wins by order, never by `!important`), redrawn when the signal changes, and
//! absent when the style is empty.

use dioxus::core::VirtualDom;
use dioxus::prelude::*;
use ds::{Appearance, Ds, Inject, Material};
use ds_style::kit::UserStyle;

use std::cell::Cell;
use std::rc::Rc;

/// The signal the root under test reads, handed out so a test can change it.
#[derive(Clone, Default)]
struct Handle(Rc<Cell<Option<Signal<UserStyle>>>>);

impl PartialEq for Handle {
    fn eq(&self, other: &Self) -> bool {
        Rc::ptr_eq(&self.0, &other.0)
    }
}

#[derive(Props, Clone, PartialEq)]
struct Setup {
    text: String,
    stylesheet: Inject,
    handle: Handle,
}

#[allow(non_snake_case)]
fn Root(setup: Setup) -> Element {
    let style = use_signal(|| UserStyle(setup.text.clone()));
    setup.handle.0.set(Some(style));
    rsx! {
        Ds {
            appearance: Appearance::default(),
            material: Material::Popover,
            stylesheet: setup.stylesheet,
            user_style: style,
            div { class: "probe" }
        }
    }
}

fn dom(text: &str, stylesheet: Inject) -> (VirtualDom, Handle) {
    let handle = Handle::default();
    let setup = Setup {
        text: text.to_owned(),
        stylesheet,
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
        .find("<style data-ds-user=\"\">.ds { --accent: red }</style>")
        .unwrap_or_else(|| panic!("{html}"));
    assert!(ours < theirs, "the person's rules must come last: {html}");
    let probe = html
        .find("class=\"probe\"")
        .unwrap_or_else(|| panic!("{html}"));
    assert!(theirs < probe, "and before the content: {html}");
    assert_eq!(html.matches("data-ds-user").count(), 1, "{html}");
}

#[test]
fn the_user_sheet_follows_a_host_injected_sheet_too() {
    let html = html(".a{}", Inject::Host);
    assert!(
        html.contains("<style data-ds-user=\"\">.a{}</style>"),
        "{html}"
    );
    assert!(!html.contains("== tokens =="), "{html}");
}

#[test]
fn an_empty_style_draws_no_element() {
    for text in ["", "  \n\t"] {
        let html = html(text, Inject::Inline);
        assert!(!html.contains("data-ds-user"), "{text:?}: {html}");
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
        vdom.in_scope(ScopeId::ROOT, || style.set(UserStyle(text.to_owned())));
        vdom.render_immediate(&mut dioxus::core::NoOpMutations);
        let html = dioxus_ssr::render(&vdom);
        assert_eq!(html.contains("data-ds-user"), present, "{text:?}: {html}");
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
