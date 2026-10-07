//! An alert's vocabulary and the rules that decide from it (design/30 section 2.5, `NSAlert`):
//! what its buttons are, which one Return presses and which one Escape does.

use crate::root::common::Common;
use dioxus::prelude::EventHandler;
use ds_core::vocab::Check;
use ds_core::word::Word;

/// What an alert is about: `data-style` (`NSAlert.Style`).
#[derive(Debug, Clone, Copy, PartialEq, Eq, Hash, Default, Word)]
pub enum AlertStyle {
    /// A plain question or notice.
    #[default]
    Informational,
    /// Something the person should think twice about.
    Warning,
    /// Something that cannot be undone or has gone wrong.
    Critical,
}

/// What a button means to the alert.
#[derive(Debug, Clone, Copy, PartialEq, Eq, Hash, Default, Word)]
pub enum AlertRole {
    /// An ordinary action.
    #[default]
    Normal,
    /// An action that destroys something ("Delete", "Erase"): its label is drawn in the danger
    /// red, and it is never the default, so a reflexive Return never destroys anything.
    Destructive,
    /// Backs out: Escape presses it.
    Cancel,
}

/// One of an alert's buttons. The first is the default (`NSAlert` order) unless it is
/// destructive; the alert draws it last in a row, or first in a column.
#[derive(Debug, Clone, PartialEq)]
pub struct AlertButton {
    /// Its label.
    pub label: String,
    /// What it means.
    pub role: AlertRole,
    /// Hears the button's press.
    pub onpress: EventHandler<()>,
    /// The button's own `id`, `data-*` and class, so a host or a test can name which one it is.
    /// Its `mounted` is the alert's, which keeps the keyboard among the buttons.
    pub common: Common,
}

impl AlertButton {
    /// A button with `label` and `role`.
    pub fn new(label: impl Into<String>, role: AlertRole, onpress: EventHandler<()>) -> Self {
        AlertButton {
            label: label.into(),
            role,
            onpress,
            common: Common::default(),
        }
    }

    /// This button with `common` (its `id`, `data-*` and class) on its element.
    pub fn with_common(self, common: Common) -> Self {
        AlertButton { common, ..self }
    }
}

/// The suppression checkbox under an alert's message ("Do not show this again").
#[derive(Debug, Clone, PartialEq)]
pub struct Suppression {
    /// Its label.
    pub label: String,
    /// Whether it is checked.
    pub value: Check,
    /// Hears the person's change.
    pub onchange: EventHandler<Check>,
}

/// How the buttons lie: side by side, or one above the next.
#[derive(Debug, Clone, Copy, PartialEq, Eq, Hash, Word)]
pub enum FooterLayout {
    /// One or two buttons, side by side at equal width, the default on the right.
    Row,
    /// Three or more, stacked full width, the default on top.
    Stack,
}

impl FooterLayout {
    /// The layout for `count` buttons.
    pub fn of(count: usize) -> Self {
        if count > 2 {
            FooterLayout::Stack
        } else {
            FooterLayout::Row
        }
    }
}

/// The button Return presses and the keyboard starts on: the first that is not destructive.
pub fn default_button(roles: &[AlertRole]) -> Option<usize> {
    roles
        .iter()
        .position(|&role| role != AlertRole::Destructive)
}

/// The button Escape presses: the first that cancels.
pub fn escape_button(roles: &[AlertRole]) -> Option<usize> {
    roles.iter().position(|&role| role == AlertRole::Cancel)
}

/// The button Tab or Shift+Tab lands on from `from`, wrapping: a modal alert keeps the keyboard
/// among its own buttons.
pub fn tab_target(count: usize, from: usize, backwards: bool) -> usize {
    if backwards {
        (from + count - 1) % count.max(1)
    } else {
        (from + 1) % count.max(1)
    }
}

#[cfg(test)]
mod tests {
    use super::{AlertRole, FooterLayout, default_button, escape_button, tab_target};
    use AlertRole::{Cancel, Destructive, Normal};

    #[test]
    fn return_is_never_destructive_and_escape_cancels() {
        // (roles, default, escape)
        let cases: &[(&[AlertRole], Option<usize>, Option<usize>)] = &[
            (&[Normal, Cancel], Some(0), Some(1)),
            (&[Destructive, Cancel], Some(1), Some(1)),
            (&[Destructive, Destructive], None, None),
            (&[Normal], Some(0), None),
            (&[Normal, Normal, Cancel], Some(0), Some(2)),
        ];
        for &(roles, default, escape) in cases {
            assert_eq!(default_button(roles), default, "{roles:?}");
            assert_eq!(escape_button(roles), escape, "{roles:?}");
        }
    }

    #[test]
    fn a_footer_stacks_from_three_buttons() {
        let cases = [
            (1, FooterLayout::Row),
            (2, FooterLayout::Row),
            (3, FooterLayout::Stack),
        ];
        for (count, want) in cases {
            assert_eq!(FooterLayout::of(count), want, "{count}");
        }
    }

    #[test]
    fn tab_wraps_in_both_directions() {
        // (count, from, backwards, want)
        let cases = [
            (2, 0, false, 1),
            (2, 1, false, 0),
            (2, 0, true, 1),
            (3, 0, true, 2),
            (3, 2, false, 0),
            (1, 0, false, 0),
        ];
        for (count, from, backwards, want) in cases {
            assert_eq!(tab_target(count, from, backwards), want, "{count} {from}");
        }
        assert_eq!(AlertRole::default(), AlertRole::Normal);
    }
}
