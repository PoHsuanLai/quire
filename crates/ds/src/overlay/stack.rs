//! Which floating layer Escape and an outside click close: the topmost only
//! (design/04-COMPONENTS.md section 21, design/06-INTERACTIONS.md sections 5 and 18).

use crate::components::popover::Dismiss;

/// One open layer.
#[derive(Debug, Clone, Copy, PartialEq, Eq, Hash, PartialOrd, Ord)]
pub struct LayerId(pub u32);

/// What an Escape or an outside click did.
#[derive(Debug, Clone, Copy, PartialEq, Eq, Hash)]
pub enum Dismissal {
    /// Close this layer.
    Close(LayerId),
    /// No layer takes that dismissal; let it through.
    PassThrough,
}

/// The open layers, bottom first. `Ds` provides one as a `Signal<LayerStack>` context; a
/// floating component pushes itself when it opens and removes itself when it closes.
#[derive(Debug, Clone, PartialEq, Eq, Default)]
pub struct LayerStack {
    layers: Vec<(LayerId, Dismiss)>,
}

impl LayerStack {
    /// Open `id` on top, dismissed as `dismiss` says.
    ///
    /// A layer already open moves to the top.
    pub fn push(self, id: LayerId, dismiss: Dismiss) -> Self {
        let mut layers = self.remove(id).layers;
        layers.push((id, dismiss));
        LayerStack { layers }
    }

    /// `id` closed.
    pub fn remove(self, id: LayerId) -> Self {
        LayerStack {
            layers: self
                .layers
                .into_iter()
                .filter(|&(open, _)| open != id)
                .collect(),
        }
    }

    /// Escape was pressed.
    ///
    /// One Escape, one layer: the topmost closes if it takes Escape. If it does not (its
    /// owner closes it), the key passes through to that owner and nothing below closes
    /// (design/06-INTERACTIONS.md section 18).
    pub fn escape(&self) -> Dismissal {
        self.top_if(|dismiss| matches!(dismiss, Dismiss::EscAndOutside | Dismiss::EscOnly))
    }

    /// A click landed outside every layer.
    ///
    /// Only the topmost layer is asked; a click outside it that lands on a lower layer is
    /// that layer's own click.
    pub fn outside_click(&self) -> Dismissal {
        self.top_if(|dismiss| dismiss == Dismiss::EscAndOutside)
    }

    /// The topmost open layer.
    pub fn top(&self) -> Option<LayerId> {
        self.layers.last().map(|&(id, _)| id)
    }

    fn top_if(&self, takes: impl Fn(Dismiss) -> bool) -> Dismissal {
        match self.layers.last() {
            Some(&(id, dismiss)) if takes(dismiss) => Dismissal::Close(id),
            _ => Dismissal::PassThrough,
        }
    }
}
