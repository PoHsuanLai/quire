//! A wheel or a touchpad over an `<iframe srcdoc>` scrolls the frame's own document by the
//! engine's rules, eased like any scroller; at the frame's end the scroller around the frame
//! takes over; and a wheel over nothing that scrolls goes on to the document.

use blitz_kit::element_id::ElementId;
use blitz_kit::scroll::doc::{geom, scroller_by_id};
use blitz_kit::scroll::geom::ScrollAxis;
use dioxus::prelude::*;
use ds::host::gesture::GesturePhase;
use ds::prelude::*;
use ds_harness::{Clock, DocQuery, Driver, Harness, HarnessConfig, Input, Viewport};
use std::cell::RefCell;
use std::time::Duration;

const VIEW: Viewport = Viewport {
    width: 400,
    height: 400,
    scale_percent: 100,
};

const FRAME: Duration = Duration::from_millis(8);

/// A frame whose document is 600 px tall in a 150 px window.
const BODY: &str =
    "<html><body style='margin:0'><div style='height:600px'>text</div></body></html>";

thread_local! {
    /// The raw wheels the plain box heard, per thread: each test runs its harness on its own.
    static HEARD: RefCell<u32> = const { RefCell::new(0) };
}

/// A 300 px scroller holding the frame and 1000 px more below it, and a box beside it that
/// scrolls nothing but listens for the wheel.
#[allow(non_snake_case)]
fn Page() -> Element {
    rsx! {
        div { style: "position:relative; width:380px; height:380px; margin:0",
            div {
                id: "outer",
                style: "position:absolute; left:0; top:0; width:300px; height:300px; overflow-y:auto",
                iframe { srcdoc: BODY, style: "display:block; width:300px; height:150px; border:0" }
                div { style: "height:1000px", "rows" }
            }
            div {
                id: "plain",
                style: "position:absolute; left:320px; top:0; width:70px; height:100px",
                onwheel: |_| HEARD.with(|count| *count.borrow_mut() += 1),
                "still"
            }
        }
    }
}

fn harness() -> Harness {
    HEARD.with(|count| *count.borrow_mut() = 0);
    Harness::new(Page, HarnessConfig::new(VIEW).with_clock(Clock::Virtual))
}

fn outer(harness: &Harness) -> f64 {
    harness.with_doc(|doc| {
        let scroller = scroller_by_id(doc, &ElementId::new("outer")).expect("laid out");
        geom(doc, scroller, ScrollAxis::Y)
            .expect("geometry")
            .offset
            .0
    })
}

/// How far the frame's document is scrolled.
fn frame(harness: &Harness) -> f64 {
    harness.with_doc(|doc| {
        let iframe = doc
            .query_selector("iframe")
            .ok()
            .flatten()
            .expect("an iframe");
        let sub = doc
            .get_node(iframe)
            .and_then(|n| n.subdoc())
            .expect("a frame");
        sub.inner().viewport_scroll().y
    })
}

fn over_frame(harness: &Harness) -> Point {
    harness.centre("iframe").expect("laid out")
}

#[test]
fn a_wheel_over_a_frame_scrolls_its_document_eased() {
    let mut harness = harness();
    let at = over_frame(&harness);
    harness.send(Input::detents(at, 0.0, -1.0));
    let mut seen = vec![frame(&harness)];
    for _ in 0..30 {
        harness.advance(FRAME);
        seen.push(frame(&harness));
    }
    let moving = seen.iter().filter(|y| **y > 0.0 && **y < 60.0).count();
    assert!(moving >= 3, "eased over frames, not a jump: {seen:?}");
    assert_eq!(*seen.last().expect("frames"), 60.0, "one detent is 60 px");
    assert_eq!(
        outer(&harness),
        0.0,
        "the frame took it, not the outer scroller"
    );
}

#[test]
fn fingers_over_a_frame_scroll_its_document() {
    let mut harness = harness();
    let at = over_frame(&harness);
    for (n, step) in [-20.0, -20.0, -20.0].into_iter().enumerate() {
        let phase = match n {
            0 => GesturePhase::Began,
            _ => GesturePhase::Changed,
        };
        harness.send(Input::fingers(at, Px(0.0), Px(step), phase));
        harness.advance(FRAME);
    }
    harness.send(Input::fingers(at, Px(0.0), Px(0.0), GesturePhase::Ended));
    harness.advance(Duration::from_millis(500));
    assert!(
        frame(&harness) >= 50.0,
        "the frame moved: {}",
        frame(&harness)
    );
    assert_eq!(outer(&harness), 0.0);
}

#[test]
fn at_the_frames_end_the_outer_scroller_moves() {
    let mut harness = harness();
    let at = over_frame(&harness);
    // 600 - 150 = 450 px of frame to scroll; eight detents of 60 px carry it all the way.
    for _ in 0..8 {
        harness.send(Input::detents(at, 0.0, -1.0));
        harness.advance(Duration::from_millis(400));
    }
    assert_eq!(frame(&harness), 450.0, "the frame is at its end");
    assert_eq!(outer(&harness), 0.0, "and the outer scroller waited");
    harness.send(Input::detents(at, 0.0, -1.0));
    harness.advance(Duration::from_millis(400));
    assert_eq!(frame(&harness), 450.0);
    assert_eq!(outer(&harness), 60.0, "the next detent chained out");
}

#[test]
fn a_wheel_over_nothing_that_scrolls_goes_to_the_document() {
    let mut harness = harness();
    let at = harness.centre("#plain").expect("laid out");
    harness.send(Input::detents(at, 0.0, -1.0));
    assert_eq!(
        HEARD.with(|count| *count.borrow()),
        1,
        "the document heard it"
    );
    let on_frame = over_frame(&harness);
    harness.send(Input::detents(on_frame, 0.0, -1.0));
    assert_eq!(
        HEARD.with(|count| *count.borrow()),
        1,
        "a scroller took the other"
    );
}

#[test]
fn fingers_over_nothing_that_scrolls_go_to_the_document_too() {
    let mut harness = harness();
    let at = harness.centre("#plain").expect("laid out");
    harness.send(Input::fingers(at, Px(0.0), Px(-20.0), GesturePhase::Began));
    assert_eq!(
        HEARD.with(|count| *count.borrow()),
        1,
        "the document heard it"
    );
}
