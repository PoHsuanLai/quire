//! AccountBadge: an account as a line: its provider's mark and its name.

use dioxus::prelude::*;
use ds::components::content::label::{Label, LabelStyle};
use ds::components::content::provider_mark::{MarkProvider, ProviderMark};
use ds_style::tokens::control_size::ControlSize;

/// `provider`'s mark beside `label`.
#[component]
pub fn AccountBadge(provider: MarkProvider, #[props(into)] label: String) -> Element {
    rsx! {
        span { class: "ds-acc-badge",
            ProviderMark { provider, size: ControlSize::Regular }
            Label { text: label, style: LabelStyle::Body }
        }
    }
}
