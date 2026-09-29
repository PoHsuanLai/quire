//! Drawing a [`UserPicture`]: one wrapper (`div.ds-user-picture`) around a face, a photo or an
//! animated emoji. [`UserPortrait`] is the public form; the lock and polkit prompts draw through
//! [`Drawn`] at their own sizes.

use super::mood::{Mood, PictureSize};
use super::picture::UserPicture;
use crate::components::content::avatar::{AvatarFace, AvatarSize, face};
use crate::components::content::image_source::ImageSource;
use crate::core::word::Word;
use crate::motion::wake::WakeStamp;
use crate::shell::emoji::AnimatedEmoji;
use dioxus::prelude::*;

/// The size a letter disc is drawn at.
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub(crate) enum FaceAt {
    /// The size the face carries.
    Own,
    /// This size, whatever the face carries.
    Size(AvatarSize),
}

/// How large each kind of picture is drawn at one place.
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub(crate) struct Sizes {
    pub(crate) face: FaceAt,
    pub(crate) photo: u16,
    pub(crate) emoji: PictureSize,
}

impl Sizes {
    /// A face at its own size; a photo and an emoji at `size`.
    pub(crate) fn of(size: PictureSize) -> Sizes {
        Sizes {
            face: FaceAt::Own,
            photo: size.px(),
            emoji: size,
        }
    }
}

/// What a moving picture is told: its mood and its wake stamp.
#[derive(Debug, Clone, Copy, PartialEq, Eq, Default)]
pub(crate) struct Liveliness {
    pub(crate) mood: Mood,
    pub(crate) wake: WakeStamp,
}

/// `picture` at `sizes`, in its wrapper.
#[component]
pub(crate) fn Drawn(picture: UserPicture, sizes: Sizes, life: Liveliness) -> Element {
    rsx! {
        div { class: "ds-user-picture", "data-mood": life.mood.slug(),
            {body(picture, sizes, life)}
        }
    }
}

fn body(picture: UserPicture, sizes: Sizes, life: Liveliness) -> Element {
    match picture {
        UserPicture::Face(avatar) => match sizes.face {
            FaceAt::Own => face(avatar),
            FaceAt::Size(size) => face(AvatarFace { size, ..avatar }),
        },
        UserPicture::Photo(source) => photo(source, sizes.photo),
        UserPicture::Emoji(emoji) => rsx! {
            AnimatedEmoji { emoji, size: sizes.emoji, mood: life.mood, wake: life.wake }
        },
    }
}

/// A [`UserPicture`], drawn: a face as `Avatar` draws it (at its own size), a photo round at
/// `size`, an emoji as [`AnimatedEmoji`] at `size` playing `mood` (design/25-EMOJI.md section
/// 5). Every kind plays the accept beat when `mood` turns Happy; only an emoji plays the other
/// moods, and only an emoji is woken by `wake`.
#[component]
pub fn UserPortrait(
    picture: UserPicture,
    size: PictureSize,
    #[props(default)] mood: Mood,
    #[props(default)] wake: WakeStamp,
) -> Element {
    rsx! {
        Drawn { picture, sizes: Sizes::of(size), life: Liveliness { mood, wake } }
    }
}

/// A photo `px` across, cropped round: `div.ds-user-photo[data-size]` clipping an
/// `img.ds-user-photo-image` that covers it. Decorative: the name beside it is what a screen
/// reader reads.
pub(crate) fn photo(source: ImageSource, px: u16) -> Element {
    rsx! {
        div { class: "ds-user-photo", "data-size": "{px}", "aria-hidden": "true",
            img { class: "ds-user-photo-image", alt: "", src: source.0, draggable: "false" }
        }
    }
}
