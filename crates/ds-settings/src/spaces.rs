//! `quire/spaces.json`: which look each workspace wears (design/21-SPACES.md section 10), read,
//! written and watched through the generic [`crate::file`] and [`crate::watch`] API. The store
//! and its lookup are `ds::SpaceStore`.

use crate::file::{FileName, Format, Settings};
use crate::watch::FileWatch;
use ds::SpaceStore;

/// `spaces.json`, holding a [`SpaceStore`]; it lives in quire's own config directory
/// (`config_dir(AppName::QUIRE)`), beside `appearance.toml`.
pub const SPACES: Settings<SpaceStore> = Settings::new(FileName("spaces.json"), Format::Json);

/// A running watch on one directory's `spaces.json`.
pub type SpacesWatch = FileWatch<SpaceStore>;

#[cfg(test)]
mod tests {
    use super::SPACES;
    use crate::appearance_file::{self, FILE_NAME};
    use crate::settings::AppearanceFile;
    use crate::test_dir::TempDir;
    use crate::watch::DEBOUNCE;
    use ds::{
        CardAccent, Dot, Grain, PRESETS, SpaceDefaults, SpaceLook, SpaceStore, Theme, Workspace,
        WorkspaceId, WorkspaceIndex,
    };
    use std::time::Duration;

    fn look(hue: f32, grain: u8) -> SpaceLook {
        SpaceLook {
            dots: vec![Dot { hue, chroma: 0.6 }],
            grain: Grain(grain),
            theme: Theme::Light,
            card_accent: CardAccent::SpaceHue,
        }
    }

    fn workspace(id: Option<&str>, index: usize) -> Workspace {
        Workspace {
            id: id.map(|id| WorkspaceId(id.to_owned())),
            index: WorkspaceIndex(index),
        }
    }

    fn entries(dir: &std::path::Path) -> Vec<String> {
        let mut names: Vec<String> = std::fs::read_dir(dir)
            .unwrap_or_else(|e| panic!("{e}"))
            .map(|entry| {
                entry
                    .unwrap_or_else(|e| panic!("{e}"))
                    .file_name()
                    .to_string_lossy()
                    .into_owned()
            })
            .collect();
        names.sort();
        names
    }

    #[test]
    fn a_store_round_trips_through_the_file() {
        let dir = TempDir::new();
        let cases = [
            SpaceStore::default(),
            SpaceStore::default().with_look(&workspace(Some("ws-1"), 0), look(268.0, 35)),
            SpaceStore::default()
                .with_look(&workspace(None, 3), look(20.0, 90))
                .with_look(&workspace(Some("b"), 1), look(95.0, 0)),
        ];
        for store in cases {
            SPACES
                .save(dir.path(), &store)
                .unwrap_or_else(|e| panic!("{store:?}: {e}"));
            assert_eq!(SPACES.load(dir.path()), store, "{store:?}");
            assert_eq!(entries(dir.path()), ["spaces.json"], "no temp file left");
        }
    }

    #[test]
    fn a_missing_or_damaged_file_falls_back_to_the_presets() {
        let dir = TempDir::new();
        let path = dir.path().join("spaces.json");
        const CASES: &[(&str, Option<&str>)] = &[
            ("missing", None),
            ("empty", Some("")),
            ("not json", Some("{{{ nope")),
            ("json but not an object", Some("[1, 2, 3]")),
        ];
        for &(name, text) in CASES {
            let _ = std::fs::remove_file(&path);
            if let Some(text) = text {
                std::fs::write(&path, text).unwrap_or_else(|e| panic!("{name}: {e}"));
            }
            let store = SPACES.load(dir.path());
            assert_eq!(store, SpaceStore::default(), "{name}");
            let got = store.look_for(WorkspaceIndex(1), SpaceDefaults::default());
            assert_eq!(got.dots, PRESETS[1].dots, "{name}");
            assert_eq!(got.grain, Grain(55), "{name}");
        }
    }

