//! LayerGlyph: an icon drawn as stacked layers, one `svg` per shape, so a pending loop can light
//! them in turn and a settle can fill them (design/26-DETAILS.md section 3.2: "a Rust step timer
//! sets `data-lit` on stacked layers, as `LevelGlyph` does").

use super::pending::{Lit, PendingFrame, PendingSpec};
use crate::icon::Icon;
use crate::icon::render::IconSize;
use crate::icon::shape::Shape;
use crate::icon::stroke::stroke_width;
use crate::root::use_scale;
use dioxus::prelude::*;

/// Which layers a `LayerGlyph` lights.
#[derive(Debug, Clone, Copy, PartialEq, Eq, Hash, Default)]
pub enum Layering {
    /// Every layer: the glyph as it is.
    #[default]
    Whole,
    /// A pending loop's frame (`use_pending`): its lit layers; the whole glyph dimmed when
    /// `Stalled`.
    Pending(PendingFrame, PendingSpec),
    /// A cumulative fill settling (`Settling::Filling`): layers up to this one.
    Filling(u8),
}

impl Layering {
    /// Whether `layer` is lit.
    fn lit(self, layer: u8) -> Lit {
        match self {
            Layering::Whole => Lit::On,
            Layering::Pending(frame, spec) => frame.lit(spec, layer),
            Layering::Filling(upto) if layer <= upto => Lit::On,
            Layering::Filling(_) => Lit::Off,
        }
    }

    /// The `data-pending` word: `still` when a pending loop holds its still frame.
    fn slug(self) -> &'static str {
        match self {
            Layering::Pending(frame, _) => frame.slug(),
            Layering::Whole | Layering::Filling(_) => "idle",
        }
    }
}

/// The layer each of `icon`'s shapes is, from the inside out: a Wi-Fi glyph's dot, then its arcs
/// from the smallest (Lucide draws the largest first). Other icons layer in drawing order.
fn layer_of(icon: Icon, shape: usize, count: usize) -> u8 {
    let place = match icon {
        Icon::Wifi if shape > 0 => count - shape,
        _ => shape,
    };
    u8::try_from(place).unwrap_or(u8::MAX)
}

/// `icon` at `size` as stacked layers (`span.ds-layer-glyph`, one `svg.ds-ic.ds-layer-part` per
/// shape, `data-layer` its place from the inside out, `data-lit` on or off). Lit layers are
/// drawn, unlit ones faint; a stalled pending loop dims the whole glyph. Decorative.
#[component]
pub fn LayerGlyph(icon: Icon, size: IconSize, #[props(default)] layering: Layering) -> Element {
    let px = size.px();
    let stroke = stroke_width(size, use_scale());
    let shapes = icon.shapes();
    let count = shapes.len();
    rsx! {
        span { class: "ds-layer-glyph", "data-pending": layering.slug(), "aria-hidden": "true",
            for (index, shape) in shapes.iter().enumerate() {
                svg {
                    key: "{index}",
                    class: "ds-ic ds-layer-part",
                    "data-layer": "{layer_of(icon, index, count)}",
                    "data-lit": match layering.lit(layer_of(icon, index, count)) { Lit::On => "on", Lit::Off => "off" },
                    width: "{px}",
                    height: "{px}",
                    view_box: "0 0 24 24",
                    "stroke": "currentColor",
                    "stroke-width": stroke.clone(),
                    "stroke-linecap": "round",
                    "stroke-linejoin": "round",
                    "fill": "none",
                    {shape_child(shape)}
                }
            }
        }
    }
}

fn shape_child(shape: &Shape) -> Element {
    match shape {
        Shape::Path(d) => rsx! { path { d: "{d}" } },
        Shape::Circle { cx, cy, r } => rsx! { circle { cx: "{cx}", cy: "{cy}", r: "{r}" } },
        Shape::Rect {
            x,
            y,
            width,
            height,
            rx,
        } => rsx! {
            rect { x: "{x}", y: "{y}", width: "{width}", height: "{height}", rx: "{rx}" }
        },
    }
}

#[cfg(test)]
mod tests {
    use super::{Layering, layer_of};
    use crate::detail::{Layers, Lit, PendingFrame, PendingSpec, PendingStyle};
    use crate::icon::Icon;

    #[test]
    fn a_wifi_glyph_layers_from_the_dot_out() {
        let order: Vec<u8> = (0..4).map(|shape| layer_of(Icon::Wifi, shape, 4)).collect();
        assert_eq!(order, [0, 3, 2, 1]);
        assert_eq!(layer_of(Icon::Check, 0, 1), 0);
    }

    #[test]
    fn each_layering_lights_its_layers() {
        let spec = PendingSpec {
            style: PendingStyle::Iterate,
            layers: Layers(4),
        };
        let lit = |layering: Layering| (0..4).map(|l| layering.lit(l)).collect::<Vec<_>>();
        use Lit::{Off, On};
        assert_eq!(lit(Layering::Whole), [On, On, On, On]);
        assert_eq!(lit(Layering::Filling(1)), [On, On, Off, Off]);
        assert_eq!(
            lit(Layering::Pending(PendingFrame::Step(2), spec)),
            [Off, Off, On, Off]
        );
    }
}
