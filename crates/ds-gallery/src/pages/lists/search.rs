//! Lists, for mail: rows as a search draws them, the hits marked by the caller, and the
//! strip revealed on the keyboard's row (Up and Down here are the two buttons) rather than on
//! hover, titled, with its label button's menu open.

use crate::pages::Section;
use dioxus::prelude::*;
use ds::components::app::hover_strip::{ActionId, HoverStrip, StripAction, Titles};
use ds::components::app::thread_row::ThreadRow;
use ds::components::content::text_runs::RunTone;
use ds::prelude::*;
use ds::root::common::Common;
use ds::style::tokens::control_size::ControlSize;
use ds_core::vocab::RowState;

/// A search's rows: sender, the subject and snippet as runs around the hit, time.
fn hits() -> [(&'static str, TextLine, TextLine, &'static str); 2] {
    let hit = |before: &str, word: &str, after: &str| {
        TextLine::Runs(vec![
            TextRun::new(before, RunTone::Plain),
            TextRun::new(word, RunTone::Mark),
            TextRun::new(after, RunTone::Plain),
        ])
    };
    [
        (
            "Dana Okafor",
            TextLine::Runs(vec![
                TextRun::new("Re: ", RunTone::Faint),
                TextRun::new("UIDL", RunTone::Mark),
                TextRun::new(" stability across servers", RunTone::Plain),
            ]),
            hit("Treat ", "UIDL", " as stable only while UIDVALIDITY holds."),
            "09:41",
        ),
        (
            "Sam Lindqvist",
            hit("Notes on ", "UIDL", " from the sync review"),
            hit("Three things to change before ", "UIDL", " moves."),
            "09:12",
        ),
    ]
}

/// The strip's actions, the label one opening a menu.
fn actions() -> Vec<StripAction> {
    [
        (
            "archive",
            Icon::Archive,
            "Archive",
            "Archive → out of Inbox",
        ),
        ("label", Icon::Tag, "Label", "Label…"),
    ]
    .into_iter()
    .map(|(id, icon, label, fly)| StripAction {
        id: ActionId(id.to_string()),
        icon,
        label: label.to_string(),
        fly: fly.to_string(),
        onhover: None,
        onclick: EventHandler::new(|_| {}),
    })
    .collect()
}

#[component]
pub fn SearchRows() -> Element {
    let mut at = use_signal(|| 0usize);
    rsx! {
        Section {
            title: "Search hits and a keyboard-shown strip",
            note: "Subject and snippet are Text runs the caller marked. The strip shows on the selected row through shown, not hover (Blitz never matches :focus-within); its buttons carry titles and the label button's menu is open.",
            div { class: "g-row",
                Button { size: ControlSize::Mini, label: "Up", onclick: move |_| at.set(0) }
                Button { size: ControlSize::Mini, label: "Down", onclick: move |_| at.set(1) }
            }
            div { class: "g-list g-stage-pad",
                List::<usize> {
                    label: "Search results",
                    cursor: Some(at()),
                    items: hits()
                        .into_iter()
                        .enumerate()
                        .map(|(index, (name, subject, snippet, time))| {
                            ListItem::row(
                                index,
                                name,
                                rsx! {
                                    ThreadRow {
                                        state: RowState { selection: if at() == index { Selection::Selected } else { Selection::Unselected }, emphasis: Emphasis::Plain, ..RowState::default() },
                                        name,
                                        via: None,
                                        subject: subject.clone(),
                                        snippet,
                                        time,
                                        tags: rsx! {},
                                        star: None,
                                        strip: rsx! {
                                            HoverStrip {
                                                actions: actions(),
                                                shown: if at() == index { Shown::Visible } else { Shown::Hidden },
                                                titles: Titles::FromLabel,
                                                expanded: vec![(ActionId("label".to_string()), if at() == index { Shown::Visible } else { Shown::Hidden })],
                                            }
                                        },
                                        common: Common { aria_label: Some(format!("Open {}", subject.plain_text())), ..Common::default() },
                                        onclick: move |_| at.set(index),
                                    }
                                },
                            )
                        })
                        .collect::<Vec<_>>(),
                    onselect: move |index: usize| at.set(index),
                }
            }
        }
    }
}
