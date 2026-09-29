use super::Word;
use super::testing::word_matches_serde;
use serde::Serialize;

#[derive(Debug, Clone, Copy, PartialEq, Eq, Hash, Serialize, Word)]
#[serde(rename_all = "kebab-case")]
enum Access {
    ReadOnly,
    Full,
    #[word(slug = "off-limits", label = "Nobody")]
    #[serde(rename = "off-limits")]
    Locked,
}

#[derive(Debug, Clone, Copy, PartialEq, Eq, Hash, Serialize, Word)]
#[serde(rename_all = "snake_case")]
#[word(case = snake)]
enum Stored {
    SameAsLight,
    Custom,
}

#[test]
fn all_lists_every_variant_in_declaration_order() {
    assert_eq!(
        Access::ALL,
        &[Access::ReadOnly, Access::Full, Access::Locked]
    );
}

#[test]
fn slug_and_label_come_from_the_variant_or_its_attribute() {
    const CASES: &[(&str, Access, &str, &str)] = &[
        ("two words", Access::ReadOnly, "read-only", "Read only"),
        ("one word", Access::Full, "full", "Full"),
        ("overridden", Access::Locked, "off-limits", "Nobody"),
    ];
    for (name, word, slug, label) in CASES {
        assert_eq!((word.slug(), word.label()), (*slug, *label), "{name}");
    }
}

#[test]
fn parse_finds_a_variant_by_its_slug_and_nothing_else() {
    assert_eq!(Access::parse("read-only"), Some(Access::ReadOnly));
    assert_eq!(Access::parse("off-limits"), Some(Access::Locked));
    assert_eq!(Access::parse("ReadOnly"), None);
    assert_eq!(Access::parse(""), None);
}

#[test]
fn a_stored_enum_matches_its_serde_name() {
    word_matches_serde::<Access>();
    word_matches_serde::<Stored>();
    assert_eq!(Stored::SameAsLight.slug(), "same_as_light");
}
