//! Window chrome: the geometry of a client-decorated window's titlebar, lights and resize edges,
//! and the state the lights are in.

pub mod light;
pub mod scale;
pub mod token;

pub use light::{GroupHover, LightFill, LightState, light_state};
pub use scale::{CHROME_SCALE, ChromeScale};
pub use token::ChromeToken;
