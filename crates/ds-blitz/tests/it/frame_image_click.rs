//! A click on an image inside a frame, for mailo's Peek of an inline image: with
//! `FrameLinks::with_image_click` the app hears a primary release on an `img` in a frame, with its
//! `src` (a `cid:` reference as written), its `alt`, the frame's tag and the pointer's point;
//! nothing for an image inside a link (the link's click), for a press off every image, for a
//! secondary press, or when the frames' links are inert.

use dioxus::prelude::*;
use ds::base::press::PointerButton;
use ds::prelude::*;
use ds_blitz::{FrameImageClick, FrameLink, FrameLinks, FrameTag};
use ds_harness::{Clock, Harness, HarnessConfig, Input, Viewport};
use std::sync::{Arc, Mutex, PoisonError};
use std::time::Duration;

const VIEW: Viewport = Viewport {
    width: 400,
    height: 400,
    scale_percent: 100,
};

const BODY: &str = "<body style='margin:0'>\
    <img class='logo' src='cid:logo@example.org' alt='Logo' style='display:block; width:120px; height:60px'>\
    <a href='https://shop.example.net/' style='display:block'>\
      <img class='linked' src='cid:banner@example.org' style='display:block; width:120px; height:60px'></a>\
    <p class='plain' style='margin:0; font-size:30px'>Nothing here</p>\
    </body>";

#[allow(non_snake_case)]
fn Reader() -> Element {
    rsx! {
        iframe {
            class: "body",
            "data-frame-tag": "msg-7",
            srcdoc: BODY,
            style: "display: block; margin-left: 20px; width: 300px; height: 300px; border: 0",
        }
    }
}

#[derive(Default)]
struct Heard {
    images: Mutex<Vec<FrameImageClick>>,
    links: Mutex<Vec<String>>,
}

impl Heard {
    fn images(&self) -> Vec<FrameImageClick> {
        self.images
            .lock()
            .unwrap_or_else(PoisonError::into_inner)
            .clone()
    }

    fn links(&self) -> Vec<String> {
        self.links
            .lock()
            .unwrap_or_else(PoisonError::into_inner)
            .clone()
    }
}

fn reader(intercept: bool) -> (Harness, Arc<Heard>) {
    let heard = Arc::new(Heard::default());
    let (images, links) = (Arc::clone(&heard), Arc::clone(&heard));
    let base = if intercept {
        FrameLinks::intercept(move |link: FrameLink| {
            links
                .links
                .lock()
                .unwrap_or_else(PoisonError::into_inner)
                .push(link.href);
        })
    } else {
        FrameLinks::default()
    };
    let frame_links = base.with_image_click(move |click| {
        images
            .images
            .lock()
            .unwrap_or_else(PoisonError::into_inner)
            .push(click);
    });
    let config = HarnessConfig::new(VIEW)
        .with_clock(Clock::Virtual)
        .with_frame_links(frame_links);
    (Harness::new(Reader, config), heard)
}

fn at(harness: &Harness, selector: &str) -> Point {
    harness
        .frame("iframe.body")
        .and_then(|frame| frame.centre(selector))
        .unwrap_or_else(|| panic!("{selector} in the frame"))
}

fn click(harness: &mut Harness, at: Point, button: PointerButton) {
    harness.send(Input::pointer_move(at));
    harness.send(Input::button_down(at, button));
    harness.send(Input::button_up(at, button));
    harness.advance(Duration::from_millis(50));
}

#[test]
fn a_click_on_an_image_in_a_frame_reports_its_source_and_the_frame() {
    let (mut harness, heard) = reader(true);
    let logo = at(&harness, "img.logo");
    let frame = harness.frame("iframe.body").expect("the frame").id();
    click(&mut harness, logo, PointerButton::Primary);
    let want = FrameImageClick {
        frame,
        tag: Some(FrameTag::new("msg-7")),
        src: "cid:logo@example.org".to_owned(),
        alt: Some("Logo".to_owned()),
        at: logo,
    };
    assert_eq!(heard.images(), vec![want]);
}

#[test]
fn an_image_inside_a_link_is_the_links_click() {
    let (mut harness, heard) = reader(true);
    let linked = at(&harness, "img.linked");
    click(&mut harness, linked, PointerButton::Primary);
    assert_eq!(heard.images(), Vec::new());
    assert_eq!(heard.links(), vec!["https://shop.example.net/".to_owned()]);
}

#[test]
fn a_click_off_every_image_or_a_secondary_press_reports_nothing() {
    let (mut harness, heard) = reader(true);
    let plain = at(&harness, "p.plain");
    click(&mut harness, plain, PointerButton::Primary);
    let logo = at(&harness, "img.logo");
    click(&mut harness, logo, PointerButton::Secondary);
    assert_eq!(heard.images(), Vec::new());
}

#[test]
fn inert_frames_report_no_image() {
    let (mut harness, heard) = reader(false);
    let logo = at(&harness, "img.logo");
    click(&mut harness, logo, PointerButton::Primary);
    assert_eq!(heard.images(), Vec::new());
}
