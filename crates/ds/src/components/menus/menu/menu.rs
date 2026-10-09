//! Menu: every list of commands in the window (`NSMenu`, design/30 section 2.4). Up and Down wrap
//! and skip disabled items, Home and End jump to the ends, letters jump to the next item that
//! starts with them, Enter or Tab picks, Escape closes the topmost layer only. A picked item
//! blinks twice, then the menu yields the value and fades out. A [`MenuItem::Submenu`] opens its
//! children beside it, driven by the menu-tracking machine (design/13-BEHAVIOUR-menus-windows.md
//! section 13.3.4): `panel` holds what the menu and its submenus share.

use crate::components::controls::press::{button_of, press_of};
use crate::components::menus::alive::use_alive;
use crate::components::menus::item::context::available;
use crate::components::menus::item::item::MenuItem;
use crate::components::menus::menu::active::{asks, follow_active};
use crate::components::menus::menu::blink::Blink;
use crate::components::menus::menu::choices::{Act, Choice, choices};
use crate::components::menus::menu::cursor::MenuCursor;
use crate::components::menus::menu::decide::{Decision, Level};
use crate::components::menus::menu::hand_back::{give_back, hand_back};
use crate::components::menus::menu::hung::Hung;
use crate::components::menus::menu::panel::Panel;
use crate::components::menus::menu::pick::{Closer, Closing, Gesture, Picked, picker};
use crate::components::menus::menu::placement::{MENU_INSET, MenuPlacement};
use crate::components::menus::menu::surface::{Surface, stacking};
use crate::components::menus::menu::tracker::{Via, use_tracker};
use crate::components::overlays::flow::Flow;
use crate::components::overlays::popover::{escape_closes, use_float};
use crate::host::measure::{Anchor, MountedRef};
use crate::root::common::Common;
use crate::stack::menu_track::types::MenuTiming;
use dioxus::prelude::*;
use ds_core::geometry::units::Rect;
use ds_core::press::{PointerButton, Press};
use ds_core::vocab::Availability;
use ds_core::word::Word;
use ds_motion::anim::Anim;
use ds_motion::timer::use_motion_timer;
use ds_style::tokens::layer::ZLayer;

