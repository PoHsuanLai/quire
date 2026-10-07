//! A folded `SplitView` pane that hosts an `EdgePeek` (design/30 sections 2.7 and 2.11): once the
//! pane has folded away its edge strip still takes the pointer, the sidebar floats out over the
//! content beside it, and a click on the strip pins it again. A pane that clips (the default)
//! keeps none of that.

use dioxus::prelude::*;
use ds::base::geometry::units::Point;
use ds::base::geometry::units::Px;
use ds::components::app::edge_peek::EdgePeek;
use ds::components::chrome::split_view::model::{PaneSpec, SplitPane};
use ds::components::chrome::split_view::view::SplitView;
use ds::prelude::*;
use ds_harness::{Clock, Driver, Harness, HarnessConfig, Input, Query, Viewport};
use std::cell::Cell;
use std::time::Duration;

const VIEW: Viewport = Viewport {
    width: 800,
    height: 400,
    scale_percent: 100,
};

thread_local! {
    static PEEKING: Cell<bool> = const { Cell::new(true) };
}

#[allow(non_snake_case)]
fn Page() -> Element {
    let mut pinned = use_signal(|| Shown::Hidden);
    let pane = SplitPane::new(
        PaneSpec::SIDEBAR,
        rsx! {
            EdgePeek { label: "Sidebar", pinned: pinned(), onpin: move |()| pinned.set(Shown::Visible),
                span { class: "side-words", "Places" }
            }
        },
    )
    .shown(pinned());
    let pane = if PEEKING.with(Cell::get) {
        pane.peeking()
    } else {
        pane
    };
    rsx! {
        Ds { appearance: Appearance::default(), material: Material::Window, extent: RootExtent::Viewport,
            SplitView { label: "Window", panes: vec![pane],
                div { class: "card", style: "height:400px", "Content" }
            }
        }
    }
}

fn start(peeking: bool) -> Harness {
    PEEKING.with(|cell| cell.set(peeking));
    let mut harness = Harness::new(Page, HarnessConfig::new(VIEW).with_clock(Clock::Virtual));
    harness.advance(Duration::from_millis(600));
    harness
}

fn at(x: f32, y: f32) -> Point {
    Point { x: Px(x), y: Px(y) }
}

#[test]
fn a_folded_peeking_pane_shows_its_sidebar_when_the_pointer_reaches_the_edge() {
    let mut harness = start(true);
    assert_eq!(
        harness.attr(".ds-split-pane", "data-away").as_deref(),
        Some("true")
    );
    assert_eq!(
        harness.attr(".ds-side", "data-side").as_deref(),
        Some("hidden")
    );
    harness.send(Input::pointer_move(at(3.0, 200.0)));
    harness.advance(Duration::from_millis(900));
    assert_eq!(
        harness.attr(".ds-side", "data-side").as_deref(),
        Some("peek")
    );
    let side = harness.rect(".ds-side").expect("the sidebar");
    assert!(side.size.width.0 > 200.0, "{side:?}");
    let card = harness.rect(".card").expect("the content");
    assert!(
        side.origin.x.0 + side.size.width.0 > card.origin.x.0 + 100.0,
        "the sidebar stands over the content, not inside the folded pane: {side:?} {card:?}"
    );
}

#[test]
fn a_click_on_the_strip_of_a_folded_peeking_pane_pins_the_sidebar() {
    let mut harness = start(true);
    harness.send(Input::click(at(3.0, 200.0)));
    harness.advance(Duration::from_millis(600));
    assert_eq!(
        harness.attr(".ds-side", "data-side").as_deref(),
        Some("shown")
    );
    assert_eq!(
        harness.attr(".ds-split-pane", "data-shown").as_deref(),
        Some("visible")
    );
}

#[test]
fn a_clipping_pane_keeps_its_peek_out_of_reach() {
    let mut harness = start(false);
    harness.send(Input::pointer_move(at(3.0, 200.0)));
    harness.advance(Duration::from_millis(900));
    assert_eq!(
        harness.attr(".ds-side", "data-side").as_deref(),
        Some("hidden"),
        "the folded pane clips what floats out of it"
    );
    harness.send(Input::click(at(3.0, 200.0)));
    harness.advance(Duration::from_millis(600));
    assert_eq!(
        harness.attr(".ds-split-pane", "data-shown").as_deref(),
        Some("hidden")
    );
}
