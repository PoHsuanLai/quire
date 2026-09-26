//! What a preview pane shows (sill Q292): the content as data, and how each kind draws its
//! media and its caption. Split from `preview_pane`, which frames them with the actions.

use crate::components::icon_view::IconView;
use crate::components::image_source::{ImageSize, ImageSource};
use crate::components::pdf_thumb::{PdfPage, PdfThumb, sheet_rect};
use crate::components::shot_frame::picture_style;
use crate::geometry::{Px, Size};
use crate::icon::Icon;
use crate::icon::external::IconSource;
use crate::icon::family::PlateFamily;
use crate::icon::render::{IconPx, IconSize};
use dioxus::prelude::*;

/// The pane's media box: a picture or a page is fitted into it at its own aspect, and a PDF's
/// page is rasterised for it (`ds_native::use_pdf_page(path, PANE_MEDIA)`). 328 is a 360 pane's
/// width inside its 16 px padding.
pub const PANE_MEDIA: Size = Size {
    width: Px(328.0),
    height: Px(220.0),
};

/// Whether a text is drawn in the code face.
#[derive(Debug, Clone, Copy, PartialEq, Eq, Hash)]
pub enum Mono {
    /// `--font-code`: a clipboard snippet, a source file.
    Monospace,
    /// The text face.
    Proportional,
}

/// What the pane previews.
#[derive(Debug, Clone, PartialEq)]
pub enum PaneContent {
    /// A picture, fitted into the media box at its own aspect.
    Image {
        /// Its pixels.
        src: ImageSource,
        /// Its size; only the ratio is read.
        size: ImageSize,
    },
    /// Text: a note, a snippet, a file's start.
    Text {
        /// What is shown; the pane clips what does not fit.
        excerpt: String,
        /// Its face.
        mono: Mono,
    },
    /// A PDF's first page, as `PdfThumb` draws it. ds reads no file: a Blitz app gets the page
    /// from `ds_native::use_pdf_page(Some(path), PANE_MEDIA)` (feature `pdf-thumb`).
    Pdf {
        /// The page, or where its reading is.
        page: PdfPage,
        /// The file's name, the caption and the page's label.
        name: String,
    },
    /// An app: its icon, its name, one line about it.
    App {
        /// Its icon (an app's icon image, drawn as it is).
        icon: IconSource,
        /// Its name.
        name: String,
        /// A line under it (its comment, its version).
        detail: Option<String>,
    },
    /// Anything described by a few facts (a setting, a contact, a folder): an icon, a title and
    /// label-value rows.
    Facts {
        /// Its icon.
        icon: IconSource,
        /// Its name.
        title: String,
        /// `(label, value)` rows, in order.
        rows: Vec<(String, String)>,
    },
    /// One emoji, large, and its name.
    Emoji {
        /// The emoji.
        glyph: String,
        /// Its name.
        name: String,
    },
    /// A web search or address: the globe, the host and the address.
    Web {
        /// The host ("duckduckgo.com").
        host: String,
        /// The whole address.
        url: String,
    },
}

impl PaneContent {
    /// The `data-content` word.
    pub(crate) fn slug(&self) -> &'static str {
        match self {
            PaneContent::Image { .. } => "image",
            PaneContent::Text { .. } => "text",
            PaneContent::Pdf { .. } => "pdf",
            PaneContent::App { .. } => "app",
            PaneContent::Facts { .. } => "facts",
            PaneContent::Emoji { .. } => "emoji",
            PaneContent::Web { .. } => "web",
        }
    }

    /// What a screen reader calls the pane.
    pub(crate) fn label(&self) -> String {
        match self {
            PaneContent::Image { .. } => "Picture preview".to_owned(),
            PaneContent::Text { .. } => "Text preview".to_owned(),
            PaneContent::Pdf { name, .. } | PaneContent::App { name, .. } => {
                format!("{name} preview")
            }
            PaneContent::Facts { title, .. } => format!("{title} preview"),
            PaneContent::Emoji { name, .. } => format!("{name} preview"),
            PaneContent::Web { host, .. } => format!("{host} preview"),
        }
    }
}

/// The media box's contents.
pub(crate) fn media(content: &PaneContent) -> Element {
    match content {
        PaneContent::Image { src, size } => {
            let style = picture_style(sheet_rect(PANE_MEDIA, *size));
            rsx! {
                img { class: "ds-preview-image", alt: "", src: src.0.clone(), draggable: "false", style }
            }
        }
        PaneContent::Text { excerpt, mono } => {
            let face = match mono {
                Mono::Monospace => "mono",
                Mono::Proportional => "prose",
            };
            rsx! {
                div { class: "ds-preview-text", "data-face": face, "{excerpt}" }
            }
        }
        PaneContent::Pdf { page, name } => rsx! {
            PdfThumb { page: page.clone(), size: PANE_MEDIA, label: name.clone() }
        },
        PaneContent::App { icon, .. } => icon_at(icon.clone(), 96, None),
        PaneContent::Facts { icon, .. } => icon_at(icon.clone(), 64, None),
        PaneContent::Emoji { glyph, .. } => rsx! {
            span { class: "ds-preview-emoji ds-emoji-text", "aria-hidden": "true", "{glyph}" }
        },
        PaneContent::Web { .. } => {
            icon_at(IconSource::Glyph(Icon::Globe), 64, Some(PlateFamily::Blue))
        }
    }
}

/// An icon at `side` px, on a plate when given one.
fn icon_at(source: IconSource, side: u8, plate: Option<PlateFamily>) -> Element {
    rsx! {
        IconView { source, size: IconSize::Px(IconPx(side)), plate }
    }
}

/// The caption under the media: a title and a line, and a list of facts.
pub(crate) fn caption(content: &PaneContent) -> Element {
    let (title, line): (Option<&str>, Option<&str>) = match content {
        PaneContent::Image { .. } | PaneContent::Text { .. } => (None, None),
        PaneContent::Pdf { name, .. } => (Some(name), None),
        PaneContent::App { name, detail, .. } => (Some(name), detail.as_deref()),
        PaneContent::Facts { title, .. } => (Some(title), None),
        PaneContent::Emoji { name, .. } => (Some(name), None),
        PaneContent::Web { host, url } => (Some(host), Some(url)),
    };
    let facts = match content {
        PaneContent::Facts { rows, .. } => rows.clone(),
        _ => Vec::new(),
    };
    let title = title.map(str::to_owned);
    let line = line.map(str::to_owned);
    rsx! {
        if title.is_some() || line.is_some() {
            div { class: "ds-preview-caption",
                if let Some(title) = title {
                    b { class: "ds-preview-title", "{title}" }
                }
                if let Some(line) = line {
                    small { class: "ds-preview-line ds-truncate", "{line}" }
                }
            }
        }
        if !facts.is_empty() {
            dl { class: "ds-preview-facts",
                for (index , (label , value)) in facts.into_iter().enumerate() {
                    div { key: "{index}", class: "ds-preview-fact",
                        dt { "{label}" }
                        dd { "{value}" }
                    }
                }
            }
        }
    }
}
