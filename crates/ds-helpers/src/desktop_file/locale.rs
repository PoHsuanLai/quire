//! A locale as a localized key names it: `lang_COUNTRY@MODIFIER`, and the keys it matches.

/// A person's locale, as `de_DE.UTF-8@euro` or `sr@latin`: language, then optionally a country
/// and a modifier. The encoding is not part of a key's locale and is dropped.
#[derive(Debug, Clone, PartialEq, Eq, Hash)]
pub struct EntryLocale {
    lang: String,
    country: Option<String>,
    modifier: Option<String>,
}

impl EntryLocale {
    /// The locale `text` names; none for an empty text, `C` and `POSIX`, which no translation
    /// answers.
    pub fn parse(text: &str) -> Option<EntryLocale> {
        let (rest, modifier) = match text.split_once('@') {
            Some((rest, modifier)) => (rest, Some(modifier)),
            None => (text, None),
        };
        let rest = rest.split('.').next().unwrap_or_default();
        let (lang, country) = match rest.split_once('_') {
            Some((lang, country)) => (lang, Some(country)),
            None => (rest, None),
        };
        let present = |part: Option<&str>| part.filter(|p| !p.is_empty()).map(str::to_owned);
        (!matches!(lang, "" | "C" | "POSIX")).then(|| EntryLocale {
            lang: lang.to_owned(),
            country: present(country),
            modifier: present(modifier),
        })
    }

    /// The locales a key may carry for this one, best first: `lang_COUNTRY@MODIFIER`,
    /// `lang_COUNTRY`, `lang@MODIFIER`, `lang`, each only when its parts are there.
    pub fn variants(&self) -> Vec<String> {
        let lang = &self.lang;
        let both = self
            .country
            .iter()
            .zip(&self.modifier)
            .map(|(country, modifier)| format!("{lang}_{country}@{modifier}"));
        let country = self.country.iter().map(|c| format!("{lang}_{c}"));
        let modifier = self.modifier.iter().map(|m| format!("{lang}@{m}"));
        both.chain(country)
            .chain(modifier)
            .chain(std::iter::once(lang.clone()))
            .collect()
    }
}

#[cfg(test)]
mod tests {
    use super::EntryLocale;

    #[test]
    fn a_locale_text_gives_the_keys_it_matches() {
        // `(text, variants)`; none when the text names no locale.
        const CASES: &[(&str, Option<&[&str]>)] = &[
            ("de", Some(&["de"])),
            ("de_DE", Some(&["de_DE", "de"])),
            ("de_DE.UTF-8", Some(&["de_DE", "de"])),
            (
                "de_DE.UTF-8@euro",
                Some(&["de_DE@euro", "de_DE", "de@euro", "de"]),
            ),
            ("sr@latin", Some(&["sr@latin", "sr"])),
            ("zh_TW", Some(&["zh_TW", "zh"])),
            ("en_US.", Some(&["en_US", "en"])),
            ("de_", Some(&["de"])),
            ("", None),
            ("C", None),
            ("POSIX", None),
            ("C.UTF-8", None),
            ("_DE", None),
        ];
        for (text, want) in CASES {
            let got = EntryLocale::parse(text).map(|locale| locale.variants());
            let want: Option<Vec<String>> =
                want.map(|keys| keys.iter().map(|key| (*key).to_owned()).collect());
            assert_eq!(got, want, "{text:?}");
        }
    }
}
