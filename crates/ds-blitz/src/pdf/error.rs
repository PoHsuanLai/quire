//! What can stop a PDF from being made.

/// A PDF that was not made.
#[derive(Debug, thiserror::Error)]
pub enum PdfError {
    /// The margins leave no room on the sheet for any content.
    #[error("the margins leave no room on the page")]
    NoContentArea,
    /// pdfrum could not write the file (a face it cannot read or subset, an image it cannot
    /// embed); the message is pdfrum's.
    #[error("the PDF could not be written: {0}")]
    Write(String),
}
