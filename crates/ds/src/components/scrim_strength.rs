//! How hard a scrim dims (sheet and modal parts): `--scrim`'s .22 pushes a peek back, which is
//! too light behind a dialog that asks for a decision, such as shutting down.

/// A scrim's strength.
#[derive(Debug, Clone, Copy, PartialEq, Eq, Hash, Default)]
pub enum ScrimStrength {
    /// `--scrim`: behind a peek, a palette, a settings sheet.
    #[default]
    Standard,
    /// `--scrim-modal` (.40 light, .55 dark): behind a modal decision. Written
    /// `data-strength="modal"`.
    Modal,
}

impl ScrimStrength {
    /// The `data-strength` value: only a modal scrim carries one, so a standard scrim's markup is
    /// what it was.
    pub(crate) fn attribute(self) -> Option<&'static str> {
        match self {
            ScrimStrength::Standard => None,
            ScrimStrength::Modal => Some("modal"),
        }
    }
}

#[cfg(test)]
mod tests {
    use super::ScrimStrength;

    #[test]
    fn only_a_modal_scrim_writes_its_strength() {
        assert_eq!(ScrimStrength::default().attribute(), None);
        assert_eq!(ScrimStrength::Modal.attribute(), Some("modal"));
    }
}
