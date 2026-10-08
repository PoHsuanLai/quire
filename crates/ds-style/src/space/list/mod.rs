//! The list of an app's Spaces: Arc-style tinted contexts (Work, Home...) as pure data. Generic
//! over the app's payload `P` (what a Space holds that is the app's own: mail scope, a working
//! directory) and over `R`, the place an app was left at in a Space.
//!
//! Persistence is `ds_settings::SpacesStorage`; the hook and the views are `ds::components::app::spaces`.
//! design/21-SPACES.md section 13.

mod first_run;
mod model;
mod ops;
mod recall;
mod remove;
mod switch;
mod time;
mod today;
mod today_ops;
mod wire;

pub use model::{Space, SpaceId, Spaces};
pub use recall::Recall;
pub use remove::{Refused, Removed};
pub use switch::{SlideIn, Switched};
pub use time::{Epoch, IDLE};
pub use today::{Entry, NoPark, Parked, Today};
pub use wire::MOST_DOTS;
pub(crate) use wire::clamp_look;

#[cfg(test)]
mod tests;
