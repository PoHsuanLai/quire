//! `quire/spaces.json` through a `Store` on a scratch directory: a Space saved with the retired
//! `card_accent = "postmark"` keeps every other key, and so does every other Space.

use crate::support;

use ds_style::appearance::theme::Theme;
use ds_style::space::look::{CardAccent, Grain};
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
        (old.grain, old.theme, old.card_accent),
        (Grain(20), Theme::Dark, CardAccent::Chosen)
    );
    let new = &loaded.value.by_id[&id("new")];
    assert_eq!(
        (new.grain, new.card_accent),
        (Grain(60), CardAccent::SpaceHue)
    );
    let indexed = |at: usize| {
        loaded.value.by_index[at]
            .as_ref()
            .map(|l| (l.grain, l.card_accent))
    };
    assert_eq!(indexed(0), Some((Grain(25), CardAccent::Chosen)));
    assert_eq!(indexed(1), Some((Grain(65), CardAccent::SpaceHue)));
    let invalid: Vec<&str> = loaded.invalid.iter().map(|key| key.path.as_str()).collect();
    assert_eq!(invalid, ["by_id.old.card_accent"]);
}
