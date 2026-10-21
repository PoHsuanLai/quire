//! A source list's row under the pointer takes a faint wash (`--hover-wash`), fainter than the
//! selected row's, in light and in dark; the row the pointer left goes back to the ground.

use crate::support::probe;

use dioxus::prelude::*;
use ds::prelude::*;
use ds_harness::{Clock, Harness, HarnessConfig, Input, Viewport};
use probe::{distance, modal, pixels, rect};

const VIEW: Viewport = Viewport {
    width: 300,
    height: 140,
    scale_percent: 100,
};

fn row(id: &'static str, selected: bool) -> Element {
    rsx! {
        div { id, class: "ds-row", "data-focus": "highlight", "aria-selected": if selected { "true" } else { "false" },
            style: "height:28px", "Inbox"
        }
    }
}

fn stage(theme: Theme) -> Element {
    rsx! {
        Ds {
            appearance: Appearance { theme, ..Appearance::default() },
            material: Material::Window,
            div { class: "ds-list", "data-style": "source-list", style: "padding:8px",
                {row("selected", true)}
                {row("hovered", false)}
                {row("plain", false)}
            }
        }
    }
}

#[allow(non_snake_case)]
fn Light() -> Element {
    stage(Theme::Light)
}

#[allow(non_snake_case)]
fn Dark() -> Element {
    stage(Theme::Dark)
}

#[test]
fn a_row_under_the_pointer_is_washed_fainter_than_the_selection() {
    for (theme, root) in [("light", Light as fn() -> Element), ("dark", Dark)] {
        let mut harness = Harness::new(root, HarnessConfig::new(VIEW).with_clock(Clock::Virtual));
        let at = harness.centre("#hovered").expect("the row");
        harness.send(Input::pointer_move(at));
        let frame = harness.render().expect("renders");
        let fill = |id: &str| modal(&pixels(&frame, rect(&harness, id), 2.0));
        let (selected, hovered, plain) = (fill("#selected"), fill("#hovered"), fill("#plain"));
        assert!(
            distance(hovered, plain) >= 4,
            "{theme}: the hovered row {hovered:?} is the ground {plain:?}"
        );
        assert!(
            distance(hovered, plain) < distance(selected, plain),
            "{theme}: the hovered row {hovered:?} is as strong as the selection {selected:?}"
        );

        let away = harness.centre("#plain").expect("the row");
        harness.send(Input::pointer_move(away));
        let frame = harness.render().expect("renders");
        let fill = |id: &str| modal(&pixels(&frame, rect(&harness, id), 2.0));
        assert!(
            distance(fill("#hovered"), plain) < 2,
            "{theme}: the row the pointer left keeps its wash"
        );
    }
}
