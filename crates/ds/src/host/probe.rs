//! One read or write through the host, and how it went.

/// One read or write through the host.
#[derive(Debug, Clone, PartialEq)]
pub enum Probe<T> {
    /// The answer.
    Found(T),
    /// The document is busy (rendering); ask again next frame.
    Busy,
    /// The host cannot answer: no host, the surface is not its node or is gone, nothing
    /// addressable is there, or the document has not been laid out yet.
    Unknown,
}

impl<T> Probe<T> {
    /// The answer, if there was one.
    pub fn found(self) -> Option<T> {
        match self {
            Probe::Found(value) => Some(value),
            Probe::Busy | Probe::Unknown => None,
        }
    }
}
