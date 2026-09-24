//! The card's colour tokens, light and dark (design/03-COLOR.md sections 3, 10-12).
//!
//! `--frame` is renamed `--foreign-ground`; `color-mix()` is replaced by precomputed washes.
//! The frame's `--f-*` are not here: they are derived per Space ([`crate::FrameVars`]).
//!
//! Precomputed washes (section 11 and open decisions 3-4; every value here is proposed):
//! `--ok-wash` is `--ok` at .16 over transparent, exactly what `color-mix(in oklab, var(--ok)
//! 16%, transparent)` paints; `--warn-wash` is `--warn` 14% over `--surface`, mixed in OKLab
//! and rounded to 8 bits; `--danger-wash` takes the failing pill's form, `--danger` at .16 over
//! transparent (the spoof flag's 12% over `--raise` is within a step of it on white);
//! `--accent-ring` is `--accent` at .35. The stylesheet writes `--accent-ring` as a mix of
//! whatever `--accent` the root resolves (see `crate::css::tokens_css`), so it follows every
//! accent; the value here is Postmark's.

use super::accent_table::quad;
use super::hex::{Alpha, Colour, Hex};
use super::name::VarName;
use crate::appearance::{Accent, Scheme};

const fn solid(rgb: u32) -> Colour {
    Colour::Solid(rgb_hex(rgb))
}

const fn alpha(rgb: u32, thousandths: u16) -> Colour {
    Colour::Alpha(rgb_hex(rgb), Alpha(thousandths))
}

/// `0xRRGGBB` as a [`Hex`]; the shifts keep each byte.
const fn rgb_hex(rgb: u32) -> Hex {
    Hex([(rgb >> 16) as u8, (rgb >> 8) as u8, rgb as u8])
}

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
    /// `--danger-ink`: text on `--danger` (the lying link pill's `#fff` in light, O-3; a dark
    /// ink in dark, where white on the lifted `--danger` measured 3.17:1).
    DangerInk,
    /// `--mark-ground`: the white chip under a provider mark (O-3).
    MarkGround,
    /// `--handle-ring`: the slider thumb's border (design/04-COMPONENTS.md section 5 and O-3;
    /// proposed: a quarter-strength ink ring in either scheme).
    HandleRing,
    /// `--on-hue`: the letter on a computed hue ground — the avatar's person and account
    /// tones (design/04-COMPONENTS.md O-3's `#fff`/`#FFFFFF` list, "fav text"; proposed:
    /// white in either scheme, since the hue itself is always mid-toned enough for it).
    OnHue,
    /// `--ok-ink`: text on `--ok` (a status pill), clearing 4.5:1 in either scheme.
    OkInk,
    /// `--warn-ink`: text on `--warn`, clearing 4.5:1 in either scheme.
    WarnInk,
}

impl ColourToken {
    /// Every colour token, in stylesheet order.
    pub const ALL: [ColourToken; 28] = [
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
        ColourToken::HandleRing,
        ColourToken::OnHue,
        ColourToken::OkInk,
        ColourToken::WarnInk,
    ];

    /// The custom property: `--paper`, `--surface-2`, …
    pub fn var(self) -> VarName {
        VarName(match self {
            ColourToken::Paper => "--paper",
            ColourToken::Surface => "--surface",
            ColourToken::Surface2 => "--surface-2",
            ColourToken::Raise => "--raise",
            ColourToken::Ink => "--ink",
            ColourToken::InkSoft => "--ink-soft",
            ColourToken::InkFaint => "--ink-faint",
            ColourToken::Line => "--line",
            ColourToken::LineSoft => "--line-soft",
            ColourToken::Accent => "--accent",
            ColourToken::AccentInk => "--accent-ink",
            ColourToken::AccentSoft => "--accent-soft",
            ColourToken::Seal => "--seal",
            ColourToken::Ok => "--ok",
            ColourToken::Warn => "--warn",
            ColourToken::Danger => "--danger",
            ColourToken::Scrim => "--scrim",
            ColourToken::ForeignGround => "--foreign-ground",
            ColourToken::OkWash => "--ok-wash",
            ColourToken::WarnWash => "--warn-wash",
            ColourToken::DangerWash => "--danger-wash",
            ColourToken::AccentRing => "--accent-ring",
            ColourToken::DangerInk => "--danger-ink",
            ColourToken::MarkGround => "--mark-ground",
            ColourToken::HandleRing => "--handle-ring",
            ColourToken::OnHue => "--on-hue",
            ColourToken::OkInk => "--ok-ink",
            ColourToken::WarnInk => "--warn-ink",
        })
    }

