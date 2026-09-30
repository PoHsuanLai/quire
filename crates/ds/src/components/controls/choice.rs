//! Choice: one entry of an option list (design/30 section 1.7): the shape a segmented control, a
//! radio group and a pop-up button all take, so a caller builds its options once.

use crate::components::content::icon_source::IconSource;
use crate::components::content::text_runs::TextLine;
use ds_core::vocab::Availability;
use ds_core::word::Word;
use ds_style::appearance::accent::Accent;
use ds_style::css::accents_css::swatch_var;

/// One option: the value it stands for, its label, an optional image and whether it can be
/// picked.
#[derive(Debug, Clone, PartialEq)]
pub struct Choice<T> {
    /// What picking it yields.
    pub value: T,
    /// What it says.
    pub label: TextLine,
    /// An image beside or above the label (a segment or a radio button with a picture).
    pub icon: Option<IconSource>,
    /// A colour circle standing for the option, as the name of a custom property
    /// (`--swatch-blue`): a `RadioGroup` with `Arrangement::Swatches` draws it in place of the
    /// indicator and the words, which then only name the button.
    pub swatch: Option<String>,
    /// Whether it can be picked; a disabled one is shown and passed over by the arrows.
    pub availability: Availability,
}

impl<T> Choice<T> {
    /// `value` labelled `label`, enabled, with no image.
    pub fn new(value: T, label: impl Into<TextLine>) -> Self {
        Choice {
            value,
            label: label.into(),
            icon: None,
            swatch: None,
            availability: Availability::Enabled,
        }
    }

    /// The same choice with `icon`.
    pub fn with_icon(self, icon: impl Into<IconSource>) -> Self {
        Choice {
            icon: Some(icon.into()),
            ..self
        }
    }

    /// The same choice drawn as a colour circle painted with the custom property `var`.
    pub fn with_swatch(self, var: impl Into<String>) -> Self {
        Choice {
            swatch: Some(var.into()),
            ..self
        }
    }

    /// The same choice, `availability`.
    pub fn with_availability(self, availability: Availability) -> Self {
        Choice {
            availability,
            ..self
        }
    }

    /// One enabled choice for each `(value, label)` pair, in order.
    pub fn pairs<L: Into<TextLine>>(pairs: impl IntoIterator<Item = (T, L)>) -> Vec<Choice<T>> {
        pairs
            .into_iter()
            .map(|(value, label)| Choice::new(value, label))
            .collect()
    }
}

impl Choice<Accent> {
    /// `accent`, named by its label and drawn as its own colour circle.
    pub fn accent(accent: Accent) -> Self {
        Choice::new(accent, accent.label()).with_swatch(swatch_var(accent))
    }
}

#[cfg(test)]
mod tests {
    use super::Choice;
    use crate::components::content::text_runs::TextLine;
    use ds_core::vocab::Availability;

    #[test]
    fn pairs_become_enabled_choices_in_order() {
        let choices = Choice::pairs([(1, "one"), (2, "two")]);
        assert_eq!(
            choices,
            [
                Choice::new(1, TextLine::from("one")),
                Choice::new(2, TextLine::from("two"))
            ]
        );
        assert!(
            choices
                .iter()
                .all(|choice| choice.availability == Availability::Enabled && choice.icon.is_none())
        );
    }

    #[test]
    fn a_choice_can_be_disabled() {
        let choice = Choice::new('a', "A").with_availability(Availability::Disabled);
        assert_eq!(choice.availability, Availability::Disabled);
    }
}
