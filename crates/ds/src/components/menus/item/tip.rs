//! A menu row's tooltip: a transparent zone over the row that feeds the hover hub, so the tip
//! shows at once (the Tip profile) on a disabled row too. The row's own handlers still hear the
//! pointer, which bubbles from the zone.

use crate::components::content::tip_text::TipText;
use crate::components::content::title_tip::use_tip;
use dioxus::prelude::*;

/// The tip `tip` over the row it sits in. The row's markup carries the full wording as
/// `aria-description`.
#[component]
pub(crate) fn RowTip(tip: TipText) -> Element {
    let tip = use_tip(Some(tip));
    rsx! {
        span {
            class: "ds-menu-tip",
            onmouseover: {
                let tip = tip.clone();
                move |event| tip.over(&event)
            },
            onmouseleave: {
                let tip = tip.clone();
                move |_| tip.out()
            },
            onmounted: {
                let tip = tip.clone();
                move |event| tip.mounted(&event)
            },
        }
        {tip.surface()}
    }
}
