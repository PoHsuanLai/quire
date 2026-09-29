//! The alert as markup: every state matches its golden under
//! `tests/snapshots/alert/`, lints clean and uses only `ds-` classes the stylesheet styles; the
//! markup says which button is the default (Primary) and how a destructive action reads.
//!
//! `DS_BLESS=1 cargo test -p ds --features lint --test alert_ssr` rewrites the goldens.

#[path = "support/golden.rs"]
mod golden;

use dioxus::core::NoOpMutations;
use dioxus::prelude::*;
use ds::lint::{LintConfig, markup};
use ds::{
    Alert, AlertEmphasis, Appearance, Ds, Flow, Icon, IconSource, Inject, Material, RootExtent,
    Shown, TextLine, Theme,
};

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

fn bluetooth(emphasis: AlertEmphasis, flow: Flow, theme: Theme) -> Element {
    let alert = rsx! {
        Alert { title: TITLE, message: Some(TextLine::from(MESSAGE)), action: "Turn Off", emphasis, flow,
            onaction: |_| {}, oncancel: |_| {} }
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
                action: "Erase", emphasis: AlertEmphasis::Destructive, icon: Some(IconSource::Glyph(Icon::Trash)),
                onaction: |_| {}, oncancel: |_| {} }
        },
    )
}

fn hidden(flow: Flow) -> Element {
    root(
        Theme::Light,
        rsx! {
            Alert { title: TITLE, action: "Turn Off", flow, shown: Some(Shown::Hidden),
                onaction: |_| {}, oncancel: |_| {} }
        },
    )
}

type Specimen = (&'static str, fn() -> Element);

const SPECIMENS: &[Specimen] = &[
    ("floating-light", || {
        bluetooth(AlertEmphasis::Default, Flow::Floating, Theme::Light)
    }),
    ("floating-dark", || {
        bluetooth(AlertEmphasis::Default, Flow::Floating, Theme::Dark)
    }),
    ("inline-popover", || {
        bluetooth(AlertEmphasis::Default, Flow::Inline, Theme::Light)
    }),
    ("destructive", || {
        bluetooth(AlertEmphasis::Destructive, Flow::Floating, Theme::Light)
    }),
    ("destructive-icon", erase),
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

/// The buttons in order, as `(label, variant)`.
fn buttons(html: &str) -> Vec<(String, String)> {
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
                .split("<span>")
                .nth(1)
                .and_then(|rest| rest.split('<').next())
                .unwrap_or_default()
                .to_owned();
            (label, attr("data-variant"))
        })
        .collect()
}

#[test]
fn the_default_is_the_filled_button_on_the_right_unless_the_action_destroys() {
    let plain = by_name("floating-light");
    assert!(plain.contains(TITLE) && plain.contains(MESSAGE), "{plain}");
    assert_eq!(
        buttons(&plain),
        [
            ("Cancel".to_owned(), "secondary".to_owned()),
            ("Turn Off".to_owned(), "primary".to_owned())
        ]
    );
    for want in [
        "data-placement=\"centre\"",
        "data-width=\"narrow\"",
        "data-strength=\"modal\"",
    ] {
        assert!(plain.contains(want), "{want} in {plain}");
    }
    let destructive = by_name("destructive");
    assert_eq!(
        buttons(&destructive),
        [
            ("Cancel".to_owned(), "primary".to_owned()),
            ("Turn Off".to_owned(), "danger".to_owned())
        ]
    );
    assert!(destructive.contains("data-emphasis=\"destructive\""));
    assert!(destructive.contains("data-size=\"regular\""));
    let icon = by_name("destructive-icon");
    assert!(icon.contains("class=\"ds-alert-icon\""), "{icon}");
}

#[test]
fn inline_it_stands_in_its_container_with_its_own_scrim() {
    let inline = by_name("inline-popover");
    for want in [
        "class=\"ds-alert-stage\" data-flow=\"inline\"",
        "class=\"ds-scrim\"",
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
