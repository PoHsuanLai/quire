use super::{InvalidKey, Loaded, Read, UnknownKey, read};
use crate::doc::{FileName, Format};
use serde::{Deserialize, Serialize};
use std::collections::BTreeMap;

#[derive(Debug, Clone, PartialEq, Eq, Serialize, Deserialize)]
#[serde(default)]
struct Inner {
    depth: u8,
    label: Option<String>,
}

impl Default for Inner {
    fn default() -> Self {
        Inner {
            depth: 3,
            label: None,
        }
    }
}

#[derive(Debug, Clone, PartialEq, Eq, Serialize, Deserialize)]
#[serde(default)]
struct Probe {
    name: String,
    count: u8,
    inner: Inner,
    by_name: BTreeMap<String, u8>,
}

impl Default for Probe {
    fn default() -> Self {
        Probe {
            name: "quire".to_owned(),
            count: 5,
            inner: Inner::default(),
            by_name: BTreeMap::new(),
        }
    }
}

fn loaded(text: &str, format: Format) -> Loaded<Probe> {
    match read::<Probe>(text, format) {
        Read::Loaded(loaded) => loaded,
        Read::Garbled { reason } => panic!("{text:?} is not {format:?}: {reason}"),
    }
}

fn unknown(loaded: &Loaded<Probe>) -> Vec<&str> {
    loaded.unknown.iter().map(|key| key.0.as_str()).collect()
}

fn invalid(loaded: &Loaded<Probe>) -> Vec<&str> {
    loaded.invalid.iter().map(|key| key.path.as_str()).collect()
}

#[test]
fn a_bad_value_costs_only_its_own_key_and_is_reported() {
    struct Case {
        name: &'static str,
        format: Format,
        text: &'static str,
        want: Probe,
        invalid: &'static [&'static str],
    }
    let cases = [
        Case {
            name: "toml: a word where a number belongs",
            format: Format::Toml,
            text: "count = \"many\"\nname = \"kept\"\n",
            want: Probe {
                name: "kept".to_owned(),
                ..Probe::default()
            },
            invalid: &["count"],
        },
        Case {
            name: "toml: out of range for the field",
            format: Format::Toml,
            text: "count = 300\ninner.depth = 9\n",
            want: Probe {
                inner: Inner {
                    depth: 9,
                    label: None,
                },
                ..Probe::default()
            },
            invalid: &["count"],
        },
        Case {
            name: "json: a number where a table belongs",
            format: Format::Json,
            text: r#"{"inner": 3, "count": 9}"#,
            want: Probe {
                count: 9,
                ..Probe::default()
            },
            invalid: &["inner"],
        },
    ];
    for case in cases {
        let got = loaded(case.text, case.format);
        assert_eq!(got.value, case.want, "{}", case.name);
        assert_eq!(invalid(&got), case.invalid, "{}", case.name);
        assert!(got.unknown.is_empty(), "{}: {:?}", case.name, got.unknown);
    }
}

#[test]
fn a_key_the_type_does_not_read_is_reported_at_its_first_unknown_table() {
    struct Case {
        name: &'static str,
        format: Format,
        text: &'static str,
        unknown: &'static [&'static str],
    }
    const CASES: &[Case] = &[
        Case {
            name: "toml: a top-level key",
            format: Format::Toml,
            text: "future = 1\ncount = 6\n",
            unknown: &["future"],
        },
        Case {
            name: "toml: a key inside a known table",
            format: Format::Toml,
            text: "[inner]\ndepth = 4\npuppy = true\n",
            unknown: &["inner.puppy"],
        },
        Case {
            name: "toml: a whole unknown table is one entry",
            format: Format::Toml,
            text: "[later]\nanswer = 42\nquestion = \"?\"\n",
            unknown: &["later"],
        },
        Case {
            name: "json: an unknown key",
            format: Format::Json,
            text: r#"{"extra": {"a": 1}, "count": 2}"#,
            unknown: &["extra"],
        },
    ];
    for case in CASES {
        let got = loaded(case.text, case.format);
        assert_eq!(unknown(&got), case.unknown, "{}", case.name);
        assert!(got.invalid.is_empty(), "{}: {:?}", case.name, got.invalid);
    }
}

#[test]
fn a_known_key_is_never_reported_unknown() {
    let got = loaded(
        "count = 5\n[inner]\nlabel = \"set\"\n[by_name]\nada = 1\nbob = 2\n",
        Format::Toml,
    );
    assert!(got.unknown.is_empty(), "{:?}", got.unknown);
    assert_eq!(
        got.value.inner.label.as_deref(),
        Some("set"),
        "an absent-by-default key"
    );
    assert_eq!(got.value.by_name.len(), 2, "keys of a map field");
}

#[test]
fn text_that_is_not_the_format_is_garbled_not_defaulted() {
    const CASES: &[(&str, Format)] = &[
        ("not toml [[[ = =", Format::Toml),
        ("\u{0}\u{1}garbage", Format::Toml),
        ("{ not json", Format::Json),
        ("[1, 2]", Format::Json),
    ];
    for &(text, format) in CASES {
        assert!(
            matches!(read::<Probe>(text, format), Read::Garbled { .. }),
            "{text:?}"
        );
    }
}

#[test]
fn an_empty_file_is_the_defaults_with_nothing_to_report() {
    for format in [Format::Toml, Format::Json] {
        let text = if format == Format::Toml { "" } else { "{}" };
        assert_eq!(
            loaded(text, format),
            Loaded::clean(Probe::default()),
            "{format:?}"
        );
    }
}

#[test]
fn diagnostics_name_the_file_and_the_key() {
    let got = Loaded {
        value: Probe::default(),
        unknown: vec![UnknownKey("dock.puppy".to_owned())],
        invalid: vec![InvalidKey {
            path: "count".to_owned(),
            reason: "invalid type".to_owned(),
        }],
    };
    let lines = got.diagnostics(FileName("settings.toml"));
    assert_eq!(lines.len(), 2, "{lines:?}");
    assert!(
        lines[0].starts_with("settings.toml: unknown key `dock.puppy`"),
        "{lines:?}"
    );
    assert!(
        lines[1].starts_with("settings.toml: `count` is not valid"),
        "{lines:?}"
    );
}
