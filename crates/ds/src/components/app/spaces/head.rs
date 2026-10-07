//! The sidebar head: the Space's name, quiet, with its menu on a right click only.

use super::controller::SpacesHandle;
use crate::components::content::label::{Label, LabelRole, LabelStyle};
use crate::root::common::Common;
use crate::root::pass_through::ExtraClass;
use dioxus::prelude::*;
use ds_core::geometry::units::{Point, Px};

/// The current Space's name heading the sidebar, as a browser's does. A right click on it opens
/// the Space's menu; a plain press does nothing (a quiet Dia, not an Arc).
#[component]
pub fn SpaceHead<P, R>(handle: SpacesHandle<P, R>) -> Element
where
    P: Clone + PartialEq + 'static,
    R: Clone + Default + PartialEq + 'static,
{
    let (id, name) = {
        let signal = handle.spaces();
        let spaces = signal.read();
        (spaces.current().id, spaces.current().name.clone())
    };
    rsx! {
        div {
            class: "ds-space-head",
            oncontextmenu: move |event: MouseEvent| {
                event.prevent_default();
                let at = event.client_coordinates();
                handle.open_menu(id, Point { x: Px(at.x as f32), y: Px(at.y as f32) });
            },
            Label {
                text: name.clone(),
                style: LabelStyle::Headline,
                role: LabelRole::Secondary,
                common: Common {
                    aria_label: Some(format!("The {name} Space")),
                    extra_class: ExtraClass::parse("ds-truncate").ok(),
                    ..Common::default()
                },
            }
        }
    }
}
