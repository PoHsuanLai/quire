//! The window's application id: the Wayland `app_id` (and the X11 `WM_CLASS`) a desktop matches
//! against the app's `.desktop` file, for its icon, its name in the task switcher and its
//! grouping. winit takes it as platform attributes, one backend's at a time, so the backend is
//! chosen the way winit's own event loop chooses it.

use dioxus_native::WindowAttributes;

/// A desktop application id, the `.desktop` file's name without the extension:
/// `AppId("dev.mailo.Mailo".into())`.
#[derive(Debug, Clone, PartialEq, Eq, Hash)]
pub struct AppId(pub String);

/// `window` with `id` as its application id, on the backend winit will open it on.
#[cfg(target_os = "linux")]
pub(crate) fn with_app_id(window: WindowAttributes, id: &AppId) -> WindowAttributes {
    use dioxus_native::winit::platform::{
        wayland::WindowAttributesWayland, x11::WindowAttributesX11,
    };
    let AppId(id) = id;
    match backend() {
        Backend::Wayland => window.with_platform_attributes(Box::new(
            WindowAttributesWayland::default().with_name(id.as_str(), id.as_str()),
        )),
        Backend::X11 => window.with_platform_attributes(Box::new(
            WindowAttributesX11::default().with_name(id.as_str(), id.as_str()),
        )),
    }
}

/// Elsewhere the application id comes from the bundle, not the window.
#[cfg(not(target_os = "linux"))]
pub(crate) fn with_app_id(window: WindowAttributes, _id: &AppId) -> WindowAttributes {
    window
}

/// The Linux display server winit opens its window on.
#[cfg(target_os = "linux")]
enum Backend {
    Wayland,
    X11,
}

/// Wayland whenever `WAYLAND_DISPLAY` or `WAYLAND_SOCKET` is set and not empty, as winit's
/// event loop decides; X11 otherwise.
#[cfg(target_os = "linux")]
fn backend() -> Backend {
    let set = |name: &str| std::env::var(name).is_ok_and(|value| !value.is_empty());
    if set("WAYLAND_DISPLAY") || set("WAYLAND_SOCKET") {
        Backend::Wayland
    } else {
        Backend::X11
    }
}
