//! The root's typeface, as context: `Ds` provides it, and every nested `.ds` scope (`Surface`)
//! stamps it again, because the stylesheet declares the family tokens on every `.ds` and a
//! nested scope without `data-typeface` would reset to the default.

use crate::appearance::Typeface;
use dioxus::prelude::*;

/// The enclosing root's typeface, as `Ds` provides it.
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
struct RootTypeface(Signal<Typeface>);

/// Provide `typeface` to the subtree, updating it when it changes.
///
/// A nested root provides its own context rather than writing to the enclosing one.
pub(crate) fn use_typeface_provider(typeface: Typeface) {
    let RootTypeface(mut provided) = use_context_provider(|| RootTypeface(Signal::new(typeface)));
    if *provided.peek() != typeface {
        provided.set(typeface);
    }
}

/// The enclosing root's typeface; [`Typeface::System`] outside a `Ds`.
pub fn use_typeface() -> Typeface {
    use_hook(try_consume_context::<RootTypeface>)
        .map_or(Typeface::default(), |RootTypeface(typeface)| typeface())
}
