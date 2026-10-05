//! The activation token a process is started with (`XDG_ACTIVATION_TOKEN` on Wayland,
//! `DESKTOP_STARTUP_ID` on X11): a launcher, a notification click or a D-Bus activation hands it
//! over so the compositor lets the new window take the keyboard. It is good for one window, so the
//! first window created takes it, which also keeps child processes from inheriting a token that
//! is spent.
//!
//! The environment is read and cleared once, at the top of `launch::run`, before the process has
//! any thread but the main one: `remove_var` while another thread reads the environment is
//! undefined behaviour on glibc, and the Tokio runtime and the event loop start threads. What was
//! read waits in a [`LaunchTokens`] for the first window, which knows its session by then.

use dioxus_native::winit::event_loop::ActiveEventLoop;
#[cfg(target_os = "linux")]
use dioxus_native::winit::platform::startup_notify::reset_activation_token_env;
#[cfg(target_os = "linux")]
use dioxus_native::winit::platform::wayland::ActiveEventLoopExtWayland;
use dioxus_native::winit::window::ActivationToken;

/// The variable a Wayland launcher sets.
const WAYLAND_VAR: &str = "XDG_ACTIVATION_TOKEN";

/// The variable an X11 launcher sets.
const X11_VAR: &str = "DESKTOP_STARTUP_ID";

/// Which session a window is created in: which variable's token it can use.
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
enum Session {
    Wayland,
    X11,
}

/// The tokens the process was started with, until the first window takes them.
#[derive(Debug, Default)]
pub(crate) struct LaunchTokens {
    wayland: Option<String>,
    x11: Option<String>,
}

impl LaunchTokens {
    /// Read both variables and clear them from the environment. Call it before any thread is
    /// spawned (see the module documentation).
    #[cfg(target_os = "linux")]
    pub(crate) fn from_env() -> Self {
        Self::read(|name| std::env::var(name).ok(), reset_activation_token_env)
    }

    #[cfg(not(target_os = "linux"))]
    pub(crate) fn from_env() -> Self {
        Self::default()
    }

    /// The tokens `var` answers, with `reset` run once both are read and not before. An empty
    /// variable is no token; only a token that was there makes `reset` run.
    #[cfg(any(target_os = "linux", test))]
    fn read(var: impl Fn(&str) -> Option<String>, reset: impl FnOnce()) -> Self {
        let tokens = LaunchTokens {
            wayland: var(WAYLAND_VAR).filter(|raw| !raw.is_empty()),
            x11: var(X11_VAR).filter(|raw| !raw.is_empty()),
        };
        if tokens.wayland.is_some() || tokens.x11.is_some() {
            reset();
        }
        tokens
    }

    /// The token for a window created in `event_loop`'s session, taken: the next call answers
    /// `None`, since a token is good for one window.
    #[cfg(target_os = "linux")]
    pub(crate) fn take(&mut self, event_loop: &dyn ActiveEventLoop) -> Option<ActivationToken> {
        let session = if event_loop.is_wayland() {
            Session::Wayland
        } else {
            Session::X11
        };
        self.take_for(session)
    }

    #[cfg(not(target_os = "linux"))]
    pub(crate) fn take(&mut self, _event_loop: &dyn ActiveEventLoop) -> Option<ActivationToken> {
        None
    }

    /// The token of `session`; both are spent, whichever answered.
    #[cfg(any(target_os = "linux", test))]
    fn take_for(&mut self, session: Session) -> Option<ActivationToken> {
        let (wayland, x11) = (self.wayland.take(), self.x11.take());
        match session {
            Session::Wayland => wayland,
            Session::X11 => x11,
        }
        .map(ActivationToken::from_raw)
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use std::cell::Cell;

    /// The tokens read from `vars`, and how many times the environment was reset.
    fn read(vars: &[(&str, &str)]) -> (LaunchTokens, u32) {
        let resets = Cell::new(0);
        let tokens = LaunchTokens::read(
            |name| {
                vars.iter()
                    .find(|(var, _)| *var == name)
                    .map(|(_, raw)| (*raw).to_owned())
            },
            || resets.set(resets.get() + 1),
        );
        (tokens, resets.get())
    }

    fn raw(token: Option<ActivationToken>) -> Option<String> {
        token.map(ActivationToken::into_raw)
    }

    #[test]
    fn both_variables_are_read_and_the_environment_cleared_once() {
        let (mut tokens, resets) = read(&[(WAYLAND_VAR, "wl"), (X11_VAR, "x")]);
        assert_eq!(resets, 1);
        assert_eq!(
            raw(tokens.take_for(Session::Wayland)).as_deref(),
            Some("wl")
        );
    }

    #[test]
    fn a_window_takes_its_own_sessions_token_and_spends_both() {
        let (mut tokens, _) = read(&[(WAYLAND_VAR, "wl"), (X11_VAR, "x")]);
        assert_eq!(raw(tokens.take_for(Session::X11)).as_deref(), Some("x"));
        assert_eq!(raw(tokens.take_for(Session::X11)), None, "spent");
        assert_eq!(raw(tokens.take_for(Session::Wayland)), None, "both spent");
    }

    #[test]
    fn a_token_for_the_other_session_is_no_token() {
        let (mut tokens, _) = read(&[(X11_VAR, "x")]);
        assert_eq!(raw(tokens.take_for(Session::Wayland)), None);
    }

    #[test]
    fn no_variable_is_no_token_and_leaves_the_environment_alone() {
        let (mut tokens, resets) = read(&[]);
        assert_eq!((raw(tokens.take_for(Session::Wayland)), resets), (None, 0));
    }

    #[test]
    fn an_empty_variable_is_no_token_and_leaves_the_environment_alone() {
        let (mut tokens, resets) = read(&[(WAYLAND_VAR, "")]);
        assert_eq!((raw(tokens.take_for(Session::Wayland)), resets), (None, 0));
    }
}
