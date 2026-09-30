//! An [`EditSurface`](crate::EditSurface)'s press, drag and release. A press on the surface
//! captures the pointer through the host: every move and the release until the button comes up
//! reach the surface, wherever the pointer is, so a drag selection keeps following it outside
//! the surface's box (FINDINGS "Edit surface"). With no host (or before the capture lands) the
//! surface's own `pointermove` and `pointerup` serve instead.

use crate::components::editor::ctx::SurfaceCtx;
use crate::components::editor::focus::focus_surface;
use crate::components::editor::state::Capture;
use crate::edit::clicks::Clicks;
use crate::edit::pointer::{EditPointer, Extend};
use crate::host::captured::{CapturedPointer, PointerPhase};
use crate::host::probe::Probe;
use dioxus::html::input_data::MouseButton;
use dioxus::prelude::*;
use ds_core::geometry::units::{Point, Px};
use ds_core::vocab::PressPhase;

/// The primary button went down on the surface: tell the app, capture the pointer, take the
/// keyboard.
pub(crate) fn press(
    ctx: &SurfaceCtx,
    event: &PointerEvent,
    captured: EventHandler<CapturedPointer>,
    told: EventHandler<()>,
) {
    if event.trigger_button() != Some(MouseButton::Primary) {
        return;
    }
    // Blitz's default would start its own text selection, and the click after it would clear
    // the focus (FINDINGS "Edit surface").
    event.prevent_default();
    let at = point_of(event);
    let clicks = ctx.state.press(at);
    report(ctx, at, event.modifiers(), PointerPhase::Press, clicks);
    if let (Some(edit), Some(element)) = (ctx.host.edit(), ctx.state.element())
        && edit.capture(&element, captured) == Probe::Found(())
    {
        ctx.state.capture.set(Capture::Held);
    }
    focus_surface(ctx, told);
}

/// A move over the surface, when no capture carries it.
pub(crate) fn moved(ctx: &SurfaceCtx, event: &PointerEvent) {
    if ctx.state.pressing.get() == PressPhase::Idle || ctx.state.capture.get() == Capture::Held {
        return;
    }
    if !event.held_buttons().contains(MouseButton::Primary) {
        ctx.state.pressing.set(PressPhase::Idle);
        return;
    }
    let clicks = ctx.state.last_clicks();
    report(
        ctx,
        point_of(event),
        event.modifiers(),
        PointerPhase::Drag,
        clicks,
    );
}

/// A release over the surface, when no capture carries it.
pub(crate) fn released(ctx: &SurfaceCtx, event: &PointerEvent) {
    if ctx.state.capture.get() == Capture::Held {
        return;
    }
    if ctx.state.pressing.replace(PressPhase::Idle) == PressPhase::Pressed {
        let clicks = ctx.state.last_clicks();
        report(
            ctx,
            point_of(event),
            event.modifiers(),
            PointerPhase::Release,
            clicks,
        );
    }
}

/// A move or the release the host routed to the surface while it holds the capture.
pub(crate) fn captured(ctx: &SurfaceCtx, pointer: CapturedPointer) {
    if ctx.state.pressing.get() == PressPhase::Idle {
        return;
    }
    if pointer.phase == PointerPhase::Release {
        ctx.state.pressing.set(PressPhase::Idle);
        ctx.state.capture.set(Capture::Free);
    }
    let clicks = ctx.state.last_clicks();
    report(ctx, pointer.at, pointer.modifiers, pointer.phase, clicks);
}

/// Tell the app about a pointer event at `at`, with the text position the host finds there.
fn report(ctx: &SurfaceCtx, at: Point, modifiers: Modifiers, phase: PointerPhase, clicks: Clicks) {
    let Some(on_pointer) = ctx.on_pointer else {
        return;
    };
    let position = match (ctx.host.edit(), ctx.state.element()) {
        (Some(edit), Some(element)) => edit.hit_test(&element, at).found(),
        _ => None,
    };
    let extend = if modifiers.contains(Modifiers::SHIFT) {
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

fn point_of(event: &PointerEvent) -> Point {
    let at = event.client_coordinates();
    Point {
        x: Px(at.x as f32),
        y: Px(at.y as f32),
    }
}

/// Where a mouse event (a right-click's `contextmenu`) happened, in the window.
pub(crate) fn point_of_mouse(event: &MouseEvent) -> Point {
    let at = event.client_coordinates();
    Point {
        x: Px(at.x as f32),
        y: Px(at.y as f32),
    }
}
