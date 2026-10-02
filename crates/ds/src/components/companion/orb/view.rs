//! The companion orb component.

use super::model::OrbSize;
use crate::root::common::Common;
use dioxus::prelude::*;
use ds_core::vocab::CompanionPresence;
use ds_core::word::Word;

/// The companion's presence as an orb: a wrapper over the voice orb whose look follows the
/// presence. `common.aria_label` defaults to the presence's words ("Companion is working").
#[component]
pub fn CompanionOrb(
    presence: CompanionPresence,
    #[props(default)] size: OrbSize,
    #[props(default)] common: Common,
) -> Element {
    let class = common.class("ds-companion-orb");
    let data = common.data_attributes();
    rsx! {
        div {
            id: common.id.clone(),
            class,
            "data-presence": presence.slug(),
            "data-size": size.slug(),
            onmounted: move |event| common.mounted(event),
            ..data,
        }
    }
}
