//! MenuBar: the model of an app's menu bar (`NSMenu` mainMenu, design/30 section 2.4): the six
//! menus in the order the Mac draws them (App, File, Edit, View, Window, Help), each with its
//! standard items and their standard key equivalents, Help led by its search. Data only: the bar
//! that draws it is the shell's, and a menu's rows are `MenuItem`s the one `Menu` draws.
//!
//! An item's value is the caller's command type `T`: `MenuBarModel::standard` is given the
//! function that turns a [`BarCommand`] into it, and the app adds its own items to a section
//! with [`MenuBarModel::with_items`].

use crate::components::menus::item::item::{MenuImage, MenuItem};
use ds_core::standard_action::StandardAction;
use ds_core::vocab::{Availability, Shortcut};
use ds_core::word::Word;
use ds_style::icon::Icon;
use serde::{Deserialize, Serialize};

/// A menu of the bar, in the order the bar draws them.
#[derive(Debug, Clone, Copy, PartialEq, Eq, Hash, Word, Serialize, Deserialize)]
#[serde(rename_all = "kebab-case")]
pub enum BarSection {
    /// The app's own: About, Settings, Hide, Quit. Titled with the app's name.
    App,
    /// File.
    File,
    /// Edit.
    Edit,
    /// View.
    View,
    /// Window.
    Window,
    /// Help: the search, then the app's help.
    Help,
}

/// What a standard item of the bar asks for.
#[derive(Debug, Clone, Copy, PartialEq, Eq, Hash)]
pub enum BarCommand {
    /// The app's About panel.
    About,
    /// One of the standard actions, bound to its reserved keys.
    Standard(StandardAction),
    /// Focus the Help menu's search field.
    HelpSearch,
}

/// One menu: its section, its title and its rows.
#[derive(Debug, Clone, PartialEq)]
pub struct BarMenu<T> {
    /// Which of the six.
    pub section: BarSection,
    /// What the bar says: the app's name for `App`, the section's label otherwise.
    pub title: String,
    /// Its rows.
    pub items: Vec<MenuItem<T>>,
}

/// An app's menu bar.
#[derive(Debug, Clone, PartialEq)]
pub struct MenuBarModel<T> {
    menus: Vec<BarMenu<T>>,
}

/// A standard item: its title and the standard action whose keys it shows.
struct Standard {
    title: &'static str,
    action: StandardAction,
}

const fn std_item(title: &'static str, action: StandardAction) -> Standard {
    Standard { title, action }
}

const FILE: &[Standard] = &[
    std_item("New", StandardAction::New),
    std_item("Open…", StandardAction::Open),
    std_item("Close Window", StandardAction::Close),
    std_item("Save", StandardAction::Save),
    std_item("Print…", StandardAction::Print),
];

const EDIT: &[Standard] = &[
    std_item("Undo", StandardAction::Undo),
    std_item("Redo", StandardAction::Redo),
    std_item("Cut", StandardAction::Cut),
    std_item("Copy", StandardAction::Copy),
    std_item("Paste", StandardAction::Paste),
    std_item("Select All", StandardAction::SelectAll),
    std_item("Find…", StandardAction::Find),
];

const VIEW: &[Standard] = &[
    std_item("Show Sidebar", StandardAction::ToggleSidebar),
    std_item("Show Toolbar", StandardAction::ToggleToolbar),
    std_item("Enter Full Screen", StandardAction::FullScreen),
];

const WINDOW: &[Standard] = &[
    std_item("Minimize", StandardAction::Minimize),
    std_item("Minimize All", StandardAction::MinimizeAll),
];

/// The rows of a run of standard items.
fn rows<T>(standard: &[Standard], command: &impl Fn(BarCommand) -> T) -> Vec<MenuItem<T>> {
    standard
        .iter()
        .map(|item| {
            MenuItem::new(command(BarCommand::Standard(item.action)), item.title)
                .with_key(Shortcut::standard(item.action))
        })
        .collect()
}

impl<T: Clone> MenuBarModel<T> {
    /// The six standard menus of an app called `app`. `command` turns each standard item's
    /// [`BarCommand`] into the value the app's menu handler takes.
    pub fn standard(app: &str, command: impl Fn(BarCommand) -> T) -> Self {
        let keyed = |title: String, action: StandardAction| {
            MenuItem::new(command(BarCommand::Standard(action)), title)
                .with_key(Shortcut::standard(action))
        };
        let app_menu = vec![
            MenuItem::new(command(BarCommand::About), format!("About {app}")),
            MenuItem::Separator,
            keyed("Settings…".to_owned(), StandardAction::Settings),
            MenuItem::Separator,
            keyed(format!("Hide {app}"), StandardAction::Hide),
            keyed("Hide Others".to_owned(), StandardAction::HideOthers),
            MenuItem::Separator,
            keyed(format!("Quit {app}"), StandardAction::Quit),
        ];
        let help = vec![
            MenuItem::new(command(BarCommand::HelpSearch), "Search")
                .with_image(MenuImage::Icon(Icon::Search)),
            MenuItem::Separator,
            keyed(format!("{app} Help"), StandardAction::Help),
        ];
        let menu = |section: BarSection, items: Vec<MenuItem<T>>| BarMenu {
            title: match section {
                BarSection::App => app.to_owned(),
                other => other.label().to_owned(),
            },
            section,
            items,
        };
        MenuBarModel {
            menus: vec![
                menu(BarSection::App, app_menu),
                menu(BarSection::File, rows(FILE, &command)),
                menu(BarSection::Edit, rows(EDIT, &command)),
                menu(BarSection::View, rows(VIEW, &command)),
                menu(BarSection::Window, rows(WINDOW, &command)),
                menu(BarSection::Help, help),
            ],
        }
    }

