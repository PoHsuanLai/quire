//! The six accents, four properties each, light and dark (the plan's "6 hues x 4 props";
//! design/03-COLOR.md section 5 and open decision 6).
//!
//! Postmark is `S`'s own quad (section 3). The five Candy accents (FINDINGS F5) have no values in
//! any source; this table builds each from its label hue's family (section 15), **proposed**:
//! the accent is the text-safe `deep` member, its tint is the `soft` member, and `--seal` is the
//! accent, as Postmark's is. Text on the accent is white in light (the Space accent's
//! `#FFFFFF`, section 5) and the hue's own dark `soft` in dark, the way Postmark's dark ink is a
//! near-black of its hue. Every quad clears section 6's gates with room: the accent on
//! `--surface` at 5.1:1 or better, `--ink` on the tint at 11:1 or better, and the ink on the
//! accent at 5.4:1 or better (`tests/legibility.rs`).

use super::hex::Hex;
use super::label_hue::{HueMember, LabelHue};
use crate::appearance::{Accent, Scheme};

/// The four properties one accent sets: `--accent`, `--accent-ink`, `--accent-soft`, `--seal`.
#[derive(Debug, Clone, Copy, PartialEq, Eq, Hash)]
pub struct AccentQuad {
    /// `--accent`.
    pub accent: Hex,
    /// `--accent-ink`: text on the accent.
    pub ink: Hex,
    /// `--accent-soft`: the accent's tint.
    pub soft: Hex,
    /// `--seal`.
    pub seal: Hex,
}

/// The quad `accent` paints in `scheme`.
pub fn quad(accent: Accent, scheme: Scheme) -> AccentQuad {
    let hue = match accent {
        Accent::Postmark => return postmark(scheme),
        Accent::Red => LabelHue::Red,
        Accent::Amber => LabelHue::Amber,
        Accent::Green => LabelHue::Green,
        Accent::Blue => LabelHue::Blue,
        Accent::Violet => LabelHue::Violet,
    };
    let deep = hue.value(HueMember::Deep, scheme);
    let soft = hue.value(HueMember::Soft, scheme);
    let ink = match scheme {
        Scheme::Light => Hex([0xFF, 0xFF, 0xFF]),
        Scheme::Dark => soft,
    };
    AccentQuad {
        accent: deep,
        ink,
        soft,
        seal: deep,
    }
}

/// Postmark, `S:11` and `S:30` (design/03-COLOR.md sections 3 and 5).
fn postmark(scheme: Scheme) -> AccentQuad {
    match scheme {
        Scheme::Light => AccentQuad {
            accent: Hex([0x23, 0x50, 0x8F]),
            ink: Hex([0xF4, 0xF8, 0xFF]),
            soft: Hex([0xDC, 0xE5, 0xF3]),
            seal: Hex([0x23, 0x50, 0x8F]),
        },
        Scheme::Dark => AccentQuad {
            accent: Hex([0x7F, 0xA6, 0xE6]),
            ink: Hex([0x0B, 0x14, 0x2A]),
            soft: Hex([0x1E, 0x2A, 0x44]),
            seal: Hex([0x7F, 0xA6, 0xE6]),
        },
    }
}
