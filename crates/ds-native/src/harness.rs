//! Driving a quire app in a test the way a user would: pointer, keys, and time, against a real
//! Blitz document (no window). Timers run on the harness's clock, so a test advances 450 ms and
//! sees the hover card open, not before.
//!
//! Time. quire's timers are `futures-timer` sleeps and its hover intent reads `Instant::now()`,
//! both on the wall clock, so a harness cannot fake time: [`Harness::advance`] really lets it
//! pass. It does not spin: it sleeps on a condition variable that the document's waker signals,
//! so it wakes exactly when a timer fires (or a resource lands), runs the renders that queued,
//! resolves the document at the matching animation time, and sleeps again until the deadline.
//! Animation time (`resolve(t)`) is the harness's own clock: the sum of every `advance`, so a
//! frame's CSS time never depends on how slow the machine running the test is.

use crate::contexts::RootContexts;
use crate::error::NativeError;
use crate::frame_view::FrameView;
use crate::harness_config::HarnessConfig;
use crate::harness_input::{HeldButtons, blitz_button, keyboard, modifier, pointer};
use crate::headless::{Backdrop, Headless, Layout};
use crate::snapshot::Viewport;
use blitz_dom::{BaseDocument, Document as _, LocalName, NodeId};
use blitz_traits::events::{BlitzKeyEvent, KeyState, MouseEventButton, UiEvent};
use dioxus::prelude::*;
use ds::{InputModality, Key, Point, PointerButton, Px, Rect, Size};
use keyboard_types::{Location, Modifiers};
use std::time::{Duration, Instant};

/// A headless document under test.
pub struct Harness {
    viewport: Viewport,
    pub(crate) doc: Headless,
    /// Animation time: the sum of every `advance`.
    clock: Duration,
    /// The mouse buttons down now.
    held: HeldButtons,
    /// Keeps a Tokio runtime entered on this thread for as long as the harness lives, so a
    /// component under test that calls `ds_settings::use_environment` does not panic; see
    /// `crate::runtime`. Never read, only held: it does its work by staying alive and being
    /// dropped with the harness.
    _runtime: tokio::runtime::EnterGuard<'static>,
}

impl std::fmt::Debug for Harness {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        f.debug_struct("Harness")
            .field("viewport", &self.viewport)
            .field("clock", &self.clock)
            .finish_non_exhaustive()
    }
}

impl Harness {
    /// Build `app` at `viewport` and render its first frame.
    pub fn new(app: fn() -> Element, viewport: Viewport) -> Self {
        Harness::with_config(app, HarnessConfig::new(viewport))
    }

    /// Build `app` with `contexts` provided at its root, as `AppConfig::with_contexts` gives a
    /// window, and render its first frame.
    pub fn with_contexts(app: fn() -> Element, viewport: Viewport, contexts: RootContexts) -> Self {
        Harness::with_config(app, HarnessConfig::new(viewport).with_contexts(contexts))
    }

    /// Build `app` as `config` says and render its first frame.
    pub fn with_config(app: fn() -> Element, config: HarnessConfig) -> Self {
        Harness::start(app, config, Layout::Running)
    }

    /// Build `app` at `viewport` as a shell surface is built before it is mapped: its renders
    /// run and its tasks are polled, but nothing is styled or laid out until [`Harness::map`]
    /// (every rect reads 0 x 0 until then, sill FINDINGS Q60).
    pub fn unmapped(app: fn() -> Element, viewport: Viewport) -> Self {
        Harness::start(app, HarnessConfig::new(viewport), Layout::Held)
    }

    fn start(app: fn() -> Element, config: HarnessConfig, layout: Layout) -> Self {
        // Entered before `Headless::new`, whose `initial_build` runs `app`'s first render and
        // so is where a `use_future` calling `tokio::spawn` (e.g. `ds_settings::use_environment`)
        // would run.
        let runtime = crate::runtime::enter();
        let viewport = config.viewport();
        let mut doc = Headless::new(app, viewport, config.setup());
        doc.layout = layout;
        let mut harness = Harness {
            viewport,
            doc,
            clock: Duration::ZERO,
            _runtime: runtime,
            held: HeldButtons::default(),
        };
        harness.settle();
        harness
    }

    /// Lay the document out from now on, as the compositor maps its surface, and resolve it.
    pub fn map(&mut self) {
        self.doc.layout = Layout::Running;
        self.settle();
    }

    /// Move the pointer to `at`, with whatever buttons are down (a drag while one is). Coming
    /// onto or leaving a link in a frame is reported as the window reports it
    /// (`FrameLinks::with_hover`).
    pub fn pointer_move(&mut self, at: Point) {
        self.pointer_move_with(at, Modifiers::empty());
    }

