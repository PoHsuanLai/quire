//! Which look each workspace wears: the `spaces.json` store (design/21-SPACES.md section 10)
//! and the lookup that falls back to the preset table (section 4).
//!
//! Data and a pure lookup only; reading, writing and watching the file is
//! `ds_settings::spaces` (ds stays effect-free).

use super::look::{CardAccent, Grain, SpaceLook};
use super::presets::default_look;
use serde::{Deserialize, Deserializer, Serialize};
use std::collections::BTreeMap;

/// A workspace's 0-based position on its output.
#[derive(Debug, Clone, Copy, PartialEq, Eq, Hash, PartialOrd, Ord, Serialize, Deserialize)]
#[serde(transparent)]
pub struct WorkspaceIndex(pub usize);

/// The compositor's own id for a workspace (ext-workspace `id`), when it gives one.
#[derive(Debug, Clone, PartialEq, Eq, Hash, PartialOrd, Ord, Serialize, Deserialize)]
#[serde(transparent)]
pub struct WorkspaceId(pub String);

/// One workspace as the store addresses it: always its position, and its id when the
/// compositor names one.
#[derive(Debug, Clone, PartialEq, Eq, Hash)]
pub struct Workspace {
    /// The stable id, if any.
    pub id: Option<WorkspaceId>,
    /// The position on its output.
    pub index: WorkspaceIndex,
}

/// What a workspace with no stored look falls back to beyond its preset's dots: the settings
/// keys `spaces.default_grain` and `spaces.default_card_accent`.
#[derive(Debug, Clone, Copy, PartialEq, Eq, Hash)]
pub struct SpaceDefaults {
    /// The grain for presets that ship none (proposed 40).
    pub grain: Grain,
    /// The card accent (proposed Postmark).
    pub card_accent: CardAccent,
}

impl Default for SpaceDefaults {
    /// design/21 section 4's proposed defaults: grain 40, Postmark.
    fn default() -> Self {
        SpaceDefaults {
            grain: Grain(40),
            card_accent: CardAccent::Postmark,
        }
    }
}

/// `quire/spaces.json`, whole.
///
/// `by_index` holds `None` for a position with no stored look, so a look stored for workspace
/// 3 does not invent looks for 0 to 2. A malformed entry there costs that position only; the
/// lenient reader in ds-settings handles every other key.
#[derive(Debug, Clone, PartialEq, Serialize, Deserialize)]
#[serde(default)]
pub struct SpaceStore {
    /// The schema version, 1.
    pub version: u16,
    /// Looks by the compositor's workspace id.
    pub by_id: BTreeMap<WorkspaceId, SpaceLook>,
    /// Looks by position on the output.
    #[serde(deserialize_with = "each_or_none")]
    pub by_index: Vec<Option<SpaceLook>>,
    /// Keys this build does not know, kept for the next write.
    #[serde(flatten)]
    pub extra: serde_json::Map<String, serde_json::Value>,
}

impl Default for SpaceStore {
    fn default() -> Self {
        SpaceStore {
            version: 1,
            by_id: BTreeMap::new(),
            by_index: Vec::new(),
            extra: serde_json::Map::new(),
        }
    }
}

impl SpaceStore {
    /// The look stored for position `index`, else its preset (`PRESETS[index % 8]`).
    pub fn look_for(&self, index: WorkspaceIndex, defaults: SpaceDefaults) -> SpaceLook {
        self.stored_at(index)
            .cloned()
            .unwrap_or_else(|| preset_look(index, defaults))
    }

    /// The look for `workspace`: by its id, else by its position, else its preset
    /// (design/21 section 10's lookup).
    pub fn look_for_workspace(&self, workspace: &Workspace, defaults: SpaceDefaults) -> SpaceLook {
        workspace
            .id
            .as_ref()
            .and_then(|id| self.by_id.get(id))
            .cloned()
            .unwrap_or_else(|| self.look_for(workspace.index, defaults))
    }

    /// The store with `look` recorded for `workspace`: under its id when it has one, and
    /// always under its position, so a restart that renumbers ids still finds it.
    pub fn with_look(mut self, workspace: &Workspace, look: SpaceLook) -> SpaceStore {
        if let Some(id) = &workspace.id {
            self.by_id.insert(id.clone(), look.clone());
        }
        let at = workspace.index.0;
        if self.by_index.len() <= at {
            self.by_index.resize(at + 1, None);
        }
        self.by_index[at] = Some(look);
        self
    }

