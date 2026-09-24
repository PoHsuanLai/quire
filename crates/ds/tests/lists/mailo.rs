//! The mailo gaps 2 row cases: a search hit's runs in the subject and snippet, a row named for
//! a screen reader, and the strip revealed or held down by its caller, titled, with a button
//! whose menu is open.

use crate::cases::Case;
use crate::rows::strip_actions;
use dioxus::prelude::*;
use ds::components::vocab::{Emphasis, Expanded, PulseKey, Selection, StaggerIndex};
use ds::{ActionId, Anim, HoverStrip, ListRow, Presence, Run, RunTone, Shown, Text, Titles};

/// "Re: UIDL stability" with the hit marked and the prefix faint.
fn marked_subject() -> Text {
    Text::Runs(vec![
        Run::new("Re: ", RunTone::Faint),
        Run::new("UIDL", RunTone::Mark),
        Run::new(" stability across ", RunTone::Plain),
        Run::new("servers", RunTone::Strong),
    ])
}

/// A read row carrying `strip`, its subject and snippet marked.
fn row_with(strip: Option<Element>) -> Element {
    rsx! {
        ListRow {
            selection: Selection::Selected,
            emphasis: Emphasis::Plain,
            index: StaggerIndex::new(0),
            presence: Presence::Present,
            name: "Dana Okafor",
            via: None,
            subject: marked_subject(),
            snippet: Text::Runs(vec![
                Run::new("Treat ", RunTone::Plain),
                Run::new("UIDL", RunTone::Mark),
                Run::new(" as stable only while UIDVALIDITY holds.", RunTone::Plain),
            ]),
            time: "09:41",
            tags: rsx! {},
            star: None,
            star_pulse: PulseKey::rest(Anim::StarPop),
            strip,
            onclick: |_| {},
            aria_label: "Open Re: UIDL stability across servers".to_string(),
        }
    }
}

/// The strip revealed by the caller, each button titled, the label menu open.
fn caller_strip(shown: Shown) -> Element {
    rsx! {
        HoverStrip {
            actions: strip_actions(),
            shown,
            titles: Titles::FromLabel,
            expanded: vec![
                (ActionId("snooze".to_string()), Expanded::Closed),
                (ActionId("label".to_string()), Expanded::Open),
            ],
        }
    }
}

pub const MAILO_CASES: &[Case] = &[
    Case {
        component: "list_row",
        state: "marked-runs",
        make: || row_with(None),
    },
    Case {
        component: "list_row",
        state: "strip-shown-by-caller",
        make: || row_with(Some(caller_strip(Shown::Visible))),
    },
    Case {
        component: "hover_strip",
        state: "shown-visible",
        make: || caller_strip(Shown::Visible),
    },
    Case {
        component: "hover_strip",
        state: "shown-hidden",
        make: || caller_strip(Shown::Hidden),
    },
];
