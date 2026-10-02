//! The viewport's colour scheme follows the appearance the root resolved: `Ds` writes it as
//! `data-theme` on the outermost `.ds`, and the host copies it to the viewport so the canvas,
//! the user-agent sheet and `prefers-color-scheme` agree with the design system.

use blitz_dom::{BaseDocument, LocalName};
use blitz_traits::shell::ColorScheme;

/// The scheme the outermost `.ds` resolved, if one is mounted.
pub(crate) fn resolved(doc: &BaseDocument) -> Option<ColorScheme> {
    // Selected by class: an unprefixed attribute selector never matches under Blitz (spike S2).
    let root = doc.query_selector(".ds").ok().flatten()?;
    let theme = doc.get_node(root)?.attr(LocalName::from("data-theme"))?;
    Some(match theme {
        "dark" => ColorScheme::Dark,
        _ => ColorScheme::Light,
    })
}

/// Point the viewport at the root's scheme; the scheme it changed to, when it changed.
///
/// A colour-only restyle leaves the boxes as they are: Blitz bakes a text run's colour into its
/// inline layout and an `<svg>`'s `currentColor` into its parsed tree when the box is built, and a
/// `color` change builds neither again (after a switch to Dark the glyphs and text kept their
/// light-scheme colours). So the switch also turns incremental layout off, which makes every
/// resolve rebuild every box, and it stays off while a transition is still running (a box built
/// mid-transition bakes the colour it was built at, not the one the transition ends on). The host
/// and the harness call this once per frame, before the resolve, so the first call after the
/// last transition has ended turns it back on.
pub fn follow_root(doc: &mut BaseDocument) -> Option<ColorScheme> {
    let wanted = resolved(doc)?;
    if doc.viewport().color_scheme == wanted {
        if !doc.incremental_layout() && !doc.is_animating() {
            doc.set_incremental_layout(true);
        }
        return None;
    }
    doc.viewport_mut().color_scheme = wanted;
    doc.set_incremental_layout(false);
    Some(wanted)
}
