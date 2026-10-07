//! The mail-app states (controls): each new prop or variant rendered through dioxus-ssr
//! and compared with a golden under its component's directory, so the controls' and lists'
//! class scans cover them too. Every state here is additive; the goldens of the states that
//! existed before are in `components_controls.rs`, `components_lists.rs` and
//! `button_field_tile_states_ssr.rs`.
//!
//! `DS_BLESS=1 cargo test -p ds --test it provider_and_button_face_states_ssr` rewrites these goldens.

#[path = "provider_and_button_face_states/cases.rs"]
mod cases;
use crate::support::golden;
use crate::support::scoped;

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
fn every_provider_and_button_face_state_matches_its_golden() {
    let failures: Vec<String> = CASES
        .iter()
        .filter_map(|case| golden::check(case.golden, &render(case.make)).err())
        .collect();
    assert!(failures.is_empty(), "{}", failures.join("\n"));
}
