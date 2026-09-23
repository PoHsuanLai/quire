//! HoverStrip: a pill of icon buttons that appears on a hovered row, each previewing its
//! result through a Fly tooltip (design/04-COMPONENTS.md section 17).

use crate::components::vocab::{Here, StaggerIndex};
use crate::geometry::Rect;
use crate::geometry::measure::client_rect;
use crate::icon::Icon;
use crate::icon::render::{Glyph, IconSize};
use dioxus::prelude::*;
use std::rc::Rc;

/// Which action a strip button is, by the consumer's own name: `archive`, `snooze`.
#[derive(Debug, Clone, PartialEq, Eq, Hash, PartialOrd, Ord)]
pub struct ActionId(pub String);

/// One strip button.
#[derive(Debug, Clone, PartialEq)]
pub struct StripAction {
    /// Which action, written as `data-op`.
    pub id: ActionId,
    /// Its glyph.
    pub icon: Icon,
    /// Its `aria-label`.
    pub label: String,
    /// What the Fly preview says: "Archive → out of Inbox".
    pub fly: String,
    /// Destination preview: `Current` while hovered, `Elsewhere` on leave.
    pub onhover: Option<EventHandler<Here>>,
    /// Pressed; the button's rect anchors the snooze and label menus.
    pub onclick: EventHandler<Rect>,
}

/// A row's action strip. It shows while its row is hovered; the buttons pop in staggered by
/// `--j` (capped like every stagger), and each one's Fly label appears after `--d-fly`, at once
/// while hover is warm.
#[component]
pub fn HoverStrip(actions: Vec<StripAction>) -> Element {
    rsx! {
        div { class: "ds-strip",
            for (j, action) in actions.into_iter().enumerate() {
                StripButton { key: "{action.id.0}", action, j: StaggerIndex::new(j) }
            }
        }
    }
}

/// One strip button: its own component so it can hold its mounted element for the rect.
#[component]
fn StripButton(action: StripAction, j: StaggerIndex) -> Element {
    let mut element = use_signal(|| None::<Rc<MountedData>>);
    let StripAction {
        id,
        icon,
        label,
        fly,
        onhover,
        onclick,
    } = action;
    let j = j.get();
    rsx! {
        button {
            r#type: "button",
            class: "ds-icon-button",
            "data-variant": "strip",
            "data-op": "{id.0}",
            style: "--j:{j}",
            "aria-label": "{label}",
            onmounted: move |event| element.set(Some(event.data())),
            onpointerenter: move |_| {
                if let Some(onhover) = onhover {
                    onhover.call(Here::Current);
                }
            },
            onpointerleave: move |_| {
                if let Some(onhover) = onhover {
                    onhover.call(Here::Elsewhere);
                }
            },
            onclick: move |_| {
                // Any click clears the destination preview (design/06-INTERACTIONS.md
                // section 14).
                if let Some(onhover) = onhover {
                    onhover.call(Here::Elsewhere);
                }
                if let Some(mounted) = element() {
                    spawn(async move {
                        if let Some(measured) = client_rect(&mounted).await {
                            onclick.call(measured);
                        }
                    });
                }
            },
            Glyph { icon, size: IconSize::Compact }
            span { class: "ds-fly", role: "tooltip", "{fly}" }
        }
    }
}
