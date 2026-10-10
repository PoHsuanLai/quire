//! The sidebar foot: [leading slot] [the Space dots] [New Space, unless hidden] [trailing slot].

use super::chord::SwitchChord;
use super::controller::SpacesHandle;
use super::new_space::NewSpace;
use super::open_menu::Showing;
use crate::components::app::space_editor::dot::SpaceDot;
use crate::components::controls::button::Button;
use crate::components::controls::button_model::{Bezel, ImagePosition};
use dioxus::prelude::*;
use ds_core::geometry::units::{Point, Px};
use ds_core::vocab::{Selection, Shortcut};
use ds_style::icon::Icon;
use ds_style::scope::use_scope;
use ds_style::space::frame_vars::FrameVars;
use ds_style::space::look::SpaceLook;

/// The foot of a sidebar that has Spaces.
///
/// `leading` is the app's own (mailo puts Downloads there); `trailing` the app's (Settings, Hide
/// sidebar). The dots take the free width between. Every dot is the quiet neutral one and the
/// others are dimmed, as Dia draws them: a Space's own colours are its frame's, not the
/// picker's. A right click on a dot is that Space's menu; a click switches, unless a part of a
/// Space's menu is open. `+` adds a Space over `new_payload` and opens its name.
///
/// `new_space: NewSpace::Hidden` leaves the `+` out, for a foot whose one menu holds New Space
/// (Dia's chevron, a `PullDownButton` in `trailing`). That menu's item makes the Space itself:
/// `let made = handle.add(payload); handle.show(made, at, Showing::Rename);`, both reached
/// through `SpacesHandle` and `Showing` (in the prelude).
#[component]
pub fn SpacesFoot<P, R>(
    handle: SpacesHandle<P, R>,
    new_payload: Callback<(), P>,
    #[props(default)] chord: SwitchChord,
    #[props(default)] new_space: NewSpace,
    #[props(default)] new_shortcut: Option<Shortcut>,
    #[props(default)] leading: Option<Element>,
    #[props(default)] trailing: Option<Element>,
) -> Element
where
    P: Clone + PartialEq + 'static,
    R: Clone + Default + PartialEq + 'static,
{
    let scheme = use_scope().scheme;
    let current = handle.current();
    let dots: Vec<_> = handle
        .spaces()
        .read()
        .list()
        .iter()
        .enumerate()
        .map(|(index, space)| (index, space.id, space.name.clone()))
        .collect();
    rsx! {
        div { class: "ds-spaces-foot",
            if let Some(leading) = leading {
                {leading}
            }
            div { class: "ds-spaces-dots", role: "group", "aria-label": "Spaces",
                for (index, id, name) in dots {
                    span {
                        key: "{id.0}",
                        class: "ds-spaces-dot-hold",
                        "data-current": if id == current { "true" } else { "false" },
                        oncontextmenu: move |event: MouseEvent| {
                            event.prevent_default();
                            event.stop_propagation();
                            let at = event.client_coordinates();
                            handle.open_menu(id, Point { x: Px(at.x as f32), y: Px(at.y as f32) });
                        },
                        SpaceDot {
                            name,
                            frame: FrameVars::of(&SpaceLook::default(), scheme),
                            selection: Selection::of(&id, &current),
                            shortcut: chord.shortcut(index),
                            onclick: move |()| {
                                handle.switch(id);
                            },
                        }
                    }
                }
            }
            if new_space == NewSpace::Shown {
                Button {
                    bezel: Bezel::Toolbar,
                    image: ImagePosition::Only,
                    icon: Icon::Plus,
                    label: "New Space",
                    title: "New Space".to_owned(),
                    title_shortcut: new_shortcut,
                    onclick: move |press: ds_core::press::Press| {
                        if handle.editing().is_none() {
                            let made = handle.add(new_payload.call(()));
                            handle.show(made, press.at, Showing::Rename);
                        }
                    },
                }
            }
            if let Some(trailing) = trailing {
                {trailing}
            }
        }
    }
}
