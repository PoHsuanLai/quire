//! The install command line: the one line to copy, and the Copy button beside it. Return on the
//! button copies; it does not reach the sheet's own Return, which closes.

use crate::accounts::adapter::CopyAction;
use crate::accounts::model::CopyState;
use dioxus::prelude::*;

/// `text` as one selectable line with a Copy button. The host does the copying: it hears
/// `on_copy` with `text`, and `copy` is its word on whether it has.
#[component]
pub(crate) fn CommandLine(text: String, copy: CopyState, on_copy: EventHandler<String>) -> Element {
    rsx! {
        div {
            class: "ds-helper-command",
            onkeydown: move |event| {
                if event.key() == Key::Enter {
                    event.stop_propagation();
                }
            },
            span { class: "ds-helper-command-text", "{text}" }
            CopyAction {
                label: "Copy",
                copied: "Copied",
                text: text.clone(),
                state: copy,
                oncopy: on_copy,
            }
        }
    }
}
