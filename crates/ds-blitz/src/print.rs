//! Handing a PDF to the system to print (the `print` feature). On Linux that is the desktop
//! portal's print dialog (`org.freedesktop.portal.Print`, implemented by the GTK and KDE portal
//! backends; COSMIC's falls back to GTK's): the dialog chooses the printer and settings, and
//! the portal prints the PDF from a sealed in-memory file. Where there is no portal or no print
//! backend, and on every other system, the PDF is written to a temporary file and opened in the
//! system's viewer, which prints it.
//!
//! The call blocks until the person has answered the dialog; call it off the UI thread (a
//! `spawn_blocking`, a worker thread). The dialog is not parented to the app's window: the
//! portal wants `wayland:<xdg-foreign handle>` for that, and neither winit nor `launch` exports
//! one yet, so it opens as its own window. A GTK portal backend has been seen to lose the file
//! on a second PreparePrint + Print in one backend process (xdg-desktop-portal-gtk issue #562);
//! a lost print is not detected here.

mod open;
#[cfg(target_os = "linux")]
mod portal;

use std::path::PathBuf;

/// What became of a print.
#[derive(Debug, Clone, PartialEq, Eq)]
pub enum PrintOutcome {
    /// The dialog was accepted and the portal took the PDF for printing.
    Printed,
    /// The person closed the dialog without printing.
    Cancelled,
    /// There was no print dialog; the PDF was written here and opened in the system's viewer.
    Opened(PathBuf),
}

/// Why a PDF could not be handed over.
#[derive(Debug, thiserror::Error)]
pub enum PrintError {
    /// The print dialog was shown but the portal failed afterwards.
    #[error("the print portal failed: {0}")]
    Portal(String),
    /// The temporary file for the viewer could not be written.
    #[error("the PDF could not be written for the viewer: {0}")]
    Write(#[from] std::io::Error),
    /// No program opened the PDF.
    #[error("no viewer opened the PDF: {0}")]
    NoViewer(String),
}

/// Show the system print dialog for `pdf`, titled `title`, and print it if the person accepts;
/// where there is no dialog, open it in the system's viewer instead.
pub fn print_dialog(pdf: &[u8], title: &str) -> Result<PrintOutcome, PrintError> {
    #[cfg(target_os = "linux")]
    match portal::print(pdf, title) {
        portal::Attempt::Answered(outcome) => return outcome,
        portal::Attempt::Unavailable => {}
    }
    open::open(pdf, title)
}
