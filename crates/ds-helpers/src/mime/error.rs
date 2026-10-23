//! Why a name or a type is refused, or the database is missing, in the words a person reads.

/// Why a name was refused, in the words a person reads.
#[derive(Debug, Clone, Copy, PartialEq, Eq, thiserror::Error)]
#[non_exhaustive]
pub enum NameError {
    /// Nothing was typed.
    #[error("A name can't be empty.")]
    Empty,
    /// `.` or `..`, which name this folder and the one above it.
    #[error("The names \u{201c}.\u{201d} and \u{201c}..\u{201d} are reserved.")]
    Reserved,
    /// A `/`, which separates folders in a path.
    #[error("A name can't contain \u{201c}/\u{201d}.")]
    Slash,
    /// A NUL character, which no Linux name may hold.
    #[error("A name can't contain a null character.")]
    Nul,
    /// Over 255 bytes.
    #[error("That name is too long.")]
    TooLong,
}

/// Why a text is not a MIME type.
#[derive(Debug, Clone, Copy, PartialEq, Eq, thiserror::Error)]
#[non_exhaustive]
pub enum MimeError {
    /// It is not one `type/subtype` pair of non-empty parts.
    #[error("not a type/subtype pair")]
    Shape,
}

/// Why the file type database could not be loaded.
#[derive(Debug, Clone, Copy, PartialEq, Eq, thiserror::Error)]
#[non_exhaustive]
pub enum LoadError {
    /// No data folder held any file name patterns.
    #[error("No file type database was found.")]
    NoDatabase,
}
