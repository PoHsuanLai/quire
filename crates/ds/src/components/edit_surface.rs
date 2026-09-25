//! EditSurface: a focusable block that hosts an app's own rendered text and hands the app its
//! input (FINDINGS "Edit surface"). mailo's composer renders its paragraphs and objects as the
//! children, marked with `data-edit-node`; the surface owns the focus and the IME, turns keys,
//! IME composition and clipboard gestures into [`EditInput`]s, resolves pointer presses to
//! [`TextPosition`](crate::TextPosition)s through the host, and never draws a caret or a
//! selection: the app does, from its [`EditHandle`]'s rects.

use crate::components::edit_surface_state::{Pressing, SurfaceState, write_soon};
use crate::edit::clicks::Clicks;
use crate::edit::composition::{Composing, on_ime, settle};
use crate::edit::handle::EditHandle;
use crate::edit::host::{HostEdit, ImeEvent, ImeSwitch, Probe};
use crate::edit::input::EditInput;
use crate::edit::keys::{KeyAction, classify};
use crate::edit::pointer::{EditFocus, EditPointer, Extend, PointerPhase};
use crate::focus::Select;
use crate::focus::host::focus_soon_told;
use crate::geometry::measure::BUSY_ATTEMPTS;
use crate::geometry::{Point, Px, Rect};
use crate::time::{FRAME_SLACK, sleep};
use dioxus::html::input_data::MouseButton;
use dioxus::prelude::*;
use std::rc::Rc;

/// An editing surface over the app's own content.
///
/// - `on_input`: text, keys, IME composition steps, paste, cut and copy, in order.
/// - `on_pointer`: presses, drags and releases of the primary button, with the text position
///   under the pointer.
/// - `on_focus`: the surface took or lost the keyboard.
/// - `handle`: the app's handle for caret and selection rects (`use_edit_handle`).
/// - `ime_area`: where the IME's candidate window should sit (the caret's rect); applied while
///   the surface has the keyboard, and again each time it takes it.
#[component]
pub fn EditSurface(
    on_input: EventHandler<EditInput>,
    #[props(default)] on_pointer: Option<EventHandler<EditPointer>>,
    #[props(default)] on_focus: Option<EventHandler<EditFocus>>,
    #[props(default)] handle: Option<EditHandle>,
    #[props(default)] ime_area: Option<Rect>,
    #[props(into, default)] label: Option<String>,
    #[props(into, default)] id: Option<String>,
    children: Element,
) -> Element {
    let host = use_hook(try_consume_context::<HostEdit>);
    let state = use_hook(|| Rc::new(SurfaceState::default()));
    let heard = {
        let state = Rc::clone(&state);
        use_callback(move |event: ImeEvent| {
            let (next, inputs) = on_ime(state.composing.get(), event);
            state.composing.set(next);
            inputs.into_iter().for_each(|input| on_input.call(input));
        })
    };
    let focused_in = {
        let state = Rc::clone(&state);
        use_callback(move |()| {
            let was = state.focus.replace(EditFocus::In);
            if let (Some(host), Some(element)) = (host, state.element()) {
                ime_on(host, element, state.ime_area.get());
            }
            if was == EditFocus::Out
                && let Some(told) = on_focus
            {
                told.call(EditFocus::In);
            }
        })
    };
    {
        let state = Rc::clone(&state);
        use_effect(use_reactive!(|ime_area| {
            state.ime_area.set(ime_area);
            if let (Some(host), Some(element), Some(area), EditFocus::In) =
                (host, state.element(), ime_area, state.focus.get())
            {
                write_soon(host, element, move |host, element| {
                    (host.set_ime_cursor_area)(element, area)
                });
            }
        }));
    }
    {
        let state = Rc::clone(&state);
        use_drop(move || {
            if let (Some(host), Some(listener)) = (host, state.listener.take()) {
                (host.forget)(listener);
            }
        });
    }

    let mounted = {
        let state = Rc::clone(&state);
        move |event: MountedEvent| {
            let element = event.data();
            state.element.replace(Some(Rc::clone(&element)));
            if let Some(handle) = handle {
                handle.set(Rc::clone(&element));
            }
            if let Some(host) = host {
                listen_soon(host, element, heard, Rc::clone(&state));
            }
        }
    };
    let keydown = {
        let state = Rc::clone(&state);
        move |event: KeyboardEvent| key_down(&state, host, on_input, &event)
    };
    let pointerdown = {
        let state = Rc::clone(&state);
        move |event: PointerEvent| {
            if event.trigger_button() != Some(MouseButton::Primary) {
                return;
            }
            // Blitz's default would start its own text selection, and the click after it
            // would clear the focus (FINDINGS "Edit surface").
            event.prevent_default();
            let at = point_of(&event);
            let clicks = state.press(at);
            report(
                &state,
                host,
                on_pointer,
                &event,
                PointerPhase::Press,
                clicks,
            );
            if let Some(element) = state.element() {
                focus_soon_told(element, Select::None, focused_in);
            }
        }
    };
    let pointermove = {
        let state = Rc::clone(&state);
        move |event: PointerEvent| {
            if state.pressing.get() == Pressing::Up {
                return;
            }
            if !event.held_buttons().contains(MouseButton::Primary) {
                state.pressing.set(Pressing::Up);
                return;
            }
            report(
                &state,
                host,
                on_pointer,
                &event,
                PointerPhase::Drag,
                last_clicks(&state),
            );
        }
    };
    let pointerup = {
        let state = Rc::clone(&state);
        move |event: PointerEvent| {
            if state.pressing.replace(Pressing::Up) == Pressing::Down {
                report(
                    &state,
                    host,
                    on_pointer,
                    &event,
                    PointerPhase::Release,
                    last_clicks(&state),
                );
            }
        }
    };
    let blurred = {
        let state = Rc::clone(&state);
        move |_: FocusEvent| focused_out(&state, host, on_input, on_focus)
    };

    rsx! {
        div {
            class: "ds-edit",
            id,
            role: "textbox",
            "aria-multiline": "true",
            "aria-label": label,
            tabindex: "0",
            onmounted: mounted,
            onkeydown: keydown,
            onpointerdown: pointerdown,
            onpointermove: pointermove,
            onpointerup: pointerup,
            onclick: move |event: MouseEvent| event.prevent_default(),
            onfocus: move |_: FocusEvent| focused_in.call(()),
            onblur: blurred,
            {children}
        }
    }
}

