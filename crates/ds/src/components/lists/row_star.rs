//! A thread row's star (design/04-COMPONENTS.md section 16): the button that toggles it. Split
//! from `list_row` so the row's own file holds the row.

use crate::core::vocab::Check;
use crate::focus::click::kept_click;
use crate::style::icon::Icon;
use crate::style::icon::shape::Shape;
use dioxus::prelude::*;

/// The star's glyph: the outline, filled with its own colour once starred. `Glyph` only
/// strokes, and a CSS `fill` never reaches SVG on Blitz (spike S6), so the fill is written as an
/// attribute here.
fn star_glyph(state: Check) -> Element {
    let fill = match state {
        Check::On => "currentColor",
        Check::Off | Check::Mixed => "none",
    };
    rsx! {
        svg {
            class: "ds-ic",
            "data-size": "14",
            width: "14",
            height: "14",
            view_box: "0 0 24 24",
            "aria-hidden": "true",
            "stroke": "currentColor",
            "stroke-width": "2",
            "stroke-linecap": "round",
            "stroke-linejoin": "round",
            "fill": fill,
            for shape in Icon::Star.shapes() {
                if let Shape::Path(d) = shape {
                    path { d: "{d}" }
                }
            }
        }
    }
}

/// The star button.
pub(crate) fn star_button(state: Check, onchange: EventHandler<Check>) -> Element {
    let label = match state {
        Check::On => "Unstar this thread",
        Check::Off | Check::Mixed => "Star this thread",
    };
    rsx! {
        button {
            r#type: "button",
            class: "ds-star",
            "aria-pressed": state.aria(),
            "aria-label": label,
            onclick: move |event| {
                // The star acts on its own; the row must not also open.
                event.stop_propagation();
                onchange.call(state.flipped());
                kept_click(&event);
            },
            span { class: "ds-star-glyph", {star_glyph(state)} }
        }
    }
}
