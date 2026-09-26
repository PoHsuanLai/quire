//! A user's picture, either kind (design/24-PERSONA.md section 5): the letter disc the lock
//! prompt shows today, or the user's persona. A surface takes a [`UserPicture`] and draws it
//! with [`UserPortrait`], so switching a person to their persona changes data, not markup.

use super::Persona;
use super::mood::{Mood, PersonaSize, WakeStamp};
use super::spec::PersonaSpec;
use crate::components::avatar::{AvatarFace, face};
use dioxus::prelude::*;

/// What stands for a user: a letter disc or a persona.
#[derive(Debug, Clone, Copy, PartialEq, Eq, Hash)]
pub enum UserPicture {
    /// A coloured disc with one letter, at its own size.
    Face(AvatarFace),
    /// The user's persona.
    Persona(PersonaSpec),
}

/// A [`UserPicture`], drawn: a face as `Avatar` draws it (it has no moods; `size`, `mood` and
/// `wake` are the persona's), a persona as [`Persona`] at `size` playing `mood`.
#[component]
pub fn UserPortrait(
    picture: UserPicture,
    size: PersonaSize,
    #[props(default)] mood: Mood,
    #[props(default)] wake: WakeStamp,
) -> Element {
    match picture {
        UserPicture::Face(avatar) => face(avatar),
        UserPicture::Persona(spec) => rsx! { Persona { spec, size, mood, wake } },
    }
}
