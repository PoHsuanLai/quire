//! What a host needs to decide how an icon it did not draw is shown, and to recolour one.

pub use crate::style::icon::{
    classify::{IconKind, classify_with},
    retint::{IconStyle, Tint, retint, retint_in},
    stroke::stroke_device_pixels,
};

/// The squircle plate's geometry a host sizes its own shadows from.
pub mod plate {
    pub use crate::style::tokens::plate::shadow_radius_share;
}
