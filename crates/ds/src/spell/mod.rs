//! Spellchecking an [`EditSurface`](crate::components::fields::edit_surface::EditSurface) (design/04-COMPONENTS.md section 50):
//! the prop that turns it on, the pure rules (which text is a word, which marks survive an
//! edit, which word the caret is typing), and the host seam a checker plugs into.

pub(crate) mod host;
pub(crate) mod lang;
pub(crate) mod marks;
pub(crate) mod script;
pub(crate) mod words;
