//! Under `Ds` a hint reaches the person through the hover hub's tooltip, never as the plain
//! `title` attribute: a provider mark, a pin tile and a Space dot write none, and hovering the
//! dot draws the tip. Outside `Ds` there is no hub, so the plain `title` stays.

use dioxus::prelude::*;
use ds::components::app::pin_tile::{PinFace, PinTile};
use ds::components::app::row_more::RowMore;
use ds::components::content::avatar::AvatarSize;
use ds::components::content::provider_mark::MarkProvider;
use ds::components::controls::chip::{Chip, ChipVariant};
use ds::components::controls::segmented::Tracking;
use ds::components::lists::row::row::{Outline, Row};
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
        SegmentedControl::<u8> {
            label: "View".to_owned(),
            choices: vec![
                Choice { icon: Some(Icon::Music.into()), name: Some("Music".to_owned()), ..Choice::new(1, "") },
                Choice::new(2, "Words"),
            ],
            tracking: Tracking::SelectOne(1),
            onchange: |_| {},
        }
        RowMore { shown: Some(Shown::Visible), onclick: |_| {} }
        Chip {
            variant: ChipVariant::Person(MarkProvider::Imap.avatar(AvatarSize::Size28)),
            text: "Ada".to_owned(),
            onremove: |_| {},
        }
        Row { title: "Folder", outline: Outline::Branch(Shown::Hidden), on_toggle: |_| {} }
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
        7,
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

/// Each named control with no words of its own, and the tip it shows: the name it carries.
const NAMED: &[(&str, &str)] = &[
    (".ds-segmented-segment:first-child", "Music"),
    (".ds-row-more", "More actions"),
    (".ds-chip-remove", "Remove Ada"),
    (".ds-row-disclosure", "Expand"),
];

#[test]
fn a_named_control_with_no_words_shows_its_name_as_a_tip() {
    for &(selector, tip) in NAMED {
        let mut harness = started(UnderDs);
        let at = harness
            .centre(selector)
            .unwrap_or_else(|| panic!("{selector} is not drawn:\n{}", harness.html()));
        harness.send(Input::pointer_move(at));
        for _ in 0..30 {
            harness.advance(Duration::from_millis(100));
        }
        assert_eq!(
            harness.text_of(".ds-tooltip").as_deref(),
            Some(tip),
            "{selector}"
        );
    }
}
