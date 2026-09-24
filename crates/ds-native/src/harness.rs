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

use crate::error::NativeError;
use crate::headless::{Backdrop, Headless};
use crate::snapshot::Viewport;
use blitz_dom::{BaseDocument, Document as _, LocalName, NodeId};
use blitz_traits::events::{
    BlitzKeyEvent, BlitzPointerEvent, BlitzPointerId, KeyState, MouseEventButton,
    MouseEventButtons, PointerCoords, UiEvent,
};
use dioxus::prelude::*;
use ds::{InputModality, Key, Point, PointerButton, Px, Rect, Size};
use keyboard_types::{Code, Key as DomKey, Location, Modifiers};
use std::time::{Duration, Instant};

/// A headless document under test.
pub struct Harness {
    viewport: Viewport,
    doc: Headless,
    /// Animation time: the sum of every `advance`.
    clock: Duration,
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
        // Entered before `Headless::new`, whose `initial_build` runs `app`'s first render and
        // so is where a `use_future` calling `tokio::spawn` (e.g. `ds_settings::use_environment`)
        // would run.
        let runtime = crate::runtime::enter();
        let mut harness = Harness {
            viewport,
            doc: Headless::new(app, viewport),
            clock: Duration::ZERO,
            _runtime: runtime,
        };
        harness.frame();
        harness
    }

    /// Move the pointer to `at`.
    pub fn pointer_move(&mut self, at: Point) {
        self.send(UiEvent::PointerMove(pointer(
            at,
            MouseEventButton::Main,
            MouseEventButtons::None,
        )));
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
        self.doc.set_modality(InputModality::Pointer);
        let (which, held) = blitz_button(button);
        self.send(UiEvent::PointerDown(pointer(at, which, held)));
    }

    /// Release `button` at `at`.
    pub fn button_up(&mut self, at: Point, button: PointerButton) {
        let (which, _) = blitz_button(button);
        self.send(UiEvent::PointerUp(pointer(
            at,
            which,
            MouseEventButtons::None,
        )));
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
        self.doc.set_modality(InputModality::Keyboard);
        let (key, code) = keyboard(key);
        for state in [KeyState::Pressed, KeyState::Released] {
            let event = BlitzKeyEvent {
                key: key.clone(),
                code,
                modifiers: Modifiers::empty(),
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
        self.frame();
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
        self.frame();
        self.doc.paint(Backdrop::Scheme)
    }

    /// Hand `event` to the document and bring it up to date.
    fn send(&mut self, event: UiEvent) {
        self.doc.doc.handle_ui_event(event);
        self.frame();
    }

    fn frame(&mut self) {
        self.doc.frame(self.clock);
    }

    fn with_doc<T>(&self, read: impl FnOnce(&BaseDocument) -> T) -> T {
        read(&self.doc.doc.inner())
    }
}

/// The first element matching `selector`; an unparseable selector matches nothing.
fn first(doc: &BaseDocument, selector: &str) -> Option<NodeId> {
    doc.query_selector(selector).ok().flatten()
}

/// The Blitz button, and the held-buttons set while it is down, for a quire pointer button.
fn blitz_button(button: PointerButton) -> (MouseEventButton, MouseEventButtons) {
    match button {
        PointerButton::Primary => (MouseEventButton::Main, MouseEventButtons::Primary),
        PointerButton::Secondary => (MouseEventButton::Secondary, MouseEventButtons::Secondary),
        PointerButton::Middle => (MouseEventButton::Auxiliary, MouseEventButtons::Auxiliary),
    }
}

/// A mouse pointer event at `at`, for `button`, with `buttons` held.
fn pointer(at: Point, button: MouseEventButton, buttons: MouseEventButtons) -> BlitzPointerEvent {
    let (x, y) = (at.x.0, at.y.0);
    BlitzPointerEvent {
        id: BlitzPointerId::Mouse,
        is_primary: true,
        coords: PointerCoords {
            page_x: x,
            page_y: y,
            screen_x: x,
            screen_y: y,
            client_x: x,
            client_y: y,
        },
        button,
        buttons,
        mods: Modifiers::empty(),
        details: Default::default(),
        element: Default::default(),
        active_pointers: Default::default(),
    }
}

/// The DOM key and physical code for a quire key.
fn keyboard(key: Key) -> (DomKey, Code) {
    match key {
        Key::Ctrl => (DomKey::Control, Code::ControlLeft),
        Key::Shift => (DomKey::Shift, Code::ShiftLeft),
        Key::Alt => (DomKey::Alt, Code::AltLeft),
        Key::Super => (DomKey::Meta, Code::MetaLeft),
        Key::Char(c) => (DomKey::Character(c.to_string()), letter(c)),
        Key::Space => (DomKey::Character(" ".into()), Code::Space),
        Key::Enter => (DomKey::Enter, Code::Enter),
        Key::Escape => (DomKey::Escape, Code::Escape),
        Key::Tab => (DomKey::Tab, Code::Tab),
        Key::Backspace => (DomKey::Backspace, Code::Backspace),
        Key::Up => (DomKey::ArrowUp, Code::ArrowUp),
        Key::Down => (DomKey::ArrowDown, Code::ArrowDown),
        Key::Left => (DomKey::ArrowLeft, Code::ArrowLeft),
        Key::Right => (DomKey::ArrowRight, Code::ArrowRight),
    }
}

/// The physical key a US layout types `c` with, where it is a letter or a digit.
fn letter(c: char) -> Code {
    format!("Key{}", c.to_ascii_uppercase())
        .parse()
        .or_else(|_| format!("Digit{c}").parse())
        .unwrap_or(Code::Unidentified)
}

#[cfg(test)]
mod tests {
    use super::{keyboard, letter};
    use ds::Key;
    use keyboard_types::Code;

    const LETTERS: &[(char, Code)] = &[
        ('a', Code::KeyA),
        ('Z', Code::KeyZ),
        ('7', Code::Digit7),
        ('/', Code::Unidentified),
    ];

    #[test]
    fn letters_map_to_their_physical_key() {
        for &(c, code) in LETTERS {
            assert_eq!(letter(c), code, "{c:?}");
        }
    }

    #[test]
    fn escape_is_the_named_key() {
        assert_eq!(
            keyboard(Key::Escape),
            (keyboard_types::Key::Escape, Code::Escape)
        );
    }
}
