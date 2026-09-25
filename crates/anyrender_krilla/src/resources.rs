//! What a painter keeps across the pages of one PDF, so a face used on ten pages is parsed once
//! and an image drawn on two pages is one object in the file.

use crate::fonts::Faces;
use krilla::image::Image;
use std::collections::HashMap;
use std::fmt;

/// Parsed faces and prepared images for one document; hand the same value to every page's
/// [`KrillaScene`](crate::KrillaScene).
#[derive(Default)]
pub struct Resources {
    pub(crate) faces: Faces,
    /// Images by the id of the decoded blob they were drawn from.
    pub(crate) images: HashMap<u64, Image>,
}

impl Resources {
    /// Nothing loaded yet.
    pub fn new() -> Self {
        Resources::default()
    }

    /// How many faces (a variable face counts once per instance) the pages drew with.
    pub fn face_count(&self) -> usize {
        self.faces.len()
    }

    /// How many distinct images the pages drew.
    pub fn image_count(&self) -> usize {
        self.images.len()
    }
}

impl fmt::Debug for Resources {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        f.debug_struct("Resources")
            .field("faces", &self.face_count())
            .field("images", &self.image_count())
            .finish()
    }
}
