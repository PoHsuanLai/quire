//! The slash across a glyph that is off, drawn solid: a capsule from the top left corner that
//! grows towards the bottom right as it is drawn on, and the gap it cuts in the glyph under it
//! (design/08-ICONS.md section 1.2, rule 8). The macOS speaker, Wi-Fi and Bluetooth slashes read
//! the same way: a solid bar with a clear margin either side.

use super::posed::Thousandths;
use super::shape::Shape;
use dioxus::prelude::*;

/// The slash's thickness on the 24 grid.
const WIDTH: f32 = 2.2;
/// The clear margin each side of the slash, cut out of the glyph under it.
const MARGIN: f32 = 1.2;
/// Where the slash starts and ends on the 24 grid.
const FROM: (f32, f32) = (3.0, 3.0);
const TO: (f32, f32) = (21.0, 21.0);

/// A capsule of `width` from `from` to `to`, as a filled path (a 0 length is a disc).
fn capsule(from: (f32, f32), to: (f32, f32), width: f32) -> String {
    let (dx, dy) = (to.0 - from.0, to.1 - from.1);
    let length = dx.hypot(dy);
    let (ux, uy) = if length > 0.0 {
        (dx / length, dy / length)
    } else {
        (1.0, 0.0)
    };
    let h = width / 2.0;
    let (nx, ny) = (-uy * h, ux * h);
    format!(
        "M{:.2} {:.2}L{:.2} {:.2}A{h} {h} 0 0 0 {:.2} {:.2}L{:.2} {:.2}A{h} {h} 0 0 0 {:.2} {:.2}Z",
        from.0 + nx,
        from.1 + ny,
        to.0 + nx,
        to.1 + ny,
        to.0 - nx,
        to.1 - ny,
        from.0 - nx,
        from.1 - ny,
        from.0 + nx,
        from.1 + ny,
    )
}

/// The end of the slash when `drawn` thousandths of it is drawn.
fn tip(drawn: Thousandths) -> (f32, f32) {
    let share = drawn.0.clamp(0, 1000) as f32 / 1000.0;
    (
        FROM.0 + (TO.0 - FROM.0) * share,
        FROM.1 + (TO.1 - FROM.1) * share,
    )
}

/// The slash drawn `drawn` thousandths of the way, as a path `d`; `None` when nothing is drawn.
pub fn slash_path(drawn: Thousandths) -> Option<String> {
    (drawn.0 > 0).then(|| capsule(FROM, tip(drawn), WIDTH))
}

/// The whole slash as a shape, for a glyph that holds it still (the level glyph's slash layer).
pub const SLASH_SHAPE: Shape = Shape::Solid(
    "M2.22 3.78L20.22 21.78A1.1 1.1 0 0 0 21.78 20.22L3.78 2.22A1.1 1.1 0 0 0 2.22 3.78Z",
);

/// The slash as the `path` that paints it, in `currentColor`.
pub fn slash_mark(drawn: Thousandths) -> Element {
    match slash_path(drawn) {
        Some(d) => rsx! { path { d, fill: "currentColor", "data-drawn": "{drawn.0.min(1000)}" } },
        None => rsx! {},
    }
}

/// `children` with the gap the slash drawn `drawn` thousandths of the way leaves around itself
/// cut out, as a `g` under a mask. Put it inside an `svg` on the 24 grid; with nothing drawn it is
/// the children alone. `id` names the mask, unique in the document.
#[component]
pub fn Cut(id: String, drawn: Thousandths, children: Element) -> Element {
    let Some(cut) = (drawn.0 > 0).then(|| capsule(FROM, tip(drawn), WIDTH + 2.0 * MARGIN)) else {
        return rsx! { {children} };
    };
    rsx! {
        defs {
            mask { id: "{id}", "maskUnits": "userSpaceOnUse", x: "0", y: "0", width: "24", height: "24",
                rect { x: "0", y: "0", width: "24", height: "24", fill: "white" }
                path { d: "{cut}", fill: "black" }
            }
        }
        g { mask: "url(#{id})", {children} }
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn nothing_drawn_is_no_path_and_a_full_slash_spans_the_glyph() {
        assert_eq!(slash_path(Thousandths(0)), None);
        let full = slash_path(Thousandths(1000)).unwrap_or_default();
        assert!(full.starts_with("M2.22 3.78L20.22 21.78"), "{full}");
        assert!(
            matches!(SLASH_SHAPE, Shape::Solid(d) if d == full),
            "{full}"
        );
    }

    #[test]
    fn a_slash_part_way_ends_part_way_along() {
        assert_eq!(tip(Thousandths(500)), (12.0, 12.0));
        assert_eq!(tip(Thousandths(5000)), TO);
    }
}
