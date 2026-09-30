//! A button's size, apart from its variant: a power menu sets Cancel (Secondary),
//! Shut Down (Primary) and Restart (Danger) side by side, and Danger's own size is the Mini's, a
//! row action's, which sat a size smaller than its neighbours.

use ds_core::vocab::Availability;
use ds_core::word::Word;

/// How big a button is drawn. A button given no size keeps its variant's own: Regular for
/// Primary, Secondary, Quiet and Frame, Mini for Mini and Danger (design/04-COMPONENTS.md
/// section 1), so no existing button changes.
#[derive(Debug, Clone, Copy, PartialEq, Eq, Hash, Default, Word)]
pub enum ButtonSize {
    /// Primary's geometry: the ladder's Regular height (22, design/29-SIZING.md), `--fs-control`
    /// at 700. `data-size="regular"`.
    #[default]
    Regular,
    /// Mini's geometry: the ladder's Small height (16, radius 4), `--fs-small` at 600, one line.
    /// `data-size="mini"`.
    Mini,
    /// The ladder's Large height, 28, at Primary's label. `data-size="large"`.
    Large,
}

/// What a button's availability writes on the element besides `aria-disabled`: `disabled`,
/// so the platform neither focuses nor activates it. Present only when disabled, as
/// an attribute string: a `bool` attribute reaches dioxus-native as `disabled="false"` on every
/// enabled button, which Blitz reads as disabled (a click then no longer toggles an enclosing
/// `<details>`).
pub fn disabled(availability: Availability) -> Option<&'static str> {
    match availability {
        Availability::Enabled | Availability::Busy => None,
        Availability::Disabled => Some("true"),
    }
}

#[cfg(test)]
mod tests {
    use super::{ButtonSize, disabled};
    use ds_core::vocab::Availability;
    use ds_core::word::Word;

    #[test]
    fn each_size_has_its_word() {
        assert_eq!(ButtonSize::default().slug(), "regular");
        assert_eq!(ButtonSize::Mini.slug(), "mini");
        assert_eq!(ButtonSize::Large.slug(), "large");
    }

    #[test]
    fn only_a_disabled_button_is_disabled() {
        assert_eq!(disabled(Availability::Disabled), Some("true"));
        assert_eq!(disabled(Availability::Enabled), None);
        assert_eq!(disabled(Availability::Busy), None);
    }
}
