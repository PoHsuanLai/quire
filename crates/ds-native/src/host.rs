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
//!   and `ds::HostBlur` the same way, and an element named by selector is found through
//!   `ds::HostFind` (`crate::focus`); an edit surface's geometry and IME through `ds::HostEdit`
//!   (`crate::edit`).
//! - Under `FocusFallback::Ancestor` (the default), `ds::HostClickFocus` and
//!   `ds::HostPressFocus`: a click on nothing focusable leaves the keyboard on the nearest
//!   focusable ancestor, and a click a quire control kept to itself on the pressed control
//!   (`crate::click_focus`),
//!   and so does the removal of the focused element (`crate::focus_keep`), looked at on every
//!   window event before the document hears it, so a key after a menu closed reaches the app.
//! - IME events: dioxus-native-dom drops them, but this window hook hears each winit event
//!   before the document does, so an IME event goes to the edit surface that has the keyboard
//!   (`crate::edit_ime`).
//! - Before each frame, the viewport's colour scheme (and the window's decorations) follow the
//!   scheme the root `.ds` resolved.
//! - The window itself, as `ds::WindowHost` over [`crate::window::WinitWindow`]: a frame's
//!   titlebar moves, resizes, zooms, minimizes and closes it; its state is re-read on every
//!   resize and focus change, so the frame redraws when the window is zoomed or deactivated.
//! - Files dragged in from outside (winit's data-transfer events, which blitz-shell ignores), as
//!   `ds::HostFileDrop`: the drag is hit-tested through the document and the target under a
//!   release hears its `ondrop` (`crate::window_drop`, `crate::drop_hit`).
//! - The window's scale factor, as `ds::HostScale`, so `Ds` writes the pixel tokens for it and a
//!   hairline is one device pixel wide. The window path cannot snap positions (blitz-shell
//!   resolves and paints in one call, with nothing between; FINDINGS "Pixel snapping"), so at a
//!   fractional scale a line may still start half-way through a device pixel here.
//!
//! The document is reached through a hidden element's `onmounted` handle: dioxus-native builds
//! the document itself and hands the app nothing else that can see it.

use crate::click_focus::FocusFallback;
use crate::clipboard::HostClipboard;
use crate::edit_ime::{EditListeners, ime_of};
use crate::edit_window::{captured_of, modifiers_of};
use crate::focus_keep::{FocusKeeper, hand_back_seam, keep};
use crate::frame_book::FrameBook;
use crate::frame_hover::report;
use crate::frame_links::{frame_links, read_link};
use crate::install::install;
use crate::node_ref::DocRef;
use crate::scheme;
use crate::setup::Setup;
use crate::window::WinitWindow;
use crate::window_build::WindowSlot;
use crate::window_drop::WindowDrop;
use crate::window_hover::WindowHover;
use crate::window_requests::Root;
use blitz_traits::shell::ColorScheme;
use blitz_traits::shell::ShellProvider;
use dioxus::prelude::*;
use dioxus_native::winit::event::{ElementState, WindowEvent};
use dioxus_native::winit::keyboard::{Key as WinitKey, NamedKey};
use dioxus_native::winit::window::Theme;
use dioxus_native::{use_window, use_window_event};
use dioxus_native_dom::NodeHandle;
use ds::{HostModality, HostScale, InputModality, Scale};
use std::cell::RefCell;
use std::rc::Rc;
use std::sync::Arc;

/// What `Host` wraps, and what the app gave its document.
#[derive(Props, Clone)]
pub(crate) struct HostProps {
    root: Root,
    /// Fixed for the window's life: read once, as the document mounts.
    setup: Setup,
}

impl std::fmt::Debug for HostProps {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        f.debug_struct("HostProps")
            .field("setup", &self.setup)
            .finish_non_exhaustive()
    }
}

impl HostProps {
    pub(crate) fn new(root: Root, setup: Setup) -> Self {
        HostProps { root, setup }
    }
}

impl PartialEq for HostProps {
    /// The same root is the same window: `setup` never changes after `launch`.
    fn eq(&self, other: &Self) -> bool {
        self.root.same(&other.root)
    }
}

/// A root opened with its props (`ds_native::open_window_with`), as a component of its own.
#[derive(Clone)]
struct SharedRoot(Rc<dyn Fn() -> Element>);

impl PartialEq for SharedRoot {
    fn eq(&self, other: &Self) -> bool {
        Rc::ptr_eq(&self.0, &other.0)
    }
}

#[allow(non_snake_case)] // A component: rsx names it like a type.
#[component]
fn Rooted(root: SharedRoot) -> Element {
    (root.0)()
}

