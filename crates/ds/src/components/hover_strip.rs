//! HoverStrip: a pill of icon buttons that appears on a hovered row, each previewing its
//! result through a Fly tooltip (design/04-COMPONENTS.md section 17).

use crate::components::vocab::Here;
use crate::geometry::Rect;
use crate::icon::Icon;
use dioxus::prelude::*;

/// Which action a strip button is, by the consumer's own name: `archive`, `snooze`.
#[derive(Debug, Clone, PartialEq, Eq, Hash, PartialOrd, Ord)]
pub struct ActionId(pub String);

/// One strip button.
#[derive(Debug, Clone, PartialEq)]
pub struct StripAction {
    /// Which action, written as `data-op`.
    pub id: ActionId,
    /// Its glyph.
    pub icon: Icon,
    /// Its `aria-label`.
    pub label: String,
    /// What the Fly preview says: "Archive → out of Inbox".
    pub fly: String,
    /// Destination preview: `Current` while hovered, `Elsewhere` on leave.
    pub onhover: Option<EventHandler<Here>>,
    /// Pressed; the button's rect anchors the snooze and label menus.
    pub onclick: EventHandler<Rect>,
}

/// A row's action strip.
#[component]
pub fn HoverStrip(actions: Vec<StripAction>) -> Element {
    todo!()
}