    fn stored_at(&self, index: WorkspaceIndex) -> Option<&SpaceLook> {
        self.by_index.get(index.0).and_then(Option::as_ref)
    }
}

/// The preset look for `index` (design/21 section 4).
fn preset_look(index: WorkspaceIndex, defaults: SpaceDefaults) -> SpaceLook {
    default_look(index.0, defaults.grain, defaults.card_accent)
}

/// `by_index`, one entry at a time: an entry that is not a look is `None`, and anything that
/// is not a list is empty.
fn each_or_none<'de, D: Deserializer<'de>>(
    deserializer: D,
) -> Result<Vec<Option<SpaceLook>>, D::Error> {
    let value = serde_json::Value::deserialize(deserializer)?;
    let serde_json::Value::Array(entries) = value else {
        return Ok(Vec::new());
    };
    Ok(entries
        .into_iter()
        .map(|entry| serde_json::from_value::<Option<SpaceLook>>(entry).unwrap_or(None))
        .collect())
}

#[cfg(test)]
mod tests {
    use super::{SpaceDefaults, SpaceStore, Workspace, WorkspaceId, WorkspaceIndex};
    use crate::appearance::Theme;
    use crate::space::look::{CardAccent, Grain, SpaceLook};
    use crate::space::palette::Dot;
    use crate::space::presets::PRESETS;

    fn look(hue: f32) -> SpaceLook {
        SpaceLook {
            dots: vec![Dot { hue, chroma: 0.5 }],
            grain: Grain(70),
            theme: Theme::Dark,
            card_accent: CardAccent::SpaceHue,
        }
    }

    fn workspace(id: Option<&str>, index: usize) -> Workspace {
        Workspace {
            id: id.map(|id| WorkspaceId(id.to_owned())),
            index: WorkspaceIndex(index),
        }
    }

    #[test]
    fn an_empty_store_falls_back_to_the_presets() {
        let store = SpaceStore::default();
        for index in [0, 1, 2, 7, 8, 13] {
            let got = store.look_for(WorkspaceIndex(index), SpaceDefaults::default());
            assert_eq!(got.dots, PRESETS[index % 8].dots, "workspace {index}");
            assert_eq!(got.card_accent, CardAccent::Postmark, "workspace {index}");
        }
    }

    #[test]
    fn lookup_is_by_id_then_index_then_preset() {
        let store = SpaceStore::default()
            .with_look(&workspace(Some("a"), 0), look(10.0))
            .with_look(&workspace(None, 3), look(30.0));
        let defaults = SpaceDefaults::default();
        let cases: &[(&str, Workspace, Option<f32>)] = &[
            ("by id", workspace(Some("a"), 5), Some(10.0)),
            (
                "unknown id, stored index",
                workspace(Some("b"), 3),
                Some(30.0),
            ),
            ("no id, stored index", workspace(None, 0), Some(10.0)),
            ("a gap below a stored index", workspace(None, 1), None),
            ("past the end", workspace(Some("z"), 9), None),
        ];
        for (name, at, want) in cases {
            let got = store.look_for_workspace(at, defaults);
            let want = match want {
                Some(hue) => look(*hue),
                None => store.look_for(at.index, defaults),
            };
            assert_eq!(got, want, "{name}");
        }
        assert_eq!(
            store.by_index.len(),
            4,
            "only up to the highest stored index"
        );
        assert_eq!(store.by_index[1], None);
    }

    #[test]
    fn the_store_round_trips_as_json() {
        let store = SpaceStore::default()
            .with_look(&workspace(Some("ws-1"), 0), look(268.0))
            .with_look(&workspace(None, 2), look(152.0));
        let text = serde_json::to_string(&store).unwrap_or_else(|e| panic!("{e}"));
        let back: SpaceStore =
            serde_json::from_str(&text).unwrap_or_else(|e| panic!("{text}: {e}"));
        assert_eq!(back, store, "{text}");
    }

    #[test]
    fn a_bad_index_entry_costs_only_that_position() {
        let text = r#"{"by_index": [{"grain": 20}, "nonsense", null, {"theme": "dark"}]}"#;
        let store: SpaceStore = serde_json::from_str(text).unwrap_or_else(|e| panic!("{e}"));
        assert_eq!(store.by_index.len(), 4);
        assert_eq!(store.by_index[0].as_ref().map(|l| l.grain), Some(Grain(20)));
        assert_eq!(store.by_index[1], None);
        assert_eq!(store.by_index[2], None);
        assert_eq!(
            store.by_index[3].as_ref().map(|l| l.theme),
            Some(Theme::Dark)
        );
    }
}
