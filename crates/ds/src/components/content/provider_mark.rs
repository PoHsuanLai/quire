//! ProviderMark: a letter in the provider's colour on a white chip, never the provider's logo,
//! or the favicon the app supplies (design/04-COMPONENTS.md section 28). A local-folders
//! account, which has no provider, shows a neutral folder instead of a letter.

use crate::components::content::image_source::ImageSource;
use dioxus::prelude::*;
use ds_core::word::Word;
use ds_style::icon::Icon;
use ds_style::icon::render::{Glyph, IconPx, IconSize};

/// A mail provider.
#[derive(Debug, Clone, Copy, PartialEq, Eq, Hash)]
pub enum MarkProvider {
    /// `G` #1A73E8.
    Google,
    /// `M` #0F6CBD.
    Microsoft,
    /// `F` #2A5DB0.
    Fastmail,
    /// `i` #3A82F7.
    ICloud,
    /// `Y` #6001D2.
    Yahoo,
    /// `@` #5D6660.
    Imap,
    /// No provider: mail kept in local folders. Not a brand, so no letter: a
    /// folder glyph in the neutral IMAP grey, and no favicon even under `MarkStyle::Image`.
    Local,
}

/// Where a mark sits.
#[derive(Debug, Clone, Copy, PartialEq, Eq, Hash, Word)]
pub enum MarkSize {
    /// 14, on an account tile.
    Tile,
    /// 11, in a row's via.
    Row,
    /// 13, inline.
    Inline,
}

/// Letter or image.
#[derive(Debug, Clone, PartialEq, Eq, Hash)]
pub enum MarkStyle {
    /// The provider's letter in its colour.
    Letter,
    /// The provider's own favicon.
    Image(ImageSource),
}

impl MarkProvider {
    /// The letter the mark shows (`S:1130-1136`).
    fn letter(self) -> char {
        match self {
            MarkProvider::Google => 'G',
            MarkProvider::Microsoft => 'M',
            MarkProvider::Fastmail => 'F',
            MarkProvider::ICloud => 'i',
            MarkProvider::Yahoo => 'Y',
            MarkProvider::Imap | MarkProvider::Local => '@',
        }
    }

    /// The provider's identity colour: data, not a theme colour, so it enters through the
    /// inline `--pc` (design/04-COMPONENTS.md section 28 Blitz notes, O-3).
    fn colour(self) -> &'static str {
        match self {
            MarkProvider::Google => "#1A73E8",
            MarkProvider::Microsoft => "#0F6CBD",
            MarkProvider::Fastmail => "#2A5DB0",
            MarkProvider::ICloud => "#3A82F7",
            MarkProvider::Yahoo => "#6001D2",
            MarkProvider::Imap | MarkProvider::Local => "#5D6660",
        }
    }

    /// The name in the mark's `title`.
    pub(crate) fn name(self) -> &'static str {
        match self {
            MarkProvider::Google => "Google",
            MarkProvider::Microsoft => "Microsoft 365",
            MarkProvider::Fastmail => "Fastmail",
            MarkProvider::ICloud => "iCloud",
            MarkProvider::Yahoo => "Yahoo",
            MarkProvider::Imap => "IMAP",
            MarkProvider::Local => "Local folders",
        }
    }
}

impl MarkSize {
    /// The folder glyph inside a local mark: the chip less its padding (14, 11, 13 → 10, 8, 9).
    fn glyph(self) -> IconSize {
        match self {
            MarkSize::Tile => IconSize::Px(IconPx(10)),
            MarkSize::Row => IconSize::Px(IconPx(8)),
            MarkSize::Inline => IconSize::Px(IconPx(9)),
        }
    }
}

/// A local account's mark: the folder in the neutral grey, whatever `style` asks for, since
/// there is no provider whose favicon an app could hold.
fn local_mark(size: MarkSize) -> Element {
    let colour = MarkProvider::Local.colour();
    let title = MarkProvider::Local.name();
    rsx! {
        span {
            class: "ds-provider",
            "data-size": size.slug(),
            "data-kind": "local",
            style: "--pc:{colour}",
            title: "{title}",
            Glyph { icon: Icon::Folder, size: size.glyph() }
        }
    }
}

/// A provider mark. Static: no hover, focus or motion.
#[component]
pub fn ProviderMark(provider: MarkProvider, size: MarkSize, style: MarkStyle) -> Element {
    if provider == MarkProvider::Local {
        return local_mark(size);
    }
    let title = provider.name();
    match style {
        MarkStyle::Letter => {
            let colour = provider.colour();
            let letter = provider.letter();
            rsx! {
                span {
                    class: "ds-provider",
                    "data-size": size.slug(),
                    "data-kind": "letter",
                    style: "--pc:{colour}",
                    title: "{title}",
                    "{letter}"
                }
            }
        }
        MarkStyle::Image(ImageSource(src)) => rsx! {
            span {
                class: "ds-provider",
                "data-size": size.slug(),
                "data-kind": "image",
                title: "{title}",
                img { alt: "", src }
            }
        },
    }
}
