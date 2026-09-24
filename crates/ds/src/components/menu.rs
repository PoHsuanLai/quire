//! Menu: "ONE MENU: every list of choices in the window uses this" (design/04-COMPONENTS.md
//! section 20). Filtering and ranking are pure Rust (design/06-INTERACTIONS.md section 11).
//!
//! The menu takes focus when it opens and handles its own keys wherever it opens
//! (design/06-INTERACTIONS.md section 2.4): Up and Down wrap and skip disabled items, Enter or
//! Tab picks, Escape closes the topmost layer only, and with [`Filter::Typing`] typing filters
//! with the fuzzy ranker and resets the selection. A [`MenuEntry::Submenu`] opens its children
//! beside it, driven by the menu-tracking machine (design/13-BEHAVIOUR-menus-windows.md section
//! 13.3.4): `menu_panel` holds what the menu and its submenus share.

pub use crate::components::menu_kind::{MenuEntrance, MenuKind};

use crate::components::menu_cursor::Cursor;
use crate::components::menu_entry::MenuEntry;
use crate::components::menu_keys::{Decision, Level};
use crate::components::menu_lines::{Act, Choice, Filter, KeyAct, choices, key_act, lines};
use crate::components::menu_panel::Panel;
use crate::components::menu_tracker::{Via, use_tracker};
use crate::components::popover::{
    Dismiss, Stacking, escape_closes, position_style, use_entrance, use_float,
};
use crate::components::press::{PointerButton, Press, button_of};
use crate::components::vocab::Availability;
use crate::geometry::{Anchor, MountedRef, Point};
use crate::motion::anim::Anim;
use crate::motion::presence::Presence;
use crate::motion::timer::use_motion_timer;
use crate::overlay::menu_track::MenuTiming;
use crate::tokens::ZLayer;
use dioxus::core::queue_effect;
use dioxus::prelude::*;

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
///
/// `active` says whose highlight it shows. [`Cursor::Auto`] is the menu's own, and `on_active`
/// hears every change of it. [`Cursor::Controlled`] is a field's beside the menu (the
/// composer's `/` and `@` menus): the menu shows that choice, leaves the keyboard in the field,
/// and Up, Down and the pointer only ask for a move through `on_active`; the field's own keys
/// pick with the value it knows. `onquery` hears the typed filter's text on every change
/// ([`Filter::Typing`]), for an entry that names it ("Create label '…'").
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
    #[props(default)] active: Cursor,
    #[props(default)] on_active: Option<EventHandler<Option<usize>>>,
    #[props(default)] onquery: Option<EventHandler<String>>,
) -> Element {
    let float = use_float(ZLayer::Menu, Stacking::Layer(Dismiss::EscAndOutside));
    let presence = use_entrance(kind.entrance());
    let fade = use_motion_timer(Anim::MenuOut);
    let mut closing = use_signal(|| Closing::No);
    let mut query = use_signal(String::new);
    let tracker = use_tracker(timing, kind.pad());
    let mut gesture = use_hook(|| CopyValue::new(Gesture::Outside));
    let mut hovered = use_hook(|| CopyValue::new(None::<usize>));
    let reported = use_hook(|| CopyValue::new(None::<Option<usize>>));
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
        | MenuEntry::Row(_)
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
                asks(active, index, on_active);
            }
        })),
        onrelease: None,
        cursor: active,
        on_active,
    };
    follow_active(active, panel.current(), reported, on_active);
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
                let mut next = query.peek().clone();
                match key_act(&event.key(), event.modifiers(), filter) {
                    Some(KeyAct::Type(text)) => next.push_str(&text),
                    Some(KeyAct::Erase) => {
                        next.pop();
                    }
                    _ => {}
                }
                if *query.peek() != next {
                    query.set(next.clone());
                    if let Some(onquery) = onquery {
                        onquery.call(next);
                    }
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
                    // A field beside the menu that drives its cursor keeps the keyboard.
                    if active.takes_focus() {
                        crate::focus::host::focus_soon(element);
                    }
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

/// Under a caller's cursor, the pointer over another choice asks for it.
fn asks(active: Cursor, pointed: Option<usize>, on_active: Option<EventHandler<Option<usize>>>) {
    if let (Cursor::Controlled(shown), Some(index), Some(on_active)) = (active, pointed, on_active)
        && shown != Some(index)
    {
        on_active.call(Some(index));
    }
}

/// Under the menu's own cursor, tell the caller after this render when the highlight moved.
fn follow_active(
    active: Cursor,
    current: Option<usize>,
    reported: CopyValue<Option<Option<usize>>>,
    on_active: Option<EventHandler<Option<usize>>>,
) {
    let (Cursor::Auto, Some(on_active)) = (active, on_active) else {
        return;
    };
    let mut reported = reported;
    if *reported.peek() == Some(current) {
        return;
    }
    reported.set(Some(current));
    queue_effect(move || on_active.call(current));
}
