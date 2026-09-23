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
pub(crate) fn follow_root(doc: &mut BaseDocument) -> Option<ColorScheme> {
    let wanted = resolved(doc)?;
    if doc.viewport().color_scheme == wanted {
        return None;
    }
    doc.viewport_mut().color_scheme = wanted;
    Some(wanted)
}
