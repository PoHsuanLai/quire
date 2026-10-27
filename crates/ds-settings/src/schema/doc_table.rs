//! The key tables of design/22-SETTINGS.md, generated from the schemas.
//!
//! The doc marks where a domain's table goes with a pair of comment lines,
//! `<!-- keys:begin appearance -->` and `<!-- keys:end appearance -->`. [`regenerate`] rewrites
//! everything between the pair from the keys of the schemas it is given, so a default, a range or
//! a page cannot drift from the code: the doc is checked against this output byte for byte.

use super::key::{KeyKind, KeySpec};
use super::program::Schema;
use super::{AgentSetting, Exposure};

/// Why a doc could not be regenerated.
#[derive(Debug, Clone, PartialEq, Eq, thiserror::Error)]
#[non_exhaustive]
pub enum DocTableError {
    /// A marker names a domain no schema has a key in.
    #[error("`keys:begin {domain}` names a domain no schema has keys for")]
    NoKeys {
        /// The domain after `keys:begin`.
        domain: String,
    },
    /// A `keys:begin` with no `keys:end` after it.
    #[error("`keys:begin {domain}` has no `keys:end {domain}` after it")]
    Unterminated {
        /// The domain after `keys:begin`.
        domain: String,
    },
}

const BEGIN: &str = "<!-- keys:begin ";
const END: &str = "<!-- keys:end ";
const CLOSE: &str = " -->";

/// `doc` with every marked table rewritten from `schemas`. Text outside the markers is returned
/// as it was; a doc with no markers comes back unchanged.
pub fn regenerate(doc: &str, schemas: &[Schema]) -> Result<String, DocTableError> {
    let mut out = String::with_capacity(doc.len());
    let mut open: Option<String> = None;
    for line in doc.split_inclusive('\n') {
        if let Some(domain) = &open {
            if marker(line, END) == Some(domain.as_str()) {
                out.push_str(line);
                open = None;
            }
            continue;
        }
        match marker(line, BEGIN) {
            Some(domain) => {
                out.push_str(line);
                out.push_str(&table_for(domain, schemas)?);
                out.push('\n');
                open = Some(domain.to_owned());
            }
            None => out.push_str(line),
        }
    }
    match open {
        Some(domain) => Err(DocTableError::Unterminated { domain }),
        None => Ok(out),
    }
}

/// The domain a marker line names, if `line` is a marker with `prefix`.
fn marker<'a>(line: &'a str, prefix: &str) -> Option<&'a str> {
    let text = line.trim();
    let domain = text.strip_prefix(prefix)?.strip_suffix(CLOSE)?.trim();
    (!domain.is_empty()).then_some(domain)
}

/// The table of every key under `domain` in `schemas`.
fn table_for(domain: &str, schemas: &[Schema]) -> Result<String, DocTableError> {
    let keys: Vec<&KeySpec> = schemas
        .iter()
        .flat_map(|schema| schema.key.iter())
        .filter(|key| key.path.0.split('.').next() == Some(domain))
        .collect();
    if keys.is_empty() {
        return Err(DocTableError::NoKeys {
            domain: domain.to_owned(),
        });
    }
    Ok(key_table(&keys))
}

/// A Markdown table of `keys`: path, shape, default, range, page, exposure and who may set it.
pub fn key_table(keys: &[&KeySpec]) -> String {
    let mut text = String::from(
        "\n| Key | Type | Default | Range | Page | Shown | Agent |\n| --- | --- | --- | --- | --- | --- | --- |\n",
    );
    for key in keys {
        text.push_str(&row(key));
    }
    text
}

fn row(key: &KeySpec) -> String {
    let (shape, range) = shape(&key.kind);
    let exposure = match key.exposure {
        Exposure::Basic => "Settings",
        Exposure::Advanced => "file only",
    };
    let agent = match key.agent {
        AgentSetting::Settable => "may set",
        _ => "",
    };
    let cells = [
        format!("`{}`", key.path.0),
        shape,
        code(&default_text(&key.default)),
        range,
        key.page.label().to_owned(),
        exposure.to_owned(),
        agent.to_owned(),
    ];
    let cells: Vec<String> = cells.iter().map(|cell| escaped(cell)).collect();
    format!("| {} |\n", cells.join(" | "))
}

