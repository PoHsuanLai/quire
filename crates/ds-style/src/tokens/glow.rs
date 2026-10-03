//! The glow the compositor draws around the window the companion acts in (design/32 section 3).
//! quire owns the values and the compositor links this crate to read them, so the glow it draws
//! is the one the design system describes; this module is plain data and names no renderer.

use crate::appearance::theme::Scheme;
use ds_core::colour::srgb::Srgb;
use ds_core::geometry::units::Px;
use ds_core::vocab::Fraction;
use std::time::Duration;

/// A spot inside the window the glow brightens, in logical pixels.
#[derive(Debug, Clone, Copy, PartialEq)]
pub struct GlowSpot {
    /// Across, from the window's left edge.
    pub x: Px,
    /// Down, from the window's top edge.
    pub y: Px,
    /// How far the spot reaches.
    pub radius: Px,
}

/// What the glow shows: the view input to [`glow_spec`].
#[derive(Debug, Clone, Copy, PartialEq)]
pub enum GlowLook {
    /// No glow.
    None,
    /// Working out of sight: a soft ring.
    Working,
    /// Acting in the window, brightest at the spot where it acts.
    Acting {
        /// Where it acts, when it knows.
        spot: Option<GlowSpot>,
    },
    /// Waiting for the person: still.
    Waiting,
}

/// The values the glow is drawn with. Holds floats because it is built from [`Px`].
#[derive(Debug, Clone, Copy, PartialEq)]
pub struct GlowSpec {
    /// The three colours it fades between.
    pub colours: [Srgb; 3],
    /// How wide the ring is.
    pub width: Px,
    /// How soft its edge is.
    pub blur: Px,
    /// How long one pulse takes; none holds it still (zero frames).
    pub period: Option<Duration>,
    /// How strong it is.
    pub strength: Fraction,
}

/// The glow for `look` in `scheme`, from the orb's tokens. `None` and `Waiting` have no period.
pub fn glow_spec(_look: &GlowLook, _scheme: Scheme) -> GlowSpec {
    todo!(
        "glow_spec: values per design/32 section 3; glow_spec_idle_and_waiting_have_no_period pins the still looks"
    )
}
