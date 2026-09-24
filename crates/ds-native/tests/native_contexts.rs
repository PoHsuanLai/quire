//! G1 (mailo Phase B): the app's own root contexts reach a component under the harness and in a
//! snapshot, as `AppConfig::with_context` gives them to the window.

use dioxus::prelude::*;
use ds_native::{Harness, HarnessConfig, RootContexts, Viewport, snapshot_with};
use std::sync::Arc;
use std::time::Duration;

const VIEW: Viewport = Viewport {
    width: 240,
    height: 120,
    scale_percent: 100,
};

/// A store-shaped context: shared, `Send + Sync`, built before the document.
#[derive(Debug)]
struct Store {
    name: String,
}

/// A plain-data context.
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
struct Start(u32);

#[allow(non_snake_case)]
fn Reader() -> Element {
    let store = use_context::<Arc<Store>>();
    let Start(thread) = use_context::<Start>();
    rsx! {
        p { class: "store", "{store.name}" }
        p { class: "start", "{thread}" }
    }
}

#[test]
fn a_root_reads_contexts_the_harness_was_given() {
    let contexts = RootContexts::new()
        .with(Arc::new(Store {
            name: "inbox.sqlite".to_owned(),
        }))
        .with(Start(42));
    let harness = Harness::with_contexts(Reader, VIEW, contexts);
    assert_eq!(harness.text_of(".store").as_deref(), Some("inbox.sqlite"));
    assert_eq!(harness.text_of(".start").as_deref(), Some("42"));
}

#[test]
fn a_later_context_of_the_same_type_shadows_an_earlier_one() {
    let config = HarnessConfig::new(VIEW)
        .with_context(Arc::new(Store {
            name: "first".to_owned(),
        }))
        .with_context(Start(1))
        .with_context(Start(2));
    let harness = Harness::with_config(Reader, config);
    assert_eq!(harness.text_of(".start").as_deref(), Some("2"));
}

#[test]
fn a_snapshot_renders_with_the_same_contexts() {
    let config = HarnessConfig::new(VIEW)
        .with_context(Arc::new(Store {
            name: "snap".to_owned(),
        }))
        .with_context(Start(7));
    let frames = snapshot_with(Reader, config, &[Duration::ZERO]).expect("snapshot");
    assert_eq!(frames.len(), 1);
}
