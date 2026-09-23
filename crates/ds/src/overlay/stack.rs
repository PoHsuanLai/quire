//! Which floating layer Escape and an outside click close: the topmost only
//! (design/04-COMPONENTS.md section 21, design/06-INTERACTIONS.md sections 5 and 18).
#![allow(unused_variables)] // Freeze stubs: remove with the last todo!().

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

/// The open layers, bottom first.
#[derive(Debug, Clone, PartialEq, Eq, Default)]
pub struct LayerStack {
    layers: Vec<(LayerId, Dismiss)>,
}

impl LayerStack {
    /// Open `id` on top, dismissed as `dismiss` says.
    pub fn push(self, id: LayerId, dismiss: Dismiss) -> Self {
        todo!()
    }

    /// `id` closed.
    pub fn remove(self, id: LayerId) -> Self {
        todo!()
    }

    /// Escape was pressed.
    pub fn escape(&self) -> Dismissal {
        todo!()
    }

    /// A click landed outside every layer.
    pub fn outside_click(&self) -> Dismissal {
        todo!()
    }
}
