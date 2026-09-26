//! Animated emoji: the user's picture as an emoji they pick, moving (design/25-EMOJI.md).
//! The frames are Noto Animated Emoji (CC BY 4.0, `assets/emoji/ATTRIBUTION.txt`),
//! pre-rendered into sprite sheets by `tools/emoji`, because the renderer draws one frame of an
//! image and plays no Lottie; quire plays them by moving the sheet's `background-position`
//! from a Rust timer, only inside the 20 s awake window, as the persona blinks.

mod disc;
mod id;
mod life;
mod script;
mod sheet;
#[cfg(test)]
mod tests;

pub use disc::{EmojiDisc, EmojiPlayback};
pub use id::EmojiId;

use crate::components::persona::{Mood, PersonaSize, WakeStamp};
use crate::root::env::use_env;
use dioxus::prelude::*;
use sheet::{SheetPx, position, timing, uri};

/// The attribution a surface that shows these emoji carries in its credits (design/25
/// section 2).
pub const EMOJI_ATTRIBUTION: &str = "Animated emoji: Noto Animated Emoji by Google, \
    CC BY 4.0 (https://creativecommons.org/licenses/by/4.0/); frames resampled and packed.";

/// The user's `emoji` at `size`, playing `mood` (design/25-EMOJI.md section 5). It plays its
/// loop for 20 s after mounting, after each new `wake` stamp, each mood change and each new
/// pick, then rests on its first frame and paints nothing. Wince shows the wrong-password
/// emoji once through, Happy the unlock emoji, then the user's own again; Asleep shows the
/// sleeping face, still. Under Reduced motion, or with `playback: EmojiPlayback::Still`,
/// only still frames are shown. Decorative, as the
/// persona: the name beside it is what a screen reader reads.
#[component]
pub fn AnimatedEmoji(
    emoji: EmojiId,
    size: PersonaSize,
    #[props(default)] mood: Mood,
    #[props(default)] wake: WakeStamp,
    #[props(default)] disc: EmojiDisc,
    #[props(default)] playback: EmojiPlayback,
) -> Element {
    let scheme = use_env().scheme;
    let shown = life::use_frames(emoji, mood, wake, playback);
    let px = SheetPx::for_size(size);
    let (at, fit) = position(timing(shown.emoji), shown.frame);
    let sheet = format!("url(\"{}\")", uri(shown.emoji, px));
    let style = match disc {
        EmojiDisc::Tinted(backdrop) => format!("--em-disc:{}", disc::tint(backdrop, scheme)),
        EmojiDisc::None => String::new(),
    };
    rsx! {
        div {
            class: "ds-emoji",
            "data-size": size_slug(size),
            "data-mood": mood.slug(),
            "data-disc": disc.slug(),
            "data-playback": playback.slug(),
            "aria-hidden": "true",
            style,
            div {
                class: "ds-emoji-face",
                "data-emoji": shown.emoji.slug(),
                "data-frame": "{shown.frame}",
                background_image: sheet,
                background_size: fit,
                background_position: at,
            }
        }
    }
}

fn size_slug(size: PersonaSize) -> &'static str {
    match size {
        PersonaSize::Small => "28",
        PersonaSize::Medium => "64",
        PersonaSize::Large => "128",
    }
}
