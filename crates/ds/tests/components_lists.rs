//! The list, sidebar and Space-frame components (design/04-COMPONENTS.md sections 8, 16, 17, 19,
//! 26-28, 32-35): every component in every state rendered through dioxus-ssr and compared with
//! a golden in `tests/snapshots/lists/<component>/<state>.html`; roster-driven lists at each
//! moment of a row's exit; the drag ghost driven by a `DragTracker`; the Space editor's field
//! plane decoded and probed; the contrast pills against `space::readout`; and every golden
//! linted as markup against the stylesheet.
//!
//! `DS_BLESS=1 cargo test -p ds --test components_lists` rewrites the goldens.

#[path = "lists/cases.rs"]
mod cases;
#[path = "controls/css_scan.rs"]
#[allow(dead_code)] // The controls' STYLES table is not used here.
mod css_scan;
#[path = "support/golden.rs"]
mod golden;
#[path = "lists/live.rs"]
mod live;
#[path = "lists/motion.rs"]
mod motion;
#[path = "lists/rows.rs"]
mod rows;
#[path = "support/scoped.rs"]
mod scoped;
#[path = "lists/strip_press.rs"]
mod strip_press;
#[path = "lists/thread_row.rs"]
mod thread_row;

use cases::{CASES, Case};
use css_scan::{classes, styles_class, token_violations};
use dioxus::prelude::*;
use ds_motion::drag::{DragPhase, DragTracker, use_drag};
use ds_motion::presence::Exit;
use rows::ROW_CASES;
use strip_press::STRIP_PRESS_CASES;
use thread_row::THREAD_ROW_CASES;

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

fn golden_name(case: &Case) -> String {
    format!("lists/{}/{}.html", case.component, case.state)
}

#[test]
fn every_list_component_matches_its_golden() {
    let failures: Vec<String> = CASES
        .iter()
        .chain(ROW_CASES)
        .chain(THREAD_ROW_CASES)
        .chain(STRIP_PRESS_CASES)
        .filter_map(|case| golden::check(&golden_name(case), &render(case.make)).err())
        .collect();
    assert!(failures.is_empty(), "{}", failures.join("\n"));
}

/// The stylesheets that style one component's markup: its own, then those of the components it
/// renders.
fn sheets(component: &str) -> Vec<&'static str> {
    let own: &[&str] = match component {
        "thread_row" | "roster" => &[
            "thread_row",
            "row",
            "list",
            "hover_strip",
            "button",
            "provider_mark",
            "chip",
            "text_runs",
        ],
        "row" => &[
            "row",
            "disclosure",
            "badge",
            "avatar",
            "button",
            "icon_view",
            "toggle",
            "progress",
            "key_equivalent",
            "status_glyph",
            "text_field",
            "text_runs",
        ],
        "list" => &[
            "list",
            "row",
            "thread_row",
            "hover_strip",
            "button",
            "provider_mark",
            "chip",
            "section_header",
            "disclosure",
            "text_runs",
        ],
        "pin_tile" => &["pin_tile", "avatar", "provider_mark", "badge"],
        "hover_strip" => &["hover_strip", "icon_button"],
        "drag" => &["drag_ghost"],
        other => return sheet(other).into_iter().collect(),
    };
    own.iter().filter_map(|name| sheet(name)).collect()
}

/// The stylesheet of the component called `name`.
fn sheet(name: &str) -> Option<&'static str> {
    ds::component_sheets()
        .iter()
        .find(|(n, _)| *n == name)
        .map(|(_, css)| *css)
}

/// Classes every component may use that its own sheets do not define: `Glyph`'s `ds-ic` (sized
/// by its attributes, spike S6) and the `.ds-truncate` utility.
const SHARED: &[&str] = &["ds-ic", "ds-truncate"];

/// A class a component here draws that another component's sheet styles: the picker's Space dot
/// is the `space_editor` sheet's (FINDINGS "The picker's Space dot").
const STYLED_ABOVE: &[&str] = &["ds-space-dot"];

