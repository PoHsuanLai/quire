//! The root `launch` runs: the app, plus what a window host owes quire.
//!
//! - The input modality (spike S12): every key press (other than a lone modifier) makes it
//!   `keyboard`, every pointer press makes it `pointer`; `Ds` reads it through `HostModality`
//!   and stamps `data-modality`.
//! - The document's providers become the app's (`crate::install`): the net policy, for the
//!   document and every frame it builds, waking the shell to paint when a resource lands
//!   (S7/S8). The app renders once they are in place, a frame after the host.
//! - Rect reads go through `ds::HostMeasure`, which waits out a document the renderer is
//!   holding instead of panicking (`crate::measure`); focus changes go through `ds::HostFocus`
//!   the same way (`crate::focus`); an edit surface's geometry and IME through `ds::HostEdit`
//!   (`crate::edit`).
//! - IME events: dioxus-native-dom drops them, but this window hook hears each winit event
//!   before the document does, so an IME event goes to the edit surface that has the keyboard
//!   (`crate::edit_ime`).
//! - Before each frame, the viewport's colour scheme (and the window's decorations) follow the
//!   scheme the root `.ds` resolved.
//! - The window's scale factor, as `ds::HostScale`, so `Ds` writes the pixel tokens for it and a
//!   hairline is one device pixel wide. The window path cannot snap positions (blitz-shell
//!   resolves and paints in one call, with nothing between; FINDINGS "Pixel snapping"), so at a
//!   fractional scale a line may still start half-way through a device pixel here.
//!
//! The document is reached through a hidden element's `onmounted` handle: dioxus-native builds
//! the document itself and hands the app nothing else that can see it.

use crate::clipboard::HostClipboard;
use crate::edit_ime::{EditListeners, ime_of};
use crate::frame_links::frame_links;
use crate::install::install;
use crate::node_ref::DocRef;
use crate::scheme;
use crate::setup::Setup;
use blitz_traits::shell::ColorScheme;
use dioxus::prelude::*;
use dioxus_native::winit::event::{ElementState, WindowEvent};
use dioxus_native::winit::keyboard::{Key as WinitKey, NamedKey};
use dioxus_native::winit::window::Theme;
use dioxus_native::{use_window, use_window_event};
use dioxus_native_dom::NodeHandle;
use ds::{HostModality, HostScale, InputModality, Scale};
use std::cell::RefCell;
use std::rc::Rc;

/// What `Host` wraps, and what the app gave its document.
#[derive(Props, Debug, Clone)]
pub(crate) struct HostProps {
    app: fn() -> Element,
    /// Fixed for the window's life: read once, as the document mounts.
    setup: Setup,
}

impl HostProps {
    pub(crate) fn new(app: fn() -> Element, setup: Setup) -> Self {
        HostProps { app, setup }
    }
}

impl PartialEq for HostProps {
    /// The same app is the same root: `setup` never changes after `launch`.
    fn eq(&self, other: &Self) -> bool {
        std::ptr::fn_addr_eq(self.app, other.app)
    }
}

/// The app with the host's modality, net provider and scheme around it.
#[allow(non_snake_case)] // A component: rsx and launch name it like a type.
pub(crate) fn Host(props: HostProps) -> Element {
    let modality = use_context_provider(|| HostModality(Signal::new(InputModality::default())));
    use_context_provider(|| crate::measure::MEASURE);
    use_context_provider(|| crate::focus::FOCUS);
    use_context_provider(|| crate::focus::BLUR);
    use_context_provider(|| crate::focus::SELECT);
    use_context_provider(|| crate::edit::EDIT);
    let listeners = use_context_provider(EditListeners::default);
    let clipboard = use_context_provider(HostClipboard::default);
    let document = use_hook(|| Rc::new(RefCell::new(None::<NodeHandle>)));
    let found = Rc::clone(&document);
    use_context_provider(move || {
        crate::focus::finder(move || found.borrow().clone().map(DocRef::Handle))
    });
    let window = use_window();
    let factor = window.scale_factor();
    let scale = use_context_provider(|| HostScale(Signal::new(scale_of(factor))));
    let seen = Rc::clone(&document);
    use_window_event(move |event, _| {
        if let WindowEvent::ScaleFactorChanged { scale_factor, .. } = event {
            let HostScale(mut current) = scale;
            let next = scale_of(*scale_factor);
            if *current.peek() != next {
                current.set(next);
            }
        }
        if let Some(next) = modality_after(event) {
            let HostModality(mut current) = modality;
            if *current.peek() != next {
                current.set(next);
            }
        }
        if let WindowEvent::Ime(ime) = event
            && let Some(ime) = ime_of(ime)
        {
            let sink = seen
                .borrow()
                .as_ref()
                .and_then(|handle| handle.try_doc().and_then(|doc| listeners.target(&doc)));
            if let Some(sink) = sink {
                sink.call(ime);
            }
        }
        if matches!(event, WindowEvent::RedrawRequested)
            && let Some(handle) = seen.borrow().as_ref()
            && let Some(changed) = scheme::follow_root(&mut handle.doc_mut())
        {
            window.set_theme(Some(theme(changed)));
        }
    });
    let frame_nav = use_hook(|| {
        let (nav, inbox) = frame_links(&props.setup.frame_links);
        spawn(inbox.serve());
        nav
    });
    let mut installed = use_signal(|| Installed::Pending);
    let setup = props.setup.clone();
    let App = props.app;
    rsx! {
        // The app waits one frame for its document's providers: a frame in its first render
        // would otherwise be parsed with the parent's `file:` provider (`crate::frames`).
        if installed() == Installed::Done {
            App {}
        }
        div {
            style: "display:none",
            onmounted: move |mounted| {
                if let Some(handle) = mounted.data().downcast::<NodeHandle>() {
                    install(handle, &setup, &clipboard, frame_nav.clone());
                    document.replace(Some(handle.clone()));
                    installed.set(Installed::Done);
                }
            },
        }
    }
}

/// Whether the document has the app's providers yet.
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
enum Installed {
    /// Not yet: the app is not rendered.
    Pending,
    /// Yes: the app renders.
    Done,
}

/// A winit scale factor in 120ths, the unit `ds::Scale` shares with the Wayland protocol.
fn scale_of(factor: f64) -> Scale {
    Scale((factor * f64::from(Scale::DENOMINATOR)).round().max(1.0) as u32)
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
