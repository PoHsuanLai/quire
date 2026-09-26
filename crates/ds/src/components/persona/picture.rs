//! A user's picture, any kind (design/24-PERSONA.md section 5): the letter disc, their own
//! photo (the freedesktop `~/.face` or AccountsService's icon), or their persona. A surface
//! takes a [`UserPicture`] and draws it with [`UserPortrait`], so switching a person from one
//! kind to another changes data, not markup.

use super::Persona;
use super::mood::{Mood, PersonaSize, WakeStamp};
use super::spec::PersonaSpec;
use crate::components::avatar::{AvatarFace, face};
use crate::components::image_source::ImageSource;
use dioxus::prelude::*;

/// What stands for a user: a letter disc, a photo or a persona.
#[derive(Debug, Clone, PartialEq, Eq, Hash)]
pub enum UserPicture {
    /// A coloured disc with one letter, at its own size.
    Face(AvatarFace),
    /// The user's own picture, cropped round to fill the disc (`object-fit: cover`), at the
    /// size a face would have there. Pass `$HOME/.face` or AccountsService's icon as
    /// `ImageSource::file(path)`.
    Photo(ImageSource),
    /// The user's persona.
    Persona(PersonaSpec),
}

impl From<AvatarFace> for UserPicture {
    fn from(face: AvatarFace) -> Self {
        UserPicture::Face(face)
    }
}

impl From<PersonaSpec> for UserPicture {
    fn from(spec: PersonaSpec) -> Self {
        UserPicture::Persona(spec)
    }
}

impl From<ImageSource> for UserPicture {
    fn from(source: ImageSource) -> Self {
        UserPicture::Photo(source)
    }
}

/// A [`UserPicture`], drawn: a face as `Avatar` draws it (at its own size), a photo round at
/// `size`, a persona as [`Persona`] at `size` playing `mood`. Faces and photos have no moods:
/// `mood` and `wake` are the persona's alone.
#[component]
pub fn UserPortrait(
    picture: UserPicture,
    size: PersonaSize,
    #[props(default)] mood: Mood,
    #[props(default)] wake: WakeStamp,
) -> Element {
    match picture {
        UserPicture::Face(avatar) => face(avatar),
        UserPicture::Photo(source) => photo(source, size.px()),
        UserPicture::Persona(spec) => rsx! { Persona { spec, size, mood, wake } },
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
