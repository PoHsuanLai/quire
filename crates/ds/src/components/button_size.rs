//! A button's size, apart from its variant (sill Q92): a power menu sets Cancel (Secondary),
//! Shut Down (Primary) and Restart (Danger) side by side, and Danger's own size is the Mini's, a
//! row action's, which sat a size smaller than its neighbours.

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

#[cfg(test)]
mod tests {
    use super::ButtonSize;

    #[test]
    fn each_size_has_its_word() {
        assert_eq!(ButtonSize::default().slug(), "regular");
        assert_eq!(ButtonSize::Mini.slug(), "mini");
    }
}
