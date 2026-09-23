//! AppearancePicker: THE one picker for Theme, Accent and Motion, in mailo, the control center
//! and settings (design/04-COMPONENTS.md section 26).

use crate::appearance::{Appearance, SystemPrefs};
use dioxus::prelude::*;

/// Theme, accent and motion rows.
#[component]
pub fn AppearancePicker(
    value: Appearance,
    system: SystemPrefs,
    onchange: EventHandler<Appearance>,
) -> Element {
    todo!()
}
