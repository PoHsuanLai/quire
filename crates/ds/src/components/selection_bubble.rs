//! SelectionBubble: the inline toolbar over a text selection (design/04-COMPONENTS.md
//! section 30). The composer supplies the selection rect and the marks.

use crate::components::vocab::Switch;
use crate::geometry::Rect;
use dioxus::prelude::*;

/// One bubble button.
#[derive(Debug, Clone, PartialEq)]
pub struct BubbleAction {
    /// Its face: `B`, `i`, a glyph.
    pub label: Element,
    /// Its title, with the shortcut.
    pub title: String,
    /// Whether the mark is active.
    pub pressed: Option<Switch>,
    /// Pressed.
    pub onclick: EventHandler<()>,
}

/// What the bubble shows.
#[derive(Debug, Clone, PartialEq)]
pub enum BubbleMode {
    /// Formatting buttons.
    Actions(Vec<BubbleAction>),
    /// A link field.
    Link,
}

/// The toolbar over a selection.
#[component]
pub fn SelectionBubble(
    anchor: Rect,
    mode: BubbleMode,
    onlink: EventHandler<String>,
    onclose: EventHandler<()>,
) -> Element {
    todo!()
}
