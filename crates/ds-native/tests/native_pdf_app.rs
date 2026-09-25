//! Printing a Dioxus app: `Harness::pdf` on a live harness and `pdf_app` from scratch, and the
//! variable face printed at each weight the layout used.

#[path = "support/pdf_read.rs"]
mod pdf_read;

use dioxus::prelude::*;
use ds_native::{Harness, HarnessConfig, PageSpec, Viewport, pdf_app};

fn app() -> Element {
    rsx! {
        div { style: "font: 16px/1.4 'Karla', sans-serif; color: #222;",
            h1 { style: "font-size: 28px; margin: 0 0 12px;", "Printed from Dioxus" }
            p { style: "font-weight: 400;", "Regular weight kestrel" }
            p { style: "font-weight: 700;", "Bold weight falcon" }
            section { "data-break-before": "page",
                p { "Second page heron" }
            }
        }
    }
}

const SMALL: Viewport = Viewport {
    width: 400,
    height: 300,
    scale_percent: 100,
};

#[test]
fn a_harness_prints_its_document_and_carries_on() {
    let mut harness = Harness::new(app, SMALL);
    let before = harness.rect("h1").expect("the heading is laid out");
    let bytes = harness.pdf(PageSpec::default()).expect("the app prints");
    let doc = pdf_read::open(&bytes);
    assert_eq!(doc.page_count(), 2, "the marked section starts page 2");
    let first = pdf_read::text(&doc, 0);
    assert!(first.contains("Printed from Dioxus"), "{first}");
    assert!(first.contains("Regular weight kestrel") && first.contains("Bold weight falcon"));
    assert!(pdf_read::text(&doc, 1).contains("Second page heron"));
    let after = harness.rect("h1").expect("the heading is still laid out");
    assert_eq!(before, after, "the harness is back at its own viewport");
}

#[test]
fn pdf_app_prints_at_the_page_width() {
    let bytes = pdf_app(app, HarnessConfig::new(SMALL), PageSpec::default()).expect("prints");
    let doc = pdf_read::open(&bytes);
    assert_eq!(doc.page_count(), 2);
    assert!(pdf_read::text(&doc, 0).contains("Bold weight falcon"));
}

#[test]
fn each_weight_of_a_variable_face_embeds_its_own_instance() {
    let bytes = pdf_app(app, HarnessConfig::new(SMALL), PageSpec::default()).expect("prints");
    let karla: Vec<_> = pdf_read::open(&bytes)
        .embedded_fonts()
        .into_iter()
        .filter(|font| font.name.contains("Karla"))
        .collect();
    // 400 for the paragraphs, 700 for the heading and the bold paragraph: one variable file,
    // instanced twice by the subsetter at the layout's `wght`.
    assert_eq!(
        karla.len(),
        2,
        "{:?}",
        karla.iter().map(|f| &f.name).collect::<Vec<_>>()
    );
    assert_ne!(
        karla[0].data, karla[1].data,
        "the two instances have different outlines"
    );
}

#[test]
fn each_instance_is_named_after_its_weight() {
    let mut harness = Harness::new(app, SMALL);
    let bytes = harness.pdf(PageSpec::default()).expect("the app prints");
    let mut names: Vec<String> = pdf_read::open(&bytes)
        .embedded_fonts()
        .into_iter()
        .filter_map(|font| Some(font.name.split_once('+')?.1.to_owned()))
        .filter(|name| name.starts_with("Karla"))
        .collect();
    names.sort();
    // Named after the instance each subset was cut at, not the variable face's default.
    assert_eq!(names, ["Karla-Bold", "Karla-Regular"]);
}
