//! Resolving a launcher search query to one settings key, from the schemas alone
//! (design/22-SETTINGS.md section 9.3 item 3: "the query 'dark mode' resolves to
//! `appearance.theme`... no hand-kept table").

use super::key::{KeyPath, KeySpec};
use super::program::Schema;

/// The one key across every schema in `schemas` whose label or help text mentions `query`
/// (case-insensitive), or [`None`] if no key matches or more than one does — a query that
/// resolves to two keys is not a deep link, it is a search result list, which is a different
/// feature.
pub fn deep_link<'a>(schemas: &'a [Schema], query: &str) -> Option<&'a KeySpec> {
    let query = query.to_lowercase();
    let mut matches = schemas.iter().flat_map(|schema| &schema.key).filter(|key| {
        key.label.0.to_lowercase().contains(&query) || key.help.0.to_lowercase().contains(&query)
    });
    let first = matches.next()?;
    match matches.next() {
        None => Some(first),
        Some(_) => None,
    }
}

/// The path a query resolves to, or [`None`]; a thin convenience over [`deep_link`] for a
/// caller that only wants to compare against a known [`KeyPath`].
pub fn deep_link_path(schemas: &[Schema], query: &str) -> Option<KeyPath> {
    deep_link(schemas, query).map(|key| key.path.clone())
}

#[cfg(test)]
mod tests {
    use super::{deep_link, deep_link_path};
    use crate::schema::{
        AppId, Deprecated, Exposure, FilePath, Help, KeyKind, KeyPath, KeySpec, Label, Page,
        Schema, Section,
    };

    fn key(path: &str, label: &str, help: &str) -> KeySpec {
        KeySpec {
            path: KeyPath(path.to_owned()),
            kind: KeyKind::Text,
            default: toml::Value::String(String::new()),
            label: Label(label.to_owned()),
            help: Help(help.to_owned()),
            page: Page::Appearance,
            section: Section(String::new()),
            exposure: Exposure::Basic,
            deprecated: Deprecated::No,
        }
    }

    fn schema(keys: Vec<KeySpec>) -> Schema {
        Schema {
            app: AppId("quire".to_owned()),
            file: FilePath("quire/appearance.toml".to_owned()),
            version: 1,
            key: keys,
        }
    }

    #[test]
    fn a_unique_label_match_resolves() {
        let schemas = vec![schema(vec![
            key(
                "appearance.theme",
                "Theme",
                "Follow the desktop, or force light or dark mode.",
            ),
            key("appearance.look", "Look", "The card's visual language."),
        ])];
        assert_eq!(
            deep_link_path(&schemas, "dark mode"),
            Some(KeyPath("appearance.theme".to_owned()))
        );
    }

    #[test]
    fn an_ambiguous_query_resolves_to_nothing() {
        let schemas = vec![schema(vec![
            key("appearance.theme", "Theme", "light or dark"),
            key("appearance.look", "Look", "light or dark styling"),
        ])];
        assert_eq!(deep_link(&schemas, "light"), None);
    }

    #[test]
    fn a_query_matching_nothing_resolves_to_nothing() {
        let schemas = vec![schema(vec![key(
            "appearance.theme",
            "Theme",
            "light or dark",
        )])];
        assert_eq!(deep_link(&schemas, "bluetooth"), None);
    }

    #[test]
    fn matching_is_case_insensitive() {
        let schemas = vec![schema(vec![key("appearance.theme", "Theme", "Dark Mode")])];
        assert_eq!(
            deep_link_path(&schemas, "dark mode"),
            Some(KeyPath("appearance.theme".to_owned()))
        );
    }
}
