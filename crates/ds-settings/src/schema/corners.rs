//! What a hot corner does, as the settings file keeps it (`hot_corners.<corner>`, design/22
//! section 3.23): a short list of bindings, each naming the modifier keys held when it fires and
//! what it does. On disk a binding is one table whose `action` word says which kind it is.

use ds_core::word::Word;
use serde::{Deserialize, Deserializer, Serialize, Serializer};

use super::behaviour::CornerAction;

/// One modifier key a binding can want held.
#[derive(Debug, Clone, Copy, PartialEq, Eq, Hash, Serialize, Deserialize, Word)]
#[serde(rename_all = "snake_case")]
#[word(case = snake)]
pub enum CornerKey {
    /// The Command key.
    Command,
    /// The Option (Alt) key.
    Option,
    /// The Control key.
    Control,
    /// The Shift key.
    Shift,
}

/// A set of [`CornerKey`]s, written as a list of words in the order command, option, control,
/// shift. Empty means no modifier is held.
#[derive(Debug, Clone, Copy, PartialEq, Eq, Hash, Default)]
pub struct CornerKeys(u8);

impl CornerKeys {
    /// This set plus `key`.
    #[must_use]
    pub fn with(self, key: CornerKey) -> CornerKeys {
        CornerKeys(self.0 | Self::bit(key))
    }

    /// Whether `key` is in the set.
    pub fn contains(self, key: CornerKey) -> bool {
        self.0 & Self::bit(key) != 0
    }

    /// Whether no key is in the set.
    pub fn is_empty(&self) -> bool {
        self.0 == 0
    }

    /// The keys in the set, in the written order.
    pub fn iter(self) -> impl Iterator<Item = CornerKey> {
        CornerKey::ALL
            .iter()
            .copied()
            .filter(move |key| self.contains(*key))
    }

    fn bit(key: CornerKey) -> u8 {
        1 << (key as u8)
    }
}

impl FromIterator<CornerKey> for CornerKeys {
    fn from_iter<I: IntoIterator<Item = CornerKey>>(keys: I) -> Self {
        keys.into_iter()
            .fold(CornerKeys::default(), CornerKeys::with)
    }
}

impl Serialize for CornerKeys {
    fn serialize<S: Serializer>(&self, serializer: S) -> Result<S::Ok, S::Error> {
        serializer.collect_seq(self.iter())
    }
}

impl<'de> Deserialize<'de> for CornerKeys {
    fn deserialize<D: Deserializer<'de>>(deserializer: D) -> Result<Self, D::Error> {
        Vec::<CornerKey>::deserialize(deserializer).map(|keys| keys.into_iter().collect())
    }
}

/// What an `open` binding opens.
#[derive(Debug, Clone, PartialEq, Eq)]
pub enum OpenItem {
    /// An app, by desktop id.
    App(String),
    /// A file or folder; `~` is allowed.
    Path(String),
    /// A web address.
    Url(String),
}

/// What a binding does.
#[derive(Debug, Clone, PartialEq, Eq)]
#[non_exhaustive]
pub enum CornerDo {
    /// One of the desktop's own actions.
    Desktop(CornerAction),
    /// An action an app declares in its intents manifest, with no parameters to fill.
    AppAction {
        /// The app's id.
        app: String,
        /// The action's name in the manifest.
        intent: String,
    },
    /// Open an app, a file or folder, or a web address.
    Open(OpenItem),
    /// A command, run by `/bin/sh -c`.
    Command(String),
}

/// What a corner does when the pointer rests in it with exactly `keys` held.
#[derive(Debug, Clone, PartialEq, Eq, Serialize, Deserialize)]
#[serde(try_from = "BindingTable", into = "BindingTable")]
pub struct CornerBinding {
    /// The modifier keys that must be held, no more and no fewer.
    pub keys: CornerKeys,
    /// What happens.
    pub action: CornerDo,
}

/// A corner's bindings; at most one per set of keys. Empty: the corner is inactive.
#[derive(Debug, Clone, PartialEq, Eq, Default, Serialize)]
#[serde(transparent)]
pub struct CornerBindings(pub Vec<CornerBinding>);

