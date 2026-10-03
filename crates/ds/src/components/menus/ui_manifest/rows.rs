//! Reading an app's menus and shortcut table into rows.

use crate::components::menus::item::item::MenuItem;
use crate::components::menus::menu_bar::MenuBarModel;
use crate::components::menus::ui_manifest::model::CommandSource;
use ds_core::command::{AppCommand, CommandFace, CommandId, ShortcutBinding};
use ds_core::vocab::Shortcut;

/// One menu item or shortcut binding, read from the app's tables.
#[derive(Debug, Clone, PartialEq, Eq)]
pub struct CommandRow {
    /// The command's id.
    pub id: CommandId,
    /// Where it is declared.
    pub source: CommandSource,
    /// The key equivalent it shows or is bound to.
    pub chord: Option<Shortcut>,
    /// What it is the face of.
    pub face: CommandFace,
}

/// A row for every command in `items`, submenus included; headers, status lines and rules are
/// no commands.
pub fn menu_rows<T: AppCommand>(items: &[MenuItem<T>]) -> Vec<CommandRow> {
    items
        .iter()
        .flat_map(|item| match item {
            MenuItem::Item { value, key, .. } => vec![CommandRow {
                id: value.id(),
                source: CommandSource::Menu,
                chord: key.clone(),
                face: value.face(),
            }],
            MenuItem::Submenu { children, .. } => menu_rows(children),
            MenuItem::Header(_) | MenuItem::Info { .. } | MenuItem::Separator => Vec::new(),
        })
        .collect()
}

/// A row for every command in the menu bar, its six menus in the bar's order.
pub fn bar_rows<T: AppCommand + Clone>(bar: &MenuBarModel<T>) -> Vec<CommandRow> {
    bar.menus()
        .iter()
        .flat_map(|menu| menu_rows(&menu.items))
        .collect()
}

/// A row for every binding of the app's shortcut table.
pub fn shortcut_rows<T: AppCommand>(bindings: &[ShortcutBinding<T>]) -> Vec<CommandRow> {
    bindings
        .iter()
        .map(|binding| CommandRow {
            id: binding.command.id(),
            source: CommandSource::Shortcut,
            chord: Some(binding.keys.clone()),
            face: binding.command.face(),
        })
        .collect()
}
