//! The Space's menu: a context `Menu` at the pointer, and the part each row opens.

use super::controller::SpacesHandle;
use super::delete::DeletePart;
use super::edit::EditSpaceSheet;
use super::menu_link::{DesktopSpace, LinkChange, Linking};
use super::menu_pick::{SpacePick, Then, apply, rows};
use super::open_menu::{OpenMenu, Showing};
use super::parts::RenamePart;
use crate::components::menus::item::item::MenuItem;
use crate::components::menus::menu::menu::Menu;
use crate::components::menus::menu::placement::MenuPlacement;
use crate::host::measure::Anchor;
use crate::root::common::Common;
use dioxus::prelude::*;
use ds_style::space::list::SpaceId;

/// The menu of whichever Space the handle has open; nothing while none is.
///
/// Rows, in order: Edit Space..., Appearance, the app's `extra`
/// (one submenu, say Accounts, whose picks reach `on_extra`; give them `AfterPick::KeepOpen` to
/// keep the menu up), "Link to..." and "Unlink" when `desktop` lists the desktop's Spaces, a
/// rule, New Space, and Delete Space... while there is more than one.
/// Edit Space... opens [`EditSpaceSheet`]: the name and colour, changed live, with Cancel and Save;
/// an app that edits a Space on a settings page of its own gives `on_edit` instead, and Edit
/// Space... closes the menu and calls it with the Space's id (to open that page).
/// New Space opens its name at once, a field at the pointer that renames live (Escape puts the old
/// name back). Delete asks in a critical alert, as any question that cannot be undone does. Each part takes the keyboard as it opens and
/// hands it back on close. `on_deleted` hears a
/// deleted Space's id: the app calls `TodayHandle::drop_space` there. `desktop` is the desktop's
/// Spaces (`None` for an app with no desktop to follow: no link rows); a Link or Unlink pick is
/// stored on the Space, and every link pick reaches `on_link`, New Desktop Space... included,
/// which only the app can make. The kit never clears a link of a desktop Space that is gone:
/// the app does, with `SpacesHandle::set_link`. An app with no `extra` names
/// `A` as `()`: `SpaceMenu::<_, _, ()>`.
#[component]
pub fn SpaceMenu<P, R, A>(
    handle: SpacesHandle<P, R>,
    new_payload: Callback<(), P>,
    #[props(default)] extra: Vec<MenuItem<A>>,
    #[props(default)] on_extra: Option<EventHandler<(SpaceId, A)>>,
    #[props(default)] desktop: Option<Vec<DesktopSpace>>,
    #[props(default)] on_link: Option<EventHandler<(SpaceId, LinkChange)>>,
    #[props(default = "Your data".to_owned())] kept: String,
    #[props(default)] on_deleted: Option<EventHandler<SpaceId>>,
    #[props(default)] on_edit: Option<EventHandler<SpaceId>>,
) -> Element
where
    P: Clone + PartialEq + 'static,
    R: Clone + Default + PartialEq + 'static,
    A: Clone + PartialEq + 'static,
{
    let Some(open) = handle.open() else {
        return rsx! {};
    };
    // A new Space is named before it is in the list.
    if open.showing == Showing::Rename {
        return rsx! { RenamePart { at: open.at, space: open.space, handle } };
    }
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
                    items: rows(&space.look, count, extra, linking(space.link.as_deref(), desktop.as_deref())),
                    common: Common {
                        aria_label: Some(format!("{} Space", space.name)),
                        ..Common::default()
                    },
                    onpick: move |pick: SpacePick<A>| {
                        let look = handle.spaces().read().get(id).map(|one| one.look.clone());
                        let Some(look) = look else { return };
                        match apply(pick, &look) {
                            Then::Kept(look) => handle.apply(id, |one| one.look = look),
                            Then::Open(part) => match (part, on_edit) {
                                (Showing::Edit, Some(on_edit)) => {
                                    handle.close_menu();
                                    on_edit.call(id);
                                }
                                _ => handle.show(id, at, part),
                            },
                            Then::NewSpace => handle.begin_new(new_payload.call(()), at),
                            Then::Link(change) => {
                                match &change {
                                    LinkChange::Linked(desktop) => {
                                        handle.set_link(id, Some(desktop.clone()));
                                    }
                                    LinkChange::Unlinked => handle.set_link(id, None),
                                    LinkChange::NewDesktopSpace => {}
                                }
                                if let Some(on_link) = on_link {
                                    on_link.call((id, change));
                                }
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
        Showing::Rename => rsx! {},
        Showing::Edit => rsx! { EditSpaceSheet { space: id, handle } },
        Showing::Delete => rsx! {
            DeletePart {
                space: id,
                handle,
                kept,
                on_deleted,
            }
        },
    }
}

/// What the menu offers: the link rows when the app gave the desktop's Spaces.
fn linking<'a>(link: Option<&'a str>, desktop: Option<&'a [DesktopSpace]>) -> Linking<'a> {
    desktop.map_or(Linking::Off, |desktop| Linking::Offered {
        current: link,
        desktop,
    })
}
