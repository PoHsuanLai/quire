//! `Scroller` keeps its scroll state in the owner's `ScrollerRef`: the state is measured by the
//! host's frame phase, follows a wheel scroll, and carries out the scrolls the owner asks for,
//! all within one settle and with nothing slept on.

use dioxus::prelude::*;
use ds::base::geometry::scroll::Scroll;
use ds::components::controls::scroller::handle::use_scroller;
use ds::components::controls::scroller::view::Scroller;
use ds::prelude::*;
use ds_harness::{DocQuery, Driver, Harness, Input, Query, Viewport};

const VIEW: Viewport = Viewport {
    width: 400,
    height: 300,
    scale_percent: 100,
};

#[allow(non_snake_case)]
fn App() -> Element {
    let scroller = use_scroller();
    let Scroll {
        offset,
        viewport,
        content,
    } = scroller.scroll()();
    rsx! {
        Ds { appearance: Appearance::default(), material: Material::Window,
            div { style: "height:100px; width:200px; display:flex; flex-direction:column",
                Scroller { scroller,
                    div { style: "height:1000px" }
                }
            }
            button { class: "to-300", onclick: move |_| scroller.scroll_to(Px(300.0)), "to 300" }
            button { class: "by-back", onclick: move |_| scroller.by(Px(-50.0)), "back 50" }
            button { class: "past-end", onclick: move |_| scroller.scroll_to(Px(9000.0)), "past the end" }
            p { class: "state", "{offset.0} {viewport.0} {content.0}" }
        }
    }
}

fn state(harness: &Harness) -> String {
    harness.text_of(".state").unwrap_or_default()
}

/// The scroller element's own offset in the document.
fn document_offset(harness: &Harness) -> f64 {
    harness
        .with_doc(|doc| {
            let scroller = doc.query_selector(".ds-scroller").ok().flatten()?;
            Some(doc.get_node(scroller)?.scroll_offset().y)
        })
        .expect("the scroller is in the document")
}

fn click(harness: &mut Harness, selector: &str) {
    let at = harness.centre(selector).expect("the button is laid out");
    harness.send(Input::click(at));
}

#[test]
fn the_state_is_measured_and_follows_the_owners_scrolls_and_the_wheel() {
    let mut harness = Harness::new(App, VIEW);
    assert_eq!(
        state(&harness),
        "0 100 1000",
        "measured after the first layout"
    );

    let at = harness.centre(".ds-scroller").expect("the scroller");
    harness.send(Input::wheel(at, Px(0.0), Px(-120.0)));
    assert_eq!(
        state(&harness),
        format!("{} 100 1000", document_offset(&harness)),
        "a wheel scroll reaches the model"
    );
    assert!(
        document_offset(&harness) > 0.0,
        "the wheel moved the content"
    );

    click(&mut harness, ".to-300");
    assert_eq!(state(&harness), "300 100 1000");
    assert_eq!(document_offset(&harness), 300.0);

    click(&mut harness, ".by-back");
    assert_eq!(state(&harness), "250 100 1000");
    assert_eq!(document_offset(&harness), 250.0);

    click(&mut harness, ".past-end");
    assert_eq!(state(&harness), "900 100 1000", "kept inside the content");
    assert_eq!(document_offset(&harness), 900.0);
}
