//! PathControl: a path as a row of segments (Finder's path bar, design/30 section 4). Each
//! segment is a glyph and a name, a chevron between neighbours; a click on one reports its index.
//! A long path collapses its middle segments behind a "…" that opens them as a menu. The last
//! segment is where the path ends; it is drawn in the full ink and still reports a click.
//!
//! A segment is a place that takes a drop: the owner drives `PathSegment::drop` from its drag as
//! it does a `Row`'s, and the control reports the pointer coming, going and being released on
//! the segment, each with its index (`CollectionView` hears tiles the same way).
//!
//! Markup: `nav.ds-path-control[aria-label]` of `button.ds-path-segment[data-drop]` (a glyph and
//! `span.ds-path-name`), `span.ds-path-chevron` and `button.ds-path-more`.

use crate::components::chrome::path_control::model::{
    DEFAULT_ROOM, PathPart, PathSegment, parts, rows,
};
use crate::components::content::icon_source::IconSource;
use crate::components::content::icon_view::IconView;
use crate::components::lists::collection_view::landing::{Drops, Landing};
use crate::components::menus::menu::menu::Menu;
use crate::components::menus::menu::placement::MenuPlacement;
use crate::host::measure::{Anchor, MountedRef};
use crate::root::common::Common;
use dioxus::prelude::*;
use ds_core::vocab::Shown;
use ds_style::icon::{Icon, IconSize};

/// The button for `segment`, at index `at` of a path of `len`.
fn draw(
    at: usize,
    len: usize,
    segment: &PathSegment,
    drops: Drops,
    onpick: EventHandler<usize>,
) -> Element {
    let landing = Landing::of(at, segment.drop, drops);
    let current = (at + 1 == len).then_some("location");
    rsx! {
        button {
            class: "ds-path-segment",
            r#type: "button",
            "aria-current": current,
            "data-drop": segment.drop.drop_attr(),
            onpointerenter: move |_| {
                if let Some(landing) = landing {
                    landing.enter();
                }
            },
            onpointerleave: move |_| {
                if let Some(landing) = landing {
                    landing.leave();
                }
            },
            onpointerup: move |_| {
                if let Some(landing) = landing {
                    landing.release();
                }
            },
            onclick: move |_| {
                if !landing.is_some_and(Landing::lit) {
                    onpick.call(at);
                }
            },
            IconView { source: IconSource::from(segment.icon), size: IconSize::Compact }
            span { class: "ds-path-name", "{segment.label}" }
        }
    }
}

fn chevron() -> Element {
    rsx! {
        span { class: "ds-path-chevron", "aria-hidden": "true",
            IconView { source: IconSource::from(Icon::ChevronRight), size: IconSize::Micro }
        }
    }
}

/// A path bar over `segments`, the root first. `room` is how many places the band has before the
/// middle segments collapse (5 by default; see `parts`). `onpick` hears a click on a segment, or
/// on a collapsed one in the menu, with its index in `segments`. `ondropenter`, `ondropleave`
/// and `ondrop` hear the pointer over a segment whose `drop` is `Accepts` or `Target`; a segment
/// behind the collapse mark takes no drop.
#[component]
pub fn PathControl(
    label: String,
    segments: Vec<PathSegment>,
    #[props(default = DEFAULT_ROOM)] room: usize,
    onpick: EventHandler<usize>,
    #[props(default)] ondropenter: Option<EventHandler<usize>>,
    #[props(default)] ondropleave: Option<EventHandler<usize>>,
    #[props(default)] ondrop: Option<EventHandler<usize>>,
    #[props(default)] common: Common,
) -> Element {
    let mut open = use_signal(|| Shown::Hidden);
    let mut more = use_signal(|| None::<MountedRef>);
    let drops = Drops {
        enter: ondropenter,
        leave: ondropleave,
        drop: ondrop,
    };
    let len = segments.len();
    let places = parts(len, room);
    let mounted = common.clone();
    let data = common.data_attributes();
    let anchor = more().map(Anchor::Mounted);
    let hidden = places.iter().find_map(|place| match place {
        PathPart::Collapsed(span) => Some(span.clone()),
        PathPart::Segment(_) => None,
    });
    rsx! {
        nav {
            id: common.id.clone(),
            class: common.class("ds-path-control"),
            "aria-label": label,
            onmounted: move |event| mounted.mounted(event),
            ..data,
            for (place , part) in places.iter().enumerate() {
                if place > 0 {
                    {chevron()}
                }
                match part {
                    PathPart::Segment(at) => rsx! {
                        {draw(*at, len, &segments[*at], drops, onpick)}
                    },
                    PathPart::Collapsed(_) => rsx! {
                        button {
                            class: "ds-path-more",
                            r#type: "button",
                            "aria-label": "Show hidden folders",
                            "aria-expanded": open().aria(),
                            onclick: move |_| open.set(open().flipped()),
                            onmounted: move |event| more.set(Some(MountedRef(event.data()))),
                            "…"
                        }
                    },
                }
            }
        }
        if open() == Shown::Visible && let (Some(anchor), Some(span)) = (anchor, hidden) {
            Menu::<usize> {
                placement: MenuPlacement::Popup,
                anchor,
                items: rows(&segments[span.clone()], span.start),
                onpick: move |at: usize| {
                    open.set(Shown::Hidden);
                    onpick.call(at);
                },
                onclose: move |()| open.set(Shown::Hidden),
            }
        }
    }
}
