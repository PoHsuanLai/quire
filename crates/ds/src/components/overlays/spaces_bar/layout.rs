//! The rectangles the bar's elements are drawn at, measured, for a host that places something
//! against them (the compositor's drop targets over the cards): the host's frame phase keeps each
//! element's rect ([`use_rect`]), the bar gathers them into a [`BarLayout`] and reports it to
//! `on_layout` whenever any of them changes.

use super::model::BarSpaceKey;
use crate::host::measure::{RectProbe, use_rect};
use dioxus::prelude::*;
use ds_core::geometry::units::Rect;
use std::collections::HashMap;

/// Where one Space is drawn.
#[derive(Debug, Clone, PartialEq)]
pub struct SpaceRects {
    /// Which Space.
    pub key: BarSpaceKey,
    /// Its button, in window coordinates and logical pixels.
    pub tile: Rect,
    /// Its thumbnail card; `None` while the bar is collapsed and draws no card.
    pub card: Option<Rect>,
}

/// Where the bar's parts are drawn, in window coordinates and logical pixels. A part not yet laid
/// out is left out, and nothing is reported until at least one is.
#[derive(Debug, Clone, PartialEq, Default)]
pub struct BarLayout {
    /// The Spaces, in the order of `spaces`.
    pub spaces: Vec<SpaceRects>,
    /// The plus, when it is drawn.
    pub add: Option<Rect>,
}

/// One measured part of the bar.
#[derive(Debug, Clone, PartialEq, Eq, Hash)]
enum Part {
    Tile(BarSpaceKey),
    Card(BarSpaceKey),
    Add,
}

/// The rects the bar's parts have published, shared with them through context.
#[derive(Clone, Copy)]
pub(super) struct Book(Signal<HashMap<Part, Rect>>);

impl Book {
    /// The book of the bar being built.
    pub(super) fn provide() -> Book {
        use_context_provider(|| Book(Signal::new(HashMap::new())))
    }

    /// What is published, for the Spaces named by `keys`, in their order.
    fn layout(self, keys: &[BarSpaceKey]) -> BarLayout {
        let book = self.0.read();
        let spaces = keys
            .iter()
            .filter_map(|key| {
                Some(SpaceRects {
                    key: key.clone(),
                    tile: *book.get(&Part::Tile(key.clone()))?,
                    card: book.get(&Part::Card(key.clone())).copied(),
                })
            })
            .collect();
        BarLayout {
            spaces,
            add: book.get(&Part::Add).copied(),
        }
    }
}

/// The probe for the plus.
pub(super) fn use_add_rect() -> RectProbe {
    use_part(Part::Add)
}

/// The probe for a Space's button.
pub(super) fn use_tile_rect(key: &BarSpaceKey) -> RectProbe {
    use_part(Part::Tile(key.clone()))
}

/// The probe for a Space's thumbnail card; it forgets the rect when the card leaves.
pub(super) fn use_card_rect(key: &BarSpaceKey) -> RectProbe {
    use_part(Part::Card(key.clone()))
}

/// A probe whose rect is published to the bar's book, and withdrawn when the part goes.
fn use_part(part: Part) -> RectProbe {
    let probe = use_rect();
    let book = use_context::<Book>();
    let mine = part.clone();
    use_effect(move || {
        let rect = probe.rect();
        let mut map = book.0;
        if map.peek().get(&mine).copied() != rect {
            map.with_mut(|map| match rect {
                Some(rect) => drop(map.insert(mine.clone(), rect)),
                None => drop(map.remove(&mine)),
            });
        }
    });
    use_drop(move || {
        if let Ok(mut map) = book.0.try_write() {
            map.remove(&part);
        }
    });
    probe
}

/// Draws nothing; reports the bar's layout to `on_layout` each time it changes.
#[component]
pub(super) fn Reporter(keys: Vec<BarSpaceKey>, on_layout: EventHandler<BarLayout>) -> Element {
    let layout = use_context::<Book>().layout(&keys);
    use_effect(use_reactive!(|layout| {
        if !layout.spaces.is_empty() || layout.add.is_some() {
            on_layout.call(layout);
        }
    }));
    rsx! {}
}
