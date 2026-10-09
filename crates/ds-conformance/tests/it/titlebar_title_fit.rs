//! A title too long for the titlebar ends in an ellipsis in its middle and keeps its extension,
//! as Finder shows a long file name, on a real Blitz document.

use dioxus::prelude::*;
use ds::prelude::*;
use ds_harness::{Clock, Driver, Harness, HarnessConfig, Query, Viewport};
use std::time::Duration;

const LONG: &str = "a-very-long-holiday-picture-name-that-keeps-on-going-and-going-2026.png";

fn view(width: u32) -> Viewport {
    Viewport {
        width,
        height: 240,
        scale_percent: 100,
    }
}

fn titled(title: &'static str) -> Element {
    rsx! {
        Ds {
            appearance: Appearance::default(),
            material: Material::Window,
            window: WindowFrame::titlebar(title, TrafficLights::Shown),
            p { "body" }
        }
    }
}

#[allow(non_snake_case)]
fn Long() -> Element {
    titled(LONG)
}

#[allow(non_snake_case)]
fn Short() -> Element {
    titled("Inbox.png")
}

fn start(app: fn() -> Element, width: u32) -> Harness {
    let mut harness = Harness::new(
        app,
        HarnessConfig::new(view(width)).with_clock(Clock::Virtual),
    );
    for _ in 0..10 {
        harness.advance(Duration::from_millis(100));
    }
    harness
}

fn name(harness: &Harness) -> String {
    harness
        .text_of(".ds-titlebar-name")
        .unwrap_or_else(|| panic!("no title:\n{}", harness.html()))
}

#[test]
fn a_title_too_long_for_the_titlebar_ends_in_an_ellipsis_in_the_middle() {
    let harness = start(Long, 600);
    let shown = name(&harness);
    let (head, tail) = shown
        .split_once('…')
        .unwrap_or_else(|| panic!("{shown:?} has no ellipsis"));
    assert!(
        LONG.starts_with(head) && !head.is_empty(),
        "the start stays: {shown:?}"
    );
    assert!(
        LONG.ends_with(tail) && tail.ends_with(".png"),
        "the end and the extension stay: {shown:?}"
    );
    let title = harness.rect(".ds-titlebar-title").expect("the title area");
    let drawn = harness.rect(".ds-titlebar-name").expect("the name");
    assert!(
        drawn.size.width.0 <= title.size.width.0 + 0.5,
        "the shortened name {} fits the {} px it has",
        drawn.size.width.0,
        title.size.width.0
    );
}

#[test]
fn a_wider_window_shows_more_of_the_title_and_a_short_one_all_of_it() {
    let narrow = name(&start(Long, 400));
    let wide = name(&start(Long, 900));
    assert_eq!(wide, LONG, "900 px holds the whole title");
    assert!(
        narrow.chars().count() < LONG.chars().count() && narrow.contains('…'),
        "{narrow:?}"
    );
    assert_eq!(name(&start(Short, 400)), "Inbox.png");
}
