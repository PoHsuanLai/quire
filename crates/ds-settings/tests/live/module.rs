//! A small accounts-like module the tests serve: the daemon's state is in the module, the library
//! holds none.

use std::collections::BTreeMap;
use std::sync::Mutex;

use ds_settings::live::{Access, Caller, LiveError, LiveModule, LiveSchema, Verdict};
use ds_settings::schema::{
    ActionLabel, ActionWeight, AgentSetting, Exposure, Help, KeyKind, KeyPath, KeySpec, Label,
    LiveAction, Page, Section,
};

pub const TOGGLE: &str = "accounts.a1.service.mail";
pub const CLIENT: &str = "accounts.clients.google";
pub const REMOVE: &str = "accounts.a1.remove";

pub fn spec(path: &str, kind: KeyKind, default: toml::Value) -> KeySpec {
    KeySpec {
        path: KeyPath(path.to_owned()),
        kind,
        default,
        label: Label(path.to_owned()),
        help: Help(String::new()),
        page: Page::Accounts,
        section: Section("Accounts".to_owned()),
        exposure: Exposure::Basic,
        labels: Default::default(),
        agent: AgentSetting::HandsOff,
        unavailable: Default::default(),
        groups: Default::default(),
    }
}

pub fn schema() -> LiveSchema {
    LiveSchema {
        version: 1,
        key: vec![
            spec(
                TOGGLE,
                KeyKind::Toggle {
                    variants: ["on".to_owned(), "off".to_owned()],
                },
                toml::Value::Boolean(true),
            ),
            spec(CLIENT, KeyKind::Text, toml::Value::String(String::new())),
            spec(
                REMOVE,
                KeyKind::Live {
                    action: LiveAction {
                        label: ActionLabel("Remove account".to_owned()),
                        weight: ActionWeight::Destructive,
                    },
                },
                toml::Value::Boolean(false),
            ),
        ],
    }
}

/// Who may write: the caller's connection must be this one's unique name (the "Settings role").
pub struct Accounts {
    pub settings_role: Option<String>,
    pub schema: LiveSchema,
    values: Mutex<BTreeMap<String, toml::Value>>,
    pub removed: Mutex<u32>,
}

impl Accounts {
    pub fn new(settings_role: Option<String>) -> Accounts {
        Accounts::with_schema(settings_role, schema())
    }

    pub fn with_schema(settings_role: Option<String>, schema: LiveSchema) -> Accounts {
        let values = schema
            .key
            .iter()
            .map(|key| (key.path.0.clone(), key.default.clone()))
            .collect();
        Accounts {
            settings_role,
            schema,
            values: Mutex::new(values),
            removed: Mutex::new(0),
        }
    }
}

impl LiveModule for Accounts {
    async fn permit(&self, caller: &Caller, access: Access) -> Verdict {
        match (access, &self.settings_role) {
            (Access::Read, _) => Verdict::Allow,
            (Access::Write, Some(name)) if *name == caller.sender.0 => Verdict::Allow,
            (Access::Write, _) => Verdict::Refuse,
        }
    }

    async fn describe(&self) -> LiveSchema {
        self.schema.clone()
    }

    async fn get(&self, key: &KeyPath) -> Result<toml::Value, LiveError> {
        self.values
            .lock()
            .expect("values")
            .get(&key.0)
            .cloned()
            .ok_or_else(|| LiveError::UnknownKey(key.0.clone()))
    }

    async fn set(&self, key: &KeyPath, value: toml::Value) -> Result<(), LiveError> {
        if key.0 == REMOVE {
            *self.removed.lock().expect("removed") += 1;
            return Ok(());
        }
        let mut values = self.values.lock().expect("values");
        match values.get_mut(&key.0) {
            None => Err(LiveError::UnknownKey(key.0.clone())),
            Some(slot) if std::mem::discriminant(slot) != std::mem::discriminant(&value) => {
                Err(LiveError::BadValue(format!("{} has another type", key.0)))
            }
            Some(slot) => {
                *slot = value;
                Ok(())
            }
        }
    }
}
