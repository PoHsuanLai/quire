//! The mail-app thread row cases: a search hit's runs in the subject and snippet, a row named for
//! a screen reader, and the strip revealed or held down by its caller, titled, with a button
//! whose menu is open.

use crate::cases::Case;
use crate::rows::strip_actions;
use dioxus::prelude::*;
use ds::components::app::hover_strip::{ActionId, HoverStrip, Titles};
use ds::components::app::thread_row::ThreadRow;
use ds::components::content::text_runs::RunTone;
use ds::prelude::*;
use ds::root::common::Common;
use ds_core::vocab::RowState;

/// "Re: UIDL stability" with the hit marked and the prefix faint.
fn marked_subject() -> TextLine {
    TextLine::Runs(vec![
        TextRun::new("Re: ", RunTone::Faint),
        TextRun::new("UIDL", RunTone::Mark),
        TextRun::new(" stability across ", RunTone::Plain),
        TextRun::new("servers", RunTone::Strong),
    ])
}

/// A read row carrying `strip`, its subject and snippet marked.
fn row_with(strip: Option<Element>) -> Element {
    rsx! {
        ThreadRow {
            state: RowState { selection: Selection::Selected, emphasis: Emphasis::Plain, ..RowState::default() },
            name: "Dana Okafor",
            via: None,
            subject: marked_subject(),
            snippet: TextLine::Runs(vec![
                TextRun::new("Treat ", RunTone::Plain),
                TextRun::new("UIDL", RunTone::Mark),
                TextRun::new(" as stable only while UIDVALIDITY holds.", RunTone::Plain),
            ]),
            time: "09:41",
            tags: rsx! {},
            star: None,
            strip,
            onclick: |_| {},
            common: Common { aria_label: Some("Open Re: UIDL stability across servers".to_string()), ..Common::default() },
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
                (ActionId("snooze".to_string()), Shown::Hidden),
                (ActionId("label".to_string()), Shown::Visible),
            ],
        }
    }
}

pub const THREAD_ROW_CASES: &[Case] = &[
    Case {
        component: "thread_row",
        state: "marked-runs",
        make: || row_with(None),
    },
    Case {
        component: "thread_row",
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
