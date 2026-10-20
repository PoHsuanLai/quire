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
#[cfg(target_os = "linux")]
pub(super) fn write(text: &str) -> Result<(), ClipboardError> {
    use arboard::{LinuxClipboardKind, SetExtLinux};
    arboard::Clipboard::new()
        .and_then(|mut clipboard| {
            clipboard
                .set()
                .clipboard(LinuxClipboardKind::Primary)
                .text(text)
        })
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
