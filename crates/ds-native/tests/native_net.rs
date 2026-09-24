//! G3 (mailo Phase B): the app's network policy. The app's document gets `file:` under `Local`
//! and nothing but `data:` under `Sealed`; under `Custom` every other request is put to the
//! app's handler, which sees where it came from and decides.

use dioxus::prelude::*;
use ds_native::{
    AppNet, Harness, HarnessConfig, NetDecision, NetPolicy, NetReply, NetRequest, RequestOrigin,
    Viewport,
};
use std::path::PathBuf;
use std::sync::{Arc, Mutex, OnceLock, PoisonError};

const VIEW: Viewport = Viewport {
    width: 200,
    height: 120,
    scale_percent: 100,
};

/// A 7 x 5 PNG on disk: an `img` that loads it lays out 7 px wide, one that does not, 0.
/// Written once, whole, before any test reads it (tests run in parallel).
fn swatch() -> PathBuf {
    static WRITTEN: OnceLock<PathBuf> = OnceLock::new();
    WRITTEN
        .get_or_init(|| {
            let path = std::path::Path::new(env!("CARGO_TARGET_TMPDIR")).join("net-swatch.png");
            image::RgbaImage::from_pixel(7, 5, image::Rgba([0, 160, 0, 255]))
                .save(&path)
                .expect("write the swatch");
            path
        })
        .clone()
}

/// The same PNG's bytes, for a handler to answer with.
fn swatch_bytes() -> Vec<u8> {
    std::fs::read(swatch()).expect("read the swatch")
}

#[allow(non_snake_case)]
fn LocalImage() -> Element {
    let src = format!("file://{}", swatch().display());
    rsx! { img { class: "local", src } }
}

#[allow(non_snake_case)]
fn RemoteImage() -> Element {
    rsx! { img { class: "remote", src: "https://example.org/pixel.png" } }
}

/// Records every request put to it and answers as told.
struct Recorder {
    decision: NetDecision,
    seen: Mutex<Vec<(RequestOrigin, String)>>,
}

impl Recorder {
    fn new(decision: NetDecision) -> Arc<Self> {
        Arc::new(Recorder {
            decision,
            seen: Mutex::new(Vec::new()),
        })
    }

    fn seen(&self) -> Vec<(RequestOrigin, String)> {
        self.seen
            .lock()
            .unwrap_or_else(PoisonError::into_inner)
            .clone()
    }
}

impl AppNet for Recorder {
    fn decide(&self, request: &NetRequest) -> NetDecision {
        self.seen
            .lock()
            .unwrap_or_else(PoisonError::into_inner)
            .push((request.origin(), request.url().to_owned()));
        self.decision
    }

    fn fetch(&self, _request: NetRequest, reply: NetReply) {
        reply.bytes(swatch_bytes());
    }
}

fn width(harness: &Harness, selector: &str) -> f32 {
    harness
        .rect(selector)
        .unwrap_or_else(|| panic!("{selector} missing:\n{}", harness.html()))
        .size
        .width
        .0
}

#[test]
fn the_app_document_gets_file_under_local_and_not_under_sealed() {
    let cases = [(NetPolicy::Local, 7.0), (NetPolicy::Sealed, 0.0)];
    for (policy, want) in cases {
        let name = format!("{policy:?}");
        let harness = Harness::with_config(LocalImage, HarnessConfig::new(VIEW).with_net(policy));
        assert_eq!(width(&harness, ".local"), want, "{name}");
    }
}

#[test]
fn a_custom_handler_sees_the_app_documents_request_and_answers_it() {
    let cases = [(NetDecision::Allow, 7.0), (NetDecision::Deny, 0.0)];
    for (decision, want) in cases {
        let recorder = Recorder::new(decision);
        let config = HarnessConfig::new(VIEW).with_net(NetPolicy::Custom(recorder.clone()));
        let harness = Harness::with_config(RemoteImage, config);
        assert_eq!(
            recorder.seen(),
            vec![(
                RequestOrigin::Top,
                "https://example.org/pixel.png".to_owned()
            )],
            "{decision:?}"
        );
        assert_eq!(width(&harness, ".remote"), want, "{decision:?}");
    }
}

#[test]
fn a_custom_policy_still_serves_the_app_its_own_files_without_asking() {
    let recorder = Recorder::new(NetDecision::Deny);
    let config = HarnessConfig::new(VIEW).with_net(NetPolicy::Custom(recorder.clone()));
    let harness = Harness::with_config(LocalImage, config);
    assert_eq!(width(&harness, ".local"), 7.0);
    assert!(recorder.seen().is_empty(), "{:?}", recorder.seen());
}
