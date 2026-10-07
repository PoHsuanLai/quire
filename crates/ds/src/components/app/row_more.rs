//! RowMore: the one quiet "More actions" button a row carries in its tail: an icon-only
//! button with no pill, border or shadow that opens the row's menu (design/34-MODERN-LOOK.md).

use crate::focus::click::kept_click;
use crate::host::measure::client_rect;
use dioxus::prelude::*;
use ds_core::geometry::units::Rect;
use ds_core::vocab::Shown;
use ds_core::word::Word;
use ds_style::icon::Icon;
use ds_style::icon::render::{Glyph, IconSize};
use std::rc::Rc;

/// The default `aria-label`.
const LABEL: &str = "More actions";

/// A row's trailing action button, for `ThreadRow`'s `more` slot.
///
/// It sits in the row's flow (never over the time or the text) and shows as `HoverStrip` does:
/// hidden but still laid out, so the row never reflows, until the row is hovered, the button
/// holds the keyboard focus (Tab reaches it), `shown` says `Visible` (the caller's say for a
/// keyboard selection, since Blitz never matches `:focus-within`, spike S12) or `expanded` is
/// `Visible`. `shown: Some(Shown::Hidden)` keeps it down under the pointer.
///
/// Controlled: `expanded` is whether the menu it opens is up, owned by the caller (written as
/// `aria-expanded`, and keeping the button visible). `onclick` receives the button's own rect
/// to anchor that menu, as soon as the document has measured it (a later turn, since measuring is
/// asynchronous); `on_press` is called inside the click itself, before anything is measured, so
/// the host learns of the press in the same event and can set its own state at once. A click
/// never reaches the row.
#[component]
pub fn RowMore(
    #[props(default = Icon::Ellipsis)] icon: Icon,
    #[props(into, default = LABEL.to_string())] label: String,
    #[props(default)] expanded: Shown,
    #[props(default)] shown: Option<Shown>,
    #[props(default)] on_press: EventHandler<()>,
    onclick: EventHandler<Rect>,
) -> Element {
    let mut element = use_signal(|| None::<Rc<MountedData>>);
    rsx! {
        button {
            r#type: "button",
            class: "ds-row-more",
            "data-shown": shown.map(Shown::slug),
            "aria-label": "{label}",
            "aria-haspopup": "menu",
            "aria-expanded": expanded.aria(),
            onmounted: move |event| element.set(Some(event.data())),
            onclick: move |event| {
                // The button acts on its own; the row must not also open.
                event.stop_propagation();
                on_press.call(());
                if let Some(mounted) = element() {
                    spawn(async move {
                        if let Some(measured) = client_rect(&mounted).await {
                            onclick.call(measured);
                        }
                    });
                }
                kept_click(&event);
            },
            Glyph { icon, size: IconSize::Compact }
        }
    }
}
