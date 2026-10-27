//! The desktop-wide region preferences of `appearance.toml`: today the 12 or 24 hour clock
//! (`[region] hours = "auto"`), one choice for the bar, the calendar, mail's snooze times and
//! every other clock a program draws.

use crate::schema::Page;
use ds_core::word::Word;
use serde::{Deserialize, Serialize};

/// How the person wants hours written (`region.hours`).
#[derive(Clone, Copy, Debug, PartialEq, Eq, Hash, Default, Serialize, Deserialize, Word)]
#[serde(rename_all = "snake_case")]
#[word(case = snake)]
#[non_exhaustive]
pub enum HourCycle {
    /// Follow the language and region of the session.
    #[default]
    Auto,
    /// 1 to 12 with AM or PM.
    #[serde(rename = "12")]
    #[word(slug = "12", label = "12-hour")]
    H12,
    /// 0 to 23.
    #[serde(rename = "24")]
    #[word(slug = "24", label = "24-hour")]
    H24,
}

/// The clock a program draws: [`HourCycle`] with `Auto` settled.
#[derive(Clone, Copy, Debug, PartialEq, Eq, Hash)]
pub enum ResolvedHours {
    /// 1 to 12 with AM or PM.
    H12,
    /// 0 to 23.
    H24,
}

/// Countries whose people write 12-hour clocks by default (the glibc and CLDR conventions).
const TWELVE_HOUR_COUNTRIES: &[&str] = &[
    "US", "CA", "AU", "NZ", "IN", "PH", "TW", "HK", "KR", "MX", "CO", "SA", "EG", "PK", "BD",
];

impl HourCycle {
    /// The clock to draw in `locale`, a POSIX locale name (`en_US.UTF-8`, `de_DE@euro`, `C`).
    /// `H12` and `H24` are the person's choice and ignore the locale; `Auto` reads it: the
    /// countries in a short table and a bare `en` use 12 hours, `fr_CA` and every other locale,
    /// `C` included, use 24.
    pub fn resolve(self, locale: &str) -> ResolvedHours {
        match self {
            HourCycle::H12 => ResolvedHours::H12,
            HourCycle::H24 => ResolvedHours::H24,
            HourCycle::Auto => locale_hours(locale),
        }
    }
}

/// The clock a locale name writes.
fn locale_hours(locale: &str) -> ResolvedHours {
    let name = locale.split(['.', '@']).next().unwrap_or_default();
    let mut parts = name.split(['_', '-']);
    let language = parts.next().unwrap_or_default();
    let country = parts.next().unwrap_or_default();
    let twelve = match country {
        "" => language == "en",
        country => TWELVE_HOUR_COUNTRIES.contains(&country) && language != "fr",
    };
    if twelve {
        ResolvedHours::H12
    } else {
        ResolvedHours::H24
    }
}

/// The locale name the session uses for times, by POSIX precedence: `LC_ALL`, then `LC_TIME`,
/// then `LANG`, each ignored when empty; `C` when none is set. The caller reads the environment
/// and hands the three values in.
pub fn time_locale<'a>(
    lc_all: Option<&'a str>,
    lc_time: Option<&'a str>,
    lang: Option<&'a str>,
) -> &'a str {
    [lc_all, lc_time, lang]
        .into_iter()
        .flatten()
        .find(|name| !name.is_empty())
        .unwrap_or("C")
}

/// `region.*`: the language and region preferences every program shares.
#[derive(Clone, Debug, PartialEq, Default, Serialize, Deserialize, crate::SettingsSchema)]
#[serde(default)]
#[settings(file = "quire/appearance.toml", domain = "region", page = Page::Appearance)]
#[non_exhaustive]
pub struct RegionSettings {
    /// `region.hours`: Auto, 12 or 24.
    #[settings(
        label = "Time format",
        help = "Writes times in 12 or 24 hours everywhere.",
        note = "Automatic follows your language and region.",
        section = "Text and time"
    )]
    pub hours: HourCycle,
}

#[cfg(test)]
mod tests {
    use super::{HourCycle, ResolvedHours, time_locale};

    #[test]
    fn auto_follows_the_locale_and_a_choice_ignores_it() {
        const CASES: &[(&str, ResolvedHours)] = &[
            ("en_US", ResolvedHours::H12),
            ("en_US.UTF-8", ResolvedHours::H12),
            ("en_GB", ResolvedHours::H24),
            ("de_DE", ResolvedHours::H24),
            ("de_DE@euro", ResolvedHours::H24),
            ("zh_TW", ResolvedHours::H12),
            ("zh_TW.UTF-8", ResolvedHours::H12),
            ("ja_JP", ResolvedHours::H24),
            ("C", ResolvedHours::H24),
            ("C.UTF-8", ResolvedHours::H24),
            ("POSIX", ResolvedHours::H24),
            ("fr_CA", ResolvedHours::H24),
            ("en_CA", ResolvedHours::H12),
            ("en-AU", ResolvedHours::H12),
            ("en", ResolvedHours::H12),
            ("", ResolvedHours::H24),
        ];
        for (locale, want) in CASES {
            assert_eq!(HourCycle::Auto.resolve(locale), *want, "{locale:?}");
            assert_eq!(HourCycle::H12.resolve(locale), ResolvedHours::H12);
            assert_eq!(HourCycle::H24.resolve(locale), ResolvedHours::H24);
        }
    }

    /// `LC_ALL`, `LC_TIME` and `LANG` as set, and the locale the time is shown in.
    type PosixCase = (
        Option<&'static str>,
        Option<&'static str>,
        Option<&'static str>,
        &'static str,
    );

    #[test]
    fn the_time_locale_follows_posix_precedence() {
        const CASES: &[PosixCase] = &[
            (Some("de_DE"), Some("en_US"), Some("ja_JP"), "de_DE"),
            (None, Some("en_US"), Some("ja_JP"), "en_US"),
            (Some(""), None, Some("ja_JP"), "ja_JP"),
            (None, None, None, "C"),
            (Some(""), Some(""), Some(""), "C"),
        ];
        for (all, time, lang, want) in CASES {
            assert_eq!(time_locale(*all, *time, *lang), *want);
        }
    }

    #[test]
    fn the_words_are_the_ones_the_file_holds() {
        let table: toml::Table = "hours = \"24\"\n".parse().expect("parses");
        let hours: HourCycle = table["hours"].clone().try_into().expect("a word");
        assert_eq!(hours, HourCycle::H24);
        for (cycle, word) in [
            (HourCycle::Auto, "auto"),
            (HourCycle::H12, "12"),
            (HourCycle::H24, "24"),
        ] {
            assert_eq!(
                toml::Value::try_from(cycle).ok(),
                Some(toml::Value::String(word.to_owned()))
            );
        }
    }
}
