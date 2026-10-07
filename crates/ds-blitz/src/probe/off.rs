//! The probe with the feature off: nothing.

use crate::node_ref::DocRef;

/// Writes nothing.
#[derive(Debug, Default)]
pub(crate) struct Probe;

impl Probe {
    /// Does nothing.
    pub(crate) fn publish(&self, _doc: &DocRef) {}
}
