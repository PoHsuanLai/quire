//! G4 (mailo Phase B): an `<iframe srcdoc>` is a real, separate sub-document under the harness,
//! as in the window: its text renders, its selectors are found through the frame and never
//! through the app's document, and a `<script>` in it does nothing (Blitz runs no scripts).

use dioxus::prelude::*;
use ds_native::{Harness, Viewport};

const VIEW: Viewport = Viewport {
    width: 400,
    height: 300,
    scale_percent: 100,
};

/// A sender-shaped body: its own style, text, and a script that would rewrite it.
const BODY: &str = "<html><head><style>p { color: red }</style></head><body>\
    <p class='inner'>Hello from the frame</p>\
    <script>document.body.innerHTML = '<p class=\"pwned\">rewritten</p>';</script>\
    </body></html>";

#[allow(non_snake_case)]
fn Reader() -> Element {
    rsx! {
        p { class: "outer", "The app's own text" }
        iframe { class: "body", srcdoc: BODY, style: "width: 300px; height: 150px" }
    }
}

#[test]
fn a_srcdoc_frame_renders_its_own_document() {
    let harness = Harness::new(Reader, VIEW);
    let frame = harness
        .frame("iframe.body")
        .unwrap_or_else(|| panic!("no frame document:\n{}", harness.html()));
    assert_eq!(
        frame.text_of(".inner").as_deref(),
        Some("Hello from the frame")
    );
    assert_eq!(frame.count("p.inner"), 1);
    assert!(frame.text().contains("Hello from the frame"));
}

#[test]
fn the_app_and_the_frame_never_see_each_others_nodes() {
    let harness = Harness::new(Reader, VIEW);
    let frame = harness.frame("iframe.body").expect("a frame document");
    assert_eq!(harness.count(".inner"), 0, "the app saw into the frame");
    assert_eq!(frame.count(".outer"), 0, "the frame saw the app");
}

#[test]
fn a_script_in_the_frame_does_nothing() {
    let mut harness = Harness::new(Reader, VIEW);
    harness.advance(std::time::Duration::from_millis(50));
    let frame = harness.frame("iframe.body").expect("a frame document");
    assert_eq!(frame.count(".pwned"), 0);
    assert_eq!(frame.count(".inner"), 1);
}

#[test]
fn the_frames_text_is_painted_in_its_own_colour() {
    let mut harness = Harness::new(Reader, VIEW);
    let frame = harness.render().expect("renders");
    let at = harness.rect("iframe.body").expect("the frame's box");
    let (x0, y0) = (at.origin.x.0 as u32, at.origin.y.0 as u32);
    let (x1, y1) = (x0 + at.size.width.0 as u32, y0 + at.size.height.0 as u32);
    let red = (x0..x1.min(frame.width()))
        .flat_map(|x| (y0..y1.min(frame.height())).map(move |y| (x, y)))
        .filter(|&(x, y)| {
            let [r, g, b, _] = frame.get_pixel(x, y).0;
            r > 150 && g < 90 && b < 90
        })
        .count();
    assert!(
        red > 10,
        "no red text painted inside the frame ({red} pixels)"
    );
}
