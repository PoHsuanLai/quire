//! `Resolved` reads a custom property from an element's computed style: a token declared on
//! `.ds` reaches the elements inside it, a later sheet's override wins, and a value that is not
//! a plain length or number reads as nothing.

use blitz_dom::{BaseDocument, NodeId};
use dioxus::prelude::*;
use ds::prelude::Px;
use ds_blitz::Resolved;
use ds_harness::{Clock, Harness, HarnessConfig, Viewport};

const VIEW: Viewport = Viewport {
    width: 100,
    height: 100,
    scale_percent: 100,
};

#[allow(non_snake_case)]
fn Tokens() -> Element {
    rsx! {
        style { ".ds {{ --t: 10px; --n: 1.5; --twice: calc(6px * 2); --em: 2em }}" }
        style { ".ds {{ --t: 12px }}" }
        div { class: "ds",
            div { class: "inner" }
        }
    }
}

fn inner(doc: &BaseDocument) -> NodeId {
    doc.query_selector(".inner")
        .ok()
        .flatten()
        .expect("the inner element")
}

#[test]
fn a_token_reads_back_as_a_length_or_a_number_and_else_none() {
    let harness = Harness::new(Tokens, HarnessConfig::new(VIEW).with_clock(Clock::Virtual));
    harness.with_doc(|doc| {
        let node = inner(doc);
        assert_eq!(
            Resolved::length(doc, node, "--t"),
            Some(Px(12.0)),
            "the later sheet's override, inherited from .ds"
        );
        assert_eq!(Resolved::number(doc, node, "--n"), Some(1.5));
        assert_eq!(
            Resolved::length(doc, node, "--n"),
            None,
            "no unit is not a length"
        );
        assert_eq!(Resolved::length(doc, node, "--em"), None, "another unit");
        // A custom property keeps its tokens; `calc()` is not evaluated on the way.
        assert_eq!(
            Resolved::length(doc, node, "--twice"),
            None,
            "an expression"
        );
        assert_eq!(
            Resolved::number(doc, node, "--t"),
            None,
            "a length is not a number"
        );
        assert_eq!(Resolved::length(doc, node, "--missing"), None);
        assert_eq!(
            Resolved::length(doc, node, "t"),
            None,
            "not a custom property name"
        );
    });
}

static READER: GlobalSignal<Option<ds_blitz::Tokens>> = Signal::global(|| None);

#[allow(non_snake_case)]
fn Mounted() -> Element {
    let tokens = ds_blitz::use_tokens();
    use_hook(|| *READER.write() = Some(tokens));
    rsx! {
        style { ".ds {{ --gap: 7px; --gain: 2 }}" }
        div { class: "ds",
            div { onmounted: move |event| tokens.mounted(event) }
        }
    }
}

#[test]
fn a_component_reads_the_tokens_its_element_sees() {
    let mut harness = Harness::new(Mounted, HarnessConfig::new(VIEW).with_clock(Clock::Virtual));
    harness.advance(std::time::Duration::from_millis(16));
    let (gap, gain) = harness.within(|| {
        let tokens = READER.peek().expect("the reader");
        (tokens.length("--gap"), tokens.number("--gain"))
    });
    assert_eq!(gap, Some(Px(7.0)));
    assert_eq!(gain, Some(2.0));
}
