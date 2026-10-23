//! The limits of the cost notes (DESIGN.md section 7.5, recorded in CONVENTIONS.md, "Cost notes
//! on a user stylesheet"). Each is a line past which CSS is legal but slows a frame; crossing one
//! earns a note, never a refusal.

/// The largest blur radius, in CSS pixels, that `backdrop-filter` or `filter: blur()` may name
/// without a note. quire's own materials stay under it.
pub const MAX_BLUR_RADIUS_PX: f32 = 24.0;

/// The most blur, in CSS pixels, that one `box-shadow` may add up to over all its stacked layers
/// without a note.
pub const MAX_SHADOW_BLUR_TOTAL_PX: f32 = 80.0;

/// The size of one `em` or `rem`, in CSS pixels, when a length is converted to pixels.
pub const EM_PX: f32 = 16.0;

/// The attribute that marks a surface root (`ds::selectors::SURFACE_ATTRIBUTE`; a test keeps the
/// two equal). Any blur on a rule that targets one is noted, whatever its radius, because it
/// covers a whole window.
pub const SURFACE_ATTRIBUTE: &str = "data-surface";

/// The public parts that are spinners: an animation that never stops is their job, so it earns
/// no note there. Class names without the dot.
pub const SPINNER_CLASSES: &[&str] = &[
    "ds-progress",
    "ds-progress-indicator",
    "ds-progress-glyph",
    "ds-loadable-spin",
    "ds-skeleton",
];
