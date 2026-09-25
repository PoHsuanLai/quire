//! The marks inside the traffic lights, drawn on a 12 px grid: the close cross and the minimize
//! bar stroked, the zoom and restore corners filled. Not glyphs from `Icon`: they are the
//! lights' own marks at a third of a glyph's size, with no Lucide equivalent for the pairs of
//! corners, and they are drawn in the light's deep ink rather than the text colour's weight.

use dioxus::prelude::*;

/// Which mark a light shows.
#[derive(Debug, Clone, Copy, PartialEq, Eq, Hash)]
pub(crate) enum Mark {
    /// The red light's cross.
    Close,
    /// The yellow light's bar.
    Minimize,
    /// The green light's outward corners: the window is its own size.
    Zoom,
    /// The green light's inward corners: the window is maximized.
    Restore,
}

impl Mark {
    /// The light's `data-light` word (the green light is `zoom` in either state).
    pub(crate) fn slug(self) -> &'static str {
        match self {
            Mark::Close => "close",
            Mark::Minimize => "minimize",
            Mark::Zoom | Mark::Restore => "zoom",
        }
    }

    /// The light's accessible name.
    pub(crate) fn label(self) -> &'static str {
        match self {
            Mark::Close => "Close",
            Mark::Minimize => "Minimize",
            Mark::Zoom => "Zoom",
            Mark::Restore => "Restore",
        }
    }

    /// The path, and whether it is stroked or filled.
    fn shape(self) -> (&'static str, Paint) {
        match self {
            Mark::Close => ("M3.8 3.8 8.2 8.2M8.2 3.8 3.8 8.2", Paint::Stroke),
            Mark::Minimize => ("M3.2 6H8.8", Paint::Stroke),
            Mark::Zoom => ("M3.5 3.5H7.3L3.5 7.3ZM8.5 8.5H4.7L8.5 4.7Z", Paint::Fill),
            Mark::Restore => ("M5.6 5.6H2.4L5.6 2.4ZM6.4 6.4H9.6L6.4 9.6Z", Paint::Fill),
        }
    }
}

/// How a mark is painted.
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
enum Paint {
    Stroke,
    Fill,
}

/// A light's mark: `svg.ds-light-mark`, in `currentColor`, hidden until the stylesheet shows it.
#[component]
pub(crate) fn LightMark(mark: Mark) -> Element {
    let (d, paint) = mark.shape();
    let (stroke, fill) = match paint {
        Paint::Stroke => ("currentColor", "none"),
        Paint::Fill => ("none", "currentColor"),
    };
    rsx! {
        svg {
            class: "ds-light-mark",
            // `data-ds-svg` tells the markup lint this vector is quire's own, not raw SVG.
            "data-ds-svg": "light",
            width: "12",
            height: "12",
            view_box: "0 0 12 12",
            "aria-hidden": "true",
            "stroke": stroke,
            "stroke-width": "1.2",
            "stroke-linecap": "round",
            "fill": fill,
            path { d }
        }
    }
}
