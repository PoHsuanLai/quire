use super::wire::{TomlWire, Wire};
use super::{
    ClampedKey, InvalidKey, LimitBounds, Limits, Loaded, Read, UnknownKey, read, read_within,
};
use crate::doc::{FileName, Format};
use crate::schema::KeySpec;
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
        clamped: vec![ClampedKey {
            path: "inner.depth".to_owned(),
            was: "99".to_owned(),
            now: "8".to_owned(),
        }],
    };
    let lines = got.diagnostics(FileName("settings.toml"));
    assert_eq!(lines.len(), 3, "{lines:?}");
    assert!(
        lines[0].starts_with("settings.toml: unknown key `dock.puppy`"),
        "{lines:?}"
    );
    assert!(
        lines[1].starts_with("settings.toml: `count` is not valid"),
        "{lines:?}"
    );
    assert_eq!(
        lines[2],
        "settings.toml: `inner.depth` is 99, outside its range; 8 is used"
    );
}

fn limits(ranges: &[(&str, i64, i64)]) -> Limits {
    let keys: Vec<KeySpec> = ranges
        .iter()
        .map(|(path, min, max)| {
            let text = format!(
                "path = \"{path}\"\ndefault = 0\nlabel = \"L\"\nhelp = \"\"\nsection = \"\"\nexposure = \"basic\"\n[kind]\nkind = \"bounded\"\n[kind.v]\nmin = {min}\nmax = {max}\n[page]\nkind = \"appearance\"\n"
            );
            toml::from_str(&text).unwrap_or_else(|e| panic!("{text}: {e}"))
        })
        .collect();
    Limits::of_keys(&keys)
}

#[test]
fn a_number_outside_its_range_is_held_to_the_nearest_end_and_reported_once() {
    struct Case {
        name: &'static str,
        text: &'static str,
        format: Format,
        ranges: &'static [(&'static str, i64, i64)],
        count: u8,
        depth: u8,
        clamped: &'static [(&'static str, &'static str, &'static str)],
    }
    const CASES: &[Case] = &[
        Case {
            name: "above the range",
            text: "count = 200",
            format: Format::Toml,
            ranges: &[("count", 0, 9)],
            count: 9,
            depth: 3,
            clamped: &[("count", "200", "9")],
        },
        Case {
            name: "below the range",
            text: "count = 0",
            format: Format::Toml,
            ranges: &[("count", 1, 9)],
            count: 1,
            depth: 3,
            clamped: &[("count", "0", "1")],
        },
        Case {
            name: "on an end of the range",
            text: "count = 9",
            format: Format::Toml,
            ranges: &[("count", 1, 9)],
            count: 9,
            depth: 3,
            clamped: &[],
        },
        Case {
            name: "inside the range",
            text: "count = 5",
            format: Format::Toml,
            ranges: &[("count", 1, 9)],
            count: 5,
            depth: 3,
            clamped: &[],
        },
        Case {
            name: "a key in a table",
            text: "[inner]\ndepth = 99",
            format: Format::Toml,
            ranges: &[("inner.depth", 1, 8)],
            count: 5,
            depth: 8,
            clamped: &[("inner.depth", "99", "8")],
        },
        Case {
            name: "two keys at once",
            text: "count = 200\n[inner]\ndepth = 0",
            format: Format::Toml,
            ranges: &[("count", 0, 9), ("inner.depth", 1, 8)],
            count: 9,
            depth: 1,
            clamped: &[("count", "200", "9"), ("inner.depth", "0", "1")],
        },
        Case {
            name: "a key with no range",
            text: "count = 200",
            format: Format::Toml,
            ranges: &[("inner.depth", 1, 8)],
            count: 200,
            depth: 3,
            clamped: &[],
        },
        Case {
            name: "JSON",
            text: r#"{"count": 300}"#,
            format: Format::Json,
            ranges: &[("count", 0, 9)],
            count: 9,
            depth: 3,
            clamped: &[("count", "300", "9")],
        },
    ];
    for case in CASES {
        let got = match read_within::<Probe>(case.text, case.format, &limits(case.ranges)) {
            Read::Loaded(loaded) => loaded,
            Read::Garbled { reason } => panic!("{}: {reason}", case.name),
        };
        assert_eq!(got.value.count, case.count, "{}", case.name);
        assert_eq!(got.value.inner.depth, case.depth, "{}", case.name);
        let clamped: Vec<(&str, &str, &str)> = got
            .clamped
            .iter()
            .map(|key| (key.path.as_str(), key.was.as_str(), key.now.as_str()))
            .collect();
        assert_eq!(clamped, case.clamped, "{}", case.name);
        assert!(got.invalid.is_empty(), "{}: {:?}", case.name, got.invalid);
    }
}

#[test]
fn a_value_that_is_not_a_number_is_the_types_to_refuse_not_the_range() {
    let got = match read_within::<Probe>(
        "count = \"many\"",
        Format::Toml,
        &limits(&[("count", 0, 9)]),
    ) {
        Read::Loaded(loaded) => loaded,
        Read::Garbled { reason } => panic!("{reason}"),
    };
    assert_eq!(got.value.count, 5);
    assert_eq!(got.invalid.len(), 1);
    assert!(got.clamped.is_empty());
}

#[test]
fn a_float_is_held_to_the_range_too_and_a_nan_is_left_alone() {
    let bounds = LimitBounds { min: 0, max: 100 };
    let cases = [
        (toml::Value::Float(5000.0), Some(toml::Value::Float(100.0))),
        (toml::Value::Float(-1.5), Some(toml::Value::Float(0.0))),
        (toml::Value::Float(40.5), None),
        (toml::Value::Float(f64::NAN), None),
        (toml::Value::Boolean(true), None),
    ];
    for (given, want) in cases {
        assert_eq!(TomlWire::clamp(&given, bounds), want, "{given:?}");
    }
}

#[test]
fn a_range_written_backwards_is_no_range() {
    assert_eq!(limits(&[("count", 9, 0)]).bounds("count"), None);
}
