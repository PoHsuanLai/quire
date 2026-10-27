//! `Resolved` reads a custom property from the root's computed style: a length in px, a plain
//! number, and nothing for a missing property or a value that is not that kind.

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
        div { class: "ds",
            style { ":root {{ --t: 12px; --n: 1.5; --twice: calc(6px * 2); --em: 2em }}" }
        }
    }
}

#[test]
fn a_token_reads_back_as_a_length_or_a_number_and_else_none() {
    let harness = Harness::new(Tokens, HarnessConfig::new(VIEW).with_clock(Clock::Virtual));
    harness.with_doc(|doc| {
        assert_eq!(Resolved::length(doc, "--t"), Some(Px(12.0)));
        assert_eq!(Resolved::number(doc, "--n"), Some(1.5));
        assert_eq!(
            Resolved::length(doc, "--n"),
            None,
            "no unit is not a length"
        );
        assert_eq!(Resolved::length(doc, "--em"), None, "another unit");
        // A custom property keeps its tokens; `calc()` is not evaluated on the way.
        assert_eq!(Resolved::length(doc, "--twice"), None, "an expression");
        assert_eq!(
            Resolved::number(doc, "--t"),
            None,
            "a length is not a number"
        );
        assert_eq!(Resolved::length(doc, "--missing"), None);
        assert_eq!(
            Resolved::length(doc, "t"),
            None,
            "not a custom property name"
        );
    });
}
