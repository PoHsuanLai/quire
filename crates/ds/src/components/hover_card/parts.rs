//! A hover card's content blocks as data (design/04-COMPONENTS.md section 22 "Markup"), so a
//! consumer composes a card from parts rather than writing `ds-hovercard-*` markup by hand.
//! Each part renders exactly the block the section draws; the card renders them in the order
//! given, before any children.

use crate::components::avatar::{Avatar, AvatarSize, AvatarTone};
use crate::components::kbd::Kbd;
use crate::components::text_runs::{Text, text as runs};
use crate::components::vocab::Shortcut;
use crate::icon::Icon;
use crate::icon::render::{Glyph, IconSize};
use dioxus::prelude::*;

/// One block of a hover card.
#[derive(Debug, Clone, PartialEq)]
pub enum HoverCardPart {
    /// The heading, display 14 (`h5.ds-hovercard-title`).
    Title(String),
    /// The line under it: an address, a date (`.ds-hovercard-sub`).
    Sub(String),
    /// A sender card's header: a 34 px avatar beside the title and an optional sub.
    Person {
        /// The avatar's letter.
        initial: char,
        /// The avatar's colours.
        tone: AvatarTone,
        /// The name.
        title: String,
        /// The address under it.
        sub: Option<String>,
    },
    /// A row of figures, each a value over its label: "14 threads", "Mon last wrote".
    Stats(Vec<HoverStat>),
    /// A warning or a note, tinted by tone, with its glyph.
    Flag {
        /// Danger (`--danger-wash`) or info (`--accent-soft`).
        tone: FlagTone,
        /// The glyph, coloured by the tone.
        icon: Icon,
        /// What it says.
        text: String,
    },
    /// A flag whose words are runs (mailo gaps 5): the spoof warning sets the brand and the
    /// domain in the strong tone ("Not **Acme**: sent from **acme-billing.example**"). The
    /// same block as [`HoverCardPart::Flag`], either tone; build it with
    /// [`HoverCardPart::flag`], which takes a `String`, a `&str` or a [`Text`]. A variant of its
    /// own rather than a change to `Flag`'s `text`, so every `Flag { text: String }` literal
    /// keeps compiling.
    FlagText {
        /// Danger (`--danger-wash`) or info (`--accent-soft`).
        tone: FlagTone,
        /// The glyph, coloured by the tone.
        icon: Icon,
        /// What it says, in runs.
        text: Text,
    },
    /// A thread card's latest messages, each a 22 px avatar, a name and two lines of text.
    Messages(Vec<HoverMessage>),
    /// The quiet line at the foot, with an optional key hint pushed to the right.
    Foot {
        /// "stays unread while you look".
        text: String,
        /// The key and what it does: `Space` peek.
        keys: Option<KeyHint>,
    },
    /// Mini buttons, wrapping.
    Actions(Vec<Element>),
}

impl HoverCardPart {
    /// A flag of either tone whose words are plain or runs: [`HoverCardPart::FlagText`].
    pub fn flag(tone: FlagTone, icon: Icon, text: impl Into<Text>) -> Self {
        HoverCardPart::FlagText {
            tone,
            icon,
            text: text.into(),
        }
    }
}

/// A flag's tone: `data-tone` on `.ds-hovercard-flag`.
#[derive(Debug, Clone, Copy, PartialEq, Eq, Hash)]
pub enum FlagTone {
    /// A spoof or a failure: the danger wash, a danger glyph.
    Danger,
    /// A note: the accent's soft fill, an accent glyph.
    Info,
}

impl FlagTone {
    /// The `data-tone` word.
    pub fn slug(self) -> &'static str {
        match self {
            FlagTone::Danger => "danger",
            FlagTone::Info => "info",
        }
    }
}

/// One figure in [`HoverCardPart::Stats`].
#[derive(Debug, Clone, PartialEq, Eq, Hash)]
pub struct HoverStat {
    /// The figure, display 15: "14".
    pub value: String,
    /// What it counts: "threads".
    pub label: String,
}

/// One message in [`HoverCardPart::Messages`].
#[derive(Debug, Clone, PartialEq, Eq, Hash)]
pub struct HoverMessage {
    /// The sender's letter.
    pub initial: char,
    /// The sender's colours.
    pub tone: AvatarTone,
    /// The sender's short name.
    pub name: String,
    /// The message's opening, clipped by the caller to two lines' worth
    /// (`ds::clip_chars`; Blitz has no line clamp, the sheet's height is only the guard).
    pub text: String,
}

/// A key hint in [`HoverCardPart::Foot`].
#[derive(Debug, Clone, PartialEq, Eq, Hash)]
pub struct KeyHint {
    /// The keys.
    pub shortcut: Shortcut,
    /// What they do: "peek".
    pub label: String,
}

/// One part, drawn.
pub(super) fn part(part: HoverCardPart) -> Element {
    match part {
        HoverCardPart::Title(text) => rsx! { h5 { class: "ds-hovercard-title", "{text}" } },
        HoverCardPart::Sub(text) => rsx! { div { class: "ds-hovercard-sub", "{text}" } },
        HoverCardPart::Person {
            initial,
            tone,
            title,
            sub,
        } => rsx! {
            div { class: "ds-hovercard-person",
                Avatar { initial, size: AvatarSize::Size34, tone }
                div {
                    h5 { class: "ds-hovercard-title", "{title}" }
                    if let Some(sub) = sub {
                        div { class: "ds-hovercard-sub", "{sub}" }
                    }
                }
            }
        },
        HoverCardPart::Stats(stats) => rsx! {
            div { class: "ds-hovercard-stats",
                for stat in stats {
                    span { b { "{stat.value}" } "{stat.label}" }
                }
            }
        },
        HoverCardPart::Flag { tone, icon, text } => flag(tone, icon, &Text::Plain(text)),
        HoverCardPart::FlagText { tone, icon, text } => flag(tone, icon, &text),
        HoverCardPart::Messages(messages) => rsx! {
            div { class: "ds-hovercard-msgs",
                for message in messages {
                    div { class: "ds-hovercard-msg",
                        Avatar { initial: message.initial, size: AvatarSize::Size22, tone: message.tone }
                        div {
                            b { "{message.name}" }
                            p { "{message.text}" }
                        }
                    }
                }
            }
        },
        HoverCardPart::Foot { text, keys } => rsx! {
            div { class: "ds-hovercard-foot",
                "{text}"
                if let Some(keys) = keys {
                    span { class: "ds-hovercard-keys",
                        Kbd { shortcut: keys.shortcut }
                        " {keys.label}"
                    }
                }
            }
        },
        HoverCardPart::Actions(actions) => rsx! {
            div { class: "ds-hovercard-actions",
                for action in actions {
                    {action}
                }
            }
        },
    }
}

/// A flag block: the tone's glyph beside its words.
fn flag(tone: FlagTone, icon: Icon, text: &Text) -> Element {
    rsx! {
        div { class: "ds-hovercard-flag", "data-tone": tone.slug(),
            Glyph { icon, size: IconSize::Compact }
            span { {runs(text)} }
        }
    }
}
