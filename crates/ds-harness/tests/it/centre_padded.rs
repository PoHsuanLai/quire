//! `Harness::centre` and `Harness::rect` give the border box in the window's coordinates, however
//! the target is nested: a click at the centre of a child under a padded parent lands on the child.

use dioxus::prelude::*;
use ds_core::geometry::units::Px;
use ds_harness::{Driver, Harness, Input, Query, Viewport};

const VIEW: Viewport = Viewport {
    width: 400,
    height: 300,
    scale_percent: 100,
};

#[allow(non_snake_case)]
fn Padded() -> Element {
    let mut hits = use_signal(|| 0u32);
    rsx! {
        div { id: "outer", style: "padding:37px 11px 5px 23px; border:3px solid black; margin:7px",
            div { id: "inner", style: "padding:13px",
                button { id: "child", style: "display:block; width:80px; height:30px; margin:9px 0 0 17px; padding:0; border:0",
                    onclick: move |_| hits += 1,
                    "go"
                }
            }
        }
        div { id: "hits", "{hits}" }
    }
}

#[test]
fn a_click_at_the_centre_of_a_child_under_padding_reaches_the_child() {
    let mut harness = Harness::new(Padded, VIEW);
    let child = harness.rect("#child").expect("the child");
    // 8 body margin + 7 margin + 3 border + 23 padding + 13 padding + 17 margin.
    assert_eq!(child.origin.x.0, 8.0 + 7.0 + 3.0 + 23.0 + 13.0 + 17.0);
    // The body's 8 px top margin and the outer 7 px collapse to 8.
    assert_eq!(child.origin.y.0, 8.0 + 3.0 + 37.0 + 13.0 + 9.0);
    let at = harness.centre("#child").expect("the child's centre");
    harness.send(Input::click(at));
    assert_eq!(harness.text_of("#hits").as_deref(), Some("1"));
}

#[allow(non_snake_case)]
fn Variants() -> Element {
    let mut hits = use_signal(String::new);
    rsx! {
        div { id: "flex", style: "display:flex; padding:20px 30px; gap:10px",
            button { id: "a", style: "padding:6px 10px", onclick: move |_| hits.write().push('a'), "aa" }
            button { id: "b", style: "padding:6px 10px", onclick: move |_| hits.write().push('b'), "bb" }
        }
        div { id: "scroller", style: "padding:15px; height:60px; overflow:auto",
            div { style: "height:200px",
                button { id: "c", style: "padding:4px", onclick: move |_| hits.write().push('c'), "cc" }
            }
        }
        div { id: "hits", "{hits}" }
    }
}

#[test]
fn variants_of_nesting_all_hit_their_own_target() {
    let mut harness = Harness::new(Variants, VIEW);
    for id in ["a", "b", "c"] {
        let at = harness.centre(&format!("#{id}")).expect("target");
        harness.send(Input::click(at));
        let hits = harness.text_of("#hits").unwrap();
        assert!(
            hits.ends_with(id),
            "{id}: hits {hits:?} at {at:?} rect {:?}",
            harness.rect(&format!("#{id}"))
        );
    }
}

#[test]
fn a_scrolled_container_keeps_its_own_rect() {
    let mut harness = Harness::new(Variants, VIEW);
    let before = harness.rect("#scroller").expect("the scroller");
    let inside = harness.centre("#scroller").expect("its centre");
    harness.send(Input::wheel(inside, Px(0.0), Px(40.0)));
    let child = harness.rect("#c").expect("the child");
    assert!(child.origin.y.0 < 150.0, "the content scrolled: {child:?}");
    let after = harness.rect("#scroller").expect("the scroller");
    assert_eq!(
        after, before,
        "scrolling its content does not move the box itself"
    );
}

/// A button inside an inline-block inside a padded, bordered block: Blitz places each atomic inline
/// box in its inline root's content box, so the padding of both roots lies between them and the
/// window.
#[allow(non_snake_case)]
fn Inline() -> Element {
    let mut hits = use_signal(|| 0u32);
    rsx! {
        div { id: "root", style: "padding:24px 30px; border:2px solid black",
            span { id: "box", style: "display:inline-block; padding:4px 6px",
                button { id: "btn", style: "padding:2px 8px", onclick: move |_| hits += 1, "go" }
            }
        }
        div { id: "hits", "{hits}" }
    }
}

#[test]
#[ignore = "Blitz hit test removes an inline root's padding and border before testing its atomic inline boxes, whose locations already include them; fixed in the blitz fork at the next toolchain bump"]
fn an_inline_box_under_a_padded_block_is_read_where_it_is_drawn() {
    let mut harness = Harness::new(Inline, VIEW);
    let inner = harness.rect("#btn").expect("the button");
    // 8 body margin + 2 border + 30 padding of the block, then the inline-block's 6 padding.
    assert_eq!(inner.origin.x.0, 8.0 + 2.0 + 30.0 + 6.0);
    let outer = harness.rect("#box").expect("the inline-block");
    assert_eq!(outer.origin.x.0, 8.0 + 2.0 + 30.0);
    assert!(
        outer.origin.y.0 >= 8.0 + 2.0 + 24.0,
        "below the block's top padding: {outer:?}"
    );
    let at = harness.centre("#btn").expect("the button's centre");
    assert!(
        harness.hits(at, "#btn"),
        "a press at {at:?} targets the button"
    );
    harness.send(Input::click(at));
    assert_eq!(harness.text_of("#hits").as_deref(), Some("1"));
}
