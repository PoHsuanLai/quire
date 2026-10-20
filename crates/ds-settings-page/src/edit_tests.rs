use super::{SettingEdit, value_of};
use ds_settings::schema::{
    AgentSetting, Exposure, Help, KeyKind, KeyPath, KeySpec, Label, Page, Section, WordLabels,
};

fn text(word: &str) -> toml::Value {
    toml::Value::String(word.to_owned())
}

fn spelling() -> KeySpec {
    KeySpec {
        path: KeyPath("compose.spelling".to_owned()),
        kind: KeyKind::Toggle {
            variants: ["on".to_owned(), "off".to_owned()],
        },
        default: text("on"),
        label: Label("Spelling".to_owned()),
        help: Help(String::new()),
        page: Page::Appearance,
        section: Section(String::new()),
        exposure: Exposure::Basic,
        labels: WordLabels::default(),
        agent: AgentSetting::HandsOff,
        unavailable: Default::default(),
        groups: Default::default(),
        places: Default::default(),
        note: None,
        control: None,
    }
}

fn edit(word: &str) -> SettingEdit {
    SettingEdit {
        path: KeyPath("compose.spelling".to_owned()),
        value: text(word),
    }
}

#[test]
fn a_value_is_read_and_set_by_its_path() {
    let key = spelling();
    let empty = toml::Value::Table(toml::Table::new());
    assert_eq!(
        value_of(&empty, &key),
        key.default,
        "the default when unset"
    );

    let set = edit("off").applied_to(empty);
    assert_eq!(value_of(&set, &key), text("off"));
    let again = edit("on").applied_to(set);
    assert_eq!(value_of(&again, &key), text("on"));
}

#[test]
fn an_edit_leaves_a_value_that_is_not_a_table_alone() {
    assert_eq!(edit("off").applied_to(text("x")), text("x"));
}
