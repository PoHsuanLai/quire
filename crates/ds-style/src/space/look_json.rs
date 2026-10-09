//! A Space's look as compact JSON, for a registry that stores one per desktop Space (porter's).
//! The registry bounds what it keeps, so the text is at most [`LOOK_JSON_MOST`] bytes both ways.

use super::list::clamp_look;
use super::look::SpaceLook;
use std::fmt;

/// The most bytes a look's JSON may take.
pub const LOOK_JSON_MOST: usize = 1024;

/// Why a text is not a look.
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
#[non_exhaustive]
pub enum LookJsonError {
    /// The text is longer than [`LOOK_JSON_MOST`] bytes.
    TooLong,
    /// The text is not JSON, or not a look.
    Malformed,
}

impl fmt::Display for LookJsonError {
    fn fmt(&self, out: &mut fmt::Formatter<'_>) -> fmt::Result {
        out.write_str(match self {
            LookJsonError::TooLong => "the look is longer than 1024 bytes",
            LookJsonError::Malformed => "the text is not a Space look",
        })
    }
}

impl std::error::Error for LookJsonError {}

impl SpaceLook {
    /// The look as compact JSON. It is held to the dot limits first (one to three dots), so it
    /// is always well under [`LOOK_JSON_MOST`].
    pub fn to_json(&self) -> String {
        serde_json::to_string(&clamp_look(self.clone())).unwrap_or_default()
    }

    /// The look `text` holds, held to the dot limits. A key it lacks takes the neutral Space's.
    pub fn from_json(text: &str) -> Result<SpaceLook, LookJsonError> {
        if text.len() > LOOK_JSON_MOST {
            return Err(LookJsonError::TooLong);
        }
        serde_json::from_str::<SpaceLook>(text)
            .map(clamp_look)
            .map_err(|_| LookJsonError::Malformed)
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::space::palette::Dot;

    #[test]
    fn a_look_survives_its_json() {
        let look = SpaceLook {
            dots: vec![
                Dot {
                    hue: 200.0,
                    chroma: 0.5,
                },
                Dot {
                    hue: 20.0,
                    chroma: 0.25,
                },
            ],
            grain: crate::space::look::Grain(40),
            ..SpaceLook::default()
        };
        let text = look.to_json();
        assert!(text.len() <= LOOK_JSON_MOST, "{text}");
        assert!(
            !text.contains('\n') && !text.contains("  "),
            "compact: {text}"
        );
        assert_eq!(SpaceLook::from_json(&text), Ok(look));
    }

    #[test]
    fn a_look_with_too_many_dots_is_cut_before_it_is_written() {
        let look = SpaceLook {
            dots: vec![
                Dot {
                    hue: 10.0,
                    chroma: 0.5
                };
                500
            ],
            ..SpaceLook::default()
        };
        let text = look.to_json();
        assert!(text.len() <= LOOK_JSON_MOST, "{} bytes", text.len());
        assert_eq!(
            SpaceLook::from_json(&text).map(|look| look.dots.len()),
            Ok(3)
        );
    }

    #[test]
    fn text_that_is_too_long_or_not_a_look_is_refused() {
        let long = format!("{{\"dots\":[],\"pad\":\"{}\"}}", "x".repeat(LOOK_JSON_MOST));
        assert_eq!(SpaceLook::from_json(&long), Err(LookJsonError::TooLong));
        for text in ["", "[1]", "{\"dots\":", "not json"] {
            assert_eq!(
                SpaceLook::from_json(text),
                Err(LookJsonError::Malformed),
                "{text:?}"
            );
        }
    }

    #[test]
    fn a_missing_key_takes_the_neutral_spaces() {
        assert_eq!(SpaceLook::from_json("{}"), Ok(SpaceLook::default()));
    }
}
