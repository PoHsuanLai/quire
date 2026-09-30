//! The structure components (design/30 sections 2.1 to 2.7): every component in every state
//! rendered through dioxus-ssr and compared with a golden in
//! `tests/snapshots/structure/<component>/<state>.html`, each linted against the stylesheet, and
//! every `ds-` class in one styled by its own component's sheet.
//!
//! `DS_BLESS=1 cargo test -p ds --test components_structure` rewrites the goldens.

#[path = "structure/cases.rs"]
mod cases;
#[path = "support/golden.rs"]
mod golden;

use cases::{CASES, Case};
use dioxus::prelude::*;
use ds_lint::{LintConfig, markup};

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

fn render(case: &Case) -> String {
    let mut dom = VirtualDom::new_with_props(host, HostProps { make: case.make });
    dom.rebuild_in_place();
    dioxus_ssr::render(&dom)
}

fn name(case: &Case) -> String {
    format!("structure/{}/{}.html", case.component, case.state)
}

/// The classes each component's own sheet styles, by the prefix its markup carries.
const OWN_SHEETS: &[(&str, &str, &str)] = &[
    (
        "stepper",
        "ds-stepper",
        include_str!("../src/components/fields/stepper/stepper.css"),
    ),
    (
        "table",
        "ds-table",
        include_str!("../src/components/lists/table/table.css"),
    ),
    (
        "toolbar",
        "ds-toolbar",
        include_str!("../src/components/chrome/toolbar/toolbar.css"),
    ),
    (
        "split_view",
        "ds-split",
        include_str!("../src/components/chrome/split_view/split_view.css"),
    ),
    (
        "sidebar",
        "ds-sidebar",
        include_str!("../src/components/chrome/sidebar.css"),
    ),
    (
        "tab_view",
        "ds-tab-view",
        include_str!("../src/components/chrome/tab_view.css"),
    ),
    (
        "field_row",
        "ds-field-",
        include_str!("../src/components/fields/field_row.css"),
    ),
    (
        "drag_ghost",
        "ds-drag-ghost",
        include_str!("../src/components/overlays/drag_ghost.css"),
    ),
    (
        "titlebar",
        "ds-titlebar",
        include_str!("../src/components/chrome/window_frame.css"),
    ),
];

#[test]
fn every_structure_component_matches_its_golden() {
    let failures: Vec<String> = CASES
        .iter()
        .filter_map(|case| golden::check(&name(case), &render(case)).err())
        .collect();
    assert!(failures.is_empty(), "{}", failures.join("\n"));
}

#[test]
fn every_structure_golden_lints_clean() {
    let config = LintConfig::new(&ds::kits());
    let css = ds::stylesheet();
    let failures: Vec<String> = CASES
        .iter()
        .flat_map(|case| {
            let html = render(case);
            markup(&html, css, &config)
                .into_iter()
                .map(move |offence| format!("{}: {:?} {}", name(case), offence.rule, offence.text))
        })
        .collect();
    assert!(failures.is_empty(), "{}", failures.join("\n"));
}

/// The `ds-*` classes in `html`, each once.
fn classes(html: &str) -> Vec<String> {
    let mut found: Vec<String> = html
        .split("class=\"")
        .skip(1)
        .filter_map(|rest| rest.split('"').next())
        .flat_map(str::split_whitespace)
        .filter(|class| class.starts_with("ds-"))
        .map(str::to_owned)
        .collect();
    found.sort();
    found.dedup();
    found
}

#[test]
fn every_class_a_structure_component_writes_is_styled_by_its_own_sheet() {
    let mut failures = Vec::new();
    for case in CASES {
        let Some((_, prefix, css)) = OWN_SHEETS
            .iter()
            .find(|(component, ..)| *component == case.component)
        else {
            failures.push(format!("{}: no sheet listed", case.component));
            continue;
        };
        for class in classes(&render(case))
            .into_iter()
            .filter(|class| class.starts_with(prefix))
        {
            if !css.contains(&format!(".{class}")) {
                failures.push(format!(
                    "{}: .{class} is not in {}'s sheet",
                    name(case),
                    case.component
                ));
            }
        }
    }
    assert!(failures.is_empty(), "{}", failures.join("\n"));
}

#[test]
fn a_tab_view_holds_six_tabs_at_most() {
    let case = CASES
        .iter()
        .find(|case| case.component == "tab_view" && case.state == "six-at-most");
    let html = case.map(render).unwrap_or_default();
    assert_eq!(html.matches("ds-segmented-segment").count(), 6);
}

#[test]
fn only_a_drag_of_several_things_carries_a_badge() {
    for (state, badges) in [("one", 0), ("many", 1)] {
        let case = CASES
            .iter()
            .find(|case| case.component == "drag_ghost" && case.state == state);
        let html = case.map(render).unwrap_or_default();
        assert_eq!(
            html.matches("ds-drag-ghost-count").count(),
            badges,
            "{state}"
        );
    }
}
