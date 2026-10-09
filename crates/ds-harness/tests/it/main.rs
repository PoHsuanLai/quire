//! The crate's integration tests: one executable, one module per former test file.
//! A separate test target needs a stated reason (CONVENTIONS.md, Tests).
//! Separate target: `menu_export` (feature `menus`) starts a private D-Bus and owns the names on it.

mod centre_padded;
mod forwarded_context_menu;
mod guards;
mod hybrid_backend;
mod inset;
mod render_loop;
mod resize;
mod settle;
mod snapshot;
mod texture_layer;
mod virtual_clock;
mod virtual_clock_asks;
mod window_sizer;
