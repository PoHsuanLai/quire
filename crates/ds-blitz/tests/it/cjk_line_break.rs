//! Chinese breaks at word boundaries (FINDINGS "CJK segmentation"): parley segments Chinese and
//! Japanese with ICU4X's dictionary models only under its `complex-scripts` feature; without it
//! every window logs "No segmentation model for complex script" and `word-break: keep-all`
//! text, which may break only between words, has none to break at and overflows on one line.
//! A narrow `keep-all` box therefore takes more than one line only with the dictionaries built in.
//! The logged error itself is not observable from a test.

use blitz_dom::DocumentConfig;
use dioxus::prelude::*;
use dioxus_native_dom::DioxusDocument;

const LINE_PX: f64 = 20.0;

#[component]
fn Box(break_rule: String) -> Element {
    rsx! {
        div { id: "box", style: "width:64px; font-size:16px; line-height:20px; word-break:{break_rule};",
            "我們今天去公園玩耍然後回家吃飯"
        }
    }
}

/// How many lines the sentence takes in a 64 px box under `word-break: break_rule`.
fn lines(break_rule: &str) -> u32 {
    let vdom = VirtualDom::new_with_props(
        Box,
        BoxProps {
            break_rule: break_rule.to_string(),
        },
    );
    let mut doc = DioxusDocument::new(
        vdom,
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
    (height / LINE_PX).round() as u32
}

#[test]
fn chinese_wraps_between_ideographs_normally() {
    assert!(lines("normal") > 1, "normal wrapping breaks the sentence");
}

#[test]
fn keep_all_chinese_breaks_between_dictionary_words() {
    let wrapped = lines("keep-all");
    assert!(
        wrapped > 1,
        "keep-all found no word boundary in a Chinese sentence ({wrapped} line): ICU4X's dictionary models are missing"
    );
}
