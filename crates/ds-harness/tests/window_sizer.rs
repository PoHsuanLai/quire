//! A component that sizes its window runs in the harness against a window that records what it
//! is asked and answers as the test says (`HarnessConfig::with_sizer_ack`): at once, after a
//! delay, or never. Every harness has a window sizer, so `use_window_sizer()` is `Some`.

use dioxus::prelude::*;
use ds_blitz::{
    Extent, Reserve, ScreenArea, ScreenOf, SizeOrigin, SizeRequest, WindowSizer, WorkBasis,
    use_window_sizer,
};
use ds_harness::{
    Clock, Driver, Harness, HarnessConfig, Input, Query, SizerAck, Viewport, WindowScreen,
};
use std::time::Duration;

const VIEW: Viewport = Viewport {
    width: 800,
    height: 600,
    scale_percent: 100,
};

const ASKED: Extent = Extent::new(900, 700);
const THEN: Extent = Extent::new(1000, 800);

fn ms(n: u64) -> Duration {
    Duration::from_millis(n)
}

/// Two buttons that each ask for a size, and what the sizer says about the request and the
/// origin, drawn so a re-render is visible.
#[allow(non_snake_case)]
fn Sizing() -> Element {
    let Some(sizer) = use_window_sizer() else {
        return rsx! { p { id: "sizer", "absent" } };
    };
    let first = sizer.clone();
    let second = sizer.clone();
    let request = match sizer.request() {
        SizeRequest::Idle => "idle",
        SizeRequest::Pending(_) => "pending",
        SizeRequest::Expired(_) => "expired",
    };
    let origin = match sizer.origin() {
        None => "none",
        Some(SizeOrigin::Requested) => "requested",
        Some(SizeOrigin::Person) => "person",
    };
    rsx! {
        p { id: "sizer", "present" }
        p { id: "request", "{request}" }
        p { id: "origin", "{origin}" }
        button {
            id: "ask",
            style: "position:absolute; left:0; top:0; width:80px; height:30px",
            onclick: move |_| first.request_size(ASKED),
            "ask"
        }
        button {
            id: "ask-again",
            style: "position:absolute; left:100px; top:0; width:80px; height:30px",
            onclick: move |_| second.request_size(THEN),
            "again"
        }
    }
}

fn harness(ack: SizerAck) -> Harness {
    Harness::new(
        Sizing,
        HarnessConfig::new(VIEW)
            .with_clock(Clock::Virtual)
            .with_sizer_ack(ack),
    )
}

fn click(harness: &mut Harness, id: &str) {
    let at = harness.centre(id).expect("laid out");
    harness.send(Input::click(at));
}

/// What the component shows: the request state, then the origin.
fn shown(harness: &Harness) -> (String, String) {
    let text = |id: &str| harness.text_of(id).expect("drawn");
    (text("#request"), text("#origin"))
}

fn state(request: &str, origin: &str) -> (String, String) {
    (request.to_owned(), origin.to_owned())
}

#[test]
fn every_harness_has_a_window_sizer_that_starts_at_the_viewport() {
    let harness = harness(SizerAck::Now);
    assert_eq!(harness.text_of("#sizer").as_deref(), Some("present"));
    assert_eq!(harness.window_size(), Extent::new(800, 600));
    assert_eq!(shown(&harness), state("idle", "none"));
    assert_eq!(harness.window_requests(), Vec::<Extent>::new());
}

#[test]
fn an_ack_now_answers_on_the_next_turn_not_inside_the_request() {
    let mut harness = harness(SizerAck::Now);
    let sizer = harness.window_sizer();
    harness.within(|| sizer.request_size(ASKED));
    assert_eq!(
        sizer.request(),
        SizeRequest::Pending(ASKED),
        "the answer is a resize that comes after the request"
    );
    assert_eq!(sizer.origin(), None);
    harness.advance(Duration::ZERO);
    assert_eq!(sizer.request(), SizeRequest::Idle);
    assert_eq!(sizer.origin(), Some(SizeOrigin::Requested));
    assert_eq!(harness.window_size(), ASKED);
    assert_eq!(harness.window_requests(), vec![ASKED]);
}

#[test]
fn a_component_hears_the_request_settle_through_its_own_render() {
    let mut harness = harness(SizerAck::Now);
    click(&mut harness, "#ask");
    assert_eq!(shown(&harness), state("idle", "requested"));
    assert_eq!(harness.window_requests(), vec![ASKED]);
}

