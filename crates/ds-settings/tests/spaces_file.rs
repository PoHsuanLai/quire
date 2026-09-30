//! `quire/spaces.json` through a `Store` on a scratch directory: a Space saved with the retired
//! `card_accent = "postmark"` keeps every other key, and so does every other Space; a stored
//! `grain` (retired) is reported as unknown.

mod support;

use ds_style::appearance::theme::Theme;
use ds_style::space::look::CardAccent;
use ds_style::space::store::{SpaceStore, WorkspaceId};
use support::Scratch;

const TEXT: &str = r#"{
  "version": 1,
  "by_id": {
    "old": {"grain": 20, "theme": "dark", "card_accent": "postmark"},
    "new": {"grain": 60, "card_accent": "space_hue"}
  },
  "by_index": [
    {"grain": 25, "card_accent": "postmark"},
    {"grain": 65, "card_accent": "space_hue"}
  ]
}"#;

#[test]
fn a_retired_card_accent_costs_only_its_own_key() {
    let scratch = Scratch::new();
    std::fs::create_dir_all(scratch.app_dir()).unwrap_or_else(|e| panic!("{e}"));
    std::fs::write(scratch.app_dir().join("spaces.json"), TEXT).unwrap_or_else(|e| panic!("{e}"));
    let loaded = scratch.store().load::<SpaceStore>();
    let id = |name: &str| WorkspaceId(name.to_owned());
    let old = &loaded.value.by_id[&id("old")];
    assert_eq!(
        (old.theme, old.card_accent),
        (Theme::Dark, CardAccent::Chosen)
    );
    let new = &loaded.value.by_id[&id("new")];
    assert_eq!(new.card_accent, CardAccent::SpaceHue);
    let indexed = |at: usize| loaded.value.by_index[at].as_ref().map(|l| l.card_accent);
    assert_eq!(indexed(0), Some(CardAccent::Chosen));
    assert_eq!(indexed(1), Some(CardAccent::SpaceHue));
    let invalid: Vec<&str> = loaded.invalid.iter().map(|key| key.path.as_str()).collect();
    assert_eq!(invalid, ["by_id.old.card_accent"]);
}

/// A `grain` a Space file still holds is an unknown key, reported, and costs nothing else.
#[test]
fn a_stored_grain_is_reported_as_unknown_and_costs_only_itself() {
    let scratch = Scratch::new();
    std::fs::create_dir_all(scratch.app_dir()).unwrap_or_else(|e| panic!("{e}"));
    std::fs::write(scratch.app_dir().join("spaces.json"), TEXT).unwrap_or_else(|e| panic!("{e}"));
    let loaded = scratch.store().load::<SpaceStore>();
    let unknown: Vec<String> = loaded.unknown.iter().map(|key| key.to_string()).collect();
    assert!(
        unknown.iter().any(|key| key.contains("grain")),
        "grain is not reported: {unknown:?}"
    );
    assert_eq!(loaded.value.by_id.len(), 2, "the store kept both Spaces");
}
