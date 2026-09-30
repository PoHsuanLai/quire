//! EdgePeek: a sidebar that may be hidden, and reveals itself at the window's edge (design/30
//! section 2.11, design/06-INTERACTIONS.md section 7). Pinned it is the sidebar in its column.
//! Hidden, a 10 px strip on the left edge waits for the pointer: after the `Label` profile's
//! intent delay ([`HoverIntent`](ds_motion::hover_intent::HoverIntent)) the sidebar floats
//! over the content and slides in from the left by a spring; when the pointer leaves it and the strip it slides back out. A click on the strip
//! pins it (`onpin`). The panel renders in place, not through the overlay host, because it is
//! the sidebar itself, so its children stay mounted whether it is pinned, peeking or hidden.

use crate::components::app::hover_open::use_hover_open;
use crate::root::common::Common;
use dioxus::prelude::*;
use ds_core::vocab::Shown;
use ds_core::word::Word;
use ds_motion::anim::Anim;
use ds_motion::hover_intent::HoverProfile;
use ds_motion::presence::spring::use_spring_presence;

/// Where the sidebar stands (`data-side`).
#[derive(Debug, Clone, Copy, PartialEq, Eq, Hash, Word)]
pub enum SideState {
    /// Pinned: in its column.
    Shown,
    /// Hidden, and nothing peeking.
    Hidden,
    /// Hidden, floating over the content while the pointer is at the edge.
    Peek,
}

/// What `data-side` says, from the sidebar's pin and whether the pointer has it out.
fn side_state(pinned: Shown, peeking: Shown) -> SideState {
    match (pinned, peeking) {
        (Shown::Visible, _) => SideState::Shown,
        (Shown::Hidden, Shown::Visible) => SideState::Peek,
        (Shown::Hidden, Shown::Hidden) => SideState::Hidden,
    }
}

/// A sidebar that is `pinned` (`Visible`, in its column) or hidden with an edge that peeks it.
/// `onpin` hears a click on the edge strip. `label` names the sidebar; `common` goes on it.
#[component]
pub fn EdgePeek(
    label: String,
    pinned: Shown,
    onpin: EventHandler<()>,
    #[props(default)] common: Common,
    children: Element,
) -> Element {
    let hover = use_hover_open(HoverProfile::Label);
    let peeking = hover.shown();
    let wanted = match pinned {
        Shown::Visible => Shown::Hidden,
        Shown::Hidden => peeking,
    };
    let slide = use_spring_presence(Some(wanted), None, Anim::SlideR);
    let side = match (side_state(pinned, peeking), slide.drawn()) {
        (SideState::Hidden, true) => SideState::Peek,
        (state, _) => state,
    };
    let class = common.class("ds-side");
    let data = common.data_attributes();
    let name = common.aria_label.clone().unwrap_or(label);
    rsx! {
        if pinned == Shown::Hidden {
            div {
                class: "ds-edge",
                "aria-hidden": "true",
                onpointerenter: move |_| hover.over(),
                onpointerleave: move |_| hover.out(),
                onclick: move |_| {
                    hover.reset();
                    onpin.call(());
                },
            }
        }
        nav {
            id: common.id.clone(),
            class,
            "aria-label": "{name}",
            "data-side": side.slug(),
            "data-drive": slide.drive(),
            style: (pinned == Shown::Hidden).then(|| slide.style()),
            onmounted: move |event| common.mounted(event),
            onpointerenter: move |_| hover.enter_card(),
            onpointerleave: move |_| hover.leave_card(),
            ..data,
            {children}
        }
    }
}

#[cfg(test)]
mod tests {
    use super::{SideState, side_state};
    use ds_core::vocab::Shown::{Hidden, Visible};

    #[test]
    fn a_pinned_sidebar_is_shown_whatever_the_pointer_does() {
        // (pinned, pointer has it out, state)
        const CASES: &[(ds_core::vocab::Shown, ds_core::vocab::Shown, SideState)] = &[
            (Visible, Visible, SideState::Shown),
            (Visible, Hidden, SideState::Shown),
            (Hidden, Visible, SideState::Peek),
            (Hidden, Hidden, SideState::Hidden),
        ];
        for &(pinned, peeking, want) in CASES {
            assert_eq!(side_state(pinned, peeking), want);
        }
    }
}
