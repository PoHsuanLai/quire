//! What the caller tells a moving picture (design/25-EMOJI.md section 5): its size, its mood,
//! and when to wake. The picture plays the moods; it never chooses one.

/// How large a picture is drawn (`data-size`).
#[derive(Debug, Clone, Copy, PartialEq, Eq, Hash)]
pub enum PictureSize {
    /// 28 px: beside a name in a list or a menu.
    Small,
    /// 64 px: a settings row, a switcher, the lock screen's prompt.
    Medium,
    /// 128 px: a login screen's large picture.
    Large,
}

impl PictureSize {
    /// The side in logical pixels.
    pub fn px(self) -> u16 {
        match self {
            PictureSize::Small => 28,
            PictureSize::Medium => 64,
            PictureSize::Large => 128,
        }
    }

    /// The `data-size` value.
    pub(crate) fn slug(self) -> &'static str {
        match self {
            PictureSize::Small => "28",
            PictureSize::Medium => "64",
            PictureSize::Large => "128",
        }
    }
}

/// What the picture is doing (`data-mood`). The caller sets it; each change wakes the picture
/// and plays that mood's reaction once.
#[derive(Debug, Clone, Copy, PartialEq, Eq, Hash, Default)]
pub enum Mood {
    /// At rest: an emoji plays its loop now and then for 20 s after a wake, then holds still.
    #[default]
    Idle,
    /// Watching the field below it (the lock screen, while the user types): an emoji glances
    /// with the eyes once, then plays its own loop steadily.
    Attentive,
    /// A wrong password: an emoji shows the confounded face once through.
    Wince,
    /// Unlocked: the picture's accept beat (`Anim::PictureAccept`), and an emoji shows the
    /// partying face once through.
    Happy,
    /// The display is off: an emoji shows the sleeping face, still.
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
}

/// A wake counter, shared with the other components that replay an entrance (the battery
/// ring's fill); it lives in [`crate::motion`] and is re-exported here for picture callers.
pub use crate::motion::WakeStamp;
