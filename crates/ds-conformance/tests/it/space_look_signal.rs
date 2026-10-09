//! The Space a `Ds` root draws is readable under it (`use_space_look`, `use_space_frame`), and
//! the reading changes when the app hands the root another look: a switch of Space or an edit
//! of its theme.

use crate::window_frame_controls::{VIEW, centre, ms};
use dioxus::prelude::*;
use ds::base::word::Word;
use ds::prelude::*;
use ds::style::space::frame_vars::FrameVars;
use ds_harness::{Clock, Driver, Harness, HarnessConfig, Input, Query};

fn look_of(theme: Theme) -> SpaceLook {
    SpaceLook {
        theme,
        ..SpaceLook::default()
    }
}

#[component]
fn Reader() -> Element {
    let look = use_space_look();
    let frame = use_space_frame();
    rsx! { p { class: "read", "{look.theme.slug()}|{frame.solid}" } }
}

#[allow(non_snake_case)]
fn Switching() -> Element {
    let mut theme = use_signal(|| Theme::Light);
    rsx! {
        Ds { appearance: Appearance::default(), material: Material::Window, look: look_of(theme()),
            button { class: "dark", onclick: move |_| theme.set(Theme::Dark), "Dark" }
            Reader {}
        }
    }
}

fn reading(theme: Theme, scheme: Scheme) -> String {
    format!(
        "{}|{}",
        theme.slug(),
        FrameVars::of(&look_of(theme), scheme).solid
    )
}

#[test]
fn the_look_and_frame_under_a_root_follow_the_look_it_is_given() {
    let mut harness = Harness::new(
        Switching,
        HarnessConfig::new(VIEW).with_clock(Clock::Virtual),
    );
    harness.advance(ms(50));
    let light = reading(Theme::Light, Scheme::Light);
    assert_eq!(harness.text_of(".read").as_deref(), Some(light.as_str()));
    let dark = reading(Theme::Dark, Scheme::Dark);
    assert_ne!(
        light, dark,
        "the two readings must differ for the test to see a change"
    );
    let button = centre(&harness, ".dark");
    harness.send(Input::click(button));
    harness.advance(ms(50));
    assert_eq!(harness.text_of(".read").as_deref(), Some(dark.as_str()));
}
