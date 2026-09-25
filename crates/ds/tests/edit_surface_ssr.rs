//! The edit surface's markup (FINDINGS "Edit surface"), rendered through dioxus-ssr and compared
//! with goldens under `controls/edit_surface/`, so the controls' class scan covers it. The
//! surface's behaviour needs a document and is proved in `ds-native/tests/native_edit.rs`.
//!
//! `DS_BLESS=1 cargo test -p ds --test edit_surface_ssr` rewrites these goldens.

#[path = "support/golden.rs"]
mod golden;

use dioxus::prelude::*;
use ds::{EDIT_KIND_ATTR, EDIT_NODE_ATTR, EditKind, EditSurface};

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

/// Renders a case inside a scope, so its handlers have a runtime to attach to.
fn host(props: HostProps) -> Element {
    (props.make)()
}

fn render(make: fn() -> Element) -> String {
    let mut dom = VirtualDom::new_with_props(host, HostProps { make });
    dom.rebuild_in_place();
    dioxus_ssr::render(&dom)
}

/// A golden's name and how to render it.
type Case = (&'static str, fn() -> Element);

const CASES: &[Case] = &[
    (
        "controls/edit_surface/two-paragraphs-and-a-chip.html",
        || {
            rsx! {
                EditSurface { id: "body", label: "Message", on_input: |_| {},
                    p { "data-edit-node": "0", "Dear Ada," }
                    p { "data-edit-node": "1",
                        "Lunch with "
                        span { "data-edit-node": "2", "data-edit-kind": EditKind::Atom.slug(), "Grace" }
                        " on Friday?"
                    }
                }
            }
        },
    ),
    ("controls/edit_surface/empty.html", || {
        rsx! { EditSurface { on_input: |_| {} } }
    }),
];

#[test]
fn every_edit_surface_state_matches_its_golden() {
    let failures: Vec<String> = CASES
        .iter()
        .filter_map(|(name, make)| golden::check(name, &render(*make)).err())
        .collect();
    assert!(failures.is_empty(), "{}", failures.join("\n"));
}

#[test]
fn the_marking_attributes_are_the_ones_the_host_reads() {
    assert_eq!(EDIT_NODE_ATTR, "data-edit-node");
    assert_eq!(EDIT_KIND_ATTR, "data-edit-kind");
}
