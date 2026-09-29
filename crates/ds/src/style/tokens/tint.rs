//! Each material's tint over blur (design/03-COLOR.md section 17.2): its colour, with the
//! vibrancy boost baked in, and its alpha at the default key. The accent band measures text
//! against these, and the material recipe paints them.

use crate::style::appearance::material::Material;
use crate::style::appearance::theme::Scheme;
use crate::style::tokens::hex::{Alpha, Hex};
use crate::style::tokens::vibrancy::boosted;

/// The tint's colour with the vibrancy boost baked in, and its alpha at the default key, or
/// `None` for the window.
pub(crate) fn tint(material: Material, scheme: Scheme) -> Option<(Hex, Alpha)> {
    flat_tint(material, scheme).map(|(hex, alpha)| (boosted(hex, scheme), alpha))
}

/// Section 17.2's own tint colour, before the vibrancy boost, and its alpha at the default key,
/// or `None` for the window.
///
/// Light tints are `--surface` (`rgba(248,249,246,…)`) and the popover's `--raise` white; dark
/// tints are `--paper` (`rgba(21,24,20,…)`) and the popover's dark `--raise`.
pub(crate) fn flat_tint(material: Material, scheme: Scheme) -> Option<(Hex, Alpha)> {
    const SURFACE: Hex = Hex([248, 249, 246]);
    const WHITE: Hex = Hex([255, 255, 255]);
    const PAPER_DARK: Hex = Hex([21, 24, 20]);
    const RAISE_DARK: Hex = Hex([42, 47, 40]);
    // The light widget's tint (the contrast-relax pass, 2026-09-26, design/23-WIDGETS.md
    // section 1.1 M27-M28, design/03-COLOR.md section 17): the reference's own measured
    // near-white, boosted.
    const WIDGET_NEAR_WHITE: Hex = Hex([247, 248, 248]);
    let light = scheme == Scheme::Light;
    let (hex, alpha) = match material {
        Material::Window => return None,
        Material::Bar if light => (SURFACE, 700),
        // .58 -> .66 (over an opaque ground) -> .68 (settled 2026-09-24, over blur:
        // held the dark ink at 4.36:1 over white at .66, short of 4.5).
        Material::Bar => (PAPER_DARK, 680),
        // .55 -> .59 (settled 2026-09-24, over blur: held 4.11:1 over black at .55).
        Material::Dock if light => (SURFACE, 590),
        // .50 -> .66 (as the dark bar) -> .68 (settled 2026-09-24, over blur).
        Material::Dock => (PAPER_DARK, 680),
        Material::Popover if light => (WHITE, 780),
        Material::Popover => (RAISE_DARK, 780),
        Material::Sheet if light => (SURFACE, 820),
        Material::Sheet => (PAPER_DARK, 780),
        Material::Toast if light => (SURFACE, 800),
        Material::Toast => (PAPER_DARK, 740),
        Material::Osd if light => (SURFACE, 720),
        // .66 -> .68 (settled 2026-09-24, over blur, as the dark bar and dock).
        Material::Osd => (PAPER_DARK, 680),
        // The widget adds grain over its tint; the grain is the component's, not the recipe's.
        // .50 -> .54 (light, over black) -> .60 (settled 2026-09-24, over blur: held
        // 3.91:1 over black at .54, the worst of the six) -> **.48** (the user's decision,
        // 2026-09-26, "relax the contrast then": design/23-WIDGETS.md section 1.1 M27-M28 fit
        // the reference exactly, the card's own measured near-white at its own measured alpha).
        // At .48 the card's ink is 3.86:1 over black and 16.70:1 over white: it no longer
        // clears the small-text 4.5:1 gate, but the widget's own text is large or bold (the
        // hero and figure numerals, bold city names; large-text guideline is 3:1), which the
        // relaxed gate in `tests/legibility.rs` checks instead (design/03-COLOR.md section 17).
        Material::Widget if light => (WIDGET_NEAR_WHITE, 480),
        // .45 -> .65 (dark, over white) -> .67 (settled 2026-09-24, over blur) -> .55
        // (the contrast-relax pass, 2026-09-26): the dark tint was not measured against the
        // reference, so it takes the smallest alpha that clears the 3:1 large-text floor for
        // every gate `tests/legibility.rs` runs against it, not only the flat tint over black
        // and white (3.30:1 and 16.16:1 at .55) but the Space-tinted widget card over every
        // preset's darkest stop too (worst 3.12:1, preset 2's `#291c1a` over white); .53 clears
        // the flat tint alone but falls to 2.92:1 on that preset, and .54 only reaches 2.997:1.
        Material::Widget => (PAPER_DARK, 550),
    };
    Some((hex, Alpha(alpha)))
}
