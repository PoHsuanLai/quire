//! The tiles a `CollectionView` mounts, and the ones that are leaving: the roster over the
//! window's keys, and what the view remembers between renders so a removed tile can be drawn once
//! more while it plays its exit.

use crate::components::lists::collection_view::item::CollectionItem;
use crate::components::lists::collection_view::size::CollectionSize;
use crate::components::lists::collection_view::tile;
use crate::components::lists::list::list::node_key;
use crate::components::lists::virtual_list::layout::Layout;
use crate::components::lists::virtual_list::model::{Change, departed};
use dioxus::prelude::*;
use ds_core::word::Word;
use ds_motion::presence::Presence;
use ds_motion::roster::RosterEntry;
use ds_motion::use_roster::Roster;
use std::collections::HashMap;
use std::hash::Hash;
use std::ops::Range;

/// What the view remembers between renders.
pub(crate) struct Memory<K> {
    /// The keys as of the last render.
    keys: Vec<K>,
    /// The index in `keys` of the first mounted tile.
    start: usize,
    /// The mounted tiles, in order, as they were drawn.
    shown: Vec<(K, Element)>,
    /// Tiles that left while mounted and are still playing their exit.
    exiting: Vec<(K, Element)>,
}

impl<K> Memory<K> {
    /// A view that has drawn nothing yet, over `keys`.
    pub(crate) fn new(keys: Vec<K>) -> Self {
        Memory {
            keys,
            start: 0,
            shown: Vec::new(),
            exiting: Vec::new(),
        }
    }
}

/// Where a drawn tile stands.
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub(crate) enum Stand {
    /// Listed: the tile at this index.
    Live(usize),
    /// No longer listed, playing this exit (a `data-exit` word).
    Leaving(&'static str),
}

/// One tile to draw: its node, its motion state and its content.
pub(crate) struct Shown {
    /// The element's stable key.
    pub(crate) node: String,
    /// Listed or leaving.
    pub(crate) stand: Stand,
    /// The roster's `data-presence` word.
    pub(crate) presence: &'static str,
    /// The thumbnail and caption.
    pub(crate) content: Element,
}

/// Start the exits of the mounted tiles that `keys` no longer lists, and remember the new keys.
/// A `Change::Replace` plays none: its tiles are gone at once. Call it before the roster is told
/// the window's keys.
pub(crate) fn reconcile<K: Clone + Eq + Hash + 'static>(
    state: &mut Memory<K>,
    keys: &[K],
    change: Change,
    layout: &Layout,
    roster: Roster<K>,
) {
    if state.keys == keys {
        return;
    }
    let gone: Vec<K> = match change {
        Change::Edit => {
            let before = state.start..state.start + state.shown.len();
            departed(&state.keys, before, keys, layout)
                .into_iter()
                .map(|departed| departed.key)
                .collect()
        }
        Change::Replace => Vec::new(),
    };
    for key in &gone {
        let content = state
            .shown
            .iter()
            .find(|(held, _)| held == key)
            .map_or_else(|| rsx! {}, |(_, content)| content.clone());
        state.exiting.push((key.clone(), content));
    }
    roster.leave_from_render(gone);
    state.keys = keys.to_vec();
}

/// The roster's tiles in order, each leaving tile where it stood. `window` is the range of
/// `items` the roster was told; what was drawn is recorded for the next render.
pub(crate) fn shown<K: Clone + Eq + Hash + 'static>(
    state: &mut Memory<K>,
    entries: &[RosterEntry<K>],
    items: &[CollectionItem<K>],
    window: Range<usize>,
    size: CollectionSize,
) -> Vec<Shown> {
    state.exiting.retain(|(key, _)| {
        entries
            .iter()
            .any(|entry| entry.key == *key && matches!(entry.presence, Presence::Leaving(_)))
    });
    let at_of: HashMap<&K, usize> = items[window.clone()]
        .iter()
        .enumerate()
        .map(|(offset, item)| (&item.key, window.start + offset))
        .collect();
    let mut drawn = Vec::with_capacity(entries.len());
    let mut kept = Vec::with_capacity(entries.len());
    for entry in entries {
        if let Presence::Leaving(exit) = entry.presence {
            if let Some((_, content)) = state.exiting.iter().find(|(key, _)| *key == entry.key) {
                drawn.push(Shown {
                    node: node_key(&entry.key),
                    stand: Stand::Leaving(exit.slug()),
                    presence: entry.presence.slug(),
                    content: content.clone(),
                });
            }
            continue;
        }
        let Some(at) = at_of.get(&entry.key).copied() else {
            continue;
        };
        let content = tile::content(&items[at], size);
        drawn.push(Shown {
            node: node_key(&entry.key),
            stand: Stand::Live(at),
            presence: entry.presence.slug(),
            content: content.clone(),
        });
        kept.push((entry.key.clone(), content));
    }
    state.start = window.start;
    state.shown = kept;
    drawn
}
