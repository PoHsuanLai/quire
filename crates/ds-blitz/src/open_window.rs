//! A second window from a running app (a message in its own window).
//!
//! [`open_window`] asks the event loop `launch` runs for a new window with its own VirtualDom.
//! The new document gets exactly what `launch` gives the first one: the same `Host` around the
//! root (input modality, focus seams, caret, click focus, the edit surface's IME, the window
//! frame's `WindowHost`, the scale, file drops), the app's `AppConfig` setup (its root contexts,
//! net policy, frame links, focus fallback), the quire faces, and dioxus-native's own window
//! contexts (the document, `use_window`, `use_window_event`). Its root renders its own `Ds`, as
//! the first window's does, with the appearance it reads from the shared contexts.
//!
//! A `Signal` belongs to one VirtualDom and cannot cross into another: pass data by props
//! ([`open_window_with`]) or through shared state both windows read (an `Arc<Mutex<_>>`, a
//! `tokio::sync::watch` channel, or the app's own store provided with `AppConfig::with_context`).

use crate::app_id::AppId;
use crate::error::OpenWindowError;
use crate::window::Decorations;
use crate::window_requests::{Requests, Root, WindowKey, WindowLife};
use crate::window_size::WindowSize;
use dioxus::prelude::*;
use std::rc::Rc;

/// How a window starts: the first one `launch` opens (inside its `AppConfig`) and any opened
/// later with [`open_window`].
#[derive(Debug, Clone, PartialEq, Eq)]
pub struct WindowSpec {
    title: String,
    size: WindowSize,
    /// `None`: the first window's.
    app_id: Option<AppId>,
    /// `None`: the first window's.
    decorations: Option<Decorations>,
}

impl WindowSpec {
    /// A window of `size` titled `title`, with the first window's application id and
    /// decorations.
    pub fn new(title: impl Into<String>, size: WindowSize) -> Self {
        WindowSpec {
            title: title.into(),
            size,
            app_id: None,
            decorations: None,
        }
    }

    /// A desktop application id of its own, instead of the first window's.
    pub fn with_app_id(mut self, id: AppId) -> Self {
        self.app_id = Some(id);
        self
    }

    /// Who draws its frame, instead of the first window's choice.
    pub fn with_decorations(mut self, decorations: Decorations) -> Self {
        self.decorations = Some(decorations);
        self
    }

    /// The window's title.
    pub fn title(&self) -> &str {
        &self.title
    }

    /// What it opens at, and the least it may be resized to.
    pub fn size(&self) -> WindowSize {
        self.size
    }

    /// The application id, falling back to `first`'s.
    pub(crate) fn app_id_or(&self, first: Option<&AppId>) -> Option<AppId> {
        self.app_id.clone().or_else(|| first.cloned())
    }

    /// The decorations, falling back to `first`.
    pub(crate) fn decorations_or(&self, first: Decorations) -> Decorations {
        self.decorations.unwrap_or(first)
    }
}

/// A window opened with [`open_window`]. Dropping the handle leaves the window open.
#[derive(Clone)]
pub struct WindowHandle {
    key: WindowKey,
    requests: Requests,
}

impl std::fmt::Debug for WindowHandle {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        f.debug_struct("WindowHandle")
            .field("key", &self.key)
            .field("life", &self.life())
            .finish()
    }
}

impl WindowHandle {
    /// The handle of the window `requests` knows as `key`.
    pub(crate) fn new(key: WindowKey, requests: Requests) -> Self {
        WindowHandle { key, requests }
    }

    /// Close the window: it leaves the screen and its VirtualDom is dropped. Closing one already
    /// closed does nothing.
    pub fn close(&self) {
        self.requests.close(self.key);
    }

    /// Ask the platform to raise the window and give it the keyboard: X11 does; winit's request
    /// does nothing on Wayland, where only an activation token focuses a window
    /// ([`focus_with_token`](WindowHandle::focus_with_token)).
    pub fn focus(&self) {
        self.requests.focus(self.key, None);
    }

    /// Raise the window and give it the keyboard with an activation token: the one a launcher, a
    /// notification click or a second launch's D-Bus `Activate` (`platform_data`'s
    /// `activation-token`) handed the running app. On Wayland the token goes to the compositor
    /// as xdg-activation's `activate` for this window's surface, which is the only way to raise
    /// a window that exists; on X11, with `None`, with an empty token, or when the compositor
    /// offers no xdg-activation, it is [`focus`](WindowHandle::focus). A token is good once.
    pub fn focus_with_token(&self, token: Option<String>) {
        self.requests.focus(self.key, token);
    }

