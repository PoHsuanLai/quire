//! Whole-gallery tests: every page renders through the real root, and what it renders is
//! quire's markup only (coherence rule 2); one page snapshots to real pixels.

use crate::app::App;
use crate::axes::{Axes, Showcase, start_with};
use crate::page::Page;
use crate::style;
use dioxus::prelude::*;
use ds::lint::{Exception, LintConfig, Profile, markup};
use ds_native::{Viewport, snapshot_at};
use std::time::Duration;

/// Offences in quire's own markup the gallery lives with: none. Every one would be drawn by a
/// quire component, not by the gallery, and a gap reported to quire (the Gaps page lists them).
/// The computed custom properties quire writes inline on its own elements (the `--f-*` frame,
/// `--f-grad`, `--av-bg`, `--pc`, a Space dot's `--dot-c*`) and the SendPill's marked ring are
/// not offences. The Space editor's five inline gradients and fills were the last (mailo gaps 3).
const EXCEPTIONS: &[Exception] = &[];

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

/// The Space page, SpaceDot and the whole Space editor on it, lints clean under Strict with no
/// exception: every Space colour is a custom property its stylesheet paints (mailo gaps 3).
#[test]
fn the_space_page_is_clean_under_strict() {
    let css = format!("{}\n{}", ds::stylesheet(), style::CSS);
    let strict = LintConfig {
        profile: Profile::Strict,
        ..LintConfig::default()
    };
    let html = rendered(Page::Space);
    assert!(
        html.contains("ds-space-dot"),
        "the page drew its Space dots"
    );
    assert!(html.contains("--dot-c1:"), "the dots carry their colours");
    let offences = markup(&html, &css, &strict);
    assert!(offences.is_empty(), "{offences:#?}");
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
