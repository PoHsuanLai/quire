//! Rich: a body of runs that may hold links (notification parts, sill Q124). The Notifications
//! spec lets a body carry `<b>`, `<i>`, `<u>` and `<a href>`; the server parses that markup into
//! runs once, and quire draws them: a toned run as [`Text`] draws it, a link as
//! `a.ds-run-link`. A link keeps its press ([`Propagation::Stop`](crate::Propagation)'s rule:
//! propagation stopped and the default prevented), so a press on a link inside a notification
//! opens the link and never also opens the notification, and Blitz never navigates the
//! document itself.
//!
//! A [`Text`], a `String` or a `&str` converts into a `Rich`, so a body that has no links is
//! written as before.

use crate::components::text_runs::{OptionalText, Run, RunTone, Text, edges, run};
use crate::focus::click::kept_click;
use dioxus::core::SuperFrom;
use dioxus::prelude::*;

/// One piece of a rich body.
#[derive(Debug, Clone, PartialEq, Eq, Hash)]
pub enum RichRun {
    /// Characters in a tone, as in a [`Text`].
    Run(Run),
    /// A link: its words, and where it goes, handed to the caller's `on_link` on a press.
    Link {
        /// The words drawn.
        text: String,
        /// The target, as the markup gave it; quire never opens it.
        href: String,
    },
}

impl RichRun {
    /// A link run.
    pub fn link(text: impl Into<String>, href: impl Into<String>) -> Self {
        RichRun::Link {
            text: text.into(),
            href: href.into(),
        }
    }

    /// The characters, tone and target dropped.
    fn chars(&self) -> &str {
        match self {
            RichRun::Run(run) => &run.text,
            RichRun::Link { text, .. } => text,
        }
    }
}

/// A body: runs in order, some of them links.
#[derive(Debug, Clone, PartialEq, Eq, Hash, Default)]
pub struct Rich(pub Vec<RichRun>);

impl Rich {
    /// The characters with tones and links dropped: what a label reads and a search matches.
    pub fn plain_text(&self) -> String {
        self.0.iter().map(RichRun::chars).collect()
    }
}

impl From<Text> for Rich {
    fn from(text: Text) -> Self {
        match text {
            Text::Plain(plain) => Rich(vec![RichRun::Run(Run::new(plain, RunTone::Plain))]),
            Text::Runs(runs) => Rich(runs.into_iter().map(RichRun::Run).collect()),
        }
    }
}

impl From<String> for Rich {
    fn from(text: String) -> Self {
        Rich::from(Text::Plain(text))
    }
}

impl From<&str> for Rich {
    fn from(text: &str) -> Self {
        Rich::from(text.to_owned())
    }
}

impl From<Vec<RichRun>> for Rich {
    fn from(runs: Vec<RichRun>) -> Self {
        Rich(runs)
    }
}

/// An optional `Rich` prop takes a `String` as `Some`, as an optional `Text` prop does.
impl SuperFrom<String, OptionalText> for Option<Rich> {
    fn super_from(text: String) -> Self {
        Some(Rich::from(text))
    }
}

/// An optional `Rich` prop takes a `&str` as `Some`.
impl<'a> SuperFrom<&'a str, OptionalText> for Option<Rich> {
    fn super_from(text: &'a str) -> Self {
        Some(Rich::from(text))
    }
}

/// An optional `Rich` prop takes a `Text` as `Some`.
impl SuperFrom<Text, OptionalText> for Option<Rich> {
    fn super_from(text: Text) -> Self {
        Some(Rich::from(text))
    }
}

/// A rich body drawn where the caller puts it: the runs in their tones and the links as
/// `a.ds-run-link`, each press on a link handed to `on_link` and kept there. `NotificationCard`
/// draws its body with it; a history row of the caller's own can too.
#[component]
pub fn RichText(
    #[props(into)] body: Rich,
    #[props(default)] on_link: Option<EventHandler<String>>,
) -> Element {
    rich(&body, on_link)
}

/// `body` drawn: each run in its tone, each link as `a.ds-run-link` reporting to `on_link`.
pub(crate) fn rich(body: &Rich, on_link: Option<EventHandler<String>>) -> Element {
    let drawn: Vec<Element> = body
        .0
        .iter()
        .map(|piece| match piece {
            RichRun::Run(one) => run(one),
            RichRun::Link { text, href } => link(text, href, on_link),
        })
        .collect();
    rsx! {
        for (key , piece) in drawn.into_iter().enumerate() {
            Fragment { key: "{key}", {piece} }
        }
    }
}

/// One link. Its edge spaces are drawn outside the element (Blitz drops an inline box's
/// trailing whitespace, as for a toned run). A press or Enter keeps the event at the link and
/// hands the target to `on_link`; with no handler the press still stops there, so it never
/// opens what the link sits in.
fn link(text: &str, href: &str, on_link: Option<EventHandler<String>>) -> Element {
    let (lead, core, trail) = edges(text);
    let (lead, core, trail) = (lead.to_owned(), core.to_owned(), trail.to_owned());
    let target = href.to_owned();
    let pressed = target.clone();
    rsx! {
        if !lead.is_empty() {
            "{lead}"
        }
        a {
            class: "ds-run-link",
            href: "{href}",
            tabindex: "0",
            onclick: move |event| {
                event.stop_propagation();
                event.prevent_default();
                if let Some(on_link) = on_link {
                    on_link.call(pressed.clone());
                }
                kept_click(&event);
            },
            onkeydown: move |event| {
                if event.key() == Key::Enter {
                    event.stop_propagation();
                    event.prevent_default();
                    if let Some(on_link) = on_link {
                        on_link.call(target.clone());
                    }
                }
            },
            "{core}"
        }
        if !trail.is_empty() {
            "{trail}"
        }
    }
}

#[cfg(test)]
mod tests {
    use super::{Rich, RichRun};
    use crate::components::text_runs::{Run, RunTone, Text};

    #[test]
    fn a_rich_body_reads_as_its_characters() {
        let cases = [
            (Rich::from("Build finished"), "Build finished"),
            (
                Rich(vec![
                    RichRun::Run(Run::new("See ", RunTone::Plain)),
                    RichRun::link("the log", "https://ci.example/42"),
                    RichRun::Run(Run::new(" now", RunTone::Italic)),
                ]),
                "See the log now",
            ),
            (
                Rich::from(Text::Runs(vec![Run::new("a", RunTone::Strong)])),
                "a",
            ),
            (Rich::default(), ""),
        ];
        for (body, want) in cases {
            assert_eq!(body.plain_text(), want, "{body:?}");
        }
    }
}