#[test]
fn every_class_in_a_golden_is_styled_by_its_component() {
    let goldens = golden::all_in("lists");
    assert!(
        goldens.len()
            >= CASES.len() + ROW_CASES.len() + THREAD_ROW_CASES.len() + STRIP_PRESS_CASES.len(),
        "only {} goldens",
        goldens.len()
    );
    let mut failures = Vec::new();
    for (name, html) in &goldens {
        let component = name.split('/').nth(1).unwrap_or_default();
        let css = sheets(component);
        if css.is_empty() {
            failures.push(format!("{name}: no stylesheet listed for {component}"));
            continue;
        }
        for class in classes(html) {
            if SHARED.contains(&class) || STYLED_ABOVE.contains(&class) || !class.starts_with("ds-")
            {
                continue;
            }
            if !css.iter().any(|sheet| styles_class(sheet, class)) {
                failures.push(format!(
                    "{name}: .{class} is in no stylesheet of {component}"
                ));
            }
        }
    }
    assert!(failures.is_empty(), "{}", failures.join("\n"));
}

/// The stylesheets of the components listed here.
const OWN: &[&str] = &[
    "thread_row",
    "row",
    "list",
    "disclosure",
    "hover_strip",
    "command_pill",
    "pin_tile",
    "provider_mark",
    "drag_ghost",
    "edge_peek",
    "text_runs",
];

#[test]
fn list_stylesheets_use_tokens_only() {
    let mut failures = Vec::new();
    let all = ds::component_sheets();
    for name in OWN {
        let Some((_, css)) = all.iter().find(|(n, _)| n == name) else {
            failures.push(format!("{name}: not in the component list"));
            continue;
        };
        if css.trim().starts_with("/*") && css.lines().count() < 3 {
            failures.push(format!("{name}.css is still the stub"));
        }
        for problem in token_violations(css) {
            failures.push(format!("{name}.css: {problem}"));
        }
    }
    assert!(failures.is_empty(), "{}", failures.join("\n"));
}

/// What these components write inline that a consumer may not: colours computed per account,
/// provider or Space. Each is data the palette or the app computes, not a theme colour.
const MARKUP_EXCEPTIONS: &[ds_lint::Exception] = {
    use ds_lint::{Exception, Rule};
    &[
        Exception {
            rule: Rule::HexColour,
            selector: "span.ds-avatar",
            reason: "the person hue and account colour are computed per face (O-7); the letter is #fff (O-3)",
        },
        Exception {
            rule: Rule::HexColour,
            selector: "span.ds-provider",
            reason: "a provider's identity colour enters as --pc (section 28, O-3)",
        },
        Exception {
            rule: Rule::HexColour,
            selector: "button.ds-space-dot",
            reason: "a Space dot is painted with its FrameVars gradient",
        },
    ]
};

/// Coherence rule 2 on these components' output: every golden uses only classes the stylesheet
/// styles, no hand-written SVG or form control, and no colour inline beyond the exceptions.
#[test]
fn every_list_golden_lints_clean() {
    use ds_lint::{LintConfig, markup};
    let config = LintConfig {
        exceptions: MARKUP_EXCEPTIONS,
        ..LintConfig::new(&ds::kits())
    };
    let goldens = golden::all_in("lists");
    assert!(
        goldens.len()
            >= CASES.len() + ROW_CASES.len() + THREAD_ROW_CASES.len() + STRIP_PRESS_CASES.len(),
        "only {} goldens",
        goldens.len()
    );
    // The picker's Space dot is styled by a sheet above `ds`; a stand-in rule defines its class.
    let css = format!("{}\n.ds-space-dot{{}}", ds::stylesheet());
    let failures: Vec<String> = goldens
        .iter()
        .flat_map(|(name, html)| {
            markup(html, &css, &config)
                .into_iter()
                .map(move |offence| format!("{name}: {:?} {}", offence.rule, offence.text))
        })
        .collect();
    assert!(failures.is_empty(), "{}", failures.join("\n"));
}
