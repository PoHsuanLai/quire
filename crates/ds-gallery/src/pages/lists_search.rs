//! Lists, mailo gaps 2: rows as a search draws them, the hits marked by the caller, and the
//! strip revealed on the keyboard's row (Up and Down here are the two buttons) rather than on
//! hover, titled, with its label button's menu open.

use super::Section;
use dioxus::prelude::*;
use ds::{
    ActionId, Anim, Button, ButtonVariant, Emphasis, HoverStrip, Icon, ListRow, Presence, Run,
    RunTone, Selection, Shown, StaggerIndex, StripAction, Switch, Text, Titles,
};

/// A search's rows: sender, the subject and snippet as runs around the hit, time.
fn hits() -> [(&'static str, Text, Text, &'static str); 2] {
    let hit = |before: &str, word: &str, after: &str| {
        Text::Runs(vec![
            Run::new(before, RunTone::Plain),
            Run::new(word, RunTone::Mark),
            Run::new(after, RunTone::Plain),
        ])
    };
    [
        (
            "Dana Okafor",
            Text::Runs(vec![
                Run::new("Re: ", RunTone::Faint),
                Run::new("UIDL", RunTone::Mark),
                Run::new(" stability across servers", RunTone::Plain),
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
                Button { variant: ButtonVariant::Mini, label: "Up", onclick: move |_| at.set(0) }
                Button { variant: ButtonVariant::Mini, label: "Down", onclick: move |_| at.set(1) }
            }
            ul { class: "g-list g-stage-pad",
                for (index , (name , subject , snippet , time)) in hits().into_iter().enumerate() {
                    ListRow {
                        key: "{name}",
                        selection: if at() == index { Selection::Selected } else { Selection::Unselected },
                        emphasis: Emphasis::Plain,
                        index: StaggerIndex::new(index),
                        presence: Presence::Present,
                        name,
                        via: None,
                        aria_label: format!("Open {}", subject.plain_text()),
                        subject,
                        snippet,
                        time,
                        tags: rsx! {},
                        star: None,
                        star_pulse: ds::PulseKey::rest(Anim::StarPop),
                        strip: rsx! {
                            HoverStrip {
                                actions: actions(),
                                shown: if at() == index { Shown::Visible } else { Shown::Hidden },
                                titles: Titles::FromLabel,
                                expanded: vec![(ActionId("label".to_string()), if at() == index { Switch::On } else { Switch::Off })],
                            }
                        },
                        onclick: move |_| at.set(index),
                    }
                }
            }
        }
    }
}
