//! An [`EditSurface`](crate::EditSurface) taking and giving up the keyboard. A press, Tab and
//! `EditHandle::focus` all end in [`focused_in`]; a blur event and `EditHandle::blur` in
//! [`focused_out`]: the app hears `on_focus`, the IME is switched and pointed, and the surface
//! is the IME's target while it has the keyboard (FINDINGS "Edit surface 2").

use crate::components::edit_surface_ctx::SurfaceCtx;
use crate::components::edit_surface_state::write_soon;
use crate::edit::composition::settle;
use crate::edit::host::{HostEdit, ImeEvent, ImeSwitch, Probe};
use crate::edit::pointer::EditFocus;
use crate::focus::Select;
use crate::focus::host::{blur_element, focus_soon_told};
use crate::geometry::Rect;
use crate::geometry::measure::BUSY_ATTEMPTS;
use crate::time::{FRAME_SLACK, sleep};
use dioxus::prelude::*;
use std::rc::Rc;

/// The surface has the keyboard: the IME on and pointed at the app's area, the app told once.
pub(crate) fn focused_in(ctx: &SurfaceCtx) {
    let was = ctx.state.focus.replace(EditFocus::In);
    if let (Some(host), Some(element)) = (ctx.host, ctx.state.element()) {
        ime_on(host, element, ctx.state.ime_area.get());
    }
    if was == EditFocus::Out
        && let Some(told) = ctx.on_focus
    {
        told.call(EditFocus::In);
    }
}

/// The surface lost the keyboard: end an open composition, switch the IME off, tell the app.
pub(crate) fn focused_out(ctx: &SurfaceCtx) {
    let (idle, ended) = settle(ctx.state.composing.get());
    ctx.state.composing.set(idle);
    ctx.tell(ended);
    if let (Some(host), Some(element)) = (ctx.host, ctx.state.element()) {
        write_soon(host, element, |host, element| {
            (host.set_ime)(element, ImeSwitch::Off)
        });
    }
    if ctx.state.focus.replace(EditFocus::Out) == EditFocus::In
        && let Some(told) = ctx.on_focus
    {
        told.call(EditFocus::Out);
    }
}

/// Focus the surface as a press does: through the host's focus write, then [`focused_in`] once
/// it lands (`told`). With no host the renderer's own focus fires the element's `focus` event,
/// which calls it instead.
pub(crate) fn focus_surface(ctx: &SurfaceCtx, told: EventHandler<()>) {
    if let Some(element) = ctx.state.element() {
        focus_soon_told(element, Select::None, told);
    }
}

/// Take the keyboard away from the surface: the host's blur write (`ds::HostBlur`; no `blur`
/// event follows it), then [`focused_out`].
pub(crate) fn blur_surface(ctx: &SurfaceCtx) {
    if let Some(element) = ctx.state.element() {
        spawn(async move {
            let _ = blur_element(&element).await;
        });
    }
    focused_out(ctx);
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
pub(crate) fn listen_soon(
    ctx: &SurfaceCtx,
    element: Rc<MountedData>,
    heard: EventHandler<ImeEvent>,
) {
    let Some(host) = ctx.host else {
        return;
    };
    let state = Rc::clone(&ctx.state);
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
