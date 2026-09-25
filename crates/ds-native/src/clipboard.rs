//! The system clipboard, for the app (G5): `write_text` behind a "Copy address" item,
//! `read_text` behind a paste. Copy and paste inside a text field need nothing from the app:
//! Blitz handles Ctrl+C/X/V there through the same shell provider.
//!
//! The clipboard is the document's shell provider's, reached through a context every ds-native
//! document provides: the window's winit shell (blitz-shell's `clipboard` feature, over
//! `arboard`), or the harness's in-memory one, so a test copies and pastes without touching the
//! desktop's clipboard.
//!
//! `text/html` (an edit surface's paste, `read_html`) is not in blitz's `ShellProvider`, so the
//! window reads it from `arboard` itself and the harness from its memory clipboard's HTML slot.

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
    /// Not called inside a ds-native document (`launch`, the harness), or before the window's
    /// document exists: there is no clipboard to reach. Call it from a handler.
    #[error("no ds-native document to reach a clipboard through")]
    NoHost,
    /// The system clipboard refused, or holds no text.
    #[error("the clipboard is unavailable or holds no text")]
    Unavailable,
}

/// The shell whose clipboard a document's app uses: set when the window's document exists
/// (`crate::install`), at once in a headless one; and where its HTML is read.
#[derive(Clone, Default)]
pub(crate) struct HostClipboard {
    shell: Rc<RefCell<Option<Arc<dyn ShellProvider>>>>,
    html: HtmlSource,
}

/// Where a document's clipboard HTML comes from.
#[derive(Clone, Default)]
enum HtmlSource {
    /// The desktop's clipboard, through arboard: the window.
    #[default]
    System,
    /// The harness's memory clipboard.
    Memory(Arc<MemoryShell>),
}

impl HostClipboard {
    /// The harness's clipboard: text and HTML in `shell`'s memory.
    pub(crate) fn memory(shell: Arc<MemoryShell>) -> Self {
        let clipboard = HostClipboard {
            shell: Rc::default(),
            html: HtmlSource::Memory(Arc::clone(&shell)),
        };
        clipboard.set(shell);
        clipboard
    }

    /// Reach the clipboard through `shell` from now on.
    pub(crate) fn set(&self, shell: Arc<dyn ShellProvider>) {
        self.shell.replace(Some(shell));
    }

    fn shell(&self) -> Result<Arc<dyn ShellProvider>, ClipboardError> {
        self.shell.borrow().clone().ok_or(ClipboardError::NoHost)
    }

    fn html(&self) -> Result<String, ClipboardError> {
        match &self.html {
            HtmlSource::Memory(shell) => shell.html().ok_or(ClipboardError::Unavailable),
            HtmlSource::System => {
                let mut read = None;
                guarded(|| {
                    read = arboard::Clipboard::new()
                        .and_then(|mut clipboard| clipboard.get().html())
                        .ok();
                    read.is_some()
                })?;
                read.ok_or(ClipboardError::Unavailable)
            }
        }
    }
}

/// Put `text` on the clipboard. Call it from a handler inside a ds-native document.
pub fn write_text(text: &str) -> Result<(), ClipboardError> {
    let shell = host()?.shell()?;
    let text = text.to_owned();
    guarded(move || shell.set_clipboard_text(text).is_ok())
}

/// The text on the clipboard. Call it from a handler inside a ds-native document.
pub fn read_text() -> Result<String, ClipboardError> {
    let shell = host()?.shell()?;
    let mut read = None;
    guarded(|| {
        read = shell.get_clipboard_text().ok();
        read.is_some()
    })?;
    read.ok_or(ClipboardError::Unavailable)
}

/// The clipboard's `text/html`. Call it from a handler inside a ds-native document.
pub fn read_html() -> Result<String, ClipboardError> {
    host()?.html()
}

/// What a paste inserts: the HTML with its plain text when the clipboard has HTML, else the text.
pub(crate) fn read_pasted() -> Option<Pasted> {
    let text = read_text().ok();
    match (read_html().ok().filter(|html| !html.is_empty()), text) {
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
fn host() -> Result<HostClipboard, ClipboardError> {
    dioxus::core::Runtime::try_current()
        .and_then(|_| try_consume_context::<HostClipboard>())
        .ok_or(ClipboardError::NoHost)
}

/// Run a clipboard call, reading a refusal or a panic as `Unavailable`: blitz-shell unwraps
/// `arboard::Clipboard::new()`, which fails on a session with no clipboard at all.
fn guarded(call: impl FnOnce() -> bool) -> Result<(), ClipboardError> {
    match catch_unwind(AssertUnwindSafe(call)) {
        Ok(true) => Ok(()),
        Ok(false) | Err(_) => Err(ClipboardError::Unavailable),
    }
}
