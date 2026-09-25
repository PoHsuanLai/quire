//! Driving an edit surface in a test: the IME a person would use, a paste with HTML, and the
//! host's hit test and IME requests read back. The IME events go through the same routing the
//! window uses (`crate::edit_ime`): to the surface that has the keyboard.

use crate::edit_hit::hit;
use crate::harness::{Harness, first};
use ds::{ImeEvent, ImeSwitch, Key, Point, Rect, TextPosition};

impl Harness {
    /// The IME attaches to the focused surface (winit's `Ime::Enabled`); a composition starts
    /// with the first [`Harness::ime_update`].
    pub fn ime_start(&mut self) {
        self.ime(ImeEvent::Enabled);
    }

    /// The IME shows `text` as its preedit, with its cursor at bytes `cursor` of it.
    pub fn ime_update(&mut self, text: &str, cursor: usize) {
        self.ime(ImeEvent::Preedit {
            text: text.to_owned(),
            cursor: Some((cursor, cursor)),
        });
    }

    /// The IME commits `text`, clearing its preedit first as winit does.
    pub fn ime_commit(&mut self, text: &str) {
        self.ime(ImeEvent::Preedit {
            text: String::new(),
            cursor: None,
        });
        self.ime(ImeEvent::Commit(text.to_owned()));
    }

    /// The IME detaches (winit's `Ime::Disabled`).
    pub fn ime_end(&mut self) {
        self.ime(ImeEvent::Disabled);
    }

    /// Put `html` and its plain `text` on the clipboard, as a browser's copy would, and press
    /// Ctrl+V where the focus is.
    pub fn paste_html(&mut self, html: &str, text: &str) {
        self.doc.shell.put_html(html.to_owned(), text.to_owned());
        self.chord(&[Key::Ctrl], Key::Char('v'));
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

    /// Hand `event` to the surface that has the keyboard, and bring the document up to date.
    fn ime(&mut self, event: ImeEvent) {
        let sink = self.with_doc(|doc| self.doc.listeners.target(doc));
        if let Some(sink) = sink {
            self.doc.doc.vdom.in_runtime(|| sink.call(event));
        }
        self.settle_now();
    }
}
