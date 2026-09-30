//! One quire document with no window: the pieces `Harness` and `snapshot` share. It gets the
//! shared font context, the app's net policy, the HTML parser (so an `<iframe srcdoc>` is a
//! real sub-document, as in the window), a shell whose clipboard is in memory, sequential styling (deterministic, and
//! no rayon pool per test), the host's input modality, device scale, rect read and focus write as
//! root context, and a waker to sleep on. Every frame's layout is snapped to the device pixel
//! grid (`crate::snap`), so a picture at a fractional scale is what a snapping host shows.

use crate::painter::{Canvas, PaintTime, Painter};
use crate::snapshot::Viewport;
use blitz_dom::{Document as _, DocumentConfig, StyleThreading};
use blitz_html::HtmlProvider;
use blitz_traits::events::UiEvent;
use blitz_traits::net::NetWaker;
use blitz_traits::shell::{ColorScheme, ShellProvider, Viewport as BlitzViewport};
use dioxus::prelude::*;
use dioxus_native_dom::DioxusDocument;
use ds::{Activity, FileDropBoard, HostSignals, InputModality, Scale};
use ds_blitz::FocusFallback;
use ds_blitz::FrameHover;
use ds_blitz::NativeError;
use ds_blitz::clipboard::Memory;
use ds_blitz::font_context;
use ds_blitz::seam::DocRef;
use ds_blitz::seam::DsNet;
use ds_blitz::seam::EditListeners;
use ds_blitz::seam::FrameBook;
use ds_blitz::seam::FrameParser;
use ds_blitz::seam::MemoryShell;
use ds_blitz::seam::Setup;
use ds_blitz::seam::Wakeup;
use ds_blitz::seam::follow_scheme;
use ds_blitz::seam::{FocusKeeper, Kept, focus_finder, keep};
use ds_blitz::seam::{HoverTracker, link_under, live_frames, report_frame_hover};
use ds_blitz::seam::{LinkInbox, frame_links, read_link};
use ds_blitz::seam::{Provided, Wiring};
use ds_blitz::seam::{RestingPointer, Synced, sync_hover};
use peniko::Color;
use std::cell::RefCell;
use std::rc::Rc;
use std::sync::Arc;
use std::task::{Context, Waker};
use std::time::Duration;

/// More rounds than this without the document settling is a render loop in the app.
const MAX_ROUNDS: usize = 64;

/// Whether a document is laid out as it renders: a shell surface is not until it is mapped.
#[derive(Debug, Clone, Copy, PartialEq, Eq, Hash, Default)]
pub(crate) enum Layout {
    /// Styled and laid out every frame.
    #[default]
    Running,
    /// Rendered (components run, tasks are polled) but never styled or laid out: a surface
    /// whose document was built before the compositor mapped it.
    Held,
}

/// A headless document and what drives it.
pub(crate) struct Headless {
    pub(crate) doc: DioxusDocument,
    modality: Signal<InputModality>,
    wakeup: Arc<Wakeup>,
    viewport: Viewport,
    pub(crate) layout: Layout,
    /// The document's shell, whose clipboard is in memory.
    pub(crate) shell: Arc<MemoryShell>,
    /// Links clicked in the document's frames, on their way to the app.
    links: LinkInbox,
    /// The document's frames and their tags.
    book: FrameBook,
    /// Whether the app hears the pointer cross links in frames, and the link it is on.
    hover: (FrameHover, HoverTracker),
    /// The edit surfaces listening for IME events.
    pub(crate) listeners: EditListeners,
    /// Under `FocusFallback::Ancestor`, where the keyboard goes when its element is removed.
    keeper: Keeper,
    /// The last pointer event, replayed when a resolve moves the hover by itself
    /// (`crate::hover_sync`).
    resting: Option<RestingPointer>,
    /// What paints it: vello_cpu unless the harness asked for vello_hybrid.
    pub(crate) painter: Painter,
}

/// Whether the document hands the keyboard on when its element is removed.
#[derive(Debug, Clone)]
enum Keeper {
    /// To the nearest focusable ancestor (`crate::focus_keep`).
    Ancestor(Rc<RefCell<FocusKeeper>>),
    /// Blitz's own: nowhere.
    Off,
}

