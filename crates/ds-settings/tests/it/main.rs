//! The crate's integration tests: one executable, one module per former test file.
//! A separate test target needs a stated reason (CONVENTIONS.md, Tests).
//! Separate target: `live` (feature `live`) runs the settings service on a private D-Bus.

mod appearance_file;
mod derive;
mod keys;
mod rows;
mod schema;
mod spaces_file;
mod store;
mod support;
mod user_style;
mod watch;
