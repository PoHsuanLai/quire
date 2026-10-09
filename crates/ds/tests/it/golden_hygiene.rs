//! The specimen suites' hygiene, in one place: every specimen of each lints clean as markup and
//! has every `ds-` class styled. A suite with specimens adds a `hygiene_failures` and a line here.

#[test]
fn every_specimen_lints_clean_and_every_class_is_styled() {
    let failures: Vec<String> = [
        crate::alert_ssr::hygiene_failures(),
        crate::launcher_parts_ssr::hygiene_failures(),
        crate::pdf_thumb_ssr::hygiene_failures(),
        crate::voice_orb_ssr::hygiene_failures(),
    ]
    .concat();
    assert!(failures.is_empty(), "{}", failures.join("\n"));
}
