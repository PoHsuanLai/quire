//! The parts of a Space's menu that open where the menu stood: its name, and its colour.

use super::controller::SpacesHandle;
use crate::components::app::space_editor::DotIndex;
use crate::components::app::space_editor::colour::SpaceColour;
use crate::components::fields::text_field::TextField;
use crate::components::fields::text_field_focus::FieldFocus;
use crate::components::overlays::popover::{Arrow, Popover};
use crate::focus::request::use_focus_request;
use crate::focus::select::Select;
use crate::focus::selector::focus_by_selector;
use crate::host::measure::Anchor;
use crate::root::common::Common;
use dioxus::prelude::*;
use ds_core::geometry::placement::{Align, Placement, Side};
use ds_core::geometry::units::{Point, Px};
use ds_style::scope::use_scope;
use ds_style::space::list::SpaceId;
use ds_style::space::look::SpaceLook;

/// The colour part's first dot handle.
const FIRST_HANDLE: &str = ".ds-space-colour .ds-handle";

/// Where a part stands: below and after the point its menu opened at, as the menu did.
pub(super) fn placed() -> Placement {
    Placement::new(Side::Bottom, Align::Start)
}

/// The Space's name in a field that takes the keyboard, selected so typing replaces it. Each
/// keystroke renames the Space; Return and a click outside close it, keeping the name, and
/// Escape closes it with the name it opened on put back.
#[component]
pub(super) fn RenamePart<P, R>(at: Point, space: SpaceId, handle: SpacesHandle<P, R>) -> Element
where
    P: Clone + PartialEq + 'static,
    R: Clone + Default + PartialEq + 'static,
{
    let focus = FieldFocus::Controlled(use_focus_request().with_select_all());
    let opened_on = use_hook(|| {
        handle
            .spaces()
            .peek()
            .get(space)
            .map(|space| space.name.clone())
            .unwrap_or_default()
    });
    let Some(name) = handle
        .spaces()
        .read()
        .get(space)
        .map(|space| space.name.clone())
    else {
        return rsx! {};
    };
    rsx! {
        Popover {
            anchor: Anchor::Point(at),
            placement: placed(),
            gap: Px(2.0),
            arrow: Arrow::None,
            common: Common { aria_label: Some("Rename Space".to_owned()), ..Common::default() },
            onclose: move |()| handle.close_menu(),
            div { class: "ds-space-part ds-space-rename",
                TextField {
                    label: "Space name",
                    value: name,
                    focus,
                    oninput: move |typed: String| handle.change(space, |one| one.name = typed),
                    onkey: move |event: KeyboardEvent| {
                        match event.key() {
                            Key::Enter => {
                                event.prevent_default();
                                handle.close_menu();
                            }
                            Key::Escape => {
                                event.prevent_default();
                                event.stop_propagation();
                                let old = opened_on.clone();
                                handle.change(space, move |one| one.name = old);
                                handle.close_menu();
                            }
                            _ => {}
                        }
                    },
                }
            }
        }
    }
}

/// The Space's colour: the existing `SpaceColour` (field, stops, grain, presets), in the scheme
/// the window is drawn in. The frame repaints with each change. The first dot's handle takes the
/// keyboard as the part opens, so the arrow keys move it; Escape and a click outside close the
/// part, keeping what was changed (as every part does: it has no Cancel).
#[component]
pub(super) fn ColourPart<P, R>(at: Point, space: SpaceId, handle: SpacesHandle<P, R>) -> Element
where
    P: Clone + PartialEq + 'static,
    R: Clone + Default + PartialEq + 'static,
{
    let scheme = use_scope().scheme;
    let mut active = use_signal(DotIndex::default);
    use_hook(|| {
        spawn(async move {
            let _ = focus_by_selector(FIRST_HANDLE, Select::None).await;
        });
    });
    let Some(look) = handle
        .spaces()
        .read()
        .get(space)
        .map(|space| space.look.clone())
    else {
        return rsx! {};
    };
    rsx! {
        Popover {
            anchor: Anchor::Point(at),
            placement: placed(),
            gap: Px(2.0),
            arrow: Arrow::None,
            common: Common { aria_label: Some("Space colour".to_owned()), ..Common::default() },
            onclose: move |()| handle.close_menu(),
            div { class: "ds-space-part ds-space-colour",
                SpaceColour {
                    look,
                    scheme,
                    active_dot: active(),
                    onchange: move |look: SpaceLook| handle.change(space, |one| one.look = look),
                    on_active_dot: move |dot: DotIndex| active.set(dot),
                }
            }
        }
    }
}
