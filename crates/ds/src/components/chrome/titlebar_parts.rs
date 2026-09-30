//! What goes with a window's title (design/30 section 2.7, `NSWindow` titlebar): a proxy icon
//! before it, the edited dot after it, and a subtitle in the faint ink. Data, and the one
//! function that draws the title area from it.
//!
//! Markup: `span.ds-titlebar-title` holding `span.ds-titlebar-proxy`, `span.ds-titlebar-name`,
//! `span.ds-titlebar-edited` and `span.ds-titlebar-subtitle`, each present only when it has
//! something to say.

use dioxus::prelude::*;
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

impl TitleParts {
    /// The title area: `title` with these parts around it.
    pub(crate) fn view(&self, title: &str) -> Element {
        rsx! {
            span { class: "ds-titlebar-title", "data-document": self.document.slug(),
                if let Some(icon) = self.proxy {
                    span { class: "ds-titlebar-proxy", "aria-hidden": "true",
                        Glyph { icon, size: IconSize::Compact }
                    }
                }
                span { class: "ds-titlebar-name ds-truncate", "{title}" }
                if self.document == DocumentState::Edited {
                    span { class: "ds-titlebar-edited", role: "img", "aria-label": "Edited" }
                }
                if let Some(subtitle) = self.subtitle.as_ref() {
                    span { class: "ds-titlebar-subtitle ds-truncate", "{subtitle}" }
                }
            }
        }
    }
}
