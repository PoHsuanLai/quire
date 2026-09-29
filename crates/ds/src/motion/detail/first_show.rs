//! Whether a value's first frame is an Appear (design/26-DETAILS.md R1).

/// Whether the surface showing an element was just opened by the person (Appear plays), or the
/// element is always-there chrome or re-mounted in place (it does not). Only Appear is held back
/// by `Still`: an operation already running on mount is still Pending.
#[derive(Debug, Clone, Copy, PartialEq, Eq, Hash, Default)]
pub enum FirstShow {
    /// The surface was just opened: the first frame plays its Appear.
    Animate,
    /// The element was already there: no Appear.
    #[default]
    Still,
}
