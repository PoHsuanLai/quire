//! What `ListRow`'s `snippet: Option<Text>` accepts, compiled and rendered (CONSUMING.md). Each
//! form draws the snippet it was given, or none.

use dioxus::prelude::*;
use ds::{Emphasis, ListRow, Presence, RowState, RunTone, Selection, TextLine, TextRun};

/// A `ListRow` with everything fixed but its snippet, written as the expression given.
macro_rules! row {
    ($snippet:expr) => {
        rsx! {
            ListRow {
                state: RowState { selection: Selection::Unselected, emphasis: Emphasis::Plain, ..RowState::default() },
                presence: Presence::Present,
                name: "Dana",
                via: None,
                subject: "Re: UIDL",
                snippet: $snippet,
                time: "09:41",
                tags: rsx! {},
                star: None,
                strip: None,
                onclick: |_| {},
            }
        }
    };
}

/// Every accepted form, in CONSUMING.md's order.
fn forms() -> Element {
    let owned = "an owned String".to_string();
    let held: Option<String> = Some("an Option String you hold".to_string());
    let n = 3;
    rsx! {
        {row!("a str")}
        {row!(owned)}
        {row!(TextLine::Runs(vec![TextRun::new("runs", RunTone::Mark)]))}
        {row!(Some(TextLine::from("Some(Text)")))}
        {row!(Some("Some(string.into())".to_string().into()))}
        {row!(held.map(TextLine::from))}
        {row!(None)}
        ListRow {
            state: RowState { selection: Selection::Unselected, emphasis: Emphasis::Plain, ..RowState::default() },
            presence: Presence::Present,
            name: "Dana",
            via: None,
            subject: "Re: UIDL",
            snippet: "formatted {n}",
            time: "09:41",
            tags: rsx! {},
            star: None,
            strip: None,
            onclick: |_| {},
        }
    }
}

fn render(make: fn() -> Element) -> String {
    let mut dom = VirtualDom::new(make);
    dom.rebuild_in_place();
    dioxus_ssr::render(&dom)
}

#[test]
fn every_accepted_snippet_form_draws_its_text() {
    let html = render(forms);
    let snippets: Vec<&str> = html
        .split("class=\"ds-row-snip ds-truncate\">")
        .skip(1)
        .filter_map(|rest| rest.split('<').next())
        .collect();
    assert_eq!(
        snippets,
        [
            "a str",
            "an owned String",
            "",
            "Some(Text)",
            "Some(string.into())",
            "an Option String you hold",
            "formatted 3",
        ],
        "the runs row starts with its mark, and None draws no snippet"
    );
}
