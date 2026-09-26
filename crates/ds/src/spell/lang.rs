//! Which dictionary a surface checks against, and whether it checks at all.

use crate::error::DsError;
use serde::{Deserialize, Serialize};

/// A dictionary's language, as Hunspell names its files: `en_US` for `en_US.aff`/`en_US.dic`,
/// `de` for `de.aff`. A language code of two or three lower-case letters, then optionally `_`
/// and a two-letter upper-case region.
#[derive(Debug, Clone, PartialEq, Eq, Hash, PartialOrd, Ord, Serialize, Deserialize)]
pub struct Lang(String);

impl Lang {
    /// A language from a dictionary name or a locale: `en_US`, `en-US`, `en_US.UTF-8`,
    /// `de_DE@euro` and `pt` all parse; `C`, `POSIX` and anything else without a language code
    /// are [`DsError::Language`].
    pub fn parse(value: &str) -> Result<Lang, DsError> {
        let refused = || DsError::Language {
            value: value.to_owned(),
        };
        let bare = value
            .split(['.', '@'])
            .next()
            .unwrap_or_default()
            .replace('-', "_");
        let mut parts = bare.split('_');
        let code = parts.next().unwrap_or_default();
        let region = parts.next();
        if parts.next().is_some() || !language_code(code) {
            return Err(refused());
        }
        match region {
            None => Ok(Lang(code.to_owned())),
            Some(region) if region_code(region) => Ok(Lang(format!("{code}_{region}"))),
            Some(_) => Err(refused()),
        }
    }

    /// The name the dictionary's files carry: `en_US`.
    pub fn name(&self) -> &str {
        &self.0
    }

    /// The language without its region: `en` for `en_US`.
    pub fn code(&self) -> &str {
        self.0.split('_').next().unwrap_or(&self.0)
    }
}

fn language_code(code: &str) -> bool {
    (2..=3).contains(&code.len()) && code.chars().all(|c| c.is_ascii_lowercase())
}

fn region_code(region: &str) -> bool {
    region.len() == 2 && region.chars().all(|c| c.is_ascii_uppercase())
}

/// Whether an [`EditSurface`](crate::EditSurface) checks its spelling. Off by default: a surface
/// that says nothing checks nothing and draws exactly what it drew before.
#[derive(Debug, Clone, PartialEq, Eq, Hash, Default, Serialize, Deserialize)]
#[serde(tag = "kind", content = "v", rename_all = "snake_case")]
pub enum Spell {
    /// No checking, no marks, no suggestions.
    #[default]
    Off,
    /// Check, in `lang`, or in the host's default languages (the locale's) when `None`.
    On {
        /// This surface's own language, overriding the host's list.
        lang: Option<Lang>,
    },
}

#[cfg(test)]
mod tests {
    use super::Lang;

    const CASES: &[(&str, Option<&str>)] = &[
        ("en_US", Some("en_US")),
        ("en-US", Some("en_US")),
        ("en_US.UTF-8", Some("en_US")),
        ("de_DE@euro", Some("de_DE")),
        ("pt", Some("pt")),
        ("fil_PH", Some("fil_PH")),
        ("zh_TW.UTF-8", Some("zh_TW")),
        ("C", None),
        ("C.UTF-8", None),
        ("POSIX", None),
        ("", None),
        ("en_us", None),
        ("english", None),
        ("en_US_x", None),
    ];

    #[test]
    fn a_locale_or_dictionary_name_parses_to_its_language() {
        for (input, expected) in CASES {
            let parsed = Lang::parse(input).ok();
            assert_eq!(parsed.as_ref().map(Lang::name), *expected, "{input:?}");
        }
    }

    #[test]
    fn the_code_drops_the_region() {
        assert_eq!(
            Lang::parse("en_GB").map(|l| l.code().to_owned()),
            Ok("en".to_owned())
        );
        assert_eq!(
            Lang::parse("de").map(|l| l.code().to_owned()),
            Ok("de".to_owned())
        );
    }
}
