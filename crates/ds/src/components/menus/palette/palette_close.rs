//! Why a command palette asked to close, so a consumer can treat Escape, a click outside and a
//! pick differently (a bar that Escape closes whole and an outside click only dismisses).

/// What made the palette call `onclose`.
#[derive(Debug, Clone, Copy, PartialEq, Eq, Hash)]
#[non_exhaustive]
pub enum PaletteClose {
    /// Escape, while the palette was the topmost layer.
    Escape,
    /// A pointer down outside the card, while the palette was the topmost layer.
    Outside,
    /// A row was picked and its pick closes the palette; `onpick` follows with the value.
    Pick,
}
