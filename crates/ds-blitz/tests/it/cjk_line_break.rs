//! Chinese segments into words (FINDINGS "CJK segmentation"): parley segments Chinese and Japanese
//! with ICU4X's dictionary models only under its `complex-scripts` feature; without it every
//! window logs "No segmentation model for complex script". The log is not observable from a test,
//! and parley's cluster flags do not expose the dictionary's word starts, so this checks only that
//! Chinese still wraps between ideographs (UAX #14, no dictionary needed); a real window shows
//! whether the log line is gone.

use blitz_dom::DocumentConfig;
use dioxus::prelude::*;
use dioxus_native_dom::DioxusDocument;

const LINE_PX: f64 = 20.0;
const SENTENCE: &str = "我們今天去公園";

#[component]
fn Sentence() -> Element {
    rsx! {
        div { id: "box", style: "width:64px; font-size:16px; line-height:20px;", "{SENTENCE}" }
    }
}

#[test]
fn chinese_wraps_between_ideographs() {
    let mut doc = DioxusDocument::new(
        VirtualDom::new(Sentence),
        DocumentConfig {
            font_ctx: Some(ds_blitz::font_context()),
            ..Default::default()
        },
    );
    doc.initial_build();
    let mut inner = doc.inner.borrow_mut();
    inner.resolve(0.0);
    let node = inner
        .query_selector("#box")
        .ok()
        .flatten()
        .expect("the box is in the document");
    let height = inner
        .get_client_bounding_rect(node)
        .expect("the box is laid out")
        .height;
    assert!((height / LINE_PX).round() > 1.0, "the sentence wraps");
}
