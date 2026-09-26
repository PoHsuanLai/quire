//! The PDF thumbnail as markup (sill M9's launcher preview): each state, loading (inside the
//! grace), pending (past it), a portrait and a landscape page, an empty document, and the two
//! failures, matches its golden under `tests/snapshots/pdf_thumb/`, lints clean, and uses only
//! `ds-` classes the stylesheet styles.
//!
//! `DS_BLESS=1 cargo test -p ds --features lint --test pdf_thumb_ssr` rewrites the goldens.

#[path = "support/golden.rs"]
mod golden;

use dioxus::core::NoOpMutations;
use dioxus::prelude::*;
use ds::lint::{LintConfig, markup};
use ds::{
    Appearance, Ds, ImageSize, ImageSource, Inject, Material, PDF_THUMB_GRACE, PdfPage, PdfThumb,
    PdfTrouble, Px, Size, Theme,
};
use std::pin::pin;
use std::sync::Arc;
use std::task::{Context, Poll, Wake, Waker};
use std::thread::{self, Thread};
use std::time::{Duration, Instant};

const ROOM: Size = Size {
    width: Px(160.0),
    height: Px(200.0),
};

fn root(theme: Theme, body: Element) -> Element {
    rsx! {
        Ds { appearance: Appearance { theme, ..Appearance::default() }, material: Material::Window, stylesheet: Inject::Host,
            {body}
        }
    }
}

fn page(width: u32, height: u32) -> PdfPage {
    PdfPage::Ready {
        image: ImageSource("data:image/png;base64,AAAA".to_owned()),
        sheet: ImageSize { width, height },
    }
}

fn loading() -> Element {
    root(
        Theme::Light,
        rsx! { PdfThumb { page: PdfPage::Loading, size: ROOM } },
    )
}

fn letter() -> Element {
    root(
        Theme::Light,
        rsx! { PdfThumb { page: page(612, 792), size: ROOM, label: "Invoice.pdf" } },
    )
}

fn landscape_dark() -> Element {
    root(
        Theme::Dark,
        rsx! { PdfThumb { page: page(842, 595), size: ROOM } },
    )
}

fn empty() -> Element {
    root(
        Theme::Light,
        rsx! { PdfThumb { page: PdfPage::Empty, size: ROOM } },
    )
}

fn unreadable() -> Element {
    root(
        Theme::Light,
        rsx! { PdfThumb { page: PdfPage::Failed(PdfTrouble::Unreadable), size: ROOM } },
    )
}

fn locked_dark() -> Element {
    root(
        Theme::Dark,
        rsx! { PdfThumb { page: PdfPage::Failed(PdfTrouble::Locked), size: ROOM } },
    )
}

/// A specimen: its golden name, how it is made, and how long its timers run first.
type Specimen = (&'static str, fn() -> Element, Duration);

const NOW: Duration = Duration::ZERO;

const SPECIMENS: &[Specimen] = &[
    ("loading", loading, NOW),
    ("pending", loading, PDF_THUMB_GRACE.saturating_mul(2)),
    ("letter", letter, NOW),
    ("landscape-dark", landscape_dark, NOW),
    ("empty", empty, NOW),
    ("unreadable", unreadable, NOW),
    ("locked-dark", locked_dark, NOW),
];

struct Unpark(Thread);

impl Wake for Unpark {
    fn wake(self: Arc<Self>) {
        self.0.unpark();
    }
}

/// Let the dom's timers run for `span`, rendering whatever they dirty.
fn run_for(dom: &mut VirtualDom, span: Duration) {
    let end = Instant::now() + span;
    let waker = Waker::from(Arc::new(Unpark(thread::current())));
    let mut cx = Context::from_waker(&waker);
    while Instant::now() < end {
        let ready = {
            let mut work = pin!(dom.wait_for_work());
            loop {
                if let Poll::Ready(()) = work.as_mut().poll(&mut cx) {
                    break true;
                }
                let now = Instant::now();
                if now >= end {
                    break false;
                }
                thread::park_timeout(end - now);
            }
        };
        if ready {
            dom.render_immediate(&mut NoOpMutations);
        }
    }
}

fn render(make: fn() -> Element, wait: Duration) -> String {
    let mut dom = VirtualDom::new(make);
    dom.rebuild_in_place();
    run_for(&mut dom, wait);
    dioxus_ssr::render(&dom)
}

#[test]
fn every_specimen_matches_its_golden() {
    let failures: Vec<String> = SPECIMENS
        .iter()
        .filter_map(|(name, make, wait)| {
            golden::check(&format!("pdf_thumb/{name}.html"), &render(*make, *wait)).err()
        })
        .collect();
    assert!(failures.is_empty(), "{}", failures.join("\n"));
}

#[test]
fn every_specimen_lints_clean_and_every_class_is_styled() {
    let sheet = ds::stylesheet();
    let mut failures = Vec::new();
    for (name, make, wait) in SPECIMENS {
        let html = render(*make, *wait);
        for offence in markup(&html, sheet, &LintConfig::default()) {
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

/// Each state stamps its word, the sheet sits at the page's aspect, and a failure draws a plate
/// and no sheet.
#[test]
fn the_markup_carries_the_state() {
    let fresh = render(loading, NOW);
    assert!(fresh.contains("data-state=\"loading\""), "{fresh}");
    assert!(fresh.contains("aria-busy=\"true\""), "{fresh}");
    let later = render(loading, PDF_THUMB_GRACE.saturating_mul(2));
    assert!(later.contains("data-state=\"pending\""), "{later}");
    let ready = render(letter, NOW);
    assert!(ready.contains("data-state=\"ready\""), "{ready}");
    assert!(
        ready.contains("left:2.73px;top:0px;width:154.55px;height:200px"),
        "{ready}"
    );
    assert!(ready.contains("aria-label=\"Invoice.pdf\""), "{ready}");
    let failed = render(locked_dark, NOW);
    assert!(failed.contains("data-trouble=\"locked\""), "{failed}");
    assert!(!failed.contains("ds-pdf-thumb-sheet"), "{failed}");
    assert!(failed.contains("aria-label=\"Locked PDF\""), "{failed}");
}
