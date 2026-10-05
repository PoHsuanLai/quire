//! The tree as `com.canonical.dbusmenu` (version 3) says it: the `(ia{sv}av)` layout of
//! `GetLayout`, the property rows of `GetGroupProperties`, and an `Event` mapped back to the
//! command it runs. Still plain data: ds-blitz turns these into D-Bus values.
//!
//! One property is ours: `x-quire-command-id`, the [`CommandId`] a command item runs, so an
//! agent can list the menu and run an item by the id the app's manifest names it by. Clients
//! ignore properties they do not know.

use super::chord::Chord;
use super::ids::NodeId;
use super::tree::{MenuNode, MenuTree, NodeKind};
use ds_core::command::CommandId;
use ds_core::vocab::{Availability, Check};

/// `type`: how an item is drawn.
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum ItemType {
    /// A row.
    Standard,
    /// A rule.
    Separator,
}

/// `toggle-type`: the mark an item can carry.
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum ToggleType {
    /// A check mark.
    Checkmark,
}

/// `toggle-state`: whether the mark is shown.
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum ToggleState {
    /// Off.
    Off,
    /// On.
    On,
    /// Neither: some of what it covers is on.
    Indeterminate,
}

impl ToggleState {
    /// The `i` the property is written as.
    pub fn wire(self) -> i32 {
        match self {
            ToggleState::Off => 0,
            ToggleState::On => 1,
            ToggleState::Indeterminate => -1,
        }
    }
}

/// `children-display`: that an item opens a submenu.
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum ChildrenDisplay {
    /// It opens one.
    Submenu,
}

/// One property of an item, with its value.
#[derive(Debug, Clone, PartialEq, Eq)]
pub enum Prop {
    /// `type`.
    Type(ItemType),
    /// `label`, mnemonics escaped (a literal `_` is `__`; ours has no accelerator).
    Label(String),
    /// `enabled`: only [`Availability::Enabled`] is true.
    Enabled(Availability),
    /// `toggle-type`.
    ToggleType(ToggleType),
    /// `toggle-state`.
    ToggleState(ToggleState),
    /// `shortcut`: one chord.
    Shortcut(Chord),
    /// `children-display`.
    ChildrenDisplay(ChildrenDisplay),
    /// `x-quire-command-id`.
    CommandId(CommandId),
}

impl Prop {
    /// The property's name on the wire.
    pub fn name(&self) -> &'static str {
        match self {
            Prop::Type(_) => "type",
            Prop::Label(_) => "label",
            Prop::Enabled(_) => "enabled",
            Prop::ToggleType(_) => "toggle-type",
            Prop::ToggleState(_) => "toggle-state",
            Prop::Shortcut(_) => "shortcut",
            Prop::ChildrenDisplay(_) => "children-display",
            Prop::CommandId(_) => "x-quire-command-id",
        }
    }
}

/// Which properties a call asks for; none named means all.
#[derive(Debug, Clone, PartialEq, Eq, Default)]
pub struct PropNames(pub Vec<String>);

impl PropNames {
    fn keeps(&self, prop: &Prop) -> bool {
        self.0.is_empty() || self.0.iter().any(|name| name == prop.name())
    }
}

/// An item and its properties: a row of `GetGroupProperties`, or a layout node's head.
#[derive(Debug, Clone, PartialEq, Eq)]
pub struct DbusItem {
    /// The item's id.
    pub id: NodeId,
    /// Its properties.
    pub props: Vec<Prop>,
}

/// A layout node: `(ia{sv}av)`.
#[derive(Debug, Clone, PartialEq, Eq)]
pub struct DbusLayout {
    /// The item and its properties.
    pub item: DbusItem,
    /// Its rows, as deep as the call asked.
    pub children: Vec<DbusLayout>,
}

/// How deep `GetLayout` goes below the parent.
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum Depth {
    /// To the leaves (`-1`).
    All,
    /// This many levels: 0 is the parent alone.
    Levels(u32),
}

impl Depth {
    /// The depth `recursionDepth` means.
    pub fn from_wire(depth: i32) -> Depth {
        u32::try_from(depth).map_or(Depth::All, Depth::Levels)
    }

    fn below(self) -> Option<Depth> {
        match self {
            Depth::All => Some(Depth::All),
            Depth::Levels(0) => None,
            Depth::Levels(n) => Some(Depth::Levels(n - 1)),
        }
    }
}

/// What an `Event` says happened.
#[derive(Debug, Clone, PartialEq, Eq)]
pub enum EventKind {
    /// The item was picked.
    Clicked,
    /// Its submenu opened, closed or was hovered; no command.
    Seen,
    /// An event we do not know.
    Other(String),
}

impl EventKind {
    /// The kind `eventId` names.
    pub fn parse(event_id: &str) -> EventKind {
        match event_id {
            "clicked" => EventKind::Clicked,
            "opened" | "closed" | "hovered" => EventKind::Seen,
            other => EventKind::Other(other.to_owned()),
        }
    }
}

/// What an event asks the app to do.
#[derive(Debug, Clone, PartialEq, Eq)]
pub enum Dispatch {
    /// Run this command, as the app's own menu or shortcut would.
    Run(CommandId),
    /// Nothing.
    Nothing,
}

