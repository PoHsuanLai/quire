//! The activation token a process is started with (`XDG_ACTIVATION_TOKEN` on Wayland,
//! `DESKTOP_STARTUP_ID` on X11): a launcher, a notification click or a D-Bus activation hands it
//! over so the compositor lets the new window take the keyboard. It is good for one window, so the
//! first window created takes it and the environment is cleared, which also keeps child
//! processes from inheriting a token that is spent.

use dioxus_native::winit::event_loop::ActiveEventLoop;
#[cfg(target_os = "linux")]
use dioxus_native::winit::platform::startup_notify::{
    EventLoopExtStartupNotify, reset_activation_token_env,
};
use dioxus_native::winit::window::ActivationToken;

/// The token the process was started with, taken: reading it clears the environment, so the
/// next call answers `None`. An empty variable is no token.
#[cfg(target_os = "linux")]
pub(crate) fn take_startup_token(event_loop: &dyn ActiveEventLoop) -> Option<ActivationToken> {
    take(
        || event_loop.read_token_from_env(),
        reset_activation_token_env,
    )
}

#[cfg(not(target_os = "linux"))]
pub(crate) fn take_startup_token(_event_loop: &dyn ActiveEventLoop) -> Option<ActivationToken> {
    None
}

/// `read`'s token, with `reset` run once it has been read and not before.
#[cfg(any(target_os = "linux", test))]
fn take(
    read: impl FnOnce() -> Option<ActivationToken>,
    reset: impl FnOnce(),
) -> Option<ActivationToken> {
    let token = read().filter(|token| !token.as_raw().is_empty())?;
    reset();
    Some(token)
}

#[cfg(test)]
mod tests {
    use super::*;
    use std::cell::Cell;

    /// What `take` answered, and how many times it reset the environment.
    fn run(read: Option<&str>) -> (Option<String>, u32) {
        let resets = Cell::new(0);
        let token = take(
            || read.map(|raw| ActivationToken::from_raw(raw.to_owned())),
            || resets.set(resets.get() + 1),
        );
        (token.map(ActivationToken::into_raw), resets.get())
    }

    #[test]
    fn a_token_is_taken_and_the_environment_cleared_once() {
        assert_eq!(run(Some("abc")), (Some("abc".to_owned()), 1));
    }

    #[test]
    fn no_variable_is_no_token_and_leaves_the_environment_alone() {
        assert_eq!(run(None), (None, 0));
    }

    #[test]
    fn an_empty_variable_is_no_token_and_leaves_the_environment_alone() {
        assert_eq!(run(Some("")), (None, 0));
    }
}
