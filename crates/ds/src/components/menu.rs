//! Menu: "ONE MENU: every list of choices in the window uses this" (design/04-COMPONENTS.md
//! section 20). Filtering and ranking are pure Rust (design/06-INTERACTIONS.md section 11).
//!
//! The menu takes focus when it opens and handles its own keys wherever it opens
//! (design/06-INTERACTIONS.md section 2.4): Up and Down wrap and skip disabled items, Enter or
//! Tab picks, Escape closes the topmost layer only, and with [`Filter::Typing`] typing filters
//! with the fuzzy ranker and resets the selection. A [`MenuEntry::Submenu`] opens its children
//! beside it, driven by the menu-tracking machine (design/13-BEHAVIOUR-menus-windows.md section
//! 13.3.4): `menu_panel` holds what the menu and its submenus share.

use crate::components::menu_entry::{MenuEntry, Row};
use crate::components::menu_keys::{Decision, Level};
use crate::components::menu_lines::{Filter, KeyAct, choices, key_act, lines};
use crate::components::menu_panel::Panel;
use crate::components::menu_tracker::{Via, use_tracker};
use crate::components::popover::{
    Dismiss, Stacking, escape_closes, position_style, use_entrance, use_float,
};
use crate::geometry::{Align, Anchor, MountedRef, Placement, Point, Px, Rect, Side};
use crate::motion::anim::Anim;
use crate::overlay::menu_track::MenuTiming;
use crate::tokens::ZLayer;
use dioxus::prelude::*;

/// Which menu shape.
#[derive(Debug, Clone, Copy, PartialEq, Eq, Hash)]
pub enum MenuKind {
    /// 280 wide, 34 px tiles, title and help and shortcut.
    Rich,
    /// 220 wide, 22 px tiles.
    Slim,
    /// C's check-column menu, anchored under its button's right edge.
    Dropdown,
    /// Anchored at the pointer.
    Context,
}

impl MenuKind {
    /// The `data-kind` word.
    pub(crate) fn slug(self) -> &'static str {
        match self {
            MenuKind::Rich => "rich",
            MenuKind::Slim => "slim",
            MenuKind::Dropdown => "dropdown",
            MenuKind::Context => "context",
        }
    }

    /// Where the menu goes against its anchor: S's floating menus at `left - 8`, 6 below
    /// (`S:2060-2065`); C's dropdown under the trigger's right edge, 7 below; a context menu
    /// at the pointer (design/06-INTERACTIONS.md section 4).
    fn placement(self, anchor: Rect) -> (Rect, Placement, Px) {
        match self {
            MenuKind::Rich | MenuKind::Slim => (
                Rect {
                    origin: Point {
                        x: anchor.origin.x - Px(8.0),
                        ..anchor.origin
                    },
                    ..anchor
                },
                Placement::new(Side::Bottom, Align::Start),
                Px(6.0),
            ),
            MenuKind::Dropdown => (anchor, Placement::new(Side::Bottom, Align::End), Px(7.0)),
            MenuKind::Context => (anchor, Placement::new(Side::Bottom, Align::Start), Px(0.0)),
        }
    }

    /// The entrance: `menu-in` for C's dropdown, `menu-pop` for the rest.
    fn entrance(self) -> Anim {
        match self {
            MenuKind::Dropdown => Anim::MenuIn,
            MenuKind::Rich | MenuKind::Slim | MenuKind::Context => Anim::MenuPop,
        }
    }

    /// The panel's padding (design/04-COMPONENTS.md section 20): 6 for the Dropdown, 5 for the
    /// rest. A submenu's top sits this far above its parent row.
    pub(crate) fn pad(self) -> Px {
        match self {
            MenuKind::Dropdown => Px(6.0),
            MenuKind::Rich | MenuKind::Slim | MenuKind::Context => Px(5.0),
        }
    }

    /// The item layout.
    pub(crate) fn row(self) -> Row {
        match self {
            MenuKind::Dropdown => Row::Checked,
            MenuKind::Rich | MenuKind::Slim | MenuKind::Context => Row::Tiled,
        }
    }
}

/// A floating list of choices. `timing` is the submenu delay and safe-triangle timeout, read
/// by the caller from `menus.submenu_delay_ms` and `menus.submenu_triangle_timeout_ms`
/// (design/22-SETTINGS.md; the defaults are the proposed 200 and 300 ms). `expanded` opens
/// that choice's submenu as the menu mounts (a restored menu, a posed picture); choices are
/// numbered over items and submenu parents, headers and rules not counted.
#[component]
pub fn Menu<T: Clone + PartialEq + 'static>(
    kind: MenuKind,
    anchor: Anchor,
    entries: Vec<MenuEntry<T>>,
    #[props(default)] filter: Filter,
    onpick: EventHandler<T>,
    onclose: EventHandler<()>,
    #[props(default)] timing: MenuTiming,
    #[props(default)] expanded: Option<usize>,
) -> Element {
    let float = use_float(ZLayer::Menu, Stacking::Layer(Dismiss::EscAndOutside));
    let presence = use_entrance(kind.entrance());
    let mut query = use_signal(String::new);
    let tracker = use_tracker(timing, kind.pad());
    use_effect(move || {
        if let Some(index) = expanded {
            tracker.expand(index, Via::Pointer);
        }
    });
    let typed = query();
    let shown = lines(&entries, &typed);
    let label = entries.iter().find_map(|entry| match entry {
        MenuEntry::Header(text) => Some(text.clone()),
        MenuEntry::Item { .. } | MenuEntry::Submenu { .. } | MenuEntry::Separator => None,
    });
    let at = float
        .anchor_rect(&anchor)
        .map(|rect| kind.placement(rect))
        .map_or(Point::default(), |(rect, want, gap)| {
            float.origin(Some(rect), want, gap)
        });
    let panel = Panel {
        tracker,
        choices: choices(&shown),
        level: Level::Root,
        onpick: EventHandler::new(move |value: T| {
            onclose.call(());
            onpick.call(value);
        }),
        onhover: None,
    };
    let onkey = {
        let panel = panel.clone();
        move |event: KeyboardEvent| match panel.key(&event, filter) {
            Decision::CloseMenu => escape_closes(float, &event, onclose),
            Decision::Query => {
                match key_act(&event.key(), event.modifiers(), filter) {
                    Some(KeyAct::Type(text)) => query.with_mut(|query| query.push_str(&text)),
                    Some(KeyAct::Erase) => {
                        query.with_mut(|query| {
                            query.pop();
                        });
                    }
                    _ => {}
                }
                tracker.reset(0);
            }
            _ => {}
        }
    };
    let body = if shown.is_empty() {
        rsx! {
            div { class: "ds-menu-empty", "Nothing matches \"{typed}\"." }
        }
    } else {
        panel.body(&shown, kind)
    };
    let child = panel.submenu(kind, timing, 0);
    let probe = float.surface();
    let hover = panel.clone();
    float.show(
        rsx! {
            div {
                class: "ds-popover ds-menu",
                "data-elevation": "pop",
                "data-layer": "menu",
                "data-kind": kind.slug(),
                role: "listbox",
                "aria-label": label,
                "data-presence": presence.slug(),
                tabindex: "-1",
                style: position_style(at),
                onmounted: move |event| {
                    let element = event.data();
                    tracker.panel_mounted(MountedRef(element.clone()));
                    probe.on_mounted(event);
                    spawn(async move {
                        let _ = element.set_focus(true).await;
                    });
                },
                onmousemove: move |event| hover.hovered(&event),
                onkeydown: onkey,
                {body}
            }
        },
        onclose,
    );
    rsx! {
        {child}
    }
}
