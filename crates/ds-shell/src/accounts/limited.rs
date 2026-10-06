//! LimitedNote: the one secondary line under an account or a service that is less than the
//! provider's full product (design/31 section 2.5).

use super::model::Limitation;
use super::wording::limitation;
use dioxus::prelude::*;
use ds::components::content::label::{Label, LabelRole, LabelStyle};

/// Why it is limited, in the secondary ink.
#[component]
pub fn LimitedNote(limit: Limitation) -> Element {
    rsx! {
        Label { text: limitation(limit), role: LabelRole::Secondary, style: LabelStyle::Footnote }
    }
}
