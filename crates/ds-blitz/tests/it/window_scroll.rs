//! A window scrolls its containers by design/11's rules (the engine of `blitz_kit::scroll`), not
//! by Blitz's own 20 px a line in one jump: a wheel detent is 60 px eased over at most 200 ms,
//! detents accumulate, an element that takes the wheel (`data-wheel="capture"`) is left alone,
//! the page key moves `max(0.8 v, v - 40)`, and a component that moves its own content can ask
//! for the same ease of the detents it hears. The harness runs the window's scroll step before
//! every frame it lays out, as the window loop does, on its virtual clock.

use blitz_kit::element_id::ElementId;
use blitz_kit::scroll::doc::{geom, scroller_by_id};
use blitz_kit::scroll::geom::ScrollAxis;
use dioxus::prelude::*;
use ds::host::gesture::{Gesture, WheelDelivery, use_gestures, use_gestures_with};
use ds::prelude::*;
use ds_blitz::{ScrollAnimate, ScrollCmd, use_scroll_handle};
use ds_harness::{Clock, DocQuery, Driver, Harness, HarnessConfig, Input, Viewport};
use std::cell::RefCell;
use std::time::Duration;

const VIEW: Viewport = Viewport {
    width: 400,
    height: 400,
    scale_percent: 100,
};

/// One frame, as the window's 120 Hz at its slowest.
const FRAME: Duration = Duration::from_millis(8);

thread_local! {
    /// The raw wheels the capturing element heard, and the gestures each kind of listener heard:
    /// per thread, as each test runs its harness on its own.
    static CAPTURED: RefCell<u32> = const { RefCell::new(0) };
    static AS_RECEIVED: RefCell<Vec<f32>> = const { RefCell::new(Vec::new()) };
    static EASED: RefCell<Vec<f32>> = const { RefCell::new(Vec::new()) };
}

/// A list that scrolls 3000 px of content in 300 px, a capturing element beside it that takes the
/// wheel itself, and listeners for the wheel's gestures.
#[allow(non_snake_case)]
fn Page() -> Element {
    use_gestures(|gesture| {
        if let Gesture::Scroll { by, .. } = gesture {
            AS_RECEIVED.with(|heard| heard.borrow_mut().push(by.y.0));
        }
    });
    use_gestures_with(WheelDelivery::Eased, |gesture| {
        if let Gesture::Scroll { by, .. } = gesture {
            EASED.with(|heard| heard.borrow_mut().push(by.y.0));
        }
    });
    let handle = use_scroll_handle();
    rsx! {
        div { style: "position:relative; width:400px; height:400px; margin:0",
            div {
                id: "list",
                style: "position:absolute; left:0; top:0; width:200px; height:300px; overflow-y:auto",
                div { style: "height:3000px", "rows" }
            }
            div {
                id: "slider",
                "data-wheel": "capture",
                style: "position:absolute; left:220px; top:0; width:150px; height:300px; overflow-y:auto",
                onwheel: |event| {
                    event.prevent_default();
                    CAPTURED.with(|count| *count.borrow_mut() += 1);
                },
                div { style: "height:3000px", "values" }
            }
            button {
                id: "jump",
                onclick: move |_| {
                    if let Some(handle) = &handle {
                        handle.send(ScrollCmd::To {
                            element: ElementId::new("list"),
                            axis: ScrollAxis::Y,
                            offset: 500.0,
                            animate: ScrollAnimate::Smooth,
                        });
                    }
                },
                style: "position:absolute; left:0; top:340px; width:80px; height:30px",
                "jump"
            }
        }
    }
}

fn harness() -> Harness {
    CAPTURED.with(|count| *count.borrow_mut() = 0);
    AS_RECEIVED.with(|heard| heard.borrow_mut().clear());
    EASED.with(|heard| heard.borrow_mut().clear());
    Harness::new(Page, HarnessConfig::new(VIEW).with_clock(Clock::Virtual))
}

/// The vertical offset of the element with `id`.
fn offset(harness: &Harness, id: &str) -> f64 {
    harness.with_doc(|doc| {
        let scroller = scroller_by_id(doc, &ElementId::new(id)).expect("laid out");
        geom(doc, scroller, ScrollAxis::Y)
            .expect("has geometry")
            .offset
            .0
    })
}

/// The offset after each of `frames` frames of 8 ms, starting with the one now.
fn trace(harness: &mut Harness, id: &str, frames: usize) -> Vec<f64> {
    let mut seen = vec![offset(harness, id)];
    for _ in 0..frames {
        harness.advance(FRAME);
        seen.push(offset(harness, id));
    }
    seen
}

fn over(harness: &Harness, selector: &str) -> Point {
    harness.centre(selector).expect("laid out")
}

#[test]
fn one_wheel_detent_moves_a_container_60_px_over_frames() {
    let mut harness = harness();
    let at = over(&harness, "#list");
    assert_eq!(offset(&harness, "list"), 0.0);
    // winit's sign: a click that scrolls the page down is -1.
    harness.send(Input::detents(at, 0.0, -1.0));
    let seen = trace(&mut harness, "list", 30);
    assert!(
        seen[0].abs() < 1e-6,
        "nothing moves before the first frame: {}",
        seen[0]
    );
    let moving = seen.iter().filter(|x| **x > 0.0 && **x < 60.0).count();
    assert!(moving >= 3, "intermediate frames, not a jump: {seen:?}");
    assert!(
        seen.windows(2).all(|pair| pair[1] >= pair[0]),
        "it only moves forward: {seen:?}"
    );
    let arrived = seen.iter().position(|x| *x == 60.0).expect("reaches 60 px");
    let by = Duration::from_millis(200);
    assert!(
        FRAME * u32::try_from(arrived).expect("small") <= by,
        "within 200 ms: frame {arrived}"
    );
    assert_eq!(*seen.last().expect("frames"), 60.0, "and stops there");
}