    /// Move the pointer to `at` with `mods` held.
    pub fn pointer_move_with(&mut self, at: Point, mods: Modifiers) {
        self.send(UiEvent::PointerMove(pointer(
            at,
            MouseEventButton::Main,
            self.held.blitz(),
            mods,
        )));
        self.doc.hover_at(at);
    }

    /// Press the primary button at `at`. A pointer press makes the modality `pointer`.
    pub fn pointer_down(&mut self, at: Point) {
        self.button_down(at, PointerButton::Primary);
    }

    /// Release the primary button at `at`.
    pub fn pointer_up(&mut self, at: Point) {
        self.button_up(at, PointerButton::Primary);
    }

    /// Press `button` at `at`. A pointer press makes the modality `pointer`.
    pub fn button_down(&mut self, at: Point, button: PointerButton) {
        self.button_down_with(at, button, Modifiers::empty());
    }

    /// Press `button` at `at` with `mods` held (Shift+click is `Modifiers::SHIFT`); it stays
    /// down until [`Harness::button_up_with`].
    pub fn button_down_with(&mut self, at: Point, button: PointerButton, mods: Modifiers) {
        self.doc.set_modality(InputModality::Pointer);
        self.held = self.held.with(button);
        let (which, _) = blitz_button(button);
        self.send(UiEvent::PointerDown(pointer(
            at,
            which,
            self.held.blitz(),
            mods,
        )));
    }

    /// Release `button` at `at`.
    pub fn button_up(&mut self, at: Point, button: PointerButton) {
        self.button_up_with(at, button, Modifiers::empty());
    }

    /// Release `button` at `at` with `mods` held.
    pub fn button_up_with(&mut self, at: Point, button: PointerButton, mods: Modifiers) {
        self.held = self.held.without(button);
        let (which, _) = blitz_button(button);
        self.send(UiEvent::PointerUp(pointer(
            at,
            which,
            self.held.blitz(),
            mods,
        )));
    }

    /// The mouse buttons down now.
    pub fn held_buttons(&self) -> HeldButtons {
        self.held
    }

    /// Press the primary button at `from`, move to `to` in `steps` even steps with it held, and
    /// release it there: a drag selection.
    pub fn drag(&mut self, from: Point, to: Point, steps: u16) {
        self.pointer_move(from);
        self.pointer_down(from);
        let steps = steps.max(1);
        for step in 1..=steps {
            let part = f32::from(step) / f32::from(steps);
            self.pointer_move(Point {
                x: Px(from.x.0 + (to.x.0 - from.x.0) * part),
                y: Px(from.y.0 + (to.y.0 - from.y.0) * part),
            });
        }
        self.pointer_up(to);
    }

    /// Press and release the primary button at `at` with `mods` held, after moving there.
    pub fn click_with(&mut self, at: Point, mods: Modifiers) {
        self.pointer_move_with(at, mods);
        self.button_down_with(at, PointerButton::Primary, mods);
        self.button_up_with(at, PointerButton::Primary, mods);
    }

    /// Press and release `button` at `at`, after moving there: a right-click is
    /// `press(at, PointerButton::Secondary)`, which Blitz delivers as `contextmenu`.
    pub fn press(&mut self, at: Point, button: PointerButton) {
        self.pointer_move(at);
        self.button_down(at, button);
        self.button_up(at, button);
    }

    /// Press and release the primary button at `at`, after moving there: the order a host
    /// synthesises, so hover intent arms before the press.
    pub fn click(&mut self, at: Point) {
        self.pointer_move(at);
        self.pointer_down(at);
        self.pointer_up(at);
    }

    /// Press and release `key` with the focus where it is. A key makes the modality `keyboard`.
    pub fn key(&mut self, key: Key) {
        self.chord(&[], key);
    }

    /// Press and release `key` while `held` modifiers (`Key::Ctrl`, `Shift`, `Alt`, `Super`) are
    /// down: `chord(&[Key::Ctrl], Key::Char('k'))` is Ctrl+K. A key that is not a modifier in
    /// `held` adds nothing.
    pub fn chord(&mut self, held: &[Key], key: Key) {
        self.doc.set_modality(InputModality::Keyboard);
        let modifiers = held
            .iter()
            .copied()
            .map(modifier)
            .fold(Modifiers::empty(), |all, one| all | one);
        let (key, code) = keyboard(key);
        for state in [KeyState::Pressed, KeyState::Released] {
            let event = BlitzKeyEvent {
                key: key.clone(),
                code,
                modifiers,
                location: Location::Standard,
                is_auto_repeating: false,
                is_composing: false,
                state,
                text: None,
            };
            self.send(match state {
                KeyState::Pressed => UiEvent::KeyDown(event),
                KeyState::Released => UiEvent::KeyUp(event),
            });
        }
    }

