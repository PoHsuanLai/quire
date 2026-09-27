//! The token table: every colour, duration, delay, easing, scalar, radius, spacing step,
//! shadow, type size, layer and device-pixel line width, as Rust data. The stylesheet is
//! generated from it (`crate::css`), so CSS and the Rust timers cannot drift
//! (design/05-MOTION.md section 7.1).

pub mod accent_band;
pub mod accent_table;
pub mod colour;
pub mod control_center;
pub mod control_size;
pub mod delay;
pub mod dock;
pub mod easing;
pub mod elevation;
pub mod emoji_face;
pub mod hex;
pub mod label_hue;
pub mod layer;
pub mod name;
pub mod notifications;
pub mod opacity;
pub mod osd;
pub mod person;
pub mod pixel;
pub mod scalar;
pub mod shape;
pub mod shell;
pub mod shell_scale;
pub mod size_scale;
pub mod size_vars;
pub mod spacing;
pub mod timing;
pub mod tuned;
pub mod type_scale;
pub mod type_voice;
pub mod widget_paint;
pub mod widgets;

#[cfg(test)]
mod size_rules_tests;

pub use accent_band::AccentRoles;
pub use accent_table::accent_of;
pub use colour::ColourToken;
pub use control_center::{CONTROL_CENTER, ControlCenterScale};
pub use control_size::ControlSize;
pub use delay::DelayToken;
pub use dock::{DockFloorSetting, DockMetrics};
pub use easing::{CubicBezier, Easing, EasingToken};
pub use elevation::Shadow;
pub use emoji_face::{FONT_EMOJI, FONT_EMOJI_STACK};
pub use hex::{Alpha, Colour, Hex};
pub use label_hue::{HueMember, LabelHue};
pub use layer::ZLayer;
pub use name::VarName;
pub use notifications::NotificationMetrics;
pub use opacity::OpacityToken;
pub use osd::OsdMetrics;
pub use person::PersonSwatch;
pub use pixel::PixelToken;
pub use scalar::{ScalarToken, ScalarValue};
pub use shape::{Corner, Radius};
pub use shell::{BarType, FontWeight, LauncherType, MenuType, ShellMetrics};
pub use shell_scale::{SHELL_SCALE, ShellScale};
pub use size_scale::{HalfPx, KNOB_INSET, SPACING_GRID, SizeScale, WholePx, on_grid};
pub use size_vars::SizeVar;
pub use spacing::SpacingToken;
pub use timing::{DurationKind, DurationToken};
pub use tuned::Tuned;
pub use type_scale::{Family, FontSize, Voiced};
pub use type_voice::VoiceToken;
pub use widget_paint::WidgetPaint;
pub use widgets::WidgetMetrics;
