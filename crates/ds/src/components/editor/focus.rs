//! An [`EditSurface`](crate::EditSurface) taking and giving up the keyboard. A press, Tab and
//! `EditHandle::focus` all end in [`focused_in`]; a blur event and `EditHandle::blur` in
//! [`focused_out`]: the app hears `on_focus`, the IME is switched and pointed, and the surface
//! is the IME's target while it has the keyboard (FINDINGS "Edit surface").

use crate::components::editor::ctx::SurfaceCtx;
use crate::components::editor::state::write_soon;
use crate::core::geometry::units::Rect;
use crate::core::time::{FRAME_SLACK, clock::sleep};
use crate::edit::composition::settle;
use crate::edit::pointer::EditFocus;
use crate::focus::select::Select;
use crate::focus::soon::{blur_element, focus_soon_told};
use crate::host::document::DocumentHost;
use crate::host::ime::{ImeEvent, ImeSwitch};
use crate::host::measure::BUSY_ATTEMPTS;
use crate::host::probe::Probe;
use dioxus::prelude::*;
use std::rc::Rc;

/// The surface has the keyboard: the IME on and pointed at the app's area, the app told once.
pub(crate) fn focused_in(ctx: &SurfaceCtx) {
    let was = ctx.state.focus.replace(EditFocus::In);
    if let Some(element) = ctx.state.element() {
        ime_on(Rc::clone(&ctx.host), element, ctx.state.ime_area.get());
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
    if let Some(element) = ctx.state.element() {
        write_soon(Rc::clone(&ctx.host), element, |edit, element| {
            edit.ime().switch(element, ImeSwitch::Off)
        });
    }
    if ctx.state.focus.replace(EditFocus::Out) == EditFocus::In
        && let Some(told) = ctx.on_focus
    {
        told.call(EditFocus::Out);
    }
}

/// Focus the surface as a press does: through the host's focus write, then [`focused_in`] once
/// it lands (`told`).
pub(crate) fn focus_surface(ctx: &SurfaceCtx, told: EventHandler<()>) {
    if let Some(element) = ctx.state.element() {
        focus_soon_told(element, Select::None.into(), told);
    }
}

/// Take the keyboard away from the surface: the host's blur write (no `blur`
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
fn ime_on(host: Rc<dyn DocumentHost>, element: Rc<MountedData>, area: Option<Rect>) {
    write_soon(host, element, move |edit, element| {
        match edit.ime().switch(element, ImeSwitch::On) {
            Probe::Found(()) => match area {
                Some(area) => edit.ime().cursor_area(element, area),
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
    let host = Rc::clone(&ctx.host);
    let state = Rc::clone(&ctx.state);
    spawn(async move {
        let Some(edit) = host.edit() else {
            return;
        };
        for _ in 0..BUSY_ATTEMPTS {
            match edit.ime().listen(&element, heard) {
                Probe::Busy => sleep(FRAME_SLACK).await,
                Probe::Found(listener) => {
                    if let Some(stale) = state.listener.replace(Some(listener)) {
                        edit.ime().forget(stale);
                    }
                    return;
                }
                Probe::Unknown => return,
            }
        }
    });
}
