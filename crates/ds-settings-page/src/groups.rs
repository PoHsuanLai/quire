//! A schema's keys grouped by section.

use ds_settings::schema::{KeySpec, Page, Schema};

/// A section of a schema: its title (empty for keys with none) and its keys.
#[derive(Debug, Clone, PartialEq)]
pub(crate) struct KeyGroup {
    pub title: String,
    pub keys: Vec<KeySpec>,
}

/// The sections of `schema`, in the order their first key comes, each with its keys. With `page`
/// named, only the keys of that page (a key's `page` is the schema's page id); `None` keeps all.
pub(crate) fn sections(schema: &Schema, page: Option<&Page>) -> Vec<KeyGroup> {
    let mut out: Vec<KeyGroup> = Vec::new();
    for key in schema
        .key
        .iter()
        .filter(|key| page.is_none_or(|page| key.page == *page))
    {
        match out.iter_mut().find(|group| group.title == key.section.0) {
            Some(group) => group.keys.push(key.clone()),
            None => out.push(KeyGroup {
                title: key.section.0.clone(),
                keys: vec![key.clone()],
            }),
        }
    }
    out
}

#[cfg(test)]
#[path = "groups_tests.rs"]
mod tests;
