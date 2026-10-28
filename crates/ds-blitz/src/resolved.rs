//! A token's resolved value for Rust geometry: a custom property as an element's computed style
//! holds it after the cascade (the quire sheet, the look, the person's own sheet), with every
//! `var()` substituted. Tokens are declared on `.ds` and inherit, so any element inside a
//! surface sees them with their overrides; a dock's gap or a menu's row height that a look may
//! move is read here, not from a settings key.
//!
//! The sheets reach the document as a `<style>` whose text follows the `Styles` signal (a look
//! or a style sheet change restyles the document on the next frame). A component that sizes
//! itself from tokens uses [`use_resolved`]: the frame phase reads them after every resolved
//! frame, outside render, and sets the component's signal only when the reading changed, so a
//! restyle re-renders it with the new values and nothing waits on a timer.
//!
//! [`Tokens`] reads on demand instead, for a read in an event handler or a task. It answers
//! `None` while the document is borrowed, which is always the case during a component's render.

use crate::phase::Phase;
use blitz_dom::{Atom, BaseDocument, NodeId};
use dioxus::prelude::*;
use dioxus_native_dom::NodeHandle;
use ds::host::phase::Watch;
use ds::prelude::Px;
use ds::style::task::try_set;
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

/// A component's on-demand reader of the tokens its own element sees: give [`Tokens::mounted`]
/// to the element's `onmounted`, then read lengths and numbers in an event handler or a task (a
/// hit test, a drag). Before the element mounts, or while the document is borrowed (during any
/// render), a read is `None` and the caller keeps its shipped constant. A value render needs
/// comes from [`use_resolved`].
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

/// The tokens one element sees, for a [`use_resolved`] reader.
#[derive(Clone, Copy)]
pub struct Lookup<'a> {
    doc: &'a BaseDocument,
    node: NodeId,
}

impl Lookup<'_> {
    /// `name` as a length on the element ([`Resolved::length`]).
    pub fn length(&self, name: &str) -> Option<Px> {
        Resolved::length(self.doc, self.node, name)
    }

    /// `name` as a plain number on the element ([`Resolved::number`]).
    pub fn number(&self, name: &str) -> Option<f32> {
        Resolved::number(self.doc, self.node, name)
    }
}

impl std::fmt::Debug for Lookup<'_> {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        f.debug_struct("Lookup")
            .field("node", &self.node)
            .finish_non_exhaustive()
    }
}

/// What a [`use_resolved`] reader last read: give [`ResolvedTokens::mounted`] to the element's
/// `onmounted` and read [`ResolvedTokens::value`] in render.
pub struct ResolvedTokens<T: 'static> {
    value: Signal<Option<T>>,
    watch: Signal<Option<Watch>>,
    phase: CopyValue<Option<Phase>>,
    read: fn(&Lookup) -> T,
}

impl<T> Clone for ResolvedTokens<T> {
    fn clone(&self) -> Self {
        *self
    }
}

impl<T> Copy for ResolvedTokens<T> {}

impl<T> PartialEq for ResolvedTokens<T> {
    fn eq(&self, other: &Self) -> bool {
        self.value == other.value
    }
}

impl<T> std::fmt::Debug for ResolvedTokens<T> {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        f.debug_struct("ResolvedTokens").finish_non_exhaustive()
    }
}

/// A value the calling component builds from the tokens its element sees, kept current by the
/// frame phase: `read` runs after every resolved frame from the element's mount on, and the value
/// changes (re-rendering the component) only when what `read` returns does. `None` until the
/// first frame after mount; the caller keeps its shipped value until then.
pub fn use_resolved<T: PartialEq + Clone + 'static>(read: fn(&Lookup) -> T) -> ResolvedTokens<T> {
    ResolvedTokens {
        value: use_signal(|| None),
        watch: use_signal(|| None),
        phase: use_hook(|| CopyValue::new(try_consume_context::<Phase>())),
        read,
    }
}

impl<T: PartialEq + Clone + 'static> ResolvedTokens<T> {
    /// The value as last read; `None` before the first frame after mount, and without a
    /// document phase (outside a ds-blitz window or harness).
    pub fn value(&self) -> ReadSignal<Option<T>> {
        self.value.into()
    }

    /// Start reading the element `event` names, replacing a reading of an earlier element.
    pub fn mounted(mut self, event: MountedEvent) {
        let Some(phase) = self.phase.peek().clone() else {
            return;
        };
        let (sink, read) = (self.value, self.read);
        let mut last: Option<T> = None;
        let watch = phase.watch_tokens(
            &event.data(),
            Box::new(move |doc, node| {
                let now = read(&Lookup { doc, node });
                if last.as_ref() == Some(&now) {
                    return None;
                }
                last = Some(now.clone());
                Some(Box::new(move || try_set(sink, Some(now)).is_ok()))
            }),
        );
        self.watch.set(watch);
    }
}

/// A finite decimal with no unit.
fn parse_number(text: &str) -> Option<f32> {
    text.trim()
        .parse::<f32>()
        .ok()
        .filter(|value| value.is_finite())
}