/// The type column and the range column of a kind.
fn shape(kind: &KeyKind) -> (String, String) {
    let words = |variants: &[String]| format!("{{{}}}", variants.join(", "));
    match kind {
        KeyKind::Fixed { variant } => (format!("fixed `{variant}`"), String::new()),
        KeyKind::Toggle { variants } => (format!("choice {}", words(&variants[..])), String::new()),
        KeyKind::Segmented { variants } | KeyKind::Menu { variants } => {
            (format!("choice {}", words(variants)), String::new())
        }
        KeyKind::Bounded { min, max, unit } => {
            let unit = unit.as_deref().unwrap_or_default();
            (
                "number".to_owned(),
                format!("`{min}..={max}` {unit}").trim_end().to_owned(),
            )
        }
        KeyKind::Provided { capability } => (format!("provided ({capability})"), String::new()),
        KeyKind::Text => ("text".to_owned(), String::new()),
        KeyKind::Colour => ("colour".to_owned(), String::new()),
        KeyKind::Shortcut => ("shortcut".to_owned(), String::new()),
        KeyKind::List(inner) => (format!("list of {}", shape(inner).0), String::new()),
        KeyKind::Rows { columns } => {
            let names: Vec<&str> = columns
                .iter()
                .map(|column| column.name.0.as_str())
                .collect();
            (format!("rows ({})", names.join(", ")), String::new())
        }
        KeyKind::Live { .. } => ("action".to_owned(), String::new()),
    }
}

/// A default as the file writes it: a word without its quotes, anything else as TOML.
fn default_text(value: &toml::Value) -> String {
    match value {
        toml::Value::String(word) if word.is_empty() => "\"\"".to_owned(),
        toml::Value::String(word) => word.clone(),
        other => other.to_string(),
    }
}

fn code(text: &str) -> String {
    format!("`{text}`")
}

/// `cell` fit for one table cell: no pipe, no line break.
fn escaped(cell: &str) -> String {
    cell.replace('|', "\\|").replace('\n', " ")
}

#[cfg(test)]
mod tests {
    use super::{DocTableError, regenerate};
    use crate::schema::{AppId, FilePath, KeySpec, Schema};

    fn key(path: &str, extra: &str) -> KeySpec {
        let text = format!(
            "path = \"{path}\"\nlabel = \"L\"\nhelp = \"\"\nsection = \"\"\nexposure = \"basic\"\n{extra}[page]\nkind = \"dock\"\n"
        );
        toml::from_str(&text).unwrap_or_else(|e| panic!("{text}: {e}"))
    }

    fn schema() -> Schema {
        Schema {
            footnotes: Vec::new(),
            app: AppId("sill".to_owned()),
            file: FilePath("sill/settings.toml".to_owned()),
            version: 1,
            foreign: Default::default(),
            pages: Vec::new(),
            key: vec![
                key(
                    "dock.tile_size_px",
                    "default = 48\n[kind]\nkind = \"bounded\"\n[kind.v]\nmin = 32\nmax = 80\nunit = \"px\"\n",
                ),
                key(
                    "dock.autohide",
                    "default = \"off\"\n[kind]\nkind = \"toggle\"\nv = { variants = [\"off\", \"on\"] }\n",
                ),
                key("bar.title", "default = \"a|b\"\n[kind]\nkind = \"text\"\n"),
            ],
        }
    }

    const BEFORE: &str =
        "# Doc\n\n<!-- keys:begin dock -->\nstale\n<!-- keys:end dock -->\n\ntail\n";

    #[test]
    fn a_marked_table_is_rewritten_from_the_schema_and_the_rest_is_kept() {
        let got = regenerate(BEFORE, &[schema()]).expect("regenerates");
        let want = "# Doc\n\n<!-- keys:begin dock -->\n\n| Key | Type | Default | Range | Page | Shown | Agent |\n| --- | --- | --- | --- | --- | --- | --- |\n| `dock.tile_size_px` | number | `48` | `32..=80` px | Desktop & Dock | Settings |  |\n| `dock.autohide` | choice {off, on} | `off` |  | Desktop & Dock | Settings |  |\n\n<!-- keys:end dock -->\n\ntail\n";
        assert_eq!(got, want);
        assert_eq!(regenerate(&got, &[schema()]).as_deref(), Ok(got.as_str()));
    }

    #[test]
    fn a_pipe_in_a_default_does_not_break_the_row() {
        let text = "<!-- keys:begin bar -->\n<!-- keys:end bar -->\n";
        let got = regenerate(text, &[schema()]).expect("regenerates");
        assert!(got.contains("| `bar.title` | text | `a\\|b` |"), "{got}");
    }

    #[test]
    fn a_doc_without_markers_is_returned_as_it_was() {
        let text = "no markers here\n| `dock.x` | hand row |\n";
        assert_eq!(regenerate(text, &[schema()]).as_deref(), Ok(text));
    }

    #[test]
    fn a_marker_that_cannot_be_filled_is_an_error() {
        let cases = [
            (
                "<!-- keys:begin nowhere -->\n<!-- keys:end nowhere -->\n",
                DocTableError::NoKeys {
                    domain: "nowhere".to_owned(),
                },
            ),
            (
                "<!-- keys:begin dock -->\nrows\n",
                DocTableError::Unterminated {
                    domain: "dock".to_owned(),
                },
            ),
        ];
        for (text, want) in cases {
            assert_eq!(regenerate(text, &[schema()]), Err(want), "{text:?}");
        }
    }
}
