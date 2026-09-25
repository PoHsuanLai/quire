//! What a link in a frame says, for mailo's link honesty check and its link pill: a click
//! carries the anchor's visible text (whitespace-collapsed) and title beside its destination,
//! and with `FrameLinks::with_hover` the app hears the pointer come onto and leave each link,
//! once per crossing, at the pointer's point in the app's document.

use dioxus::prelude::*;
use ds::{Point, Px};
use ds_native::{
    FrameLink, FrameLinkHover, FrameLinks, FrameTag, Harness, HarnessConfig, HoverPhase, Viewport,
};
use std::sync::{Arc, Mutex, PoisonError};
use std::time::Duration;

const VIEW: Viewport = Viewport {
    width: 400,
    height: 400,
    scale_percent: 100,
};

const BODY: &str = "<body style='margin:0; font-size:30px'>\
    <a class='bank' href='https://bank.example.net/login' title='Sign in' style='display:block'>\
      Your\n   bank </a>\
    <p class='plain' style='margin:0'>Nothing here</p>\
    <a class='offer' href='https://example.org/offer' style='display:block'>Offer</a>\
    </body>";

#[allow(non_snake_case)]
fn Reader() -> Element {
    rsx! {
        div { class: "above", style: "height: 40px", "Subject" }
        iframe {
            class: "body",
            "data-frame-tag": "msg-7",
            srcdoc: BODY,
            style: "display: block; margin-left: 20px; width: 300px; height: 200px; border: 0",
        }
    }
}

/// Everything the app heard.
#[derive(Default)]
struct Heard {
    clicks: Mutex<Vec<FrameLink>>,
    hovers: Mutex<Vec<FrameLinkHover>>,
}

impl Heard {
    fn clicks(&self) -> Vec<FrameLink> {
        self.clicks
            .lock()
            .unwrap_or_else(PoisonError::into_inner)
            .clone()
    }

    fn hovers(&self) -> Vec<FrameLinkHover> {
        self.hovers
            .lock()
            .unwrap_or_else(PoisonError::into_inner)
            .clone()
    }
}

/// The reader, with the app listening to clicks and hovers.
fn reader() -> (Harness, Arc<Heard>) {
    let heard = Arc::new(Heard::default());
    let (clicks, hovers) = (Arc::clone(&heard), Arc::clone(&heard));
    let links = FrameLinks::intercept(move |link| {
        clicks
            .clicks
            .lock()
            .unwrap_or_else(PoisonError::into_inner)
            .push(link);
    })
    .with_hover(move |hover| {
        hovers
            .hovers
            .lock()
            .unwrap_or_else(PoisonError::into_inner)
            .push(hover);
    });
    let config = HarnessConfig::new(VIEW).with_frame_links(links);
    (Harness::with_config(Reader, config), heard)
}

/// The centre of `selector` in the frame, in the app document's coordinates.
fn at(harness: &Harness, selector: &str) -> Point {
    harness
        .frame("iframe.body")
        .and_then(|frame| frame.centre(selector))
        .unwrap_or_else(|| panic!("{selector} in the frame"))
}

/// `point` moved right by `dx`.
fn nudged(point: Point, dx: f32) -> Point {
    Point {
        x: Px(point.x.0 + dx),
        y: point.y,
    }
}

#[test]
fn a_click_carries_the_links_text_and_title() {
    let (mut harness, heard) = reader();
    let bank = at(&harness, "a.bank");
    harness.click(bank);
    let offer = at(&harness, "a.offer");
    harness.click(offer);
    harness.advance(Duration::from_millis(50));
    let said: Vec<_> = heard
        .clicks()
        .into_iter()
        .map(|link| (link.href, link.text, link.title))
        .collect();
    assert_eq!(
        said,
        vec![
            (
                "https://bank.example.net/login".to_owned(),
                "Your bank".to_owned(),
                Some("Sign in".to_owned())
            ),
            (
                "https://example.org/offer".to_owned(),
                "Offer".to_owned(),
                None
            ),
        ]
    );
}

#[test]
fn moving_onto_a_link_reports_enter_once_and_moving_off_reports_leave() {
    let (mut harness, heard) = reader();
    let bank = at(&harness, "a.bank");
    let frame = harness.frame("iframe.body").expect("the frame").id();
    harness.pointer_move(bank);
    harness.pointer_move(nudged(bank, 5.0));
    harness.pointer_move(nudged(bank, -5.0));
    let plain = at(&harness, "p.plain");
    harness.pointer_move(plain);
    let hovers = heard.hovers();
    let enter = FrameLinkHover {
        frame,
        tag: Some(FrameTag::new("msg-7")),
        href: "https://bank.example.net/login".to_owned(),
        text: "Your bank".to_owned(),
        title: Some("Sign in".to_owned()),
        at: bank,
        phase: HoverPhase::Enter,
    };
    let leave = FrameLinkHover {
        at: plain,
        phase: HoverPhase::Leave,
        ..enter.clone()
    };
    assert_eq!(hovers, vec![enter, leave]);
}

#[test]
fn moving_from_one_link_to_another_leaves_then_enters() {
    let (mut harness, heard) = reader();
    let bank = at(&harness, "a.bank");
    let offer = at(&harness, "a.offer");
    harness.pointer_move(bank);
    harness.pointer_move(offer);
    let above = harness.centre("div.above").expect("the app's own text");
    harness.pointer_move(above);
    let crossings: Vec<_> = heard
        .hovers()
        .into_iter()
        .map(|hover| (hover.phase, hover.text, hover.at))
        .collect();
    assert_eq!(
        crossings,
        vec![
            (HoverPhase::Enter, "Your bank".to_owned(), bank),
            (HoverPhase::Leave, "Your bank".to_owned(), offer),
            (HoverPhase::Enter, "Offer".to_owned(), offer),
            (HoverPhase::Leave, "Offer".to_owned(), above),
        ]
    );
}

#[test]
fn inert_links_report_no_hover() {
    let heard = Arc::new(Heard::default());
    let log = Arc::clone(&heard);
    let links = FrameLinks::Inert.with_hover(move |hover| {
        log.hovers
            .lock()
            .unwrap_or_else(PoisonError::into_inner)
            .push(hover);
    });
    let mut harness =
        Harness::with_config(Reader, HarnessConfig::new(VIEW).with_frame_links(links));
    let bank = at(&harness, "a.bank");
    harness.pointer_move(bank);
    let plain = at(&harness, "p.plain");
    harness.pointer_move(plain);
    assert_eq!(heard.hovers(), Vec::new());
}
