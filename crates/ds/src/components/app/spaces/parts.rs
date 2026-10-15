//! The part of a Space's menu that opens where the menu stood: its name.

use super::controller::SpacesHandle;
use crate::components::fields::text_field::TextField;
use crate::components::fields::text_field_focus::FieldFocus;
use crate::components::overlays::popover::{Arrow, Popover};
use crate::focus::request::use_focus_request;
use crate::host::measure::Anchor;
use crate::root::common::Common;
use dioxus::prelude::*;
use ds_core::geometry::placement::{Align, Placement, Side};
use ds_core::geometry::units::{Point, Px};
use ds_style::space::list::SpaceId;

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
