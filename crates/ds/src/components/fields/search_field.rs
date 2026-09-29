//! SearchField: a search icon and an inline field heading a list of results, with operator
//! tokens under it (design/04-COMPONENTS.md section 7).

use crate::components::controls::chip::{Chip, ChipVariant};
use crate::components::fields::{
    text_input::{InputVariant, TextInput},
    text_input_focus::FieldFocus,
};
use crate::focus::field::FieldHandle;
use crate::style::icon::Icon;
use crate::style::icon::render::Glyph;
use dioxus::prelude::*;

/// The command menu's and the launcher's search row. `focus` is the field's
/// ([`Focus::Controlled`] in the command palette); `onkey` hears each key as the event itself.
/// `handle` is the field's, as `TextInput { handle }` takes it.
#[component]
pub fn SearchField(
    label: String,
    value: String,
    placeholder: String,
    tokens: Vec<String>,
    oninput: EventHandler<String>,
    onkey: EventHandler<KeyboardEvent>,
    #[props(default)] focus: FieldFocus,
    #[props(default)] handle: Option<FieldHandle>,
) -> Element {
    // The icon keeps the base 16: S's `.cmdk-in` does not size it (C draws 18).
    // The tokens row is drawn only when there are tokens; the doc does not say whether an
    // empty row keeps its 8 px bottom padding.
    rsx! {
        div { class: "ds-search",
            Glyph { icon: Icon::Search }
            TextInput {
                variant: InputVariant::Inline,
                label,
                value,
                placeholder,
                oninput,
                onkey,
                focus,
                handle,
            }
        }
        if !tokens.is_empty() {
            div { class: "ds-search-tokens",
                for token in tokens {
                    Chip { variant: ChipVariant::Token, text: token }
                }
            }
        }
    }
}
