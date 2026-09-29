//! What the host found for a selector.

use dioxus::prelude::MountedData;
use std::rc::Rc;

/// One attempt at finding an element.
#[derive(Clone)]
pub enum Found {
    /// The first element that matches, as a handle the host's focus writes accept.
    Element(Rc<MountedData>),
    /// Nothing matches (yet).
    Missing,
    /// The document is busy (rendering); try again next frame.
    Busy,
    /// Not a selector the host's document can read.
    BadSelector,
}

impl std::fmt::Debug for Found {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        f.write_str(match self {
            Found::Element(_) => "Element",
            Found::Missing => "Missing",
            Found::Busy => "Busy",
            Found::BadSelector => "BadSelector",
        })
    }
}
