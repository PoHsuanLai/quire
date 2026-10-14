//! How wide a sheet is (lock and switcher parts, M11): the settings sheet's 560, a password
//! prompt's narrow column, or an alert's.

/// A sheet's width.
#[derive(Debug, Clone, Copy, PartialEq, Eq, Hash, Default)]
pub enum SheetWidth {
    /// `min(560px, 88%)`: a settings sheet, a power menu.
    #[default]
    Regular,
    /// `min(var(--sheet-narrow-w), 88%)` (340): a password prompt (`PolkitPrompt`). Written
    /// `data-width="narrow"`.
    Narrow,
    /// `min(var(--sheet-alert-w), 88%)` (260): an alert, as the Mac's `NSAlert`. Written
    /// `data-width="alert"`.
    Alert,
    /// `min(1040px, 92%)`: a gallery of cards (Edit Widgets). Written `data-width="wide"`.
    Wide,
}

impl SheetWidth {
    /// The `data-width` value: only a narrow sheet carries one, so a regular sheet's markup is
    /// what it was.
    pub(crate) fn attribute(self) -> Option<&'static str> {
        match self {
            SheetWidth::Regular => None,
            SheetWidth::Narrow => Some("narrow"),
            SheetWidth::Alert => Some("alert"),
            SheetWidth::Wide => Some("wide"),
        }
    }
}
