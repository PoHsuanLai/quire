//! The frame phase: a component watches an element's rect and queues a scroll through the host's
//! geometry part, and the window loop's one step per frame (the harness runs it the same way)
//! publishes the rect when it changes and applies the scroll, with the document free. Nothing
//! here waits a frame or retries: the harness settles and the values are there.

use dioxus::prelude::*;
use ds::host::document::{DocumentHost, use_document_host};
use ds::host::measure::MountedRef;
use ds::host::phase::{Observe, Observed, PhaseWrite, Queued, Watch};
use ds::prelude::*;
use ds_harness::{DocQuery, Driver, Harness, Input, Query, Viewport};
use std::rc::Rc;
use std::time::Duration;

const VIEW: Viewport = Viewport {
    width: 480,
    height: 320,
    scale_percent: 100,
};

/// A box that grows when its button is pressed, watched through the host.
#[allow(non_snake_case)]
fn Growing() -> Element {
    let host = use_document_host();
    let mut wide = use_signal(|| false);
    let rect = use_signal(|| None::<Rect>);
    let mut watch = use_signal(|| None::<Watch>);
    // Counted outside the signals, so counting does not itself change anything.
    let mut publishes = use_hook(|| CopyValue::new(0u32));
    use_effect(move || {
        let _ = rect();
        publishes += 1;
    });
    let width = if wide() { 200 } else { 100 };
    let shown = rect().map_or_else(
        || "none".to_owned(),
        |rect| format!("{}x{}", rect.size.width.0, rect.size.height.0),
    );
    rsx! {
        div { class: "box", style: "width:{width}px; height:40px",
            onmounted: move |mounted: MountedEvent| {
                let observed = host.geometry().observe(&mounted.data(), Observe::Rect(rect));
                if let Observed::Watching(found) = observed {
                    watch.set(Some(found));
                }
            },
        }
        button { class: "grow", onclick: move |_| wide.set(true), "grow" }
        p { class: "rect", {shown} }
        p { class: "publishes", "{publishes}" }
    }
}

fn click(harness: &mut Harness, selector: &str) {
    let at = harness
        .centre(selector)
        .unwrap_or_else(|| panic!("{selector} is not laid out:\n{}", harness.html()));
    harness.send(Input::click(at));
}

#[test]
fn a_watched_rect_is_published_after_layout_and_again_only_when_it_changes() {
    let mut harness = Harness::new(Growing, VIEW);
    assert_eq!(harness.text_of(".rect").as_deref(), Some("100x40"));
    let first = harness.text_of(".publishes");

    click(&mut harness, ".grow");
    assert_eq!(harness.text_of(".rect").as_deref(), Some("200x40"));
    // Every frame since the box last moved found the same rect and published nothing.
    harness.advance(Duration::from_millis(200));
    let counts = (first, harness.text_of(".publishes"));
    assert_eq!(
        counts,
        (Some("1".to_owned()), Some("2".to_owned())),
        "one effect run for the rect first published, one for the width change"
    );
}

/// Queue `writes` to the scroller through the host.
fn queue(host: &Rc<dyn DocumentHost>, scroller: Option<MountedRef>, writes: &[f32]) {
    let Some(MountedRef(element)) = scroller else {
        return;
    };
    for &to in writes {
        let queued = host
            .geometry()
            .write(&element, PhaseWrite::ScrollTo(Px(to)));
        assert_eq!(queued, Queued::Yes);
    }
}

/// A scroller of 1000 px of content in 100, and buttons that queue writes to it.
#[allow(non_snake_case)]
fn Scrolling() -> Element {
    let host = use_document_host();
    let mut scroller = use_signal(|| None::<MountedRef>);
    let (one, many, past) = (host.clone(), host.clone(), host);
    rsx! {
        div { class: "scroller", style: "height:100px; overflow-y:auto; scrollbar-width:none",
            onmounted: move |mounted: MountedEvent| scroller.set(Some(MountedRef(mounted.data()))),
            div { style: "height:1000px" }
        }
        button { class: "to-300", onclick: move |_| queue(&one, scroller(), &[300.0]), "to 300" }
        button { class: "to-many", onclick: move |_| queue(&many, scroller(), &[40.0, 120.0, 250.0]), "to 250" }
        button { class: "past-end", onclick: move |_| queue(&past, scroller(), &[5000.0]), "past the end" }
    }
}

fn offset(harness: &Harness) -> f64 {
    harness
        .with_doc(|doc| {
            let scroller = doc.query_selector(".scroller").ok().flatten()?;
            Some(doc.get_node(scroller)?.scroll_offset().y)
        })
        .expect("the scroller is in the document")
}

#[test]
fn a_queued_scroll_is_applied_in_the_frame_phase() {
    // (button, the offset it leaves the scroller at)
    const CASES: &[(&str, f64)] = &[
        (".to-300", 300.0),
        // Three writes in one handler are one scroll to the last.
        (".to-many", 250.0),
        // The scrollable range ends at 1000 - 100.
        (".past-end", 900.0),
    ];
    for &(button, want) in CASES {
        let mut harness = Harness::new(Scrolling, VIEW);
        assert_eq!(offset(&harness), 0.0, "{button}: starts at the top");
        click(&mut harness, button);
        assert_eq!(offset(&harness), want, "{button}");
    }
}
