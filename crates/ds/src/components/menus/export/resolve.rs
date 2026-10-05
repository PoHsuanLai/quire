//! From the id an outside process names back to the value the app's own menu would yield.

use crate::components::menus::item::item::MenuItem;
use crate::components::menus::menu_bar::{BarCommand, MenuBarModel};
use ds_core::command::{AppCommand, CommandId};
use ds_core::standard_action::StandardAction;

impl BarCommand {
    /// The stable id of a standard bar item (`standard.undo`, `bar.about`, `bar.help_search`):
    /// an app whose command type wraps [`BarCommand`] answers
    /// [`AppCommand::id`] with it, so every quire app names its standard items alike.
    pub fn id(&self) -> CommandId {
        CommandId(match self {
            BarCommand::About => "bar.about".to_owned(),
            BarCommand::HelpSearch => "bar.help_search".to_owned(),
            BarCommand::Standard(action) => format!("standard.{}", snake(action)),
        })
    }
}

/// `PasteAndMatchStyle` as `paste_and_match_style`.
fn snake(action: &StandardAction) -> String {
    format!("{action:?}")
        .chars()
        .enumerate()
        .flat_map(|(at, c)| match (at, c.is_uppercase()) {
            (0, _) | (_, false) => vec![c.to_ascii_lowercase()],
            (_, true) => vec!['_', c.to_ascii_lowercase()],
        })
        .collect()
}

fn find<'a, T: AppCommand>(items: &'a [MenuItem<T>], id: &CommandId) -> Option<&'a T> {
    items.iter().find_map(|item| match item {
        MenuItem::Item { value, .. } if &value.id() == id => Some(value),
        MenuItem::Submenu { children, .. } => find(children, id),
        _ => None,
    })
}

impl<T: AppCommand + Clone> MenuBarModel<T> {
    /// The value of the first item that runs `id`: what the app's own menu yields when that
    /// item is picked, so an activation from outside runs the same handler.
    pub fn command(&self, id: &CommandId) -> Option<&T> {
        self.menus().iter().find_map(|menu| find(&menu.items, id))
    }
}
