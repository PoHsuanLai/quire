//! The Emoji page (design/25-EMOJI.md): the whole shipped set at Large, the reactions a mood
//! swaps in, one pick in every mood, the three sizes, and the eight discs. Every picture here
//! is `EmojiPlayback::Still` (a sheet of 42 loops is what a picker must not do), so each shows
//! its rest frame; the motion is proved by `ds-native/tests/emoji_life.rs`.

use super::{Caption, Section};
use dioxus::prelude::*;
use ds::{
    AnimatedEmoji, Backdrop, EMOJI_ATTRIBUTION, EmojiDisc, EmojiId, EmojiPlayback, Mood,
    PersonaSize,
};

const STILL: EmojiPlayback = EmojiPlayback::Still;

const BACKDROPS: [Backdrop; 8] = [
    Backdrop::Clay,
    Backdrop::Ochre,
    Backdrop::Sage,
    Backdrop::Jade,
    Backdrop::Teal,
    Backdrop::Slate,
    Backdrop::Plum,
    Backdrop::Rose,
];

/// The reactions, with what shows them.
const REACTIONS: [(EmojiId, &str); 4] = [
    (EmojiId::WRONG, "Wince: a wrong password, once through"),
    (EmojiId::UNLOCKED, "Happy: unlocked, once through"),
    (EmojiId::ASLEEP, "Asleep: the display is off, still"),
    (
        EmojiId::ATTENTIVE,
        "Attentive keeps the pick; this is in the set",
    ),
];

/// The page.
#[component]
pub fn EmojiPage() -> Element {
    rsx! {
        Section { title: "The set", note: "EmojiId::ALL at Large, EmojiPlayback::Still, each at its rest frame: 42 Noto Animated Emoji the user can pick for their picture. The default is Blush.",
            div { class: "g-emoji-grid",
                for emoji in EmojiId::ALL {
                    div { key: "{emoji.slug()}", class: "g-col g-emoji-cell",
                        AnimatedEmoji { emoji, size: PersonaSize::Large, playback: STILL }
                        Caption { name: emoji.slug().to_string() }
                    }
                }
            }
        }
        Section { title: "Reactions", note: "What a mood swaps in for a moment; the user's own emoji comes back after one loop.",
            div { class: "g-row g-emoji-reactions",
                for (emoji, note) in REACTIONS {
                    div { key: "{emoji.slug()}", class: "g-col g-emoji-cell",
                        AnimatedEmoji { emoji, size: PersonaSize::Large, playback: STILL }
                        Caption { name: emoji.slug().to_string(), code: note.to_string() }
                    }
                }
            }
        }
        Section { title: "Moods at rest", note: "Wink in Idle, Attentive, Wince, Happy and Asleep, as the lock screen would leave each once its window has closed: the pick, except Asleep's sleeping face.",
            div { class: "g-row g-emoji-reactions",
                for mood in Mood::ALL {
                    div { key: "{mood.slug()}", class: "g-col g-emoji-cell",
                        AnimatedEmoji { emoji: EmojiId::Wink, size: PersonaSize::Large, mood, playback: STILL }
                        Caption { name: mood.slug().to_string() }
                    }
                }
            }
        }
        Section { title: "Sizes and discs", note: "Small 28, Medium 64 and Large 128, bare; then Medium on each of the icon palette's eight hues (EmojiDisc::Tinted).",
            div { class: "g-col",
                div { class: "g-row g-emoji-sizes",
                    AnimatedEmoji { emoji: EmojiId::HeartEyes, size: PersonaSize::Small, playback: STILL }
                    AnimatedEmoji { emoji: EmojiId::HeartEyes, size: PersonaSize::Medium, playback: STILL }
                    AnimatedEmoji { emoji: EmojiId::HeartEyes, size: PersonaSize::Large, playback: STILL }
                }
                div { class: "g-row g-emoji-sizes",
                    for (index, backdrop) in BACKDROPS.into_iter().enumerate() {
                        AnimatedEmoji { key: "{index}", emoji: EmojiId::ALL[index * 5], size: PersonaSize::Medium, disc: EmojiDisc::Tinted(backdrop), playback: STILL }
                    }
                }
            }
        }
        Section { title: "Credit", note: EMOJI_ATTRIBUTION.to_string(),
            div {}
        }
    }
}
