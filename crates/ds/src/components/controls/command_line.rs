//! CommandLine: a command to copy, as one selectable line in the data face with a Copy button
//! beside it. Markup: `div.ds-command-line` holding `span.ds-command-line-text` and a `Button`.

use crate::components::controls::button::Button;
use crate::components::controls::copy_state::CopyState;
use dioxus::prelude::*;

/// `text` as one selectable line with a Copy button.
///
/// The host does the copying: it hears `on_copy` with `text` when the button is pressed and
/// writes the clipboard (`ds_blitz::clipboard::write_text`; `ds` reaches no renderer), and `copy`
/// is its word on whether it has, so the label says "Copied" until the host sets it back to
/// `Idle`. Return on the button presses it and goes no further, so a sheet around the line keeps
/// its own Return (the default button).
#[component]
pub fn CommandLine(
    text: String,
    #[props(default)] copy: CopyState,
    on_copy: EventHandler<String>,
) -> Element {
    let label = match copy {
        CopyState::Idle => "Copy",
        CopyState::Copied => "Copied",
    };
    let copied = text.clone();
    rsx! {
        div {
            class: "ds-command-line",
            onkeydown: move |event| {
                if event.key() == Key::Enter {
                    event.stop_propagation();
                }
            },
            span { class: "ds-command-line-text", "{text}" }
            Button {
                label,
                onclick: move |_| on_copy.call(copied.clone()),
            }
        }
    }
}
