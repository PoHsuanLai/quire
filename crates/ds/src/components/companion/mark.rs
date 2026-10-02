//! An app, as the companion's views name it.

use crate::components::content::icon_source::IconSource;

/// An app a view mentions: its icon and its name. The view of the wire's app name, with the
/// icon the shell resolved for it.
#[derive(Debug, Clone, PartialEq, Eq, Hash)]
pub struct AppMark {
    /// The app's icon.
    pub icon: IconSource,
    /// The app's name as the person reads it.
    pub name: String,
}
