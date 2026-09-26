//! The person's picture in the lock and polkit prompts (design/04-COMPONENTS.md section 42):
//! a face redrawn at the prompt's size, a photo cropped round at it, or the persona playing the
//! prompt's mood. Faces and photos have no moods.

use crate::components::avatar::{AvatarFace, AvatarSize, face};
use crate::components::persona::{Mood, Persona, PersonaSize, UserPicture, WakeStamp, photo};
use dioxus::prelude::*;

/// How large a prompt draws its person, per kind of picture.
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub(crate) struct PromptSize {
    face: AvatarSize,
    photo: u16,
    persona: PersonaSize,
}

/// The lock screen's: 64 for all three.
pub(crate) const AT_LOCK: PromptSize = PromptSize {
    face: AvatarSize::Size64,
    photo: 64,
    persona: PersonaSize::Medium,
};

/// The polkit sheet's: 48. The persona is drawn at `Medium` and `polkit_prompt.css` sets its
/// box to 48 (it is drawn in shares of its box, so it scales whole).
pub(crate) const AT_POLKIT: PromptSize = PromptSize {
    face: AvatarSize::Size48,
    photo: 48,
    persona: PersonaSize::Medium,
};

/// What the persona is told: its mood and its wake stamp.
#[derive(Debug, Clone, Copy, PartialEq, Eq, Default)]
pub(crate) struct Liveliness {
    pub(crate) mood: Mood,
    pub(crate) wake: WakeStamp,
}

/// `picture` at `size`; a persona plays `life`.
pub(crate) fn prompt_picture(picture: UserPicture, size: PromptSize, life: Liveliness) -> Element {
    match picture {
        UserPicture::Face(avatar) => face(AvatarFace {
            size: size.face,
            ..avatar
        }),
        UserPicture::Photo(source) => photo(source, size.photo),
        UserPicture::Persona(spec) => rsx! {
            Persona { spec, size: size.persona, mood: life.mood, wake: life.wake }
        },
    }
}
