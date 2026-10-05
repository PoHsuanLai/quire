//! The menu tree: an app's menu bar as serialisable data. Every command item carries the
//! [`CommandId`] it runs, so a process that lists the tree (the shell's bar, an agent) can run
//! any item by the id the app's own tables, its UI manifest and its intents already use.

use super::chord::Chord;
use super::ids::{Minter, NodeId};
use crate::components::menus::item::item::MenuItem;
use crate::components::menus::menu_bar::{BarSection, MenuBarModel};
use ds_core::command::{AppCommand, CommandId};
use ds_core::vocab::{Availability, Check};
use ds_core::word::Word;
use serde::{Deserialize, Serialize};

/// Why a bar cannot be exported.
#[derive(Debug, Clone, PartialEq, Eq, thiserror::Error)]
pub enum TreeError {
    /// A menu item whose command has an empty id: nothing could run it from outside.
    #[error("the menu item {title:?} has no command id")]
    EmptyCommandId {
        /// The item's title.
        title: String,
    },
}

/// What a node is.
#[derive(Debug, Clone, PartialEq, Eq, Serialize, Deserialize)]
#[serde(tag = "kind", rename_all = "snake_case")]
pub enum NodeKind {
    /// An item that runs a command.
    Command {
        /// What it runs.
        command: CommandId,
        /// Its key equivalent.
        shortcut: Option<Chord>,
        /// Its state mark, for an item that is on, off or mixed.
        check: Option<Check>,
        /// Whether it can be picked.
        availability: Availability,
    },
    /// A menu of the bar, or an item that opens a submenu.
    Submenu {
        /// Which of the bar's six it is; `None` for a nested submenu.
        section: Option<BarSection>,
        /// Whether it opens.
        availability: Availability,
        /// Its rows.
        children: Vec<MenuNode>,
    },
    /// A rule between groups.
    Separator,
    /// A group title or status line: text that is never a choice.
    Label,
}

/// One node of the tree: its id on the wire, its words and what it is.
#[derive(Debug, Clone, PartialEq, Eq, Serialize, Deserialize)]
pub struct MenuNode {
    /// The id a `com.canonical.dbusmenu` client names it by.
    pub id: NodeId,
    /// What it says (empty for a rule).
    pub label: String,
    /// What it is.
    #[serde(flatten)]
    pub kind: NodeKind,
}

/// An app's menu bar: its menus, in the bar's order, each a [`NodeKind::Submenu`].
#[derive(Debug, Clone, PartialEq, Eq, Default, Serialize, Deserialize)]
pub struct MenuTree {
    /// The bar's menus.
    pub menus: Vec<MenuNode>,
}

impl MenuTree {
    /// The tree of `bar`. Ids follow the bar's order and the commands' own ids, so the same bar
    /// is the same tree.
    pub fn from_bar<T: AppCommand + Clone>(bar: &MenuBarModel<T>) -> Result<MenuTree, TreeError> {
        let mut minter = Minter::default();
        let menus = bar
            .menus()
            .iter()
            .map(|menu| {
                let key = format!("menu:{}", menu.section.slug());
                let id = minter.mint(&key);
                let children = nodes(&mut minter, &key, &menu.items)?;
                Ok(MenuNode {
                    id,
                    label: menu.title.clone(),
                    kind: NodeKind::Submenu {
                        section: Some(menu.section),
                        availability: Availability::Enabled,
                        children,
                    },
                })
            })
            .collect::<Result<Vec<_>, TreeError>>()?;
        Ok(MenuTree { menus })
    }

    /// Every node, parents before their rows, in the order the bar draws them.
    pub fn walk(&self) -> Vec<(&MenuNode, usize)> {
        fn visit<'a>(node: &'a MenuNode, depth: usize, out: &mut Vec<(&'a MenuNode, usize)>) {
            out.push((node, depth));
            if let NodeKind::Submenu { children, .. } = &node.kind {
                children
                    .iter()
                    .for_each(|child| visit(child, depth + 1, out));
            }
        }
        let mut out = Vec::new();
        self.menus.iter().for_each(|menu| visit(menu, 0, &mut out));
        out
    }

    /// The node with `id`.
    pub fn node(&self, id: NodeId) -> Option<&MenuNode> {
        self.walk()
            .into_iter()
            .find_map(|(node, _)| (node.id == id).then_some(node))
    }

    /// The id of the first item that runs `command`.
    pub fn id_of(&self, command: &CommandId) -> Option<NodeId> {
        self.walk()
            .into_iter()
            .find_map(|(node, _)| match &node.kind {
                NodeKind::Command { command: own, .. } if own == command => Some(node.id),
                _ => None,
            })
    }

    /// The command item `id` runs, if it is a command item.
    pub fn command_of(&self, id: NodeId) -> Option<&CommandId> {
        match self.node(id).map(|node| &node.kind) {
            Some(NodeKind::Command { command, .. }) => Some(command),
            _ => None,
        }
    }

    /// The menu of `section`.
    pub fn menu(&self, section: BarSection) -> Option<&MenuNode> {
        self.menus.iter().find(|menu| {
            matches!(&menu.kind, NodeKind::Submenu { section: Some(own), .. } if *own == section)
        })
    }
}

impl MenuNode {
    /// The rows of a submenu; none for any other node.
    pub fn children(&self) -> &[MenuNode] {
        match &self.kind {
            NodeKind::Submenu { children, .. } => children,
            _ => &[],
        }
    }
}

/// The nodes of `items`, their keys built from `parent`'s.
fn nodes<T: AppCommand + Clone>(
    minter: &mut Minter,
    parent: &str,
    items: &[MenuItem<T>],
) -> Result<Vec<MenuNode>, TreeError> {
    items
        .iter()
        .enumerate()
        .map(|(at, item)| node(minter, parent, at, item))
        .collect()
}

fn node<T: AppCommand + Clone>(
    minter: &mut Minter,
    parent: &str,
    at: usize,
    item: &MenuItem<T>,
) -> Result<MenuNode, TreeError> {
    Ok(match item {
        MenuItem::Item {
            value,
            title,
            key,
            check,
            availability,
            ..
        } => {
            let command = value.id();
            if command.0.trim().is_empty() {
                return Err(TreeError::EmptyCommandId {
                    title: title.clone(),
                });
            }
            MenuNode {
                id: minter.mint(&command.0),
                label: title.clone(),
                kind: NodeKind::Command {
                    command,
                    shortcut: key.as_ref().map(Chord::of),
                    check: *check,
                    availability: *availability,
                },
            }
        }
        MenuItem::Submenu {
            title,
            availability,
            children,
            ..
        } => {
            let key = format!("{parent}/{title}");
            let id = minter.mint(&key);
            MenuNode {
                id,
                label: title.clone(),
                kind: NodeKind::Submenu {
                    section: None,
                    availability: *availability,
                    children: nodes(minter, &key, children)?,
                },
            }
        }
        MenuItem::Header(title) | MenuItem::Info { title, .. } => MenuNode {
            id: minter.mint(&format!("{parent}/#{at}")),
            label: title.clone(),
            kind: NodeKind::Label,
        },
        MenuItem::Separator => MenuNode {
            id: minter.mint(&format!("{parent}/-{at}")),
            label: String::new(),
            kind: NodeKind::Separator,
        },
    })
}
