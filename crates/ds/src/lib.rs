//! quire's renderer-free design system: tokens, palette, icons, components and motion.
//!
//! Every module is one concept; this file declares them and re-exports the flat public surface.
//! The linter stays a module (`ds::lint`), behind the `lint` feature. DESIGN.md maps each
//! module to the design doc section it implements.

pub mod appearance;
pub(crate) mod busy;
pub mod catalog;
pub mod components;
pub(crate) mod core;
pub mod css;
pub mod detail;
pub mod edit;
pub mod error;
pub mod file_drop;
pub mod focus;
pub mod fonts;
pub mod geometry;
pub(crate) mod guarded;
pub mod icon;
#[cfg(feature = "lint")]
pub mod lint;
pub mod material;
pub mod motion;
pub mod overlay;
pub mod root;
pub mod space;
pub mod spell;
pub(crate) mod task;
pub mod text;
pub mod time;
pub mod tokens;
pub mod widget;
pub mod window;

pub use crate::appearance::{
    accent::Accent,
    appearance::Appearance,
    look::{Look, Warmth},
    motion::{Motion, MotionLevel},
    peek::PeekMode,
    resolve::{Resolved, resolve},
    system::{Contrast, ReducedMotion, SystemPrefs},
    theme::{Scheme, Theme},
    typeface::Typeface,
};
pub use crate::appearance::{
    blur::{Blur, BlurState},
    material::Material,
};
pub use crate::core::colour::contrast::{Verdict, ratio};
pub use crate::css::stylesheet::stylesheet;
pub use crate::edit::{
    clicks::Clicks,
    handle::{EditHandle, use_edit_handle},
    host::{HostEdit, ImeEvent, ImeListener, ImeSwitch, Probe},
    input::{Composition, EditInput, KeyInput, Pasted, PreeditCursor},
    pointer::{CapturedPointer, EditFocus, EditPointer, Extend, PointerPhase},
    position::{
        EDIT_KIND_ATTR, EDIT_NODE_ATTR, EditKind, EditNode, TextOffset, TextPosition, TextRange,
    },
};
pub use crate::file_drop::{
    drag::{DropAcceptance, DropHit, FileDrag, FileDragInput, FileDrop, Offer},
    hook::{FileDropHandle, use_file_drop},
    host::HostFileDrop,
};
pub use crate::focus::{
    caret::{
        Caret, Collapsed, FieldSelection, HostCaret, HostPlaceCaret, HostSelection, InitialCaret,
        caret_at,
    },
    click::{Fallback, HostClickFocus, HostPressFocus},
    field::{FieldHandle, use_field_handle},
    hand_back::HostHandBack,
    host::{Focused, HostBlur, HostFocus, focus_soon, focus_soon_selecting},
    request::{FocusRequest, FocusTicket, use_focus_request},
    select::{HostSelect, Select},
    selector::{FocusError, Found, HostFind, focus_by_selector},
};
pub use crate::geometry::{
    measure::{Anchor, HostMeasure, Measured, MountedRef, RectProbe, use_rect},
    placement::{Align, Flip, Placed, Placement, PopoverRequest, Side, place},
    reveal::{HostReveal, ScrollSpan, Scrolled, nearest_scroll},
    scale::{Grid, Scale},
    units::{Point, Px, Rect, Size},
};
pub use crate::icon::{
    classify::{ChromaLimit, IconKind},
    external::{ExternalIcon, IconSource, IconUrl},
    family::PlateFamily,
    plate_tint::PlateTint,
    shape::Shape,
};
pub use crate::material::{
    recipe::{MaterialRecipe, recipe},
    stack::MaterialStack,
};
pub use crate::motion::hover_intent::HoverWarmth;
pub use crate::motion::{
    anim::Anim,
    drag::{DRAG_THRESHOLD, Drag, DragPhase, DragTracker, use_drag},
    entrance::use_entrance,
    hover_intent::{HoverEvent, HoverIntent, IntentEffect, IntentPhase},
    pane_slide::{Pane, PaneRole, PaneRound, PaneSlide},
    presence::{Exit, ListPresence, Presence},
    pulse::{Pulse, use_pulse},
    recipe::{Fill, Iteration, Recipe},
    roster::{RosterEntry, RosterState, RowPitch, StayError, Stayed},
    settle::settle,
    timer::{MotionTimer, TimerPhase, use_motion_timer},
    use_roster::{Roster, use_roster},
};
pub use crate::motion::{
    level_run::{LevelRun, RunFrame, RunPhase, RunTail, RunTiming, RunTokens},
    use_level_run::use_level_run,
};
pub use crate::motion::{
    swipe::{Click, Speed, Stamp, SwipeEffect, SwipeInput, SwipeLook, SwipeMetrics, SwipeState},
    use_swipe::{Held, Swiper, use_swipe},
};
pub use crate::overlay::{
    host::{OverlayHost, OverlayId, Overlays, use_overlays},
    hover_hub::{HoverHub, HoverKey, HoverKind, use_hover_hub},
    menu_track::types::{
        ItemPath, MenuAnim, MenuDirection, MenuKey, MenuPhase, MenuTarget, MenuTiming, MenuTrack,
        MenuTrackEffect, MenuTrackEvent,
    },
    stack::{Dismissal, LayerId, LayerStack},
    toast_hub::{ToastHub, ToastState, UndoToken, use_toast_hub},
};
pub use crate::root::{
    chrome::{FrameTint, Ground, RootChrome},
    ds::{Ds, Inject},
    env::{Env, HostModality, InputModality, use_env},
    extent::RootExtent,
    scale::{HostScale, use_scale},
    surface::Surface,
    typeface::use_typeface,
};
pub use crate::space::{
    frame_vars::FrameVars,
    look::{CardAccent, Grain, SpaceLook},
    palette::{
        Capping, Dot, NEUTRAL_DOT, Palette,
        card::{Card, POST_DARK, POST_LIGHT, card},
        derive, gradient,
        readout::{ContrastCheck, readout},
        swatch,
    },
    presets::{PRESETS, Preset, default_look},
    store::{SpaceDefaults, SpaceStore, Workspace, WorkspaceId, WorkspaceIndex},
};
pub use crate::spell::{
    host::{HostSpell, Learned, Paragraph, SpellFuture, SpellService},
    lang::{Lang, Spell},
    marks::{Misspelt, SpellReplace, Typing},
    script::is_cjk,
    words::Span,
};
pub use crate::text::clip::clip_chars;
pub use crate::time::clock::{ClockGuard, VirtualClock, sleep};
pub use crate::tokens::{
    accent_band::roles::AccentRoles,
    accent_table::accent_of,
    colour::ColourToken,
    control_center::{CONTROL_CENTER, ControlCenterScale},
    control_size::ControlSize,
    delay::DelayToken,
    dock::{DockFloorSetting, DockMetrics},
    easing::{CubicBezier, Easing, EasingToken},
    elevation::Shadow,
    hex::{Alpha, Colour, Hex},
    label_hue::{HueMember, LabelHue},
    layer::ZLayer,
    name::VarName,
    notifications::NotificationMetrics,
    opacity::OpacityToken,
    osd::OsdMetrics,
    person::PersonSwatch,
    pixel::PixelToken,
    scalar::{ScalarToken, ScalarValue},
    shape::{Corner, Radius},
    shell::{BarType, FontWeight, LauncherType, MenuType, ShellMetrics},
    shell_scale::{SHELL_SCALE, ShellScale},
    size_scale::{HalfPx, SizeScale, WholePx},
    size_vars::SizeVar,
    spacing::SpacingToken,
    timing::{DurationKind, DurationToken},
    tuned::Tuned,
    type_scale::{Family, FontSize, Voiced},
    type_voice::VoiceToken,
    widgets::WidgetMetrics,
};
pub use crate::widget::{
    battery::{BatteryCell, BatteryEntry, BatteryWidget},
    calendar::{EventLine, MonthEntry, MonthFace, MonthIntent, MonthWidget, TodayLine},
    card::WidgetCard,
    clock::{ClockCity, ClockEntry, WorldClockWidget},
    contract::{NoIntent, Widget, WidgetContext, WidgetKind},
    gallery::{GalleryWords, WidgetGallery},
    layout::{DesktopGrid, GridCell, WidgetAt, WidgetEdit, WidgetLayout},
    registry::{WidgetInfo, WidgetRegistry, provide_widget_registry, use_widget_registry},
    timeline::{Dated, EntryDate, Refresh, RefreshAsk, Timeline},
    use_widget::use_widget,
    wire::WireTimeline,
};
pub use crate::window::{
    host::{HostWindow, WindowHost, use_window_host, use_window_host_provider, use_window_state},
    timing::FrameTiming,
    vocab::{
        Activation, Fullscreen, Maximized, ResizeEdge, Support, TileError, WindowState, WindowTile,
        Zoom,
    },
};
pub use components::*;
pub use error::DsError;
pub use fonts::{FACES, Face, FaceStyle, Subset, Weight};
pub use icon::Icon;
pub use icon::render::{Glyph, IconPx, IconSize};
pub use time::{FRAME_SLACK, FRAME_TICK};

use futures_timer as _;
use serde_json as _;
use thiserror as _;

#[cfg(feature = "lint")]
use cssparser as _;