impl Headless {
    /// Build `app` at `viewport` with the app's `setup` and run its first render (no layout
    /// yet).
    pub(crate) fn new(app: fn() -> Element, viewport: Viewport, setup: &Setup) -> Self {
        let wakeup = Arc::new(Wakeup::default());
        let fetches = Arc::clone(&wakeup);
        let net_waker: Arc<dyn NetWaker> = Arc::new(move |_doc: usize| fetches.note_fetch());
        let shell = Arc::new(MemoryShell::default());
        let book = FrameBook::new();
        let (frame_nav, links) = frame_links(&setup.frame_links, book.clone());
        let frame_net = DsNet::frame(
            setup.net.clone(),
            Some(Arc::clone(&net_waker)),
            book.clone(),
        );
        let config = DocumentConfig {
            viewport: Some(blitz_viewport(viewport)),
            font_ctx: Some(font_context()),
            net_provider: Some(DsNet::top(setup.net.clone(), None, Some(net_waker))),
            html_parser_provider: Some(FrameParser::shared(
                Arc::new(HtmlProvider),
                frame_net,
                frame_nav,
            )),
            shell_provider: Some(Arc::clone(&shell) as Arc<dyn ShellProvider>),
            style_threading: StyleThreading::Sequential,
            ..Default::default()
        };
        let listeners = EditListeners::default();
        let mut vdom = VirtualDom::new(app);
        // The app's own first, so a quire context of the same type (none today) would win.
        setup.contexts.install(&mut vdom);
        let signals = vdom.in_runtime(|| HostSignals {
            modality: Signal::new_in_scope(InputModality::default(), ScopeId::ROOT),
            scale: Signal::new_in_scope(Scale::from_percent(viewport.scale_percent), ScopeId::ROOT),
            activity: Signal::new_in_scope(Activity::Active, ScopeId::ROOT),
        });
        vdom.provide_root_context(signals);
        let keeper = match setup.focus_fallback {
            FocusFallback::Ancestor => {
                Keeper::Ancestor(Rc::new(RefCell::new(FocusKeeper::default())))
            }
            FocusFallback::BlitzDefault => Keeper::Off,
        };
        let mut doc = DioxusDocument::new(vdom, config);
        let found = DocRef::Cell(Rc::clone(&doc.inner));
        let provided = Provided::of(Wiring {
            keeper: match &keeper {
                Keeper::Ancestor(shared) => Some(Rc::clone(shared)),
                Keeper::Off => None,
            },
            find: Some(focus_finder(move || Some(found.clone()))),
            clipboard: Rc::new(Memory(Arc::clone(&shell))),
            listeners: listeners.clone(),
        });
        doc.vdom.provide_root_context(provided.clipboard);
        doc.vdom
            .provide_root_context(FileDropBoard::new(Rc::clone(&provided.host)));
        doc.vdom.provide_root_context(provided.host);
        doc.initial_build();
        Headless {
            doc,
            modality: signals.modality,
            wakeup,
            viewport,
            layout: Layout::Running,
            shell,
            links,
            book,
            hover: (setup.frame_links.hover(), HoverTracker::default()),
            listeners,
            keeper,
            resting: None,
            painter: Painter::Cpu,
        }
    }

    /// Remember where `event` leaves the pointer, if it is a pointer event.
    pub(crate) fn note_pointer(&mut self, event: &UiEvent) {
        if let Some(resting) = RestingPointer::from_event(event) {
            self.resting = Some(resting);
        }
    }

    /// What wakes this document.
    pub(crate) fn wakeup(&self) -> &Arc<Wakeup> {
        &self.wakeup
    }

    /// Record the kind of input that arrived last, so `Ds` stamps `data-modality`.
    pub(crate) fn set_modality(&mut self, next: InputModality) {
        let mut modality = self.modality;
        self.doc.vdom.in_runtime(|| {
            if *modality.peek() != next {
                modality.set(next);
            }
        });
    }

    /// Run every render the document has queued; whether there was any.
    fn flush(&mut self) -> bool {
        let waker = Waker::from(Arc::clone(&self.wakeup));
        (0..MAX_ROUNDS)
            .take_while(|_| self.doc.poll(Some(Context::from_waker(&waker))))
            .count()
            > 0
    }

    /// Bring the document to animation time `at`: render what is queued, follow the root's
    /// scheme, style and lay out, and go round again while a round produced more work (an image
    /// that landed during styling is applied on the next resolve, spike S7).
    pub(crate) fn frame(&mut self, at: Duration) {
        for _ in 0..MAX_ROUNDS {
            let inner = &self.doc.inner;
            self.links
                .drain(&|frame, href| read_link(&inner.borrow(), frame, href));
            let rendered = self.flush();
            // After the renders and tasks have run, so a component's own hand-back (a menu's
            // to its anchor) goes first.
            let kept = self.keep_focus();
            self.find_frames();
            if self.layout == Layout::Held {
                return;
            }
            let restyled = follow_scheme(&mut self.doc.inner.borrow_mut()).is_some();
            let fetched = self.wakeup.fetched();
            let synced = self.resolve(at);
            let landed = self.wakeup.fetched() != fetched;
            if !(rendered
                || restyled
                || landed
                || kept == Kept::Moved
                || synced == Synced::Replayed)
            {
                return;
            }
        }
    }

    /// Style and lay out at `at`, then dispatch the hover change the layout made under a
    /// resting pointer, which Blitz's resolve records silently (`crate::hover_sync`). At most
    /// one replay per round: the replayed move leaves Blitz's hover where the next resolve
    /// finds it, so that one's re-hit-test changes nothing.
    fn resolve(&mut self, at: Duration) -> Synced {
        let mut inner = self.doc.inner.borrow_mut();
        let before = inner.get_hover_node_id();
        inner.resolve(at.as_secs_f64());
        ds_blitz::snap_to_device(&mut inner);
        drop(inner);
        sync_hover(&mut self.doc, before, self.resting.as_ref())
    }

