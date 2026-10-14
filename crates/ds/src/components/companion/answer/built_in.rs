//! The buttons a plan and a replacement carry themselves, since their models hold phases and no
//! action list: each has a fixed id the companion hears back.

use super::action::{ActionRole, CardAction, CardActionId};
use ds_core::vocab::{Availability, EffectMark};

/// A built-in action: enabled, with no key of its own.
pub fn built_in(id: &str, label: &str, role: ActionRole, effect: EffectMark) -> CardAction {
    CardAction {
        id: CardActionId(id.to_owned()),
        label: label.to_owned(),
        role,
        effect,
        keys: None,
        availability: Availability::Enabled,
    }
}
