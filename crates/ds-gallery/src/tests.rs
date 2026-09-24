//! Whole-gallery tests: every page renders through the real root, and what it renders is
//! quire's markup only (coherence rule 2); one page snapshots to real pixels.

use crate::app::App;
use crate::axes::{Axes, Showcase, start_with};
use crate::page::Page;
use crate::style;
use dioxus::prelude::*;
use ds::lint::{Exception, LintConfig, Profile, Rule, markup};
use ds_native::{Viewport, snapshot_at};
use std::time::Duration;

/// Offences in quire's own markup: every one is drawn by a quire component, not by the
/// gallery, and each is a gap reported to quire (the Gaps page lists them). The computed
/// custom properties quire writes inline on its own elements (the `--f-*` frame, `--f-grad`,
/// `--av-bg`, `--pc`) and the SendPill's marked ring are not offences; what is left paints a
/// literal colour straight into a non-custom property, or is a class nothing styles.
const EXCEPTIONS: &[Exception] = &[
    Exception {
        rule: Rule::HexColour,
        selector: "button.ds-space-dot",
        reason: "SpaceDot paints its Space's gradient inline as background, not a custom property",
    },
    Exception {
        rule: Rule::HexColour,
        selector: "button.ds-preset",
        reason: "the Space editor's preset buttons paint their gradients inline as background",
    },
    Exception {
        rule: Rule::HexColour,
        selector: "div.ds-handle",
        reason: "the Space editor's dot handles are filled with their picked colour inline as background",
    },
    Exception {
        rule: Rule::HexColour,
        selector: "i.ds-stop-disc",
        reason: "the Space editor's stop discs are their stop colours inline as background",
    },
    Exception {
        rule: Rule::HexColour,
        selector: "span.ds-space-swatch",
        reason: "the Space editor's swatch is its picked colour inline as background",
    },
];

/// The page's markup after its first render, posed.
fn rendered(page: Page) -> String {
    start_with(Axes {
        page,
        showcase: Showcase::Posed,
        ..Axes::default()
    });
    let mut dom = VirtualDom::new(App);
    dom.rebuild_in_place();
    dom.render_immediate_to_vec();
    dioxus_ssr::render(&dom)
}

#[test]
fn every_page_renders_quire_markup_only() {
    let css = format!("{}\n{}", ds::stylesheet(), style::CSS);
    let config = LintConfig {
        profile: Profile::Standard,
        exceptions: EXCEPTIONS,
        ..LintConfig::default()
    };
    let mut failures = Vec::new();
    for page in Page::ALL {
        let html = rendered(page);
        assert!(
            html.contains("class=\"g-card\""),
            "{page:?} did not render the page card"
        );
        for offence in markup(&html, &css, &config) {
            failures.push(format!(
                "{page:?}: {:?} {} {}",
                offence.rule, offence.selector, offence.text
            ));
        }
    }
    failures.sort();
    failures.dedup();
    assert!(failures.is_empty(), "{failures:#?}");
}

#[test]
fn every_exception_still_suppresses_something() {
    let css = format!("{}\n{}", ds::stylesheet(), style::CSS);
    let bare = LintConfig {
        profile: Profile::Standard,
        ..LintConfig::default()
    };
    let offences: Vec<_> = Page::ALL
        .into_iter()
        .flat_map(|page| markup(&rendered(page), &css, &bare))
        .collect();
    let stale: Vec<&str> = EXCEPTIONS
        .iter()
        .filter(|exception| !offences.iter().any(|offence| exception.covers(offence)))
        .map(|exception| exception.selector)
        .collect();
    assert!(stale.is_empty(), "exceptions that match nothing: {stale:?}");
}

#[test]
fn a_page_snapshots_to_real_pixels() {
    start_with(Axes {
        page: Page::Controls,
        showcase: Showcase::Posed,
        ..Axes::default()
    });
    let view = Viewport {
        width: 400,
        height: 300,
        scale_percent: 100,
    };
    let frames = snapshot_at(App, view, &[Duration::from_secs(10)]).expect("renders");
    let frame = &frames[0];
    assert_eq!(frame.dimensions(), (400, 300));
    let mut colours = frame.pixels().map(|pixel| pixel.0).collect::<Vec<_>>();
    colours.sort_unstable();
    colours.dedup();
    assert!(colours.len() > 64, "only {} colours: blank", colours.len());
}
