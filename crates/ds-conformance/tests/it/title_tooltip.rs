//! A control's `title` is a Mac tooltip (design/30 section 2.5): through the hover hub, after
//! the Tip delay cold (none by default, 1 s under `TipDelay::Standard`), at once while warm, gone on a press; one tip at most; and the wrapper it
//! needs changes nothing about the control's size or its neighbours' places.

use dioxus::prelude::*;
use ds::components::controls::button_model::{Bezel, ImagePosition};
use ds::prelude::*;
use ds::style::tokens::control_size::ControlSize;
use ds_harness::{Clock, Driver, Harness, HarnessConfig, Input, Query, Viewport};
use std::time::Duration;

const VIEW: Viewport = Viewport {
    width: 640,
    height: 400,
    scale_percent: 100,
};

fn ms(n: u64) -> Duration {
    Duration::from_millis(n)
}

fn icon(id: &'static str, title: Option<&str>, icon: Icon) -> Element {
    rsx! {
        Button {
            label: id,
            title: title.map(str::to_owned),
            bezel: Bezel::Toolbar,
            size: ControlSize::Large,
            image: ImagePosition::Only,
            icon: Some(IconSource::from(icon)),
            onclick: |_| {},
            common: Common { id: Some(id.to_owned()), ..Common::default() },
        }
    }
}

#[allow(non_snake_case)]
fn Titled() -> Element {
    rsx! {
        Ds { appearance: Appearance::default(), material: Material::Window, extent: RootExtent::Viewport,
            div { style: "position:absolute;left:100px;top:100px;display:flex;gap:8px",
                {icon("first", Some("Archive"), Icon::Archive)}
                {icon("second", Some("Reply"), Icon::Reply)}
            }
        }
    }
}

#[allow(non_snake_case)]
fn Slow() -> Element {
    rsx! {
        Ds { appearance: Appearance::default(), material: Material::Window, extent: RootExtent::Viewport, tip_delay: TipDelay::Standard,
            div { style: "position:absolute;left:100px;top:100px;display:flex;gap:8px",
                {icon("first", Some("Archive"), Icon::Archive)}
                {icon("second", Some("Reply"), Icon::Reply)}
            }
        }
    }
}

#[allow(non_snake_case)]
fn Doubled() -> Element {
    rsx! {
        Ds { appearance: Appearance::default(), material: Material::Window, extent: RootExtent::Viewport,
            div { style: "position:absolute;left:100px;top:100px;display:flex;gap:8px",
                Tooltip { text: "Archive", {icon("first", Some("Also archive"), Icon::Archive)} }
            }
        }
    }
}

#[allow(non_snake_case)]
fn Neutral() -> Element {
    rsx! {
        Ds { appearance: Appearance::default(), material: Material::Window, extent: RootExtent::Viewport,
            div { style: "position:absolute;left:100px;top:100px;display:flex;gap:8px",
                {icon("wrapped-a", Some("Archive"), Icon::Archive)}
                {icon("wrapped-b", Some("Reply"), Icon::Reply)}
                {icon("wrapped-c", Some("Trash"), Icon::Trash)}
            }
            div { style: "position:absolute;left:100px;top:200px;display:flex;gap:8px",
                {icon("bare-a", None, Icon::Archive)}
                {icon("bare-b", None, Icon::Reply)}
                {icon("bare-c", None, Icon::Trash)}
            }
        }
    }
}

fn started(app: fn() -> Element) -> Harness {
    let mut harness = Harness::new(app, HarnessConfig::new(VIEW).with_clock(Clock::Virtual));
    harness.advance(ms(200));
    harness
}

fn tip_text(harness: &Harness) -> Option<String> {
    harness.text_of(".ds-tooltip")
}

#[test]
fn a_title_under_the_standard_tip_delay_waits_cold_shows_at_once_warm_and_hides_on_a_press() {
    let mut harness = started(Slow);
    assert_eq!(harness.count(".ds-tooltip"), 0);
    let first = harness.centre("#first").expect("first");
    harness.send(Input::pointer_move(first));
    harness.advance(ms(700));
    assert_eq!(harness.count(".ds-tooltip"), 0, "not before the delay");
    harness.advance(ms(500));
    assert_eq!(tip_text(&harness).as_deref(), Some("Archive"));
    // Over the neighbour while the hub is warm: its tip stands at once.
    let second = harness.centre("#second").expect("second");
    harness.send(Input::pointer_move(second));
    harness.advance(ms(50));
    assert_eq!(
        tip_text(&harness).as_deref(),
        Some("Reply"),
        "warm: at once"
    );
    // A press removes it, and it does not come back while the pointer rests.
    harness.send(Input::pointer_down(second));
    harness.advance(ms(300));
    assert_eq!(harness.count(".ds-tooltip"), 0, "hidden on press");
    harness.send(Input::pointer_up(second));
    harness.advance(ms(1500));
    assert_eq!(
        harness.count(".ds-tooltip"),
        0,
        "not again until the pointer moves on"
    );
}

#[test]
fn a_title_shows_at_once_by_default_and_hides_on_a_press() {
    let mut harness = started(Titled);
    let first = harness.centre("#first").expect("first");
    harness.send(Input::pointer_move(first));
    harness.advance(ms(50));
    assert_eq!(tip_text(&harness).as_deref(), Some("Archive"), "no wait");
    harness.send(Input::pointer_down(first));
    harness.advance(ms(300));
    assert_eq!(harness.count(".ds-tooltip"), 0, "hidden on press");
}

#[test]
fn a_titled_control_has_no_native_title_and_keeps_its_name() {
    let harness = started(Titled);
    assert_eq!(harness.attr("#first", "title"), None);
    assert_eq!(
        harness.attr("#first", "aria-label").as_deref(),
        Some("first")
    );
}

#[test]
fn a_control_inside_a_tooltip_shows_that_one_tip_only() {
    let mut harness = started(Doubled);
    assert_eq!(
        harness.attr("#first", "title"),
        None,
        "no native tip either"
    );
    let first = harness.centre("#first").expect("first");
    harness.send(Input::pointer_move(first));
    harness.advance(ms(1300));
    assert_eq!(harness.count(".ds-tooltip"), 1);
    assert_eq!(tip_text(&harness).as_deref(), Some("Archive"));
}

#[test]
fn wrapping_an_item_in_its_tip_changes_neither_its_size_nor_its_neighbours() {
    let harness = started(Neutral);
    for item in ["a", "b", "c"] {
        let wrapped = harness.rect(&format!("#wrapped-{item}")).expect("wrapped");
        let bare = harness.rect(&format!("#bare-{item}")).expect("bare");
        assert_eq!(wrapped.size, bare.size, "{item}: size");
        assert_eq!(
            wrapped.origin.x, bare.origin.x,
            "{item}: place along the row"
        );
        assert_eq!(
            wrapped.origin.y.0 + 100.0,
            bare.origin.y.0 + 0.0,
            "{item}: place across the row"
        );
    }
}
