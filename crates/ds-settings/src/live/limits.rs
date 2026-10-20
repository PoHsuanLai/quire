//! How much a module may send. A module is another program: a reply that carries a million rows or
//! a megabyte label would stall a page that draws one widget per row, so the client refuses what
//! no real module sends.

use super::{LiveError, LiveSchema};
use crate::schema::{KeyKind, KeySpec};

/// The longest schema text (`Describe`, `Snapshot`) read, in bytes. This also bounds the sum of
/// every string inside it.
pub const SCHEMA_JSON_MAX: usize = 1 << 20;
/// The most keys a schema, or a snapshot's values, may hold. An accounts page with a few dozen
/// accounts has a few hundred.
pub const KEYS_MAX: usize = 1024;
/// The longest key path, label, help line, note, section name or control name, in bytes.
pub const TEXT_MAX: usize = 2048;
/// The most choices one menu or segmented key may list.
pub const CHOICES_MAX: usize = 512;
/// The longest string value (`Get`, `Snapshot`, `Changed`), in bytes.
pub const VALUE_TEXT_MAX: usize = 8 * 1024;
/// The most items in a list value.
pub const VALUE_ITEMS_MAX: usize = 512;

/// The refusal for a reply over a limit: plain words, no limit named.
fn too_big() -> LiveError {
    LiveError::Failed("The service sent more than a settings page can show.".to_owned())
}

/// `Ok` when `count` rows fit a page.
pub(super) fn rows(count: usize) -> Result<(), LiveError> {
    if count <= KEYS_MAX {
        Ok(())
    } else {
        Err(too_big())
    }
}

/// The schema `json` holds, checked as [`LiveSchema::from_json`] checks it and kept within the
/// limits.
pub(super) fn schema(json: &str) -> Result<LiveSchema, LiveError> {
    if json.len() > SCHEMA_JSON_MAX {
        return Err(too_big());
    }
    let schema = LiveSchema::from_json(json).map_err(|why| LiveError::Failed(why.to_string()))?;
    rows(schema.key.len())?;
    schema.key.iter().try_for_each(key)?;
    Ok(schema)
}

fn key(spec: &KeySpec) -> Result<(), LiveError> {
    let choices: &[String] = match &spec.kind {
        KeyKind::Menu { variants } | KeyKind::Segmented { variants } => variants,
        _ => &[],
    };
    let texts = [
        spec.path.0.as_str(),
        spec.label.0.as_str(),
        spec.help.0.as_str(),
        spec.section.0.as_str(),
        spec.note.as_ref().map_or("", |note| note.0.as_str()),
        spec.control
            .as_ref()
            .map_or("", |control| control.0.as_str()),
    ];
    let fits = choices.len() <= CHOICES_MAX
        && texts.iter().all(|text| text.len() <= TEXT_MAX)
        && choices.iter().all(|word| word.len() <= TEXT_MAX)
        && value_fits(&spec.default);
    if fits { Ok(()) } else { Err(too_big()) }
}

/// `value` when it fits: a string or list a page can show.
pub(super) fn value(value: toml::Value) -> Result<toml::Value, LiveError> {
    if value_fits(&value) {
        Ok(value)
    } else {
        Err(too_big())
    }
}

fn value_fits(value: &toml::Value) -> bool {
    match value {
        toml::Value::String(text) => text.len() <= VALUE_TEXT_MAX,
        toml::Value::Array(items) => {
            items.len() <= VALUE_ITEMS_MAX
                && items.iter().all(|item| match item {
                    toml::Value::String(text) => text.len() <= VALUE_TEXT_MAX,
                    _ => true,
                })
        }
        _ => true,
    }
}
