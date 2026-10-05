//! SpaceColour: the colour half of the Space editor on its own, for a host that edits a Space a
//! part at a time (a popover per part, opened from the Space's menu) rather than in one panel:
//! the field and its handles, the stops, the grain and the presets. The name, the theme, the
//! card's accent and the contrast readout are the host's to offer elsewhere; the theme and the
//! accent are each one `SegmentedControl` (or a menu of checks) over [`SpaceLook`].
//!
//! Controlled as [`super::SpaceEditor`] is: every edit is a whole new look through `onchange`,
//! and `active_dot` seeds which dot the handles and keys move, a pick inside reported through
//! `on_active_dot`.

use super::handles::Field;
use super::parts::{GrainRow, Presets, Stops};
use super::rows::EditorFrame;
use super::{ActiveDot, DotIndex, Picker, active};
use crate::components::lists::section_header::SectionHeader;
use crate::root::common::Common;
use dioxus::prelude::*;
use ds_core::word::Word;
use ds_style::appearance::theme::Scheme;
use ds_style::space::look::SpaceLook;

/// The Space's colour: field, stops, grain and presets, in the editor's own markup
/// (`aside.ds-space-editor[data-part=colour]`), so it is styled as the editor is. Unframed by
/// default, since its host (a popover) is already a surface.
#[component]
pub fn SpaceColour(
    look: SpaceLook,
    scheme: Scheme,
    active_dot: DotIndex,
    onchange: EventHandler<SpaceLook>,
    #[props(default)] on_active_dot: Option<EventHandler<ActiveDot>>,
    #[props(default = EditorFrame::Frameless)] frame: EditorFrame,
    #[props(default)] common: Common,
) -> Element {
    let picked = use_signal(|| None::<(DotIndex, DotIndex)>);
    let picker = Picker {
        picked,
        prop: active_dot,
        report: on_active_dot,
    };
    let current = active(active_dot, picked(), look.dots.len());
    let class = common.class("ds-space-editor");
    let data = common.data_attributes();
    let label = common
        .aria_label
        .clone()
        .unwrap_or_else(|| "Space colour".to_owned());
    rsx! {
        aside {
            class,
            id: common.id.clone(),
            "aria-label": "{label}",
            "data-part": "colour",
            "data-frame": match frame {
                EditorFrame::Card => None,
                EditorFrame::Frameless => Some(frame.slug()),
            },
            onmounted: move |event| common.mounted(event),
            ..data,
            div {
                SectionHeader { title: "Colour", value: "drag a dot".to_string() }
                Field { look: look.clone(), scheme, current, picker, onchange }
                Stops { look: look.clone(), scheme, current, picker, onchange }
            }
            GrainRow { look: look.clone(), onchange }
            Presets { look, scheme, picker, onchange }
        }
    }
}
