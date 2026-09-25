//! peniko gradients as krilla's: linear, radial (two-point conical) and sweep map one to one.
//! Stops interpolate in sRGB whatever colour space the gradient names (PDF shadings here are
//! sRGB), and a stop's alpha becomes its opacity.

use crate::geometry;
use crate::paint::{opacity, rgb_of};
use krilla::num::NormalizedF32;
use krilla::paint::{
    LinearGradient, Paint as KrillaPaint, RadialGradient, SpreadMethod, Stop, SweepGradient,
};
use peniko::kurbo::Affine;
use peniko::{Extend, Gradient, GradientKind};

/// `gradient` as a krilla paint, placed by `brush_transform` in the shape's space.
pub(crate) fn paint(gradient: &Gradient, brush_transform: Affine) -> KrillaPaint {
    let transform = geometry::transform(brush_transform);
    let spread_method = spread(gradient.extend);
    let stops = stops(gradient);
    match gradient.kind {
        GradientKind::Linear(line) => LinearGradient {
            x1: line.start.x as f32,
            y1: line.start.y as f32,
            x2: line.end.x as f32,
            y2: line.end.y as f32,
            transform,
            spread_method,
            stops,
            anti_alias: false,
        }
        .into(),
        GradientKind::Radial(radial) => RadialGradient {
            fx: radial.start_center.x as f32,
            fy: radial.start_center.y as f32,
            fr: radial.start_radius,
            cx: radial.end_center.x as f32,
            cy: radial.end_center.y as f32,
            cr: radial.end_radius,
            transform,
            spread_method,
            stops,
            anti_alias: false,
        }
        .into(),
        GradientKind::Sweep(sweep) => SweepGradient {
            cx: sweep.center.x as f32,
            cy: sweep.center.y as f32,
            start_angle: sweep.start_angle.to_degrees(),
            end_angle: sweep.end_angle.to_degrees(),
            transform,
            spread_method,
            stops,
            anti_alias: false,
        }
        .into(),
    }
}

fn spread(extend: Extend) -> SpreadMethod {
    match extend {
        Extend::Pad => SpreadMethod::Pad,
        Extend::Repeat => SpreadMethod::Repeat,
        Extend::Reflect => SpreadMethod::Reflect,
    }
}

fn stops(gradient: &Gradient) -> Vec<Stop> {
    gradient
        .stops
        .iter()
        .map(|stop| {
            let (color, alpha) = rgb_of(stop.color.to_alpha_color());
            Stop {
                offset: NormalizedF32::new(stop.offset.clamp(0.0, 1.0))
                    .unwrap_or(NormalizedF32::ZERO),
                color: color.into(),
                opacity: opacity(alpha),
            }
        })
        .collect()
}

#[cfg(test)]
mod tests {
    use super::stops;
    use peniko::{Color, Gradient};

    #[test]
    fn stops_keep_offsets_and_alpha() {
        let gradient = Gradient::new_linear((0.0, 0.0), (10.0, 0.0)).with_stops([
            (0.0, Color::from_rgba8(255, 0, 0, 255)),
            (1.0, Color::from_rgba8(0, 0, 255, 0)),
        ]);
        let made = stops(&gradient);
        assert_eq!(made.len(), 2);
        assert_eq!(made[1].offset.get(), 1.0);
        assert_eq!(made[1].opacity.get(), 0.0);
    }
}
