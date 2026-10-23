//! The desktop-entry layer every repo shares: where entries live, the walk of an `applications`
//! folder, the desktop id of a file, the match of a window's app id to an entry, and the per-app
//! identity record an entry declares.
//!
//! One answer to each question, so the dock, Mission Control and the launcher cannot disagree:
//!
//! - [`XdgEnv::applications_dirs`]: the folders to search, most important first;
//! - [`walk_applications`] and [`first_of_each_id`]: the files in them and their [`DesktopId`]s
//!   (`kde/foo.desktop` is `kde-foo.desktop`, as the specification says);
//! - [`load_entries`] and [`entry_for_window`]: the parsed entries and the one a window belongs to,
//!   by the five rules of [`match_app`];
//! - [`AppIdentity`]: the names one app is known by, read from `X-Quire-` keys of its entry.
//!
//! The matching, the id and the identity are pure. The walk and the loading read files, and only
//! the folders they are given; [`XdgEnv::from_process`] is the one place that reads the
//! environment.

mod data_dirs;
mod desktop_id;
mod entry;
mod identity;
mod match_app;
mod walk;

pub use data_dirs::XdgEnv;
pub use desktop_id::{DesktopId, DesktopIdError};
pub use entry::{ApplicationEntry, entry_for_window, load_entries};
pub use identity::AppIdentity;
pub use match_app::{AppKeys, match_app};
pub use walk::{EntryFile, first_of_each_id, walk_applications};