#[test]
fn an_ack_after_a_delay_is_pending_until_it_arrives() {
    let mut harness = harness(SizerAck::After(ms(200)));
    click(&mut harness, "#ask");
    assert_eq!(shown(&harness), state("pending", "none"));
    harness.advance(ms(150));
    assert_eq!(shown(&harness), state("pending", "none"), "150 ms: not yet");
    harness.advance(ms(100));
    assert_eq!(
        shown(&harness),
        state("idle", "requested"),
        "250 ms: answered"
    );
    assert_eq!(harness.window_size(), ASKED);
}

#[test]
fn a_request_nobody_answers_expires_and_the_origin_stays_as_it_was() {
    let mut harness = harness(SizerAck::Never);
    let sizer = harness.window_sizer();
    click(&mut harness, "#ask");
    harness.advance(ms(400));
    assert_eq!(
        shown(&harness),
        state("pending", "none"),
        "400 ms: still waiting"
    );
    harness.advance(ms(200));
    assert_eq!(
        shown(&harness),
        state("expired", "none"),
        "600 ms: given up"
    );
    assert_eq!(sizer.request(), SizeRequest::Expired(ASKED));
    assert_eq!(
        harness.window_size(),
        Extent::new(800, 600),
        "never resized"
    );
    assert_eq!(harness.window_requests(), vec![ASKED]);
}

#[test]
fn after_an_expired_request_the_persons_resize_is_the_persons_even_at_the_asked_size() {
    let mut harness = harness(SizerAck::Never);
    click(&mut harness, "#ask");
    harness.advance(ms(600));
    harness.resize_window(ASKED);
    assert_eq!(shown(&harness), state("idle", "person"));
}

#[test]
fn an_answer_that_comes_too_late_reads_as_the_persons() {
    let mut harness = harness(SizerAck::After(ms(700)));
    click(&mut harness, "#ask");
    harness.advance(ms(600));
    assert_eq!(shown(&harness), state("expired", "none"));
    harness.advance(ms(200));
    assert_eq!(shown(&harness), state("idle", "person"), "800 ms: late");
}

#[test]
fn an_old_requests_timer_does_not_expire_a_newer_request() {
    let mut harness = harness(SizerAck::Never);
    click(&mut harness, "#ask");
    harness.advance(ms(300));
    click(&mut harness, "#ask-again");
    harness.advance(ms(250));
    assert_eq!(
        shown(&harness),
        state("pending", "none"),
        "550 ms: the first lapsed, the second has 250 ms left"
    );
    harness.advance(ms(300));
    assert_eq!(harness.window_sizer().request(), SizeRequest::Expired(THEN));
    assert_eq!(harness.window_requests(), vec![ASKED, THEN]);
}

#[test]
fn a_person_resizing_with_nothing_asked_reads_as_the_person() {
    let mut harness = harness(SizerAck::Now);
    harness.resize_window(Extent::new(640, 480));
    assert_eq!(shown(&harness), state("idle", "person"));
    assert_eq!(harness.window_size(), Extent::new(640, 480));
}

#[test]
fn the_window_reports_the_output_the_test_chose() {
    let standard = Harness::new(Sizing, HarnessConfig::new(VIEW));
    let area = standard.window_sizer().screen().expect("a standard output");
    assert_eq!(
        (area.output, area.work, area.of, area.basis),
        (
            Extent::new(1920, 1080),
            Extent::new(1920, 1080),
            ScreenOf::Window,
            WorkBasis::WholeOutput
        )
    );

    let hidpi = Viewport {
        scale_percent: 200,
        ..VIEW
    };
    let at_2x = Harness::new(Sizing, HarnessConfig::new(hidpi));
    let scale = at_2x.window_sizer().screen().expect("output").scale;
    assert_eq!(scale, ds::prelude::Scale(240), "the viewport's scale");

    let none = Harness::new(
        Sizing,
        HarnessConfig::new(VIEW).with_window_screen(WindowScreen::Absent),
    );
    assert_eq!(none.window_sizer().screen(), None);

    let panel = ScreenArea::new(
        Extent::new(2560, 1440),
        ds::prelude::Scale(120),
        ScreenOf::Window,
    )
    .expect("scale")
    .less(Reserve {
        top: 32,
        ..Reserve::default()
    });
    let given = Harness::new(
        Sizing,
        HarnessConfig::new(VIEW).with_window_screen(WindowScreen::Area(panel)),
    );
    let sizer: WindowSizer = given.window_sizer();
    assert_eq!(sizer.screen(), Some(panel));
}
