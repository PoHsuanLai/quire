//! What a `Button` is, apart from what it says: its bezel, its role, which window key it answers
//! and whether its label is drawn (design/30 section 2.1, `NSButton`). Data only.

use ds_core::word::Word;

/// The bezel: what the button looks like at rest (`NSButton.BezelStyle`), `data-variant`.
#[derive(Debug, Clone, Copy, PartialEq, Eq, Hash, Default, Word)]
pub enum Bezel {
    /// The push button: a rounded rectangle with a line, at the control size's height.
    #[default]
    Push,
    /// The toolbar button: no bezel at rest, one under the pointer or while pressed. A glyph,
    /// or a glyph and a label.
    Toolbar,
    /// The inline button: words with no bezel, the quiet link-button.
    Inline,
    /// The help button: a round bezel with a question mark, drawn by the button itself.
    Help,
}

/// What the button does to the thing it acts on (`NSButton.hasDestructiveAction`), `data-role`.
#[derive(Debug, Clone, Copy, PartialEq, Eq, Hash, Default, Word)]
pub enum ButtonRole {
    /// An ordinary action.
    #[default]
    Normal,
    /// An action that deletes or discards: red under the pointer, and red as a default button.
    Destructive,
}

/// Which key of the window the button answers, `data-answers`. A dialog routes Return to the
/// button that answers it and Escape to the one that answers Cancel. A button that answers
/// Escape is activated by Space only, leaving Return to the default button; any other focused
/// button answers Return and Space itself.
#[derive(Debug, Clone, Copy, PartialEq, Eq, Hash, Default, Word)]
pub enum Answers {
    /// Neither.
    #[default]
    Nothing,
    /// Return: the default button, drawn in the accent (`NSButton.keyEquivalent`).
    Return,
    /// Escape: the Cancel button.
    Escape,
}

/// When the button takes the keyboard.
#[derive(Debug, Clone, Copy, PartialEq, Eq, Hash, Default)]
pub enum ButtonFocus {
    /// Only when the person or the caller puts it there.
    #[default]
    Manual,
    /// As soon as it is mounted: the default button of a sheet whose first stop is not a field.
    OnMount,
}

/// How a busy button shows it is working (`availability: Busy`), `data-busy`.
#[derive(Debug, Clone, Copy, PartialEq, Eq, Hash, Default, Word)]
pub enum BusyLook {
    /// A spinner in the leading slot (`NSProgressIndicator`).
    #[default]
    Spinner,
    /// No spinner: the button's own icon keeps its glyph and turns continuously, as Mail's Get
    /// Mail arrow does. Under Reduced motion the glyph holds still.
    TurnIcon,
}

/// Whether the button's label is drawn beside its image (`NSButton.imagePosition`),
/// `data-image`.
#[derive(Debug, Clone, Copy, PartialEq, Eq, Hash, Default, Word)]
pub enum ImagePosition {
    /// The image, when there is one, then the label.
    #[default]
    Leading,
    /// The image alone: the label names the button to assistive technology and is not drawn.
    Only,
}

/// How the button's glyph changes when its `icon` does (design/30 section 1.3, Cross-fade):
/// a play button that becomes a pause button fades one into the other, and never pulses.
#[derive(Debug, Clone, Copy, PartialEq, Eq, Hash, Default, Word)]
pub enum IconSwap {
    /// The new glyph is there at once.
    #[default]
    Instant,
    /// The glyphs cross-fade over `--t-quick`; a quire glyph only, anything else swaps at once.
    CrossFade,
}

impl Answers {
    /// `data-answers`, written only for a button that answers a key.
    pub(crate) fn attr(self) -> Option<&'static str> {
        (self != Answers::Nothing).then(|| self.slug())
    }
}

#[cfg(test)]
mod tests {
    use super::{Answers, Bezel, ButtonRole, IconSwap, ImagePosition};
    use ds_core::word::Word;

    #[test]
    fn every_word_parses_back_from_its_slug() {
        for bezel in Bezel::ALL {
            assert_eq!(Bezel::parse(bezel.slug()), Some(*bezel));
        }
        for role in ButtonRole::ALL {
            assert_eq!(ButtonRole::parse(role.slug()), Some(*role));
        }
        for answers in Answers::ALL {
            assert_eq!(Answers::parse(answers.slug()), Some(*answers));
        }
        for image in ImagePosition::ALL {
            assert_eq!(ImagePosition::parse(image.slug()), Some(*image));
        }
        for swap in IconSwap::ALL {
            assert_eq!(IconSwap::parse(swap.slug()), Some(*swap));
        }
    }

    #[test]
    fn only_a_button_that_answers_a_key_says_so() {
        assert_eq!(Answers::Nothing.attr(), None);
        assert_eq!(Answers::Return.attr(), Some("return"));
        assert_eq!(Answers::Escape.attr(), Some("escape"));
    }
}
