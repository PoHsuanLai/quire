//! The content inset check on real Blitz layouts: what it flags, what it lets be, and how it
//! measures nested boxes.

use dioxus::prelude::*;
use ds::prelude::Px;
use ds_harness::inset::{Allow, ContentKind, Policy, Side, insets, measure, read};
use ds_harness::{Harness, Viewport};

const VIEW: Viewport = Viewport {
    width: 400,
    height: 200,
    scale_percent: 100,
};

const BOX: &str = "background:#336;color:#fff;font:14px sans-serif;";

fn padded(padding: &str) -> Element {
    rsx! {
        div { style: "padding:20px;background:#fff",
            div { class: "row", style: "{BOX}padding:4px {padding};width:200px", "Hello world" }
        }
    }
}

#[component]
#[allow(non_snake_case)]
fn TwoPx() -> Element {
    padded("2px")
}

#[component]
#[allow(non_snake_case)]
fn EightPx() -> Element {
    padded("8px")
}

#[component]
#[allow(non_snake_case)]
fn IconButton() -> Element {
    rsx! {
        div { style: "padding:20px;background:#fff",
            button {
                class: "tool",
                style: "{BOX}width:28px;height:28px;display:flex;align-items:center;justify-content:center;border:0;padding:0",
                svg { width: "16", height: "16", view_box: "0 0 16 16",
                    circle { cx: "8", cy: "8", r: "6", fill: "white" }
                }
            }
        }
    }
}

#[component]
#[allow(non_snake_case)]
fn Nested() -> Element {
    rsx! {
        div { style: "padding:20px;background:#fff",
            div { class: "outer", style: "background:#ddd;padding:12px;width:300px;color:#000;font:14px sans-serif",
                div { class: "inner", style: "background:#336;color:#fff;padding:4px 2px;width:120px", "Inner text" }
            }
        }
    }
}

#[component]
#[allow(non_snake_case)]
fn CornerMark() -> Element {
    rsx! {
        div { style: "padding:20px;background:#fff",
            div { class: "tile", style: "{BOX}position:relative;width:60px;height:60px",
                svg { style: "position:absolute;top:2px;left:2px", width: "10", height: "10", view_box: "0 0 10 10",
                    circle { cx: "5", cy: "5", r: "4", fill: "white" }
                }
            }
        }
    }
}

fn found(app: fn() -> Element) -> ds_harness::inset::Findings {
    let harness = Harness::new(app, VIEW);
    insets(&harness, &Policy::new())
}

#[test]
fn a_box_with_two_pixels_of_padding_is_flagged_on_its_left() {
    let findings = found(TwoPx);
    let left: Vec<_> = findings
        .offences
        .iter()
        .filter(|offence| offence.side == Side::Left)
        .collect();
    assert_eq!(left.len(), 1, "{}", findings.text());
    assert!((left[0].gap.0 - 2.0).abs() < 0.5, "{}", left[0]);
    assert!(left[0].path.ends_with("div.row"), "{}", left[0]);
    assert!(left[0].content.contains("Hello"), "{}", left[0]);
}

#[test]
fn a_box_with_eight_pixels_of_padding_is_clean() {
    let findings = found(EightPx);
    assert!(findings.offences.is_empty(), "{}", findings.text());
}

#[test]
fn a_centred_icon_button_is_not_flagged() {
    let harness = Harness::new(IconButton, VIEW);
    let scene = read_scene(&harness);
    assert!(
        scene
            .contents
            .iter()
            .any(|content| content.kind == ContentKind::Glyph),
        "the svg was read as a glyph: {scene:#?}"
    );
    let findings = measure(&scene, &Policy::new());
    assert!(findings.offences.is_empty(), "{}", findings.text());
}

#[test]
fn nested_boxes_measure_against_the_nearest_box() {
    let findings = found(Nested);
    let paths: Vec<_> = findings.offences.iter().map(|o| o.path.as_str()).collect();
    assert!(
        paths.iter().all(|path| path.ends_with("div.inner")),
        "only the inner box is too tight: {}",
        findings.text()
    );
    assert!(!paths.is_empty(), "the inner box was flagged");
}

#[test]
fn an_allowance_lowers_the_minimum_for_its_class_and_is_counted() {
    let harness = Harness::new(TwoPx, VIEW);
    let policy = Policy::new().allowing(&[Allow {
        when: &["row"],
        min: Px(2.0),
        reason: "test",
    }]);
    let findings = insets(&harness, &policy);
    assert!(findings.offences.is_empty(), "{}", findings.text());
    assert!(
        !findings.excused.is_empty(),
        "the allowance excused the gap"
    );
}

fn read_scene(harness: &Harness) -> ds_harness::inset::Scene {
    use ds_harness::DocQuery;
    harness.with_doc(read)
}

#[test]
fn an_absolutely_positioned_mark_is_placed_by_its_own_offsets_not_inset() {
    let findings = found(CornerMark);
    assert!(findings.offences.is_empty(), "{}", findings.text());
}
