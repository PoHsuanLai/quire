//! Driving an edit surface in a test: the IME a person would use, a paste with HTML, and the
//! host's hit test and IME requests read back. The IME events go through the same routing the
//! window uses (`crate::edit_ime`): to the surface that has the keyboard.

use crate::driver::{DocQuery, first};
use crate::harness::Harness;
use crate::harness_input::paste_keys;
use crate::input::{ImeInput, PasteChord, RawKeyInput, RawKeyPhase};
use blitz_traits::events::{MouseEventButton, UiEvent};
use ds::host::captured::{CapturedPointer, PointerPhase};
use ds::host::ime::{ImeEvent, ImeSwitch};
use ds::host::position::TextPosition;
use ds::prelude::*;
use ds_blitz::seam::edit_hit as hit;

impl Harness {
    /// Deliver one input-method step, through the routing the window uses: to the surface that
    /// has the keyboard.
    pub(crate) fn ime(&mut self, input: ImeInput) {
        match input {
            ImeInput::Start => self.ime_event(ImeEvent::Enabled),
            ImeInput::Update { text, cursor } => self.ime_event(ImeEvent::Preedit {
                text,
                cursor: Some((cursor, cursor)),
            }),
            ImeInput::Commit(text) => {
                self.ime_event(ImeEvent::Preedit {
                    text: String::new(),
                    cursor: None,
                });
                self.ime_event(ImeEvent::Commit(text));
            }
            ImeInput::End => self.ime_event(ImeEvent::Disabled),
        }
    }

    /// Put `html` and its plain `text` on the clipboard, as a browser's copy would, and press
    /// `chord` where the focus is.
    pub(crate) fn paste(&mut self, html: &str, text: &str, chord: PasteChord) {
        self.doc.shell.put_html(html.to_owned(), text.to_owned());
        let (key, code, mods) = paste_keys(chord, self.doc.platform);
        self.raw_key(RawKeyInput::down(key.clone(), code).with_mods(mods));
        self.raw_key(
            RawKeyInput::down(key, code)
                .in_phase(RawKeyPhase::Up)
                .with_mods(mods),
        );
    }

    /// The text position the host resolves at `at` inside the first edit surface matching
    /// `selector`, as a press there reports it.
    pub fn hit_test(&self, selector: &str, at: Point) -> Option<TextPosition> {
        self.with_doc(|doc| hit(doc, first(doc, selector)?, at))
    }

    /// Whether the document has the IME on (a surface switches it on as it takes the keyboard).
    pub fn ime_switch(&self) -> ImeSwitch {
        self.doc.shell.ime_switch()
    }

    /// Where the document last put the IME's candidate window.
    pub fn ime_cursor_area(&self) -> Option<Rect> {
        self.doc.shell.ime_area()
    }

    /// Hand a pointer move or primary release to the edit surface holding the pointer, if one
    /// does.
    pub(crate) fn route_captured(&mut self, event: &UiEvent) {
        let (phase, pointer) = match event {
            UiEvent::PointerMove(pointer) => (PointerPhase::Drag, pointer),
            UiEvent::PointerUp(pointer) if pointer.button == MouseEventButton::Main => {
                (PointerPhase::Release, pointer)
            }
            _ => return,
        };
        let Some(sink) = self.doc.listeners.captured(phase) else {
            return;
        };
        let captured = CapturedPointer {
            phase,
            at: Point {
                x: Px(pointer.coords.client_x),
                y: Px(pointer.coords.client_y),
            },
            modifiers: pointer.mods,
        };
        self.doc.doc.vdom.in_runtime(|| sink.call(captured));
    }

    /// Hand `event` to the surface that has the keyboard, and bring the document up to date.
    fn ime_event(&mut self, event: ImeEvent) {
        let sink = self.with_doc(|doc| self.doc.listeners.target(doc));
        if let Some(sink) = sink {
            self.doc.doc.vdom.in_runtime(|| sink.call(event));
        }
        self.settle_now();
    }
}
