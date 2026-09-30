//! The system clipboard, for the app: `write_text` behind a "Copy address" item,
//! `read_text` behind a paste. Copy and paste inside a text field need nothing from the app:
//! Blitz handles Ctrl+C/X/V there through the same shell provider.
//!
//! The clipboard is a [`Clipboard`], reached through a context every ds-blitz document
//! provides: [`System`], the window's winit shell (blitz-shell's `clipboard` feature, over
//! `arboard`), or [`Memory`], the harness's, so a test copies and pastes without touching the
//! desktop's clipboard.
//!
//! `text/html` (an edit surface's paste) is not in blitz's `ShellProvider`, so the window reads it
//! from `arboard` itself and the harness from its memory clipboard's HTML slot.

use crate::memory_shell::MemoryShell;
use blitz_traits::shell::ShellProvider;
use dioxus::prelude::*;
use ds::Pasted;
use std::cell::RefCell;
use std::panic::{AssertUnwindSafe, catch_unwind};
use std::rc::Rc;
use std::sync::Arc;

/// Why the clipboard was not read or written.
#[derive(Debug, Clone, Copy, PartialEq, Eq, Hash, thiserror::Error)]
pub enum ClipboardError {
    /// Not called inside a ds-blitz document (`launch`, the harness): there is no clipboard to
    /// reach. Call it from a handler.
    #[error("no ds-blitz document to reach a clipboard through")]
    NoHost,
    /// The system clipboard refused, or holds no text.
    #[error("the clipboard is unavailable or holds no text")]
    Unavailable,
}

/// A clipboard a document's app and its edit surfaces read and write.
pub trait Clipboard {
    /// The text on the clipboard.
    fn read_text(&self) -> Option<String>;
    /// What a paste inserts: the HTML with its plain text when the clipboard has HTML, else the
    /// text.
    fn read_html(&self) -> Option<Pasted>;
    /// Put `text` on the clipboard.
    fn write_text(&self, text: &str);
}

/// The desktop's clipboard, through the window's shell: the shell is reached once the window's
/// document exists (`crate::install`).
#[derive(Default)]
pub struct System {
    shell: RefCell<Option<Arc<dyn ShellProvider>>>,
}

impl std::fmt::Debug for System {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        f.write_str("System")
    }
}

impl System {
    /// Reach the clipboard through `shell` from now on.
    pub(crate) fn reach(&self, shell: Arc<dyn ShellProvider>) {
        self.shell.replace(Some(shell));
    }

    fn shell(&self) -> Option<Arc<dyn ShellProvider>> {
        self.shell.borrow().clone()
    }
}

impl Clipboard for System {
    fn read_text(&self) -> Option<String> {
        let shell = self.shell()?;
        guarded(|| shell.get_clipboard_text().ok())
    }

    fn read_html(&self) -> Option<Pasted> {
        let html = guarded(|| {
            arboard::Clipboard::new()
                .and_then(|mut clipboard| clipboard.get().html())
                .ok()
        });
        pasted(html, self.read_text())
    }

    fn write_text(&self, text: &str) {
        if let Some(shell) = self.shell() {
            let text = text.to_owned();
            guarded(move || shell.set_clipboard_text(text).ok());
        }
    }
}

/// The harness's clipboard: text and HTML in a shell's memory.
#[derive(Debug)]
pub struct Memory(pub Arc<MemoryShell>);

impl Clipboard for Memory {
    fn read_text(&self) -> Option<String> {
        self.0.text()
    }

    fn read_html(&self) -> Option<Pasted> {
        pasted(self.0.html(), self.0.text())
    }

    fn write_text(&self, text: &str) {
        self.0.put(text.to_owned());
    }
}

/// Put `text` on the clipboard. Call it from a handler inside a ds-blitz document.
pub fn write_text(text: &str) -> Result<(), ClipboardError> {
    host()?.write_text(text);
    Ok(())
}

/// The text on the clipboard. Call it from a handler inside a ds-blitz document.
pub fn read_text() -> Result<String, ClipboardError> {
    host()?.read_text().ok_or(ClipboardError::Unavailable)
}

/// The clipboard's `text/html`. Call it from a handler inside a ds-blitz document.
pub fn read_html() -> Result<String, ClipboardError> {
    match host()?.read_html() {
        Some(Pasted::Html { html, .. }) => Ok(html),
        Some(Pasted::Text(_)) | None => Err(ClipboardError::Unavailable),
    }
}

/// What a paste inserts: the HTML with its plain text when there is HTML, else the text.
fn pasted(html: Option<String>, text: Option<String>) -> Option<Pasted> {
    match (html.filter(|html| !html.is_empty()), text) {
        (Some(html), text) => Some(Pasted::Html {
            html,
            text: text.unwrap_or_default(),
        }),
        (None, Some(text)) => Some(Pasted::Text(text)),
        (None, None) => None,
    }
}

/// The calling document's clipboard; outside any Dioxus runtime there is none (asking would
/// panic).
fn host() -> Result<Rc<dyn Clipboard>, ClipboardError> {
    dioxus::core::Runtime::try_current()
        .and_then(|_| try_consume_context::<Rc<dyn Clipboard>>())
        .ok_or(ClipboardError::NoHost)
}

/// Run a clipboard call, reading a panic as no answer: blitz-shell unwraps
/// `arboard::Clipboard::new()`, which fails on a session with no clipboard at all.
fn guarded<T>(call: impl FnOnce() -> Option<T>) -> Option<T> {
    catch_unwind(AssertUnwindSafe(call)).ok().flatten()
}
