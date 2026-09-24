//! An abstract icon's TOML spec (design/08-ICONS.md 2.8): the plate family, the grain and the
//! layers, bottom to top. Parsing is pure: text in, a [`Spec`] or an error out.

use serde::{Deserialize, Deserializer};

use crate::{Family, IconsError, Shape, Srgb8};

/// The four colours an abstract icon may use besides its plate: the family's two tones, its
/// soft tint, paper and ink (07-LOOKS 3.1 Post light `--surface` and `--ink`).
#[derive(Debug, Clone, Copy, PartialEq, Eq, Deserialize)]
#[serde(rename_all = "kebab-case")]
pub enum Paint {
    Base,
    Deep,
    Soft,
    Paper,
    Ink,
}

/// Whether a cut-out layer sits lifted off what is under it (a hard-edged paper shadow).
#[derive(Debug, Clone, Copy, PartialEq, Eq, Default, Deserialize)]
#[serde(rename_all = "kebab-case")]
pub enum Lift {
    #[default]
    Flat,
    Lifted,
}

/// One layer: a shape with an optional fill and an optional outline at the one stroke weight.
/// Open shapes (chevron, bar) use `stroke` only.
#[derive(Debug, Clone, PartialEq, Deserialize)]
pub struct Layer {
    #[serde(flatten)]
    pub shape: Shape,
    #[serde(default)]
    pub fill: Option<Paint>,
    #[serde(default)]
    pub stroke: Option<Paint>,
    #[serde(default)]
    pub lift: Lift,
}

/// Grain strength 0..=100, as a Space's grain (03-COLOR 8).
#[derive(Debug, Clone, Copy, PartialEq, Eq, Deserialize)]
pub struct Grain(pub u8);

/// One app's abstract icon.
#[derive(Debug, Clone, PartialEq, Deserialize)]
pub struct Spec {
    pub name: String,
    #[serde(deserialize_with = "family")]
    pub family: Family,
    pub grain: Grain,
    #[serde(rename = "layer")]
    pub layers: Vec<Layer>,
}

fn family<'de, D: Deserializer<'de>>(d: D) -> Result<Family, D::Error> {
    let name = String::deserialize(d)?;
    name.parse().map_err(serde::de::Error::custom)
}

/// Parses a spec from TOML text.
pub fn parse_spec(text: &str) -> Result<Spec, IconsError> {
    Ok(toml::from_str(text)?)
}

/// The sRGB colour of a paint on a family's plate (08 2.3 base/deep/soft).
pub fn paint(p: Paint, family: Family) -> Srgb8 {
    match p {
        Paint::Base => family.stops().base,
        Paint::Deep => family.stops().deep,
        Paint::Soft => family.soft(),
        Paint::Paper => Srgb8::hex(0xF8F9F6),
        Paint::Ink => Srgb8::hex(0x1A1E1A),
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::{Pt, mark::Heading};

    const TEXT: &str = r#"
name = "terminal"
family = "violet"
grain = 30

[[layer]]
kind = "chevron"
tip = [11, 12]
reach = 4
heading = "right"
stroke = "ink"

[[layer]]
kind = "rect"
at = [13, 15]
size = [6, 2.5]
corners = [1, 1, 1, 1]
fill = "paper"
lift = "lifted"
"#;

    #[test]
    fn parses_layers_in_order() {
        let s = parse_spec(TEXT).expect("parses");
        assert_eq!(s.family, Family::Violet);
        assert_eq!(s.grain, Grain(30));
        assert_eq!(s.layers.len(), 2);
        assert_eq!(
            s.layers[0].shape,
            Shape::Chevron {
                tip: Pt(11.0, 12.0),
                reach: 4.0,
                heading: Heading::Right
            }
        );
        assert_eq!(s.layers[0].stroke, Some(Paint::Ink));
        assert_eq!(s.layers[0].lift, Lift::Flat);
        assert_eq!(s.layers[1].lift, Lift::Lifted);
    }

    #[test]
    fn rejects_unknown_family_and_kind() {
        assert!(parse_spec(&TEXT.replace("violet", "teal")).is_err());
        assert!(parse_spec(&TEXT.replace("chevron", "star")).is_err());
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
