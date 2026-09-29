//! A pending loop's look as data (design/26-DETAILS.md section 3.2, R4; design/30 section 1.3):
//! how it moves and what it draws in a frame. The frame at a time is
//! [`crate::motion::timeline::pending::Pending`].

/// How a pending loop moves.
#[derive(Debug, Clone, Copy, PartialEq, Eq, Hash)]
pub enum PendingStyle {
    /// One layer at a time: the Wi-Fi bars searching.
    Iterate,
    /// A ring turns a twelfth per step: the spinner.
    Spin,
}

/// How many layers a style steps through (the Wi-Fi glyph: the dot and three arcs = 4).
#[derive(Debug, Clone, Copy, PartialEq, Eq, Hash)]
pub struct PendingLayers(pub u8);

/// A pending loop's look.
#[derive(Debug, Clone, Copy, PartialEq, Eq, Hash)]
pub struct PendingSpec {
    /// How it moves.
    pub style: PendingStyle,
    /// Over how many layers (1 for a ring).
    pub layers: PendingLayers,
}

/// The steps of one turn of the spinner: twelve spokes.
pub const SPIN_STEPS: u8 = 12;

impl PendingSpec {
    /// Steps in one cycle: a layer each for Iterate, twelve twelfths for Spin.
    pub fn cycle(self) -> u8 {
        match self.style {
            PendingStyle::Iterate => self.layers.0.max(1),
            PendingStyle::Spin => SPIN_STEPS,
        }
    }
}

/// What to draw.
#[derive(Debug, Clone, Copy, PartialEq, Eq, Hash)]
pub enum PendingFrame {
    /// No operation: draw the state as it is.
    Idle,
    /// The loop's `n`th step since it started, counted modulo [`SPIN_STEPS`] (0 first).
    /// [`PendingFrame::lit`] reads a layer.
    Step(u8),
}

/// Whether a layer is drawn in a frame.
#[derive(Debug, Clone, Copy, PartialEq, Eq, Hash)]
pub enum Lit {
    /// Drawn.
    On,
    /// Not drawn.
    Off,
}

impl PendingFrame {
    /// The `data-pending` word: `idle` or `step`.
    pub fn slug(self) -> &'static str {
        match self {
            PendingFrame::Idle => "idle",
            PendingFrame::Step(_) => "step",
        }
    }

    /// Whether `layer` (0 first) is drawn in this frame of `spec`: Idle draws every layer;
    /// Iterate lights one layer a step; Spin is always On (it turns instead).
    pub fn lit(self, spec: PendingSpec, layer: u8) -> Lit {
        let PendingFrame::Step(n) = self else {
            return Lit::On;
        };
        let on = match spec.style {
            PendingStyle::Iterate => layer == n % spec.cycle(),
            PendingStyle::Spin => true,
        };
        if on { Lit::On } else { Lit::Off }
    }
}

#[cfg(test)]
mod tests {
    use super::{Lit, PendingFrame, PendingLayers, PendingSpec, PendingStyle};

    #[test]
    fn each_style_lights_its_own_layers() {
        let wifi = PendingSpec {
            style: PendingStyle::Iterate,
            layers: PendingLayers(4),
        };
        let lit =
            |frame: PendingFrame, spec| (0..4).map(|l| frame.lit(spec, l)).collect::<Vec<_>>();
        use Lit::{Off, On};
        assert_eq!(lit(PendingFrame::Step(2), wifi), [Off, Off, On, Off]);
        assert_eq!(lit(PendingFrame::Step(5), wifi), [Off, On, Off, Off]);
        assert_eq!(lit(PendingFrame::Idle, wifi), [On, On, On, On]);
        let ring = PendingSpec {
            style: PendingStyle::Spin,
            layers: PendingLayers(1),
        };
        assert_eq!(PendingFrame::Step(7).lit(ring, 0), On);
    }
}
