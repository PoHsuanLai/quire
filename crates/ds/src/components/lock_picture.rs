//! The person's picture in the lock and polkit prompts (design/04-COMPONENTS.md section 42):
//! a face redrawn at the prompt's size, a photo cropped round at it, or an animated emoji playing
//! the prompt's mood. Every kind plays the accept beat when the mood turns Happy.

use crate::components::avatar::AvatarSize;
use crate::components::user_picture::{Drawn, FaceAt, Liveliness, PictureSize, Sizes, UserPicture};
use dioxus::prelude::*;

/// The lock screen's: 64 for all three.
pub(crate) const AT_LOCK: Sizes = Sizes {
    face: FaceAt::Size(AvatarSize::Size64),
    photo: 64,
    emoji: PictureSize::Medium,
};

/// The polkit sheet's: 48. The emoji is drawn at `Medium` and `polkit_prompt.css` sets its box
/// to 48 (it is drawn in shares of its box, so it scales whole).
pub(crate) const AT_POLKIT: Sizes = Sizes {
    face: FaceAt::Size(AvatarSize::Size48),
    photo: 48,
    emoji: PictureSize::Medium,
};

/// `picture` at `sizes`, playing `life`.
pub(crate) fn prompt_picture(picture: UserPicture, sizes: Sizes, life: Liveliness) -> Element {
    rsx! {
        Drawn { picture, sizes, life }
    }
}
