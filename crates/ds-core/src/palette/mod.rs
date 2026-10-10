//! The palette's pure machine: a query, a selection over rows someone else ranked, and what
//! Enter runs. Generic over the row type, so a launcher, a command palette, a new-tab bar and a
//! viewer share one. It lives here, below the renderer, so a crate of pure machines can hold it;
//! `ds::components::menus::palette::machine` re-exports it and
//! [`CommandPalette`](https://docs.rs/ds) draws it.

pub mod model;
mod step;
#[cfg(test)]
mod tests;
