//! mailo gaps 4, item 4: the Space editor's Motion row with `MotionLevels::Contact` offers the
//! three levels a Space sets, and a pick among them is reported.

use dioxus::prelude::*;
use ds::{
    Appearance, DotIndex, Ds, Material, Motion, MotionChoice, MotionLevels, Scheme, SpaceEditor,
    SpaceLook,
};
use ds_native::{Harness, Viewport};
use std::time::Duration;

const VIEW: Viewport = Viewport {
    width: 420,
    height: 1100,
    scale_percent: 100,
};

/// A controlled editor offering the Contact levels, its motion shown beside it.
#[allow(non_snake_case)]
fn Editor() -> Element {
    let mut motion = use_signal(|| Motion::Standard);
    rsx! {
        Ds { appearance: Appearance::default(), material: Material::Sheet,
            p { class: "motion", "{motion().slug()}" }
            div { style: "width:380px",
                SpaceEditor {
                    look: SpaceLook::default(),
                    scheme: Scheme::Light,
                    active_dot: DotIndex(0),
                    onchange: |_| {},
                    motion: MotionChoice { level: motion(), on_motion: EventHandler::new(move |next| motion.set(next)) },
                    motion_levels: MotionLevels::Contact,
                }
            }
        }
    }
}

#[test]
fn the_contact_row_offers_three_levels_and_reports_a_pick() {
    let mut harness = Harness::new(Editor, VIEW);
    let row = "[*|aria-label=Motion] .ds-segment";
    assert_eq!(harness.count(row), 3);
    let names: Vec<String> = (1..=3)
        .filter_map(|n| harness.text_of(&format!("{row}:nth-child({n})")))
        .collect();
    assert_eq!(names, ["Calm", "Standard", "Extra"]);

    assert_eq!(harness.text_of(".motion").as_deref(), Some("standard"));
    let extra = format!("{row}:nth-child(3)");
    harness.click(harness.centre(&extra).expect("Extra"));
    harness.advance(Duration::from_millis(30));
    assert_eq!(harness.text_of(".motion").as_deref(), Some("extra"));
    assert_eq!(
        harness.attr(&extra, "aria-pressed").as_deref(),
        Some("true")
    );
}
