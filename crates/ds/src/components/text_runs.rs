//! Text: a line a caller hands a component either whole or as runs in several tones, so a
//! search hit can mark the words it matched and a palette row can set a name in a stronger
//! tone than its path (mailo gaps 2). The caller computes the runs; quire only draws them, as
//! `<mark class="ds-mark">` for a hit and `span.ds-run[data-tone]` for a weight, never as markup
//! from strings.
//!
//! A `String` or `&str` converts into [`Text::Plain`], so a prop that took a `String` takes a
//! `Text` with no change at the call site; an optional one takes the string itself, `None`, or a
//! `Text` (`Some(string)` must be written `Some(string.into())`: `None` could not be inferred if
//! both `Option<String>` and `Option<Text>` were accepted).

use dioxus::core::SuperFrom;
use dioxus::prelude::*;

/// How one run is set.
#[derive(Debug, Clone, Copy, PartialEq, Eq, Hash, Default)]
pub enum RunTone {
    /// The line's own tone.
    #[default]
    Plain,
    /// A search hit: `<mark class="ds-mark">`, a soft accent ground.
    Mark,
    /// Stronger than the line: `data-tone="strong"`.
    Strong,
    /// Quieter than the line: `data-tone="faint"`.
    Faint,
}

impl RunTone {
    /// The `data-tone` word of a span run; `None` for the tones drawn another way.
    fn slug(self) -> Option<&'static str> {
        match self {
            RunTone::Strong => Some("strong"),
            RunTone::Faint => Some("faint"),
            RunTone::Plain | RunTone::Mark => None,
        }
    }
}

/// A piece of a line in one tone.
#[derive(Debug, Clone, PartialEq, Eq, Hash)]
pub struct Run {
    /// The characters.
    pub text: String,
    /// How they are set.
    pub tone: RunTone,
}

impl Run {
    /// `text` in `tone`.
    pub fn new(text: impl Into<String>, tone: RunTone) -> Self {
        Run {
            text: text.into(),
            tone,
        }
    }
}

/// A line of text: whole, or as runs the caller computed.
#[derive(Debug, Clone, PartialEq, Eq, Hash)]
pub enum Text {
    /// One run in the line's own tone: what a `String` becomes.
    Plain(String),
    /// Runs in order, each in its own tone.
    Runs(Vec<Run>),
}

impl Text {
    /// The characters with the tones dropped: what a filter matches and a label reads.
    pub fn plain_text(&self) -> String {
        match self {
            Text::Plain(text) => text.clone(),
            Text::Runs(runs) => runs.iter().map(|run| run.text.as_str()).collect(),
        }
    }
}

impl Default for Text {
    fn default() -> Self {
        Text::Plain(String::new())
    }
}

impl From<String> for Text {
    fn from(text: String) -> Self {
        Text::Plain(text)
    }
}

impl From<&String> for Text {
    fn from(text: &String) -> Self {
        Text::Plain(text.clone())
    }
}

impl From<&str> for Text {
    fn from(text: &str) -> Self {
        Text::Plain(text.to_owned())
    }
}

impl From<Vec<Run>> for Text {
    fn from(runs: Vec<Run>) -> Self {
        Text::Runs(runs)
    }
}

/// Marks the conversions below, so they do not collide with dioxus's own.
#[doc(hidden)]
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub struct OptionalText;

/// An optional `Text` prop takes a `String` as `Some`, as an optional `String` prop does.
impl SuperFrom<String, OptionalText> for Option<Text> {
    fn super_from(text: String) -> Self {
        Some(Text::Plain(text))
    }
}

/// An optional `Text` prop takes a `&str` as `Some`.
impl<'a> SuperFrom<&'a str, OptionalText> for Option<Text> {
    fn super_from(text: &'a str) -> Self {
        Some(Text::from(text))
    }
}

/// `text` drawn: a bare text node when plain, else each run in its tone.
pub(crate) fn text(text: &Text) -> Element {
    match text {
        Text::Plain(plain) => rsx! { "{plain}" },
        Text::Runs(runs) => {
            let drawn: Vec<Element> = runs.iter().map(run).collect();
            rsx! {
                for (key , piece) in drawn.into_iter().enumerate() {
                    Fragment { key: "{key}", {piece} }
                }
            }
        }
    }
}

/// One run in its tone.
fn run(run: &Run) -> Element {
    let body = run.text.clone();
    match (run.tone, run.tone.slug()) {
        (RunTone::Mark, _) => rsx! {
            mark { class: "ds-mark", "{body}" }
        },
        (_, Some(tone)) => rsx! {
            span { class: "ds-run", "data-tone": tone, "{body}" }
        },
        (_, None) => rsx! { "{body}" },
    }
}

#[cfg(test)]
mod tests {
    use super::{Run, RunTone, Text};

    #[test]
    fn the_plain_text_drops_the_tones() {
        let cases = [
            (Text::from("Re: UIDL"), "Re: UIDL"),
            (
                Text::Runs(vec![
                    Run::new("Re: ", RunTone::Faint),
                    Run::new("UIDL", RunTone::Mark),
                    Run::new(" stability", RunTone::Plain),
                ]),
                "Re: UIDL stability",
            ),
            (Text::Runs(Vec::new()), ""),
        ];
        for (text, want) in cases {
            assert_eq!(text.plain_text(), want, "{text:?}");
        }
    }
}
