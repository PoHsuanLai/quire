//! Snapping a laid-out document to the device pixel grid (FINDINGS "Pixel snapping"): the
//! rounding is `blitz_kit::snap::snap_layout`; this reads the scale off the document's viewport.
//!
//! It must run after every `resolve` and before painting, because every resolve re-runs taffy's
//! rounding. `ds-blitz`'s headless documents (`Harness`, `snapshot`) run it; a host that drives
//! its own documents (shell-host) calls it after each `resolve`. At a whole scale it does
//! nothing: Blitz's rounding is already right there, and every picture stays as it was.

use blitz_dom::BaseDocument;
use blitz_kit::snap::snap_layout;
use blitz_kit::units::Scale120;

/// Snap every box of `doc` to its viewport's device pixel grid; nothing at a whole scale.
pub fn snap_to_device(doc: &mut BaseDocument) {
    let scale = Scale120::from_factor(doc.viewport().scale_f64());
    snap_layout(doc, scale);
}
