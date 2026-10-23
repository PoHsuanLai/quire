//! The control a key kind gets.

use ds_settings::schema::{KeyKind, TogglePair};

/// The control a key gets.
#[derive(Debug, Clone, PartialEq, Eq)]
pub(crate) enum Control {
    Switch(TogglePair),
    /// Two short choices, side by side.
    Segments(Vec<String>),
    /// Three or more choices, or a menu: a pop-up button showing the chosen one, as the Mac
    /// draws any choice whose segments would squeeze the row's label.
    PopUp(Vec<String>),
    /// A kind this page does not draw yet: the value, read-only.
    Shown,
}

/// What draws `kind`. A provided key (`KeyKind::Provided`) is read-only here until the caller
/// that knows what is installed fills its choices (`KeySpec::offering`), which makes it a menu.
pub(crate) fn control_of(kind: &KeyKind) -> Control {
    match kind {
        KeyKind::Toggle { variants } => {
            TogglePair::of(variants).map_or_else(|| choice(variants.to_vec()), Control::Switch)
        }
        KeyKind::Segmented { variants } => choice(variants.clone()),
        KeyKind::Menu { variants } => Control::PopUp(variants.clone()),
        KeyKind::Fixed { .. }
        | KeyKind::Bounded { .. }
        | KeyKind::Text
        | KeyKind::Colour
        | KeyKind::Shortcut
        | KeyKind::Provided { .. }
        | KeyKind::List(_)
        | KeyKind::Rows { .. }
        | KeyKind::Live { .. } => Control::Shown,
    }
}

/// The control for a choice between `words`: segments for two, a pop-up for more.
fn choice(words: Vec<String>) -> Control {
    match words.len() {
        0..=2 => Control::Segments(words),
        _ => Control::PopUp(words),
    }
}

#[cfg(test)]
#[path = "control_tests.rs"]
mod tests;
