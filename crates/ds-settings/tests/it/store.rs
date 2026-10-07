//! `Store` over a JSON document: the same lenient read, report and atomic write as TOML.

use crate::support;

use ds_settings::{FileName, Format, Loaded, SettingsDoc};
use serde::{Deserialize, Serialize};
use std::collections::BTreeMap;
use support::Scratch;

#[derive(Debug, Clone, PartialEq, Eq, Default, Serialize, Deserialize)]
#[serde(default)]
struct Ledger {
    owner: String,
    by_index: BTreeMap<String, u8>,
}

impl SettingsDoc for Ledger {
    const FILE: FileName = FileName("ledger.json");
    const FORMAT: Format = Format::Json;
}

#[test]
fn a_json_document_round_trips_through_its_own_file() {
    let scratch = Scratch::new();
    let store = scratch.store();
    let ledger = Ledger {
        owner: "ada".to_owned(),
        by_index: BTreeMap::from([("0".to_owned(), 7), ("1".to_owned(), 9)]),
    };
    store.save(&ledger).unwrap_or_else(|e| panic!("{e}"));
    assert!(scratch.app_dir().join("ledger.json").is_file());
    assert_eq!(store.load::<Ledger>(), Loaded::clean(ledger));
}

#[test]
fn a_json_key_nobody_reads_is_reported_and_a_map_key_is_not() {
    let scratch = Scratch::new();
    std::fs::create_dir_all(scratch.app_dir()).unwrap_or_else(|e| panic!("{e}"));
    std::fs::write(
        scratch.app_dir().join("ledger.json"),
        r#"{"owner": "bob", "stale": true, "by_index": {"3": 4}}"#,
    )
    .unwrap_or_else(|e| panic!("{e}"));
    let loaded = scratch.store().load::<Ledger>();
    assert_eq!(loaded.value.owner, "bob");
    assert_eq!(loaded.value.by_index.get("3"), Some(&4));
    let unknown: Vec<&str> = loaded.unknown.iter().map(|key| key.0.as_str()).collect();
    assert_eq!(unknown, ["stale"]);
}
