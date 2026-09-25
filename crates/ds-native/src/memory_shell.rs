//! The headless document's shell: a clipboard in memory (text and HTML), so a harness test
//! copies from one field and pastes into another, and reads what a copy wrote, without the
//! desktop's clipboard; and the IME requests an edit surface makes, so a test sees them. Nothing
//! else a shell does (a cursor, a title, a redraw) has anywhere to go headless.

use blitz_traits::shell::{ClipboardError, ShellProvider};
use ds::{ImeSwitch, Point, Px, Rect, Size};
use std::sync::{Mutex, PoisonError};

/// A shell whose clipboard and IME state are in memory.
#[derive(Debug)]
pub(crate) struct MemoryShell {
    clipboard: Mutex<Option<String>>,
    html: Mutex<Option<String>>,
    ime: Mutex<ImeState>,
}

/// What the document last asked of the IME.
#[derive(Debug, Clone, Copy, PartialEq)]
struct ImeState {
    switch: ImeSwitch,
    area: Option<Rect>,
}

impl Default for MemoryShell {
    fn default() -> Self {
        MemoryShell {
            clipboard: Mutex::new(None),
            html: Mutex::new(None),
            ime: Mutex::new(ImeState {
                switch: ImeSwitch::Off,
                area: None,
            }),
        }
    }
}

fn lock<T>(mutex: &Mutex<T>) -> std::sync::MutexGuard<'_, T> {
    mutex.lock().unwrap_or_else(PoisonError::into_inner)
}

impl MemoryShell {
    /// What was last copied, if anything.
    pub(crate) fn text(&self) -> Option<String> {
        lock(&self.clipboard).clone()
    }

    /// The clipboard's HTML, if the last copy had any.
    pub(crate) fn html(&self) -> Option<String> {
        lock(&self.html).clone()
    }

    /// Put `text` on the clipboard, as another app's copy would; it has no HTML.
    pub(crate) fn put(&self, text: String) {
        *lock(&self.clipboard) = Some(text);
        *lock(&self.html) = None;
    }

    /// Put `html` and its plain `text` on the clipboard, as a browser's copy would.
    pub(crate) fn put_html(&self, html: String, text: String) {
        *lock(&self.clipboard) = Some(text);
        *lock(&self.html) = Some(html);
    }

    /// Whether the document has the IME on.
    pub(crate) fn ime_switch(&self) -> ImeSwitch {
        lock(&self.ime).switch
    }

    /// Where the document last put the IME's candidate window.
    pub(crate) fn ime_area(&self) -> Option<Rect> {
        lock(&self.ime).area
    }
}

impl ShellProvider for MemoryShell {
    fn get_clipboard_text(&self) -> Result<String, ClipboardError> {
        self.text().ok_or(ClipboardError)
    }

    fn set_clipboard_text(&self, text: String) -> Result<(), ClipboardError> {
        self.put(text);
        Ok(())
    }

    fn set_ime_enabled(&self, is_enabled: bool) {
        lock(&self.ime).switch = if is_enabled {
            ImeSwitch::On
        } else {
            ImeSwitch::Off
        };
    }

    fn set_ime_cursor_area(&self, x: f32, y: f32, width: f32, height: f32) {
        lock(&self.ime).area = Some(Rect {
            origin: Point { x: Px(x), y: Px(y) },
            size: Size {
                width: Px(width),
                height: Px(height),
            },
        });
    }
}
