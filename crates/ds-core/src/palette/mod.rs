//! The palette's pure machine: a query, a selection over rows someone else ranked, and what
//! Enter runs. It names rows by index, so a launcher, a command palette, a new-tab bar and a
//! viewer share one over their own row types. It lives here, below the renderer, so a crate of pure machines can hold it;
//! `ds::components::menus::palette::machine` re-exports it and
//! [`CommandPalette`](https://docs.rs/ds) draws it.

pub mod model;
mod step;
#[cfg(test)]
mod tests;
