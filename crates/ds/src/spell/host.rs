//! The host seam an [`EditSurface`](crate::EditSurface) checks spelling through. `ds` stays
//! effect-free: reading dictionaries, checking on a worker thread and writing the user's
//! dictionary are `ds_native::spell`'s (its `spellcheck` feature), which provides
//! [`HostSpell`]. Without one a surface with [`Spell::On`](super::Spell) checks nothing and
//! draws nothing.

use super::lang::Lang;
use super::words::Span;
use crate::edit::host::Probe;
use crate::edit::position::EditNode;
use dioxus::prelude::MountedData;
use std::future::Future;
use std::pin::Pin;
use std::rc::Rc;

/// An answer the host gives later, off the UI thread. Awaited from the surface's own tasks.
pub type SpellFuture<T> = Pin<Box<dyn Future<Output = T>>>;

/// One addressable text element's text, as the surface's positions count it.
#[derive(Debug, Clone, PartialEq, Eq)]
pub struct Paragraph {
    /// The element, by its `data-edit-node`.
    pub node: EditNode,
    /// Its own text: the text nodes whose nearest addressable ancestor it is, in order.
    pub text: String,
    /// Stretches never checked: text inside a `code`, `pre`, `kbd` or `samp` element.
    pub skips: Vec<Span>,
}

/// What became of a learned word.
#[derive(Debug, Clone, Copy, PartialEq, Eq, Hash)]
pub enum Learned {
    /// Written to the user's dictionary: accepted from now on, in every session.
    Saved,
    /// Accepted for this session, but the user's dictionary could not be written.
    SessionOnly,
}

/// A spellchecker. Two implementations: `ds_native::spell`'s (system Hunspell dictionaries on a
/// worker thread) and a test's fake.
pub trait SpellService {
    /// The languages a surface with no language of its own checks in: the locale's.
    fn languages(&self) -> Vec<Lang>;

    /// The addressable text elements inside `surface`, in document order. Busy while the
    /// renderer holds the document.
    fn paragraphs(&self, surface: &MountedData) -> Probe<Vec<Paragraph>>;

    /// Which of `words` no dictionary in `langs` accepts (and none was ignored or learned).
    fn check(&self, langs: Vec<Lang>, words: Vec<String>) -> SpellFuture<Vec<String>>;

    /// Replacements for `word`, best first.
    fn suggest(&self, langs: Vec<Lang>, word: String) -> SpellFuture<Vec<String>>;

    /// Accept `word` everywhere until the process ends ("Ignore Spelling").
    fn ignore(&self, word: String);

    /// Accept `word` in `lang` from now on, and remember it ("Learn Spelling").
    fn learn(&self, lang: Lang, word: String) -> SpellFuture<Learned>;
}

/// The spellchecker a root provides as context (`ds_native::spell::provide`).
#[derive(Clone)]
pub struct HostSpell(pub Rc<dyn SpellService>);

impl std::fmt::Debug for HostSpell {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        f.write_str("HostSpell(..)")
    }
}
