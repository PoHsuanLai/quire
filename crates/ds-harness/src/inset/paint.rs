//! Whether an element paints an edge, from its computed style and layout: a fill that differs
//! from what is behind it, a gradient, a border on a side, an outline, a shadow.

use crate::inset::scene::{Edges, Paint};
use blitz_dom::Node;
use blitz_dom::util::ToColorColor;

/// What a box sits on, composited down the tree: an opaque colour, and the background image the
/// nearest ancestor that has one paints (a child painting the same one is not an edge).
#[derive(Debug, Clone, PartialEq)]
pub(crate) struct Ground {
    pub colour: [f32; 3],
    pub picture: String,
}

impl Ground {
    /// The page: white, with no image.
    pub(crate) fn page() -> Self {
        Ground {
            colour: [1.0, 1.0, 1.0],
            picture: String::new(),
        }
    }
}

/// Below this much difference in any channel a fill is the colour behind it.
const SAME: f32 = 0.01;

/// What `node` paints: its edges, why, and the ground its children sit on.
pub(crate) struct Painted {
    pub edges: Edges,
    pub paints: Vec<Paint>,
    pub ground: Ground,
}

/// The paint of `node` over `ground`; `None` when it has no computed style.
pub(crate) fn painted(node: &Node, ground: &Ground) -> Option<Painted> {
    let styles = node.primary_styles()?;
    let ink = styles.clone_color();
    let alpha_of = |colour: [f32; 4]| colour[3];
    let fill = styles
        .get_background()
        .background_color
        .resolve_to_absolute(&ink)
        .as_color_color()
        .components;
    let behind = ground.colour;
    let over = [0, 1, 2].map(|c| fill[c] * fill[3] + behind[c] * (1.0 - fill[3]));
    let differs = alpha_of(fill) > 0.0 && (0..3).any(|c| (over[c] - behind[c]).abs() > SAME);
    let marked = is_marked(node) && alpha_of(fill) > 0.0;
    // `background-clip: text` paints the text, not a box (a gradient-filled word).
    let text_clip = styles
        .get_background()
        .background_clip
        .0
        .iter()
        .any(|clip| format!("{clip:?}").to_ascii_lowercase().contains("text"));
    let images: Vec<String> = styles
        .get_background()
        .background_image
        .0
        .iter()
        .map(|layer| format!("{layer:?}"))
        .filter(|layer| layer != "none")
        .collect();
    let sheet = images.join(",");
    let picture = !text_clip && !images.is_empty() && sheet != ground.picture;
    let layout = node.unrounded_layout();
    let border = styles.get_border();
    macro_rules! shown {
        ($colour:expr) => {
            alpha_of(
                $colour
                    .resolve_to_absolute(&ink)
                    .as_color_color()
                    .components,
            ) > 0.0
        };
    }
    let left = layout.border.left > 0.0 && shown!(border.border_left_color);
    let right = layout.border.right > 0.0 && shown!(border.border_right_color);
    // Blitz paints no `auto` outline, and a style of `none` leaves the computed width at `medium`.
    let outline_style = format!("{:?}", styles.get_outline().outline_style).to_ascii_lowercase();
    let outline = !["none", "hidden", "auto"]
        .iter()
        .any(|word| outline_style.contains(word))
        && styles.get_outline().outline_width.0.to_f64_px() > 0.0
        && alpha_of(
            styles
                .get_outline()
                .outline_color
                .resolve_to_absolute(&ink)
                .as_color_color()
                .components,
        ) > 0.0;
    let shadow = styles.get_effects().box_shadow.0.iter().any(|shadow| {
        alpha_of(
            shadow
                .base
                .color
                .resolve_to_absolute(&ink)
                .as_color_color()
                .components,
        ) > 0.0
    });
    let mut paints = Vec::new();
    let mut flag = |on: bool, paint: Paint| {
        if on {
            paints.push(paint);
        }
    };
    flag(differs, Paint::Fill);
    flag(!differs && marked, Paint::Marked);
    flag(picture, Paint::Picture);
    flag(left || right, Paint::Border);
    flag(outline, Paint::Outline);
    flag(shadow, Paint::Shadow);
    let whole = differs || marked || picture || outline || shadow;
    let ground = Ground {
        colour: if alpha_of(fill) > 0.0 { over } else { behind },
        picture: if images.is_empty() {
            ground.picture.clone()
        } else {
            sheet
        },
    };
    Some(Painted {
        edges: Edges {
            left: whole || left,
            right: whole || right,
        },
        paints,
        ground,
    })
}

/// Whether the element says it is highlighted, selected or hovered.
fn is_marked(node: &Node) -> bool {
    let element = match node.element_data() {
        Some(element) => element,
        None => return false,
    };
    element.attrs().iter().any(|attribute| {
        let name = attribute.name.local.as_ref();
        let value = attribute.value.as_str();
        match name {
            "aria-selected" | "aria-current" | "aria-checked" | "aria-pressed" => value != "false",
            "data-selected" | "data-hover" | "data-hovered" | "data-active" | "data-posed" => {
                value != "false"
            }
            _ => false,
        }
    })
}
