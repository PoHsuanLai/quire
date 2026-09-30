//! Driving a quire app in a test the way a user would: pointer, keys, and time, against a real
//! Blitz document (no window). Timers run on the harness's clock, so a test advances 450 ms and
//! sees the hover card open, not before.
//!
//! Time. quire's timers are `futures-timer` sleeps and its hover intent reads `Instant::now()`,
//! both on the wall clock, so a harness cannot fake time: [`Driver::advance`] really lets it
//! pass. It does not spin: it sleeps on a condition variable that the document's waker signals,
//! so it wakes exactly when a timer fires (or a resource lands), runs the renders that queued,
//! resolves the document at the matching animation time, and sleeps again until the deadline.
//! Animation time (`resolve(t)`) is the harness's own clock: the sum of every `advance`, so a
//! frame's CSS time never depends on how slow the machine running the test is.
//!
//! That is the default, [`Clock::Wall`]. A harness built with
//! `HarnessConfig::with_clock(Clock::Virtual)` runs quire's timers on the same clock as its CSS
//! instead, and `advance` takes no wall-clock time at all (`crate::harness_clock`).

use crate::driver::{DocQuery, Driver, Query, first};
use crate::error::HarnessError;
use crate::frame_view::FrameView;
use crate::harness_clock::{Clock, HarnessClock};
use crate::harness_config::HarnessConfig;
use crate::harness_input::HeldButtons;
pub use crate::harness_settle::{
    QUIET, SETTLE_BOUND, VIRTUAL_DRAIN_BOUND, assert_settles_to_zero_frames, settle_until,
};
use crate::headless::{Backdrop, Headless, Layout};
use crate::input::Input;
use crate::snapshot::Viewport;
use blitz_dom::{BaseDocument, Document as _};
use blitz_traits::events::UiEvent;
use dioxus::prelude::*;
use ds::file_drop::drag::DropAcceptance;
use ds::prelude::*;
use ds_core::time::clock::VirtualClock;
use std::time::{Duration, Instant};

/// A headless document under test.
pub struct Harness {
    viewport: Viewport,
    pub(crate) doc: Headless,
    /// Animation time: the sum of every `advance`.
    clock: Duration,
    /// The mouse buttons down now.
    pub(crate) held: HeldButtons,
    /// What the window would have told the platform about the last file drag step.
    drop_answer: DropAcceptance,
    /// Keeps a Tokio runtime entered on this thread for as long as the harness lives, so a
    /// component under test that calls `ds_settings::use_environment` does not panic; see
    /// `ds_blitz::enter_runtime`. Never read, only held: it does its work by staying alive and being
    /// dropped with the harness.
    _runtime: ds_blitz::RuntimeGuard,
    /// The clock timers run on. Last, so a virtual clock stays installed while the document
    /// (and every task sleeping on it) drops.
    time: HarnessClock,
}

impl std::fmt::Debug for Harness {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        f.debug_struct("Harness")
            .field("viewport", &self.viewport)
            .field("clock", &self.clock)
            .field("time", &self.time.choice())
            .finish_non_exhaustive()
    }
}

impl Harness {
    /// Build `app` as `config` says (a bare [`Viewport`] is a config with no contexts) and
    /// render its first frame. A [`Backend::Hybrid`](crate::Backend::Hybrid) harness with no
    /// GPU device to open returns the error from every picture; [`Harness::try_new`] returns it
    /// at once.
    pub fn new(app: fn() -> Element, config: impl Into<HarnessConfig>) -> Self {
        let config = config.into();
        // Entered before `Headless::new`, whose `initial_build` runs `app`'s first render and
        // so is where a `use_future` calling `tokio::spawn` (e.g. `ds_settings::use_environment`)
        // would run.
        let runtime = ds_blitz::enter_runtime();
        // Installed before the first render, so a hook that notes the time as it mounts reads
        // the harness's clock.
        let time = HarnessClock::start(config.clock());
        let viewport = config.viewport();
        let mut doc = Headless::new(app, viewport, config.setup());
        doc.layout = config.layout();
        doc.painter = crate::harness_backend::painter(&config);
        let mut harness = Harness {
            viewport,
            doc,
            clock: Duration::ZERO,
            _runtime: runtime,
            held: HeldButtons::default(),
            drop_answer: DropAcceptance::Refuse,
            time,
        };
        harness.settle();
        harness
    }

    /// Lay the document out from now on, as the compositor maps its surface, and resolve it.
    pub fn map(&mut self) {
        self.doc.layout = Layout::Running;
        self.settle();
    }

    /// What the window would have told the platform about the last [`Input::FileDrag`] step: a
    /// copy over a drop target, a refusal elsewhere (a refusal before any step).
    pub fn drop_answer(&self) -> DropAcceptance {
        self.drop_answer
    }

    /// The mouse buttons down now.
    pub fn held_buttons(&self) -> HeldButtons {
        self.held
    }

    /// Let `time` pass: fire due timers and render. On the wall clock it takes `time` of
    /// wall-clock time; see the module documentation for why. On the virtual clock it steps to
    /// each timer's due instant in order, rendering and resolving at each, and returns at once.
    pub(crate) fn pass(&mut self, time: Duration) {
        match self.time.virtual_clock() {
            Some(clock) => crate::harness_clock::advance(self, &clock, time),
            None => self.advance_wall(time),
        }
    }

    /// Which clock this harness's timers run on.
    pub fn clock(&self) -> Clock {
        self.time.choice()
    }

    /// Now on this harness's clock: the wall clock's now, or the virtual clock's (which
    /// `ds::base::time::clock::now` also reads on this thread). Compare it with other instants from the same
    /// harness, e.g. the one [`settle_until`] returns.
    pub fn now(&self) -> Instant {
        self.time.now()
    }

