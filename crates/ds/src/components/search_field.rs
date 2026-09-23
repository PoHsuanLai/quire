//! SearchField: a search icon and an inline field heading a list of results, with operator
//! tokens under it (design/04-COMPONENTS.md section 7).

use crate::components::chip::{Chip, ChipVariant};
use crate::components::text_input::{Focus, InputVariant, TextInput};
use crate::icon::Icon;
use crate::icon::render::Glyph;
use dioxus::prelude::*;

/// The command menu's and the launcher's search row. `focus` is the field's
/// ([`Focus::OnMount`] in the command palette).
#[component]
pub fn SearchField(
    label: String,
    value: String,
    placeholder: String,
    tokens: Vec<String>,
    oninput: EventHandler<String>,
    onkey: EventHandler<KeyboardData>,
    #[props(default)] focus: Focus,
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
