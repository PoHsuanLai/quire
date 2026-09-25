//! The mailo gaps 7 states: a tree item renamed in place (its editing slot in the label's place)
//! rendered through dioxus-ssr and compared with a golden under the lists' directory, so the
//! lists' class scans cover the new class too. The trailing slot's `data-slot="trailing"` shows
//! in these and in the mailo gaps 6 tree goldens.
//!
//! `DS_BLESS=1 cargo test -p ds --test mailo_gaps7_ssr` rewrites these goldens.

#[path = "mailo_gaps7/cases.rs"]
mod cases;
#[path = "support/golden.rs"]
mod golden;

use cases::CASES;
use dioxus::prelude::*;

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

#[test]
fn every_mailo_gap_7_state_matches_its_golden() {
    let failures: Vec<String> = CASES
        .iter()
        .filter_map(|case| golden::check(case.golden, &render(case.make)).err())
        .collect();
    assert!(failures.is_empty(), "{}", failures.join("\n"));
}