/// A key on the surface: nothing while the IME composes (it owns the keys), else end a cleared
/// composition, then hand over what the key means.
fn key_down(
    state: &SurfaceState,
    host: Option<HostEdit>,
    on_input: EventHandler<EditInput>,
    event: &KeyboardEvent,
) {
    if state.composing.get() == Composing::Active {
        return;
    }
    let (idle, ended) = settle(state.composing.get());
    state.composing.set(idle);
    ended.into_iter().for_each(|input| on_input.call(input));
    let input = match classify(&event.key(), event.modifiers()) {
        KeyAction::Text(text) => EditInput::Text(text),
        KeyAction::Key(key) => EditInput::Key(key),
        KeyAction::Cut => EditInput::Cut,
        KeyAction::Copy => EditInput::Copy,
        KeyAction::Paste => {
            event.prevent_default();
            match host.and_then(|host| (host.read_clipboard_html)()) {
                Some(pasted) => EditInput::Paste(pasted),
                None => return,
            }
        }
    };
    on_input.call(input);
}

/// The surface lost the keyboard: end an open composition, switch the IME off, tell the app.
fn focused_out(
    state: &SurfaceState,
    host: Option<HostEdit>,
    on_input: EventHandler<EditInput>,
    on_focus: Option<EventHandler<EditFocus>>,
) {
    let (idle, ended) = settle(state.composing.get());
    state.composing.set(idle);
    ended.into_iter().for_each(|input| on_input.call(input));
    if let (Some(host), Some(element)) = (host, state.element()) {
        write_soon(host, element, |host, element| {
            (host.set_ime)(element, ImeSwitch::Off)
        });
    }
    if state.focus.replace(EditFocus::Out) == EditFocus::In
        && let Some(told) = on_focus
    {
        told.call(EditFocus::Out);
    }
}

/// Switch the IME on for the surface and put its candidate window at `area`, when the app gave
/// one.
fn ime_on(host: HostEdit, element: Rc<MountedData>, area: Option<Rect>) {
    write_soon(host, element, move |host, element| {
        match (host.set_ime)(element, ImeSwitch::On) {
            Probe::Found(()) => match area {
                Some(area) => (host.set_ime_cursor_area)(element, area),
                None => Probe::Found(()),
            },
            other => other,
        }
    });
}

/// Register the surface for IME events once the document is free.
fn listen_soon(
    host: HostEdit,
    element: Rc<MountedData>,
    heard: EventHandler<ImeEvent>,
    state: Rc<SurfaceState>,
) {
    spawn(async move {
        for _ in 0..BUSY_ATTEMPTS {
            match (host.listen)(&element, heard) {
                Probe::Busy => sleep(FRAME_SLACK).await,
                Probe::Found(listener) => {
                    if let Some(stale) = state.listener.replace(Some(listener)) {
                        (host.forget)(stale);
                    }
                    return;
                }
                Probe::Unknown => return,
            }
        }
    });
}

/// Tell the app about a pointer event, with the text position the host finds under it.
fn report(
    state: &SurfaceState,
    host: Option<HostEdit>,
    on_pointer: Option<EventHandler<EditPointer>>,
    event: &PointerEvent,
    phase: PointerPhase,
    clicks: Clicks,
) {
    let Some(on_pointer) = on_pointer else {
        return;
    };
    let at = point_of(event);
    let position = match (host, state.element()) {
        (Some(host), Some(element)) => (host.hit_test)(&element, at).found(),
        _ => None,
    };
    let extend = if event.modifiers().contains(Modifiers::SHIFT) {
        Extend::FromAnchor
    } else {
        Extend::Fresh
    };
    on_pointer.call(EditPointer {
        phase,
        at,
        position,
        extend,
        clicks,
    });
}

fn last_clicks(state: &SurfaceState) -> Clicks {
    state.last_clicks()
}

fn point_of(event: &PointerEvent) -> Point {
    let at = event.client_coordinates();
    Point {
        x: Px(at.x as f32),
        y: Px(at.y as f32),
    }
}
