//! A thing an app owns, as the companion's context names it.

/// One thing in an app (a mail thread, an event, a file), reported by the row or list that
/// shows it. The kind and key are the app's own words, opaque here; the title and subtitle are
/// what the person reads.
#[derive(Debug, Clone, PartialEq, Eq, Hash)]
pub struct ThingMark {
    /// What sort of thing it is, dotted and app-prefixed: `mail.thread`.
    pub kind: String,
    /// Which one, in the app's own key.
    pub key: String,
    /// What the row shows as its title.
    pub title: String,
    /// What the row shows under it.
    pub subtitle: String,
}
