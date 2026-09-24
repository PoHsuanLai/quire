//! The headless document's shell: a clipboard in memory, so a harness test copies from one
//! field and pastes into another, and reads what a copy wrote, without the desktop's clipboard.
//! Nothing else a shell does (a cursor, a title, a redraw) has anywhere to go headless.

use blitz_traits::shell::{ClipboardError, ShellProvider};
use std::sync::{Mutex, PoisonError};

/// A shell whose clipboard is one string in memory.
#[derive(Debug, Default)]
pub(crate) struct MemoryShell {
    clipboard: Mutex<Option<String>>,
}

impl MemoryShell {
    /// What was last copied, if anything.
    pub(crate) fn text(&self) -> Option<String> {
        self.clipboard
            .lock()
            .unwrap_or_else(PoisonError::into_inner)
            .clone()
    }

    /// Put `text` on the clipboard, as another app's copy would.
    pub(crate) fn put(&self, text: String) {
        *self
            .clipboard
            .lock()
            .unwrap_or_else(PoisonError::into_inner) = Some(text);
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
}
