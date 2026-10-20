//! `SettingsWriter::patch`: one key set in place, everything else in the file as it was.

use crate::support::{Scratch, names};
use ds_settings::schema::KeyPath;
use ds_settings::{Edit, FileName, Format, SettingsDoc, SettingsError, SettingsWriter};
use serde::{Deserialize, Serialize};

#[derive(Debug, Clone, PartialEq, Eq, Default, Serialize, Deserialize)]
#[serde(default)]
struct Probe {
    size: u16,
}

impl SettingsDoc for Probe {
    const FILE: FileName = FileName("probe.toml");
    const FORMAT: Format = Format::Toml;
}

fn key(path: &str) -> KeyPath {
    KeyPath(path.to_owned())
}

fn file(scratch: &Scratch) -> std::path::PathBuf {
    scratch.app_dir().join("probe.toml")
}

fn written(scratch: &Scratch, text: &str) -> SettingsWriter {
    std::fs::create_dir_all(scratch.app_dir()).unwrap_or_else(|e| panic!("{e}"));
    std::fs::write(file(scratch), text).unwrap_or_else(|e| panic!("{e}"));
    SettingsWriter::new(file(scratch))
}

fn read(scratch: &Scratch) -> String {
    std::fs::read_to_string(file(scratch)).unwrap_or_else(|e| panic!("{e}"))
}

const WITH_COMMENTS: &str = "# my settings\nversion = 1\n\n[dock]\ntile_size_px   =  48 # pixels\n# hand-added\nnote = \"mine\"\n\n[future]\nthing = true\n";

#[test]
fn a_patch_changes_one_value_and_leaves_every_other_byte() {
    struct Case {
        name: &'static str,
        key: &'static str,
        value: toml::Value,
        expect: &'static str,
    }
    let cases = [
        Case {
            name: "an existing key keeps its spacing and comment",
            key: "dock.tile_size_px",
            value: toml::Value::Integer(64),
            expect: "# my settings\nversion = 1\n\n[dock]\ntile_size_px   =  64 # pixels\n# hand-added\nnote = \"mine\"\n\n[future]\nthing = true\n",
        },
        Case {
            name: "a key of an unknown table is just a key",
            key: "future.thing",
            value: toml::Value::Boolean(false),
            expect: "# my settings\nversion = 1\n\n[dock]\ntile_size_px   =  48 # pixels\n# hand-added\nnote = \"mine\"\n\n[future]\nthing = false\n",
        },
    ];
    for case in cases {
        let scratch = Scratch::new();
        let writer = written(&scratch, WITH_COMMENTS);
        writer
            .patch(&key(case.key), &case.value)
            .unwrap_or_else(|e| panic!("{}: {e}", case.name));
        assert_eq!(read(&scratch), case.expect, "{}", case.name);
    }
}

#[test]
fn a_new_key_joins_its_table_and_every_old_line_stays() {
    let scratch = Scratch::new();
    let writer = written(&scratch, WITH_COMMENTS);
    writer
        .patch(&key("dock.autohide"), &toml::Value::String("on".to_owned()))
        .unwrap_or_else(|e| panic!("{e}"));
    let text = read(&scratch);
    for line in WITH_COMMENTS.lines() {
        assert!(text.contains(line), "lost {line:?} in {text:?}");
    }
    let table: toml::Table = text.parse().unwrap_or_else(|e| panic!("{text}: {e}"));
    assert_eq!(table["dock"]["autohide"].as_str(), Some("on"));
}

#[test]
fn a_patch_makes_the_file_and_the_tables_on_the_way() {
    let scratch = Scratch::new();
    let writer = SettingsWriter::of::<Probe>(&scratch.store()).unwrap_or_else(|e| panic!("{e}"));
    writer
        .patch(
            &key("wallpaper.picture"),
            &toml::Value::String("/p/a.png".to_owned()),
        )
        .unwrap_or_else(|e| panic!("{e}"));
    assert_eq!(read(&scratch), "[wallpaper]\npicture = \"/p/a.png\"\n");
    assert!(names(&scratch.app_dir()).contains(&"probe.toml".to_owned()));
}

#[test]
fn a_patch_loads_back_through_the_store() {
    let scratch = Scratch::new();
    let writer = written(&scratch, "size = 3\n");
    writer
        .patch(&key("size"), &toml::Value::Integer(9))
        .unwrap_or_else(|e| panic!("{e}"));
    assert_eq!(scratch.store().load::<Probe>().value, Probe { size: 9 });
}

