//! G7 (mailo Phase B): a link clicked inside a frame never navigates the frame. With
//! `FrameLinks::Intercept` the app hears the click (mailo opens it in the browser); with the
//! default `Inert` nothing happens. Either way no request is made for the link's target.

use dioxus::prelude::*;
use ds_native::{
    AppNet, FrameLink, FrameLinks, Harness, HarnessConfig, NetDecision, NetPolicy, NetReply,
    NetRequest, Viewport,
};
use std::sync::{Arc, Mutex, PoisonError};

const VIEW: Viewport = Viewport {
    width: 400,
    height: 300,
    scale_percent: 100,
};

const BODY: &str = "<body style='margin:0'>\
    <a class='link' href='https://example.org/offer' style='display:block; font-size:40px'>Offer</a>\
    </body>";

#[allow(non_snake_case)]
fn Reader() -> Element {
    rsx! {
        iframe { class: "body", srcdoc: BODY, style: "width: 300px; height: 150px; border: 0" }
    }
}

/// Every request any document makes, refused.
#[derive(Default)]
struct Requests(Mutex<Vec<String>>);

impl AppNet for Requests {
    fn decide(&self, request: &NetRequest) -> NetDecision {
        self.0
            .lock()
            .unwrap_or_else(PoisonError::into_inner)
            .push(request.url().to_owned());
        NetDecision::Deny
    }

    fn fetch(&self, _request: NetRequest, _reply: NetReply) {}
}

impl Requests {
    fn seen(&self) -> Vec<String> {
        self.0
            .lock()
            .unwrap_or_else(PoisonError::into_inner)
            .clone()
    }
}

/// Build the reader with `links`, click the frame's link, and say what the frame was before and
/// after, and what was requested.
fn click_the_link(links: FrameLinks) -> (Harness, ds_native::FrameId, Arc<Requests>) {
    let requests = Arc::new(Requests::default());
    let config = HarnessConfig::new(VIEW)
        .with_net(NetPolicy::Custom(requests.clone()))
        .with_frame_links(links);
    let mut harness = Harness::with_config(Reader, config);
    let frame = harness.frame("iframe.body").expect("a frame document");
    let before = frame.id();
    let at = frame.centre("a.link").expect("the link in the frame");
    harness.click(at);
    harness.advance(std::time::Duration::from_millis(50));
    (harness, before, requests)
}

#[test]
fn an_intercepted_click_reaches_the_app_and_the_frame_stays() {
    let heard = Arc::new(Mutex::new(Vec::<FrameLink>::new()));
    let log = Arc::clone(&heard);
    let links = FrameLinks::intercept(move |link| {
        log.lock()
            .unwrap_or_else(PoisonError::into_inner)
            .push(link);
    });
    let (harness, before, requests) = click_the_link(links);
    let heard = heard.lock().unwrap_or_else(PoisonError::into_inner).clone();
    assert_eq!(
        heard,
        vec![FrameLink {
            frame: before,
            tag: None,
            href: "https://example.org/offer".to_owned()
        }]
    );
    let after = harness
        .frame("iframe.body")
        .expect("still a frame document");
    assert_eq!(after.id(), before, "the frame was reloaded");
    assert_eq!(after.text_of("a.link").as_deref(), Some("Offer"));
    assert!(
        requests.seen().is_empty(),
        "requested {:?}",
        requests.seen()
    );
}

#[test]
fn an_inert_click_does_nothing() {
    let (harness, before, requests) = click_the_link(FrameLinks::Inert);
    let after = harness
        .frame("iframe.body")
        .expect("still a frame document");
    assert_eq!(after.id(), before, "the frame was reloaded");
    assert!(
        requests.seen().is_empty(),
        "requested {:?}",
        requests.seen()
    );
}
