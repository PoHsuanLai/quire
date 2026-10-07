//! A glyph drawn with its parts moved: the frame of a part effect or a draw-on, as an `svg` whose
//! moving shapes sit in a `g` carrying the pose (design/35-SYMBOL-EFFECTS.md). Blitz's stylesheet
//! does not reach inside an SVG, so motion that moves a piece of one is written as attributes,
//! frame by frame, like `MorphGlyph`'s slash. The glyph is solid: its parts are the filled
//! shapes of [`Icon::solid_parts`], and a draw-on wipes it in from the left.

use super::Icon;
use super::parts::{Part, Tenths};
use super::render::{IconSize, shape_element};
use super::shape::Shape;
use super::solid_parts::SolidParts;
use dioxus::core::current_scope_id;
use dioxus::prelude::*;

/// A number in thousandths: 1000 is one (full size, full ink).
#[derive(Debug, Clone, Copy, PartialEq, Eq, Hash, PartialOrd, Ord, Default)]
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

/// `icon` at `size` with each of its [`Icon::parts`] at the pose given in the same order (a part
/// past the poses rests), wiped in from the left `drawn` thousandths of the way. The shapes in no
/// part never move.
#[component]
pub fn PosedGlyph(
    icon: Icon,
    size: IconSize,
    poses: Vec<PartPose>,
    #[props(default = Thousandths(1000))] drawn: Thousandths,
) -> Element {
    let px = size.px();
    let wipe = use_hook(|| format!("ds-wipe-{}", current_scope_id().0));
    let solid = icon.solid_parts().unwrap_or_else(|| SolidParts {
        still: icon.solid_shapes().to_vec(),
        parts: Vec::new(),
    });
    let parts = icon.parts().map_or(&[][..], |annotation| annotation.parts);
    let width = format!("{:.2}", 0.024 * drawn.0.clamp(0, 1000) as f32);
    let content = rsx! {
        for shape in solid.still.iter() {
            {shape_element(shape)}
        }
        for (number , part) in parts.iter().enumerate() {
            {part_group(part, poses.get(number).copied().unwrap_or(PartPose::REST), solid.parts.get(number))}
        }
    };
    rsx! {
        svg {
            class: "ds-ic",
            "data-size": "{px}",
            "data-style": "solid",
            width: "{px}",
            height: "{px}",
            view_box: "0 0 24 24",
            "aria-hidden": "true",
            "stroke": "none",
            "fill": "currentColor",
            if drawn.0 < 1000 {
                defs {
                    clipPath { id: "{wipe}",
                        rect { x: "0", y: "0", width: "{width}", height: "24" }
                    }
                }
            }
            if drawn.0 < 1000 {
                g {
                    opacity: (drawn.0 <= 0).then_some("0"),
                    "clip-path": (drawn.0 > 0).then(|| format!("url(#{wipe})")),
                    {content}
                }
            } else {
                {content}
            }
        }
    }
}

/// One part's `g`: its shapes under its pose.
fn part_group(part: &Part, pose: PartPose, shapes: Option<&Vec<Shape>>) -> Element {
    rsx! {
        g {
            transform: transform(part, pose),
            opacity: (pose.opacity != PartPose::REST.opacity).then(|| thousandths(pose.opacity)),
            for shape in shapes.into_iter().flatten() {
                {shape_element(shape)}
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
        // Ink is drawn on the group, not by the transform.
        let dimmed = PartPose {
            opacity: Thousandths(400),
            ..PartPose::REST
        };
        assert_eq!(transform(&part, dimmed), None);
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
        assert!(lid.contains("M4.5 4h15"), "{html}");
        assert!(
            !lid.contains("M5.5 8.5h13") && before.contains("M5.5 8.5h13"),
            "{html}"
        );
    }

    #[test]
    fn the_glyph_is_solid_a_part_at_rest_is_a_bare_group_and_a_layer_dims_by_opacity() {
        let rest = markup(Icon::Trash, vec![PartPose::REST], Thousandths(1000));
        assert!(
            rest.contains("<g>") && !rest.contains("transform"),
            "{rest}"
        );
        assert!(
            rest.contains("fill=\"currentColor\"") && rest.contains("stroke=\"none\""),
            "{rest}"
        );
        assert!(
            !rest.contains("stroke-width") && !rest.contains("dash"),
            "{rest}"
        );
        let dim = PartPose {
            opacity: Thousandths(300),
            ..PartPose::REST
        };
        let html = markup(Icon::Wifi, vec![dim; 4], Thousandths(1000));
        assert_eq!(html.matches("opacity=\"0.300\"").count(), 4, "{html}");
    }

    #[test]
    fn a_draw_on_wipes_from_the_left_and_a_fully_hidden_one_draws_nothing() {
        let half = markup(Icon::Trash, Vec::new(), Thousandths(500));
        assert!(
            half.contains("<clipPath") && half.contains("width=\"12.00\""),
            "{half}"
        );
        assert!(half.contains("clip-path=\"url(#ds-wipe-"), "{half}");
        let none = markup(Icon::Trash, Vec::new(), Thousandths(0));
        assert!(none.contains("opacity=\"0\""), "{none}");
        let whole = markup(Icon::Trash, Vec::new(), Thousandths(1000));
        assert!(!whole.contains("clip"), "{whole}");
    }
}
