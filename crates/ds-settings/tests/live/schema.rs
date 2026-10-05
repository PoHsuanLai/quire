use ds_settings::live::{LiveSchema, LiveSchemaError};
use ds_settings::schema::{AgentSetting, KeyKind, KeyPath, Widget, is_never_settable};

use crate::module::{TOGGLE, schema, spec};

#[test]
fn describe_json_round_trips() {
    let schema = schema();
    let json = schema.to_json();
    assert_eq!(LiveSchema::from_json(&json), Ok(schema));
}

#[test]
fn the_live_action_wire_form_is_frozen() {
    let json = schema().to_json();
    assert!(
        json.contains(
            r#""kind":{"kind":"live","v":{"action":{"label":"Remove account","weight":"destructive"}}}"#
        ),
        "{json}"
    );
}

#[test]
fn a_live_schema_with_a_never_settable_key_marked_settable_is_rejected() {
    for path in [
        "accounts.a1.service.mail",
        "accounts.clients.google",
        "ai.local_only",
        "ai.models.default",
        "cua.enabled",
        "agent.budget",
        "memory.on",
        "companion.mood",
        "session.lock_grace_s",
        "idle.dim_after_s",
    ] {
        assert!(is_never_settable(path), "{path}");
        let mut bad = schema();
        let mut key = spec(path, KeyKind::Text, toml::Value::String(String::new()));
        key.agent = AgentSetting::Settable;
        bad.key.push(key);
        assert_eq!(
            bad.check(),
            Err(LiveSchemaError::NeverSettable(KeyPath(path.to_owned()))),
            "{path}"
        );
        assert!(LiveSchema::from_json(&bad.to_json()).is_err(), "{path}");
    }
}

#[test]
fn an_ordinary_key_may_be_agent_settable_in_a_live_schema() {
    let mut ok = schema();
    let mut key = spec(
        "wifi.airplane",
        KeyKind::Text,
        toml::Value::String(String::new()),
    );
    key.agent = AgentSetting::Settable;
    ok.key.push(key);
    assert_eq!(ok.check(), Ok(()));
}

#[test]
fn a_key_twice_is_rejected() {
    let mut twice = schema();
    twice.key.push(twice.key[0].clone());
    assert_eq!(
        twice.check(),
        Err(LiveSchemaError::Duplicate(KeyPath(TOGGLE.to_owned())))
    );
}

#[test]
fn garbage_is_a_parse_error() {
    assert!(matches!(
        LiveSchema::from_json("{"),
        Err(LiveSchemaError::Parse(_))
    ));
}

#[test]
fn widget_for_kind_is_total_over_live() {
    let widgets: Vec<Widget> = schema().key.iter().map(|key| key.kind.widget()).collect();
    assert_eq!(
        widgets,
        [Widget::Toggle, Widget::TextInput, Widget::ActionButton]
    );
}
