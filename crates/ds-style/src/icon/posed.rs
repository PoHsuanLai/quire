//! A glyph drawn with its parts moved: the frame of a part effect or a draw-on, as an `svg` whose
//! moving shapes sit in a `g` carrying the pose (design/35-SYMBOL-EFFECTS.md). Blitz's stylesheet
//! does not reach inside an SVG, so motion that moves a piece of one is written as attributes,
//! frame by frame, like `MorphGlyph`'s slash.

use super::Icon;
use super::length::stroke_length;
use super::parts::{Part, Tenths};
use super::render::{IconSize, shape_element};
use super::shape::Shape;
use super::stroke::stroke_width;
use crate::scale::use_scale;
use dioxus::prelude::*;

/// A number in thousandths: 1000 is one (full size, full ink).
#[derive(Debug, Clone, Copy, PartialEq, Eq, Hash, PartialOrd, Ord)]
pub struct Thousandths(pub i32);

/// Where one part is in its motion: what to turn, shift, scale and tint it by.
#[derive(Debug, Clone, Copy, PartialEq, Eq, Hash)]
pub struct PartPose {
    /// Turn about the part's pivot, in tenths of a degree.
    pub rotate: Tenths,
    /// Shift across.
    pub dx: Tenths,
    /// Shift down.
    pub dy: Tenths,
    /// Scale across about the pivot.
    pub scale_x: Thousandths,
    /// Scale down about the pivot; negative flips.
    pub scale_y: Thousandths,
    /// How much of the part's ink shows.
    pub opacity: Thousandths,
    /// How much of the part's outline is filled in.
    pub fill: Thousandths,
}

impl PartPose {
    /// The part at rest.
    pub const REST: PartPose = PartPose {
        rotate: Tenths(0),
        dx: Tenths(0),
        dy: Tenths(0),
        scale_x: Thousandths(1000),
        scale_y: Thousandths(1000),
        opacity: Thousandths(1000),
        fill: Thousandths(0),
    };
}

/// `value` in tenths as a decimal, `-1.5`.
fn tenths(value: Tenths) -> String {
    let Tenths(value) = value;
    format!(
        "{}{}.{}",
        if value < 0 { "-" } else { "" },
        value.abs() / 10,
        value.abs() % 10
    )
}

/// `value` in thousandths as a decimal, `1.25`.
fn thousandths(value: Thousandths) -> String {
    let Thousandths(value) = value;
    format!(
        "{}{}.{:03}",
        if value < 0 { "-" } else { "" },
        value.abs() / 1000,
        value.abs() % 1000
    )
}

/// The `transform` that carries a part from rest to `pose` about `part`'s pivot.
fn transform(part: &Part, pose: PartPose) -> Option<String> {
    let moved = PartPose {
        opacity: PartPose::REST.opacity,
        fill: PartPose::REST.fill,
        ..pose
    };
    if moved == PartPose::REST {
        return None;
    }
    let (x, y) = (tenths(part.pivot.x), tenths(part.pivot.y));
    Some(format!(
        "translate({} {}) translate({x} {y}) rotate({}) scale({} {}) translate(-{x} -{y})",
        tenths(pose.dx),
        tenths(pose.dy),
        tenths(pose.rotate),
        thousandths(pose.scale_x),
        thousandths(pose.scale_y),
    ))
}

/// A shape's dash while `drawn` thousandths of it is drawn: the dash is as long as the stroke
/// and shifted back by what is still to draw.
fn dash(shape: &Shape, drawn: Thousandths) -> (String, String) {
    let length = stroke_length(shape);
    let left = length * (1000 - drawn.0.clamp(0, 1000)) as f32 / 1000.0;
    (format!("{length:.2}"), format!("{left:.2}"))
}

/// `shape`, drawn `drawn` thousandths of the way when that is under 1000.
fn drawn_shape(shape: &Shape, drawn: Thousandths) -> Element {
    if drawn.0 >= 1000 {
        return shape_element(shape);
    }
    // A round cap would leave a dot at the start of a stroke with nothing drawn.
    if drawn.0 <= 0 {
        return rsx! {
            g { opacity: "0", {shape_element(shape)} }
        };
    }
    let (array, offset) = dash(shape, drawn);
    rsx! {
        g { "stroke-dasharray": "{array}", "stroke-dashoffset": "{offset}", {shape_element(shape)} }
    }
}

/// `icon` at `size` with each of its [`Icon::parts`] at the pose given in the same order (a part
/// past the poses rests), and every stroke drawn `drawn` thousandths of the way. The shapes in no
/// part never move.
#[component]
pub fn PosedGlyph(
    icon: Icon,
    size: IconSize,
    poses: Vec<PartPose>,
    #[props(default = Thousandths(1000))] drawn: Thousandths,
) -> Element {
    let px = size.px();
    let stroke = stroke_width(size, use_scale());
    let shapes = icon.shapes();
    let parts = icon.parts().map_or(&[][..], |annotation| annotation.parts);
    let moving = |index: usize| parts.iter().any(|part| part.shapes.contains(&index));
    rsx! {
        svg {
            class: "ds-ic",
            "data-size": "{px}",
            width: "{px}",
            height: "{px}",
            view_box: "0 0 24 24",
            "aria-hidden": "true",
            "stroke": "currentColor",
            "stroke-width": stroke,
            "stroke-linecap": "round",
            "stroke-linejoin": "round",
            "fill": "none",
            for shape in shapes.iter().enumerate().filter(|(index, _)| !moving(*index)).map(|(_, shape)| shape) {
                {drawn_shape(shape, drawn)}
            }
            for (number , part) in parts.iter().enumerate() {
                {part_group(part, poses.get(number).copied().unwrap_or(PartPose::REST), shapes, drawn)}
            }
        }
    }
}