    /// The menus, in the bar's order.
    pub fn menus(&self) -> &[BarMenu<T>] {
        &self.menus
    }

    /// The menu of `section`.
    pub fn menu(&self, section: BarSection) -> Option<&BarMenu<T>> {
        self.menus.iter().find(|menu| menu.section == section)
    }

    /// The same bar with the app's own `items` after `section`'s standard ones, a rule between.
    /// The Help menu keeps its search first, so the app's items follow its help.
    pub fn with_items(self, section: BarSection, items: Vec<MenuItem<T>>) -> Self {
        let menus = self
            .menus
            .into_iter()
            .map(|mut menu| {
                if menu.section == section && !items.is_empty() {
                    menu.items.push(MenuItem::Separator);
                    menu.items.extend(items.clone());
                }
                menu
            })
            .collect();
        MenuBarModel { menus }
    }

    /// The same bar with each item's availability as `available` says for its command: a
    /// command with nothing to act on (Paste with an empty clipboard) is drawn disabled.
    pub fn availability(self, available: impl Fn(&T) -> Availability) -> Self {
        let menus = self
            .menus
            .into_iter()
            .map(|mut menu| {
                menu.items = menu
                    .items
                    .into_iter()
                    .map(|item| match &item {
                        MenuItem::Item { value, .. } => {
                            let availability = available(value);
                            item.with_availability(availability)
                        }
                        _ => item,
                    })
                    .collect();
                menu
            })
            .collect();
        MenuBarModel { menus }
    }
}

/// The menu a key press opens rather than an item it picks: `⌘?` opens Help, with its search
/// focused.
pub fn opens(pressed: &Shortcut) -> Option<BarSection> {
    (pressed == &Shortcut::standard(StandardAction::Help)).then_some(BarSection::Help)
}

#[cfg(test)]
mod tests {
    use super::{BarCommand, BarSection, MenuBarModel, opens};
    use crate::components::menus::item::item::MenuItem;
    use ds_core::standard_action::StandardAction;
    use ds_core::vocab::{Availability, Shortcut};
    use ds_core::word::Word;

    fn bar() -> MenuBarModel<BarCommand> {
        MenuBarModel::standard("Notes", |command| command)
    }

    #[test]
    fn the_bar_has_the_six_menus_in_the_macs_order() {
        let sections: Vec<BarSection> = bar().menus().iter().map(|menu| menu.section).collect();
        assert_eq!(sections, BarSection::ALL);
        let bar = bar();
        let titles: Vec<&str> = bar.menus().iter().map(|menu| menu.title.as_str()).collect();
        assert_eq!(titles, ["Notes", "File", "Edit", "View", "Window", "Help"]);
    }

    #[test]
    fn a_standard_item_shows_its_standard_keys() {
        const CASES: &[(BarSection, &str, StandardAction)] = &[
            (BarSection::App, "Quit Notes", StandardAction::Quit),
            (BarSection::App, "Settings…", StandardAction::Settings),
            (BarSection::File, "Close Window", StandardAction::Close),
            (BarSection::Edit, "Paste", StandardAction::Paste),
            (
                BarSection::View,
                "Show Sidebar",
                StandardAction::ToggleSidebar,
            ),
        ];
        let bar = bar();
        for &(section, title, action) in CASES {
            let found = bar.menu(section).and_then(|menu| {
                menu.items.iter().find_map(|item| match item {
                    MenuItem::Item {
                        title: own,
                        key,
                        value,
                        ..
                    } if own == title => Some((key.clone(), *value)),
                    _ => None,
                })
            });
            assert_eq!(
                found,
                Some((
                    Some(Shortcut::standard(action)),
                    BarCommand::Standard(action)
                )),
                "{title}"
            );
        }
    }

    #[test]
    fn help_leads_with_its_search_and_command_question_mark_opens_it() {
        let bar = bar();
        let first = bar
            .menu(BarSection::Help)
            .and_then(|menu| menu.items.first().cloned());
        assert_eq!(
            first.map(|item| matches!(
                item,
                MenuItem::Item {
                    value: BarCommand::HelpSearch,
                    ..
                }
            )),
            Some(true)
        );
        assert_eq!(
            opens(&Shortcut::standard(StandardAction::Help)),
            Some(BarSection::Help)
        );
        assert_eq!(opens(&Shortcut::standard(StandardAction::Copy)), None);
    }

    #[test]
    fn an_apps_own_items_follow_the_standard_ones_after_a_rule() {
        let extra = vec![MenuItem::new(BarCommand::About, "Export…")];
        let bar = bar().with_items(BarSection::File, extra);
        let items = bar
            .menu(BarSection::File)
            .map(|menu| menu.items.clone())
            .unwrap_or_default();
        let at = items.len().saturating_sub(2);
        assert_eq!(items.get(at), Some(&MenuItem::Separator));
        assert!(matches!(items.last(), Some(MenuItem::Item { title, .. }) if title == "Export…"));
    }

    #[test]
    fn availability_is_asked_of_every_command() {
        let bar = bar().availability(|command| match command {
            BarCommand::Standard(StandardAction::Paste) => Availability::Disabled,
            _ => Availability::Enabled,
        });
        let paste = bar.menu(BarSection::Edit).and_then(|menu| {
            menu.items.iter().find_map(|item| match item {
                MenuItem::Item {
                    title,
                    availability,
                    ..
                } if title == "Paste" => Some(*availability),
                _ => None,
            })
        });
        assert_eq!(paste, Some(Availability::Disabled));
    }
}
