//! An alert's vocabulary: how its action reads, and which button is the default.

/// How an alert's action button reads.
#[derive(Debug, Clone, Copy, PartialEq, Eq, Hash, Default)]
pub enum AlertEmphasis {
    /// An ordinary action ("Turn Off"): the accent-filled default button, pressed by Return and
    /// holding the keyboard as the alert opens.
    #[default]
    Default,
    /// An action that destroys something ("Delete", "Erase"): its label is drawn in the danger
    /// red, and it is not the default. Cancel is: accent-filled, pressed by Return, holding the
    /// keyboard as the alert opens, so a reflexive Return never destroys anything (the HIG's
    /// buttons rule: don't give the primary role to a button that performs a destructive action).
    Destructive,
}

/// One of an alert's two buttons.
#[derive(Debug, Clone, Copy, PartialEq, Eq, Hash)]
pub enum AlertButton {
    /// Cancel: Escape and a press on the scrim press it too.
    Cancel,
    /// The action.
    Action,
}

impl AlertEmphasis {
    /// The default button: the one Return presses and the keyboard starts on.
    pub fn default_button(self) -> AlertButton {
        match self {
            AlertEmphasis::Default => AlertButton::Action,
            AlertEmphasis::Destructive => AlertButton::Cancel,
        }
    }

    /// The `data-emphasis` word, written only for a destructive alert.
    pub(crate) fn attribute(self) -> Option<&'static str> {
        match self {
            AlertEmphasis::Default => None,
            AlertEmphasis::Destructive => Some("destructive"),
        }
    }
}

#[cfg(test)]
mod tests {
    use super::{AlertButton, AlertEmphasis};

    #[test]
    fn a_destructive_action_is_never_the_default() {
        let cases = [
            (AlertEmphasis::Default, AlertButton::Action, None),
            (
                AlertEmphasis::Destructive,
                AlertButton::Cancel,
                Some("destructive"),
            ),
        ];
        for (emphasis, button, attribute) in cases {
            assert_eq!(emphasis.default_button(), button, "{emphasis:?}");
            assert_eq!(emphasis.attribute(), attribute, "{emphasis:?}");
        }
    }
}
