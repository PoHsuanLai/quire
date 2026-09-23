//! The root `launch` runs: the app, plus what a window host owes quire.
//!
//! - The input modality (spike S12): every key press (other than a lone modifier) makes it
//!   `keyboard`, every pointer press makes it `pointer`; `Ds` reads it through `HostModality`
//!   and stamps `data-modality`.
//! - The document's net provider becomes ds-native's `data:`/`file:` one, delegating other
//!   schemes to dioxus-native's, and waking the shell to paint when a resource lands (S7/S8).
//! - Before each frame, the viewport's colour scheme (and the window's decorations) follow the
//!   scheme the root `.ds` resolved.
//!
//! The document is reached through a hidden element's `onmounted` handle: dioxus-native builds
//! the document itself and hands the app nothing else that can see it.

use crate::net::DsNet;
use crate::scheme;
use blitz_traits::net::NetWaker;
use blitz_traits::shell::ColorScheme;
use dioxus::prelude::*;
use dioxus_native::winit::event::{ElementState, WindowEvent};
use dioxus_native::winit::keyboard::{Key as WinitKey, NamedKey};
use dioxus_native::winit::window::Theme;
use dioxus_native::{use_window, use_window_event};
use dioxus_native_dom::NodeHandle;
use ds::{HostModality, InputModality};
use std::cell::RefCell;
use std::rc::Rc;
use std::sync::Arc;

/// What `Host` wraps.
#[derive(Props, Debug, Clone, Copy)]
pub(crate) struct HostProps {
    app: fn() -> Element,
}

impl HostProps {
    pub(crate) fn new(app: fn() -> Element) -> Self {
        HostProps { app }
    }
}

impl PartialEq for HostProps {
    fn eq(&self, other: &Self) -> bool {
        std::ptr::fn_addr_eq(self.app, other.app)
    }
}

/// The app with the host's modality, net provider and scheme around it.
#[allow(non_snake_case)] // A component: rsx and launch name it like a type.
pub(crate) fn Host(props: HostProps) -> Element {
    let modality = use_context_provider(|| HostModality(Signal::new(InputModality::default())));
    let document = use_hook(|| Rc::new(RefCell::new(None::<NodeHandle>)));
    let window = use_window();
    let seen = Rc::clone(&document);
    use_window_event(move |event, _| {
        if let Some(next) = modality_after(event) {
            let HostModality(mut current) = modality;
            if *current.peek() != next {
                current.set(next);
            }
        }
        if matches!(event, WindowEvent::RedrawRequested)
            && let Some(handle) = seen.borrow().as_ref()
            && let Some(changed) = scheme::follow_root(&mut handle.doc_mut())
        {
            window.set_theme(Some(theme(changed)));
        }
    });
    let App = props.app;
    rsx! {
        App {}
        div {
            style: "display:none",
            onmounted: move |mounted| {
                if let Some(handle) = mounted.data().downcast::<NodeHandle>() {
                    install_net(handle);
                    document.replace(Some(handle.clone()));
                }
            },
        }
    }
}

/// Serve `data:` and `file:` on the document, waking its shell to paint when one lands.
fn install_net(handle: &NodeHandle) {
    let mut doc = handle.doc_mut();
    let shell = Arc::clone(&doc.shell_provider);
    let waker: Arc<dyn NetWaker> = Arc::new(move |_doc: usize| shell.request_redraw());
    let fallback = Arc::clone(&doc.net_provider);
    doc.set_net_provider(DsNet::shared(Some(fallback), Some(waker)));
}

/// The modality a window event implies, if it implies one.
fn modality_after(event: &WindowEvent) -> Option<InputModality> {
    match event {
        WindowEvent::KeyboardInput { event, .. }
            if event.state == ElementState::Pressed && !is_modifier(&event.logical_key) =>
        {
            Some(InputModality::Keyboard)
        }
        WindowEvent::PointerButton {
            state: ElementState::Pressed,
            ..
        } => Some(InputModality::Pointer),
        _ => None,
    }
}

/// A lone modifier does not show focus rings: it is usually half of a pointer gesture.
fn is_modifier(key: &WinitKey) -> bool {
    matches!(
        key,
        WinitKey::Named(NamedKey::Shift | NamedKey::Control | NamedKey::Alt | NamedKey::Meta)
    )
}

fn theme(scheme: ColorScheme) -> Theme {
    match scheme {
        ColorScheme::Dark => Theme::Dark,
        ColorScheme::Light => Theme::Light,
    }
}
