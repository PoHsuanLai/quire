//! What a context menu does with an item that cannot be picked.

use ds_core::word::Word;

/// How a `Context` menu treats a disabled item. Other placements always draw it dimmed.
#[derive(Debug, Clone, Copy, PartialEq, Eq, Hash, Default, Word)]
#[non_exhaustive]
pub enum ContextUnavailable {
    /// Leave it out, with any header or rule it empties (design/27: a context menu shows only
    /// what can be done).
    #[default]
    Hide,
    /// Keep it, drawn dimmed and skipped by the keys: a menu whose items are always the same
    /// set, where seeing why one is off (its tooltip) matters more than a shorter list.
    Dim,
}
