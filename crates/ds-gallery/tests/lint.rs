//! The gallery's own stylesheet passes quire's lint at the strict profile: layout only, every
//! colour, size, radius, shadow and duration a token (ORCHESTRATION coherence rule 1).
//! Every margin, padding and gap is a `--s-*` step (`Rule::RawSpacing`); the three lengths the
//! scale lacked (20, 28, 40) took the step below (18, 26, 36) in the polish pass.

use ds::lint::{LintConfig, Profile, stylesheet};

const GALLERY_CSS: &str = include_str!("../src/gallery.css");

#[test]
fn the_gallery_stylesheet_is_clean_at_the_strict_profile() {
    let offences: Vec<String> = stylesheet(
        GALLERY_CSS,
        &LintConfig {
            profile: Profile::Strict,
            ..LintConfig::default()
        },
    )
    .into_iter()
    .map(|offence| {
        format!(
            "{:?} {}:{}: {}",
            offence.rule, offence.line, offence.column, offence.text
        )
    })
    .collect();
    assert!(offences.is_empty(), "{}", offences.join("\n"));
}

#[test]
fn every_gallery_class_is_the_gallery_s_own() {
    let selectors = GALLERY_CSS
        .split('}')
        .filter_map(|rule| rule.split('{').next())
        .map(|selector| match selector.rfind("*/") {
            Some(end) => &selector[end + 2..],
            None => selector,
        });
    let classes = selectors.flat_map(|selector| {
        selector
            .split(|c: char| !(c.is_ascii_alphanumeric() || c == '-' || c == '.'))
            .flat_map(|word| word.split('.').skip(1))
            .filter(|class| !class.is_empty())
            .collect::<Vec<_>>()
    });
    for class in classes {
        assert!(class.starts_with("g-"), "{class} is not a gallery class");
    }
}
