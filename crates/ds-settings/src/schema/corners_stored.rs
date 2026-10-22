//! How `hot_corners` is read: the new list of bindings, and the older files that wrote one word
//! per corner (`top_left = "lock"`, with `top_left_command` for `"command"`). The older forms are
//! read and never written back.

use ds_core::word::Word;
use serde::Deserialize;

use super::behaviour::{CornerAction, HotCornerSettings};
use super::corners::{CornerBinding, CornerBindings, CornerDo};
use crate::units::{Ms, Px};

/// One corner's value in a file.
#[derive(Deserialize)]
#[serde(untagged)]
pub(super) enum StoredCorner {
    /// The older form: `"none"`, `"command"` or a desktop action's word.
    Word(String),
    /// The list of bindings. Each is read on its own, so one bad binding costs only itself.
    List(Vec<toml::Value>),
}

impl StoredCorner {
    /// The bindings this value stands for; `command` is the corner's older `*_command` key.
    pub(super) fn into_bindings(self, command: &str) -> Result<CornerBindings, String> {
        match self {
            StoredCorner::List(values) => {
                Ok(CornerBindings::one_per_keys(values.into_iter().filter_map(
                    |value| value.try_into::<CornerBinding>().ok(),
                )))
            }
            StoredCorner::Word(word) => match word.as_str() {
                "none" => Ok(CornerBindings::default()),
                "command" if command.is_empty() => Ok(CornerBindings::default()),
                "command" => Ok(CornerBindings::single(CornerDo::Command(
                    command.to_owned(),
                ))),
                other => CornerAction::parse(other)
                    .map(|action| CornerBindings::single(CornerDo::Desktop(action)))
                    .ok_or_else(|| format!("unknown corner action `{other}`")),
            },
        }
    }
}

impl<'de> Deserialize<'de> for CornerBindings {
    fn deserialize<D: serde::Deserializer<'de>>(deserializer: D) -> Result<Self, D::Error> {
        StoredCorner::deserialize(deserializer)?
            .into_bindings("")
            .map_err(serde::de::Error::custom)
    }
}

/// `hot_corners` as a file holds it, either form.
#[derive(Deserialize)]
pub(super) struct StoredHotCorners {
    top_left: Option<StoredCorner>,
    top_right: Option<StoredCorner>,
    bottom_left: Option<StoredCorner>,
    bottom_right: Option<StoredCorner>,
    #[serde(default)]
    top_left_command: String,
    #[serde(default)]
    top_right_command: String,
    #[serde(default)]
    bottom_left_command: String,
    #[serde(default)]
    bottom_right_command: String,
    dwell_ms: Option<Ms>,
    rearm_ms: Option<Ms>,
    size_px: Option<Px>,
}

impl TryFrom<StoredHotCorners> for HotCornerSettings {
    type Error = String;

    fn try_from(stored: StoredHotCorners) -> Result<Self, String> {
        let corner = |value: Option<StoredCorner>, command: &str| match value {
            Some(value) => value.into_bindings(command),
            None => Ok(CornerBindings::default()),
        };
        let kept = HotCornerSettings::default();
        Ok(HotCornerSettings {
            top_left: corner(stored.top_left, &stored.top_left_command)?,
            top_right: corner(stored.top_right, &stored.top_right_command)?,
            bottom_left: corner(stored.bottom_left, &stored.bottom_left_command)?,
            bottom_right: corner(stored.bottom_right, &stored.bottom_right_command)?,
            dwell_ms: stored.dwell_ms.unwrap_or(kept.dwell_ms),
            rearm_ms: stored.rearm_ms.unwrap_or(kept.rearm_ms),
            size_px: stored.size_px.unwrap_or(kept.size_px),
        })
    }
}

/// The corners of a file's `hot_corners` table that still hold the older one word take their
/// `*_command` key in: `top_left = "command"` with `top_left_command = "x"` becomes the list
/// `[{ action = "command", command = "x" }]` (an empty `x`, an empty list), and the `*_command`
/// key of any word corner goes. The lenient reader then sees only keys the type writes back, so
/// an older command is kept and its key is not reported as unknown.
///
/// A program that edits the settings file as TOML text (Settings) runs the same fold before it
/// reads the corners.
pub fn fold_older_commands(file: &mut toml::Table) {
    let Some(toml::Value::Table(corners)) = file.get_mut("hot_corners") else {
        return;
    };
    for corner in ["top_left", "top_right", "bottom_left", "bottom_right"] {
        let Some(toml::Value::String(word)) = corners.get(corner).cloned() else {
            continue;
        };
        let command = corners.remove(&format!("{corner}_command"));
        if word != "command" {
            continue;
        }
        let bindings = match command {
            Some(toml::Value::String(command)) if !command.is_empty() => {
                let mut binding = toml::Table::new();
                binding.insert("action".to_owned(), toml::Value::from("command"));
                binding.insert("command".to_owned(), toml::Value::from(command));
                vec![toml::Value::Table(binding)]
            }
            _ => Vec::new(),
        };
        corners.insert(corner.to_owned(), toml::Value::Array(bindings));
    }
}
