//! Scrolling that feels like the platform it runs on, and a scroll offset a component reads in
//! the frame that draws it.
//!
//! On a Mac the system follows the fingers' lift with momentum of its own, which winit reports as
//! more touchpad events: the window applies it as it comes and adds no glide, boost or fling gain
//! of its own, and a wheel click is the 40 px Chromium and WebKit use, with no acceleration on top
//! of the system's. A component that windows over a scroller's offset (`VirtualList`) is handed
//! the offset the frame draws, not the previous frame's.

use blitz_kit::element_id::ElementId;
use blitz_kit::scroll::doc::{geom, scroller_by_id};
use blitz_kit::scroll::geom::ScrollAxis;
use chordkit::Platform;
use dioxus::prelude::*;
use ds::base::geometry::scroll::Scroll;
use ds::host::document::use_document_host;
use ds::host::gesture::GesturePhase;
use ds::host::phase::Observe;
use ds::prelude::*;
use ds_blitz::{ScrollAnimate, ScrollCmd, use_scroll_handle};
use ds_harness::{Clock, Harness, HarnessConfig, Input, Viewport};
use std::time::Duration;

const VIEW: Viewport = Viewport {
    width: 400,
    height: 400,
    scale_percent: 100,
};

thread_local! {
    static HANDLE: std::cell::RefCell<Option<ds_blitz::ScrollHandle>> = const { std::cell::RefCell::new(None) };
}

const FRAME: Duration = Duration::from_millis(8);

const AT: Point = Point {
    x: Px(100.0),
    y: Px(150.0),
};

/// A list of 3000 px of content in 300, and what a component watching its scroll state reads.
#[allow(non_snake_case)]
fn Page() -> Element {
    let host = use_document_host();
    let handle = use_scroll_handle();
    HANDLE.with(|slot| *slot.borrow_mut() = handle.clone());
    let seen = use_signal(|| None::<Scroll>);
    let mut watch = use_signal(|| None::<ds::host::phase::Watch>);
    let shown = seen().map_or_else(|| "none".to_owned(), |seen| format!("{}", seen.offset.0));
    rsx! {
        div {
            id: "list",
            style: "position:absolute; left:0; top:0; width:200px; height:300px; overflow-y:auto; scrollbar-width:none",
            onmounted: move |mounted: MountedEvent| {
                let observed = host.geometry().observe(&mounted.data(), Observe::Scroll(seen));
                if let ds::host::phase::Observed::Watching(found) = observed {
                    watch.set(Some(found));
                }
            },
            div { style: "height:3000px", "rows" }
        }
        p { id: "seen", {shown} }
    }
}

fn harness(platform: Platform) -> Harness {
    Harness::new(
        Page,
        HarnessConfig::new(VIEW)
            .with_clock(Clock::Virtual)
            .with_platform(platform),
    )
}

fn offset(harness: &Harness) -> f64 {
    harness.with_doc(|doc| {
        let scroller = scroller_by_id(doc, &ElementId::new("list")).expect("laid out");
        geom(doc, scroller, ScrollAxis::Y)
            .expect("has geometry")
            .offset
            .0
    })
}

/// Let `time` pass a frame at a time, as a window's frames come (the virtual clock's advance
/// steps only to the end, and an animation is a closed form of the time since it began).
fn run_for(harness: &mut Harness, time: Duration) {
    for _ in 0..(time.as_millis() / FRAME.as_millis()) {
        harness.advance(FRAME);
    }
}

fn fingers(harness: &mut Harness, dy: f32, phase: GesturePhase) {
    harness.send(Input::fingers(AT, Px(0.0), Px(dy), phase));
    harness.advance(FRAME);
}

/// A flick of `steps` strokes of 30 px up the page (winit's sign: negative), lifted.
fn flick(harness: &mut Harness, steps: usize) -> f64 {
    for n in 0..steps {
        let phase = if n == 0 {
            GesturePhase::Began
        } else {
            GesturePhase::Changed
        };
        fingers(harness, -30.0, phase);
    }
    fingers(harness, 0.0, GesturePhase::Ended);
    30.0 * steps as f64
}

