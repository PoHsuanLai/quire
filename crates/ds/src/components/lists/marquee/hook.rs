//! The pointer's side of a rubber band: where a drag from empty space began, where the pointer is
//! now, the selection it makes as it moves, and the scrolling at the view's edges. The view owns
//! the handle (`use_marquee`) and feeds it the pointer's events; what the band touches is the
//! view's to say (rows of a table, tiles of a grid), and the selection it makes goes to the
//! owner's `onmarquee`.
//!
//! A press on an item is not a band: the item says so (`item_pressed`) before the press reaches
//! the view, since a child hears it first. The band scrolls the content a little on each pointer
//! move made near an edge; a pointer held still near the edge does not scroll on its own, as quire
//! keeps no timer for a gesture.

use crate::components::controls::scroller::handle::ScrollerRef;
use crate::components::lists::marquee::model::{MarqueeHow, band, edge_scroll, selected};
use crate::host::event_point::pointer_point;
use crate::host::measure::{RectProbe, use_rect};
use crate::keys::use_platform;
use crate::root::common::Common;
use chordkit::Platform;
use dioxus::html::input_data::MouseButton;
use dioxus::prelude::*;
use ds_core::geometry::units::{Point, Px, Rect};
use ds_core::selection::RowSelection;
use std::ops::Range;

/// A drag in progress, in the scroller's content coordinates.
#[derive(Debug, Clone, PartialEq)]
struct Active {
    /// Where the press began.
    from: Point,
    /// Where the pointer is now.
    to: Point,
    how: MarqueeHow,
    /// The selection the press found.
    base: RowSelection,
    /// The selection last reported to the owner.
    reported: RowSelection,
}

/// Whether a band is being dragged.
#[derive(Debug, Clone, PartialEq)]
enum Drag {
    Idle,
    Active(Active),
}

/// Where the press now reaching the view began.
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
enum Press {
    /// On empty space: a band may begin.
    Space,
    /// On an item: the item's own press.
    Item,
}

/// A view's rubber band: made by [`use_marquee`], handed to the view's pointer handlers and to
/// [`MarqueeBand`](crate::components::lists::marquee::band::MarqueeBand).
#[derive(Clone, Copy, PartialEq)]
pub(crate) struct Marquee {
    drag: Signal<Drag>,
    press: CopyValue<Press>,
    viewport: RectProbe,
    scroller: ScrollerRef,
    platform: Platform,
    mounted: Callback<MountedEvent>,
}

/// The band of the calling view, drawn and scrolled in `scroller`.
pub(crate) fn use_marquee(scroller: ScrollerRef) -> Marquee {
    let viewport = use_rect();
    let mounted = use_callback(move |event: MountedEvent| viewport.on_mounted(event));
    Marquee {
        drag: use_signal(|| Drag::Idle),
        press: use_hook(|| CopyValue::new(Press::Space)),
        viewport,
        scroller,
        platform: use_platform(),
        mounted,
    }
}

impl Marquee {
    /// The `Common` for the view's `Scroller`: its mounted element is the box the band's points
    /// are measured from.
    pub(crate) fn common(self) -> Common {
        Common {
            mounted: Some(self.mounted),
            ..Common::default()
        }
    }

    /// An item heard a press: the press that goes on to the view is the item's, not a band's.
    pub(crate) fn item_pressed(self) {
        let mut press = self.press;
        press.set(Press::Item);
    }

    /// The view heard a press. On empty space inside the scroller, with the primary button, a
    /// band begins at the pointer: with no modifier it replaces the selection, so the selection is
    /// cleared at once and `report` hears it (a click on empty space deselects, as in Finder); with
    /// Shift or the primary modifier it adds to `base`.
    pub(crate) fn pressed(
        self,
        event: &PointerEvent,
        base: RowSelection,
        report: Option<EventHandler<RowSelection>>,
    ) {
        let on_item = *self.press.peek() == Press::Item;
        let mut press = self.press;
        press.set(Press::Space);
        let Some(report) = report else {
            return;
        };
        if on_item || event.trigger_button() != Some(MouseButton::Primary) {
            return;
        }
        let at = pointer_point(event);
        let Some(view) = self.viewport.rect().filter(|view| holds(*view, at)) else {
            return;
        };
        let from = self.content_point(view, at);
        let how = MarqueeHow::of(self.platform, event.modifiers());
        let reported = match how {
            MarqueeHow::Replace => RowSelection::none(),
            MarqueeHow::Add => base.clone(),
        };
        let mut drag = self.drag;
        drag.set(Drag::Active(Active {
            from,
            to: from,
            how,
            base,
            reported: reported.clone(),
        }));
        if how == MarqueeHow::Replace {
            report.call(reported);
        }
    }

    /// The pointer moved. While a band is on, the content scrolls if the pointer is near an edge,
    /// the band follows the pointer, and `report` hears the selection when it changed. `hit` says
    /// which rows or tiles a band touches, in the content's coordinates.
    pub(crate) fn moved(
        self,
        event: &PointerEvent,
        hit: &dyn Fn(Rect) -> Vec<Range<usize>>,
        report: Option<EventHandler<RowSelection>>,
    ) {
        let Drag::Active(active) = self.drag.peek().clone() else {
            return;
        };
        let Some(view) = self.viewport.rect() else {
            return;
        };
        let at = pointer_point(event);
        let delta = edge_scroll(at.y.0 - view.top().0, view.size.height.0);
        if delta.abs() > 0.0 {
            self.scroller.by(Px(delta));
        }
        let to = self.content_point(view, at);
        let selection = selected(&active.base, active.how, &hit(band(active.from, to)));
        let changed = selection != active.reported;
        let mut drag = self.drag;
        drag.set(Drag::Active(Active {
            to,
            reported: selection.clone(),
            ..active
        }));
        if let (true, Some(report)) = (changed, report) {
            report.call(selection);
        }
    }

    /// The pointer was released or left: the band is gone.
    pub(crate) fn end(self) {
        if *self.drag.peek() != Drag::Idle {
            let mut drag = self.drag;
            drag.set(Drag::Idle);
        }
    }

    /// The band now, in the content's coordinates; `None` with no drag.
    pub(crate) fn rect(self) -> Option<Rect> {
        match self.drag.read().clone() {
            Drag::Active(active) => Some(band(active.from, active.to)),
            Drag::Idle => None,
        }
    }

    /// `at` (the window's coordinates) in the scroller's content: from the scroller's corner, and
    /// down by how far it is scrolled.
    fn content_point(self, view: Rect, at: Point) -> Point {
        let scrolled = self.scroller.scroll().read().offset;
        Point {
            x: Px(at.x.0 - view.left().0),
            y: Px(at.y.0 - view.top().0 + scrolled.0),
        }
    }
}

/// Whether `rect` holds `at`.
fn holds(rect: Rect, at: Point) -> bool {
    at.x.0 >= rect.left().0
        && at.x.0 < rect.right().0
        && at.y.0 >= rect.top().0
        && at.y.0 < rect.bottom().0
}
