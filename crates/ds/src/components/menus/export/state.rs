//! What the exporter keeps between two trees: the current tree and its revision. A client that
//! has fetched a layout asks again only when `LayoutUpdated` names a newer revision, so the
//! revision rises exactly when the tree changed, and [`Change`] says what to announce.

use super::ids::NodeId;
use super::tree::{MenuNode, MenuTree};

/// A layout's revision (`GetLayout`'s `u`): it rises by one on every change.
#[derive(Debug, Clone, Copy, PartialEq, Eq, PartialOrd, Ord)]
pub struct Revision(pub u32);

/// How a new tree differs from the one before it.
#[derive(Debug, Clone, PartialEq, Eq)]
pub enum Change {
    /// It does not: announce nothing.
    Same,
    /// The same items in the same places, these with other properties (availability, a check, a
    /// label): `ItemsPropertiesUpdated` for them.
    Properties(Vec<NodeId>),
    /// Items were added, removed or moved: `LayoutUpdated` for the whole bar.
    Layout,
}

/// The tree being served and its revision.
#[derive(Debug, Clone, PartialEq, Eq)]
pub struct MenuState {
    /// The tree.
    pub tree: MenuTree,
    /// Its revision, 1 at the start.
    pub revision: Revision,
}

/// Each node's id and depth, in drawing order: the tree's shape.
fn shape(tree: &MenuTree) -> Vec<(NodeId, usize)> {
    tree.walk()
        .into_iter()
        .map(|(node, depth)| (node.id, depth))
        .collect()
}

fn nodes(tree: &MenuTree) -> Vec<&MenuNode> {
    tree.walk().into_iter().map(|(node, _)| node).collect()
}

/// How `new` differs from `old`.
pub fn change(old: &MenuTree, new: &MenuTree) -> Change {
    if shape(old) != shape(new) {
        return Change::Layout;
    }
    let changed: Vec<NodeId> = nodes(old)
        .into_iter()
        .zip(nodes(new))
        .filter(|(before, after)| flat(before) != flat(after))
        .map(|(_, after)| after.id)
        .collect();
    match changed.as_slice() {
        [] => Change::Same,
        _ => Change::Properties(changed),
    }
}

/// A node without its rows: what its own properties say.
fn flat(node: &MenuNode) -> MenuNode {
    use super::tree::NodeKind;
    MenuNode {
        kind: match &node.kind {
            NodeKind::Submenu {
                section,
                availability,
                ..
            } => NodeKind::Submenu {
                section: *section,
                availability: *availability,
                children: Vec::new(),
            },
            other => other.clone(),
        },
        ..node.clone()
    }
}

impl MenuState {
    /// A state serving `tree` at revision 1.
    pub fn new(tree: MenuTree) -> MenuState {
        MenuState {
            tree,
            revision: Revision(1),
        }
    }

    /// The state after `tree` replaces this one, and what changed. A tree that is the same
    /// keeps the revision.
    pub fn replaced(self, tree: MenuTree) -> (MenuState, Change) {
        let changed = change(&self.tree, &tree);
        let revision = match changed {
            Change::Same => self.revision,
            _ => Revision(self.revision.0.saturating_add(1)),
        };
        (MenuState { tree, revision }, changed)
    }
}
