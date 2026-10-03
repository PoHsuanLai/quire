//! Raising a window that exists, with or without an activation token. On X11, and with no
//! token, it is winit's `focus_window` (`_NET_ACTIVE_WINDOW`); on Wayland a token is the only
//! way to ask, and it goes to the compositor as xdg-activation's `activate`
//! (`crate::xdg_activation`) for the window's own surface. A request that cannot be sent falls
//! back to `focus_window`, which a Wayland compositor ignores.

use dioxus_native::winit::window::Window;
use raw_window_handle::{HasDisplayHandle, RawDisplayHandle};

/// How a window is raised.
#[derive(Debug, Clone, PartialEq, Eq)]
enum Route {
    /// Ask winit.
    Focus,
    /// Hand the compositor this token for the window's surface.
    Token(String),
}

/// A token travels only on Wayland, and an empty one is no token.
fn route(token: Option<String>, display: Option<RawDisplayHandle>) -> Route {
    match (token, display) {
        (Some(token), Some(RawDisplayHandle::Wayland(_))) if !token.is_empty() => {
            Route::Token(token)
        }
        _ => Route::Focus,
    }
}

/// Raise `window`; with `token`, as the activation the compositor was handed it for.
pub(crate) fn raise(window: &dyn Window, token: Option<String>) {
    let display = window.display_handle().ok().map(|handle| handle.as_raw());
    match route(token, display) {
        Route::Focus => window.focus_window(),
        Route::Token(token) => {
            if activate(window, &token).is_err() {
                window.focus_window();
            }
        }
    }
}

#[cfg(target_os = "linux")]
fn activate(window: &dyn Window, token: &str) -> Result<(), ActivateFailed> {
    use crate::{wayland_surface::adopt, xdg_activation};
    use raw_window_handle::HasWindowHandle;
    let adopted = adopt(window.display_handle()?, window.window_handle()?)?;
    Ok(xdg_activation::activate(&adopted, token)?)
}

/// Elsewhere there is no Wayland to ask.
#[cfg(not(target_os = "linux"))]
fn activate(_window: &dyn Window, _token: &str) -> Result<(), ActivateFailed> {
    Err(ActivateFailed::NotLinux)
}

/// Why a token could not be handed over.
#[derive(Debug, thiserror::Error)]
enum ActivateFailed {
    #[error("the window has no handle: {0}")]
    Handle(#[from] raw_window_handle::HandleError),
    #[cfg(target_os = "linux")]
    #[error(transparent)]
    Adopt(#[from] crate::wayland_surface::AdoptError),
    #[cfg(target_os = "linux")]
    #[error(transparent)]
    Activation(#[from] crate::xdg_activation::ActivationError),
    #[cfg(not(target_os = "linux"))]
    #[error("only Linux has xdg-activation")]
    NotLinux,
}

#[cfg(test)]
mod tests {
    use super::*;
    use raw_window_handle::{WaylandDisplayHandle, XlibDisplayHandle};
    use std::ptr::NonNull;

    fn wayland() -> Option<RawDisplayHandle> {
        Some(RawDisplayHandle::Wayland(WaylandDisplayHandle::new(
            NonNull::dangling(),
        )))
    }

    fn x11() -> Option<RawDisplayHandle> {
        Some(RawDisplayHandle::Xlib(XlibDisplayHandle::new(None, 0)))
    }

    fn token(raw: &str) -> Option<String> {
        Some(raw.to_owned())
    }

    #[test]
    fn a_token_on_wayland_is_handed_to_the_compositor() {
        assert_eq!(
            route(token("abc"), wayland()),
            Route::Token("abc".to_owned())
        );
    }

    #[test]
    fn no_token_is_a_plain_focus() {
        assert_eq!(route(None, wayland()), Route::Focus);
        assert_eq!(route(None, x11()), Route::Focus);
    }

    #[test]
    fn an_empty_token_is_no_token() {
        assert_eq!(route(token(""), wayland()), Route::Focus);
    }

    #[test]
    fn x11_focuses_through_winit_whatever_the_token() {
        assert_eq!(route(token("abc"), x11()), Route::Focus);
    }

    #[test]
    fn a_window_with_no_display_handle_is_focused_through_winit() {
        assert_eq!(route(token("abc"), None), Route::Focus);
    }
}
