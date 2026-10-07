//! The alert as markup: every state matches its golden under
//! `tests/snapshots/alert/`, lints clean and uses only `ds-` classes the stylesheet styles; the
//! markup says which button is the default (Primary), how a destructive action reads and how the
//! footer lies for one, two and three buttons.
//!
//! `DS_BLESS=1 cargo test -p ds --features lint --test it alert_ssr` rewrites the goldens.

use crate::support::golden;

use dioxus::core::NoOpMutations;
use dioxus::prelude::*;
use ds::assembly::ds::Inject;
use ds::components::overlays::alert_model::{AlertButton, AlertRole, AlertStyle, Suppression};
use ds::prelude::*;
use ds_lint::{LintConfig, markup};

const TITLE: &str = "Turn Bluetooth off?";
const MESSAGE: &str = "Bluetooth devices such as keyboards and mice will be disconnected.";

fn root(theme: Theme, body: Element) -> Element {
    rsx! {
        Ds { appearance: Appearance { theme, ..Appearance::default() }, material: Material::Sheet,
            extent: RootExtent::Viewport, stylesheet: Inject::Host,
            {body}
        }
    }
}

fn button(label: &str, role: AlertRole) -> AlertButton {
    AlertButton::new(label, role, EventHandler::new(|()| {}))
}

fn bluetooth(action: AlertRole, flow: Flow, theme: Theme) -> Element {
    let buttons = vec![
        button("Turn Off", action),
        button("Cancel", AlertRole::Cancel),
    ];
    let alert = rsx! {
        Alert { title: TITLE, message: Some(TextLine::from(MESSAGE)), buttons, flow }
    };
    match flow {
        Flow::Floating => root(theme, alert),
        // In place: inside a positioned popover body of its own.
        Flow::Inline => root(
            theme,
            rsx! {
                div { style: "position:relative;width:320px;height:360px", {alert} }
            },
        ),
    }
}

fn erase() -> Element {
    root(
        Theme::Light,
        rsx! {
            Alert { title: "Erase this disk?", message: Some(TextLine::from("Everything on it will be lost.")),
                buttons: vec![button("Erase", AlertRole::Destructive), button("Cancel", AlertRole::Cancel)],
                style: AlertStyle::Critical, icon: Some(IconSource::Glyph(Icon::Trash)) }
        },
    )
}

fn hero() -> Element {
    root(
        Theme::Light,
        rsx! {
            Alert { title: TITLE, message: Some(TextLine::from(MESSAGE)),
                buttons: vec![button("Turn Off", AlertRole::Normal), button("Cancel", AlertRole::Cancel)],
                icon: Some(IconSource::Glyph(Icon::Bluetooth)) }
        },
    )
}

fn notice() -> Element {
    root(
        Theme::Light,
        rsx! {
            Alert { title: "Bluetooth is off", buttons: vec![button("OK", AlertRole::Normal)] }
        },
    )
}

fn extras() -> Element {
    root(
        Theme::Light,
        rsx! {
            Alert { title: "Delete the message?", buttons: vec![button("Delete", AlertRole::Destructive), button("Cancel", AlertRole::Cancel)],
                suppression: Some(Suppression { label: "Do not ask again".to_owned(), value: Check::Off, onchange: EventHandler::new(|_| {}) }),
                help: Some(EventHandler::new(|()| {})) }
        },
    )
}

fn save() -> Element {
    root(
        Theme::Light,
        rsx! {
            Alert { title: "Save the changes?", style: AlertStyle::Warning,
                buttons: vec![
                    button("Save", AlertRole::Normal),
                    button("Don’t Save", AlertRole::Destructive),
                    button("Cancel", AlertRole::Cancel),
                ] }
        },
    )
}

fn hidden(flow: Flow) -> Element {
    root(
        Theme::Light,
        rsx! {
            Alert { title: TITLE, buttons: vec![button("Turn Off", AlertRole::Normal)], flow, shown: Some(Shown::Hidden) }
        },
    )
}