#[test]
fn a_file_or_key_that_cannot_take_the_edit_is_left_untouched() {
    struct Case {
        name: &'static str,
        text: &'static str,
        key: &'static str,
    }
    let cases = [
        Case {
            name: "not TOML",
            text: "size = = 3\n",
            key: "size",
        },
        Case {
            name: "a key inside a value that is not a table",
            text: "size = 3\n",
            key: "size.inner",
        },
        Case {
            name: "an empty segment",
            text: "size = 3\n",
            key: "size..x",
        },
    ];
    for case in cases {
        let scratch = Scratch::new();
        let writer = written(&scratch, case.text);
        let error = writer
            .patch(&key(case.key), &toml::Value::Integer(1))
            .expect_err(case.name);
        assert!(
            matches!(error, SettingsError::Patch { .. }),
            "{}: {error}",
            case.name
        );
        assert_eq!(read(&scratch), case.text, "{}", case.name);
    }
}

#[test]
fn a_table_value_is_written_inline_and_loads_back() {
    #[derive(Debug, Clone, PartialEq, Eq, Default, Serialize, Deserialize)]
    #[serde(default)]
    struct Consent {
        given: u64,
    }
    #[derive(Debug, Clone, PartialEq, Eq, Default, Serialize, Deserialize)]
    #[serde(default)]
    struct Voice {
        consent: Consent,
    }
    #[derive(Debug, Clone, PartialEq, Eq, Default, Serialize, Deserialize)]
    #[serde(default)]
    struct Shell {
        voice: Voice,
    }
    impl SettingsDoc for Shell {
        const FILE: FileName = FileName("probe.toml");
        const FORMAT: Format = Format::Toml;
    }
    let scratch = Scratch::new();
    let writer = written(&scratch, "# mine\n[voice]\nother = 1\n");
    let mut table = toml::Table::new();
    table.insert("given".to_owned(), toml::Value::Integer(1_760_000_000));
    writer
        .patch(&key("voice.consent"), &toml::Value::Table(table))
        .unwrap_or_else(|e| panic!("{e}"));
    let text = read(&scratch);
    assert!(text.contains("consent = { given = 1760000000 }"), "{text}");
    assert!(
        text.contains("# mine") && text.contains("other = 1"),
        "{text}"
    );
    let loaded = scratch.store().load::<Shell>();
    assert_eq!(loaded.value.voice.consent.given, 1_760_000_000);
}

#[test]
fn patch_all_applies_every_edit_in_one_write_or_none() {
    let scratch = Scratch::new();
    let writer = written(&scratch, "a = 1\nb = 2\n");
    writer
        .patch_all(&[
            (key("a"), toml::Value::Integer(10)),
            (key("c.d"), toml::Value::String("x".to_owned())),
        ])
        .unwrap_or_else(|e| panic!("{e}"));
    let table: toml::Table = read(&scratch).parse().unwrap_or_else(|e| panic!("{e}"));
    assert_eq!(table["a"].as_integer(), Some(10));
    assert_eq!(table["b"].as_integer(), Some(2));
    assert_eq!(table["c"]["d"].as_str(), Some("x"));

    let before = read(&scratch);
    let error = writer
        .patch_all(&[
            (key("a"), toml::Value::Integer(99)),
            (key("b.inner"), toml::Value::Integer(1)),
        ])
        .expect_err("b is not a table");
    assert!(matches!(error, SettingsError::Patch { .. }), "{error}");
    assert_eq!(read(&scratch), before);
}

#[test]
fn unset_takes_one_key_out_and_leaves_every_other_byte() {
    let scratch = Scratch::new();
    let writer = written(&scratch, WITH_COMMENTS);
    writer.unset(&key("dock.tile_size_px")).expect("unset");
    assert_eq!(
        read(&scratch),
        "# my settings\nversion = 1\n\n[dock]\n# hand-added\nnote = \"mine\"\n\n[future]\nthing = true\n"
    );
}

#[test]
fn unsetting_a_key_that_is_not_there_leaves_the_file_as_it_was() {
    let scratch = Scratch::new();
    let writer = written(&scratch, WITH_COMMENTS);
    writer.unset(&key("dock.missing")).expect("a missing key");
    writer
        .unset(&key("nowhere.at.all"))
        .expect("a missing table");
    assert_eq!(read(&scratch), WITH_COMMENTS);
}

#[test]
fn unset_refuses_a_path_through_a_value_and_set_and_unset_go_in_one_write() {
    let scratch = Scratch::new();
    let writer = written(&scratch, WITH_COMMENTS);
    let refused = writer.unset(&key("version.inner"));
    assert!(
        matches!(refused, Err(SettingsError::Patch { .. })),
        "{refused:?}"
    );
    writer
        .apply(&[
            Edit::Set(key("dock.tile_size_px"), toml::Value::Integer(64)),
            Edit::Unset(key("future.thing")),
        ])
        .expect("both");
    let text = read(&scratch);
    assert!(text.contains("tile_size_px   =  64 # pixels"), "{text}");
    assert!(!text.contains("thing"), "{text}");
}
