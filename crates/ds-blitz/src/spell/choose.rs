//! Which installed dictionary answers for a language, and the languages the locale names. Pure.

use ds::Lang;

/// Regions a bare language code means when its own `xx_XX` is not installed (`en` is `en_US`,
/// not the alphabetically first `en_AG`).
const USUAL: &[(&str, &str)] = &[("en", "en_US"), ("pt", "pt_BR"), ("zh", "zh_CN")];

/// The installed dictionary for `lang`: the same name; else, for a language or a region with no
/// dictionary of its own, the usual region (`en_US`), the language's own region (`de_DE`), the
/// bare language (`de`), or the first of its regions installed.
pub fn pick(lang: &Lang, installed: &[Lang]) -> Option<Lang> {
    let code = lang.code();
    let usual = USUAL
        .iter()
        .find(|(bare, _)| *bare == code)
        .map(|(_, full)| (*full).to_owned());
    let own = format!("{code}_{}", code.to_uppercase());
    let named = |name: &str| installed.iter().find(|have| have.name() == name).cloned();
    named(lang.name())
        .or_else(|| usual.as_deref().and_then(named))
        .or_else(|| named(&own))
        .or_else(|| named(code))
        .or_else(|| {
            let mut regions: Vec<&Lang> = installed
                .iter()
                .filter(|have| have.code() == code)
                .collect();
            regions.sort();
            regions.first().map(|found| (*found).clone())
        })
}

/// The locale's language: `LC_ALL`, else `LANG` (`en_US.UTF-8` is `en_US`); none for `C` or
/// `POSIX`.
pub fn locale_lang(lc_all: Option<&str>, lang: Option<&str>) -> Option<Lang> {
    [lc_all, lang]
        .into_iter()
        .flatten()
        .find(|value| !value.is_empty())
        .and_then(|value| Lang::parse(value).ok())
}

#[cfg(test)]
mod tests {
    use super::{locale_lang, pick};
    use ds::Lang;

    fn langs(names: &[&str]) -> Vec<Lang> {
        names
            .iter()
            .filter_map(|name| Lang::parse(name).ok())
            .collect()
    }

    #[test]
    fn a_language_finds_its_installed_dictionary() {
        let installed = langs(&[
            "en_AG", "en_GB", "en_US", "de_AT", "de_DE", "fr", "nl_BE", "nl_NL",
        ]);
        let cases: &[(&str, Option<&str>)] = &[
            ("en_GB", Some("en_GB")),
            ("en", Some("en_US")),
            ("en_NZ", Some("en_US")),
            ("de", Some("de_DE")),
            ("de_CH", Some("de_DE")),
            ("fr_FR", Some("fr")),
            ("nl", Some("nl_NL")),
            ("it", None),
        ];
        for (asked, expected) in cases {
            let lang = Lang::parse(asked).expect("a language");
            let found = pick(&lang, &installed);
            assert_eq!(found.as_ref().map(Lang::name), *expected, "{asked}");
        }
        let only = langs(&["pt_PT"]);
        let pt = Lang::parse("pt").expect("a language");
        assert_eq!(pick(&pt, &only).as_ref().map(Lang::name), Some("pt_PT"));
    }

    #[test]
    fn the_locale_names_the_default_language() {
        let cases: &[(Option<&str>, Option<&str>, Option<&str>)] = &[
            (None, Some("en_US.UTF-8"), Some("en_US")),
            (Some("de_DE.UTF-8"), Some("en_US.UTF-8"), Some("de_DE")),
            (Some(""), Some("fr_FR.UTF-8"), Some("fr_FR")),
            (None, Some("C.UTF-8"), None),
            (None, Some("POSIX"), None),
            (None, None, None),
        ];
        for (lc_all, lang, expected) in cases {
            let found = locale_lang(*lc_all, *lang);
            assert_eq!(
                found.as_ref().map(Lang::name),
                *expected,
                "{lc_all:?} {lang:?}"
            );
        }
    }
}