#[test]
fn detents_in_a_burst_add_up_and_a_reversed_one_takes_back() {
    let mut harness = harness();
    let at = over(&harness, "#list");
    for _ in 0..3 {
        harness.send(Input::detents(at, 0.0, -1.0));
        harness.advance(FRAME);
    }
    harness.advance(Duration::from_millis(300));
    // Three clicks 8 ms apart are a spun wheel, which carries each further than 60 px.
    let three = offset(&harness, "list");
    assert!(three >= 180.0, "three detents: {three}");
    harness.send(Input::detents(at, 0.0, 1.0));
    harness.advance(Duration::from_millis(300));
    assert_eq!(offset(&harness, "list"), three - 60.0, "one back");
}

#[test]
fn an_element_that_takes_the_wheel_is_left_alone_by_the_engine() {
    let mut harness = harness();
    let at = over(&harness, "#slider");
    harness.send(Input::detents(at, 0.0, -1.0));
    let seen = trace(&mut harness, "slider", 30);
    assert!(
        seen.iter().all(|x| *x == 0.0),
        "the engine did not scroll it: {seen:?}"
    );
    assert_eq!(offset(&harness, "list"), 0.0);
    assert_eq!(
        CAPTURED.with(|count| *count.borrow()),
        1,
        "it heard the raw wheel (and kept Blitz from scrolling)"
    );
}

#[test]
fn page_down_moves_a_page_of_the_container_under_the_pointer() {
    let mut harness = harness();
    harness.send(Input::pointer_move(over(&harness, "#list")));
    harness.send(Input::key(ShortcutKey::PageDown));
    let seen = trace(&mut harness, "list", 30);
    // The list is 300 px tall: max(0.8 * 300, 300 - 40) = 260.
    assert_eq!(*seen.last().expect("frames"), 260.0);
    let moving = seen.iter().filter(|x| **x > 0.0 && **x < 260.0).count();
    assert!(moving >= 3, "eased over frames: {seen:?}");
    let arrived = seen.iter().position(|x| *x == 260.0).expect("260 px");
    assert!(
        FRAME * u32::try_from(arrived).expect("small") <= Duration::from_millis(208),
        "a page takes 200 ms at most: frame {arrived}"
    );
    harness.send(Input::key(ShortcutKey::Down));
    harness.advance(Duration::from_millis(100));
    assert_eq!(offset(&harness, "list"), 300.0, "an arrow is a 40 px line");
}

#[test]
fn a_scroll_command_runs_through_the_engine_by_element_id() {
    let mut harness = harness();
    harness.send(Input::click(over(&harness, "#jump")));
    let seen = trace(&mut harness, "list", 40);
    assert_eq!(*seen.last().expect("frames"), 500.0);
    let moving = seen.iter().filter(|x| **x > 0.0 && **x < 500.0).count();
    assert!(moving >= 3, "smooth, not a jump: {seen:?}");
}

#[test]
fn a_listener_that_asks_gets_the_detents_eased_and_the_others_get_them_whole() {
    let mut harness = harness();
    let at = over(&harness, "#list");
    harness.send(Input::detents(at, 0.0, -1.0));
    trace(&mut harness, "list", 40);
    assert_eq!(
        AS_RECEIVED.with(|heard| heard.borrow().clone()),
        vec![-60.0],
        "one detent, 60 px of content motion, in one event"
    );
    let eased = EASED.with(|heard| heard.borrow().clone());
    assert!(eased.len() >= 3, "a share per frame: {eased:?}");
    assert!(
        eased.iter().all(|by| *by < 0.0),
        "toward the detent: {eased:?}"
    );
    let total: f32 = eased.iter().sum();
    assert!(
        (total + 60.0).abs() < 1e-3,
        "summing to the detent: {total}"
    );
}

/// `clicks` wheel clicks `gap` apart, then the frames until everything has settled; the list's
/// final offset.
fn spun(clicks: u32, gap: Duration) -> f64 {
    let mut harness = harness();
    let at = over(&harness, "#list");
    for _ in 0..clicks {
        harness.send(Input::detents(at, 0.0, -1.0));
        harness.advance(gap);
    }
    harness.advance(Duration::from_millis(400));
    offset(&harness, "list")
}

#[test]
fn a_wheel_read_at_a_slow_pace_moves_60_px_a_click() {
    // Four clicks a second: not spun.
    let end = spun(5, Duration::from_millis(250));
    assert!((end - 300.0).abs() < 1.0, "{end}");
}

#[test]
fn a_spun_wheel_carries_each_click_further_than_60_px() {
    // Twenty clicks a second for a second: past the 1200 px unaccelerated.
    let end = spun(20, Duration::from_millis(48));
    assert!(end > 1800.0, "a spin of 20 clicks moved only {end}");
    // And a pause between spins starts afresh: one click back is 60 px again.
    let mut harness = harness();
    let at = over(&harness, "#list");
    for _ in 0..20 {
        harness.send(Input::detents(at, 0.0, -1.0));
        harness.advance(Duration::from_millis(48));
    }
    harness.advance(Duration::from_millis(600));
    let before = offset(&harness, "list");
    harness.send(Input::detents(at, 0.0, 1.0));
    harness.advance(Duration::from_millis(300));
    let step = before - offset(&harness, "list");
    assert!((step - 60.0).abs() < 1.0, "{step}");
}
