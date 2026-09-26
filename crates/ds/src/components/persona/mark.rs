//! One flat shape of a persona: a path in one colour, filled or stroked with round ends
//! (design/24-PERSONA.md section 3.1: no outlines on filled shapes, no gradients).

/// How a mark is painted. Colours are hexes from the persona's palette, written as SVG
/// attributes (stylesheet rules do not reach SVG children on Blitz, spike S6).
#[derive(Debug, Clone, PartialEq)]
pub(crate) enum Paint {
    /// Filled.
    Fill(String),
    /// Stroked at a width in canvas units, round caps and joins.
    Stroke(String, f64),
    /// Filled and stroked in the same colour, to round a filled shape's corners.
    Rounded(String, f64),
}

/// One shape.
#[derive(Debug, Clone, PartialEq)]
pub(crate) struct Mark {
    pub d: String,
    pub paint: Paint,
    /// An SVG `transform`, for a shape turned about a point (a rabbit's ear, a bow).
    pub turn: Option<String>,
    /// An opacity below 1, for a blush.
    pub opacity: Option<&'static str>,
}

impl Paint {
    /// This paint with its colour passed through `recolour`.
    fn map(self, recolour: impl Fn(String) -> String) -> Paint {
        match self {
            Paint::Fill(colour) => Paint::Fill(recolour(colour)),
            Paint::Stroke(colour, width) => Paint::Stroke(recolour(colour), width),
            Paint::Rounded(colour, width) => Paint::Rounded(recolour(colour), width),
        }
    }
}

impl Mark {
    /// This mark with its colour passed through `recolour`.
    pub(crate) fn recoloured(self, recolour: impl Fn(String) -> String) -> Mark {
        Mark {
            paint: self.paint.map(recolour),
            ..self
        }
    }

    /// A filled shape.
    pub(crate) fn fill(d: String, colour: String) -> Mark {
        Mark {
            d,
            paint: Paint::Fill(colour),
            turn: None,
            opacity: None,
        }
    }

    /// A stroked line.
    pub(crate) fn stroke(d: String, colour: String, width: f64) -> Mark {
        Mark {
            d,
            paint: Paint::Stroke(colour, width),
            turn: None,
            opacity: None,
        }
    }

    /// A filled shape with rounded corners.
    pub(crate) fn rounded(d: String, colour: String, width: f64) -> Mark {
        Mark {
            d,
            paint: Paint::Rounded(colour, width),
            turn: None,
            opacity: None,
        }
    }

    /// This mark turned by `degrees` about `(x, y)`.
    pub(crate) fn turned(self, degrees: f64, (x, y): (f64, f64)) -> Mark {
        use super::geometry::num;
        Mark {
            turn: Some(format!("rotate({} {} {})", num(degrees), num(x), num(y))),
            ..self
        }
    }

    /// This mark at `opacity`.
    pub(crate) fn faded(self, opacity: &'static str) -> Mark {
        Mark {
            opacity: Some(opacity),
            ..self
        }
    }

    /// `(fill, stroke, stroke-width)` attribute values.
    pub(crate) fn attrs(&self) -> (String, String, String) {
        match &self.paint {
            Paint::Fill(colour) => (colour.clone(), "none".into(), "0".into()),
            Paint::Stroke(colour, width) => {
                ("none".into(), colour.clone(), super::geometry::num(*width))
            }
            Paint::Rounded(colour, width) => {
                (colour.clone(), colour.clone(), super::geometry::num(*width))
            }
        }
    }
}
