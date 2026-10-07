//! The Space's menu: a context `Menu` at the pointer, and the part each row opens.

use super::controller::SpacesHandle;
use super::delete::DeletePart;
use super::menu_pick::{SpacePick, Then, apply, rows};
use super::open_menu::{OpenMenu, Showing};
use super::parts::{ColourPart, RenamePart};
use crate::components::menus::item::item::MenuItem;
use crate::components::menus::menu::menu::Menu;
use crate::components::menus::menu::placement::MenuPlacement;
use crate::host::measure::Anchor;
use crate::root::common::Common;
use dioxus::prelude::*;
use ds_style::space::list::SpaceId;

/// The menu of whichever Space the handle has open; nothing while none is.
///
/// Rows, in order: Rename..., Colour..., Appearance, Accent Inside the Card, the app's `extra`
/// (one submenu, say Accounts, whose picks reach `on_extra`; give them `AfterPick::KeepOpen` to
/// keep the menu up), a rule, New Space, and Delete Space... while there is more than one.
/// New Space opens its Rename at once. Colour, Rename and Delete are popovers at the pointer; a
/// part changes the Space live and keeps it on close, with no Cancel. `on_deleted` hears a
/// deleted Space's id: the app calls `TodayHandle::drop_space` there. An app with no `extra` names
/// `A` as `()`: `SpaceMenu::<_, _, ()>`.
#[component]
pub fn SpaceMenu<P, R, A>(
    handle: SpacesHandle<P, R>,
    new_payload: Callback<(), P>,
    #[props(default)] extra: Vec<MenuItem<A>>,
    #[props(default)] on_extra: Option<EventHandler<(SpaceId, A)>>,
    #[props(default = "Your data".to_owned())] kept: String,
    #[props(default)] on_deleted: Option<EventHandler<SpaceId>>,
) -> Element
where
    P: Clone + PartialEq + 'static,
    R: Clone + Default + PartialEq + 'static,
    A: Clone + PartialEq + 'static,
{
    let Some(open) = handle.open() else {
        return rsx! {};
    };
    let Some(space) = handle.spaces().read().get(open.space).cloned() else {
        return rsx! {};
    };
    let OpenMenu {
        space: id,
        at,
        showing,
    } = open;
    match showing {
        Showing::Menu => {
            let count = handle.spaces().read().count();
            rsx! {
                Menu::<SpacePick<A>> {
                    placement: MenuPlacement::Context,
                    anchor: Anchor::Point(at),
                    items: rows(&space.look, count, extra),
                    common: Common {
                        aria_label: Some(format!("{} Space", space.name)),
                        ..Common::default()
                    },
                    onpick: move |pick: SpacePick<A>| {
                        let look = handle.spaces().read().get(id).map(|one| one.look.clone());
                        let Some(look) = look else { return };
                        match apply(pick, &look) {
                            Then::Kept(look) => handle.apply(id, |one| one.look = look),
                            Then::Open(part) => handle.show(id, at, part),
                            Then::NewSpace => {
                                let made = handle.add(new_payload.call(()));
                                handle.show(made, at, Showing::Rename);
                            }
                            Then::App(pick) => {
                                if let Some(on_extra) = on_extra {
                                    on_extra.call((id, pick));
                                }
                            }
                        }
                    },
                    // A pick that opened a part has already put the part in the menu's place:
                    // only the menu itself closes here.
                    onclose: move |()| {
                        if handle.open().map(|open| open.showing) == Some(Showing::Menu) {
                            handle.close_menu();
                        }
                    },
                }
            }
        }
        Showing::Rename => rsx! { RenamePart { at, space: id, handle } },
        Showing::Colour => rsx! { ColourPart { at, space: id, handle } },
        Showing::Delete => rsx! {
            DeletePart {
                at,
                space: id,
                handle,
                kept,
                on_deleted,
            }
        },
    }
}
