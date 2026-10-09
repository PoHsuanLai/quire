//! A submenu: a panel of `items` placed right of its parent panel (left when it would cross the
//! edge), its first row level with the parent row, opening with no animation (design/13 section
//! 13.3.4). It floats on the menu layer without joining the layer stack: its menu's layer takes
//! Escape and the outside click, and it closes with its parent.

use crate::components::menus::alive::use_alive;
use crate::components::menus::item::item::MenuItem;
use crate::components::menus::menu::blink::Blink;
use crate::components::menus::menu::choices::choices;
use crate::components::menus::menu::cursor::MenuCursor;
use crate::components::menus::menu::decide::{Decision, Level};
use crate::components::menus::menu::panel::Panel;
use crate::components::menus::menu::pick::Picked;
use crate::components::menus::menu::placement::{Keys, MENU_INSET};
use crate::components::menus::menu::tracker::{Via, use_tracker};
use crate::components::overlays::popover::{Stacking, layer_slug, position_style, use_float};
use crate::host::measure::MountedRef;
use crate::stack::menu_track::types::MenuTiming;
use dioxus::prelude::*;
use ds_core::geometry::{
    placement::{Align, Placement, Side},
    units::{Point, Px, Rect},
};
use ds_style::tokens::layer::ZLayer;

/// How far a submenu sits from its parent panel (design/13 section 13.3.4: "gap 2").
const SUB_GAP: Px = Px(2.0);

/// A submenu of `items` at `depth`, opened `via` the pointer or the keyboard.
#[component]
pub(crate) fn SubMenu<T: Clone + PartialEq + 'static>(
    anchor: Rect,
    via: Via,
    items: Vec<MenuItem<T>>,
    timing: MenuTiming,
    depth: u8,
    blink: Blink,
    keys: Keys,
    onpick: EventHandler<Picked<T>>,
    onback: EventHandler<()>,
    onhover: EventHandler<Point>,
    onplaced: EventHandler<Rect>,
) -> Element {
    let float = use_float(ZLayer::Menu, Stacking::Passive);
    let tracker = use_tracker(timing, MENU_INSET);
    let alive = use_alive();
    let (keyed, opened) = (alive.clone(), alive.clone());
    let panel = Panel {
        tracker,
        choices: choices(&items),
        level: Level::Sub,
        depth,
        blink,
        keys,
        onpick,
        onhover: Some(onhover),
        onitem: None,
        onrelease: None,
        cursor: MenuCursor::Auto,
        on_active: None,
        alive,
    };
    let want = Placement::new(Side::Right, Align::Start);
    let at = float.origin(Some(anchor), want, SUB_GAP);
    let placed = float.placed(Some(anchor), want, SUB_GAP);
    let mut reported = use_signal(|| None::<Rect>);
    use_effect(use_reactive!(|placed| {
        if placed.is_some() && *reported.peek() != placed {
            reported.set(placed);
            if let Some(rect) = placed {
                onplaced.call(rect);
            }
        }
    }));
    let body = panel.body(&items);
    let child = panel.submenu(timing);
    let probe = float.surface();
    let key_panel = panel.clone();
    let hover = panel.clone();
    float.show(
        rsx! {
            div {
                class: "ds-popover ds-menu",
                "data-elevation": "pop",
                "data-layer": layer_slug(ZLayer::Menu),
                "data-depth": "{depth}",
                role: "menu",
                "data-presence": "present",
                tabindex: "-1",
                                style: if placed.is_some() && probe.rect().is_some_and(|rect| rect.size.width.0 > 0.0) {
                    position_style(at)
                } else {
                    format!("{};visibility:hidden", position_style(at))
                },
                onmounted: move |event| {
                    opened.run(|| {
                        let element = event.data();
                        tracker.panel_mounted(MountedRef(element.clone()));
                        probe.on_mounted(event);
                        if via == Via::Keyboard {
                            crate::focus::soon::focus_soon(element);
                        }
                    });
                },
                onmousemove: move |event| hover.hovered(&event),
                onkeydown: move |event| {
                    keyed.run(|| {
                        if key_panel.key(&event) == Decision::Back {
                            onback.call(());
                        }
                    });
                },
                {body}
            }
        },
        EventHandler::new(move |()| onback.call(())),
    );
    rsx! {
        {child}
    }
}
