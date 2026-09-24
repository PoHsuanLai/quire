//! HoverStrip: a pill of icon buttons that appears on a hovered row, each previewing its
//! result through a Fly tooltip (design/04-COMPONENTS.md section 17).

use crate::components::tooltip::Shown;
use crate::components::vocab::{Expanded, Here, StaggerIndex};
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

/// Whether each strip button carries a `title` naming it: the webview's own tooltip, which a
/// caller that also relies on it (mailo's rows did) asks for. Off by default, so a strip's
/// markup is what it was.
#[derive(Debug, Clone, Copy, PartialEq, Eq, Hash, Default)]
pub enum Titles {
    /// No `title`: the Fly names the action.
    #[default]
    Omitted,
    /// Each button's `title` is its label.
    FromLabel,
}

/// A row's action strip. It shows while its row is hovered; the buttons pop in staggered by
/// `--j` (capped like every stagger), and each one's Fly label appears after `--d-fly`, at once
/// while hover is warm.
///
/// `shown` hands the reveal to the caller: `Some(Shown::Visible)` shows the strip on a row the
/// keyboard selected or one holding the focus (Blitz never matches `:focus-within`, spike S12),
/// `Some(Shown::Hidden)` keeps it down even under the pointer; `None` is the hover reveal.
/// `expanded` names the buttons that open a menu and whether theirs is open (`aria-haspopup`,
/// `aria-expanded`). A click on a strip button never reaches the row: it does not open it.
///
/// `on_press` hears which button was pressed, synchronously, inside the click and before any
/// measurement (mailo gaps 4): an archive must never wait on a layout read, and a renderer with
/// no layout (a server render, a host that cannot measure) never answers one, so the action's
/// own `onclick` would never come. When the button's rect does resolve, its `onclick` follows,
/// for the menus it anchors. A strip-level prop rather than a field of [`StripAction`], so
/// every `StripAction { .. }` literal still compiles.
#[component]
pub fn HoverStrip(
    actions: Vec<StripAction>,
    #[props(default)] shown: Option<Shown>,
    #[props(default)] titles: Titles,
    #[props(default)] expanded: Vec<(ActionId, Expanded)>,
    #[props(default)] on_press: Option<EventHandler<ActionId>>,
) -> Element {
    rsx! {
        div { class: "ds-strip", "data-shown": shown.map(Shown::slug),
            for (j, action) in actions.into_iter().enumerate() {
                StripButton {
                    key: "{action.id.0}",
                    expanded: expanded
                        .iter()
                        .find(|(id, _)| *id == action.id)
                        .map(|(_, state)| *state),
                    action,
                    j: StaggerIndex::new(j),
                    titles,
                    on_press,
                }
            }
        }
    }
}

/// One strip button: its own component so it can hold its mounted element for the rect.
#[component]
fn StripButton(
    action: StripAction,
    j: StaggerIndex,
    titles: Titles,
    expanded: Option<Expanded>,
    on_press: Option<EventHandler<ActionId>>,
) -> Element {
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
    let pressed = id.clone();
    let title = match titles {
        Titles::Omitted => None,
        Titles::FromLabel => Some(label.clone()),
    };
    rsx! {
        button {
            r#type: "button",
            class: "ds-icon-button",
            "data-variant": "strip",
            "data-op": "{id.0}",
            style: "--j:{j}",
            "aria-label": "{label}",
            title,
            "aria-haspopup": expanded.map(|_| "menu"),
            "aria-expanded": expanded.map(Expanded::aria),
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
            onclick: move |event| {
                // The strip acts on its own; the row must not also open.
                event.stop_propagation();
                // Any click clears the destination preview (design/06-INTERACTIONS.md
                // section 14).
                if let Some(onhover) = onhover {
                    onhover.call(Here::Elsewhere);
                }
                // The press acts now; the placement follows only if a rect arrives.
                if let Some(on_press) = on_press {
                    on_press.call(pressed.clone());
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
