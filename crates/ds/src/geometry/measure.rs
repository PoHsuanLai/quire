//! Measuring a mounted element, the one layout read the design system does.
//!
//! Two phases (spike S9): `get_client_rect` inside `onmounted` returns 0 x 0, and is right only
//! after the next resolve. So the `onmounted` handler only keeps the element; the rect is read
//! on the following frame, never inside the handler.
#![allow(unused_variables)] // Freeze stubs: remove with the last todo!().

use super::units::{Point, Rect};
use dioxus::prelude::*;
use std::rc::Rc;

/// A mounted element, kept so its rect can be read again when the surface moves.
#[derive(Clone)]
pub struct MountedRef(pub Rc<MountedData>);

impl PartialEq for MountedRef {
    fn eq(&self, other: &Self) -> bool {
        Rc::ptr_eq(&self.0, &other.0)
    }
}

impl std::fmt::Debug for MountedRef {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        f.write_str("MountedRef")
    }
}

/// What a floating surface is placed against.
#[derive(Debug, Clone, PartialEq)]
pub enum Anchor {
    /// A point: the caret, a right-click.
    Point(Point),
    /// A rect already known: a button's, a selection's.
    Rect(Rect),
    /// A mounted element, measured when placing.
    Mounted(MountedRef),
}

/// The rect of one mounted element, updated from its `onmounted` event.
#[derive(Debug, Clone, Copy, PartialEq)]
pub struct RectProbe {
    rect: Signal<Option<Rect>>,
    mounted: Signal<Option<MountedRef>>,
}

impl RectProbe {
    /// The last measured rect, or `None` before the element mounted.
    pub fn rect(&self) -> Option<Rect> {
        todo!()
    }

    /// Hand this the element's `onmounted` event. It keeps the element and schedules the read
    /// for the next frame; it does not measure.
    pub fn on_mounted(&self, event: Event<MountedData>) {
        todo!()
    }

    /// The element as an anchor, once mounted.
    pub fn anchor(&self) -> Option<Anchor> {
        todo!()
    }
}

/// A probe for one element's rect.
pub fn use_rect() -> RectProbe {
    todo!()
}
