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
use crate::components::menu_lines::{Act, Choice, Filter, KeyAct, choices, key_act, lines};
use crate::components::menu_panel::Panel;
use crate::components::menu_tracker::{Via, use_tracker};
use crate::components::popover::{
    Dismiss, Stacking, escape_closes, position_style, use_entrance, use_float,
};
use crate::components::press::{PointerButton, Press, button_of};
use crate::components::vocab::Availability;
use crate::geometry::{Align, Anchor, MountedRef, Placement, Point, Px, Rect, Side};
use crate::motion::anim::Anim;
use crate::motion::presence::Presence;
use crate::motion::timer::use_motion_timer;
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

/// How a menu appears: with its entrance (`menu-pop`, or `menu-in` for a Dropdown), or at once.
/// A bar menu opens at once (design/13-BEHAVIOUR-menus-windows.md section 13.3.2: the menu bar
/// has no open animation, and a hover switch shows the next menu in the same frame).
#[derive(Debug, Clone, Copy, PartialEq, Eq, Hash, Default)]
pub enum MenuEntrance {
    /// Play the kind's entrance.
    #[default]
    Animated,
    /// Appear at rest, with no entrance.
    Instant,
}

/// Where a pointer gesture over an open menu is: a press-drag-release onto an item picks it
/// (design/13 section 13.3.2), which is a release arriving after the pointer came in with no
/// press of its own inside the menu.
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
enum Gesture {
    /// The pointer has not been over the menu.
    Outside,
    /// The pointer came in; no press started inside the menu.
    Entered,
    /// A press started inside the menu: its release is a click, not a drag's end.
    Pressed,
}

/// Whether the menu is closing, and how.
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
enum Closing {
    /// Open.
    No,
    /// Something was picked and `onclose` has run: nothing else picks (Blitz follows a
    /// press-drag-release with a click on the same item).
    Picked,
    /// Playing `menu-out`; `onclose` runs when it settles.
    Fading,
}

/// A floating list of choices. `timing` is the submenu delay and safe-triangle timeout, read
/// by the caller from `menus.submenu_delay_ms` and `menus.submenu_triangle_timeout_ms`
/// (design/22-SETTINGS.md; the defaults are the proposed 200 and 300 ms). `expanded` opens
/// that choice's submenu as the menu mounts (a restored menu, a posed picture); choices are
/// numbered over items and submenu parents, headers, status lines and rules not counted.
///
/// A pick calls `onpick`, then `onclose`. Escape and an outside click play the exit fade
/// (`Anim::MenuOut`) and then call `onclose`. `on_hover` hears which choice the pointer is over
/// (`None` once it is over none), `on_release` every button released over a choice; a release
/// over an enabled choice after a press that began outside the menu (press-drag-release) picks
/// it. `entrance` is [`MenuEntrance::Instant`] for a bar menu.
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
    #[props(default)] on_hover: Option<EventHandler<Option<usize>>>,
    #[props(default)] on_release: Option<EventHandler<Press>>,
    #[props(default)] entrance: MenuEntrance,
) -> Element {
    let float = use_float(ZLayer::Menu, Stacking::Layer(Dismiss::EscAndOutside));
    let presence = use_entrance(kind.entrance());
    let fade = use_motion_timer(Anim::MenuOut);
    let mut closing = use_signal(|| Closing::No);
    let mut query = use_signal(String::new);
    let tracker = use_tracker(timing, kind.pad());
    let mut gesture = use_hook(|| CopyValue::new(Gesture::Outside));
    let mut hovered = use_hook(|| CopyValue::new(None::<usize>));
    use_effect(move || {
        if let Some(index) = expanded {
            tracker.expand(index, Via::Pointer);
        }
    });
    let typed = query();
    let shown = lines(&entries, &typed);
    let label = entries.iter().find_map(|entry| match entry {
        MenuEntry::Header(text) => Some(text.clone()),
        MenuEntry::Item { .. }
        | MenuEntry::Submenu { .. }
        | MenuEntry::Info { .. }
        | MenuEntry::Separator => None,
    });
    let at = float
        .anchor_rect(&anchor)
        .map(|rect| kind.placement(rect))
        .map_or(Point::default(), |(rect, want, gap)| {
            float.origin(Some(rect), want, gap)
        });
    // Escape and an outside click fade the menu out, then close it (design/13 section 13.3.2).
    let dismiss = EventHandler::new(move |()| {
        if *closing.peek() == Closing::No {
            closing.set(Closing::Fading);
            fade.start(onclose);
        }
    });
    let choices = choices(&shown);
    let picks = choices.clone();
    let mut panel = Panel {
        tracker,
        choices,
        level: Level::Root,
        onpick: EventHandler::new(move |value: T| {
            if *closing.peek() == Closing::No {
                closing.set(Closing::Picked);
                onpick.call(value);
                onclose.call(());
            }
        }),
        onhover: None,
        onitem: Some(EventHandler::new(move |index: Option<usize>| {
            if *gesture.peek() == Gesture::Outside {
                gesture.set(Gesture::Entered);
            }
            if *hovered.peek() != index {
                hovered.set(index);
                if let Some(on_hover) = on_hover {
                    on_hover.call(index);
                }
            }
        })),
        onrelease: None,
    };
    let picker = panel.onpick;
    panel.onrelease = Some(EventHandler::new(move |(index, press): (usize, Press)| {
        if let Some(on_release) = on_release {
            on_release.call(press);
        }
        if *gesture.peek() != Gesture::Entered {
            return;
        }
        // Press-drag-release (design/13 section 13.3.2): an enabled item picks, a parent
        // stays open for its submenu, a disabled item closes picking nothing.
        match picks.get(index) {
            Some(Choice {
                act: Act::Pick(value),
                availability: Availability::Enabled,
            }) => picker.call(value.clone()),
            Some(Choice {
                act: Act::Open(_),
                availability: Availability::Enabled,
            }) => {}
            _ => dismiss.call(()),
        }
    }));
    let onkey = {
        let panel = panel.clone();
        move |event: KeyboardEvent| match panel.key(&event, filter) {
            Decision::CloseMenu => escape_closes(float, &event, dismiss),
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
    let leave = panel.clone();
    let presence = match (closing(), entrance) {
        (Closing::Fading, _) => "leaving",
        (Closing::No | Closing::Picked, MenuEntrance::Instant) => Presence::Present.slug(),
        (Closing::No | Closing::Picked, MenuEntrance::Animated) => presence.slug(),
    };
    let instant = (entrance == MenuEntrance::Instant).then_some("instant");
    float.show(
        rsx! {
            div {
                class: "ds-popover ds-menu",
                "data-elevation": "pop",
                "data-layer": "menu",
                "data-kind": kind.slug(),
                "data-entrance": instant,
                role: "listbox",
                "aria-label": label,
                "data-presence": presence,
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
                onmouseleave: move |_| leave.left_items(),
                onmousedown: move |_| gesture.set(Gesture::Pressed),
                onmouseup: move |event| {
                    if let Some(on_release) = on_release {
                        let button = button_of(event.trigger_button()).unwrap_or(PointerButton::Primary);
                        on_release.call(Press::of(&event, button));
                    }
                    // A drag that came in and let go over no choice closes, picking nothing
                    // (design/13 section 13.3.2).
                    if *gesture.peek() == Gesture::Entered {
                        dismiss.call(());
                    }
                },
                onkeydown: onkey,
                {body}
            }
        },
        dismiss,
    );
    rsx! {
        {child}
    }
}
