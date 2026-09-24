//! The overlay registry and the host that renders it at the end of `.ds`.

use crate::tokens::ZLayer;
use dioxus::prelude::*;

/// Which overlay an entry is, so its owner can replace or remove it.
#[derive(Debug, Clone, Copy, PartialEq, Eq, Hash, PartialOrd, Ord)]
pub struct OverlayId(pub u32);

/// The overlays currently shown, provided as context by `Ds`.
#[derive(Debug, Clone, Copy, PartialEq)]
pub struct Overlays {
    entries: Signal<Vec<(OverlayId, ZLayer, Element)>>,
}

impl Overlays {
    /// Show `content` on `layer`, replacing whatever `id` showed before in its place: a
    /// surface keeps its stacking among its layer while it re-renders, and a surface shown
    /// later on the same layer (a submenu over its menu) stays above it.
    ///
    /// Call from a handler or an effect, not from render: the registry is a signal the host
    /// reads.
    pub fn show(&self, id: OverlayId, layer: ZLayer, content: Element) {
        let mut entries = self.entries;
        entries.with_mut(|entries| {
            if let Some(entry) = entries
                .iter_mut()
                .find(|(shown, at, _)| *shown == id && *at == layer)
            {
                entry.2 = content;
                return;
            }
            entries.retain(|(shown, _, _)| *shown != id);
            let at =
                entries.partition_point(|(_, shown, _)| stack_order(*shown) <= stack_order(layer));
            entries.insert(at, (id, layer, content));
        });
    }

    /// Stop showing `id`.
    pub fn hide(&self, id: OverlayId) {
        let mut entries = self.entries;
        entries.with_mut(|entries| entries.retain(|(shown, _, _)| *shown != id));
    }
}

/// A layer's place in the window's stacking order (design/01-LAYOUT.md section 12), lowest
/// first: its position in [`ZLayer::ALL`].
fn stack_order(layer: ZLayer) -> usize {
    ZLayer::ALL
        .iter()
        .position(|&l| l == layer)
        .unwrap_or(ZLayer::ALL.len())
}

/// A new, empty registry, owned by the calling scope (`Ds`).
pub(crate) fn use_overlays_provider() -> Overlays {
    use_context_provider(|| Overlays {
        entries: Signal::new(Vec::new()),
    })
}

/// The enclosing `Ds`'s overlay registry.
pub fn use_overlays() -> Overlays {
    use_context::<Overlays>()
}

/// Renders every registered overlay, lowest layer first. `Ds` places it last in `.ds`.
#[component]
pub fn OverlayHost() -> Element {
    let overlays = use_overlays();
    let entries = overlays.entries.read();
    rsx! {
        for (id , _ , content) in entries.iter() {
            Fragment { key: "{id.0}", {content.clone()} }
        }
    }
}
