//! Item ids: a `com.canonical.dbusmenu` id is an `i32`, and a client keeps it between a layout
//! and the click that names it. Ours is a hash of the item's key (its `CommandId`, or its place
//! for a menu, a submenu, a rule), so it is the same in every run and across a rebuilt tree.

use serde::{Deserialize, Serialize};
use std::collections::HashSet;

/// A menu item's id. The root (the bar itself) is 0; every other id is positive.
#[derive(Debug, Clone, Copy, PartialEq, Eq, Hash, PartialOrd, Ord, Serialize, Deserialize)]
#[serde(transparent)]
pub struct NodeId(pub i32);

impl NodeId {
    /// The bar: the layout's root, which no item has.
    pub const ROOT: NodeId = NodeId(0);

    /// The id `key` hashes to (FNV-1a, 31 bits, never 0), before any collision is resolved.
    fn hashed(key: &str) -> NodeId {
        let hash = key.bytes().fold(0x811c_9dc5_u32, |hash, byte| {
            (hash ^ u32::from(byte)).wrapping_mul(0x0100_0193)
        });
        let positive = i32::try_from(hash & 0x7fff_ffff).unwrap_or(1);
        NodeId(positive.max(1))
    }
}

/// The ids handed out so far in one tree.
#[derive(Debug, Default)]
pub(super) struct Minter {
    used: HashSet<NodeId>,
}

impl Minter {
    /// The id of `key`. A key hashing to an id already given (the same command in two menus, or a
    /// collision) is hashed again with its attempt number, so ids are unique and still depend
    /// only on the tree's order.
    pub(super) fn mint(&mut self, key: &str) -> NodeId {
        let id = (0u32..)
            .map(|attempt| match attempt {
                0 => NodeId::hashed(key),
                n => NodeId::hashed(&format!("{key}#{n}")),
            })
            .find(|id| !self.used.contains(id))
            .unwrap_or(NodeId(1));
        self.used.insert(id);
        id
    }
}
