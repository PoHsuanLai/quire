//! A bounded pending loop's look as data (design/26-DETAILS.md section 3.2, R4): how it moves and
//! what it draws in a frame. The frame at a time is [`crate::motion::timeline::pending::Pending`].

/// How a pending loop moves.
#[derive(Debug, Clone, Copy, PartialEq, Eq, Hash)]
pub enum PendingStyle {
    /// One layer at a time: the Wi-Fi bars searching.
    Iterate,
    /// Layers fill in turn and stay, then clear: a level being found.
    Cumulate,
    /// A disc's opacity between .45 and 1, one step per half.
    Breathe,
    /// A ring's dash turns a quarter per step.
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
    /// Over how many layers (1 for a ring or a disc).
    pub layers: PendingLayers,
}

impl PendingSpec {
    /// Steps in one cycle: a layer each for Iterate, each fill and the clear for Cumulate, two
    /// halves for Breathe, four quarters for Spin.
    pub fn cycle(self) -> u8 {
        let layers = self.layers.0.max(1);
        match self.style {
            PendingStyle::Iterate => layers,
            PendingStyle::Cumulate => layers.saturating_add(1),
            PendingStyle::Breathe => 2,
            PendingStyle::Spin => 4,
        }
    }
}

/// What to draw.
#[derive(Debug, Clone, Copy, PartialEq, Eq, Hash)]
pub enum PendingFrame {
    /// No operation, or one younger than `PendingGrace`: draw the state as it is.
    Idle,
    /// The loop's `n`th step since it showed (0 first). [`PendingFrame::lit`] reads a layer.
    Step(u8),
    /// Past the deadline, or under Reduced: the still frame (the glyph dimmed); 0 frames.
    Stalled,
}

/// Whether a layer is drawn in a frame.
#[derive(Debug, Clone, Copy, PartialEq, Eq, Hash)]
pub enum Lit {
    /// Drawn.
    On,
    /// Not drawn (or, for Breathe, at its low half).
    Off,
}

impl PendingFrame {
    /// The `data-pending` word: `idle`, `step` or `still`.
    pub fn slug(self) -> &'static str {
        match self {
            PendingFrame::Idle => "idle",
            PendingFrame::Step(_) => "step",
            PendingFrame::Stalled => "still",
        }
    }

    /// Whether `layer` (0 first) is drawn in this frame of `spec`: Idle and Stalled draw every
    /// layer; Iterate lights one layer a step; Cumulate lights layers up to the step's and then
    /// none; Breathe is On on its high half; Spin is always On (it turns instead).
    pub fn lit(self, spec: PendingSpec, layer: u8) -> Lit {
        let PendingFrame::Step(n) = self else {
            return Lit::On;
        };
        let phase = n % spec.cycle();
        let on = match spec.style {
            PendingStyle::Iterate => layer == phase,
            PendingStyle::Cumulate => layer <= phase && phase < spec.layers.0,
            PendingStyle::Breathe => phase == 0,
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
        let wifi = |style| PendingSpec {
            style,
            layers: PendingLayers(4),
        };
        let lit =
            |frame: PendingFrame, spec| (0..4).map(|l| frame.lit(spec, l)).collect::<Vec<_>>();
        use Lit::{Off, On};
        assert_eq!(
            lit(PendingFrame::Step(2), wifi(PendingStyle::Iterate)),
            [Off, Off, On, Off]
        );
        assert_eq!(
            lit(PendingFrame::Step(5), wifi(PendingStyle::Iterate)),
            [Off, On, Off, Off]
        );
        assert_eq!(
            lit(PendingFrame::Step(1), wifi(PendingStyle::Cumulate)),
            [On, On, Off, Off]
        );
        assert_eq!(
            lit(PendingFrame::Step(4), wifi(PendingStyle::Cumulate)),
            [Off, Off, Off, Off]
        );
        assert_eq!(
            lit(PendingFrame::Stalled, wifi(PendingStyle::Iterate)),
            [On, On, On, On]
        );
        assert_eq!(
            lit(PendingFrame::Idle, wifi(PendingStyle::Cumulate)),
            [On, On, On, On]
        );
        let disc = PendingSpec {
            style: PendingStyle::Breathe,
            layers: PendingLayers(1),
        };
        assert_eq!(PendingFrame::Step(0).lit(disc, 0), On);
        assert_eq!(PendingFrame::Step(1).lit(disc, 0), Off);
    }
}
