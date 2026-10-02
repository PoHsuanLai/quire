//! The orb's data: its size ladder, the period a moving orb turns at, its tone, and the look a
//! presence resolves to.

use ds_core::geometry::units::Px;
use ds_core::vocab::Activity;
use ds_core::word::Word;
use ds_style::tokens::timing::DurationToken;

/// The orb's size ladder, `data-size` (proposed: design/32 section 3, the user reviews the pixel
/// values).
#[derive(Debug, Clone, Copy, PartialEq, Eq, Hash, Default, Word)]
pub enum OrbSize {
    /// Inside a line of text, 14 px.
    Inline,
    /// The menu bar item, 16 px.
    Bar,
    /// The leading mark of a prompt field, 20 px.
    #[default]
    Field,
    /// A control-centre module, 40 px.
    Module,
    /// A hero, 192 px, the voice orb's own size.
    Hero,
}

impl OrbSize {
    /// How wide the orb is drawn at this rung.
    pub fn px(self) -> Px {
        match self {
            OrbSize::Inline => Px(14.0),
            OrbSize::Bar => Px(16.0),
            OrbSize::Field => Px(20.0),
            OrbSize::Module => Px(40.0),
            OrbSize::Hero => Px(192.0),
        }
    }
}

/// The period a moving orb turns at: one duration token each.
#[derive(Debug, Clone, Copy, PartialEq, Eq, Hash, Word)]
pub enum OrbPeriod {
    /// Listening: the slowest turn.
    Listen,
    /// Working out of sight.
    Work,
    /// Acting in a window: the quickest turn.
    Act,
}

impl OrbPeriod {
    /// The duration token that times one turn.
    pub fn token(self) -> DurationToken {
        match self {
            OrbPeriod::Listen => DurationToken::OrbListen,
            OrbPeriod::Work => DurationToken::OrbWork,
            OrbPeriod::Act => DurationToken::OrbAct,
        }
    }
}

/// How bright the orb looks, `data-tone` (proposed: the user's design review).
#[derive(Debug, Clone, Copy, PartialEq, Eq, Hash, Default, Word)]
pub enum OrbTone {
    /// The resting look.
    #[default]
    Calm,
    /// Taking part: listening, working, acting.
    Bright,
    /// Held still while it waits for the person.
    Held,
}

/// What a presence makes of the voice orb: whether it turns, how fast, and the tone.
#[derive(Debug, Clone, Copy, PartialEq, Eq, Hash)]
pub struct OrbLook {
    /// Whether the orb turns at all.
    pub activity: Activity,
    /// The period of the turn; none while it is inactive.
    pub period: Option<OrbPeriod>,
    /// The tone the stylesheet reads.
    pub tone: OrbTone,
}
