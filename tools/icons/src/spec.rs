//! An abstract icon's TOML spec (design/08-ICONS.md 2.8-2.10): the recommended dialect, the
//! tint from the muted palette, the grain, and the filled layers bottom to top, each in one of
//! the symbol's roles. Parsing is pure: text in, a [`Spec`] or an error out.

use serde::Deserialize;

use crate::{Dialect, IconsError, Shape, Tint};

/// Which of the dialect's colours a layer takes.
#[derive(Debug, Clone, Copy, PartialEq, Eq, Deserialize)]
#[serde(rename_all = "kebab-case")]
pub enum Role {
    /// The main shapes.
    Symbol,
    /// A quieter second shape on the plate.
    Secondary,
    /// Details pressed into the symbol.
    Detail,
    /// The one small accent (only the paper dialect shows it as a hue).
    Spot,
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

/// One filled layer.
#[derive(Debug, Clone, PartialEq, Deserialize)]
pub struct Layer {
    #[serde(flatten)]
    pub shape: Shape,
    pub fill: Role,
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
    /// The dialect this icon is recommended in; any dialect can render it.
    pub dialect: Dialect,
    pub tint: Tint,
    pub grain: Grain,
    #[serde(rename = "layer")]
    pub layers: Vec<Layer>,
}

/// Parses a spec from TOML text.
pub fn parse_spec(text: &str) -> Result<Spec, IconsError> {
    Ok(toml::from_str(text)?)
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::{Pt, mark::Heading};

    const TEXT: &str = r#"
name = "terminal"
dialect = "graphite"
tint = "plum"
grain = 20

[[layer]]
kind = "chevron"
tip = [11, 12]
reach = 4
heading = "right"
fill = "symbol"

[[layer]]
kind = "rect"
at = [13, 15]
size = [6, 2.5]
corners = [1, 1, 1, 1]
fill = "detail"
relief = "recessed"
"#;

    #[test]
    fn parses_layers_in_order() {
        let s = parse_spec(TEXT).expect("parses");
        assert_eq!(s.dialect, Dialect::Graphite);
        assert_eq!(s.tint, "plum".parse().expect("plum"));
        assert_eq!(s.grain, Grain(20));
        assert_eq!(
            s.layers[0].shape,
            Shape::Chevron {
                tip: Pt(11.0, 12.0),
                reach: 4.0,
                heading: Heading::Right,
                width: crate::BAND,
            }
        );
        assert_eq!(s.layers[0].fill, Role::Symbol);
        assert_eq!(s.layers[0].relief, Relief::Raised);
        assert_eq!(s.layers[1].relief, Relief::Recessed);
    }

    #[test]
    fn rejects_unknown_names() {
        assert!(parse_spec(&TEXT.replace("\"plum\"", "\"cyan\"")).is_err());
        assert!(parse_spec(&TEXT.replace("graphite", "neon")).is_err());
        assert!(parse_spec(&TEXT.replace("chevron", "star")).is_err());
        assert!(parse_spec(&TEXT.replace("fill = \"symbol\"", "fill = \"ink\"")).is_err());
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
