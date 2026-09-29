//! The user's picture (design/25-EMOJI.md section 7): a letter disc, an animated emoji or their
//! own photo ([`UserPicture`]), drawn by [`UserPortrait`] or by the lock and polkit prompts; the
//! stored choice ([`PictureChoice`]) and the rule that resolves it ([`resolve_picture`]); and
//! [`UserPicturePicker`], where a person picks one.

pub(crate) mod accept;
pub(crate) mod choice;
pub mod mood;
pub(crate) mod picker;
pub(crate) mod picture;
pub(crate) mod portrait;

pub use crate::motion::wake::WakeStamp;
pub use choice::{FaceFile, PictureChoice, resolve_picture};
pub use mood::{Mood, PictureSize};
pub use picker::{PICTURE_CELL, PICTURE_COLUMNS, UserPicturePicker};
pub use picture::UserPicture;
pub use portrait::UserPortrait;
