//! What the caller tells a moving picture (design/25-EMOJI.md section 5): its size, its mood,
//! and when to wake. The picture plays the moods; it never chooses one.

use crate::core::word::Word;

/// How large a picture is drawn (`data-size`).
#[derive(Debug, Clone, Copy, PartialEq, Eq, Hash, Word)]
pub enum PictureSize {
    /// 28 px: beside a name in a list or a menu.
    #[word(slug = "28")]
    Small,
    /// 64 px: a settings row, a switcher, the lock screen's prompt.
    #[word(slug = "64")]
    Medium,
    /// 128 px: a login screen's large picture.
    #[word(slug = "128")]
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
}

/// What the picture is doing (`data-mood`). The caller sets it; each change wakes the picture
/// and plays that mood's reaction once.
#[derive(Debug, Clone, Copy, PartialEq, Eq, Hash, Default, Word)]
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
