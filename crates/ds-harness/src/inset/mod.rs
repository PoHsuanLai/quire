//! The content inset check: text, glyphs and images keep their distance from the edge of the
//! box they sit in.
//!
//! A *visible box* is an element that paints an edge: a background that differs from what is
//! behind it, a gradient, a border, an outline, a shadow, or a highlighted state. The check reads
//! the laid-out document ([`read`]), gives every text run (from the inline layouts' glyph
//! runs), glyph, icon and image to its nearest box, and measures the gap from each painted
//! vertical edge to the nearest content ([`measure`]). A gap under the [`Policy`] minimum
//! (`--s-8`, 8 px, the menu and row text inset) is an [`Offence`]; a class the design holds to a
//! smaller inset is an [`Allow`] entry with its reason.
//!
//! What it cannot see: a filled child box (a dot, an avatar, a switch thumb) is not content, only
//! text and glyphs are; an inline span's own background is not a box (its text is measured
//! against the box around it); a transform or an inner scroller's offset is not applied to
//! rectangles; text is its glyph runs' advance boxes, a side bearing wider than the ink.
//!
//! [`assert_insets`] fails a test with the offences listed; [`Harness::insets`] returns them.

mod bounds;
mod ink;
mod measure;
mod offence;
mod paint;
mod policy;
mod read;
mod scene;

pub use bounds::Bounds;
pub use measure::measure;
pub use offence::{Excused, Findings, Offence, Side};
pub use policy::{Allow, Policy, text_inset};
pub use read::read;
pub use scene::{Content, ContentKind, Edges, Paint, Scene, VisibleBox};

use crate::driver::DocQuery;

/// Check every visible box of the document against `policy`.
pub fn insets(doc: &impl DocQuery, policy: &Policy) -> Findings {
    doc.with_doc(|doc| measure(&read(doc), policy))
}

/// Fail with the offences listed when any box has content closer to a painted edge than
/// `policy` allows.
pub fn assert_insets(doc: &impl DocQuery, policy: &Policy) {
    let findings = insets(doc, policy);
    assert!(findings.offences.is_empty(), "{}", findings.text());
}