/// Why an event is refused.
#[derive(Debug, Clone, PartialEq, Eq, thiserror::Error)]
pub enum EventError {
    /// No item has this id.
    #[error("no menu item has id {0}")]
    UnknownItem(i32),
    /// The item is disabled, so it cannot be picked.
    #[error("the menu item for {0:?} is disabled")]
    Disabled(CommandId),
    /// The event is not one of the protocol's.
    #[error("unknown menu event {0:?}")]
    UnknownEvent(String),
}

/// A label with its underscores doubled, so a client reads none of them as a mnemonic.
fn escape(label: &str) -> String {
    label.replace('_', "__")
}

fn root_props() -> Vec<Prop> {
    vec![Prop::ChildrenDisplay(ChildrenDisplay::Submenu)]
}

fn toggle(check: Check) -> ToggleState {
    match check {
        Check::On => ToggleState::On,
        Check::Off => ToggleState::Off,
        Check::Mixed => ToggleState::Indeterminate,
    }
}

/// Every property of `node`, in a fixed order. `visible` is left at its default (true).
fn props_of(node: &MenuNode) -> Vec<Prop> {
    match &node.kind {
        NodeKind::Command {
            command,
            shortcut,
            check,
            availability,
        } => [
            Some(Prop::Label(escape(&node.label))),
            Some(Prop::Enabled(*availability)),
            check.map(|_| Prop::ToggleType(ToggleType::Checkmark)),
            check.map(|check| Prop::ToggleState(toggle(check))),
            shortcut.clone().map(Prop::Shortcut),
            Some(Prop::CommandId(command.clone())),
        ]
        .into_iter()
        .flatten()
        .collect(),
        NodeKind::Submenu { availability, .. } => vec![
            Prop::Label(escape(&node.label)),
            Prop::Enabled(*availability),
            Prop::ChildrenDisplay(ChildrenDisplay::Submenu),
        ],
        NodeKind::Separator => vec![Prop::Type(ItemType::Separator)],
        NodeKind::Label => vec![
            Prop::Label(escape(&node.label)),
            Prop::Enabled(Availability::Disabled),
        ],
    }
}

impl MenuTree {
    fn item(&self, id: NodeId, names: &PropNames) -> Option<DbusItem> {
        let props = match id {
            NodeId::ROOT => root_props(),
            _ => props_of(self.node(id)?),
        };
        Some(DbusItem {
            id,
            props: props.into_iter().filter(|prop| names.keeps(prop)).collect(),
        })
    }

    /// `GetLayout(parent, depth, names)`: `parent` and the rows below it, or `None` for an id no
    /// item has. [`NodeId::ROOT`] is the bar, whose rows are the menus.
    pub fn layout(&self, parent: NodeId, depth: Depth, names: &PropNames) -> Option<DbusLayout> {
        let item = self.item(parent, names)?;
        let rows: &[MenuNode] = match parent {
            NodeId::ROOT => &self.menus,
            _ => self.node(parent)?.children(),
        };
        let children = depth
            .below()
            .map(|deeper| {
                rows.iter()
                    .filter_map(|row| self.layout(row.id, deeper, names))
                    .collect()
            })
            .unwrap_or_default();
        Some(DbusLayout { item, children })
    }

    /// `GetGroupProperties(ids, names)`: the items of those ids that exist; every item when
    /// `ids` is empty.
    pub fn group_properties(&self, ids: &[NodeId], names: &PropNames) -> Vec<DbusItem> {
        let wanted: Vec<NodeId> = match ids {
            [] => self.walk().into_iter().map(|(node, _)| node.id).collect(),
            some => some.to_vec(),
        };
        wanted
            .into_iter()
            .filter_map(|id| self.item(id, names))
            .collect()
    }

    /// `GetProperty(id, name)`.
    pub fn property(&self, id: NodeId, name: &str) -> Option<Prop> {
        self.item(id, &PropNames(vec![name.to_owned()]))?
            .props
            .into_iter()
            .next()
    }

    /// What `Event(id, event_id)` asks of the app: a click on an enabled command item runs its
    /// command; opening, closing and hovering do nothing; a click on a submenu does nothing.
    pub fn event(&self, id: NodeId, event_id: &str) -> Result<Dispatch, EventError> {
        let kind = EventKind::parse(event_id);
        let node = match id {
            NodeId::ROOT => None,
            _ => Some(self.node(id).ok_or(EventError::UnknownItem(id.0))?),
        };
        match (kind, node.map(|node| &node.kind)) {
            (EventKind::Other(other), _) => Err(EventError::UnknownEvent(other)),
            (
                EventKind::Clicked,
                Some(NodeKind::Command {
                    command,
                    availability: Availability::Enabled,
                    ..
                }),
            ) => Ok(Dispatch::Run(command.clone())),
            (EventKind::Clicked, Some(NodeKind::Command { command, .. })) => {
                Err(EventError::Disabled(command.clone()))
            }
            _ => Ok(Dispatch::Nothing),
        }
    }
}
