//! SidebarItem: a navigable place on the frame, with the seal, gulp and destination preview
//! (design/04-COMPONENTS.md section 19).

use crate::components::avatar::AvatarFace;
use crate::components::vocab::{Here, PulseKey};
use crate::icon::Icon;
use crate::motion::presence::Presence;
use dioxus::prelude::*;

/// What kind of place.
#[derive(Debug, Clone, PartialEq)]
pub enum ItemKind {
    /// Inbox, Starred, a label: carries the seal when current.
    Place {
        /// Its glyph.
        icon: Icon,
    },
    /// A pinned person or saved search, with a favicon.
    Pinned {
        /// The favicon.
        avatar: AvatarFace,
    },
    /// An opened thread or draft: slides in, has a close button.
    Today {
        /// The favicon.
        avatar: AvatarFace,
    },
}

/// A preview the item is showing.
#[derive(Debug, Clone, Copy, PartialEq, Eq, Hash)]
pub enum Preview {
    /// A strip button would send the thread here: the `dest` ring pulses.
    Destination,
}

/// A place in the sidebar.
#[component]
pub fn SidebarItem(
    kind: ItemKind,
    label: String,
    here: Here,
    count: Option<u32>,
    presence: Presence,
    preview: Option<Preview>,
    pulse: PulseKey,
    onclick: EventHandler<()>,
    onclose: Option<EventHandler<()>>,
) -> Element {
    todo!()
}
