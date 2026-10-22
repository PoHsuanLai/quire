//! The system clipboard, for the app: `write_text` behind a "Copy address" item,
//! `read_text` behind a paste. Copy and paste inside a text field need nothing from the app:
//! Blitz handles Ctrl+C/X/V there through the same shell provider.
//!
//! The clipboard is a [`Clipboard`], reached through a context every ds-blitz document
//! provides: [`System`], the window's winit shell (blitz-shell's `clipboard` feature, over
//! `arboard`), or [`Memory`], the harness's, so a test copies and pastes without touching the
//! desktop's clipboard.
//!
//! Files ride the same clipboard: `write_files` / `read_files` take a typed [`ClipboardFiles`]
//! (an operation and paths). The system clipboard carries them as `text/uri-list`, so another
//! app sees a copy; a cut is the app's own state. The harness's memory clipboard holds both
//! `text/uri-list` and `x-special/gnome-copied-files`.
//!
//! `text/html` (an edit surface's paste) is not in blitz's `ShellProvider`, so the window reads it
//! from `arboard` itself and the harness from its memory clipboard's HTML slot.
//!
//! The system clipboard is another program's, asked at the moment of the paste, so a read does
//! not run on the UI thread: it runs on a worker with a short wait and a size cap (`read`), and
//! an owner that is slow, silent or enormous is read as an empty clipboard. quire asks for the
//! selection with the window's own connection and does not use the privileged data-control
//! protocol (FINDINGS "Clipboard"), which would let a client read the clipboard unfocused.

mod files;
mod primary;
mod read;

pub use self::files::{ClipboardFiles, FileOp};

use self::read::Reader;
use crate::memory_shell::MemoryShell;
use blitz_traits::shell::ShellProvider;
use dioxus::prelude::*;
use ds::host::pasted::Pasted;
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
    /// The document has no window shell to reach the system clipboard through: a root
    /// [`provide_host`](crate::provide_host) wired by itself, not one `launch` built. Nothing was
    /// written.
    #[error("this document has no window shell to reach the system clipboard through")]
    NoShell,
}

/// Which selection a read or write reaches.
///
/// The regular clipboard is what Copy and Paste use. The primary selection is the text last
/// selected, pasted by a middle click, on X11 and (where the compositor has it) Wayland; a
/// terminal's OSC 52 `p` writes it. A platform with no primary selection answers
/// [`ClipboardError::Unavailable`] to a write and nothing to a read: it never falls back to the
/// regular clipboard, which would replace what the person copied.
#[derive(Debug, Clone, Copy, PartialEq, Eq, Hash, Default)]
#[non_exhaustive]
pub enum ClipboardTarget {
    /// The regular clipboard.
    #[default]
    Clipboard,
    /// The primary selection.
    Primary,
}

/// A clipboard a document's app and its edit surfaces read and write.
pub trait Clipboard {
    /// The text on `target`.
    fn read_from(&self, target: ClipboardTarget) -> Option<String>;
    /// Put `text` on `target`, or say why it was not put.
    fn write_to(&self, target: ClipboardTarget, text: &str) -> Result<(), ClipboardError>;
    /// The text on the regular clipboard.
    fn read_text(&self) -> Option<String> {
        self.read_from(ClipboardTarget::Clipboard)
    }
    /// What a paste inserts: the HTML with its plain text when the clipboard has HTML, else the
    /// text.
    fn read_html(&self) -> Option<Pasted>;
    /// Put `text` on the regular clipboard, or say why it was not put.
    fn write_text(&self, text: &str) -> Result<(), ClipboardError> {
        self.write_to(ClipboardTarget::Clipboard, text)
    }
    /// The files on the regular clipboard: a file list, else text that is only paths or URIs.
    fn read_files(&self) -> Option<ClipboardFiles> {
        self.read_text()
            .and_then(|text| ClipboardFiles::from_text(&text))
    }
    /// Put `files` on the regular clipboard, or say why it was not put. The default writes their
    /// paths as text.
    fn write_files(&self, files: &ClipboardFiles) -> Result<(), ClipboardError> {
        self.write_text(&files.to_text())
    }
}

/// The desktop's clipboard, through the window's shell: the shell is reached once the window's
/// document exists (`crate::install`).
#[derive(Default)]
pub struct System {
    shell: RefCell<Option<Arc<dyn ShellProvider>>>,
    /// Runs each read off the UI thread, under a time and a size limit.
    reader: Reader,
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
    fn read_from(&self, target: ClipboardTarget) -> Option<String> {
        match target {
            ClipboardTarget::Clipboard => {
                let shell = self.shell()?;
                self.reader.text(move || shell.get_clipboard_text().ok())
            }
            ClipboardTarget::Primary => self.reader.text(primary::read),
        }
    }

