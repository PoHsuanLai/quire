//! A scroller's scroll state, as a value the owner reads and moves.
//!
//! The model is a [`Scroll`] in a signal. It follows the document: the element's `onscroll`
//! updates the offset the frame it moves (a wheel, a drag), and the host's frame phase publishes
//! the observed state after each layout, which also covers scrolls the document makes without an
//! event. It leads the document only for scrolls the owner asks for: those update the model at
//! once and queue one write for the frame phase, so asking twice for the same place writes once.

use crate::host::document::{DocumentHost, use_document_host};
use crate::host::measure::MountedRef;
use crate::host::phase::{Observe, Observed, PhaseWrite, Watch};
use dioxus::prelude::*;
use ds_core::geometry::scroll::{Scroll, ScrollSpan};
use ds_core::geometry::units::Px;
use std::rc::Rc;

/// The scroll state of one [`Scroller`](crate::components::controls::scroller::view::Scroller),
/// made by [`use_scroller`] in the component that owns the list and handed to the scroller.
#[derive(Debug, Clone, Copy, PartialEq)]
pub struct ScrollerRef {
    scroll: Signal<Scroll>,
    node: Signal<Option<MountedRef>>,
    /// An offset asked for before the element mounted: written once it has.
    asked: Signal<Option<Px>>,
    host: CopyValue<HostRef>,
}

/// The document host, as a value a `Copy` handle can hold.
#[derive(Clone)]
struct HostRef(Rc<dyn DocumentHost>);

impl std::fmt::Debug for HostRef {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        f.write_str("HostRef")
    }
}

/// A scroller's state, observed through the host's frame phase for as long as the calling
/// component lives.
pub fn use_scroller() -> ScrollerRef {
    let host = use_document_host();
    let scroll = use_signal(Scroll::default);
    let node = use_signal(|| None::<MountedRef>);
    let asked = use_signal(|| None::<Px>);
    let seen = use_signal(|| None::<Scroll>);
    let mut watch = use_signal(|| None::<Watch>);
    let handle = ScrollerRef {
        scroll,
        node,
        asked,
        host: use_hook({
            let host = Rc::clone(&host);
            move || CopyValue::new(HostRef(host))
        }),
    };
    use_effect(move || {
        let Some(mounted) = node() else {
            watch.set(None);
            return;
        };
        match host.geometry().observe(&mounted.0, Observe::Scroll(seen)) {
            Observed::Watching(found) => watch.set(Some(found)),
            Observed::Unsupported => watch.set(None),
        }
        handle.flush_asked();
    });
    use_effect(move || {
        if let Some(observed) = seen() {
            let mut scroll = scroll;
            if *scroll.peek() != observed {
                scroll.set(observed);
            }
        }
    });
    handle
}

impl ScrollerRef {
    /// The scroll state: where the content is, what shows, how long it is.
    pub fn scroll(&self) -> ReadSignal<Scroll> {
        self.scroll.into()
    }

    /// Scroll to `offset`, kept in range once the scroller has been measured.
    pub fn scroll_to(&self, offset: Px) {
        let now = *self.scroll.peek();
        let next = match now.viewport.0 > 0.0 {
            true => now.to(offset),
            false => Scroll {
                offset: Px(offset.0.max(0.0)),
                ..now
            },
        };
        if next.offset == now.offset {
            return;
        }
        let mut scroll = self.scroll;
        scroll.set(next);
        self.write(next.offset);
    }

    /// Scroll `delta` further, negative going back.
    pub fn by(&self, delta: Px) {
        let now = *self.scroll.peek();
        self.scroll_to(now.offset + delta);
    }

    /// Scroll the least that shows `span`.
    pub fn reveal(&self, span: ScrollSpan) {
        let now = *self.scroll.peek();
        self.scroll_to(Px(ds_core::geometry::scroll::nearest_scroll(
            now.offset.0,
            now.viewport.0,
            span,
        )));
    }

    /// Scroll the least that shows row `index` of rows `pitch` apart.
    pub fn reveal_row(&self, index: usize, pitch: Px) {
        self.reveal(ScrollSpan {
            start: index as f32 * pitch.0,
            length: pitch.0,
        });
    }

    /// The element mounted.
    pub(crate) fn attach(&self, element: MountedRef) {
        let mut node = self.node;
        node.set(Some(element));
    }

    /// The element scrolled itself to `offset`: the model follows.
    pub(crate) fn heard(&self, offset: Px) {
        let mut scroll = self.scroll;
        if scroll.peek().offset != offset {
            scroll.with_mut(|scroll| scroll.offset = offset);
        }
    }

    /// Queue the write for the frame phase, or keep it until the element mounts.
    fn write(&self, offset: Px) {
        let mut asked = self.asked;
        let Some(mounted) = self.node.peek().clone() else {
            asked.set(Some(offset));
            return;
        };
        asked.set(None);
        self.host
            .peek()
            .0
            .geometry()
            .write(&mounted.0, PhaseWrite::ScrollTo(offset));
    }

    /// Write what was asked for before the element mounted.
    fn flush_asked(&self) {
        let waiting = *self.asked.peek();
        if let Some(offset) = waiting {
            self.write(offset);
        }
    }
}