/// The app with the host's modality, net provider and scheme around it.
#[allow(non_snake_case)] // A component: rsx and launch name it like a type.
pub(crate) fn Host(props: HostProps) -> Element {
    let modality = use_context_provider(|| HostModality(Signal::new(InputModality::default())));
    use_context_provider(|| crate::measure::MEASURE);
    use_context_provider(|| crate::focus::FOCUS);
    use_context_provider(|| crate::focus::BLUR);
    use_context_provider(|| crate::focus::SELECT);
    use_context_provider(|| crate::focus::CARET);
    use_context_provider(|| crate::focus::PLACE_CARET);
    use_context_provider(|| crate::reveal::REVEAL);
    let fallback = props.setup.focus_fallback;
    let keeper = use_hook(|| match fallback {
        FocusFallback::Ancestor => {
            provide_context(crate::click_focus::CLICK_FOCUS);
            provide_context(crate::click_focus::PRESS_FOCUS);
            let keeper = Rc::new(RefCell::new(FocusKeeper::default()));
            provide_context(hand_back_seam(Rc::clone(&keeper)));
            Some(keeper)
        }
        FocusFallback::BlitzDefault => None,
    });
    use_context_provider(|| crate::edit::EDIT);
    let listeners = use_context_provider(EditListeners::default);
    let clipboard = use_context_provider(HostClipboard::default);
    let document = use_hook(|| Rc::new(RefCell::new(None::<NodeHandle>)));
    let found = Rc::clone(&document);
    use_context_provider(move || {
        crate::focus::finder(move || found.borrow().clone().map(DocRef::Handle))
    });
    let window = use_window();
    use_hook(|| {
        if let Some(slot) = try_consume_context::<WindowSlot>() {
            slot.fill(Arc::clone(&window));
        }
    });
    let shell = use_hook(consume_context::<Arc<dyn ShellProvider>>);
    let framed = {
        let window = Arc::clone(&window);
        ds::use_window_host_provider(move || Rc::new(WinitWindow::new(window, shell)))
    };
    let factor = window.scale_factor();
    let scale = use_context_provider(|| HostScale(Signal::new(scale_of(factor))));
    let seen = Rc::clone(&document);
    let held = use_hook(|| Rc::new(std::cell::Cell::new(keyboard_types::Modifiers::empty())));
    let book = use_hook(FrameBook::new);
    let found = book.clone();
    let hovering = use_hook(|| Rc::new(RefCell::new(WindowHover::new(book.clone()))));
    let hover = props.setup.frame_links.hover();
    let file_drop = use_context_provider(crate::drop_hit::drop_seam);
    let dragged = use_hook(|| Rc::new(RefCell::new(WindowDrop::default())));
    use_window_event(move |event, event_loop| {
        if let (Some(keeper), Some(handle)) = (&keeper, seen.borrow().as_ref()) {
            keep(&mut keeper.borrow_mut(), &DocRef::Handle(handle.clone()));
        }
        if let crate::frame_hover::FrameHover::Report(_) = hover {
            let crossings = hovering.borrow_mut().crossings(
                event,
                window.scale_factor(),
                seen.borrow().as_ref(),
            );
            report(&hover, crossings);
        }
        if let WindowEvent::ScaleFactorChanged { scale_factor, .. } = event {
            let HostScale(mut current) = scale;
            let next = scale_of(*scale_factor);
            if *current.peek() != next {
                current.set(next);
            }
        }
        if matches!(
            event,
            WindowEvent::SurfaceResized(_) | WindowEvent::Focused(_)
        ) {
            framed.refresh();
        }
        if let Some(next) = modality_after(event) {
            let HostModality(mut current) = modality;
            if *current.peek() != next {
                current.set(next);
            }
        }
        if let WindowEvent::ModifiersChanged(state) = event {
            held.set(modifiers_of(state.state()));
        }
        if let Some(pointer) = captured_of(event, window.scale_factor(), held.get())
            && let Some(sink) = listeners.captured(pointer.phase)
        {
            sink.call(pointer);
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
        let inputs = dragged
            .borrow_mut()
            .inputs(event, event_loop, window.scale_factor());
        for input in inputs {
            let answer = file_drop.feed(input);
            dragged.borrow_mut().answer(event_loop, answer);
        }
        if matches!(event, WindowEvent::RedrawRequested) {
            find_frames(&seen.borrow(), &found);
        }
        if matches!(event, WindowEvent::RedrawRequested)
            && let Some(handle) = seen.borrow().as_ref()
            && let Some(changed) = scheme::follow_root(&mut handle.doc_mut())
        {
            window.set_theme(Some(theme(changed)));
        }
    });
    let frame_nav = use_hook(|| {
        let (nav, inbox) = frame_links(&props.setup.frame_links, book.clone());
        let read = Rc::clone(&document);
        spawn(inbox.serve(move |frame, href| {
            let handle = read.borrow().clone();
            handle.as_ref().and_then(NodeHandle::try_doc).map_or_else(
                || crate::frame_anchor::LinkFacts::bare(href),
                |doc| read_link(&doc, frame, href),
            )
        }));
        nav
    });
    let mut installed = use_signal(|| Installed::Pending);
    let setup = props.setup.clone();
    let root = props.root.clone();
    rsx! {
        // The app waits one frame for its document's providers: a frame in its first render
        // would otherwise be parsed with the parent's `file:` provider (`crate::frames`).
        if installed() == Installed::Done {
            match root {
                Root::Plain(App) => rsx! { App {} },
                Root::Shared(root) => rsx! { Rooted { root: SharedRoot(root) } },
            }
        }
        div {
            style: "display:none",
            onmounted: move |mounted| {
                if let Some(handle) = mounted.data().downcast::<NodeHandle>() {
                    install(handle, &setup, &clipboard, (frame_nav.clone(), book.clone()));
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

/// Find the frames attached under the document's `iframe`s since the last frame (each asked
/// for a redraw as it attached), so their requests reach the app tagged. The document is
/// released before any request is put to the app.
fn find_frames(document: &Option<NodeHandle>, book: &FrameBook) {
    let live = document
        .as_ref()
        .and_then(|handle| handle.try_doc())
        .map(|doc| crate::frame_tree::live_frames(&doc));
    if let Some(live) = live {
        book.bind(live);
    }
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
