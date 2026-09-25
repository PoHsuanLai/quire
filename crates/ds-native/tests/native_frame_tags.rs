//! Frame tags: the app names each `iframe` with `data-frame-tag`, and everything its document
//! does (a request put to the app, a link click) carries that name, so mailo never infers which
//! message a frame shows from the URLs it asks for. `ds_native::frames` looks a frame up either
//! way, from the id to the tag and back.

use dioxus::prelude::*;
use ds_native::frames::{frame_by_tag, tag_of};
use ds_native::{
    AppNet, FrameLink, FrameLinks, FrameTag, Harness, HarnessConfig, NetDecision, NetPolicy,
    NetReply, NetRequest, RequestOrigin, Viewport,
};
use std::sync::{Arc, Mutex, PoisonError};
use std::time::Duration;

const VIEW: Viewport = Viewport {
    width: 400,
    height: 400,
    scale_percent: 100,
};

const FIRST: &str = "<body style='margin:0'>\
    <img src='https://example.org/first.png'>\
    <a class='link' href='https://example.org/one' style='display:block; font-size:40px'>One</a>\
    </body>";

const SECOND: &str = "<body style='margin:0'>\
    <img src='https://example.org/second.png'>\
    <a class='link' href='https://example.org/two' style='display:block; font-size:40px'>Two</a>\
    </body>";

#[allow(non_snake_case)]
fn Reader() -> Element {
    rsx! {
        iframe {
            class: "first",
            "data-frame-tag": "msg-1",
            srcdoc: FIRST,
            style: "display: block; width: 300px; height: 150px; border: 0",
        }
        iframe {
            class: "second",
            "data-frame-tag": "msg-2",
            srcdoc: SECOND,
            style: "display: block; width: 300px; height: 150px; border: 0",
        }
    }
}

/// One request as the app saw it.
type Seen = (RequestOrigin, Option<FrameTag>, String);

/// Every request put to it, refused.
#[derive(Default)]
struct Recorder(Mutex<Vec<Seen>>);

impl AppNet for Recorder {
    fn decide(&self, request: &NetRequest) -> NetDecision {
        self.0.lock().unwrap_or_else(PoisonError::into_inner).push((
            request.origin(),
            request.frame_tag().cloned(),
            request.url().to_owned(),
        ));
        NetDecision::Deny
    }

    fn fetch(&self, _request: NetRequest, _reply: NetReply) {}
}

/// The reader with `requests` as its network and every link click logged.
fn reader(requests: Arc<Recorder>) -> (Harness, Arc<Mutex<Vec<FrameLink>>>) {
    let heard = Arc::new(Mutex::new(Vec::new()));
    let log = Arc::clone(&heard);
    let links = FrameLinks::intercept(move |link| {
        log.lock()
            .unwrap_or_else(PoisonError::into_inner)
            .push(link);
    });
    let config = HarnessConfig::new(VIEW)
        .with_net(NetPolicy::Custom(requests))
        .with_frame_links(links);
    (Harness::with_config(Reader, config), heard)
}

fn tag(text: &str) -> Option<FrameTag> {
    Some(FrameTag::new(text))
}

#[test]
fn each_frames_request_carries_its_tag() {
    let requests = Arc::new(Recorder::default());
    let (harness, _) = reader(Arc::clone(&requests));
    let first = harness.frame("iframe.first").expect("the first frame").id();
    let second = harness
        .frame("iframe.second")
        .expect("the second frame")
        .id();
    let mut seen = requests
        .0
        .lock()
        .unwrap_or_else(PoisonError::into_inner)
        .clone();
    seen.sort_by(|a, b| a.2.cmp(&b.2));
    assert_eq!(
        seen,
        vec![
            (
                RequestOrigin::Frame(first),
                tag("msg-1"),
                "https://example.org/first.png".to_owned()
            ),
            (
                RequestOrigin::Frame(second),
                tag("msg-2"),
                "https://example.org/second.png".to_owned()
            ),
        ]
    );
}

#[test]
fn a_link_click_carries_its_frames_tag() {
    let (mut harness, heard) = reader(Arc::new(Recorder::default()));
    for selector in ["iframe.second", "iframe.first"] {
        let at = harness
            .frame(selector)
            .and_then(|frame| frame.centre("a.link"))
            .expect("the frame's link");
        harness.click(at);
    }
    harness.advance(Duration::from_millis(50));
    let heard = heard.lock().unwrap_or_else(PoisonError::into_inner).clone();
    let tagged: Vec<_> = heard
        .iter()
        .map(|link| (link.tag.clone(), link.href.as_str()))
        .collect();
    assert_eq!(
        tagged,
        vec![
            (tag("msg-2"), "https://example.org/two"),
            (tag("msg-1"), "https://example.org/one"),
        ]
    );
}

#[test]
fn a_frame_is_found_by_its_tag_and_its_tag_by_the_frame() {
    let (harness, _) = reader(Arc::new(Recorder::default()));
    let first = harness.frame("iframe.first").expect("the first frame").id();
    let second = harness
        .frame("iframe.second")
        .expect("the second frame")
        .id();
    assert_eq!(frame_by_tag(&FrameTag::new("msg-1")), Some(first));
    assert_eq!(frame_by_tag(&FrameTag::new("msg-2")), Some(second));
    assert_eq!(frame_by_tag(&FrameTag::new("msg-3")), None);
    assert_eq!(tag_of(first), tag("msg-1"));
    assert_eq!(tag_of(second), tag("msg-2"));
    assert_ne!(first.index(), second.index(), "the keys are the frames'");
}

#[test]
fn a_frame_is_forgotten_with_its_document() {
    let first = {
        let (harness, _) = reader(Arc::new(Recorder::default()));
        harness.frame("iframe.first").expect("the first frame").id()
    };
    assert_eq!(tag_of(first), None);
    assert_eq!(frame_by_tag(&FrameTag::new("msg-1")), None);
}
