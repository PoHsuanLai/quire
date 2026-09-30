//! The Space editor and the Space dot: every state rendered through dioxus-ssr and compared with
//! a golden in `tests/snapshots/space_editor/<state>.html`; the field plane decoded and
//! probed, the handles placed from the dots, the contrast pills against `space::readout`; and
//! every golden's classes styled, its sheet on tokens, and its markup linted against the
//! stylesheet the design system draws with.
//!
//! `DS_BLESS=1 cargo test -p ds --test space_editor_ssr` rewrites the goldens.

#[path = "controls/css_scan.rs"]
#[allow(dead_code)] // The controls' STYLES table is not used here.
mod css_scan;
#[path = "lists/editor.rs"]
mod field_plane;
#[path = "support/golden.rs"]
mod golden;
#[path = "lists/png.rs"]
mod png;
#[path = "lists/space_editor_rows.rs"]
mod space_editor_rows;

use css_scan::{classes, styles_class, token_violations};
use dioxus::prelude::*;
use ds::components::app::space_editor::DotIndex;
use ds::prelude::*;
use ds_core::colour::contrast::Verdict;
use ds_style::space::look::CardAccent;
use ds_style::space::palette::readout::readout;
use ds_style::space::palette::{Capping, Dot, derive, swatch};
use ds_style::space::presets::PRESETS;

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

/// `html` with each field plane's data URI payload replaced by a marker: the plane is checked by
/// decoding it (in `field_plane`), not by 60 kB of base64 in a golden.
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

/// Preset `index` as a Space that lends its hue to the card.
fn preset_look(index: usize, grain: Grain) -> SpaceLook {
    SpaceLook {
        dots: PRESETS[index].dots.to_vec(),
        grain,
        theme: Theme::System,
        card_accent: CardAccent::SpaceHue,
    }
}

fn editor(look: SpaceLook, scheme: Scheme, active: u8) -> Element {
    rsx! { SpaceEditor { look, scheme, active_dot: DotIndex(active), onchange: |_| {} } }
}

/// One state and the golden it must match (relative to `tests/snapshots`).
struct Case {
    state: &'static str,
    make: fn() -> Element,
}

const CASES: &[Case] = &[
    // SpaceEditor: the two sample Spaces in each scheme and a one-dot Space on the chosen accent.
    Case {
        state: "work-light",
        make: || editor(preset_look(0, Grain(35)), Scheme::Light, 0),
    },
    Case {
        state: "work-dark",
        make: || editor(preset_look(0, Grain(35)), Scheme::Dark, 1),
    },
    Case {
        state: "home-light",
        make: || editor(preset_look(1, Grain(55)), Scheme::Light, 2),
    },
    Case {
        state: "one-dot-chosen",
        make: || {
            editor(
                SpaceLook {
                    card_accent: CardAccent::Chosen,
                    ..preset_look(2, Grain(40))
                },
                Scheme::Light,
                0,
            )
        },
    },
    Case {
        state: "named",
        make: || rsx! { SpaceEditor { look: preset_look(0, Grain(35)), scheme: Scheme::Light, active_dot: DotIndex(0), name: "Work".to_string(), onchange: |_| {}, on_active_dot: |_| {} } },
    },
    // SpaceDot.
    Case {
        state: "space-dot-current",
        make: || rsx! { SpaceDot { name: "Work", frame: ds_style::space::frame_vars::FrameVars::of(&preset_look(0, Grain(35)), Scheme::Light), selection: Selection::Selected, shortcut: Shortcut(vec![ShortcutKey::Ctrl, ShortcutKey::Char('1')]), onclick: |_| {} } },
    },
    Case {
        state: "space-dot-elsewhere",
        make: || rsx! { SpaceDot { name: "Home", frame: ds_style::space::frame_vars::FrameVars::of(&preset_look(1, Grain(55)), Scheme::Dark), selection: Selection::Unselected, shortcut: Shortcut(vec![ShortcutKey::Ctrl, ShortcutKey::Char('2')]), onclick: |_| {} } },
    },
];

#[test]
fn every_space_editor_state_matches_its_golden() {
    let failures: Vec<String> = CASES
        .iter()
        .filter_map(|case| {
            let name = format!("space_editor/{}.html", case.state);
            golden::check(&name, &scrub(&render(case.make))).err()
        })
        .chain(
            space_editor_rows::CASES
                .iter()
                .filter_map(|case| golden::check(case.golden, &render(case.make)).err()),
        )
        .collect();
    assert!(failures.is_empty(), "{}", failures.join("\n"));
}

/// The stylesheets that style the editor's markup: its own, then those of the components it
/// renders.
fn sheets() -> Vec<&'static str> {
    let all = ds::component_sheets();
    [
        "space_editor",
        "section_header",
        "segmented",
        "slider",
        "button",
        "chip",
        "text_field",
        "label",
        "progress",
    ]
    .iter()
    .filter_map(|name| all.iter().find(|(n, _)| n == name).map(|(_, css)| *css))
    .collect()
}

/// Classes every component may use that its own sheets do not define: `Glyph`'s `ds-ic` (sized
/// by its attributes, spike S6) and the `.ds-truncate` utility.
const SHARED: &[&str] = &["ds-ic", "ds-truncate"];

#[test]
fn every_class_in_a_golden_is_styled_by_the_editors_sheets() {
    let goldens = golden::all_in("space_editor");
    assert!(
        goldens.len() >= CASES.len() + space_editor_rows::CASES.len(),
        "only {} goldens",
        goldens.len()
    );
    let css = sheets();
    assert_eq!(css.len(), 9, "a sheet the editor draws with is missing");
    let mut failures = Vec::new();
    for (name, html) in &goldens {
        for class in classes(html) {
            if SHARED.contains(&class) || !class.starts_with("ds-") {
                continue;
            }
            if !css.iter().any(|sheet| styles_class(sheet, class)) {
                failures.push(format!(
                    "{name}: .{class} is in no stylesheet of the editor"
                ));
            }
        }
    }
    assert!(failures.is_empty(), "{}", failures.join("\n"));
}

#[test]
fn the_space_editor_stylesheet_uses_tokens_only() {
    let all = ds::component_sheets();
    let (_, css) = all
        .iter()
        .find(|(name, _)| *name == "space_editor")
        .expect("the space_editor sheet is registered");
    let problems: Vec<String> = token_violations(css)
        .into_iter()
        .map(|problem| format!("space_editor.css: {problem}"))
        .collect();
    assert!(problems.is_empty(), "{}", problems.join("\n"));
}

/// What the editor writes inline that a consumer may not: colours computed per Space. Each is
/// data the palette computes, not a theme colour.
const MARKUP_EXCEPTIONS: &[ds_lint::Exception] = {
    use ds_lint::{Exception, Rule};
    &[
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

/// Coherence rule 2 on the editor's output: every golden uses only classes the stylesheet
/// styles, no hand-written SVG or form control, and no colour inline beyond the exceptions.
#[test]
fn every_space_editor_golden_lints_clean() {
    use ds_lint::{LintConfig, markup};
    let config = LintConfig {
        exceptions: MARKUP_EXCEPTIONS,
        ..LintConfig::new(&ds::kits())
    };
    let goldens = golden::all_in("space_editor");
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
