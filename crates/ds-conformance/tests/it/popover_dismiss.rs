//! Popover's `Dismiss` policy on a real Blitz document (design/30 section 1.5,
//! `NSPopover.Behavior`): Transient closes on Escape and on a click outside, Semitransient on
//! Escape only, Manual on neither (its owner closes it); a closing popover fades out over
//! `--t-quick` before `onclose` runs, and an arrow is drawn on the side facing the anchor only
//! when it is asked for.

use dioxus::prelude::*;
use ds::base::geometry::placement::{Align, Flip, Side};
use ds::base::vocab::Dismiss;
use ds::components::overlays::popover::Arrow;
use ds::host::measure::Anchor;
use ds::prelude::*;
use ds_harness::{Clock, Driver, FocusState, Harness, HarnessConfig, Input, Query, Viewport};
use std::cell::Cell;
use std::time::Duration;

const VIEW: Viewport = Viewport {
    width: 640,
    height: 400,
    scale_percent: 100,
};

thread_local! {
    static DISMISS: Cell<Dismiss> = const { Cell::new(Dismiss::Transient) };
    static ARROW: Cell<Arrow> = const { Cell::new(Arrow::None) };
}

fn ms(n: u64) -> Duration {
    Duration::from_millis(n)
}

#[allow(non_snake_case)]
fn Page() -> Element {
    let mut open = use_signal(|| true);
    rsx! {
        Ds { appearance: Appearance::default(), material: Material::Window, extent: RootExtent::Viewport,
            span { id: "state", if open() { "open" } else { "closed" } }
            if open() {
                Popover {
                    anchor: Anchor::Point(Point { x: Px(200.0), y: Px(100.0) }),
                    placement: Placement { side: Side::Bottom, align: Align::Center, flip: Flip::Allowed },
                    gap: Px(4.0),
                    arrow: ARROW.with(Cell::get),
                    dismiss: DISMISS.with(Cell::get),
                    onclose: move |()| open.set(false),
                    button { id: "inside", "Inside" }
                }
            }
        }
    }
}

fn start(dismiss: Dismiss, arrow: Arrow) -> Harness {
    DISMISS.with(|cell| cell.set(dismiss));
    ARROW.with(|cell| cell.set(arrow));
    let mut harness = Harness::new(Page, HarnessConfig::new(VIEW).with_clock(Clock::Virtual));
    harness.advance(ms(400));
    assert_eq!(
        harness.attr(".ds-popover", "data-presence").as_deref(),
        Some("present")
    );
    harness
}

/// The keyboard in the popover, where Escape is heard (a popover holds the keyboard as its
/// content takes it).
fn take_keyboard(harness: &mut Harness) {
    harness.send(Input::key(ShortcutKey::Tab));
    harness.advance(ms(16));
    assert_eq!(
        harness.focus_of("#inside"),
        FocusState::Focused,
        "the button took the keyboard"
    );
}

fn click_outside(harness: &mut Harness) {
    let at = Point {
        x: Px(600.0),
        y: Px(360.0),
    };
    harness.send(Input::pointer_down(at));
    harness.send(Input::pointer_up(at));
    harness.advance(ms(16));
}

fn state(harness: &Harness) -> Option<String> {
    harness.text_of("#state")
}

#[test]
fn transient_closes_on_escape_and_on_an_outside_click_after_its_fade() {
    let mut harness = start(Dismiss::Transient, Arrow::None);
    take_keyboard(&mut harness);
    harness.send(Input::key(ShortcutKey::Escape));
    harness.advance(ms(16));
    assert_eq!(
        harness.attr(".ds-popover", "data-presence").as_deref(),
        Some("leaving")
    );
    assert_eq!(
        state(&harness).as_deref(),
        Some("open"),
        "not before the fade has run"
    );
    harness.advance(ms(400));
    assert_eq!(state(&harness).as_deref(), Some("closed"));

    let mut harness = start(Dismiss::Transient, Arrow::None);
    click_outside(&mut harness);
    assert_eq!(
        harness.attr(".ds-popover", "data-presence").as_deref(),
        Some("leaving")
    );
    harness.advance(ms(400));
    assert_eq!(state(&harness).as_deref(), Some("closed"));
}

#[test]
fn semitransient_closes_on_escape_only() {
    let mut harness = start(Dismiss::Semitransient, Arrow::None);
    click_outside(&mut harness);
    harness.advance(ms(400));
    assert_eq!(
        state(&harness).as_deref(),
        Some("open"),
        "an outside click leaves it"
    );
    take_keyboard(&mut harness);
    harness.send(Input::key(ShortcutKey::Escape));
    harness.advance(ms(400));
    assert_eq!(state(&harness).as_deref(), Some("closed"));
}

#[test]
fn manual_closes_on_neither() {
    let mut harness = start(Dismiss::Manual, Arrow::None);
    click_outside(&mut harness);
    take_keyboard(&mut harness);
    harness.send(Input::key(ShortcutKey::Escape));
    harness.advance(ms(600));
    assert_eq!(
        state(&harness).as_deref(),
        Some("open"),
        "only its owner closes it"
    );
}

#[test]
fn the_arrow_stands_on_the_edge_facing_its_anchor_and_only_when_asked() {
    let bare = start(Dismiss::Transient, Arrow::None);
    assert_eq!(bare.count(".ds-popover-arrow"), 0);
    let harness = start(Dismiss::Transient, Arrow::Arrow);
    assert_eq!(harness.count(".ds-popover-arrow"), 1);
    assert_eq!(
        harness.attr(".ds-popover-arrow", "data-side").as_deref(),
        Some("top"),
        "below its anchor, the arrow is on the popover's top edge"
    );
    let card = harness.rect(".ds-popover").expect("the popover");
    // The arrow's 8 px stand between the anchor (y 100) and the popover, on the 4 px gap plus 8.
    assert!(
        (card.origin.y.0 - (100.0 + 4.0 + 8.0)).abs() < 1.0,
        "{card:?}: the arrow's height is added to the gap"
    );
    let arrow = harness.rect(".ds-popover-arrow").expect("the arrow");
    let across = arrow.origin.x.0 + arrow.size.width.0 / 2.0;
    assert!(
        (across - 200.0).abs() < 1.5,
        "centred on the anchor: {arrow:?}"
    );
}
