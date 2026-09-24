//! ProviderMark: a letter in the provider's colour on a white chip, never the provider's logo,
//! or the favicon the app supplies (design/04-COMPONENTS.md section 28). A local-folders
//! account, which has no provider, shows a neutral folder instead of a letter.

use crate::icon::Icon;
use crate::icon::render::{Glyph, IconPx, IconSize};
use dioxus::prelude::*;

/// A mail provider.
#[derive(Debug, Clone, Copy, PartialEq, Eq, Hash)]
pub enum Provider {
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
    /// No provider: mail kept in local folders (mailo gaps 4). Not a brand, so no letter: a
    /// folder glyph in the neutral IMAP grey, and no favicon even under `MarkStyle::Image`.
    Local,
}

/// Where a mark sits.
#[derive(Debug, Clone, Copy, PartialEq, Eq, Hash)]
pub enum MarkSize {
    /// 14, on an account tile.
    Tile,
    /// 11, in a row's via.
    Row,
    /// 13, inline.
    Inline,
}

/// An image the app supplies, as a `data:` URI. quire never fetches.
#[derive(Debug, Clone, PartialEq, Eq, Hash)]
pub struct ImageSource(pub String);

/// Letter or image.
#[derive(Debug, Clone, PartialEq, Eq, Hash)]
pub enum MarkStyle {
    /// The provider's letter in its colour.
    Letter,
    /// The provider's own favicon.
    Image(ImageSource),
}

impl Provider {
    /// The letter the mark shows (`S:1130-1136`).
    fn letter(self) -> char {
        match self {
            Provider::Google => 'G',
            Provider::Microsoft => 'M',
            Provider::Fastmail => 'F',
            Provider::ICloud => 'i',
            Provider::Yahoo => 'Y',
            Provider::Imap | Provider::Local => '@',
        }
    }

    /// The provider's identity colour: data, not a theme colour, so it enters through the
    /// inline `--pc` (design/04-COMPONENTS.md section 28 Blitz notes, O-3).
    fn colour(self) -> &'static str {
        match self {
            Provider::Google => "#1A73E8",
            Provider::Microsoft => "#0F6CBD",
            Provider::Fastmail => "#2A5DB0",
            Provider::ICloud => "#3A82F7",
            Provider::Yahoo => "#6001D2",
            Provider::Imap | Provider::Local => "#5D6660",
        }
    }

    /// The name in the mark's `title`.
    pub(crate) fn name(self) -> &'static str {
        match self {
            Provider::Google => "Google",
            Provider::Microsoft => "Microsoft 365",
            Provider::Fastmail => "Fastmail",
            Provider::ICloud => "iCloud",
            Provider::Yahoo => "Yahoo",
            Provider::Imap => "IMAP",
            Provider::Local => "Local folders",
        }
    }
}

impl MarkSize {
    /// The `data-size` word.
    fn slug(self) -> &'static str {
        match self {
            MarkSize::Tile => "tile",
            MarkSize::Row => "row",
            MarkSize::Inline => "inline",
        }
    }

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
    let colour = Provider::Local.colour();
    let title = Provider::Local.name();
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
pub fn ProviderMark(provider: Provider, size: MarkSize, style: MarkStyle) -> Element {
    if provider == Provider::Local {
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