/// One part's `g`: its shapes under its pose.
fn part_group(
    part: &Part,
    pose: PartPose,
    shapes: &'static [Shape],
    drawn: Thousandths,
) -> Element {
    let fill = (pose.fill.0 > 0).then(|| thousandths(pose.fill));
    rsx! {
        g {
            transform: transform(part, pose),
            opacity: (pose.opacity != PartPose::REST.opacity).then(|| thousandths(pose.opacity)),
            fill: fill.as_ref().map(|_| "currentColor"),
            "fill-opacity": fill,
            for shape in part.shapes.iter().filter_map(|&index| shapes.get(index)) {
                {drawn_shape(shape, drawn)}
            }
        }
    }
}

#[cfg(test)]
mod tests {
    use super::{
        PartPose, PosedGlyph, PosedGlyphProps, Tenths, Thousandths, tenths, thousandths, transform,
    };
    use crate::icon::Icon;
    use crate::icon::parts::Part;
    use crate::icon::render::IconSize;
    use dioxus::prelude::*;

    #[test]
    fn numbers_read_as_decimals() {
        assert_eq!(tenths(Tenths(-15)), "-1.5");
        assert_eq!(tenths(Tenths(125)), "12.5");
        assert_eq!(thousandths(Thousandths(-500)), "-0.500");
        assert_eq!(thousandths(Thousandths(1250)), "1.250");
    }

    #[test]
    fn a_part_at_rest_has_no_transform_and_a_moved_one_turns_about_its_pivot() {
        let part = Part {
            shapes: &[0],
            pivot: crate::icon::parts::Pivot {
                x: Tenths(50),
                y: Tenths(60),
            },
        };
        assert_eq!(transform(&part, PartPose::REST), None);
        let tipped = PartPose {
            rotate: Tenths(-160),
            ..PartPose::REST
        };
        assert_eq!(
            transform(&part, tipped).as_deref(),
            Some(
                "translate(0.0 0.0) translate(5.0 6.0) rotate(-16.0) scale(1.000 1.000) translate(-5.0 -6.0)"
            )
        );
        // Ink and fill are drawn on the group, not by the transform.
        let flashed = PartPose {
            fill: Thousandths(400),
            ..PartPose::REST
        };
        assert_eq!(transform(&part, flashed), None);
    }

    fn markup(icon: Icon, poses: Vec<PartPose>, drawn: Thousandths) -> String {
        let mut dom = VirtualDom::new_with_props(
            PosedGlyph,
            PosedGlyphProps {
                icon,
                size: IconSize::Base,
                poses,
                drawn,
            },
        );
        dom.rebuild_in_place();
        dioxus_ssr::render(&dom)
    }

    #[test]
    fn a_tipped_lid_sits_in_a_group_with_its_pivot_and_the_body_stays_outside_it() {
        let tipped = PartPose {
            rotate: Tenths(-160),
            dy: Tenths(-10),
            ..PartPose::REST
        };
        let html = markup(Icon::Trash, vec![tipped], Thousandths(1000));
        let group = html.find("<g transform=").expect("the lid's group");
        let (before, after) = html.split_at(group);
        let lid = after.split("</g>").next().unwrap_or_default();
        assert!(
            lid.contains("rotate(-16.0)") && lid.contains("translate(5.0 6.0)"),
            "{html}"
        );
        assert!(lid.contains("M3 6h18") && lid.contains("M8 6V4"), "{html}");
        assert!(
            !lid.contains("M19 6v14") && before.contains("M19 6v14"),
            "{html}"
        );
    }

    #[test]
    fn a_part_at_rest_is_a_bare_group_and_a_layer_dims_by_opacity_and_a_pop_fills() {
        let rest = markup(Icon::Trash, vec![PartPose::REST], Thousandths(1000));
        assert!(
            rest.contains("<g>") && !rest.contains("transform"),
            "{rest}"
        );
        let dim = PartPose {
            opacity: Thousandths(300),
            fill: Thousandths(450),
            ..PartPose::REST
        };
        let html = markup(Icon::Wifi, vec![dim; 4], Thousandths(1000));
        assert_eq!(html.matches("opacity=\"0.300\"").count(), 4, "{html}");
        assert!(
            html.contains("fill=\"currentColor\"") && html.contains("fill-opacity=\"0.450\""),
            "{html}"
        );
    }

    #[test]
    fn a_stroke_draws_on_by_its_dash_and_a_fully_hidden_one_draws_nothing() {
        let half = markup(Icon::Trash, Vec::new(), Thousandths(500));
        assert!(
            half.contains("stroke-dasharray=\"18.00\"")
                && half.contains("stroke-dashoffset=\"9.00\""),
            "{half}"
        );
        let none = markup(Icon::Trash, Vec::new(), Thousandths(0));
        assert!(none.contains("opacity=\"0\""), "{none}");
        let whole = markup(Icon::Trash, Vec::new(), Thousandths(1000));
        assert!(!whole.contains("dash"), "{whole}");
    }
}
