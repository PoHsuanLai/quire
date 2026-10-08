//! The palette's pure machine: a query, a selection over rows someone else ranked, and what
//! Enter runs. Generic over the row type, so a launcher, a command palette and a new-tab bar
//! share one. [`CommandPalette`](super::command_palette::CommandPalette) draws; this decides.

pub mod model;
mod step;
#[cfg(test)]
mod tests;