/// A floating list of commands. `timing` is the submenu delay and safe-triangle timeout, read
/// by the caller from `menus.submenu_delay_ms` and `menus.submenu_triangle_timeout_ms`
/// (design/22-SETTINGS.md; the defaults are the proposed 200 and 300 ms). `expanded` opens
/// that choice's submenu as the menu mounts (a restored menu, a posed picture); choices are
/// numbered over items and submenu parents, headers, status lines and rules not counted.
///
/// `placement` says where the menu belongs. A `Bar` menu and a `Popup` menu draw their items'
/// key equivalents; a `Context` menu leaves out what cannot be picked and shows none. Every menu
/// opens at once. A pick blinks the item twice, then calls `onpick`, fades the menu out over
/// `--t-quick` and calls `onclose`; Escape and an outside click play the fade and call `onclose`.
/// An item with `AfterPick::KeepOpen` (a toggle in a set of toggles) yields its value at once and
/// leaves the menu up, with no blink; the caller redraws the items with the new state.
/// `on_hover` hears which choice the pointer is over (`None` once it is over none),
/// `on_release` every button released over a choice; a release over an enabled choice after a
/// press that began outside the menu (press-drag-release) picks it.
///
/// `active` says whose highlight it shows. [`MenuCursor::Auto`] is the menu's own, and
/// `on_active` hears every change of it. [`MenuCursor::Controlled`] is a field's beside the menu
/// (the composer's `/` and `@` menus): the menu shows that choice, leaves the keyboard in the
/// field, and Up, Down and the pointer only ask for a move through `on_active`; the field's own
/// keys pick with the value it knows.
///
/// `flow: Flow::Inline` draws the same rows where the caller renders the menu (a sender card's
/// actions): no overlay, no surface, no layer on the stack (so no Escape or outside press of its
/// own, and no press-drag-release), and no focus taken; its keys are handled when the focus is
/// inside it, and a caller that keeps the focus elsewhere drives it with
/// [`MenuCursor::Controlled`]. The flow is fixed for the menu's life: key the menu by it to
/// switch.
///
/// `hung: Hung::FromField(width)` says `anchor` is a field the menu hangs from (the suggestions
/// under a search field): presses on the field are inside the menu, so they neither close it nor
/// move the keyboard, and the menu is as wide as `width` says. Pair it with a
/// [`MenuCursor::Controlled`] cursor so the field keeps the keyboard.
///
/// `measured` is `anchor`'s rect when the caller already keeps it (a button's, from
/// [`use_rect`](crate::host::measure::use_rect)): a [`Anchor::Mounted`] menu is placed in its
/// first frame instead of waiting a few to measure the anchor. Until a mounted anchor's rect is
/// known the menu is laid out but hidden, and a press closes nothing and reaches nothing.
///
/// A floating menu that took the keyboard gives it back when it closes: to the anchor's element
/// when `anchor` is [`Anchor::Mounted`] (or its nearest focusable ancestor), if it is still
/// there, through the host's `FocusHost::hand_back`; anchored at a point or a rect, the host gives
/// it to the element focused before the menu opened.
#[component]
pub fn Menu<T: Clone + PartialEq + 'static>(
    placement: MenuPlacement,
    anchor: Anchor,
    items: Vec<MenuItem<T>>,
    onpick: EventHandler<T>,
    onclose: EventHandler<()>,
    #[props(default)] timing: MenuTiming,
    #[props(default)] expanded: Option<usize>,
    #[props(default)] on_hover: Option<EventHandler<Option<usize>>>,
    #[props(default)] on_release: Option<EventHandler<Press>>,
    #[props(default)] active: MenuCursor,
    #[props(default)] on_active: Option<EventHandler<Option<usize>>>,
    #[props(default)] flow: Flow,
    #[props(default)] hung: Hung,
    #[props(default)] measured: Option<Rect>,
    #[props(default)] common: Common,
) -> Element {
    let float = use_float(ZLayer::Menu, stacking(flow));
    use_hook(|| float.known(measured));
    let fade = use_motion_timer(Anim::MenuOut);
    let mut closing = use_signal(|| Closing::No);
    let blink = use_signal(Blink::default);
    let alive = use_alive();
    let tracker = use_tracker(timing, MENU_INSET);
    let mut gesture = use_hook(|| CopyValue::new(Gesture::Outside));
    let mut hovered = use_hook(|| CopyValue::new(None::<usize>));
    let reported = use_hook(|| CopyValue::new(None::<Option<usize>>));
    use_effect(move || {
        if let Some(index) = expanded {
            tracker.expand(index, Via::Pointer);
        }
    });
    let items = match placement {
        MenuPlacement::Context => available(&items),
        MenuPlacement::Bar | MenuPlacement::Popup => items,
    };
    let label = items.iter().find_map(|item| match item {
        MenuItem::Header(text) => Some(text.clone()),
        MenuItem::Item { .. }
        | MenuItem::Submenu { .. }
        | MenuItem::Info { .. }
        | MenuItem::Separator => None,
    });
    // Escape and an outside click fade the menu out, then close it (design/13 section 13.3.2).
    let fade_out = EventHandler::new(move |()| {
        if *closing.peek() == Closing::No {
            closing.set(Closing::Fading);
            fade.start(onclose);
        }
    });
    let handed = {
        let anchor = anchor.clone();
        EventHandler::new(move |()| give_back(&anchor, flow, active))
    };
    let choices = choices(&items);
    let picks = choices.clone();
    let mut panel = Panel {
        tracker,
        choices,
        level: Level::Root,
        depth: 0,
        blink: blink(),
        keys: placement.keys(),
        onpick: picker(
            blink,
            closing,
            flow,
            onpick,
            Closer {
                fade: fade_out,
                now: onclose,
                give_back: handed,
            },
        ),
        onhover: None,
        onitem: Some(EventHandler::new(move |index: Option<usize>| {
            // Inline rows are not a drag's destination: the gesture stays outside.
            if flow == Flow::Floating && *gesture.peek() == Gesture::Outside {
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
        alive: alive.clone(),
    };
    follow_active(active, panel.current(), reported, on_active);
    panel.onrelease = Some(released(picks, panel.onpick, fade_out, gesture, on_release));
    let onkey = {
        let (panel, alive) = (panel.clone(), alive.clone());
        move |event: KeyboardEvent| {
            alive.run(|| {
                if panel.key(&event) == Decision::CloseMenu {
                    let close = EventHandler::new(move |()| {
                        handed.call(());
                        fade_out.call(());
                    });
                    escape_closes(float, &event, close);
                }
            });
        }
    };
    let (opened, pressed, released_on) = (alive.clone(), alive.clone(), alive.clone());
    let body = panel.body(&items);
    let child = panel.submenu(timing);
    let probe = float.surface();
    let opener = anchor.clone();
    let hover = panel.clone();
    let leave = panel.clone();
    let presence = match (closing(), flow) {
        (Closing::Fading, _) => "leaving",
        (Closing::No, _) => "present",
    };
    let surface = Surface::of(flow, float, &anchor, placement, hung);
    let data = common.data_attributes();
    let mounted = common.clone();
    let menu = rsx! {
        div {
            class: common.class(surface.class),
            id: common.id.clone(),
            "data-elevation": surface.elevation,
            "data-layer": surface.layer,
            "data-flow": flow.attr(),
            "data-placement": placement.slug(),
            role: "menu",
            "aria-label": common.aria_label.clone().or(label),
            "data-presence": presence,
            tabindex: "-1",
            style: surface.style,
            onmounted: move |event| {
                opened.run(|| {
                    let element = event.data();
                    tracker.panel_mounted(MountedRef(element.clone()));
                    probe.on_mounted(event.clone());
                    mounted.mounted(event);
                    // A field beside the menu that drives its cursor keeps the keyboard, and an
                    // inline menu leaves it with its caller.
                    if active.takes_focus() && flow == Flow::Floating {
                        hand_back(&element, &opener, flow, active);
                        crate::focus::soon::focus_soon(element);
                    }
                });
            },
            onmousemove: move |event| hover.hovered(&event),
            onmouseleave: move |_| leave.left_items(),
            onmousedown: move |_| {
                pressed.run(|| gesture.set(Gesture::Pressed));
            },
            onmouseup: move |event| {
                released_on.run(|| {
                    if let Some(on_release) = on_release {
                        let button =
                            button_of(event.trigger_button()).unwrap_or(PointerButton::Primary);
                        on_release.call(press_of(&event, button));
                    }
                    // A drag that came in and let go over no choice closes, picking nothing
                    // (design/13 section 13.3.2).
                    if *gesture.peek() == Gesture::Entered {
                        fade_out.call(());
                    }
                });
            },
            onkeydown: onkey,
            ..data,
            {body}
        }
    };
    match flow {
        Flow::Floating => {
            float.show_around(menu, fade_out, hung.catcher(float, &anchor));
            rsx! {
                {child}
            }
        }
        Flow::Inline => rsx! {
            {menu}
            {child}
        },
    }
}

/// A button released over choice `index`: heard by `on_release`, and, after a press that began
/// outside the menu (press-drag-release, design/13 section 13.3.2), an enabled item picks, a
/// parent stays open for its submenu, and a disabled item closes picking nothing.
fn released<T: Clone + 'static>(
    picks: Vec<Choice<T>>,
    pick: EventHandler<Picked<T>>,
    fade_out: EventHandler<()>,
    gesture: CopyValue<Gesture>,
    on_release: Option<EventHandler<Press>>,
) -> EventHandler<(usize, Press)> {
    EventHandler::new(move |(index, press): (usize, Press)| {
        if let Some(on_release) = on_release {
            on_release.call(press);
        }
        if *gesture.peek() != Gesture::Entered {
            return;
        }
        match picks.get(index) {
            Some(Choice {
                act: Act::Pick(value, after),
                availability: Availability::Enabled,
                ..
            }) => pick.call(Picked {
                value: value.clone(),
                depth: 0,
                after: *after,
            }),
            Some(Choice {
                act: Act::Open(_),
                availability: Availability::Enabled,
                ..
            }) => {}
            _ => fade_out.call(()),
        }
    })
}
