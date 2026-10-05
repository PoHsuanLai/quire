//! Pure input machines that both the shell and the compositor run, so a behaviour has one
//! definition whichever process sees the input (12-restructure §2.1 W4): the app switcher
//! (design/13 §13.3.5), hot corners (design/13 §13.3.12), the Command key's double tap and hold
//! (the companion's summon and hold-to-talk), and the live Space swipe (design/12 §12.3.7).
//!
//! Plain data and integer maths over `ds-core`'s [`Stamp`](ds_core::time::stamp::Stamp): no
//! Dioxus, no renderer, no clock. Each machine is a `ds_core::machine::Machine`: it takes the time
//! as an argument, keeps its deadlines in its state and says when it wants to be woken
//! (`Machine::wake`), and returns what it wants done; the caller owns the state, the one timer
//! (`ds::machine::use_machine` or its own driver), the keyboard grab and the drawing.

mod dir;
pub mod hot_corner;
pub mod modifier_tap;
pub mod space_swipe;
mod span;
pub mod switcher;

pub use dir::Dir;
