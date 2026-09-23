//! quire's renderer-free design system: tokens, palette, icons, components and motion.
//!
//! Every module is one concept; this file declares them and re-exports the flat public surface.
//! The linter stays a module (`ds::lint`), behind the `lint` feature. DESIGN.md maps each
//! module to the design doc section it implements.

pub mod appearance;
pub mod components;
pub mod css;
pub mod fonts;
pub mod geometry;
pub mod icon;
#[cfg(feature = "lint")]
pub mod lint;
pub mod material;
pub mod motion;
pub mod overlay;
pub mod root;
pub mod space;
pub mod text;
pub mod time;
pub mod tokens;

pub use appearance::{
    Accent, Appearance, Contrast, Look, Motion, MotionLevel, PeekMode, ReducedMotion, Resolved,
    Scheme, SystemPrefs, Theme, Warmth, resolve,
};
pub use components::*;
pub use css::stylesheet;
#[cfg(feature = "webview-fonts")]
pub use fonts::font_face_css;
pub use fonts::{FACES, Face, FaceStyle, Subset, Weight};
pub use geometry::{
    Align, Anchor, Flip, HostMeasure, Measured, MountedRef, Placed, Placement, Point,
    PopoverRequest, Px, Rect, RectProbe, Side, Size, place, use_rect,
};
pub use icon::render::{Glyph, IconSize};
pub use icon::{Icon, Shape};
pub use material::{Blur, BlurState, Material, MaterialRecipe, recipe};
pub use motion::{
    Anim, Drag, DragPhase, DragTracker, Exit, Fill, HoverEvent, HoverIntent, IntentEffect,
    IntentPhase, Iteration, ListPresence, MotionTimer, Presence, Pulse, Recipe, Roster,
    RosterEntry, RosterState, RowPitch, TimerPhase, settle, use_drag, use_motion_timer, use_pulse,
    use_roster,
};
pub use overlay::{
    Dismissal, HoverHub, HoverKey, HoverKind, HoverWarmth, LayerId, LayerStack, OverlayHost,
    OverlayId, Overlays, ToastHub, ToastState, UndoToken, use_hover_hub, use_overlays,
    use_toast_hub,
};
pub use root::{Ds, Env, HostModality, Inject, InputModality, Surface, use_env};
pub use space::{
    Capping, Card, CardAccent, ContrastCheck, Dot, FrameVars, Grain, NEUTRAL_DOT, POST_DARK,
    POST_LIGHT, PRESETS, Palette, Preset, SpaceLook, Verdict, card, default_look, derive, gradient,
    ratio, readout, swatch,
};
pub use text::clip_chars;
pub use time::{FRAME_SLACK, sleep};
pub use tokens::{
    AccentQuad, Alpha, Colour, ColourToken, CubicBezier, DelayToken, DurationKind, DurationToken,
    Easing, EasingToken, Family, FontSize, Hex, HueMember, LabelHue, Radius, ScalarToken,
    ScalarValue, Shadow, VarName, ZLayer, quad,
};

use futures_timer as _;
use serde_json as _;
use thiserror as _;

#[cfg(feature = "lint")]
use cssparser as _;
