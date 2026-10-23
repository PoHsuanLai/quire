//! One start of a program with styles, as the guard file keeps it, and the file's small text
//! form. The file is a TOML subset the guard writes and reads itself:
//!
//! ```toml
//! [[start]]
//! hash = "0123456789abcdef"
//! state = "starting"
//! ```
//!
//! Reading is lenient: a line it does not know is skipped and a half-written record is dropped,
//! so a damaged file is the same as no history.

use ds::style::kit::Fingerprint;

/// How far a start got.
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum Reached {
    /// Styles were about to load: no first frame plus five seconds yet.
    Starting,
    /// The first frame was drawn and the program lived five seconds more.
    Good,
}

impl Reached {
    fn word(self) -> &'static str {
        match self {
            Reached::Starting => "starting",
            Reached::Good => "good",
        }
    }

    fn of(word: &str) -> Option<Reached> {
        [Reached::Starting, Reached::Good]
            .into_iter()
            .find(|reached| reached.word() == word)
    }
}

/// One start: the hash of the three sheets it loaded, and how far it got.
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub struct Record {
    /// The sheets' fingerprint.
    pub hash: Fingerprint,
    /// How far it got.
    pub reached: Reached,
}

/// The records in `text`, oldest first.
pub fn parse(text: &str) -> Vec<Record> {
    let mut records = Vec::new();
    let mut hash = None;
    let mut reached = None;
    let mut close = |hash: &mut Option<Fingerprint>, reached: &mut Option<Reached>| {
        if let (Some(hash), Some(reached)) = (hash.take(), reached.take()) {
            records.push(Record { hash, reached });
        }
    };
    for line in text.lines().map(str::trim) {
        if line == "[[start]]" {
            close(&mut hash, &mut reached);
        } else if let Some(value) = value_of(line, "hash") {
            hash = u64::from_str_radix(value, 16).ok().map(Fingerprint);
        } else if let Some(value) = value_of(line, "state") {
            reached = Reached::of(value);
        }
    }
    close(&mut hash, &mut reached);
    records
}

/// `value` of a `key = "value"` line.
fn value_of<'a>(line: &'a str, key: &str) -> Option<&'a str> {
    line.strip_prefix(key)?
        .trim_start()
        .strip_prefix('=')?
        .trim()
        .strip_prefix('"')?
        .strip_suffix('"')
}

/// The text form of `records`.
pub fn render(records: &[Record]) -> String {
    records
        .iter()
        .map(|record| {
            format!(
                "[[start]]\nhash = \"{}\"\nstate = \"{}\"\n",
                record.hash.hex(),
                record.reached.word()
            )
        })
        .collect::<Vec<_>>()
        .join("\n")
}