    /// The virtual clock this harness's timers run on, if [`Clock::Virtual`]: for a settle check
    /// that needs to see every sleep still pending, not just poll the harness for silence.
    pub(crate) fn virtual_clock(&self) -> Option<VirtualClock> {
        self.time.virtual_clock()
    }

    /// Resolve the document at animation time `at` after running what is queued: one step of a
    /// virtual advance.
    pub(crate) fn resolve_at(&mut self, at: Duration) {
        self.clock = at;
        self.settle();
    }

    fn advance_wall(&mut self, time: Duration) {
        let started = Instant::now();
        let deadline = started + time;
        loop {
            let seen = self.doc.wakeup().generation();
            let elapsed = started.elapsed().min(time);
            self.doc.frame(self.clock + elapsed);
            let now = Instant::now();
            if now >= deadline {
                break;
            }
            self.doc.wakeup().wait_past(seen, deadline - now);
        }
        self.clock += time;
        self.settle();
    }

    /// Run `f` inside the app's runtime, as its handlers run: a test reads an app-facing handle
    /// (`ds::edit::handle::EditHandle::caret_rect`) the way the app would.
    pub fn within<T>(&mut self, f: impl FnOnce() -> T) -> T {
        self.doc.doc.vdom.in_runtime(f)
    }

    /// The document as HTML, for assertions.
    pub fn html(&self) -> String {
        self.with_doc(|doc| doc.root_element().outer_html())
    }

    /// Whether the document would paint another frame on its own: a CSS animation or
    /// transition running, a canvas, a scroll animation. `false` is the idle-frame rule's
    /// "a surface at rest paints 0 frames" as the host sees it (it asks for no redraw).
    pub fn is_animating(&self) -> bool {
        self.with_doc(BaseDocument::is_animating)
    }

    /// How many times the document's waker has fired so far: a timer that fired, a resource that
    /// landed. Unchanged across an `advance` means nothing woke the document in that time, so no
    /// Rust timer asked for a frame (the idle-frame rule's other half: `is_animating` sees only
    /// CSS).
    pub fn wakes(&self) -> u64 {
        self.doc.wakeup().generation()
    }

    /// The centre of the first element matching `selector`: where a test clicks it.
    pub fn centre(&self, selector: &str) -> Option<Point> {
        self.rect(selector).map(|rect| Point {
            x: Px(rect.origin.x.0 + rect.size.width.0 / 2.0),
            y: Px(rect.origin.y.0 + rect.size.height.0 / 2.0),
        })
    }

    /// Paint the document as it is now over `backdrop`: [`Backdrop::Clear`] shows what a shell
    /// surface would, where every pixel the document leaves unpainted has alpha 0.
    pub fn render_over(&mut self, backdrop: Backdrop) -> Result<image::RgbaImage, HarnessError> {
        self.doc.paint(backdrop)
    }

    /// Resolve at `at` and paint: a snapshot at one motion moment.
    pub(crate) fn render_at(&mut self, at: Duration) -> Result<image::RgbaImage, HarnessError> {
        self.clock = at;
        self.settle();
        self.doc.paint(Backdrop::Scheme)
    }

    /// Hand `event` to the document and bring it up to date.
    pub(crate) fn deliver(&mut self, event: UiEvent) {
        // An edit surface holding the pointer hears a move or the release first, wherever it is
        // (`crate::edit_ime`), as the window's hook delivers it before the document.
        self.route_captured(&event);
        self.doc.note_pointer(&event);
        self.doc.doc.handle_ui_event(event);
        self.settle();
    }

    fn settle(&mut self) {
        self.doc.frame(self.clock);
    }

    /// Bring the document up to date after input delivered outside `send`.
    pub(crate) fn settle_now(&mut self) {
        self.settle();
    }

    /// What was last copied (Ctrl+C in a field, `ds_blitz::clipboard::write_text`): the
    /// harness's clipboard is in memory, never the desktop's.
    pub fn clipboard_text(&self) -> Option<String> {
        self.doc.shell.text()
    }

    /// Put `text` on the harness's clipboard, as another app's copy would.
    pub fn set_clipboard_text(&mut self, text: &str) {
        self.doc.shell.put(text.to_owned());
    }

    /// The text selected in the first text field matching `selector`, if it has a selection.
    pub fn selected_text(&self, selector: &str) -> Option<String> {
        self.with_doc(|doc| {
            let input = doc
                .get_node(first(doc, selector)?)?
                .element_data()?
                .text_input_data()?;
            input.editor.selected_text().map(str::to_owned)
        })
    }

    /// The sub-document of the first `iframe` matching `selector`, once it has one: what a
    /// frame shows, read without the app's document seeing into it.
    pub fn frame(&self, selector: &str) -> Option<FrameView<'_>> {
        FrameView::find(self, selector)
    }
}

impl DocQuery for Harness {
    fn with_doc<T>(&self, read: impl FnOnce(&BaseDocument) -> T) -> T {
        read(&self.doc.doc.inner())
    }
}

impl Driver for Harness {
    fn send(&mut self, input: Input) {
        match input {
            Input::Pointer(pointer) => self.pointer(pointer),
            Input::Key(key) => self.key(key),
            Input::Wheel { at, dx, dy } => self.wheel(at, dx, dy),
            Input::FileDrag(step) => self.drop_answer = self.file_drag(step),
            Input::Ime(ime) => self.ime(ime),
            Input::Paste { html, text } => self.paste(&html, &text),
        }
    }

    fn advance(&mut self, by: Duration) {
        self.pass(by);
    }

    fn render(&mut self) -> Result<image::RgbaImage, HarnessError> {
        self.doc.paint(Backdrop::Scheme)
    }
}
