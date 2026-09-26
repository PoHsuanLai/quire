//! What the caller tells a persona (design/24-PERSONA.md section 4): its size, its mood, and
//! when to wake. The component plays the moods; it never chooses one.

/// How large a persona is drawn (`data-size`).
#[derive(Debug, Clone, Copy, PartialEq, Eq, Hash)]
pub enum PersonaSize {
    /// 28 px: beside a name in a list or a menu. Drops the finest details.
    Small,
    /// 64 px: a settings row, a switcher.
    Medium,
    /// 128 px: the lock and login screen.
    Large,
}

impl PersonaSize {
    /// The side in logical pixels.
    pub fn px(self) -> u16 {
        match self {
            PersonaSize::Small => 28,
            PersonaSize::Medium => 64,
            PersonaSize::Large => 128,
        }
    }

    /// The `data-size` value.
    pub(crate) fn slug(self) -> &'static str {
        match self {
            PersonaSize::Small => "28",
            PersonaSize::Medium => "64",
            PersonaSize::Large => "128",
        }
    }

    /// Whether whiskers, freckles, brows and eye flecks are drawn: not at 28 px, where they
    /// blur into the face.
    pub(crate) fn detail(self) -> Detail {
        match self {
            PersonaSize::Small => Detail::Coarse,
            PersonaSize::Medium | PersonaSize::Large => Detail::Fine,
        }
    }
}

/// How much of the face is drawn at a size.
#[derive(Debug, Clone, Copy, PartialEq, Eq, Hash)]
pub(crate) enum Detail {
    /// Everything.
    Fine,
    /// The silhouette, eyes, mouth and cheeks only.
    Coarse,
}

/// What the persona is doing (`data-mood`). The caller sets it; each change plays its motion
/// once and wakes the persona.
#[derive(Debug, Clone, Copy, PartialEq, Eq, Hash, Default)]
pub enum Mood {
    /// At rest: blinks now and then and breathes, for 20 s after a wake, then holds still.
    #[default]
    Idle,
    /// Watching the field below it: the eyes look down (the lock screen, while the user types).
    Attentive,
    /// A wrong password: a squint and one shake of the head.
    Wince,
    /// Unlocked: a smile and one small hop.
    Happy,
    /// The display is off: closed eyes, head tilted, one `z` drifts up and fades.
    Asleep,
}

impl Mood {
    /// Every mood, in the order the gallery shows them.
    pub const ALL: [Mood; 5] = [
        Mood::Idle,
        Mood::Attentive,
        Mood::Wince,
        Mood::Happy,
        Mood::Asleep,
    ];

    /// The `data-mood` value.
    pub fn slug(self) -> &'static str {
        match self {
            Mood::Idle => "idle",
            Mood::Attentive => "attentive",
            Mood::Wince => "wince",
            Mood::Happy => "happy",
            Mood::Asleep => "asleep",
        }
    }

    /// Whether the eyes are open in this mood, so a blink has something to close.
    pub(crate) fn blinks(self) -> Blinking {
        match self {
            Mood::Idle | Mood::Attentive => Blinking::Blinks,
            Mood::Wince | Mood::Happy | Mood::Asleep => Blinking::Still,
        }
    }
}

/// Whether a mood blinks while awake.
#[derive(Debug, Clone, Copy, PartialEq, Eq, Hash)]
pub(crate) enum Blinking {
    /// Open eyes: blink now and then.
    Blinks,
    /// Closed or squinting eyes: nothing to blink.
    Still,
}

/// A wake counter, shared with the other components that replay an entrance (the battery
/// ring's fill); it lives in [`crate::motion`] and is re-exported here for persona callers.
pub use crate::motion::WakeStamp;

/// How much colour a persona carries: in full, or muted as the icons' Muted style mutes them
/// (design/08 section 2.11; chroma x .55, lightness and hue kept). A surface that follows
/// `icons.style` passes `Muted` when the icons are muted.
#[derive(Debug, Clone, Copy, PartialEq, Eq, Hash, Default)]
pub enum PersonaFinish {
    /// The palette as derived: bright, colour-carried.
    #[default]
    Colour,
    /// Every colour at .55 of its chroma.
    Muted,
}
