//! Everything the text accent lies on: the card's grounds, the translucent materials over their
//! reference backdrops, and the wash laid over either (design/03-COLOR.md section 20.6).
//!
//! The card's four grounds alone were not enough: a menu's match highlight
//! sits on a selected row's wash, and PolkitPrompt's "Details" on a Sheet that is 82 % tint over
//! whatever lies behind it. Each is measured here as a screen composites it.
//!
//! One text accent cannot serve both kinds of ground and stay the band's: over a black backdrop
//! a light Popover is `#c7c7c7`, and text that reads there is Postmark's old navy (OKLCH L .43)
//! that band B was picked to leave; over a white one a dark Popover is `#585e56`, and text that
//! reads there is nearly the ink. So there are two: [`TextOn::Card`], gated on the opaque
//! grounds and the wash over them, and [`TextOn::Material`], gated on every ground here.

use super::floors;
use super::grounds::{card_grounds, card_ink, contrast};
use crate::style::appearance::material::Material;
use crate::style::appearance::theme::Scheme;
use crate::style::tokens::hex::{Alpha, Hex};
use crate::style::tokens::tint::tint;

/// The two worst backdrops a blur can show, the same two `tests/legibility.rs` holds the
/// materials' own ink over (design/21-SPACES.md section 7).
pub const BACKDROPS: [Hex; 2] = [Hex([0, 0, 0]), Hex([0xFF, 0xFF, 0xFF])];

/// The materials that carry accent text: a menu's marks and checks (Popover), the launcher,
/// the control and notification centers and PolkitPrompt (Sheet), a banner's link (Toast).
///
/// The bar, the dock and the OSD draw no accent text. The widget's tint is gated at the
/// large-text 3:1 (design/23-WIDGETS.md section 4.3) and its accent text is the widget card's
/// own concern, so it is not here.
pub const TEXT_MATERIALS: [Material; 3] = [Material::Popover, Material::Sheet, Material::Toast];

/// Which text accent a set of grounds is for.
#[derive(Debug, Clone, Copy, PartialEq, Eq, Hash)]
pub enum TextOn {
    /// `--accent-text`: the card's grounds and the wash over them.
    Card,
    /// `--accent-text-material`: those, and the text-carrying materials over both backdrops and
    /// the wash over them. A root of one of [`TEXT_MATERIALS`] points `--accent-text` at it.
    Material,
}

/// What kind of ground a [`Ground`] is, so a failure can say which.
#[derive(Debug, Clone, Copy, PartialEq, Eq, Hash)]
pub enum GroundKind {
    /// `--paper`, `--surface`, `--surface-2` or `--raise`.
    Card,
    /// One of [`TEXT_MATERIALS`] at its default tint over one of [`BACKDROPS`].
    Material,
    /// The wash over a card ground.
    WashOnCard,
    /// The wash over a material over a backdrop.
    WashOnMaterial,
}

/// One opaque colour the text accent may be read on.
#[derive(Debug, Clone, Copy, PartialEq, Eq, Hash)]
pub struct Ground {
    /// Where it comes from.
    pub kind: GroundKind,
    /// The colour a screen shows there.
    pub hex: Hex,
}

/// Each of [`TEXT_MATERIALS`] in `scheme` over each of [`BACKDROPS`].
pub fn material_grounds(scheme: Scheme) -> Vec<Hex> {
    TEXT_MATERIALS
        .into_iter()
        .filter_map(|material| tint(material, scheme))
        .flat_map(|(hex, alpha)| BACKDROPS.map(|backdrop| hex.over(alpha, backdrop)))
        .collect()
}

/// Every ground the `on` text accent must read on in `scheme`, given the accent's `fill` and
/// `wash`.
///
/// A ground on which the card's own `--ink` falls under the text floor is left out: the accent
/// cannot be asked to read where the ink does not, and such a ground is the material's
/// shortfall (in the band as settled, one: the dark Popover's wash over a white backdrop, where
/// the ink reaches 4.1:1; `tests::the_only_ground_the_ink_misses_is_the_dark_popover_wash`).
pub fn text_grounds(on: TextOn, scheme: Scheme, fill: Hex, wash: Alpha) -> Vec<Ground> {
    let ink = card_ink(scheme);
    let plain =
        |kind: GroundKind, hexes: Vec<Hex>| hexes.into_iter().map(move |hex| Ground { kind, hex });
    let washed = |kind: GroundKind, hexes: Vec<Hex>| {
        hexes.into_iter().map(move |hex| Ground {
            kind,
            hex: fill.over(wash, hex),
        })
    };
    let card = card_grounds(scheme).to_vec();
    let material = match on {
        TextOn::Card => Vec::new(),
        TextOn::Material => material_grounds(scheme),
    };
    plain(GroundKind::Card, card.clone())
        .chain(plain(GroundKind::Material, material.clone()))
        .chain(washed(GroundKind::WashOnCard, card))
        .chain(washed(GroundKind::WashOnMaterial, material))
        .filter(|ground| contrast(ink, ground.hex) >= floors::TEXT)
        .collect()
}

/// Whether `material` points `--accent-text` at `--accent-text-material`.
pub fn text_on(material: Material) -> TextOn {
    if TEXT_MATERIALS.contains(&material) {
        TextOn::Material
    } else {
        TextOn::Card
    }
}
