//! A token's resolved value for Rust geometry: a custom property as an element's computed style
//! holds it after the cascade (the quire sheet, the look, the person's own sheet), with every
//! `var()` substituted. Tokens are declared on `.ds` and inherit, so any element inside a
//! surface sees them with their overrides; a dock's gap or a menu's row height that a look may
//! move is read here, not from a settings key.
//!
//! The sheets reach the document as a `<style>` whose text follows the `Styles` signal (a look
//! or a style sheet change restyles the document on the next frame); ds-blitz has no event of
//! its own for it. A caller reads again on the frame after that signal changes and never caches
//! a value across it.

use blitz_dom::{Atom, BaseDocument, NodeId};
use dioxus::prelude::*;
use dioxus_native_dom::NodeHandle;
use ds::prelude::Px;
use std::rc::Rc;

/// Reads of resolved custom properties on one node of a document.
#[derive(Debug)]
pub struct Resolved;

impl Resolved {
    /// `name` (`--dock-gap`) on `node` as a length in logical pixels: `Some` only for a plain
    /// `<number>px` value, `None` for a missing property, another unit, or anything not a single
    /// length (a token written as `calc()` is not evaluated, so a geometry token is a plain
    /// length).
    pub fn length(doc: &BaseDocument, node: NodeId, name: &str) -> Option<Px> {
        let text = Self::text(doc, node, name)?;
        let number = text.strip_suffix("px")?;
        parse_number(number).map(Px)
    }

    /// `name` (`--ratio`) on `node` as a plain `<number>`: `None` for a missing property or a
    /// value with a unit. A duration is not a number here.
    pub fn number(doc: &BaseDocument, node: NodeId, name: &str) -> Option<f32> {
        parse_number(&Self::text(doc, node, name)?)
    }

    /// The substituted text of the property on `node`, trimmed.
    fn text(doc: &BaseDocument, node: NodeId, name: &str) -> Option<String> {
        let key = Atom::from(name.strip_prefix("--").filter(|rest| !rest.is_empty())?);
        let styles = doc.get_node(node)?.primary_styles()?;
        let properties = styles.custom_properties();
        let value = properties
            .inherited
            .get(&key)
            .or_else(|| properties.non_inherited.get(&key))?;
        Some(value.as_universal()?.css_text().trim().to_owned())
    }
}

/// A component's reader of the tokens its own element sees: give [`Tokens::mounted`] to the
/// element's `onmounted`, then read lengths and numbers when the geometry needs them (a layer
/// size, a hit test, magnification). Before the element mounts, or while the document is
/// borrowed, a read is `None` and the caller keeps its shipped constant.
#[derive(Clone, Copy, PartialEq)]
pub struct Tokens {
    element: Signal<Option<Rc<MountedData>>>,
}

impl std::fmt::Debug for Tokens {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        f.debug_struct("Tokens").finish_non_exhaustive()
    }
}

/// The reader for the calling component's element.
pub fn use_tokens() -> Tokens {
    Tokens {
        element: use_signal(|| None),
    }
}

impl Tokens {
    /// Remember the element whose tokens this reads.
    pub fn mounted(mut self, event: MountedEvent) {
        self.element.set(Some(event.data()));
    }

    /// `name` as a length on the element ([`Resolved::length`]).
    pub fn length(&self, name: &str) -> Option<Px> {
        self.read(|doc, node| Resolved::length(doc, node, name))
    }

    /// `name` as a plain number on the element ([`Resolved::number`]).
    pub fn number(&self, name: &str) -> Option<f32> {
        self.read(|doc, node| Resolved::number(doc, node, name))
    }

    fn read<T>(&self, answer: impl FnOnce(&BaseDocument, NodeId) -> Option<T>) -> Option<T> {
        let element = self.element.peek().clone()?;
        let handle = element.downcast::<NodeHandle>()?;
        let doc = handle.try_doc()?;
        answer(&doc, handle.node_id())
    }
}

/// A finite decimal with no unit.
fn parse_number(text: &str) -> Option<f32> {
    text.trim()
        .parse::<f32>()
        .ok()
        .filter(|value| value.is_finite())
}
