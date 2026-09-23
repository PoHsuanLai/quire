//! The card's colour tokens, light and dark (design/03-COLOR.md sections 3, 10-12).
//!
//! `--frame` is renamed `--foreign-ground`; `color-mix()` is replaced by precomputed washes.
//! The frame's `--f-*` are not here: they are derived per Space ([`crate::FrameVars`]).
#![allow(unused_variables)] // Freeze stubs: remove with the last todo!().

use super::hex::Colour;
use super::name::VarName;
use crate::appearance::Scheme;

/// One card colour token.
#[derive(Debug, Clone, Copy, PartialEq, Eq, Hash)]
pub enum ColourToken {
    /// `--paper`: page ground; inverse text on ink pills.
    Paper,
    /// `--surface`: card, list, rows.
    Surface,
    /// `--surface-2`: reader, composer, peek, mini, kbd.
    Surface2,
    /// `--raise`: selected row, popovers, menus.
    Raise,
    /// `--ink`: primary text; inverse pill ground.
    Ink,
    /// `--ink-soft`: secondary text.
    InkSoft,
    /// `--ink-faint`: metadata.
    InkFaint,
    /// `--line`: borders and dividers.
    Line,
    /// `--line-soft`: inner dividers.
    LineSoft,
    /// `--accent`: from the accent table.
    Accent,
    /// `--accent-ink`: text on the accent.
    AccentInk,
    /// `--accent-soft`: the accent's tint.
    AccentSoft,
    /// `--seal`: defined, unused in the Spaces prototype (design/03-COLOR.md open decision 7).
    Seal,
    /// `--ok`: saved, live.
    Ok,
    /// `--warn`: star, dirty, attachment warning.
    Warn,
    /// `--danger`: spoof flag, lying link.
    Danger,
    /// `--scrim`: behind peek and the command menu.
    Scrim,
    /// `--foreign-ground`: the sender's page behind an original message (was `--frame`).
    ForeignGround,
    /// `--ok-wash`: `--ok` at 16% over transparent.
    OkWash,
    /// `--warn-wash`: `--warn` at 14% over `--surface`.
    WarnWash,
    /// `--danger-wash`: `--danger` over transparent or `--raise` (open decision 3).
    DangerWash,
    /// `--accent-ring`: `--accent` at 35%, the destination ring and the input focus ring.
    AccentRing,
    /// `--danger-ink`: text on `--danger` (the lying link pill's `#fff`, O-3).
    DangerInk,
    /// `--mark-ground`: the white chip under a provider mark (O-3).
    MarkGround,
}

impl ColourToken {
    /// Every colour token, in stylesheet order.
    pub const ALL: [ColourToken; 24] = [
        ColourToken::Paper,
        ColourToken::Surface,
        ColourToken::Surface2,
        ColourToken::Raise,
        ColourToken::Ink,
        ColourToken::InkSoft,
        ColourToken::InkFaint,
        ColourToken::Line,
        ColourToken::LineSoft,
        ColourToken::Accent,
        ColourToken::AccentInk,
        ColourToken::AccentSoft,
        ColourToken::Seal,
        ColourToken::Ok,
        ColourToken::Warn,
        ColourToken::Danger,
        ColourToken::Scrim,
        ColourToken::ForeignGround,
        ColourToken::OkWash,
        ColourToken::WarnWash,
        ColourToken::DangerWash,
        ColourToken::AccentRing,
        ColourToken::DangerInk,
        ColourToken::MarkGround,
    ];

    /// The custom property: `--paper`, `--surface-2`, …
    pub fn var(self) -> VarName {
        todo!()
    }

    /// The value in `scheme`, with Postmark as the accent.
    pub fn value(self, scheme: Scheme) -> Colour {
        todo!()
    }
}
