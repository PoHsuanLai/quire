//! The screen a new window opens on, in logical pixels, read from the event loop's monitors.
//!
//! winit reports a monitor's size, not its work area: the panel, dock or menu bar a compositor
//! reserves is not subtracted (Wayland has no way for a client to ask, and winit offers none on
//! X11 either). So the extent is the whole monitor, which is why a window fitted to it is capped
//! to 85% of it. winit also does not say which monitor the next window opens on (the compositor
//! decides on Wayland), so it is the primary monitor, or the first one listed when there is no
//! primary. On Wayland the monitor's scale is the integer `wl_output` scale, so on a
//! fractionally scaled output the logical size is computed at the integer scale and is off by
//! the difference.

use crate::window_size::Extent;
use dioxus_native::winit::event_loop::ActiveEventLoop;
use dioxus_native::winit::monitor::MonitorHandle;

/// The logical extent of the monitor a new window is likeliest to open on, if winit lists one.
pub(crate) fn screen_extent(event_loop: &dyn ActiveEventLoop) -> Option<Extent> {
    event_loop
        .primary_monitor()
        .into_iter()
        .chain(event_loop.available_monitors())
        .find_map(|monitor| extent_of(&monitor))
}

fn extent_of(monitor: &MonitorHandle) -> Option<Extent> {
    let size = monitor.current_video_mode()?.size();
    Extent::new(size.width, size.height).logical_at(monitor.scale_factor())
}
