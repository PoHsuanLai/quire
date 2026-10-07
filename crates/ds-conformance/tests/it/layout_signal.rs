//! `use_layout` gives a mounted element's rect as a signal the host's frame phase keeps: the rect
//! is there as soon as the document has been laid out, and follows the element when it moves or
//! grows, with no frame waited for and no read retried.

use dioxus::prelude::*;
use ds::host::layout::use_layout;
use ds::host::measure::MountedRef;
use ds_harness::{Clock, Driver, Harness, HarnessConfig, Input, Query, Viewport};

const VIEW: Viewport = Viewport {
    width: 400,
    height: 200,
    scale_percent: 100,
};

/// Where the box starts and how wide it is, as the app set them.
#[derive(Debug, Clone, Copy, PartialEq)]
enum Placed {
    Start,
    Moved,
}

#[allow(non_snake_case)]
fn App() -> Element {
    let mut placed = use_signal(|| Placed::Start);
    let mut node = use_signal(|| None::<MountedRef>);
    let rect = use_layout(node.into());
    let (left, width) = match placed() {
        Placed::Start => (10, 100),
        Placed::Moved => (50, 160),
    };
    let shown = rect().map_or_else(
        || "none".to_owned(),
        |rect| {
            format!(
                "{} {} {} {}",
                rect.origin.x.0, rect.origin.y.0, rect.size.width.0, rect.size.height.0
            )
        },
    );
    rsx! {
        div {
            class: "box",
            style: "position:absolute; top:30px; left:{left}px; width:{width}px; height:20px",
            onmounted: move |event| node.set(Some(MountedRef(event.data()))),
        }
        button {
            class: "move",
            style: "position:absolute; top:100px; left:0; width:40px; height:20px",
            onclick: move |_| placed.set(Placed::Moved),
        }
        p { class: "rect", "{shown}" }
    }
}

/// What the document itself says the box's rect is, in the app's format.
fn laid_out(harness: &Harness) -> Option<String> {
    harness.rect(".box").map(|rect| {
        format!(
            "{} {} {} {}",
            rect.origin.x.0, rect.origin.y.0, rect.size.width.0, rect.size.height.0
        )
    })
}

#[test]
fn the_rect_is_published_after_layout_and_follows_the_element() {
    let mut harness = Harness::new(App, HarnessConfig::new(VIEW).with_clock(Clock::Virtual));
    let start = laid_out(&harness);
    assert!(
        start
            .as_deref()
            .is_some_and(|rect| rect.ends_with(" 100 20"))
    );
    assert_eq!(harness.text_of(".rect"), start);

    let button = harness.centre(".move").expect("the button");
    harness.send(Input::click(button));
    let moved = laid_out(&harness);
    assert!(
        moved
            .as_deref()
            .is_some_and(|rect| rect.ends_with(" 160 20"))
    );
    assert_ne!(moved, start, "the click moved the box");
    assert_eq!(harness.text_of(".rect"), moved);
}