    #[test]
    fn a_bad_value_costs_only_its_own_key() {
        let dir = TempDir::new();
        let text = r#"{
            "version": 1,
            "by_id": {
                "a": { "dots": [{"hue": 190, "chroma": 0.6}], "grain": "loud", "theme": "dark" },
                "b": 7
            },
            "by_index": [null, {"grain": 12}],
            "later": {"kept": true}
        }"#;
        std::fs::write(dir.path().join("spaces.json"), text).unwrap_or_else(|e| panic!("{e}"));
        let store = SPACES.load(dir.path());
        let a = store.by_id.get(&WorkspaceId("a".to_owned()));
        assert_eq!(a.map(|l| l.theme), Some(Theme::Dark), "{store:?}");
        assert_eq!(
            a.map(|l| l.grain),
            Some(Grain::default()),
            "bad grain defaults"
        );
        assert_eq!(a.map(|l| l.dots.len()), Some(1));
        assert!(!store.by_id.contains_key(&WorkspaceId("b".to_owned())));
        assert_eq!(store.by_index.len(), 2);
        assert_eq!(store.by_index[1].as_ref().map(|l| l.grain), Some(Grain(12)));
        // Unknown keys survive the next write.
        SPACES
            .save(dir.path(), &store)
            .unwrap_or_else(|e| panic!("{e}"));
        let written = std::fs::read_to_string(dir.path().join("spaces.json"))
            .unwrap_or_else(|e| panic!("{e}"));
        assert!(written.contains("\"later\""), "{written}");
    }

    const MARGIN: Duration = Duration::from_millis(2_000);

    #[tokio::test]
    async fn the_spaces_watch_sees_its_own_file_and_not_appearance_toml() {
        let dir = TempDir::new();
        let mut spaces = SPACES.watch(dir.path()).unwrap_or_else(|e| panic!("{e}"));
        assert_eq!(spaces.current(), SpaceStore::default());

        // A write to the sibling settings file is not this watch's change.
        let mut appearance = AppearanceFile::default();
        appearance.appearance.theme = Theme::Dark;
        appearance_file::save(dir.path(), &appearance).unwrap_or_else(|e| panic!("{e}"));
        assert!(dir.path().join(FILE_NAME).exists());
        let early = tokio::time::timeout(DEBOUNCE * 4, spaces.changed()).await;
        assert!(
            early.is_err(),
            "appearance.toml fired the spaces watch: {early:?}"
        );

        let store = SpaceStore::default().with_look(&workspace(Some("w"), 0), look(340.0, 40));
        SPACES
            .save(dir.path(), &store)
            .unwrap_or_else(|e| panic!("{e}"));
        let got = tokio::time::timeout(MARGIN, spaces.changed())
            .await
            .unwrap_or_else(|_| panic!("no change observed within the debounce plus margin"))
            .unwrap_or_else(|| panic!("the watch stopped"));
        assert_eq!(got, store);
        let extra = tokio::time::timeout(DEBOUNCE * 4, spaces.changed()).await;
        assert!(extra.is_err(), "expected exactly one change, got {extra:?}");
    }

    #[tokio::test]
    async fn two_watches_in_one_directory_each_see_only_their_file() {
        let dir = TempDir::new();
        let mut spaces = SPACES.watch(dir.path()).unwrap_or_else(|e| panic!("{e}"));
        let mut appearance = crate::watch::watch(dir.path()).unwrap_or_else(|e| panic!("{e}"));

        let store = SpaceStore::default().with_look(&workspace(None, 2), look(55.0, 10));
        SPACES
            .save(dir.path(), &store)
            .unwrap_or_else(|e| panic!("{e}"));
        let got = tokio::time::timeout(MARGIN, spaces.changed())
            .await
            .unwrap_or_else(|_| panic!("spaces.json's write was not seen"))
            .unwrap_or_else(|| panic!("the watch stopped"));
        assert_eq!(got, store);
        let crossed = tokio::time::timeout(DEBOUNCE * 4, appearance.changed()).await;
        assert!(
            crossed.is_err(),
            "spaces.json fired the appearance watch: {crossed:?}"
        );
    }
}
