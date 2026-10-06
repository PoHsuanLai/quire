//! Form, FormSection, IconTile and PaneStack in every state rendered through dioxus-ssr and compared with
//! a golden in `tests/snapshots/forms/<component>/<state>.html`; every class in a golden is
//! styled by the components' sheets; every golden lints clean as markup.
//!
//! `DS_BLESS=1 cargo test -p ds --test forms_ssr` rewrites the goldens.

#[path = "forms/cases.rs"]
mod cases;
#[path = "controls/css_scan.rs"]
#[allow(dead_code)] // The controls' STYLES table is not used here.
mod css_scan;
#[path = "support/golden.rs"]
mod golden;
#[path = "support/scoped.rs"]
mod scoped;

use cases::{CASES, Case};
use css_scan::{classes, styles_class, token_violations};
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

fn render(case: &Case) -> String {
    let mut dom = VirtualDom::new_with_props(host, HostProps { make: case.make });
    dom.rebuild_in_place();
    dioxus_ssr::render(&dom)
}

fn golden_name(case: &Case) -> String {
    format!("forms/{}/{}.html", case.component, case.state)
}

/// The sheets of the forms components and of what a golden draws inside them.
const SHEETS: &[&str] = &[
    "form",
    "form_section",
    "icon_tile",
    "pane_stack",
    "button",
    "list",
    "row",
    "avatar",
];

fn sheet(name: &str) -> &'static str {
    ds::component_sheets()
        .iter()
        .find(|(n, _)| *n == name)
        .map_or("", |(_, css)| *css)
}

#[test]
fn every_forms_state_matches_its_golden() {
    let failures: Vec<String> = CASES
        .iter()
        .filter_map(|case| golden::check(&golden_name(case), &render(case)).err())
        .collect();
    assert!(failures.is_empty(), "{}", failures.join("\n"));
}

#[test]
fn every_class_in_a_forms_golden_is_styled() {
    let mut failures = Vec::new();
    for case in CASES {
        let html = render(case);
        for class in classes(&html) {
            let styled = class == "ds-ic"
                || class == "ds-truncate"
                || SHEETS.iter().any(|n| styles_class(sheet(n), class));
            if class.starts_with("ds-") && !styled {
                failures.push(format!(
                    "{}: .{class} is styled by no sheet",
                    golden_name(case)
                ));
            }
        }
    }
    assert!(failures.is_empty(), "{}", failures.join("\n"));
}

#[test]
fn the_forms_sheets_use_tokens_only() {
    let failures: Vec<String> = ["form", "form_section", "icon_tile", "pane_stack"]
        .iter()
        .flat_map(|name| {
            token_violations(sheet(name))
                .into_iter()
                .map(move |problem| format!("{name}.css: {problem}"))
        })
        .collect();
    assert!(failures.is_empty(), "{}", failures.join("\n"));
}

/// The tile's colour is data the caller passes (an app's or a pane's identity colour), written
/// inline as `--tile-bg`; an avatar's is computed per face.
const MARKUP_EXCEPTIONS: &[ds_lint::Exception] = {
    use ds_lint::{Exception, Rule};
    &[
        Exception {
            rule: Rule::HexColour,
            selector: "span.ds-icon-tile",
            reason: "a tile's ground is the caller's identity colour, entering as --tile-bg",
        },
        Exception {
            rule: Rule::HexColour,
            selector: "span.ds-avatar",
            reason: "the person hue is computed per face (O-7)",
        },
    ]
};

#[test]
fn every_forms_golden_lints_clean() {
    use ds_lint::{LintConfig, markup};
    let config = LintConfig {
        exceptions: MARKUP_EXCEPTIONS,
        ..LintConfig::new(&ds::kits())
    };
    let css = ds::stylesheet();
    let failures: Vec<String> = CASES
        .iter()
        .flat_map(|case| {
            markup(&render(case), css, &config)
                .into_iter()
                .map(move |offence| {
                    format!("{}: {:?} {}", golden_name(case), offence.rule, offence.text)
                })
        })
        .collect();
    assert!(failures.is_empty(), "{}", failures.join("\n"));
}

/// Reduced motion drops the slide and keeps the fade: the sheet's own rule, since the spring that
/// drives `--pane-q` is the same at every level.
#[test]
fn the_pane_stack_does_not_slide_under_reduced_motion() {
    let rule = ".ds[*|data-motion=reduced] .ds-pane-stack-page{ transform:none; }";
    assert!(sheet("pane_stack").contains(rule), "missing {rule}");
}
