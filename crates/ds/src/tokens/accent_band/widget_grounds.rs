//! What the accent's text lies on inside a desktop widget card (design/23-WIDGETS.md
//! section 4.3; the accent band, design/03-COLOR.md section 20): the `Widget` material's tint at its alpha over
//! blur, composited over the wallpapers the widget gates measure. The card is see-through
//! (alpha .48 light, .55 dark, measured from the reference), so a text accent that reads on the
//! opaque card grounds (`card_grounds`) can fall to 3.7:1 over a warm wallpaper. The widget
//! legibility gates (`tests/legibility.rs`) measure the month's title and busy dots against
//! these grounds.

use super::grounds::over;
use crate::appearance::material::Material;
use crate::appearance::theme::Scheme;
use crate::tokens::hex::Hex;
use crate::tokens::tint::tint;

/// The wallpaper backdrops a desktop widget is gated over: the default wallpaper's colours as
/// sill's desktop capture shows them (2026-09-27, `widgets-desktop-cosmic.png`: a light warm
/// sand, a warm coral, a deep teal, a blue, a dark violet) and a near-black night wallpaper.
/// Pure black and white are the material gates' (`tests/legibility.rs`), where the relaxed 3:1
/// large-text floor applies to the widget's own text.
pub const WALLPAPERS: [Hex; 7] = [
    Hex([226, 193, 139]),
    Hex([225, 182, 133]),
    Hex([223, 129, 105]),
    Hex([38, 138, 150]),
    Hex([45, 83, 141]),
    Hex([104, 64, 128]),
    Hex([28, 26, 38]),
];

/// The widget card's composite over each of [`WALLPAPERS`] in `scheme`.
pub fn widget_grounds(scheme: Scheme) -> Vec<Hex> {
    let Some((hex, alpha)) = tint(Material::Widget, scheme) else {
        return WALLPAPERS.to_vec();
    };
    WALLPAPERS
        .iter()
        .map(|&wallpaper| over(hex, alpha, wallpaper))
        .collect()
}
