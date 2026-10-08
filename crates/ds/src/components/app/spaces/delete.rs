//! Delete Space...: the question, where the Space's menu stood.

use super::controller::SpacesHandle;
use super::menu_pick::delete_words;
use super::parts::placed;
use crate::components::controls::button::Button;
use crate::components::overlays::popover::{Arrow, Popover};
use crate::focus::soon::focus_soon;
use crate::host::measure::Anchor;
use crate::root::common::Common;
use dioxus::prelude::*;
use ds_core::geometry::units::{Point, Px};
use ds_core::press::{PointerButton, Press};
use ds_style::icon::Icon;
use ds_style::space::list::SpaceId;
use ds_style::tokens::control_size::ControlSize;

/// The question, naming the Space, and what stays. Cancel, the safe default, takes the keyboard
/// as it opens, so Return cancels. Only the Delete button deletes; Escape or a click outside
/// deletes nothing. Nothing is offered for the last Space, whose menu has no Delete row.
/// `kept` names what is not deleted; `on_deleted` hears the Space that went.
#[component]
pub(super) fn DeletePart<P, R>(
    at: Point,
    space: SpaceId,
    handle: SpacesHandle<P, R>,
    kept: String,
    on_deleted: Option<EventHandler<SpaceId>>,
) -> Element
where
    P: Clone + PartialEq + 'static,
    R: Clone + Default + PartialEq + 'static,
{
    let (count, name) = {
        let signal = handle.spaces();
        let spaces = signal.read();
        (
            spaces.count(),
            spaces
                .get(space)
                .map(|one| one.name.clone())
                .unwrap_or_default(),
        )
    };
    if count <= 1 {
        return rsx! {};
    }
    let (title, body) = delete_words(&name, &kept);
    let on_primary = |act: fn(SpacesHandle<P, R>)| {
        move |press: Press| {
            if press.button == PointerButton::Primary {
                act(handle);
            }
        }
    };
    rsx! {
        Popover {
            anchor: Anchor::Point(at),
            placement: placed(),
            gap: Px(2.0),
            arrow: Arrow::None,
            common: Common { aria_label: Some(title.clone()), ..Common::default() },
            onclose: move |()| handle.close_menu(),
            div { class: "ds-space-part ds-space-delete", role: "alertdialog",
                h3 { "{title}" }
                p { class: "ds-space-delete-body", "{body}" }
                div { class: "ds-space-part-acts",
                    Button {
                        size: ControlSize::Large,
                        label: "Cancel",
                        common: Common {
                            mounted: Some(EventHandler::new(|event: MountedEvent| {
                                focus_soon(event.data());
                            })),
                            ..Common::default()
                        },
                        onclick: on_primary(|handle| handle.close_menu()),
                    }
                    Button {
                        size: ControlSize::Large,
                        label: "Delete Space",
                        icon: Icon::Trash,
                        common: Common { aria_label: Some("Delete Space".to_owned()), ..Common::default() },
                        onclick: move |press: Press| {
                            if press.button != PointerButton::Primary {
                                return;
                            }
                            handle.close_menu();
                            if handle.delete(space).is_ok()
                                && let Some(on_deleted) = on_deleted
                            {
                                on_deleted.call(space);
                            }
                        },
                    }
                }
            }
        }
    }
}
