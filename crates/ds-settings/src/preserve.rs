//! Saving a TOML settings file into the one already there. A key the type does not read (a newer
//! rev's, a hand-added one) and every comment stay where they were; only what the type owns is
//! written.
//!
//! The typed document, written as TOML, is laid over the parsed file key by key: a value that
//! means the same is left as typed, a changed one takes the old one's spacing and trailing
//! comment, a new one is added, and a key the type owns but no longer writes (an `Option` that is
//! now `None`, a map entry that was removed) is taken out. What the type owns is everything the
//! file sets except the keys [`Loaded::unknown`](crate::Loaded::unknown) reported when the file
//! was read.

use crate::damaged::InTheWay;
use crate::lenient::UnknownKey;
use std::collections::HashSet;
use toml_edit::{DocumentMut, Item, Table, Value};

/// `fresh` (the typed document as TOML text) laid over `existing`, keeping the keys in `unknown`
/// and all formatting. `fresh` text that does not parse (the encoder writes only valid TOML) is
/// returned as it is.
pub(crate) fn merge(
    existing: &str,
    fresh: &str,
    unknown: &[UnknownKey],
) -> Result<String, InTheWay> {
    let mut document = existing.parse::<DocumentMut>().map_err(|_| InTheWay)?;
    let Ok(typed) = fresh.parse::<DocumentMut>() else {
        return Ok(fresh.to_owned());
    };
    let unknown: HashSet<&str> = unknown.iter().map(|key| key.0.as_str()).collect();
    lay(document.as_table_mut(), typed.as_table(), &unknown, "");
    Ok(document.to_string())
}

/// The dotted key path of `key` inside the table at `path`.
fn dotted(path: &str, key: &str) -> String {
    if path.is_empty() {
        key.to_owned()
    } else {
        format!("{path}.{key}")
    }
}

/// `fresh`'s keys laid over `table`, whose own path is `path`.
fn lay(table: &mut Table, fresh: &Table, unknown: &HashSet<&str>, path: &str) {
    drop_owned_but_absent(table, fresh, unknown, path);
    for (key, item) in fresh.iter() {
        match table.get_mut(key) {
            Some(old) => {
                let opens = old.is_inline_table() && item.is_table();
                settle(old, item, unknown, &dotted(path, key));
                // `dock = { .. }` keeps the space before its `=`; a `[dock]` header must not.
                if opens && let Some(mut header) = table.key_mut(key) {
                    header.leaf_decor_mut().clear();
                }
            }
            None => {
                table.insert(key, unplaced(item.clone()));
            }
        }
    }
}

/// Take out of `table` every key that `fresh` does not write and the file's reader did not report
/// as unknown, descending into tables so an unknown key deeper down stays.
fn drop_owned_but_absent(table: &mut Table, fresh: &Table, unknown: &HashSet<&str>, path: &str) {
    let absent: Vec<String> = table
        .iter()
        .map(|(key, _)| key)
        .filter(|key| !fresh.contains_key(key))
        .map(str::to_owned)
        .collect();
    for key in absent {
        let here = dotted(path, &key);
        if unknown.contains(here.as_str()) {
            continue;
        }
        let emptied = match table.get_mut(&key).and_then(Item::as_table_mut) {
            Some(inner) => {
                drop_owned_but_absent(inner, &Table::new(), unknown, &here);
                inner.is_empty()
            }
            None => true,
        };
        if emptied {
            table.remove(&key);
        }
    }
}

/// Make the file's `old` item say what `new` says, changing as little of the text as it can.
fn settle(old: &mut Item, new: &Item, unknown: &HashSet<&str>, path: &str) {
    if let Some(fresh) = new.as_table() {
        open_inline(old);
        if let Some(table) = old.as_table_mut() {
            lay(table, fresh, unknown, path);
            return;
        }
    }
    if means_the_same(old, new) {
        return;
    }
    let mut item = unplaced(new.clone());
    if let (Some(was), Some(is)) = (old.as_value(), item.as_value_mut()) {
        *is.decor_mut() = was.decor().clone();
    }
    *old = item;
}

/// An inline table (`dock = { size = 3 }`) turned into a table, so keys inside it can be merged.
fn open_inline(item: &mut Item) {
    if item.as_inline_table().is_some()
        && let Ok(table) = item.clone().into_table()
    {
        *item = Item::Table(table);
    }
}

/// Whether two items hold the same value, whatever their spacing and comments.
fn means_the_same(old: &Item, new: &Item) -> bool {
    match (old.as_value(), new.as_value()) {
        (Some(old), Some(new)) => match (meaning(old), meaning(new)) {
            (Some(old), Some(new)) => old == new,
            _ => false,
        },
        (None, None) => old.to_string() == new.to_string(),
        _ => false,
    }
}

/// `value` without its decor, read back as a value; `None` when that fails.
fn meaning(value: &Value) -> Option<toml::Value> {
    let mut bare = value.clone();
    bare.decor_mut().clear();
    bare.to_string().parse().ok()
}

/// `item` with its tables' positions cleared, so a table that is new to the file is written after
/// the one before it instead of at the position it had in the typed document.
fn unplaced(mut item: Item) -> Item {
    match &mut item {
        Item::Table(table) => clear_positions(table),
        Item::ArrayOfTables(tables) => tables.iter_mut().for_each(clear_positions),
        _ => {}
    }
    item
}

fn clear_positions(table: &mut Table) {
    table.set_position(None);
    table
        .iter_mut()
        .for_each(|(_, child)| *child = unplaced(std::mem::take(child)));
}

#[cfg(test)]
mod tests;
