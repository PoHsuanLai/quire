//! Delete Space...: the question, as an alert.

use super::controller::SpacesHandle;
use super::menu_pick::delete_words;
use crate::components::content::text_runs::TextLine;
use crate::components::overlays::alert::Alert;
use crate::components::overlays::alert_model::{AlertButton, AlertRole, AlertStyle};
use dioxus::prelude::*;
use ds_style::space::list::SpaceId;

/// The question, naming the Space, and what is not deleted, drawn as every other question that cannot be
/// undone is: a critical [`Alert`]. Cancel, the safe default, takes the keyboard as it opens, so
/// Return cancels, and Escape presses it. Only Delete deletes. Nothing is offered for the last
/// Space, whose menu has no Delete row. `kept` names what is not deleted; `on_deleted` hears the
/// Space that went.
#[component]
pub(super) fn DeletePart<P, R>(
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
    let delete = AlertButton::new(
        "Delete",
        AlertRole::Destructive,
        EventHandler::new(move |()| {
            handle.close_menu();
            if handle.delete(space).is_ok()
                && let Some(on_deleted) = on_deleted
            {
                on_deleted.call(space);
            }
        }),
    );
    let cancel = AlertButton::new(
        "Cancel",
        AlertRole::Cancel,
        EventHandler::new(move |()| handle.close_menu()),
    );
    rsx! {
        Alert {
            title,
            message: Some(TextLine::from(body)),
            style: AlertStyle::Critical,
            buttons: vec![delete, cancel],
        }
    }
}
