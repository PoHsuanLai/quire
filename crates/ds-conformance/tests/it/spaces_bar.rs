//! The Spaces bar on a real Blitz document: the pointer over it expands it to thumbnails and off
//! it collapses it again, unless the caller holds the expansion (which then hears the ask); a
//! primary press on a Space reports its key and a secondary one does not; the plus reports an
//! add; the Space a drag is over is lit; and a bar its host hides lifts away, reporting
//! `on_hidden` at `settle(LiftOut)` and drawing nothing after.

use dioxus::prelude::*;
use ds::base::press::PointerButton;
use ds::prelude::*;
use ds_harness::{Clock, Driver, Harness, HarnessConfig, Input, Query, Viewport};
use std::time::Duration;

const VIEW: Viewport = Viewport {
    width: 800,
    height: 400,
    scale_percent: 100,
};

/// Well below the bar, in the page's empty space.
const ELSEWHERE: Point = Point {
    x: Px(400.0),
    y: Px(380.0),
};

fn ms(n: u64) -> Duration {
    Duration::from_millis(n)
}

/// Three Spaces, the second on show and a drag over the third.
fn tiles() -> Vec<SpaceTile> {
    vec![
        SpaceTile::new(BarSpaceKey("a".to_owned()), "Desktop 1"),
        SpaceTile::new(BarSpaceKey("b".to_owned()), "Desktop 2").current(Selection::Selected),
        SpaceTile::new(BarSpaceKey("c".to_owned()), "Desktop 3").in_drag(DropState::Target),
    ]
}

/// The bar the page hangs, with every report written to the log. `held` is the expansion the
/// caller holds, if any.
#[component]
fn Hung(held: Option<BarExpansion>) -> Element {
    let shown = use_context::<Signal<Shown>>();
    let mut log = use_signal(Vec::<String>::new);
    rsx! {
        SpacesBar {
            label: "Spaces",
            spaces: tiles(),
            shown: shown(),
            expansion: held,
            on_pick: move |key: BarSpaceKey| log.with_mut(|log| log.push(key.0)),
            on_add: move |()| log.with_mut(|log| log.push("add".to_owned())),
            on_expansion_change: move |to: BarExpansion| log.with_mut(|log| log.push(to.slug().to_owned())),
            on_hidden: move |()| log.with_mut(|log| log.push("hidden".to_owned())),
        }
        p { class: "log", {log().join(",")} }
    }
}

#[allow(non_snake_case)]
fn Page() -> Element {
    use_context_provider(|| Signal::new(Shown::Visible));
    rsx! {
        Ds { appearance: Appearance::default(), material: Material::Window, extent: RootExtent::Viewport, Hung { held: None } }
    }
}

#[allow(non_snake_case)]
fn Held() -> Element {
    use_context_provider(|| Signal::new(Shown::Visible));
    rsx! {
        Ds { appearance: Appearance::default(), material: Material::Window, extent: RootExtent::Viewport, Hung { held: Some(BarExpansion::Collapsed) } }
    }
}

fn harness(page: fn() -> Element) -> Harness {
    let mut harness = Harness::new(page, HarnessConfig::new(VIEW).with_clock(Clock::Virtual));
    harness.advance(settle(Anim::DropIn, MotionLevel::Standard) + ms(60));
    harness
}

fn set(harness: &mut Harness, to: Shown) {
    harness.within(|| {
        dioxus::core::Runtime::current()
            .in_scope(ScopeId::APP, || consume_context::<Signal<Shown>>().set(to))
    });
}

fn expansion(harness: &Harness) -> Option<String> {
    harness.attr(".ds-spaces-bar", "data-expansion")
}

fn log(harness: &Harness) -> String {
    harness.text_of(".log").unwrap_or_default()
}

/// Put the pointer on the bar and let it expand.
fn hover(harness: &mut Harness) {
    let over = harness.centre(".ds-spaces-bar-row").expect("the row");
    harness.send(Input::pointer_move(over));
    harness.advance(ms(60));
}

#[test]
fn the_pointer_over_the_bar_expands_it_and_leaving_collapses_it() {
    let mut harness = harness(Page);
    assert_eq!(expansion(&harness).as_deref(), Some("collapsed"));
    assert_eq!(harness.count(".ds-spaces-bar-thumb"), 0, "names only");
    hover(&mut harness);
    assert_eq!(expansion(&harness).as_deref(), Some("expanded"));
    assert_eq!(harness.count(".ds-spaces-bar-thumb"), 3, "a thumbnail each");
    harness.send(Input::pointer_move(ELSEWHERE));
    harness.advance(ms(60));
    assert_eq!(expansion(&harness).as_deref(), Some("collapsed"));
    assert_eq!(harness.count(".ds-spaces-bar-thumb"), 0);
    assert_eq!(log(&harness), "expanded,collapsed", "each ask is reported");
}

#[test]
fn a_primary_press_reports_the_space_and_a_secondary_one_does_not() {
    let mut harness = harness(Page);
    hover(&mut harness);
    let first = harness
        .centre(".ds-spaces-bar-space[*|data-selected=unselected]")
        .expect("Desktop 1");
    harness.send(Input::press(first, PointerButton::Secondary));
    harness.advance(ms(40));
    assert_eq!(log(&harness), "expanded", "a right-click picks nothing");
    harness.send(Input::click(first));
    harness.advance(ms(40));
    assert_eq!(log(&harness), "expanded,a");
}

#[test]
fn the_plus_reports_an_add() {
    let mut harness = harness(Page);
    hover(&mut harness);
    let plus = harness.centre(".ds-spaces-bar-add").expect("the plus");
    harness.send(Input::click(plus));
    harness.advance(ms(40));
    assert_eq!(log(&harness), "expanded,add");
}

#[test]
fn a_held_expansion_is_drawn_whatever_the_pointer_does_and_hears_the_ask() {
    let mut harness = harness(Held);
    hover(&mut harness);
    assert_eq!(
        expansion(&harness).as_deref(),
        Some("collapsed"),
        "the caller's value wins"
    );
    assert_eq!(log(&harness), "expanded", "but it hears what was asked");
}

#[test]
fn the_space_a_drag_is_over_is_lit_and_the_current_one_is_marked() {
    let harness = harness(Page);
    assert_eq!(harness.count(".ds-spaces-bar-space[*|data-drop=target]"), 1);
    assert_eq!(
        harness
            .text_of(".ds-spaces-bar-space[*|data-drop=target] .ds-spaces-bar-name")
            .as_deref(),
        Some("Desktop 3")
    );
    assert_eq!(
        harness
            .attr(
                ".ds-spaces-bar-space[*|data-selected=selected]",
                "aria-current"
            )
            .as_deref(),
        Some("true")
    );
}

#[test]
fn a_hidden_bar_lifts_away_reports_at_its_settle_and_draws_nothing_after() {
    let mut harness = harness(Page);
    let asked = harness.now();
    set(&mut harness, Shown::Hidden);
    harness.advance(ms(20));
    assert_eq!(
        harness.attr(".ds-spaces-bar", "data-presence").as_deref(),
        Some("leaving")
    );
    let exit = settle(Anim::LiftOut, MotionLevel::Standard);
    harness.advance(exit - ms(21));
    assert_eq!(log(&harness), "", "not before the settle");
    let reported = ds_harness::harness::settle_until(&mut harness, |h| log(h) == "hidden");
    assert_eq!(reported.duration_since(asked), exit);
    assert_eq!(harness.count(".ds-spaces-bar"), 0, "gone once settled");
}
