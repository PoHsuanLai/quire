//! Spaces: Arc-style tinted contexts every app shares (design/21-SPACES.md section 13).
//!
//! The model is pure (`ds_style::space::list`, re-exported here); persistence is
//! `ds_settings::SpacesStorage`. This module is what sits between: the controller hooks
//! ([`use_spaces`], [`use_today`]), the switch chord, and the views that draw a Space (the
//! sidebar head and foot, the Space's menu, the Today section). The app supplies its payload,
//! where it is (`here`), and how to show a place again (`arrived`).

pub(crate) mod chord;
pub(crate) mod controller;
pub(crate) mod delete;
pub(crate) mod foot;
pub(crate) mod head;
pub(crate) mod menu_link;
pub(crate) mod menu_pick;
pub(crate) mod menu_view;
pub(crate) mod new_space;
pub(crate) mod open_menu;
pub(crate) mod parts;
pub(crate) mod source;
pub(crate) mod today_section;
pub(crate) mod today_state;

pub use chord::SwitchChord;
pub use controller::{SpacesHandle, use_spaces};
pub use ds_style::space::list::{
    Entry, Epoch, IDLE, NoPark, Parked, Recall, Refused, Removed, SlideIn, Space, SpaceId, Spaces,
    Switched, Today,
};
pub use foot::SpacesFoot;
pub use head::SpaceHead;
pub use menu_link::{DesktopSpace, LinkChange, Linking};
pub use menu_pick::SpacePick;
pub use menu_view::SpaceMenu;
pub use new_space::NewSpace;
pub use open_menu::{OpenMenu, Showing};
pub use source::SpacesSource;
pub use today_section::{TodayRow, TodaySection};
pub use today_state::{TodayHandle, use_today};

/// The least width of a sidebar that holds the Space foot: four 28 px tiles and their gaps, as
/// design/30 section 2.11 sizes the sidebar. The app gives its `Sidebar` at least this.
pub const SIDEBAR_MIN_WIDTH: f32 = 212.0;

#[cfg(test)]
mod tests;
