use super::sections;
use ds_settings::schema::{
    AgentSetting, AppId, Exposure, FilePath, ForeignTables, Help, KeyKind, KeyPath, KeySpec, Label,
    Page, Schema, Section, WordLabels,
};

fn key(path: &str, section: &str) -> KeySpec {
    KeySpec {
        path: KeyPath(path.to_owned()),
        kind: KeyKind::Text,
        default: toml::Value::String(String::new()),
        label: Label(path.to_owned()),
        help: Help(String::new()),
        page: Page::Appearance,
        section: Section(section.to_owned()),
        exposure: Exposure::Basic,
        labels: WordLabels::default(),
        agent: AgentSetting::HandsOff,
        unavailable: Default::default(),
        groups: Default::default(),
    }
}

#[test]
fn sections_keep_the_order_their_first_key_comes_in() {
    let schema = Schema {
        app: AppId("test".to_owned()),
        file: FilePath("test/test.toml".to_owned()),
        version: 1,
        foreign: ForeignTables::default(),
        key: vec![
            key("a.one", "First"),
            key("b.two", "Second"),
            key("a.three", "First"),
        ],
    };
    let found: Vec<(String, Vec<String>)> = sections(&schema)
        .into_iter()
        .map(|group| {
            (
                group.title,
                group.keys.into_iter().map(|key| key.path.0).collect(),
            )
        })
        .collect();
    assert_eq!(
        found,
        [
            (
                "First".to_owned(),
                vec!["a.one".to_owned(), "a.three".to_owned()]
            ),
            ("Second".to_owned(), vec!["b.two".to_owned()]),
        ]
    );
}
