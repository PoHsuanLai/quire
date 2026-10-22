//! The headless document's shell: a clipboard in memory (text and HTML) and a primary
//! selection, so a harness test copies from one field and pastes into another, and reads what a
//! copy wrote, without the desktop's clipboard; and the IME requests an edit surface makes, so a test sees them. Nothing
//! else a shell does (a cursor, a title, a redraw) has anywhere to go headless.

use crate::clipboard::ClipboardFiles;
use blitz_traits::shell::{ClipboardError, ShellProvider};
use ds::host::ime::ImeSwitch;
use ds::prelude::*;
use std::sync::{Mutex, PoisonError};

/// A shell whose clipboard and IME state are in memory.
#[derive(Debug)]
pub struct MemoryShell {
    clipboard: Mutex<Option<String>>,
    html: Mutex<Option<String>>,
    primary: Mutex<Option<String>>,
    files: Mutex<FileFormats>,
    ime: Mutex<ImeState>,
}

/// The file formats on the clipboard, as the owning app offered them.
#[derive(Debug, Default)]
struct FileFormats {
    uri_list: Option<String>,
    gnome_copied: Option<String>,
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
            primary: Mutex::new(None),
            files: Mutex::new(FileFormats::default()),
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
    pub fn text(&self) -> Option<String> {
        lock(&self.clipboard).clone()
    }

    /// The clipboard's HTML, if the last copy had any.
    pub(crate) fn html(&self) -> Option<String> {
        lock(&self.html).clone()
    }

    /// The files on the clipboard: `x-special/gnome-copied-files` if offered, else
    /// `text/uri-list` (a copy).
    pub(crate) fn files(&self) -> Option<ClipboardFiles> {
        let formats = lock(&self.files);
        formats
            .gnome_copied
            .as_deref()
            .and_then(ClipboardFiles::from_gnome_copied)
            .or_else(|| {
                formats
                    .uri_list
                    .as_deref()
                    .and_then(ClipboardFiles::from_uri_list)
            })
    }

    /// Put `files` on the clipboard in every format: both file formats and their paths as text.
    pub fn put_files(&self, files: &ClipboardFiles) {
        self.put(files.to_text());
        *lock(&self.files) = FileFormats {
            uri_list: Some(files.to_uri_list()),
            gnome_copied: Some(files.to_gnome_copied()),
        };
    }

    /// Put only a `text/uri-list` on the clipboard, as an app that offers no other format would.
    pub fn put_uri_list(&self, list: String) {
        self.put(String::new());
        *lock(&self.files) = FileFormats {
            uri_list: Some(list),
            gnome_copied: None,
        };
    }

    /// Put only an `x-special/gnome-copied-files` on the clipboard, as a file manager's would.
    pub fn put_gnome_copied(&self, data: String) {
        self.put(String::new());
        *lock(&self.files) = FileFormats {
            uri_list: None,
            gnome_copied: Some(data),
        };
    }

    /// The `text/uri-list` on the clipboard, if offered.
    pub fn uri_list(&self) -> Option<String> {
        lock(&self.files).uri_list.clone()
    }

    /// The `x-special/gnome-copied-files` on the clipboard, if offered.
    pub fn gnome_copied(&self) -> Option<String> {
        lock(&self.files).gnome_copied.clone()
    }

    /// Put `text` on the clipboard, as another app's copy would; it has no HTML.
    pub fn put(&self, text: String) {
        *lock(&self.files) = FileFormats::default();
        *lock(&self.clipboard) = Some(text);
        *lock(&self.html) = None;
    }

    /// Put `html` and its plain `text` on the clipboard, as a browser's copy would.
    pub fn put_html(&self, html: String, text: String) {
        *lock(&self.files) = FileFormats::default();
        *lock(&self.clipboard) = Some(text);
        *lock(&self.html) = Some(html);
    }

    /// What was last put on the primary selection, if anything.
    pub fn primary_text(&self) -> Option<String> {
        lock(&self.primary).clone()
    }

    /// Put `text` on the primary selection, as a selection in another app would.
    pub fn put_primary(&self, text: String) {
        *lock(&self.primary) = Some(text);
    }

    /// Whether the document has the IME on.
    pub fn ime_switch(&self) -> ImeSwitch {
        lock(&self.ime).switch
    }

    /// Where the document last put the IME's candidate window.
    pub fn ime_area(&self) -> Option<Rect> {
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
