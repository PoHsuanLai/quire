//! The optional disc behind an animated emoji, and whether it plays. The disc: none, or a tinted circle on one of the icon
//! palette's eight hues (design/08 section 2.10), pale on light and deep on dark as the
//! persona's disc is.

use crate::appearance::Scheme;
use crate::components::persona::Backdrop;
use crate::space::palette::oklch_hex;
use serde::{Deserialize, Serialize};

/// What sits behind the emoji (`data-disc`). User data beside the [`super::EmojiId`].
#[derive(Debug, Clone, Copy, PartialEq, Eq, Hash, Default, Serialize, Deserialize)]
#[serde(rename_all = "snake_case")]
pub enum EmojiDisc {
    /// The emoji alone, at the full size of its box.
    #[default]
    None,
    /// A tinted circle, the emoji inset on it.
    Tinted(Backdrop),
}

impl EmojiDisc {
    /// The `data-disc` value.
    pub(crate) fn slug(self) -> &'static str {
        match self {
            EmojiDisc::None => "none",
            EmojiDisc::Tinted(_) => "tinted",
        }
    }
}

fn hue(backdrop: Backdrop) -> f64 {
    match backdrop {
        Backdrop::Clay => 40.0,
        Backdrop::Ochre => 85.0,
        Backdrop::Sage => 130.0,
        Backdrop::Jade => 175.0,
        Backdrop::Teal => 220.0,
        Backdrop::Slate => 265.0,
        Backdrop::Plum => 310.0,
        Backdrop::Rose => 355.0,
    }
}

/// The disc's colour as `#rrggbb` in `scheme`.
pub(crate) fn tint(backdrop: Backdrop, scheme: Scheme) -> String {
    match scheme {
        Scheme::Light => oklch_hex(0.91, 0.055, hue(backdrop)),
        Scheme::Dark => oklch_hex(0.40, 0.06, hue(backdrop)),
    }
}

/// Whether a picture plays at all (`data-playback`). A picker showing the whole set passes
/// `Still`: 42 loops at once is motion nobody asked for.
#[derive(Debug, Clone, Copy, PartialEq, Eq, Hash, Default)]
pub enum EmojiPlayback {
    /// Plays inside the awake window after each wake.
    #[default]
    Awake,
    /// Rest frames only, as under Reduced motion; a mood's reaction is still shown, still.
    Still,
}

impl EmojiPlayback {
    /// The `data-playback` value.
    pub(crate) fn slug(self) -> &'static str {
        match self {
            EmojiPlayback::Awake => "awake",
            EmojiPlayback::Still => "still",
        }
    }
}
