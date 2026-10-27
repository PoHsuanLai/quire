//! The standard file icon under `icons.style = monochrome` (design/08-ICONS.md section 2.10): the
//! folder and the marks take the tint, as an app icon's plate does.
//!
//! The folder's tones and the embossed glyph go through the same OKLCh rule as a plate's stops
//! (`super::retint`), once per scheme; the marks sit on the page, which stays light in the dark, so
//! they are tinted once, as the light scheme draws them. The values reach the stylesheet as custom
//! properties on the icon (`--file-tab-l` ... `--file-mark`), which it reads under the root's
//! `data-theme` as a tinted plate's are.

use super::retint::{IconStyle, Tint, recolour};
use super::tone_band::Tone;
use crate::appearance::theme::Scheme;
use crate::tokens::file_icon::FileIconColour;
use crate::tokens::hex::Hex;
use crate::tokens::name::VarName;
use ds_core::word::Word;

/// How a file icon is coloured.
#[derive(Debug, Clone, Copy, PartialEq, Default)]
pub enum FileIconStyle {
    /// Each kind in its own colour, the folder blue.
    #[default]
    Colour,
    /// The folder and the marks in the tint's one hue.
    Monochrome(Tint),
}

impl FileIconStyle {
    /// The style for `icons.style` and the Space's tint (`Tint::space`), the pair sill already
    /// hands `retint`: only Monochrome changes a file icon (Muted leaves it as designed).
    pub fn of(style: IconStyle, tint: Tint) -> FileIconStyle {
        match style {
            IconStyle::Monochrome => FileIconStyle::Monochrome(tint),
            IconStyle::Colour | IconStyle::Muted => FileIconStyle::Colour,
        }
    }

    /// The `data-icon-style` word the stylesheet selects the tinted paint by; `None` for Colour.
    pub fn slug(self) -> Option<&'static str> {
        match self {
            FileIconStyle::Colour => None,
            FileIconStyle::Monochrome(_) => Some("monochrome"),
        }
    }

    /// The inline custom properties of the tinted paint; empty for Colour.
    pub fn style_attribute(self) -> String {
        match self {
            FileIconStyle::Colour => String::new(),
            FileIconStyle::Monochrome(tint) => tint_vars(tint)
                .into_iter()
                .map(|(name, colour)| format!("{}:{};", name.as_str(), colour.css()))
                .collect(),
        }
    }
}

/// The folder parts, per scheme, and the mark: each custom property with its tinted colour.
fn tint_vars(tint: Tint) -> Vec<(VarName, Hex)> {
    const PARTS: [(FileIconColour, [VarName; 2]); 4] = [
        (
            FileIconColour::FolderTab,
            [VarName("--file-tab-l"), VarName("--file-tab-d")],
        ),
        (
            FileIconColour::FolderBody,
            [VarName("--file-body-l"), VarName("--file-body-d")],
        ),
        (
            FileIconColour::FolderInk,
            [VarName("--file-ink-l"), VarName("--file-ink-d")],
        ),
        (
            FileIconColour::FolderLight,
            [VarName("--file-light-l"), VarName("--file-light-d")],
        ),
    ];
    let tinted = |colour: Hex, scheme: Scheme, tone: Tone| {
        Hex(recolour(
            colour.0,
            u8::MAX,
            IconStyle::Monochrome,
            tint,
            (scheme, tone),
        ))
    };
    let mut vars: Vec<(VarName, Hex)> = Vec::new();
    for (token, names) in PARTS {
        for (scheme, name) in Scheme::ALL.iter().copied().zip(names) {
            let tone = match token {
                FileIconColour::FolderTab | FileIconColour::FolderBody => Tone::Plate,
                _ => Tone::Art,
            };
            vars.push((name, tinted(token.hex(scheme), scheme, tone)));
        }
    }
    let mark = FileIconColour::Label.hex(Scheme::Light);
    vars.push((
        VarName("--file-mark"),
        tinted(mark, Scheme::Light, Tone::Art),
    ));
    vars
}
