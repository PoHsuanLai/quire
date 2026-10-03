//! What a new window asks of its platform at creation: the application id the desktop matches to
//! the app's `.desktop` file, and the activation token the app was started with, which lets the
//! compositor give the window the keyboard. winit takes both as platform attributes, one
//! backend's at a time, so the backend is chosen the way winit's own event loop chooses it.

use crate::app_id::AppId;
use dioxus_native::WindowAttributes;
use dioxus_native::winit::window::ActivationToken;

/// `window` with `id` as its application id and `token` as its activation token, on the backend
/// winit will open it on. Either may be absent.
pub(crate) fn with_platform(
    window: WindowAttributes,
    id: Option<&AppId>,
    token: Option<ActivationToken>,
) -> WindowAttributes {
    with_platform_on(backend(), window, id, token)
}

/// The Linux display server winit opens its window on.
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub(crate) enum Backend {
    Wayland,
    X11,
}

#[cfg(target_os = "linux")]
pub(crate) fn with_platform_on(
    on: Backend,
    window: WindowAttributes,
    id: Option<&AppId>,
    token: Option<ActivationToken>,
) -> WindowAttributes {
    use dioxus_native::winit::platform::{
        wayland::WindowAttributesWayland, x11::WindowAttributesX11,
    };
    if id.is_none() && token.is_none() {
        return window;
    }
    match on {
        Backend::Wayland => {
            let named = match id {
                Some(AppId(id)) => {
                    WindowAttributesWayland::default().with_name(id.as_str(), id.as_str())
                }
                None => WindowAttributesWayland::default(),
            };
            let attributes = match token {
                Some(token) => named.with_activation_token(token),
                None => named,
            };
            window.with_platform_attributes(Box::new(attributes))
        }
        Backend::X11 => {
            let named = match id {
                Some(AppId(id)) => {
                    WindowAttributesX11::default().with_name(id.as_str(), id.as_str())
                }
                None => WindowAttributesX11::default(),
            };
            let attributes = match token {
                Some(token) => named.with_activation_token(token),
                None => named,
            };
            window.with_platform_attributes(Box::new(attributes))
        }
    }
}

/// Elsewhere the application id comes from the bundle and focus from the system, not the window.
#[cfg(not(target_os = "linux"))]
pub(crate) fn with_platform_on(
    _on: Backend,
    window: WindowAttributes,
    _id: Option<&AppId>,
    _token: Option<ActivationToken>,
) -> WindowAttributes {
    window
}

/// Wayland whenever `WAYLAND_DISPLAY` or `WAYLAND_SOCKET` is set and not empty, as winit's
/// event loop decides; X11 otherwise.
fn backend() -> Backend {
    let set = |name: &str| std::env::var(name).is_ok_and(|value| !value.is_empty());
    if set("WAYLAND_DISPLAY") || set("WAYLAND_SOCKET") {
        Backend::Wayland
    } else {
        Backend::X11
    }
}

#[cfg(all(test, target_os = "linux"))]
mod tests {
    use super::*;

    fn platform_of(attributes: &WindowAttributes) -> String {
        format!("{:?}", attributes.platform)
    }

    fn token(raw: &str) -> Option<ActivationToken> {
        Some(ActivationToken::from_raw(raw.to_owned()))
    }

    #[test]
    fn a_wayland_window_carries_its_id_and_token() {
        let id = AppId("dev.quire.Test".to_owned());
        let attributes = with_platform_on(
            Backend::Wayland,
            WindowAttributes::default(),
            Some(&id),
            token("wl-token-1"),
        );
        let text = platform_of(&attributes);
        assert!(text.contains("WindowAttributesWayland"), "{text}");
        assert!(text.contains("dev.quire.Test"), "{text}");
        assert!(text.contains("wl-token-1"), "{text}");
    }

    #[test]
    fn an_x11_window_carries_its_id_and_token() {
        let id = AppId("dev.quire.Test".to_owned());
        let attributes = with_platform_on(
            Backend::X11,
            WindowAttributes::default(),
            Some(&id),
            token("x-startup-1"),
        );
        let text = platform_of(&attributes);
        assert!(text.contains("WindowAttributesX11"), "{text}");
        assert!(text.contains("dev.quire.Test"), "{text}");
        assert!(text.contains("x-startup-1"), "{text}");
    }

    #[test]
    fn a_token_alone_names_no_application() {
        let attributes = with_platform_on(
            Backend::Wayland,
            WindowAttributes::default(),
            None,
            token("wl-token-2"),
        );
        let text = platform_of(&attributes);
        assert!(text.contains("wl-token-2"), "{text}");
        assert!(text.contains("name: None"), "{text}");
    }

    #[test]
    fn nothing_to_say_leaves_the_platform_attributes_unset() {
        for on in [Backend::Wayland, Backend::X11] {
            let attributes = with_platform_on(on, WindowAttributes::default(), None, None);
            assert_eq!(platform_of(&attributes), "None", "{on:?}");
        }
    }
}
