//! An app's menu bar as data a process outside it can read and drive: the serialisable tree
//! ([`MenuTree`]) built from the app's [`MenuBarModel`](crate::components::menus::menu_bar::MenuBarModel),
//! its conversion to the `com.canonical.dbusmenu` layout and properties ([`dbusmenu`]), and the
//! revision book the exporter keeps ([`MenuState`]). Pure data: ds-blitz's `menus` feature puts
//! it on the session bus (design/27 section 5.2 rule a).

mod address;
mod chord;
pub mod dbusmenu;
mod ids;
mod resolve;
mod state;
mod tree;

pub use address::{AddressError, AppMenuAddress};
pub use chord::{Chord, ChordError};
pub use dbusmenu::{
    ChildrenDisplay, DbusItem, DbusLayout, Depth, Dispatch, EventError, EventKind, ItemType, Prop,
    PropNames, ToggleState, ToggleType,
};
pub use ids::NodeId;
pub use state::{Change, MenuState, Revision};
pub use tree::{MenuNode, MenuTree, NodeKind, TreeError};

#[cfg(test)]
mod tests;
#[cfg(test)]
mod tests_names;
