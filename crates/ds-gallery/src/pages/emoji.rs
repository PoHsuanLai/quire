//! The Emoji page (design/25-EMOJI.md): the whole shipped set at Large, the reactions a mood
//! swaps in, one pick in every mood, the three sizes, the eight discs, and the user picture's
//! picker. Every picture here
//! is `EmojiPlayback::Still` (a sheet of 42 loops is what a picker must not do), so each shows
//! its rest frame; the motion is proved by `ds-native/tests/emoji_life.rs`.

use super::{Caption, Section};
use dioxus::prelude::*;
use ds::{
    AnimatedEmoji, AvatarFace, AvatarShape, AvatarSize, AvatarTone, DiscHue, EMOJI_ATTRIBUTION,
    EmojiDisc, EmojiId, EmojiPlayback, Mood, PictureChoice, PictureSize, UserPicturePicker,
    person_hue,
};

const STILL: EmojiPlayback = EmojiPlayback::Still;

const HUES: [DiscHue; 8] = [
    DiscHue::Clay,
    DiscHue::Ochre,
    DiscHue::Sage,
    DiscHue::Jade,
    DiscHue::Teal,
    DiscHue::Slate,
    DiscHue::Plum,
    DiscHue::Rose,
];

/// The reactions, with what shows them.
const REACTIONS: [(EmojiId, &str); 4] = [
    (EmojiId::WRONG, "Wince: a wrong password, once through"),
    (EmojiId::UNLOCKED, "Happy: unlocked, once through"),
    (EmojiId::ASLEEP, "Asleep: the display is off, still"),
    (
        EmojiId::ATTENTIVE,
        "Attentive: a glance once, then the pick",
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
                        AnimatedEmoji { emoji, size: PictureSize::Large, playback: STILL }
                        Caption { name: emoji.slug().to_string() }
                    }
                }
            }
        }
        Section { title: "Reactions", note: "What a mood swaps in for a moment; the user's own emoji comes back after one loop.",
            div { class: "g-row g-emoji-reactions",
                for (emoji, note) in REACTIONS {
                    div { key: "{emoji.slug()}", class: "g-col g-emoji-cell",
                        AnimatedEmoji { emoji, size: PictureSize::Large, playback: STILL }
                        Caption { name: emoji.slug().to_string(), code: note.to_string() }
                    }
                }
            }
        }
        Section { title: "Moods at rest", note: "Wink in Idle, Attentive, Wince, Happy and Asleep, as the lock screen would leave each once its window has closed: the pick, except Asleep's sleeping face.",
            div { class: "g-row g-emoji-reactions",
                for mood in Mood::ALL {
                    div { key: "{mood.slug()}", class: "g-col g-emoji-cell",
                        AnimatedEmoji { emoji: EmojiId::Wink, size: PictureSize::Large, mood, playback: STILL }
                        Caption { name: mood.slug().to_string() }
                    }
                }
            }
        }
        Section { title: "Sizes and discs", note: "Small 28, Medium 64 and Large 128, bare; then Medium on each of the icon palette's eight hues (EmojiDisc::Tinted).",
            div { class: "g-col",
                div { class: "g-row g-emoji-sizes",
                    AnimatedEmoji { emoji: EmojiId::HeartEyes, size: PictureSize::Small, playback: STILL }
                    AnimatedEmoji { emoji: EmojiId::HeartEyes, size: PictureSize::Medium, playback: STILL }
                    AnimatedEmoji { emoji: EmojiId::HeartEyes, size: PictureSize::Large, playback: STILL }
                }
                div { class: "g-row g-emoji-sizes",
                    for (index, hue) in HUES.into_iter().enumerate() {
                        AnimatedEmoji { key: "{index}", emoji: EmojiId::ALL[index * 5], size: PictureSize::Medium, disc: EmojiDisc::Tinted(hue), playback: STILL }
                    }
                }
            }
        }
        Section { title: "Picker", note: "UserPicturePicker: the letter, then the 42 as still frames at 64, one radio group with the current choice marked (here the fox). Click a disc, or use the arrows, to choose; Settings' Users page and first run show it.",
            PickerStage {}
        }
        Section { title: "Credit", note: EMOJI_ATTRIBUTION.to_string(),
            div {}
        }
    }
}

/// The picker, holding its own choice so the gallery can be clicked through.
#[component]
fn PickerStage() -> Element {
    let mut choice = use_signal(|| PictureChoice::Emoji(EmojiId::Fox));
    let letter = AvatarFace {
        initial: 'P',
        size: AvatarSize::Size64,
        tone: AvatarTone::Person(person_hue("pohsuan")),
        shape: AvatarShape::Round,
    };
    rsx! {
        UserPicturePicker { letter, choice: choice(), onpick: move |next| choice.set(next) }
    }
}