    /// Let `time` pass: fire due timers and render. It takes `time` of wall-clock time; see the
    /// module documentation for why.
    pub fn advance(&mut self, time: Duration) {
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
    /// (`ds::EditHandle::caret_rect`) the way the app would.
    pub fn within<T>(&mut self, f: impl FnOnce() -> T) -> T {
        self.doc.doc.vdom.in_runtime(f)
    }

    /// The document as HTML, for assertions.
    pub fn html(&self) -> String {
        self.with_doc(|doc| doc.root_element().outer_html())
    }

    /// The border-box rect of the first element matching `selector`, if any.
    pub fn rect(&self, selector: &str) -> Option<ds::Rect> {
        self.with_doc(|doc| {
            let found = doc.get_client_bounding_rect(first(doc, selector)?)?;
            Some(Rect {
                origin: Point {
                    x: Px(found.x as f32),
                    y: Px(found.y as f32),
                },
                size: Size {
                    width: Px(found.width as f32),
                    height: Px(found.height as f32),
                },
            })
        })
    }

    /// The text inside the first element matching `selector`, if any.
    pub fn text_of(&self, selector: &str) -> Option<String> {
        self.with_doc(|doc| Some(doc.get_node(first(doc, selector)?)?.text_content()))
    }

    /// Attribute `name` of the first element matching `selector`, if both exist.
    pub fn attr(&self, selector: &str, name: &str) -> Option<String> {
        self.with_doc(|doc| {
            let node = doc.get_node(first(doc, selector)?)?;
            node.attr(LocalName::from(name)).map(str::to_owned)
        })
    }

    /// Whether the first element matching `selector` carries `class` as a whole class token.
    pub fn has_class(&self, selector: &str, class: &str) -> bool {
        self.attr(selector, "class")
            .is_some_and(|classes| classes.split_ascii_whitespace().any(|token| token == class))
    }

    /// How many elements match `selector`.
    pub fn count(&self, selector: &str) -> usize {
        self.with_doc(|doc| {
            doc.query_selector_all(selector)
                .map_or(0, |found| found.len())
        })
    }

    /// Whether the first element matching `selector` has the keyboard focus.
    pub fn is_focused(&self, selector: &str) -> bool {
        self.with_doc(|doc| {
            first(doc, selector).is_some_and(|node| doc.get_focussed_node_id() == Some(node))
        })
    }

    /// The centre of the first element matching `selector`: where a test clicks it.
    pub fn centre(&self, selector: &str) -> Option<Point> {
        self.rect(selector).map(|rect| Point {
            x: Px(rect.origin.x.0 + rect.size.width.0 / 2.0),
            y: Px(rect.origin.y.0 + rect.size.height.0 / 2.0),
        })
    }

    /// Paint the document as it is now, at the harness's animation time, over the scheme's
    /// ground.
    pub fn render(&mut self) -> Result<image::RgbaImage, NativeError> {
        self.doc.paint(Backdrop::Scheme)
    }

    /// Paint the document as it is now over `backdrop`: [`Backdrop::Clear`] shows what a shell
    /// surface would, where every pixel the document leaves unpainted has alpha 0.
    pub fn render_over(&mut self, backdrop: Backdrop) -> Result<image::RgbaImage, NativeError> {
        self.doc.paint(backdrop)
    }

    /// Resolve at `at` and paint: a snapshot at one motion moment.
    pub(crate) fn render_at(&mut self, at: Duration) -> Result<image::RgbaImage, NativeError> {
        self.clock = at;
        self.settle();
        self.doc.paint(Backdrop::Scheme)
    }

    /// Hand `event` to the document and bring it up to date.
    fn send(&mut self, event: UiEvent) {
        // An edit surface holding the pointer hears a move or the release first, wherever it is
        // (`crate::edit_ime`), as the window's hook delivers it before the document.
        self.route_captured(&event);
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

    /// What was last copied (Ctrl+C in a field, `ds_native::clipboard::write_text`): the
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

    pub(crate) fn with_doc<T>(&self, read: impl FnOnce(&BaseDocument) -> T) -> T {
        read(&self.doc.doc.inner())
    }
}

/// The first element matching `selector`; an unparseable selector matches nothing.
pub(crate) fn first(doc: &BaseDocument, selector: &str) -> Option<NodeId> {
    doc.query_selector(selector).ok().flatten()
}
