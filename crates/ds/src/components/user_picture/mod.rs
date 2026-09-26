//! The user's picture (design/25-EMOJI.md section 7): a letter disc, an animated emoji or their
//! own photo ([`UserPicture`]), drawn by [`UserPortrait`] or by the lock and polkit prompts; the
//! stored choice ([`PictureChoice`]) and the rule that resolves it ([`resolve_picture`]); and
//! [`UserPicturePicker`], where a person picks one. The persona this replaced was dropped on
//! 2026-09-26 (design/24-PERSONA.md).

mod accept;
mod choice;
pub mod mood;
mod picker;
mod picture;
mod portrait;

pub use choice::{FaceFile, PictureChoice, resolve_picture};
pub use mood::{Mood, PictureSize, WakeStamp};
pub use picker::{PICTURE_CELL, PICTURE_COLUMNS, UserPicturePicker};
pub use picture::UserPicture;
pub use portrait::UserPortrait;
pub(crate) use portrait::{Drawn, FaceAt, Liveliness, Sizes};
