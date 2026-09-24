//! An abstract icon's TOML spec (design/08-ICONS.md 2.8): the two-hue ground, the grain and
//! the filled layers, bottom to top. Parsing is pure: text in, a [`Spec`] or an error out.

use std::str::FromStr;

use serde::{Deserialize, Deserializer};

use crate::{Family, IconsError, Shape, Srgb8};

/// A colour written as a Candy family name (its `base`, 08 2.3) or as `#RRGGBB`.
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub struct Colour(pub Srgb8);

impl FromStr for Colour {
    type Err = IconsError;

    fn from_str(s: &str) -> Result<Self, Self::Err> {
        match s.strip_prefix('#') {
            Some(hex) if hex.len() == 6 => u32::from_str_radix(hex, 16)
                .map(|v| Colour(Srgb8::hex(v)))
                .map_err(|_| IconsError::UnknownColour(s.to_owned())),
            Some(_) => Err(IconsError::UnknownColour(s.to_owned())),
            None => s
                .parse::<Family>()
                .map(|f| Colour(f.stops().base))
                .map_err(|_| IconsError::UnknownColour(s.to_owned())),
        }
    }
}

impl<'de> Deserialize<'de> for Colour {
    fn deserialize<D: Deserializer<'de>>(d: D) -> Result<Self, D::Error> {
        String::deserialize(d)?
            .parse()
            .map_err(serde::de::Error::custom)
    }
}

/// The plate's two-hue ground: `start` at the top-left corner, `end` at the bottom-right
/// (135deg, 08 2.3), mixed in OKLCh along the shorter hue arc.
#[derive(Debug, Clone, Copy, PartialEq, Eq, Deserialize)]
pub struct Ground {
    pub start: Colour,
    pub end: Colour,
}

/// What a layer is filled with: paper, ink, one of the ground's two hues, or a colour.
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum Paint {
    Paper,
    Ink,
    Start,
    End,
    Exact(Colour),
}

impl<'de> Deserialize<'de> for Paint {
    fn deserialize<D: Deserializer<'de>>(d: D) -> Result<Self, D::Error> {
        let s = String::deserialize(d)?;
        match s.as_str() {
            "paper" => Ok(Paint::Paper),
            "ink" => Ok(Paint::Ink),
            "start" => Ok(Paint::Start),
            "end" => Ok(Paint::End),
            other => other
                .parse()
                .map(Paint::Exact)
                .map_err(serde::de::Error::custom),
        }
    }
}

/// How a layer sits in the plate: pressed up out of it, pressed into it, or flush.
#[derive(Debug, Clone, Copy, PartialEq, Eq, Default, Deserialize)]
#[serde(rename_all = "kebab-case")]
pub enum Relief {
    #[default]
    Raised,
    Recessed,
    Flush,
}

/// A layer's fill opacity, 0..=1; below 1 the ground shows through (the symbol is tinted by
/// the plate it is pressed into).
#[derive(Debug, Clone, Copy, PartialEq, Deserialize)]
pub struct Opacity(pub f32);

impl Default for Opacity {
    fn default() -> Self {
        Opacity(1.0)
    }
}

/// One filled layer.
#[derive(Debug, Clone, PartialEq, Deserialize)]
pub struct Layer {
    #[serde(flatten)]
    pub shape: Shape,
    pub fill: Paint,
    #[serde(default)]
    pub opacity: Opacity,
    #[serde(default)]
    pub relief: Relief,
}

/// Grain strength 0..=100, as a Space's grain (03-COLOR 8).
#[derive(Debug, Clone, Copy, PartialEq, Eq, Deserialize)]
pub struct Grain(pub u8);

/// One app's abstract icon.
#[derive(Debug, Clone, PartialEq, Deserialize)]
pub struct Spec {
    pub name: String,
    pub ground: Ground,
    pub grain: Grain,
    #[serde(rename = "layer")]
    pub layers: Vec<Layer>,
}

/// Parses a spec from TOML text.
pub fn parse_spec(text: &str) -> Result<Spec, IconsError> {
    Ok(toml::from_str(text)?)
}

/// The sRGB colour of a paint on a ground (paper and ink are Post light `--surface`, `--ink`).
pub fn paint(p: Paint, ground: Ground) -> Srgb8 {
    match p {
        Paint::Paper => Srgb8::hex(0xF8F9F6),
        Paint::Ink => Srgb8::hex(0x1A1E1A),
        Paint::Start => ground.start.0,
        Paint::End => ground.end.0,
        Paint::Exact(c) => c.0,
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::{Pt, mark::Heading};

    const TEXT: &str = r##"
name = "terminal"
ground = { start = "violet", end = "#F0A81E" }
grain = 30

[[layer]]
kind = "chevron"
tip = [11, 12]
reach = 4
heading = "right"
fill = "paper"

[[layer]]
kind = "rect"
at = [13, 15]
size = [6, 2.5]
corners = [1, 1, 1, 1]
fill = "ink"
opacity = 0.2
relief = "recessed"
"##;

    #[test]
    fn parses_layers_in_order() {
        let s = parse_spec(TEXT).expect("parses");
        assert_eq!(s.ground.start, Colour(Family::Violet.stops().base));
        assert_eq!(s.ground.end, Colour(Srgb8::hex(0xF0A81E)));
        assert_eq!(s.grain, Grain(30));
        assert_eq!(s.layers.len(), 2);
        assert_eq!(
            s.layers[0].shape,
            Shape::Chevron {
                tip: Pt(11.0, 12.0),
                reach: 4.0,
                heading: Heading::Right,
                width: crate::BAND,
            }
        );
        assert_eq!(s.layers[0].fill, Paint::Paper);
        assert_eq!(s.layers[0].opacity, Opacity(1.0));
        assert_eq!(s.layers[0].relief, Relief::Raised);
        assert_eq!(s.layers[1].relief, Relief::Recessed);
        assert_eq!(s.layers[1].opacity, Opacity(0.2));
    }

    #[test]
    fn rejects_unknown_colours_and_kinds() {
        assert!(parse_spec(&TEXT.replace("\"violet\"", "\"teal\"")).is_err());
        assert!(parse_spec(&TEXT.replace("#F0A81E", "#F0A8")).is_err());
        assert!(parse_spec(&TEXT.replace("chevron", "star")).is_err());
        assert!(parse_spec(&TEXT.replace("fill = \"paper\"", "fill = \"chalk\"")).is_err());
    }

    /// The five bundled specs parse and use their subject's name.
    #[test]
    fn bundled_specs_parse() {
        let specs = [
            ("mail", include_str!("../specs/mail.toml")),
            ("files", include_str!("../specs/files.toml")),
            ("terminal", include_str!("../specs/terminal.toml")),
            ("notes", include_str!("../specs/notes.toml")),
            ("photos", include_str!("../specs/photos.toml")),
        ];
        for (name, text) in specs {
            let s = parse_spec(text).unwrap_or_else(|e| panic!("{name}: {e}"));
            assert_eq!(s.name, name);
            assert!(!s.layers.is_empty(), "{name}");
        }
    }
}