impl CornerBindings {
    /// A corner with one binding and no keys held.
    pub fn single(action: CornerDo) -> CornerBindings {
        CornerBindings(vec![CornerBinding {
            keys: CornerKeys::default(),
            action,
        }])
    }

    /// `bindings` with a later binding for an already used set of keys left out.
    pub(super) fn one_per_keys(bindings: impl IntoIterator<Item = CornerBinding>) -> Self {
        let mut taken: Vec<CornerKeys> = Vec::new();
        let kept = bindings
            .into_iter()
            .filter(|binding| {
                let fresh = !taken.contains(&binding.keys);
                taken.push(binding.keys);
                fresh
            })
            .collect();
        CornerBindings(kept)
    }
}

/// One binding as a TOML table: the `action` word beside the fields that kind needs.
#[derive(Serialize, Deserialize)]
struct BindingTable {
    #[serde(default, skip_serializing_if = "CornerKeys::is_empty")]
    keys: CornerKeys,
    action: String,
    #[serde(default, skip_serializing_if = "Option::is_none")]
    app: Option<String>,
    #[serde(default, skip_serializing_if = "Option::is_none")]
    intent: Option<String>,
    #[serde(default, skip_serializing_if = "Option::is_none")]
    path: Option<String>,
    #[serde(default, skip_serializing_if = "Option::is_none")]
    url: Option<String>,
    #[serde(default, skip_serializing_if = "Option::is_none")]
    command: Option<String>,
}

impl BindingTable {
    fn of(keys: CornerKeys, action: &str) -> BindingTable {
        BindingTable {
            keys,
            action: action.to_owned(),
            app: None,
            intent: None,
            path: None,
            url: None,
            command: None,
        }
    }
}

impl From<CornerBinding> for BindingTable {
    fn from(binding: CornerBinding) -> BindingTable {
        let CornerBinding { keys, action } = binding;
        match action {
            CornerDo::Desktop(word) => BindingTable::of(keys, word.slug()),
            CornerDo::AppAction { app, intent } => BindingTable {
                app: Some(app),
                intent: Some(intent),
                ..BindingTable::of(keys, "app_action")
            },
            CornerDo::Open(OpenItem::App(app)) => BindingTable {
                app: Some(app),
                ..BindingTable::of(keys, "open")
            },
            CornerDo::Open(OpenItem::Path(path)) => BindingTable {
                path: Some(path),
                ..BindingTable::of(keys, "open")
            },
            CornerDo::Open(OpenItem::Url(url)) => BindingTable {
                url: Some(url),
                ..BindingTable::of(keys, "open")
            },
            CornerDo::Command(command) => BindingTable {
                command: Some(command),
                ..BindingTable::of(keys, "command")
            },
        }
    }
}

impl TryFrom<BindingTable> for CornerBinding {
    type Error = String;

    fn try_from(table: BindingTable) -> Result<Self, String> {
        let BindingTable {
            keys,
            action,
            app,
            intent,
            path,
            url,
            command,
        } = table;
        let action = match action.as_str() {
            "app_action" => match (app, intent) {
                (Some(app), Some(intent)) => CornerDo::AppAction { app, intent },
                _ => return Err("an app action needs `app` and `intent`".to_owned()),
            },
            "open" => match (app, path, url) {
                (Some(app), _, _) => CornerDo::Open(OpenItem::App(app)),
                (None, Some(path), _) => CornerDo::Open(OpenItem::Path(path)),
                (None, None, Some(url)) => CornerDo::Open(OpenItem::Url(url)),
                (None, None, None) => {
                    return Err("open needs an `app`, a `path` or a `url`".to_owned());
                }
            },
            "command" => match command {
                Some(command) if !command.is_empty() => CornerDo::Command(command),
                _ => return Err("a command binding needs a `command`".to_owned()),
            },
            word => match CornerAction::parse(word) {
                Some(action) => CornerDo::Desktop(action),
                None => return Err(format!("unknown corner action `{word}`")),
            },
        };
        Ok(CornerBinding { keys, action })
    }
}