type Specimen = (&'static str, fn() -> Element);

const SPECIMENS: &[Specimen] = &[
    ("floating-light", || {
        bluetooth(AlertRole::Normal, Flow::Floating, Theme::Light)
    }),
    ("floating-dark", || {
        bluetooth(AlertRole::Normal, Flow::Floating, Theme::Dark)
    }),
    ("inline-popover", || {
        bluetooth(AlertRole::Normal, Flow::Inline, Theme::Light)
    }),
    ("destructive", || {
        bluetooth(AlertRole::Destructive, Flow::Floating, Theme::Light)
    }),
    ("destructive-icon", erase),
    ("hero-icon", hero),
    ("one-button", notice),
    ("three-buttons", save),
    ("suppression-help", extras),
    ("hidden-floating", || hidden(Flow::Floating)),
    ("hidden-inline", || hidden(Flow::Inline)),
];

/// Build the dom and flush the effects that register overlays, so a sheet is in the markup.
fn render(make: fn() -> Element) -> String {
    let mut dom = VirtualDom::new(make);
    dom.rebuild_in_place();
    for _ in 0..3 {
        dom.render_immediate(&mut NoOpMutations);
    }
    dioxus_ssr::render(&dom)
}

fn by_name(name: &str) -> String {
    let make = SPECIMENS
        .iter()
        .find(|(named, _)| *named == name)
        .unwrap_or_else(|| panic!("a specimen {name}"))
        .1;
    render(make)
}

#[test]
fn every_specimen_matches_its_golden() {
    let failures: Vec<String> = SPECIMENS
        .iter()
        .filter_map(|(name, make)| {
            golden::check(&format!("alert/{name}.html"), &render(*make)).err()
        })
        .collect();
    assert!(failures.is_empty(), "{}", failures.join("\n"));
}

#[test]
fn every_specimen_lints_clean_and_every_class_is_styled() {
    let sheet = ds::stylesheet();
    let mut failures = Vec::new();
    for (name, make) in SPECIMENS {
        let html = render(*make);
        for offence in markup(&html, sheet, &LintConfig::new(&ds::kits())) {
            failures.push(format!("{name}: {:?} {}", offence.rule, offence.text));
        }
        for class in html
            .split("class=\"")
            .skip(1)
            .filter_map(|rest| rest.split('"').next())
            .flat_map(str::split_whitespace)
            .filter(|class| class.starts_with("ds-"))
        {
            let needle = format!(".{class}");
            let styled = sheet.match_indices(&needle).any(|(at, _)| {
                !sheet[at + needle.len()..]
                    .starts_with(|c: char| c.is_ascii_alphanumeric() || c == '-' || c == '_')
            });
            if !styled {
                failures.push(format!("{name}: .{class} is not styled"));
            }
        }
    }
    failures.dedup();
    assert!(failures.is_empty(), "{}", failures.join("\n"));
}

/// The buttons in order, as `(label, answers, role)`; `answers` is empty for a button that
/// answers no key.
fn buttons(html: &str) -> Vec<(String, String, String)> {
    html.split("<button")
        .skip(1)
        .filter(|tag| tag.contains("class=\"ds-button"))
        .map(|tag| {
            let attr = |name: &str| {
                tag.split(&format!("{name}=\""))
                    .nth(1)
                    .and_then(|rest| rest.split('"').next())
                    .unwrap_or_default()
                    .to_owned()
            };
            let label = tag
                .split("class=\"ds-button-label\">")
                .nth(1)
                .and_then(|rest| rest.split('<').next())
                .unwrap_or_default()
                .to_owned();
            (label, attr("data-answers"), attr("data-role"))
        })
        .collect()
}

/// `(label, answers, role)` as `buttons` reads them.
fn b(label: &str, answers: &str, role: &str) -> (String, String, String) {
    (label.to_owned(), answers.to_owned(), role.to_owned())
}

#[test]
fn the_default_is_the_filled_first_button_unless_it_destroys() {
    let plain = by_name("floating-light");
    assert!(plain.contains(TITLE) && plain.contains(MESSAGE), "{plain}");
    assert_eq!(
        buttons(&plain),
        [
            b("Turn Off", "return", "normal"),
            b("Cancel", "escape", "normal")
        ]
    );
    for want in [
        "data-attach=\"centre\"",
        "data-width=\"narrow\"",
        "data-style=\"informational\"",
        "data-layout=\"row\"",
    ] {
        assert!(plain.contains(want), "{want} in {plain}");
    }
    assert!(
        !plain.contains("ds-scrim"),
        "an alert dims nothing: {plain}"
    );
    let destructive = by_name("destructive");
    assert_eq!(
        buttons(&destructive),
        [
            b("Turn Off", "", "destructive"),
            b("Cancel", "return", "normal")
        ]
    );
    assert!(
        !plain.contains("ds-alert-icon"),
        "no icon, no hero slot: {plain}"
    );
    let hero = by_name("hero-icon");
    assert!(hero.contains("class=\"ds-alert-icon\""), "{hero}");
    assert!(
        hero.contains("data-size=\"64\"") && !hero.contains("ds-alert-badge"),
        "a 64 px hero, no badge when informational: {hero}"
    );
    let icon = by_name("destructive-icon");
    assert!(icon.contains("class=\"ds-alert-icon\""), "{icon}");
    assert!(icon.contains("class=\"ds-alert-badge\""), "{icon}");
    assert!(icon.contains("data-style=\"critical\""), "{icon}");
}

#[test]
fn the_footer_lies_by_the_number_of_buttons() {
    let cases = [
        ("one-button", "row", vec![b("OK", "return", "normal")]),
        (
            "three-buttons",
            "stack",
            vec![
                b("Save", "return", "normal"),
                b("Don’t Save", "", "destructive"),
                b("Cancel", "escape", "normal"),
            ],
        ),
    ];
    for (name, layout, want) in cases {
        let html = by_name(name);
        assert!(
            html.contains(&format!("data-layout=\"{layout}\"")),
            "{name}: {html}"
        );
        assert_eq!(buttons(&html), want, "{name}");
    }
}

#[test]
fn a_suppression_checkbox_and_a_help_button_come_with_the_alert() {
    let html = by_name("suppression-help");
    for want in [
        "class=\"ds-alert-suppression\"",
        "class=\"ds-checkbox\"",
        "class=\"ds-alert-help\"",
        "data-variant=\"help\"",
    ] {
        assert!(html.contains(want), "{want} in {html}");
    }
}

#[test]
fn inline_it_stands_in_its_container_and_catches_the_pointer() {
    let inline = by_name("inline-popover");
    for want in [
        "class=\"ds-alert-stage\" data-flow=\"inline\"",
        "role=\"alertdialog\"",
        "aria-label=\"Turn Bluetooth off?\"",
    ] {
        assert!(inline.contains(want), "{want} in {inline}");
    }
    let stage = inline.find("ds-alert-stage").expect("a stage");
    let popover = inline.find("width:320px").expect("the popover");
    assert!(
        popover < stage,
        "drawn inside the popover, not in the overlay"
    );
    for name in ["hidden-floating", "hidden-inline"] {
        let html = by_name(name);
        assert!(!html.contains("ds-alert"), "{name}: {html}");
    }
}
