//! How an item leaves: the `data-exit` word and the animation it plays.

use crate::core::word::Word;

/// How an item leaves: `data-exit`.
#[derive(Debug, Clone, Copy, PartialEq, Eq, Hash, Word)]
pub enum Exit {
    /// Archive, restore, wake: `fold`.
    Fold,
    /// Snooze: `curl`.
    Curl,
    /// Trash, delete: `crumple`.
    Crumple,
    /// A Today entry closing: `tab-out`.
    TabOut,
    /// A notification banner leaving: `banner-out`, a slide to the right.
    BannerOut,
}
