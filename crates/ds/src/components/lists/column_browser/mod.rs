//! ColumnBrowser: Finder's column view (Miller columns), a row of lists where the selection in
//! one decides the next.

pub(crate) mod model;
pub(crate) mod view;

mod column;
mod item;

pub use item::{BrowserColumn, BrowserRow, BrowserRowKind, ColumnPick};
pub use view::ColumnBrowser;
