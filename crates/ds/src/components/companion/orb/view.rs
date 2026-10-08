//! The companion orb component.

use super::model::OrbSize;
use crate::components::companion::outcome::model::Outcome;
use crate::components::companion::outcome::view::OutcomeMark;
use crate::root::common::Common;
use dioxus::prelude::*;
use ds_core::vocab::CompanionPresence;
use ds_core::word::Word;

/// The companion's presence as an orb: a wrapper over the voice orb whose look follows the
/// presence. `common.aria_label` defaults to the presence's words ("Companion is working").
/// `outcome` is a result the person has not seen: a still dot on the orb's corner while it is
/// `Some`, gone when the caller passes `None`. It does not change the presence.
#[component]
pub fn CompanionOrb(
    presence: CompanionPresence,
    #[props(default)] size: OrbSize,
    #[props(default)] outcome: Option<Outcome>,
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
            OutcomeMark { outcome }
        }
    }
}
