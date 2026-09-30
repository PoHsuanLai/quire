//! The schema of `quire/appearance.toml`, as quire ships it (design/22-SETTINGS.md section 9.2).

use super::settings::{AppearanceSettings, IconsSettings};
use crate::schema::{AppId, FilePath, Schema, SettingsSchema};

/// `quire.settings.toml`: the keys of `appearance.toml`, the `appearance` and `icons` domains
/// in one schema. quire has no program of its own, so `examples/write_schema.rs` writes it
/// (`cargo run -p ds-settings --example write_schema -- --write-schema <dir>`).
pub fn quire_schema() -> Schema {
    Schema {
        app: AppId("quire".to_owned()),
        file: FilePath("quire/appearance.toml".to_owned()),
        version: 1,
        key: [AppearanceSettings::schema(), IconsSettings::schema()]
            .into_iter()
            .flat_map(|schema| schema.key)
            .collect(),
    }
}

#[cfg(test)]
mod tests {
    use super::quire_schema;
    use crate::schema::Schema;

    #[test]
    fn the_quire_schema_holds_both_domains_and_round_trips() {
        let schema = quire_schema();
        let paths: Vec<&str> = schema.key.iter().map(|key| key.path.0.as_str()).collect();
        assert!(paths.contains(&"appearance.theme"));
        assert!(paths.contains(&"icons.style"));
        assert_eq!(Schema::from_toml(&schema.to_toml()), Ok(schema));
    }
}
