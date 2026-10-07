//! The screen a window opens on or is on, read from winit's monitors.
//!
//! What a client can learn on Wayland, which winit reports as it is:
//! - The output's size is the current mode's pixels (`wl_output.mode`), not the logical size
//!   xdg-output reports, and winit does not apply the output's transform: a rotated output reads
//!   with its axes unswapped.
//! - The output's scale is the integer `wl_output` scale. A window that has been mapped knows
//!   better: `Window::scale_factor` follows `wp_fractional_scale_v1`, so a window's own area
//!   ([`window_area`]) is exact on a fractionally scaled output where the monitor's is not.
//! - The panel, dock or menu bar a compositor reserves (layer-shell exclusive zones) is never
//!   sent to clients, so the work area is the whole output (`WorkBasis::WholeOutput`), and a
//!   fitted window is capped to 85% of it. winit has no work-area query on X11 either. Under
//!   our shell the app can learn it anyway: `AppHandle::set_desktop_outputs` feeds what
//!   `ds-desktop` read from `org.quire.Outputs1` (`WorkBasis::Desktop`).
//! - Which output a new window opens on is the compositor's choice and is not announced: before
//!   the window exists it is the primary monitor, or the first one listed on Wayland (which has
//!   no primary); once the window is mapped, `Window::current_monitor` is its output.

use crate::screen_area::{ScreenArea, ScreenOf};
use crate::window_size::Extent;
use dioxus_native::winit::event_loop::ActiveEventLoop;
use dioxus_native::winit::monitor::MonitorHandle;
use dioxus_native::winit::window::Window;
use ds::prelude::Scale;
use ds_desktop::Outputs;

/// The area of the monitor a new window is likeliest to open on (the primary monitor, or the
/// first listed), if winit lists one.
///
/// `desktop` is what the shell reported (empty without it): the work area and scale it gives
/// replace the guess below for the output they match.
pub(crate) fn screen_area(
    event_loop: &dyn ActiveEventLoop,
    desktop: &Outputs,
) -> Option<ScreenArea> {
    event_loop
        .primary_monitor()
        .into_iter()
        .chain(event_loop.available_monitors())
        .find_map(|monitor| {
            area_of(&monitor, monitor.scale_factor(), ScreenOf::Primary)
                .map(|area| on_desktop(area, &monitor, desktop))
        })
}

/// The area of the output `window` is on, at the scale it draws at; with no output known yet,
/// the primary monitor's.
pub(crate) fn window_area(window: &dyn Window) -> Option<ScreenArea> {
    let on = window
        .current_monitor()
        .and_then(|monitor| area_of(&monitor, window.scale_factor(), ScreenOf::Window));
    on.or_else(|| {
        window
            .primary_monitor()
            .into_iter()
            .chain(window.available_monitors())
            .find_map(|monitor| area_of(&monitor, monitor.scale_factor(), ScreenOf::Primary))
    })
}

/// `area` of `monitor`, as the shell reports that output when it does.
fn on_desktop(area: ScreenArea, monitor: &MonitorHandle, desktop: &Outputs) -> ScreenArea {
    let physical = monitor
        .current_video_mode()
        .map(|mode| (mode.size().width, mode.size().height));
    area.on_desktop(physical.and_then(|size| desktop.of_physical(size)))
}

fn area_of(monitor: &MonitorHandle, factor: f64, of: ScreenOf) -> Option<ScreenArea> {
    let size = monitor.current_video_mode()?.size();
    let scale = Scale::from_factor(factor)?;
    ScreenArea::new(Extent::new(size.width, size.height), scale, of)
}
