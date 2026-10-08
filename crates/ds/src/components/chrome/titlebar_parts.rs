//! What goes with a window's title (design/30 section 2.7, `NSWindow` titlebar): a proxy icon
//! before it, the edited dot after it, and a subtitle in the faint ink. Data, and the one
//! function that draws the title area from it.
//!
//! Markup: `span.ds-titlebar-title` holding `span.ds-titlebar-proxy`, `span.ds-titlebar-name`,
//! `span.ds-titlebar-edited` and `span.ds-titlebar-subtitle`, each present only when it has
//! something to say.

use crate::host::measure::use_rect;
use dioxus::prelude::*;
use ds_core::geometry::units::Px;
use ds_core::text::clip::clip_middle;
use ds_core::word::Word;
use ds_style::icon::Icon;
use ds_style::icon::render::{Glyph, IconSize};

/// Whether the window's document has changes that are not saved (`isDocumentEdited`).
#[derive(Debug, Clone, Copy, PartialEq, Eq, Hash, Default, Word)]
pub enum DocumentState {
    /// Nothing unsaved.
    #[default]
    Saved,
    /// Unsaved changes: the titlebar draws the edited dot.
    Edited,
}

/// The parts beside a title.
#[derive(Debug, Clone, PartialEq, Default)]
pub struct TitleParts {
    /// A line after the title in the faint ink (`NSWindow.subtitle`).
    pub subtitle: Option<String>,
    /// The document's icon before the title: what a person drags to move or attach the file.
    pub proxy: Option<Icon>,
    /// Whether the document has unsaved changes.
    pub document: DocumentState,
}

/// The room a proxy icon takes beside the name: its 14 px and the gap after it.
const PROXY_ROOM: Px = Px(20.0);

/// The room the edited dot takes beside the name: its 6 px and the gap before it.
const EDITED_ROOM: Px = Px(12.0);

/// The gap before a subtitle.
const SUBTITLE_GAP: Px = Px(6.0);

/// How much of the room the estimate may use: the kept ends are not average text.
const FILL: f32 = 0.95;

impl TitleParts {
    /// The room beside the name that is not the subtitle's: the proxy icon and the edited dot.
    fn fixed_room(&self) -> Px {
        let proxy = self.proxy.map_or(Px(0.0), |_| PROXY_ROOM);
        let edited = match self.document {
            DocumentState::Edited => EDITED_ROOM,
            DocumentState::Saved => Px(0.0),
        };
        proxy + edited
    }
}

/// The part of `title` that fits `room` when the whole of it is `whole` wide: the title itself
/// when it fits or nothing was measured, else its middle cut to the share of its characters
/// that the room holds, so a long name still shows its start and its extension.
fn fitted(title: &str, room: Option<Px>, whole: Option<Px>) -> String {
    let (Some(room), Some(whole)) = (room, whole) else {
        return title.to_owned();
    };
    if whole.0 <= room.0 || whole.0 <= 0.0 {
        return title.to_owned();
    }
    let count = title.chars().count() as f32;
    let share = (count * FILL * room.0.max(0.0) / whole.0).floor() as usize;
    clip_middle(title, share)
}

/// The title area: `title` with `parts` around it. A name wider than the room beside the other
/// parts ends in an ellipsis in the middle (Blitz has no `text-overflow: ellipsis`): the whole
/// name is laid out unseen in `span.ds-titlebar-ruler` to be measured, and the name drawn is
/// cut to fit what that says.
#[component]
pub(crate) fn TitleArea(title: String, parts: TitleParts) -> Element {
    let area = use_rect();
    let name = use_rect();
    let subtitle = use_rect();
    let subtitle_room = match parts.subtitle {
        Some(_) => subtitle
            .rect()
            .map_or(Px(0.0), |rect| rect.size.width + SUBTITLE_GAP),
        None => Px(0.0),
    };
    let room = area
        .rect()
        .map(|rect| rect.size.width - parts.fixed_room() - subtitle_room);
    let shown = fitted(&title, room, name.rect().map(|rect| rect.size.width));
    rsx! {
        span {
            class: "ds-titlebar-title",
            "data-document": parts.document.slug(),
            onmounted: move |event| area.on_mounted(event),
            if let Some(icon) = parts.proxy {
                span { class: "ds-titlebar-proxy", "aria-hidden": "true",
                    Glyph { icon, size: IconSize::Compact }
                }
            }
            span { class: "ds-titlebar-name", "{shown}" }
            if parts.document == DocumentState::Edited {
                span { class: "ds-titlebar-edited", role: "img", "aria-label": "Edited" }
            }
            if let Some(text) = parts.subtitle.as_ref() {
                span { class: "ds-titlebar-subtitle", "{text}" }
            }
            span { class: "ds-titlebar-ruler", "aria-hidden": "true",
                span { class: "ds-ruler-name", onmounted: move |event| name.on_mounted(event), "{title}" }
                if let Some(text) = parts.subtitle.as_ref() {
                    span { class: "ds-ruler-subtitle", onmounted: move |event| subtitle.on_mounted(event), "{text}" }
                }
            }
        }
    }
}
