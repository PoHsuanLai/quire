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
#[path = "lists/mailo.rs"]
mod mailo;
#[path = "lists/motion.rs"]
mod motion;
#[path = "lists/png.rs"]
mod png;
#[path = "lists/rows.rs"]
mod rows;
#[path = "lists/editor.rs"]
mod space_editor;

use cases::{CASES, Case, editor, preset_look};
use css_scan::{classes, styles_class, token_violations};
use dioxus::prelude::*;
use ds::components::vocab::Emphasis;
use ds::{
    Anim, AnimatedList, Capping, Dot, DragGhost, DragPhase, DragTracker, Exit, Grain, ListPresence,
    Point, Px, Rect, RosterState, RowPitch, Scheme, Size, SpaceLook, Verdict, derive, readout,
    swatch, use_drag,
};
use mailo::MAILO_CASES;
use rows::{ROW_CASES, Row};

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

const PLANE: &str = "data:image/png;base64,";

/// The field plane's data URI, and `html` with its payload replaced by a marker: the plane is
/// checked by decoding it (below), not by 60 kB of base64 in a golden.
fn scrub(html: &str) -> String {
    let mut out = String::new();
    let mut rest = html;
    while let Some(at) = rest.find(PLANE) {
        let payload = &rest[at + PLANE.len()..];
        let end = payload
            .find(|c: char| !(c.is_ascii_alphanumeric() || c == '+' || c == '/' || c == '='))
            .unwrap_or(payload.len());
        out.push_str(&rest[..at + PLANE.len()]);
        out.push_str(if end > 64 {
            "[field plane]"
        } else {
            &payload[..end]
        });
        rest = &payload[end..];
    }
    out.push_str(rest);
    out
}

fn golden_name(case: &Case) -> String {
    format!("lists/{}/{}.html", case.component, case.state)
}

#[test]
fn every_list_component_matches_its_golden() {
    let failures: Vec<String> = CASES
        .iter()
        .chain(ROW_CASES)
        .chain(MAILO_CASES)
        .filter_map(|case| golden::check(&golden_name(case), &scrub(&render(case.make))).err())
        .collect();
    assert!(failures.is_empty(), "{}", failures.join("\n"));
}

/// The stylesheets that style one component's markup: its own, then those of the components it
/// renders.
fn sheets(component: &str) -> Vec<&'static str> {
    let own: &[&str] = match component {
        "list_row" | "animated_list" | "roster" => &[
            "list_row",
            "animated_list",
            "hover_strip",
            "icon_button",
            "provider_mark",
            "chip",
            "text_runs",
        ],
        "account_tile" => &[
            "account_tile",
            "icon_button",
            "avatar",
            "provider_mark",
            "count",
        ],
        "sidebar_item" => &["sidebar_item", "avatar", "count"],
        "sync_halo" => &["sync_halo", "spinner", "avatar"],
        "hover_strip" => &["hover_strip", "icon_button"],
        "appearance_picker" => &[
            "appearance_picker",
            "section_header",
            "segmented",
            "space_editor",
        ],
        "space_editor" => &[
            "space_editor",
            "section_header",
            "segmented",
            "slider",
            "button",
            "chip",
            "text_input",
        ],
        "drag" => &["drag_ghost"],
        other => return sheet(other).into_iter().collect(),
    };
    own.iter().filter_map(|name| sheet(name)).collect()
}

/// The stylesheet of the component called `name`.
fn sheet(name: &str) -> Option<&'static str> {
    ds::components::CSS
        .iter()
        .find(|(n, _)| *n == name)
        .map(|(_, css)| *css)
}

/// Classes every component may use that its own sheets do not define: `Glyph`'s `ds-ic` (sized
/// by its attributes, spike S6) and the `.ds-truncate` utility.
const SHARED: &[&str] = &["ds-ic", "ds-truncate"];

#[test]
fn every_class_in_a_golden_is_styled_by_its_component() {
    let goldens = golden::all_in("lists");
    assert!(
        goldens.len() >= CASES.len() + ROW_CASES.len() + MAILO_CASES.len(),
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
            if SHARED.contains(&class) || !class.starts_with("ds-") {
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

/// This wave's twelve stylesheets.
const OWN: &[&str] = &[
    "list_row",
    "hover_strip",
    "sidebar_item",
    "animated_list",
    "appearance_picker",
    "command_pill",
    "account_tile",
    "provider_mark",
    "sync_halo",
    "drag_ghost",
    "edge_strip",
    "space_editor",
    "text_runs",
];

#[test]
fn list_stylesheets_use_tokens_only() {
    let mut failures = Vec::new();
    for name in OWN {
        let Some((_, css)) = ds::components::CSS.iter().find(|(n, _)| n == name) else {
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
#[cfg(feature = "lint")]
const MARKUP_EXCEPTIONS: &[ds::lint::Exception] = {
    use ds::lint::{Exception, Rule};
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
            selector: "div.ds-handle",
            reason: "a handle is filled with its dot's pick colour from space::palette",
        },
        Exception {
            rule: Rule::HexColour,
            selector: "i.ds-stop-disc",
            reason: "a stop's disc is its dot's pick colour from space::palette",
        },
        Exception {
            rule: Rule::HexColour,
            selector: "span.ds-space-swatch",
            reason: "the title swatch is the Space's gradient from space::palette",
        },
        Exception {
            rule: Rule::HexColour,
            selector: "button.ds-preset",
            reason: "a preset is painted with its own derived gradient",
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
#[cfg(feature = "lint")]
#[test]
fn every_list_golden_lints_clean() {
    use ds::lint::{LintConfig, markup};
    let config = LintConfig {
        exceptions: MARKUP_EXCEPTIONS,
        ..LintConfig::default()
    };
    let goldens = golden::all_in("lists");
    assert!(
        goldens.len() >= CASES.len() + ROW_CASES.len() + MAILO_CASES.len(),
        "only {} goldens",
        goldens.len()
    );
    let failures: Vec<String> = goldens
        .iter()
        .flat_map(|(name, html)| {
            markup(html, ds::stylesheet(), &config)
                .into_iter()
                .map(move |offence| format!("{name}: {:?} {}", offence.rule, offence.text))
        })
        .collect();
    assert!(failures.is_empty(), "{}", failures.join("\n"));
    // The plane's payload is scrubbed from the goldens; the real markup must lint clean too.
    let real = render(|| editor(preset_look(0, Grain(35)), Scheme::Light, 0));
    let offences = markup(&real, ds::stylesheet(), &config);
    assert!(offences.is_empty(), "{offences:?}");
}
