//! The mail-app states (controls and tiles) and 3 (the muted avatar): each new prop or variant
//! rendered through
//! dioxus-ssr and compared with a golden under its component's directory, so the controls' and
//! lists' class scans cover them too. Every state here is additive; the goldens of the states
//! that existed before are in `components_controls.rs` and `components_lists.rs`.
//!
//! `DS_BLESS=1 cargo test -p ds --test button_field_tile_states_ssr` rewrites these goldens.

#[path = "button_field_tile_states/cases.rs"]
mod cases;
#[path = "support/golden.rs"]
mod golden;
#[path = "support/scoped.rs"]
mod scoped;

use cases::CASES;
use dioxus::prelude::*;
use scoped::Scoped;

#[derive(Props, Clone)]
struct HostProps {
    make: fn() -> Element,
}

/// Never equal: the host renders once, and function addresses are not comparable anyway.
impl PartialEq for HostProps {
    fn eq(&self, _: &Self) -> bool {
        false
    }
}

/// Renders a case inside a scope, so its handlers have a runtime to attach to and a component
/// that reads the scope (a text field) draws instead of panicking, which dioxus-ssr renders as
/// nothing: the four text-field goldens were once blessed as one blank line that way.
fn host(props: HostProps) -> Element {
    let case = (props.make)();
    rsx! { Scoped { {case} } }
}

fn render(make: fn() -> Element) -> String {
    let mut dom = VirtualDom::new_with_props(host, HostProps { make });
    dom.rebuild_in_place();
    dioxus_ssr::render(&dom)
}

#[test]
fn every_button_field_and_tile_state_matches_its_golden() {
    let failures: Vec<String> = CASES
        .iter()
        .filter_map(|case| golden::check(case.golden, &render(case.make)).err())
        .collect();
    assert!(failures.is_empty(), "{}", failures.join("\n"));
}

/// A field that failed to draw renders as nothing, and a blessed golden of nothing passes
/// forever: every text-field golden must hold the field.
#[test]
fn every_text_field_golden_draws_the_field() {
    let blank: Vec<String> = golden::all_in("controls/text_field")
        .into_iter()
        .filter(|(_, html)| !html.contains("class=\"ds-text-field"))
        .map(|(name, _)| name)
        .collect();
    assert!(blank.is_empty(), "goldens with no text field: {blank:?}");
}
