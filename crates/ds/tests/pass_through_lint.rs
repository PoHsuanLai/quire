//! mailo gaps 6, a consumer's own class on a quire button: the markup lint takes it when the
//! consumer's sheet styles it, the stylesheet lint takes the consumer's rule for it, and a `ds-`
//! class never gets that far, since `ExtraClass` refuses it when it is built. The same for a
//! `data-*` name: `DataName` refuses `ds-…` and the names quire writes itself.

use dioxus::prelude::*;
use ds::lint::{LintConfig, Rule, markup, stylesheet};
use ds::{Button, ButtonVariant, DataAttr, DataName, ExtraClass, PassThroughError};

/// The consumer's own rule for its class: a reveal on the row's hover.
const CONSUMER_CSS: &str = ".row-reveal { opacity: 0; transition: opacity var(--t-quick) var(--e-out); }\n.row:hover .row-reveal { opacity: 1; }";

#[component]
fn Page() -> Element {
    let class = ExtraClass::parse("row-reveal").ok();
    let data = DataName::parse("folder")
        .map(|name| vec![DataAttr::new(name, "INBOX")])
        .unwrap_or_default();
    rsx! {
        Button { variant: ButtonVariant::Mini, label: "Reply", extra_class: class, data, onclick: |_| {} }
    }
}

fn render() -> String {
    let mut dom = VirtualDom::new(Page);
    dom.rebuild_in_place();
    dioxus_ssr::render(&dom)
}

#[test]
fn a_consumer_class_on_a_quire_button_lints_clean() {
    let html = render();
    assert!(
        html.contains(r#"class="ds-button row-reveal""#) && html.contains(r#"data-folder="INBOX""#),
        "{html}"
    );
    let css = format!("{}\n{CONSUMER_CSS}", ds::stylesheet());
    let offences = markup(&html, &css, &LintConfig::default());
    assert!(offences.is_empty(), "{offences:#?}");
    let own = stylesheet(CONSUMER_CSS, &LintConfig::default());
    assert!(own.is_empty(), "{own:#?}");
}

#[test]
fn the_class_is_unstyled_without_the_consumers_rule() {
    // The check above is not vacuous: the same markup against quire's sheet alone names the
    // consumer's class as unstyled.
    let offences = markup(&render(), ds::stylesheet(), &LintConfig::default());
    assert!(
        offences
            .iter()
            .any(|offence| offence.rule == Rule::UnstyledClass
                && offence.text.contains("row-reveal")),
        "{offences:#?}"
    );
}

#[test]
fn a_ds_class_or_name_is_refused_before_it_reaches_a_button() {
    for class in ["ds-button", "row-reveal ds-icon-button", "ds-anything"] {
        assert!(
            matches!(
                ExtraClass::parse(class),
                Err(PassThroughError::Reserved { .. })
            ),
            "{class}"
        );
    }
    for name in [
        "ds-drop", "variant", "theme", "accent", "motion", "material", "size",
    ] {
        assert!(
            matches!(
                DataName::parse(name),
                Err(PassThroughError::Reserved { .. })
            ),
            "{name}"
        );
    }
    // And a consumer rule that did reach for quire's class is the stylesheet lint's to catch.
    let reaching = stylesheet(
        ".ds-button.row-reveal { opacity: 0; }",
        &LintConfig::default(),
    );
    assert!(
        reaching
            .iter()
            .any(|offence| offence.rule == Rule::DsInternals),
        "{reaching:#?}"
    );
}
