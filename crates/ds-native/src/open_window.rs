//! A second window from a running app (mailo item 8: a message in its own window).
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
use dioxus::prelude::*;
use std::rc::Rc;

/// How a window opened with [`open_window`] starts.
#[derive(Debug, Clone, PartialEq, Eq)]
pub struct WindowSpec {
    title: String,
    width: u32,
    height: u32,
    /// `None`: the first window's.
    app_id: Option<AppId>,
    /// `None`: the first window's.
    decorations: Option<Decorations>,
}

impl WindowSpec {
    /// A `width` x `height` window (logical pixels) titled `title`, with the first window's
    /// application id and decorations.
    pub fn new(title: impl Into<String>, width: u32, height: u32) -> Self {
        WindowSpec {
            title: title.into(),
            width,
            height,
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

    /// The initial size in logical pixels, width then height.
    pub(crate) fn size(&self) -> (u32, u32) {
        (self.width, self.height)
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
    /// Close the window: it leaves the screen and its VirtualDom is dropped. Closing one already
    /// closed does nothing.
    pub fn close(&self) {
        self.requests.close(self.key);
    }

    /// Raise the window and give it the keyboard (the compositor may only mark it as wanting
    /// attention; Wayland without an activation token does).
    pub fn focus(&self) {
        self.requests.focus(self.key);
    }

    /// Where the window is in its life.
    pub fn life(&self) -> WindowLife {
        self.requests.life(self.key)
    }
}

/// Open another window of this app, rendering `root` in a VirtualDom of its own. Call it from a
/// component or a handler inside a window `launch` opened (or one this opened); anywhere else
/// (the harness, a snapshot, a webview) there is no event loop to ask and it answers
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
