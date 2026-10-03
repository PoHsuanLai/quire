//! The window's application id: the Wayland `app_id` (and the X11 `WM_CLASS`) a desktop matches
//! against the app's `.desktop` file, for its icon, its name in the task switcher and its
//! grouping. `crate::window_platform` hands it to winit.

/// A desktop application id, the `.desktop` file's name without the extension:
/// `AppId("dev.mailo.Mailo".into())`.
#[derive(Debug, Clone, PartialEq, Eq, Hash)]
pub struct AppId(pub String);