    /// The value in `scheme`, with Postmark as the accent.
    pub fn value(self, scheme: Scheme) -> Colour {
        let postmark = quad(Accent::Postmark, scheme);
        let (light, dark) = match self {
            ColourToken::Accent => return Colour::Solid(postmark.accent),
            ColourToken::AccentInk => return Colour::Solid(postmark.ink),
            ColourToken::AccentSoft => return Colour::Solid(postmark.soft),
            ColourToken::Seal => return Colour::Solid(postmark.seal),
            ColourToken::AccentRing => return Colour::Alpha(postmark.accent, Alpha(350)),
            other => other.post(),
        };
        match scheme {
            Scheme::Light => light,
            Scheme::Dark => dark,
        }
    }

    /// The Post palette, light and dark (design/03-COLOR.md section 3, `S:7-15`, `S:25-36`),
    /// with the washes and literals of sections 11-12. The accent family is the accent
    /// table's and is answered by [`Self::value`] before this is asked.
    fn post(self) -> (Colour, Colour) {
        const WHITE: Colour = solid(0xFFFFFF);
        match self {
            ColourToken::Paper => (solid(0xE9ECE6), solid(0x151814)),
            ColourToken::Surface => (solid(0xF8F9F6), solid(0x1D211B)),
            ColourToken::Surface2 => (solid(0xF1F3EE), solid(0x232722)),
            ColourToken::Raise => (WHITE, solid(0x2A2F28)),
            ColourToken::Ink => (solid(0x1A1E1A), solid(0xE7EBE3)),
            ColourToken::InkSoft => (solid(0x586057), solid(0xA0A79B)),
            ColourToken::InkFaint => (solid(0x676E65), solid(0x8A9284)),
            ColourToken::Line => (solid(0xD6DBD0), solid(0x333A30)),
            ColourToken::LineSoft => (solid(0xE3E7DE), solid(0x282E26)),
            ColourToken::Ok => (solid(0x2C7A57), solid(0x5EB489)),
            ColourToken::Warn => (solid(0xA5761A), solid(0xD2A249)),
            ColourToken::Danger => (solid(0xB03A2A), solid(0xE0705A)),
            // Not redefined in dark (section 3).
            ColourToken::Scrim => (alpha(0x000000, 220), alpha(0x000000, 220)),
            // The sender's page stays white in a dark window (section 12, mailo's `--frame`).
            ColourToken::ForeignGround => (WHITE, WHITE),
            ColourToken::OkWash => (alpha(0x2C7A57, 160), alpha(0x5EB489, 160)),
            ColourToken::WarnWash => (solid(0xEDE6D9), solid(0x333123)),
            ColourToken::DangerWash => (alpha(0xB03A2A, 160), alpha(0xE0705A, 160)),
            // `#fff` on the lying link pill in light (`S:436`); in dark `--danger` is lifted to
            // `#E0705A`, where white measures 3.17:1, so the ink turns dark (mailo's `#1A0B08`,
            // 6.06:1). The legibility test gates every status pair in both schemes.
            ColourToken::DangerInk => (WHITE, solid(0x1A0B08)),
            // White on `--ok` in light (5.21:1); a dark green-black on the lifted dark `--ok`.
            ColourToken::OkInk => (WHITE, solid(0x0B1A12)),
            // White on light `--warn` is 4.03:1, under the gate, so both schemes take a dark ink.
            ColourToken::WarnInk => (solid(0x140D03), solid(0x140D03)),
            ColourToken::MarkGround => (WHITE, WHITE),
            // Proposed (O-3): black at .25 on light, white at .25 on dark.
            ColourToken::HandleRing => (alpha(0x000000, 250), alpha(0xFFFFFF, 250)),
            // `#fff`/`#FFFFFF` on the avatar's person and account tones in both schemes (O-3).
            ColourToken::OnHue => (WHITE, WHITE),
            ColourToken::Accent
            | ColourToken::AccentInk
            | ColourToken::AccentSoft
            | ColourToken::Seal
            | ColourToken::AccentRing => {
                let light = quad(Accent::Postmark, Scheme::Light);
                let dark = quad(Accent::Postmark, Scheme::Dark);
                (Colour::Solid(light.accent), Colour::Solid(dark.accent))
            }
        }
    }
}
