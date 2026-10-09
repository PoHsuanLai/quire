//! The IME's candidate window for a surface the app draws itself through a `TextureLayer`: the
//! layer's element, handed to `ImeHost::cursor_area`, moves the candidate window to the rect the
//! app gives, in the window's own logical pixels and untranslated (the app adds the layer's
//! origin to its caret, as `EditSurface`'s `ime_area` already expects).

use dioxus::prelude::*;
use ds::host::document::{DocumentHost, use_document_host};
use ds::host::ime::ImeSwitch;
use ds::host::probe::Probe;
use ds::prelude::*;
use ds::root::common::Common;
use ds_blitz::TextureLayer;
use ds_harness::{Clock, Driver, Harness, HarnessConfig, Viewport};
use std::cell::RefCell;
use std::rc::Rc;
use std::time::Duration;

const VIEW: Viewport = Viewport {
    width: 400,
    height: 300,
    scale_percent: 100,
};

/// The layer's element and the document's host, as the app saw them.
type Seen = (Rc<MountedData>, Rc<dyn DocumentHost>);

thread_local! {
    static SEEN: RefCell<Option<Seen>> = const { RefCell::new(None) };
}

#[allow(non_snake_case)]
fn Canvas() -> Element {
    let host = use_document_host();
    let common = Common {
        mounted: Some(EventHandler::new(move |event: MountedEvent| {
            SEEN.with(|seen| *seen.borrow_mut() = Some((event.data(), Rc::clone(&host))));
        })),
        ..Common::default()
    };
    rsx! {
        div { style: "position:absolute; top:60px; left:50px; width:200px; height:100px",
            TextureLayer { common }
        }
    }
}

fn caret() -> Rect {
    Rect {
        origin: Point {
            x: Px(55.0),
            y: Px(67.0),
        },
        size: Size {
            width: Px(1.0),
            height: Px(18.0),
        },
    }
}

#[test]
fn a_texture_layer_surface_moves_the_candidate_window() {
    let mut harness = Harness::new(Canvas, HarnessConfig::new(VIEW).with_clock(Clock::Virtual));
    harness.advance(Duration::from_millis(50));
    assert_eq!(harness.ime_cursor_area(), None, "nothing asked yet");
    let (element, host) = SEEN
        .with(|seen| seen.borrow().clone())
        .expect("the layer mounted");
    let answers = harness.within(|| {
        let ime = host.edit().expect("a Blitz host edits").ime();
        (
            ime.switch(&element, ImeSwitch::On),
            ime.cursor_area(&element, caret()),
        )
    });
    assert_eq!((answers.0, answers.1), (Probe::Found(()), Probe::Found(())));
    assert_eq!(harness.ime_switch(), ImeSwitch::On);
    assert_eq!(harness.ime_cursor_area(), Some(caret()));
}
