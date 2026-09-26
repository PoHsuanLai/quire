//! The battery ring's geometry as pure arithmetic (design/23-WIDGETS.md section 4.1): where the
//! track and the level's arc start and end on a 100-unit dial, and the SVG path for each, so a
//! table can pin every arc without rendering a ring.
//!
//! The ring is a stroke of [`STROKE`] units on a circle of [`RADIUS`] (the stroke's outer edge
//! touches the box: 0.093 of the diameter, as measured). The arc runs clockwise from twelve.
//! While charging, a gap is cut at twelve for the bolt: [`GAP`] degrees between the ends'
//! centres, so the visible gap between the round caps is about 17 degrees, as measured.

use crate::components::vocab::Fraction;

/// The stroke's width in the 100-unit box.
pub const STROKE: f32 = 9.3;

/// The circle the stroke is centred on.
pub const RADIUS: f32 = 50.0 - STROKE / 2.0;

/// Degrees between the charging gap's two end centres.
pub const GAP: f32 = 29.0;

/// Where the ring may be drawn: all the way round, or round from the gap's far side to its
/// near side while charging.
#[derive(Debug, Clone, Copy, PartialEq)]
pub struct Span {
    /// The start, in degrees clockwise from twelve.
    pub from: f32,
    /// How far round it goes, in degrees.
    pub sweep: f32,
}

impl Span {
    /// The whole circle.
    pub const FULL: Span = Span {
        from: 0.0,
        sweep: 360.0,
    };

    /// The circle less the bolt's gap at twelve.
    pub const GAPPED: Span = Span {
        from: GAP / 2.0,
        sweep: 360.0 - GAP,
    };

    /// The part of this span a `level` fills, from its start.
    pub fn filled(self, level: Fraction) -> Span {
        let share = f32::from(level.clamped().0) / 1000.0;
        Span {
            from: self.from,
            sweep: self.sweep * share,
        }
    }
}

/// A point on the ring's circle at `degrees` clockwise from twelve, written `x y`.
fn point(degrees: f32) -> String {
    let radians = degrees.to_radians();
    let x = 50.0 + RADIUS * radians.sin();
    let y = 50.0 - RADIUS * radians.cos();
    format!("{x:.2} {y:.2}")
}

/// The SVG path of `span` on the ring's circle. A whole circle is two half arcs (one arc cannot
/// end where it starts); an empty span is no path.
pub fn arc_path(span: Span) -> Option<String> {
    if span.sweep <= 0.0 {
        return None;
    }
    if span.sweep >= 360.0 {
        let half = span.from + 180.0;
        return Some(format!(
            "M{start}A{RADIUS} {RADIUS} 0 0 1 {mid}A{RADIUS} {RADIUS} 0 0 1 {start}Z",
            start = point(span.from),
            mid = point(half),
        ));
    }
    let large = if span.sweep > 180.0 { 1 } else { 0 };
    Some(format!(
        "M{}A{RADIUS} {RADIUS} 0 {large} 1 {}",
        point(span.from),
        point(span.from + span.sweep),
    ))
}

#[cfg(test)]
mod tests {
    use super::{Span, arc_path};
    use crate::components::vocab::Fraction;

    #[test]
    fn an_arc_fills_its_share_of_the_span() {
        let cases = [
            (Span::FULL, 0, 0.0),
            (Span::FULL, 500, 180.0),
            (Span::FULL, 1000, 360.0),
            (Span::FULL, 1500, 360.0),
            (Span::GAPPED, 1000, 331.0),
            (Span::GAPPED, 0, 0.0),
        ];
        for (span, permille, want) in cases {
            let got = span.filled(Fraction(permille)).sweep;
            assert!((got - want).abs() < 0.01, "{permille}: {got} != {want}");
        }
        assert_eq!(Span::GAPPED.filled(Fraction(400)).from, 14.5);
    }

    #[test]
    fn paths_start_at_twelve_and_turn_clockwise() {
        let cases = [
            (
                Span {
                    from: 0.0,
                    sweep: 90.0,
                },
                Some("M50.00 4.65A45.35 45.35 0 0 1 95.35 50.00"),
            ),
            (
                Span {
                    from: 0.0,
                    sweep: 270.0,
                },
                Some("M50.00 4.65A45.35 45.35 0 1 1 4.65 50.00"),
            ),
            (
                Span::FULL,
                Some("M50.00 4.65A45.35 45.35 0 0 1 50.00 95.35A45.35 45.35 0 0 1 50.00 4.65Z"),
            ),
            (
                Span {
                    from: 0.0,
                    sweep: 0.0,
                },
                None,
            ),
        ];
        for (span, want) in cases {
            assert_eq!(arc_path(span).as_deref(), want, "{span:?}");
        }
    }
}
