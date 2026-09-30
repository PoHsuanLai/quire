//! Shell surfaces' parts: the lock and polkit prompts, the app switcher, the bar and dock pieces,
//! the OSD, control-center modules, notifications, thumbnails, now playing, the idle dim, the Space
//! editor, the user's picture, animated emoji, the month grid, clocks, batteries, and the desktop
//! widgets with their catalog, over `ds` (the generic components and the document seam), and the
//! shell's own tokens and sheets, which `kits()` and `stylesheet()` add to the design system's.

pub(crate) mod bar;
pub mod battery;
pub mod catalog;
pub(crate) mod clock;
pub(crate) mod control_center;
pub(crate) mod dock_parts;
pub mod emoji;
pub(crate) mod idle_dim;
pub(crate) mod kit;
pub(crate) mod lock;
pub(crate) mod month_grid;
pub(crate) mod notifications;
pub(crate) mod now_playing;
pub(crate) mod osd;
pub(crate) mod sheets;
pub(crate) mod space_editor;
#[cfg(test)]
mod stored_words;
pub(crate) mod switcher;
pub(crate) mod thumbs;
pub(crate) mod tokens;
pub(crate) mod user_picture;
#[cfg(test)]
mod vocabulary_tests;
pub mod widget;

pub use crate::{
    bar::{
        menu_bar_item::MenuBarItem,
        workspace_pills::{WorkspacePill, WorkspacePills},
    },
    battery::{
        device_battery::DeviceBattery,
        device_glyph::{Device, DeviceGlyph},
        figure::BatteryFigure,
        level::{BatteryLevel, RingMark},
    },
    clock::{
        face::ClockFace,
        kind::{ClockLook, ClockTime, DayPhase, Seconds},
    },
    control_center::{
        module_grid::{GridColumns, ModuleGrid},
        module_panel::{ModulePanel, PanelPlate},
        module_tile::ModuleTile,
        module_tile_kind::{Chevron, ModuleState, TileSpan},
    },
    dock_parts::{DockFloor, DockLabel, RunningDot},
    emoji::{
        AnimatedEmoji, EMOJI_ATTRIBUTION,
        disc::{DiscHue, EmojiDisc, EmojiPlayback},
        id::EmojiId,
    },
    idle_dim::{IdleDim, IdleDimPhase},
    lock::{
        clock::LockClock,
        polkit_prompt::PolkitPrompt,
        prompt::LockPrompt,
        screen::LockScreen,
        vocab::{CapsLock, LockLook, LockUser, PromptState},
    },
    month_grid::{
        MonthGrid,
        data::{
            DayKey, DayMark, DayPlace, Eventful, IsoWeek, MonthDay, MonthGridData, MonthKey,
            MonthStep, MonthWeek, WeekNumbers,
        },
        density::MonthDensity,
    },
    notifications::{
        banner_stack::{Banner, BannerEntry, BannerKey, BannerPosition, BannerStack},
        card::NotificationCard,
        group_header::GroupHeader,
        parts::{AppMark, CardAction, GroupCount, Hover, StackLayers},
        swipe::NotificationSwipe,
    },
    now_playing::{NowPlayingTrack, kind::Playback, track_position::TrackPosition},
    osd::{Osd, OsdLevel, OsdPosition},
    space_editor::{
        DotIndex, SpaceEditor,
        dot::SpaceDot,
        rows::{MeasuredIn, MotionChoice},
    },
    switcher::{
        app_switcher::{AppKey, AppSwitcher, SwitcherApp, TilePresence},
        switcher_fit::SwitcherMetrics,
    },
    thumbs::{
        shot_ghost::ShotGhost,
        shot_press::DragStart,
        shot_thumbnail::{ShotThumbnail, ThumbAction},
    },
    tokens::{
        control_center::{CONTROL_CENTER, ControlCenterSize},
        dock::{DockFloorSetting, DockMetrics, DockToken},
        notifications::{NotificationMetrics, NotificationToken},
        osd::{OsdMetrics, OsdToken},
        widgets::{WidgetGrid, WidgetMetrics},
    },
    user_picture::{
        choice::{FaceFile, PictureChoice, resolve_picture},
        mood::{Mood, PictureSize},
        picker::UserPicturePicker,
        picture::UserPicture,
        portrait::UserPortrait,
    },
    widget::{
        battery::{BatteryCell, BatteryEntry, BatteryWidget},
        calendar::{EventLine, MonthEntry, MonthFace, MonthIntent, MonthWidget, TodayLine},
        card::WidgetCard,
        clock::{ClockCity, ClockEntry, WorldClockWidget},
        contract::{NoIntent, Widget, WidgetContext, WidgetKind},
        exit::CardPresence,
        frame::WidgetFrame,
        gallery::WidgetGallery,
        kind::{CardTint, Lift, WidgetHost, WidgetSize, WidgetTitle},
        registry::{WidgetRegistry, provide_widget_registry},
        slot::WidgetSlotGuide,
        timeline::{Dated, EntryDate, Refresh, RefreshAsk, Timeline},
    },
};

pub use crate::kit::{KIT, component_sheets, kits, stylesheet};
