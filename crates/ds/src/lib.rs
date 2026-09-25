//! quire's renderer-free design system: tokens, palette, icons, components and motion.
//!
//! Every module is one concept; this file declares them and re-exports the flat public surface.
//! The linter stays a module (`ds::lint`), behind the `lint` feature. DESIGN.md maps each
//! module to the design doc section it implements.

pub mod appearance;
pub mod components;
pub mod css;
pub mod edit;
pub mod error;
pub mod focus;
pub mod fonts;
pub mod geometry;
mod guarded;
pub mod icon;
#[cfg(feature = "lint")]
pub mod lint;
pub mod material;
pub mod motion;
pub mod overlay;
pub mod root;
pub mod space;
mod task;
pub mod text;
pub mod time;
pub mod tokens;

pub use appearance::{
    Accent, Appearance, Contrast, Look, Motion, MotionLevel, PeekMode, ReducedMotion, Resolved,
    Scheme, SystemPrefs, Theme, Warmth, resolve,
};
pub use components::*;
pub use css::stylesheet;
pub use edit::{
    Clicks, Composition, EDIT_KIND_ATTR, EDIT_NODE_ATTR, EditFocus, EditHandle, EditInput,
    EditKind, EditNode, EditPointer, Extend, HostEdit, ImeEvent, ImeListener, ImeSwitch, KeyInput,
    Pasted, PointerPhase, PreeditCursor, Probe, TextOffset, TextPosition, TextRange,
    use_edit_handle,
};
pub use error::DsError;
pub use focus::{
    FocusRequest, FocusTicket, Focused, HostFocus, HostSelect, Select, focus_soon,
    focus_soon_selecting, use_focus_request,
};
#[cfg(feature = "webview-fonts")]
pub use fonts::font_face_css;
pub use fonts::{FACES, Face, FaceStyle, Subset, Weight};
pub use geometry::{
    Align, Anchor, Flip, Grid, HostMeasure, Measured, MountedRef, Placed, Placement, Point,
    PopoverRequest, Px, Rect, RectProbe, Scale, Side, Size, place, use_rect,
};
pub use icon::render::{Glyph, IconPx, IconSize};
pub use icon::{
    ChromaLimit, ExternalIcon, Icon, IconKind, IconSource, IconUrl, PlateFamily, PlateTint, Shape,
};
pub use material::{Blur, BlurState, Material, MaterialRecipe, MaterialStack, recipe};
pub use motion::{
    Anim, Drag, DragPhase, DragTracker, Exit, Fill, HoverEvent, HoverIntent, IntentEffect,
    IntentPhase, Iteration, ListPresence, MotionTimer, Presence, Pulse, Recipe, Roster,
    RosterEntry, RosterState, RowPitch, StayError, Stayed, TimerPhase, settle, use_drag,
    use_motion_timer, use_pulse, use_roster,
};
pub use overlay::{
    Dismissal, HoverHub, HoverKey, HoverKind, HoverWarmth, ItemPath, LayerId, LayerStack, MenuAnim,
    MenuDirection, MenuKey, MenuPhase, MenuTarget, MenuTiming, MenuTrack, MenuTrackEffect,
    MenuTrackEvent, OverlayHost, OverlayId, Overlays, ToastHub, ToastState, UndoToken,
    use_hover_hub, use_overlays, use_toast_hub,
};
pub use root::{
    Ds, Env, FrameTint, Ground, HostModality, HostScale, Inject, InputModality, RootChrome,
    Surface, use_env, use_scale,
};
pub use space::{
    Capping, Card, CardAccent, ContrastCheck, Dot, FrameVars, Grain, NEUTRAL_DOT, POST_DARK,
    POST_LIGHT, PRESETS, Palette, Preset, SpaceDefaults, SpaceLook, SpaceStore, Verdict, Workspace,
    WorkspaceId, WorkspaceIndex, card, default_look, derive, gradient, ratio, readout, swatch,
};
pub use text::clip_chars;
pub use time::{FRAME_SLACK, sleep};
pub use tokens::{
    AccentQuad, Alpha, BarType, Colour, ColourToken, Corner, CubicBezier, DelayToken,
    DockFloorSetting, DockMetrics, DurationKind, DurationToken, Easing, EasingToken, Family,
    FontSize, FontWeight, Hex, HueMember, LabelHue, LauncherType, MenuType, OpacityToken,
    PersonSwatch, PixelToken, Radius, ScalarToken, ScalarValue, Shadow, ShellMetrics, SpacingToken,
    Tuned, VarName, ZLayer, quad,
};

use futures_timer as _;
use serde_json as _;
use thiserror as _;

#[cfg(feature = "lint")]
use cssparser as _;