    /// Hand the keyboard to a focusable ancestor of a focused element the renders removed.
    fn keep_focus(&mut self) -> Kept {
        match &mut self.keeper {
            Keeper::Ancestor(keeper) => keep(
                &mut keeper.borrow_mut(),
                &DocRef::Cell(Rc::clone(&self.doc.inner)),
            ),
            Keeper::Off => Kept::Still,
        }
    }

    /// Find the frames the renders attached under their `iframe`s, so their requests reach the
    /// app tagged. The document is free by then: the app's `decide` may touch it.
    fn find_frames(&mut self) {
        let live = live_frames(&self.doc.inner.borrow());
        self.book.bind(live);
    }

    /// The pointer moved to `at`: tell the app if it came onto or left a link in a frame.
    pub(crate) fn hover_at(&mut self, at: ds::Point) {
        let (hover, tracker) = &mut self.hover;
        if let FrameHover::Ignore = hover {
            return;
        }
        let under = link_under(&self.doc.inner.borrow(), at);
        let crossings = tracker.step(under, at, &self.book);
        report_frame_hover(hover, crossings);
    }

    /// Paint the document as it was last resolved, over `backdrop`.
    pub(crate) fn paint(&mut self, backdrop: Backdrop) -> Result<image::RgbaImage, NativeError> {
        let canvas = self.canvas(backdrop);
        let mut inner = self.doc.inner.borrow_mut();
        let pixels = self.painter.picture(&mut inner, canvas)?;
        let (width, height) = (canvas.width, canvas.height);
        let length = pixels.len();
        image::RgbaImage::from_raw(width, height, pixels).ok_or_else(|| {
            NativeError::Renderer(format!(
                "the renderer returned {length} bytes for {width}x{height}"
            ))
        })
    }

    /// Paint the document as it was last resolved, over `backdrop`, to the end, and time it.
    pub(crate) fn paint_timed(&mut self, backdrop: Backdrop) -> Result<PaintTime, NativeError> {
        let canvas = self.canvas(backdrop);
        let mut inner = self.doc.inner.borrow_mut();
        self.painter.time(&mut inner, canvas)
    }

    /// The frame to paint over `backdrop`, at the viewport's device size.
    fn canvas(&self, backdrop: Backdrop) -> Canvas {
        let (width, height) = physical(self.viewport);
        let ground = match (backdrop, self.doc.inner.borrow().viewport().color_scheme) {
            (Backdrop::Clear, _) => Color::TRANSPARENT,
            (Backdrop::Scheme, ColorScheme::Dark) => Color::BLACK,
            (Backdrop::Scheme, ColorScheme::Light) => Color::WHITE,
        };
        Canvas {
            width,
            height,
            scale: scale(self.viewport),
            ground,
        }
    }
}

/// What a headless picture is painted over.
#[derive(Debug, Clone, Copy, PartialEq, Eq, Hash, Default)]
pub enum Backdrop {
    /// The scheme's own ground, white or black: what a window shows behind a document.
    #[default]
    Scheme,
    /// Nothing: fully transparent, as a shell surface is where its document paints nothing, so
    /// a picture's alpha is the document's own coverage.
    Clear,
}

/// Device pixels per logical pixel.
fn scale(viewport: Viewport) -> f64 {
    f64::from(viewport.scale_percent) / 100.0
}

/// The viewport in device pixels.
pub(crate) fn physical(viewport: Viewport) -> (u32, u32) {
    let device = |logical: u32| (f64::from(logical) * scale(viewport)).round() as u32;
    (device(viewport.width), device(viewport.height))
}

/// Blitz's viewport for ours, light until the root says otherwise.
fn blitz_viewport(viewport: Viewport) -> BlitzViewport {
    let (width, height) = physical(viewport);
    BlitzViewport::new(width, height, scale(viewport) as f32, ColorScheme::Light)
}

#[cfg(test)]
mod tests {
    use super::physical;
    use crate::snapshot::Viewport;

    /// A logical size and scale, and the device pixels it renders at.
    type Case = (Viewport, (u32, u32));

    const fn at(width: u32, height: u32, scale_percent: u16) -> Viewport {
        Viewport {
            width,
            height,
            scale_percent,
        }
    }

    const CASES: &[Case] = &[
        (at(400, 300, 100), (400, 300)),
        (at(400, 300, 200), (800, 600)),
        (at(401, 301, 150), (602, 452)),
    ];

    #[test]
    fn device_pixels_follow_the_scale() {
        for &(viewport, expected) in CASES {
            assert_eq!(physical(viewport), expected, "{viewport:?}");
        }
    }
}
