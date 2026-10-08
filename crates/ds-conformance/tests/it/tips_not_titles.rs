//! Under `Ds` a hint reaches the person through the hover hub's tooltip, never as the plain
//! `title` attribute: a provider mark, a pin tile and a Space dot write none, and hovering the
//! dot draws the tip. Outside `Ds` there is no hub, so the plain `title` stays.

use dioxus::prelude::*;
use ds::components::app::pin_tile::{PinFace, PinTile};
use ds::components::content::provider_mark::MarkProvider;
use ds::prelude::*;
use ds::style::space::frame_vars::FrameVars;
use ds_harness::{Clock, Driver, Harness, HarnessConfig, Input, Query, Viewport};
use std::time::Duration;

const VIEW: Viewport = Viewport {
    width: 300,
    height: 200,
    scale_percent: 100,
};

fn parts() -> Element {
    rsx! {
        ProviderMark { provider: MarkProvider::Imap }
        PinTile {
            face: PinFace::Add { label: "Add account".to_owned(), hint: Some("Add account\u{2026}".to_owned()) },
            onclick: |_| {},
        }
        SpaceDot {
            name: "Space 1",
            frame: FrameVars::of(&SpaceLook::default(), Scheme::Light),
            selection: Selection::Unselected,
            shortcut: Shortcut(vec![ShortcutKey::Ctrl, ShortcutKey::Char('1')]),
            onclick: |_| {},
        }
    }
}

#[allow(non_snake_case)]
fn UnderDs() -> Element {
    rsx! {
        Ds { appearance: Appearance::default(), material: Material::Window, {parts()} }
    }
}

#[allow(non_snake_case)]
fn Bare() -> Element {
    parts()
}

fn started(app: fn() -> Element) -> Harness {
    let mut harness = Harness::new(app, HarnessConfig::new(VIEW).with_clock(Clock::Virtual));
    harness.advance(Duration::from_millis(100));
    harness
}

#[test]
fn under_ds_no_part_writes_a_title_attribute_and_outside_it_the_title_stays() {
    let under = started(UnderDs);
    assert!(
        !under.html().contains(" title="),
        "a bare title under Ds:\n{}",
        under.html()
    );
    let bare = started(Bare);
    assert_eq!(
        bare.html().matches(" title=").count(),
        3,
        "the plain title is the fallback without Ds:\n{}",
        bare.html()
    );
}

#[test]
fn hovering_a_space_dot_draws_its_tip() {
    let mut harness = started(UnderDs);
    assert_eq!(harness.count(".ds-tooltip"), 0);
    let at = harness.centre(".ds-space-dot").expect("the dot");
    harness.send(Input::pointer_move(at));
    for _ in 0..30 {
        harness.advance(Duration::from_millis(100));
    }
    assert!(
        harness
            .text_of(".ds-tooltip")
            .is_some_and(|text| text.contains("Space 1")),
        "{}",
        harness.html()
    );
}
