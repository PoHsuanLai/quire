//! MenuBarItem: a bar title or text status item and the pill behind it (the macOS polish pass,
//! 2026-09-24; design/13-BEHAVIOUR-menus-windows.md section 13.3.1). The item's text is the shell
//! scale's bar size and weight (`--fs-shell-bar` 13, `--fw-shell-bar` 500; the app name is
//! `Emphasis::Strong`, 700); the pill is `--shell-bar-item` (24) high with the 4 px
//! `--r-shell-bar-item` radius, `--f-pill-hover` under the pointer and `--f-pill` while its
//! menu is open, with no transition (a bar menu switches in the same frame). A `Button { Quiet }`
//! inside it gives up its own look and takes the item's; an `IconButton { Status }` draws the
//! same pill itself and needs no wrapper.

use crate::components::vocab::{Emphasis, Switch};
use dioxus::prelude::*;

/// One bar item: `children` is the control (a `Button { Quiet }`, a clock, the app name); `open`
/// is whether its menu is showing.
#[component]
pub fn MenuBarItem(
    #[props(default)] open: Switch,
    #[props(default)] emphasis: Emphasis,
    #[props(default)] id: Option<String>,
    children: Element,
) -> Element {
    let weight = match emphasis {
        Emphasis::Strong => Some("strong"),
        Emphasis::Plain => None,
    };
    rsx! {
        div {
            class: "ds-bar-item",
            id,
            "aria-expanded": open.aria(),
            "data-emphasis": weight,
            {children}
        }
    }
}
