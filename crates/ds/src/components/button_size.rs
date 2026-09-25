//! A button's size, apart from its variant (sill Q92): a power menu sets Cancel (Secondary),
//! Shut Down (Primary) and Restart (Danger) side by side, and Danger's own size is the Mini's, a
//! row action's, which sat a size smaller than its neighbours.

use crate::components::vocab::Availability;

/// How big a button is drawn. A button given no size keeps its variant's own: Regular for
/// Primary, Secondary, Quiet and Frame, Mini for Mini and Danger (design/04-COMPONENTS.md
/// section 1), so no existing button changes.
#[derive(Debug, Clone, Copy, PartialEq, Eq, Hash, Default)]
pub enum ButtonSize {
    /// Primary's geometry: 8 x 14 padding, `--fs-control` at 700. `data-size="regular"`.
    #[default]
    Regular,
    /// Mini's geometry: 5 x 10 padding, `--fs-small` at 600, one line. `data-size="mini"`.
    Mini,
}

impl ButtonSize {
    /// The `data-size` word.
    pub(crate) fn slug(self) -> &'static str {
        match self {
            ButtonSize::Regular => "regular",
            ButtonSize::Mini => "mini",
        }
    }
}

/// What a button's availability writes on the element besides `aria-disabled`: `disabled`,
/// so the platform neither focuses nor activates it (sill Q93). Present only when disabled, as
/// an attribute string: a `bool` attribute reaches dioxus-native as `disabled="false"` on every
/// enabled button, which Blitz reads as disabled (a click then no longer toggles an enclosing
/// `<details>`).
pub(crate) fn disabled(availability: Availability) -> Option<&'static str> {
    match availability {
        Availability::Enabled => None,
        Availability::Disabled => Some("true"),
    }
}

#[cfg(test)]
mod tests {
    use super::{ButtonSize, disabled};
    use crate::components::vocab::Availability;

    #[test]
    fn each_size_has_its_word() {
        assert_eq!(ButtonSize::default().slug(), "regular");
        assert_eq!(ButtonSize::Mini.slug(), "mini");
    }

    #[test]
    fn only_a_disabled_button_is_disabled() {
        assert_eq!(disabled(Availability::Disabled), Some("true"));
        assert_eq!(disabled(Availability::Enabled), None);
    }
}
