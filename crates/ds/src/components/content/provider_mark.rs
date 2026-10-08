//! ProviderMark: a letter in the provider's colour on a white chip, never the provider's logo,
//! or the favicon the app supplies (design/04-COMPONENTS.md section 28). A local-folders
//! account, which has no provider, shows a neutral folder instead of a letter.

use crate::components::content::avatar::{AvatarFace, AvatarShape, AvatarSize, AvatarTone};
use crate::components::content::image_source::ImageSource;
use crate::components::content::mark_face::{DrawnFace, MarkFace};
use crate::components::content::title_tip::{Tip, use_tip};
use crate::root::common::Common;
use dioxus::prelude::*;
use ds_core::word::Word;
use ds_style::icon::Icon;
use ds_style::icon::render::{Glyph, IconPx, IconSize};
use ds_style::tokens::control_size::ControlSize;
use ds_style::tokens::hex::{Colour, Hex};

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

/// Letter or image.
#[derive(Debug, Clone, PartialEq, Eq, Hash, Default)]
pub enum MarkStyle {
    /// The provider's letter in its colour.
    #[default]
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

    /// The provider as a round avatar of `size`: its letter on its identity colour, the shape
    /// the account screens lead their rows and headers with.
    pub fn avatar(self, size: AvatarSize) -> AvatarFace {
        let colour = self.colour();
        let channel = |from: usize| {
            colour
                .get(from..from + 2)
                .and_then(|pair| u8::from_str_radix(pair, 16).ok())
                .unwrap_or_default()
        };
        AvatarFace {
            initial: self.letter(),
            size,
            tone: AvatarTone::Account(Colour::Solid(Hex([channel(1), channel(3), channel(5)]))),
            shape: AvatarShape::Round,
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
            MarkProvider::Imap => "Mail account",
            MarkProvider::Local => "Local folders",
        }
    }
}

/// The folder glyph inside a local mark: the chip less its padding (11, 13, 14 becomes 8, 9, 10).
fn folder_glyph(size: ControlSize) -> IconSize {
    match size {
        ControlSize::Mini => IconSize::Px(IconPx(8)),
        ControlSize::Small => IconSize::Px(IconPx(9)),
        ControlSize::Regular | ControlSize::Large | ControlSize::ExtraLarge => {
            IconSize::Px(IconPx(10))
        }
    }
}

/// A local account's mark: the folder in the neutral grey, whatever `style` asks for, since
/// there is no provider whose favicon an app could hold.
fn local_mark(size: ControlSize, common: Common, tip: Tip) -> Element {
    let colour = MarkProvider::Local.colour();
    let class = common.class("ds-provider");
    let data = common.data_attributes();
    rsx! {
        span {
            id: common.id.clone(),
            class,
            "data-size": size.slug(),
            "data-kind": "local",
            style: "--pc:{colour}",
            title: tip.native(),
            onmouseover: { let tip = tip.clone(); move |event| tip.over(&event) },
            onmouseleave: { let tip = tip.clone(); move |_| tip.out() },
            "aria-label": common.aria_label.clone(),
            onmounted: { let tip = tip.clone(); move |event| { tip.mounted(&event); common.mounted(event) } },
            ..data,
            Glyph { icon: Icon::Folder, size: folder_glyph(size) }
        }
        {tip.surface()}
    }
}

/// A face's mark: the letter in its chosen ink on the face's colour.
fn face_mark(drawn: DrawnFace, size: ControlSize, common: Common, tip: Tip) -> Element {
    let class = common.class("ds-provider");
    let data = common.data_attributes();
    let DrawnFace {
        letter,
        count,
        colour,
        ink,
    } = drawn;
    let letters = count.slug();
    rsx! {
        span {
            id: common.id.clone(),
            class,
            "data-size": size.slug(),
            "data-kind": "face",
            "data-letters": letters,
            style: "--pc:{colour};--pi:{ink}",
            title: tip.native(),
            onmouseover: { let tip = tip.clone(); move |event| tip.over(&event) },
            onmouseleave: { let tip = tip.clone(); move |_| tip.out() },
            "aria-label": common.aria_label.clone(),
            onmounted: { let tip = tip.clone(); move |event| { tip.mounted(&event); common.mounted(event) } },
            ..data,
            "{letter}"
        }
        {tip.surface()}
    }
}

/// A provider mark: the provider's glyph on a tile of the control ladder (Mini 11, Small 13,
/// Regular 14; Large repeats Regular). Static: no hover, focus or motion. A `face` (a letter
/// and a colour supplied as data) is drawn first; without one the named `provider` stands,
/// and the `@` of the neutral variant is the last resort.
#[component]
pub fn ProviderMark(
    provider: MarkProvider,
    #[props(default)] face: Option<MarkFace>,
    #[props(default)] size: ControlSize,
    #[props(default = MarkStyle::Letter)] style: MarkStyle,
    #[props(default)] common: Common,
) -> Element {
    let tip = use_tip(Some(provider.name().to_owned()));
    if let (Some(drawn), MarkStyle::Letter) = (face.as_ref().and_then(MarkFace::drawn), &style) {
        return face_mark(drawn, size, common, tip);
    }
    if provider == MarkProvider::Local {
        return local_mark(size, common, tip);
    }
    let class = common.class("ds-provider");
    let data = common.data_attributes();
    match style {
        MarkStyle::Letter => {
            let colour = provider.colour();
            let letter = provider.letter();
            rsx! {
                span {
                    id: common.id.clone(),
                    class,
                    "data-size": size.slug(),
                    "data-kind": "letter",
                    style: "--pc:{colour}",
                    title: tip.native(),
            onmouseover: { let tip = tip.clone(); move |event| tip.over(&event) },
            onmouseleave: { let tip = tip.clone(); move |_| tip.out() },
                    "aria-label": common.aria_label.clone(),
                    onmounted: { let tip = tip.clone(); move |event| { tip.mounted(&event); common.mounted(event) } },
                    ..data,
                    "{letter}"
                }
                {tip.surface()}
            }
        }
        MarkStyle::Image(ImageSource(src)) => rsx! {
            span {
                id: common.id.clone(),
                class,
                "data-size": size.slug(),
                "data-kind": "image",
                title: tip.native(),
            onmouseover: { let tip = tip.clone(); move |event| tip.over(&event) },
            onmouseleave: { let tip = tip.clone(); move |_| tip.out() },
                "aria-label": common.aria_label.clone(),
                onmounted: { let tip = tip.clone(); move |event| { tip.mounted(&event); common.mounted(event) } },
                ..data,
                img { alt: "", src }
            }
            {tip.surface()}
        },
    }
}

#[cfg(test)]
mod tests {
    use super::MarkProvider;
    use crate::components::content::avatar::{AvatarShape, AvatarSize, AvatarTone};
    use ds_style::tokens::hex::{Colour, Hex};

    #[test]
    fn a_provider_as_an_avatar_is_a_disc_of_its_letter_and_colour() {
        let face = MarkProvider::Google.avatar(AvatarSize::Size28);
        assert_eq!(face.initial, 'G');
        assert_eq!(face.shape, AvatarShape::Round);
        assert_eq!(
            face.tone,
            AvatarTone::Account(Colour::Solid(Hex([0x1A, 0x73, 0xE8])))
        );
    }
}
