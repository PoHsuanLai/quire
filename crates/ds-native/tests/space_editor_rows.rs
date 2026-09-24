//! mailo gaps 2, step 5: the Space editor's consumer rows on a real Blitz document. Typing in
//! the name field renames the Space, the Motion row reports a pick, and a System Space is
//! measured in both schemes.

use dioxus::prelude::*;
use ds::{
    Appearance, DotIndex, Ds, Key, Material, MeasuredIn, Motion, MotionChoice, Scheme, SpaceEditor,
    SpaceLook,
};
use ds_native::{Harness, Viewport};
use std::time::Duration;

const VIEW: Viewport = Viewport {
    width: 420,
    height: 1100,
    scale_percent: 100,
};

fn ms(n: u64) -> Duration {
    Duration::from_millis(n)
}

/// A controlled editor: the name and motion live here, and are shown beside it.
#[allow(non_snake_case)]
fn Editor() -> Element {
    let mut name = use_signal(|| "Work".to_string());
    let mut motion = use_signal(|| Motion::System);
    rsx! {
        Ds { appearance: Appearance::default(), material: Material::Sheet,
            p { class: "name", "{name}" }
            p { class: "motion", "{motion().slug()}" }
            div { style: "width:380px",
                SpaceEditor {
                    look: SpaceLook::default(),
                    scheme: Scheme::Light,
                    active_dot: DotIndex(0),
                    onchange: |_| {},
                    name: name(),
                    on_rename: move |next: String| name.set(next),
                    motion: MotionChoice { level: motion(), on_motion: EventHandler::new(move |next| motion.set(next)) },
                    measured: MeasuredIn::EachScheme,
                }
            }
        }
    }
}

#[test]
fn typing_in_the_title_renames_the_space() {
    let mut harness = Harness::new(Editor, VIEW);
    assert_eq!(harness.text_of(".name").as_deref(), Some("Work"));
    let field = ".ds-space-editor-title .ds-input";
    harness.click(harness.centre(field).expect("the name field"));
    harness.advance(ms(30));
    assert!(harness.is_focused(field));
    harness.key(Key::Char('s'));
    harness.advance(ms(30));
    assert_eq!(harness.text_of(".name").as_deref(), Some("Works"));
    assert_eq!(harness.attr(field, "value").as_deref(), Some("Works"));
}

#[test]
fn the_motion_row_reports_a_pick() {
    let mut harness = Harness::new(Editor, VIEW);
    assert_eq!(harness.text_of(".motion").as_deref(), Some("system"));
    let reduced = "[*|aria-label=Motion] .ds-segment:nth-child(5)";
    assert_eq!(harness.text_of(reduced).as_deref(), Some("Reduced"));
    harness.click(harness.centre(reduced).expect("Reduced"));
    harness.advance(ms(30));
    assert_eq!(harness.text_of(".motion").as_deref(), Some("reduced"));
    assert_eq!(
        harness.attr(reduced, "aria-pressed").as_deref(),
        Some("true")
    );
}

#[test]
fn a_system_space_is_measured_under_both_headings() {
    let harness = Harness::new(Editor, VIEW);
    assert_eq!(harness.count(".ds-checks-heading"), 2);
    assert_eq!(
        harness
            .text_of(".ds-checks-scheme:nth-child(2) .ds-checks-heading")
            .as_deref(),
        Some("Light")
    );
    assert_eq!(
        harness
            .text_of(".ds-checks-scheme:nth-child(3) .ds-checks-heading")
            .as_deref(),
        Some("Dark")
    );
    assert_eq!(harness.count(".ds-checks-scheme .ds-check"), 8);
}
