//! ProviderList: the first step of adding an account: a search field over the providers the
//! person may pick, and a generic "Other" row for any server that speaks the open protocols
//! (design/31 section 5.6). The query and the row under the cursor are the host's.

use super::adapter::{Action, Entry, EntryLook, Intent, Landing, PickRow, PickRows};
use super::frame::StepFrame;
use super::model::{FieldText, ProviderEntry, ProviderPick, StepTitle};
use super::wording::{CursorStep, matching, pick_for_enter, step_cursor};
use dioxus::prelude::*;
use ds::components::content::provider_mark::MarkProvider;

/// The provider list. `query` is what the search field holds; `cursor` the row the arrow keys
/// rest on. `on_pick` hears a press or Return on a row (Return with no cursor picks the first
/// match), `on_cancel` Cancel and Escape. Down and Up in the search field move the cursor
/// through the rows (`on_cursor`), so the keyboard never has to leave the field.
#[component]
pub fn ProviderList(
    providers: Vec<ProviderEntry>,
    query: String,
    #[props(default)] cursor: Option<ProviderPick>,
    on_query: EventHandler<String>,
    on_cursor: EventHandler<ProviderPick>,
    on_pick: EventHandler<ProviderPick>,
    on_cancel: EventHandler<()>,
    #[props(default)] title: StepTitle,
) -> Element {
    let matches = matching(&providers, &query);
    let enter = pick_for_enter(&matches, cursor.as_ref());
    let rows: Vec<PickRow<ProviderPick>> = matches
        .iter()
        .map(|entry| PickRow {
            key: ProviderPick::Provider(entry.key.clone()),
            title: entry.label.clone(),
            mark: entry.mark,
        })
        .chain([PickRow {
            key: ProviderPick::Other,
            title: "Other\u{2026}".to_owned(),
            mark: MarkProvider::Imap,
        }])
        .collect();
    let keys: Vec<ProviderPick> = rows.iter().map(|row| row.key.clone()).collect();
    let walk = cursor.clone();
    rsx! {
        StepFrame {
            shown: title,
            step: "providers",
            title: "Add Account",
            onenter: EventHandler::new(move |()| on_pick.call(enter.clone())),
            oncancel: on_cancel,
            body: rsx! {
                Entry {
                    label: "Search providers",
                    text: FieldText::Plain(query.clone()),
                    placeholder: "Search",
                    look: EntryLook::Search,
                    landing: Landing::Here,
                    onkey: move |event: KeyboardEvent| {
                        let step = match event.key() {
                            Key::ArrowDown => CursorStep::Next,
                            Key::ArrowUp => CursorStep::Previous,
                            _ => return,
                        };
                        event.prevent_default();
                        if let Some(next) = step_cursor(&keys, walk.as_ref(), step) {
                            on_cursor.call(next);
                        }
                    },
                    oninput: move |text: FieldText| {
                        if let FieldText::Plain(next) = text {
                            on_query.call(next);
                        }
                    },
                }
                PickRows { label: "Providers", rows, cursor, oncursor: on_cursor, onpick: on_pick }
            },
            actions: rsx! {
                Action { label: "Cancel", intent: Intent::Cancel, onclick: move |()| on_cancel.call(()) }
            },
        }
    }
}
