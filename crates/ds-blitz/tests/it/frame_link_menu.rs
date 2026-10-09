//! The link under a context menu inside a frame, for mailo's "Copy Link": with
//! `FrameLinks::with_context_menu` the app hears a secondary press released over a link in a
//! frame, with the link's target resolved against the frame's base URL, its text, its title and the
//! pointer's point; nothing for a press off every link, for a primary click, or when the
//! frames' links are inert. The press still reaches the page's own `oncontextmenu`.

use dioxus::prelude::*;
use ds::base::press::PointerButton;
use ds::prelude::*;
use ds_blitz::{FrameLink, FrameLinkMenu, FrameLinks, FrameTag};
use ds_harness::{Clock, Driver, Harness, HarnessConfig, Input, Query, Viewport};
use std::sync::{Arc, Mutex, PoisonError};
use std::time::Duration;

const VIEW: Viewport = Viewport {
    width: 400,
    height: 400,
    scale_percent: 100,
};

const BODY: &str = "<body style='margin:0; font-size:30px'>\
    <a class='bank' href='https://bank.example.net/login?next=%2F' title='Sign in' style='display:block'>\
      <span class='inner'>Your\n   bank</span> </a>\
    <p class='plain' style='margin:0'>Nothing here</p>\
    </body>";

#[allow(non_snake_case)]
fn Reader() -> Element {
    let mut asked = use_signal(|| 0u32);
    rsx! {
        div {
            class: "zone",
            oncontextmenu: move |_| asked += 1,
            iframe {
                class: "body",
                "data-frame-tag": "msg-7",
                srcdoc: BODY,
                style: "display: block; margin-left: 20px; width: 300px; height: 200px; border: 0",
            }
        }
        for _ in 0..asked() {
            i { class: "asked" }
        }
    }
}

#[derive(Default)]
struct Heard {
    menus: Mutex<Vec<FrameLinkMenu>>,
}

impl Heard {
    fn menus(&self) -> Vec<FrameLinkMenu> {
        self.menus
            .lock()
            .unwrap_or_else(PoisonError::into_inner)
            .clone()
    }
}

fn reader(links: FrameLinks) -> (Harness, Arc<Heard>) {
    let heard = Arc::new(Heard::default());
    let log = Arc::clone(&heard);
    let links = match links {
        FrameLinks::Inert => links.with_context_menu(|_| {}),
        links => links.with_context_menu(move |menu| {
            log.menus
                .lock()
                .unwrap_or_else(PoisonError::into_inner)
                .push(menu);
        }),
    };
    let config = HarnessConfig::new(VIEW)
        .with_clock(Clock::Virtual)
        .with_frame_links(links);
    (Harness::new(Reader, config), heard)
}

fn listening() -> (Harness, Arc<Heard>) {
    reader(FrameLinks::intercept(|_: FrameLink| {}))
}

fn at(harness: &Harness, selector: &str) -> Point {
    harness
        .frame("iframe.body")
        .and_then(|frame| frame.centre(selector))
        .unwrap_or_else(|| panic!("{selector} in the frame"))
}

fn right_click(harness: &mut Harness, at: Point) {
    harness.send(Input::pointer_move(at));
    harness.send(Input::button_down(at, PointerButton::Secondary));
    harness.send(Input::button_up(at, PointerButton::Secondary));
    harness.advance(Duration::from_millis(50));
}

#[test]
fn a_right_click_on_a_link_in_a_frame_reports_the_link_under_it() {
    let (mut harness, heard) = listening();
    // The word sits inside the anchor: the nearest enclosing `a[href]` is the link
    let word = at(&harness, "span.inner");
    let frame = harness.frame("iframe.body").expect("the frame").id();
    right_click(&mut harness, word);
    let want = FrameLinkMenu {
        frame,
        tag: Some(FrameTag::new("msg-7")),
        href: "https://bank.example.net/login?next=%2F".to_owned(),
        text: "Your bank".to_owned(),
        title: Some("Sign in".to_owned()),
        at: word,
    };
    assert_eq!(heard.menus(), vec![want]);
    assert_eq!(
        harness.count(".asked"),
        1,
        "the page's own menu still asked"
    );
}

#[test]
fn a_right_click_off_every_link_reports_nothing() {
    let (mut harness, heard) = listening();
    let plain = at(&harness, "p.plain");
    right_click(&mut harness, plain);
    assert_eq!(heard.menus(), Vec::new());
    assert_eq!(harness.count(".asked"), 1);
}

#[test]
fn a_left_click_on_a_link_reports_no_menu() {
    let (mut harness, heard) = listening();
    let bank = at(&harness, "a.bank");
    harness.send(Input::click(bank));
    assert_eq!(heard.menus(), Vec::new());
}
