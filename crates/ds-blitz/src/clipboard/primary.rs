//! The primary selection: the text last selected, pasted by a middle click. Only X11 and Wayland
//! have one, so only Linux reaches it (through arboard's Linux extension); every other platform
//! reads nothing and refuses a write.

use super::ClipboardError;

/// The text on the primary selection.
#[cfg(target_os = "linux")]
pub(super) fn read() -> Option<String> {
    use arboard::{GetExtLinux, LinuxClipboardKind};
    arboard::Clipboard::new()
        .and_then(|mut clipboard| {
            clipboard
                .get()
                .clipboard(LinuxClipboardKind::Primary)
                .text()
        })
        .ok()
}

/// Put `text` on the primary selection.
///
/// The `arboard::Clipboard` that wrote it stays alive for the process: on X11 it owns the
/// selection and answers every paste request, and dropping it destroys its window and the
/// selection with it (only the regular clipboard is handed to a clipboard manager on drop).
/// The next write replaces it.
#[cfg(target_os = "linux")]
pub(super) fn write(text: &str) -> Result<(), ClipboardError> {
    use arboard::{LinuxClipboardKind, SetExtLinux};
    use std::sync::{Mutex, OnceLock, PoisonError};
    static OWNER: OnceLock<Mutex<Option<arboard::Clipboard>>> = OnceLock::new();
    let mut owner = OWNER
        .get_or_init(|| Mutex::new(None))
        .lock()
        .unwrap_or_else(PoisonError::into_inner);
    if owner.is_none() {
        *owner = Some(arboard::Clipboard::new().map_err(|_| ClipboardError::Unavailable)?);
    }
    let clipboard = owner.as_mut().ok_or(ClipboardError::Unavailable)?;
    clipboard
        .set()
        .clipboard(LinuxClipboardKind::Primary)
        .text(text)
        .map_err(|_| ClipboardError::Unavailable)
}

/// No primary selection here.
#[cfg(not(target_os = "linux"))]
pub(super) fn read() -> Option<String> {
    None
}

/// No primary selection here.
#[cfg(not(target_os = "linux"))]
pub(super) fn write(_text: &str) -> Result<(), ClipboardError> {
    Err(ClipboardError::Unavailable)
}