    fn write_to(&self, target: ClipboardTarget, text: &str) -> Result<(), ClipboardError> {
        let text = text.to_owned();
        match target {
            ClipboardTarget::Clipboard => {
                let shell = self.shell().ok_or(ClipboardError::NoShell)?;
                guarded(move || shell.set_clipboard_text(text).ok())
            }
            ClipboardTarget::Primary => guarded(move || primary::write(&text).ok()),
        }
        .ok_or(ClipboardError::Unavailable)
    }

    fn read_files(&self) -> Option<ClipboardFiles> {
        let list = self.reader.text(files::read_system);
        list.as_deref()
            .and_then(ClipboardFiles::from_uri_list)
            .or_else(|| self.read_text().and_then(|text| ClipboardFiles::from_text(&text)))
    }

    fn write_files(&self, set: &ClipboardFiles) -> Result<(), ClipboardError> {
        let paths = set.paths.clone();
        guarded(move || files::write_system(&paths).ok()).ok_or(ClipboardError::Unavailable)
    }

    fn read_html(&self) -> Option<Pasted> {
        let html = self.reader.text(|| {
            arboard::Clipboard::new()
                .and_then(|mut clipboard| clipboard.get().html())
                .ok()
        });
        pasted(html, self.read_text())
    }
}

/// The harness's clipboard: text and HTML in a shell's memory.
#[derive(Debug)]
pub struct Memory(pub Arc<MemoryShell>);

impl Clipboard for Memory {
    fn read_from(&self, target: ClipboardTarget) -> Option<String> {
        match target {
            ClipboardTarget::Clipboard => self.0.text(),
            ClipboardTarget::Primary => self.0.primary_text(),
        }
    }

    fn write_to(&self, target: ClipboardTarget, text: &str) -> Result<(), ClipboardError> {
        match target {
            ClipboardTarget::Clipboard => self.0.put(text.to_owned()),
            ClipboardTarget::Primary => self.0.put_primary(text.to_owned()),
        }
        Ok(())
    }

    fn read_files(&self) -> Option<ClipboardFiles> {
        self.0.files().or_else(|| {
            self.0
                .text()
                .and_then(|text| ClipboardFiles::from_text(&text))
        })
    }

    fn write_files(&self, files: &ClipboardFiles) -> Result<(), ClipboardError> {
        self.0.put_files(files);
        Ok(())
    }

    fn read_html(&self) -> Option<Pasted> {
        pasted(self.0.html(), self.0.text())
    }
}

/// Put `text` on the clipboard. Call it from a handler inside a ds-blitz document.
///
/// # Errors
/// [`ClipboardError::NoShell`] in a root [`provide_host`](crate::provide_host) wired on its own
/// (no window shell to write through), [`ClipboardError::Unavailable`] when the system
/// clipboard refuses.
pub fn write_text(text: &str) -> Result<(), ClipboardError> {
    host()?.write_text(text)
}

/// Put `text` on `target`. Call it from a handler inside a ds-blitz document.
///
/// # Errors
/// As [`write_text`]; and [`ClipboardError::Unavailable`] for [`ClipboardTarget::Primary`] where
/// the platform has no primary selection or its compositor does not offer one.
pub fn write_to(target: ClipboardTarget, text: &str) -> Result<(), ClipboardError> {
    host()?.write_to(target, text)
}

/// The text on `target`. Call it from a handler inside a ds-blitz document.
pub fn read_from(target: ClipboardTarget) -> Result<String, ClipboardError> {
    host()?.read_from(target).ok_or(ClipboardError::Unavailable)
}

/// The text on the clipboard. Call it from a handler inside a ds-blitz document.
pub fn read_text() -> Result<String, ClipboardError> {
    host()?.read_text().ok_or(ClipboardError::Unavailable)
}

/// Put `files` on the clipboard. Call it from a handler inside a ds-blitz document. Another app
/// sees a copy of these files; a cut is the app's own state (see `CONSUMING.md`).
///
/// # Errors
/// As [`write_text`].
pub fn write_files(files: &ClipboardFiles) -> Result<(), ClipboardError> {
    host()?.write_files(files)
}

/// The files on the clipboard. Call it from a handler inside a ds-blitz document.
pub fn read_files() -> Result<ClipboardFiles, ClipboardError> {
    host()?.read_files().ok_or(ClipboardError::Unavailable)
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

/// Run a clipboard write, reading a panic as no answer: blitz-shell unwraps
/// `arboard::Clipboard::new()`, which fails on a session with no clipboard at all. (Reads go
/// through [`Reader`], which runs them on a worker thread, where a panic ends only that thread.)
fn guarded<T>(call: impl FnOnce() -> Option<T>) -> Option<T> {
    catch_unwind(AssertUnwindSafe(call)).ok().flatten()
}
