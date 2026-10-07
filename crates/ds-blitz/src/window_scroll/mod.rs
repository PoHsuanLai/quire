//! One window's scrolling (design/11): the shared engine (`blitz_kit::scroll`) driving the
//! window's own Blitz document, so a wheel detent is 60 px eased over at most 200 ms, a touchpad
//! flick glides and stretches, and the arrow, page and Home/End keys scroll by the same steps as
//! in a shell surface. What is host-specific is here: winit's wheel and keys as engine inputs,
//! the window's frame requests, and the gestures `ds::host::gesture` listeners hear.
//!
//! The window loop gives every winit event to [`WindowScroll::event`] before the document: a
//! wheel the engine takes never reaches Blitz (whose own scroll jumps by 20 px a line), except
//! over a `data-wheel="capture"` element, which gets the raw wheel (design/11 §11.3.1).

mod bounds;
mod coast;
mod eased;
mod events;
mod handle;
mod keys;
mod state;
pub mod wheel;

pub use bounds::ScrollBounds;
pub use events::Intercept;
pub use handle::{ScrollHandle, use_scroll_handle};
pub use state::{Frames, WheelUse, WindowScroll};