    /// Where the window is in its life.
    pub fn life(&self) -> WindowLife {
        self.requests.life(self.key)
    }
}

/// The handle of the window the calling component renders in, the first window included: its
/// owner can raise it ([`WindowHandle::focus`]) or close it. `None` outside a window `launch`
/// runs (the harness, a snapshot), where there is no event loop to ask.
pub fn use_window_handle() -> Option<WindowHandle> {
    use_hook(try_consume_context::<WindowHandle>)
}

/// Open another window of this app, rendering `root` in a VirtualDom of its own. Call it from a
/// component or a handler inside a window `launch` opened (or one this opened); anywhere else
/// (the harness, a snapshot) there is no event loop to ask and it answers
/// [`OpenWindowError::NoHost`].
pub fn open_window(
    spec: WindowSpec,
    root: fn() -> Element,
) -> Result<WindowHandle, OpenWindowError> {
    open(spec, Root::Plain(root))
}

/// Open another window rendering the component `root` with `props`: the way to hand the new
/// window its data (a message id, an `Arc` of shared state).
pub fn open_window_with<P: Clone + 'static>(
    spec: WindowSpec,
    root: fn(P) -> Element,
    props: P,
) -> Result<WindowHandle, OpenWindowError> {
    open(spec, Root::Shared(Rc::new(move || root(props.clone()))))
}

fn open(spec: WindowSpec, root: Root) -> Result<WindowHandle, OpenWindowError> {
    let requests = try_consume_context::<Requests>().ok_or(OpenWindowError::NoHost)?;
    let key = requests.open(spec, root);
    Ok(WindowHandle { key, requests })
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::window_requests::Request;
    use std::cell::RefCell;

    /// What a probe component saw of its window.
    #[derive(Clone)]
    struct Seen(Rc<RefCell<Option<WindowHandle>>>);

    #[allow(non_snake_case)]
    fn Probe() -> Element {
        let seen = use_context::<Seen>();
        seen.0.replace(use_window_handle());
        rsx! {}
    }

    fn nothing() -> Element {
        rsx! {}
    }

    /// Render the probe in a document whose window is `handle`, or in none.
    fn seen_in(handle: Option<WindowHandle>) -> Option<WindowHandle> {
        let seen = Seen(Rc::new(RefCell::new(None)));
        let mut vdom = VirtualDom::new(Probe);
        vdom.provide_root_context(seen.clone());
        if let Some(handle) = handle {
            vdom.provide_root_context(handle);
        }
        vdom.rebuild_in_place();
        seen.0.take()
    }

    #[test]
    fn a_component_reads_the_handle_of_the_window_it_renders_in() {
        let requests = Requests::new(|| {});
        let first = requests.open(
            WindowSpec::new("First", WindowSize::new(100, 100)),
            Root::Plain(nothing),
        );
        let second = requests.open(
            WindowSpec::new("Second", WindowSize::new(100, 100)),
            Root::Plain(nothing),
        );
        requests.take();

        let handle = seen_in(Some(WindowHandle::new(second, requests.clone())))
            .expect("the document provided its window's handle");
        handle.focus();

        let keys: Vec<WindowKey> = requests
            .take()
            .into_iter()
            .filter_map(|request| match request {
                Request::Focus { key, token: None } => Some(key),
                _ => None,
            })
            .collect();
        assert_eq!(
            keys,
            [second],
            "the focus request names the component's window, not {first:?}"
        );
    }

    #[test]
    fn a_token_travels_with_the_focus_request_of_its_window() {
        let requests = Requests::new(|| {});
        let key = requests.open(
            WindowSpec::new("Only", WindowSize::new(100, 100)),
            Root::Plain(nothing),
        );
        requests.take();
        let handle = WindowHandle::new(key, requests.clone());

        handle.focus_with_token(Some("tok".to_owned()));
        handle.focus();

        let asked: Vec<(WindowKey, Option<String>)> = requests
            .take()
            .into_iter()
            .filter_map(|request| match request {
                Request::Focus { key, token } => Some((key, token)),
                _ => None,
            })
            .collect();
        assert_eq!(asked, [(key, Some("tok".to_owned())), (key, None)]);
    }

    #[test]
    fn a_document_outside_a_launched_window_has_no_handle() {
        assert!(seen_in(None).is_none());
    }
}
