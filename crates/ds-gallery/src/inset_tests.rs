//! Every gallery page, in light and dark, holds the content inset rule: no text, glyph or image
//! sits closer to the painted edge of its box than the policy allows (ds_harness::inset;
//! ARCHITECTURE.md "Content inset check").
//!
//! `QUIRE_INSET_REPORT=<file>` writes every offence of every page to `<file>` and does not fail:
//! the way to see the whole list before fixing it.

use crate::app::App;
use crate::axes::{Axes, Showcase, start_with};
use crate::page::Page;
use crate::registry;
use ds::prelude::*;
use ds_harness::inset::{Findings, Policy, insets};
use ds_harness::{Clock, Driver, Harness, HarnessConfig, Viewport};
use std::time::Duration;

/// The width every page is laid out at, as the snapshots are.
const WIDTH: u32 = 1280;

/// How long a page runs before it is read: past every entrance and every placement.
const SETTLE: Duration = Duration::from_secs(10);

/// The policy of the gallery: quire's, for the components; the gallery's own `g-*` chrome is
/// the gallery's layout and holds the same default.
fn policy() -> Policy {
    Policy::quire()
}

fn findings_of(page: Page, theme: Theme) -> Findings {
    start_with(Axes {
        page,
        theme,
        showcase: Showcase::Posed,
        ..Axes::default()
    });
    let view = Viewport {
        width: WIDTH,
        height: registry::entry(page).height,
        scale_percent: 100,
    };
    let mut harness = Harness::new(App, HarnessConfig::new(view).with_clock(Clock::Virtual));
    harness.advance(SETTLE);
    insets(&harness, &policy())
}

#[test]
fn every_gallery_page_keeps_its_content_off_the_box_edges() {
    let mut lines = Vec::new();
    let mut all = Findings::default();
    for page in Page::ALL.iter().copied() {
        for theme in [Theme::Light, Theme::Dark] {
            let found = findings_of(page, theme);
            lines.extend(
                found
                    .offences
                    .iter()
                    .map(|offence| format!("{page:?} {theme:?} {offence}")),
            );
            all.merge(found);
        }
    }
    if let Ok(file) = std::env::var("QUIRE_INSET_REPORT") {
        lines.sort();
        lines.dedup();
        std::fs::write(&file, lines.join("\n") + "\n").expect("the report file is writable");
        return;
    }
    lines.sort();
    lines.dedup();
    assert!(
        lines.is_empty(),
        "{} offence(s):\n{}",
        lines.len(),
        lines.join("\n")
    );
    let stale: Vec<_> = policy()
        .allow
        .iter()
        .filter(|entry| !all.excused.iter().any(|excused| excused.when == entry.when))
        .map(|entry| entry.when)
        .collect();
    assert!(
        stale.is_empty(),
        "allowances that excused nothing: {stale:?}"
    );
}