#[test]
fn on_a_mac_the_systems_momentum_is_the_whole_tail() {
    let mut harness = harness(Platform::MacOs);
    let flicked = flick(&mut harness, 8);
    // The system's momentum: Started, Moved.., Ended, decaying pixel deltas.
    let tail: Vec<f32> = (0..40).map(|n| 14.0 * 0.92f32.powi(n)).collect();
    for (n, dy) in tail.iter().enumerate() {
        let phase = if n == 0 {
            GesturePhase::Began
        } else {
            GesturePhase::Changed
        };
        let before = offset(&harness);
        harness.send(Input::fingers(AT, Px(0.0), Px(-dy), phase));
        assert!(
            (offset(&harness) - before - f64::from(*dy)).abs() < 1e-3,
            "tail step {n} moves exactly its delta"
        );
        harness.advance(FRAME);
    }
    fingers(&mut harness, 0.0, GesturePhase::Ended);
    let want = flicked + tail.iter().map(|d| f64::from(*d)).sum::<f64>();
    run_for(&mut harness, Duration::from_secs(3));
    assert!(
        (offset(&harness) - want).abs() < 1e-2,
        "{} vs {want}",
        offset(&harness)
    );
}

#[test]
fn on_a_mac_a_lift_the_system_sends_no_momentum_for_still_glides() {
    let mut harness = harness(Platform::MacOs);
    let flicked = flick(&mut harness, 8);
    run_for(&mut harness, Duration::from_secs(3));
    assert!(
        offset(&harness) > flicked + 100.0,
        "the engine glides when the system does not: {}",
        offset(&harness)
    );
}

#[test]
fn off_a_mac_the_lift_glides_at_once() {
    let mut harness = harness(Platform::Linux {
        desktop: chordkit::Desktop::Ours,
    });
    let flicked = flick(&mut harness, 8);
    run_for(&mut harness, Duration::from_millis(40));
    assert!(offset(&harness) > flicked + 1.0, "{}", offset(&harness));
}

#[test]
fn a_mac_wheel_click_is_40_px_and_elsewhere_60() {
    // name, platform, clicks, the offset after the ease
    let cases = [
        ("a mac", Platform::MacOs, 40.0),
        (
            "elsewhere",
            Platform::Linux {
                desktop: chordkit::Desktop::Ours,
            },
            60.0,
        ),
    ];
    for (name, platform, want) in cases {
        let mut harness = harness(platform);
        harness.send(Input::detents(AT, 0.0, -1.0));
        run_for(&mut harness, Duration::from_millis(400));
        assert!(
            (offset(&harness) - want).abs() < 1e-6,
            "{name}: {}",
            offset(&harness)
        );
    }
}

#[test]
fn a_mac_wheel_adds_no_acceleration_to_the_systems_own() {
    let mut harness = harness(Platform::MacOs);
    for _ in 0..30 {
        harness.send(Input::detents(AT, 0.0, -1.0));
        harness.advance(Duration::from_millis(10));
    }
    run_for(&mut harness, Duration::from_millis(400));
    assert!(
        (offset(&harness) - 30.0 * 40.0).abs() < 1e-6,
        "{}",
        offset(&harness)
    );
}

#[test]
fn a_component_reads_the_offset_of_the_frame_that_draws_it() {
    let mut harness = harness(Platform::MacOs);
    harness.advance(Duration::from_millis(50));
    assert_eq!(harness.text_of("#seen").as_deref(), Some("0"));
    // A window's handle wakes a frame for a command; the harness's wakes nothing, so a key owes
    // the frame in which the queued jump runs.
    HANDLE.with(|slot| {
        slot.borrow()
            .as_ref()
            .expect("the page rendered")
            .send(ScrollCmd::To {
                element: ElementId::new("list"),
                axis: ScrollAxis::Y,
                offset: 300.0,
                animate: ScrollAnimate::Instant,
            });
    });
    harness.key_pending(ShortcutKey::Char('x'));
    let mut frames = Vec::new();
    while harness.step_frame() {
        frames.push((offset(&harness), harness.text_of("#seen")));
        assert!(frames.len() < 6, "frames never ran out: {frames:?}");
    }
    assert!(
        frames.iter().any(|(offset, _)| *offset == 300.0),
        "a frame draws the jump: {frames:?}"
    );
    for (offset, seen) in &frames {
        assert_eq!(
            seen.as_deref(),
            Some(format!("{offset}").as_str()),
            "every frame shows the offset it draws: {frames:?}"
        );
    }
}
