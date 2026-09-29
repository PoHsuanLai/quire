//! How wide a sheet is (lock and switcher parts, M11): the settings sheet's 560, or an alert's
//! narrow column, as the reference desktop draws a password prompt.

/// A sheet's width.
#[derive(Debug, Clone, Copy, PartialEq, Eq, Hash, Default)]
pub enum SheetWidth {
    /// `min(560px, 88%)`: a settings sheet, a power menu.
    #[default]
    Regular,
    /// `min(340px, 88%)`: an alert or a password prompt (`PolkitPrompt`). Written
    /// `data-width="narrow"`.
    Narrow,
}

impl SheetWidth {
    /// The `data-width` value: only a narrow sheet carries one, so a regular sheet's markup is
    /// what it was.
    pub(crate) fn attribute(self) -> Option<&'static str> {
        match self {
            SheetWidth::Regular => None,
            SheetWidth::Narrow => Some("narrow"),
        }
    }
}
