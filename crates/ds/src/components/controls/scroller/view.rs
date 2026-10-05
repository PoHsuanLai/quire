//! Scroller: the container a list or a long column scrolls in, drawn so its scroll state is the
//! owner's `ScrollerRef` (`use_scroller`) rather than something read from the document.

use crate::components::controls::scroller::handle::ScrollerRef;
use crate::host::measure::MountedRef;
use crate::root::common::Common;
use dioxus::prelude::*;
use ds_core::geometry::units::Px;

/// A vertical scroll container over `children`, its scroll state kept in `scroller`: it follows
/// the element's own scrolls and carries out the ones the owner asks `scroller` for.
#[component]
pub fn Scroller(
    scroller: ScrollerRef,
    #[props(default)] common: Common,
    children: Element,
) -> Element {
    let data = common.data_attributes();
    let mounted = common.clone();
    rsx! {
        div {
            class: common.class("ds-scroller"),
            id: common.id.clone(),
            "aria-label": common.aria_label.clone(),
            onmounted: move |event: MountedEvent| {
                scroller.attach(MountedRef(event.data()));
                mounted.mounted(event);
            },
            onscroll: move |event: ScrollEvent| scroller.heard(Px(event.scroll_top() as f32)),
            ..data,
            {children}
        }
    }
}
