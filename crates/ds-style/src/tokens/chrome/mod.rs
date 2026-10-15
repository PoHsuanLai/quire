//! Window chrome: the geometry of a client-decorated window's titlebar, lights and resize edges,
//! and the state the lights are in.

pub mod light;
pub mod light_hue;
pub mod light_paint;
pub mod scale;
pub mod title_font;
pub mod token;

pub use light::{GroupHover, LightFill, LightState, light_state};
pub use light_hue::{Light, light_hue, light_rim};
pub use light_paint::{LightColours, LightPaint, LightPart, light_colours};
pub use scale::{CHROME_SCALE, ChromeScale};
pub use title_font::{TitleFont, title_font};
pub use token::ChromeToken;
