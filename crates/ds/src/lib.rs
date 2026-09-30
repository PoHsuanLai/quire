//! quire's renderer-free design system: tokens, palette, icons, components and motion.
//!
//! The crate is layered as the crates it will split into: `core` (vocabulary, geometry, time,
//! tasks, colour), `style` (appearance, tokens, materials, palettes, fonts, icons, the
//! stylesheet's sections), `motion` (animation data, timers, machines and the details), the
//! linter, then the components with the host seams under them, `shell` for the shell's own
//! parts, and `assembly` on top. A layer names only the layers below it
//! (`scripts/check-boundary.sh`). Every public item has one path: a root name, or a name in one
//! of the modules below. DESIGN.md maps each module to the design doc section it implements.

mod assembly;
pub mod catalog;
mod components;
pub mod detail;
mod edit;
mod file_drop;
mod focus;
mod host;
pub mod icon;
pub mod motion;
mod root;
mod shell;
mod spell;
mod stack;
pub mod time;
pub mod widget;
mod window;

pub use crate::assembly::selectors;
pub use crate::assembly::{
    ds::{Ds, Inject},
    kit::kits,
    stylesheet::{component_sheets, stylesheet},
};
pub use crate::components::{
    app::{
        account_tile::{AccountFace, AccountTile, AddAccountTile},
        command_pill::CommandPill,
        edge_strip::EdgeStrip,
        hover_strip::{ActionId, HoverStrip, StripAction, Titles},
        link_pill::{LinkPill, LinkTarget},
        peek::Peek,
        send_mood::SendMood,
        send_pill::{PillAction, SEND_COUNTDOWN, SEND_TICK, SendPhase, SendPill},
        sidebar_item::{ItemKind, PlaceId, Preview, SidebarItem, TodayTrailing},
        tree_item::{TreeItem, TreeShape},
    },
    chrome::{
        traffic_lights::TilePose,
        window_frame::{TrafficLights, WindowFrame, WindowTitlebar},
    },
    content::{
        avatar::{
            Avatar, AvatarFace, AvatarShape, AvatarSize, AvatarTone, PersonHue, muting_colour,
            person_hue,
        },
        icon_source::{ExternalIcon, IconSource},
        icon_view::IconView,
        image_source::{ImageSize, ImageSource},
        level_glyph::vocab::{LevelGlyph, LevelLook, LevelMode, LevelSource},
        pdf_thumb::{PdfPage, PdfThumb, PdfTrouble},
        provider_mark::{MarkProvider, MarkSize, MarkStyle, ProviderMark},
        rich_text::{Rich, RichRun, RichText},
        status::{
            battery::BatteryGlyph,
            battery_state::{BatteryPower, BatteryState, LowAt},
            bluetooth::BluetoothGlyph,
            bluetooth_state::BluetoothState,
            family::StatusState,
            volume::{VolumeGlyph, VolumeState, VolumeWaves},
            wifi::WifiGlyph,
            wifi_state::{WifiBars, WifiReach, WifiState},
        },
        text_runs::{RunTone, TextLine, TextRun},
        voice_orb::{
            model::{OrbColour, OrbColours},
            view::{ORB_PERIOD, ORB_SIZE, VoiceOrb},
        },
    },
    controls::{
        button::{Button, ButtonVariant},
        button_face::{ButtonFace, Leading, Trailing},
        button_size::ButtonSize,
        chip::{Chip, ChipVariant},
        count::{Count, CountPlace},
        icon_button::{IconButton, IconButtonVariant},
        kbd::{Kbd, KbdSize},
        level::control::LevelControl,
        press::Propagation,
        segmented::{SegSize, SegmentedControl},
        slider::Slider,
        spinner::Spinner,
        tabs::Tabs,
        toggle::Toggle,
    },
    editor::{spell_menu::SpellMarks, surface::EditSurface},
    fields::{
        search_field::SearchField,
        selection_bubble::{BubbleAction, BubbleButton, BubbleMode, SelectionBubble},
        text_input::{InputVariant, TextInput},
        text_input_focus::FieldFocus,
        text_input_kind::{Grow, Rows, TextInputKind},
    },
    lists::{
        animated_list::AnimatedList,
        appearance_picker::{AppearancePicker, PickerLayout},
        emoji_grid::grid::{EMOJI_CELL, EMOJI_COLUMNS, EmojiCell, EmojiCells, EmojiGrid},
        leaving_list::{LeavingItem, LeavingList},
        list_row::ListRow,
        preview::{
            content::{PANE_MEDIA, PaneContent, PaneMono},
            pane::{PaneAction, PreviewPane},
            switcher::PaneSwitcher,
        },
        row_hooks::PartHooks,
        section_header::{HeaderKind, SectionHeader},
        settings_row::SettingsRow,
        settings_row_phase::{RowDisc, RowPhase},
        settings_row_trailing::RowTrailing,
    },
    menus::{
        menu::Menu,
        menu_cursor::MenuCursor,
        menu_entry::{MenuEntry, MenuRow, MenuTile, MenuTrail},
        menu_filter::MenuFilter,
        menu_kind::{MenuEntrance, MenuKind},
        menu_pick::PickDismiss,
        palette::{
            command_palette::CommandPalette,
            palette_claim::{Claim, FieldKey},
            palette_group::{GroupEntries, PaletteGroup, PaletteGroups},
            palette_host::CommandPaletteHost,
            palette_motion::{PaletteHandle, use_palette_handle},
            palette_shown::Retain,
        },
        row_action::RowAction,
        row_chord::{ChordShown, RowChord},
        row_shape::{ClipBody, RowShape},
    },
    overlays::{
        alert::Alert,
        alert_vocab::AlertEmphasis,
        drag_ghost::{DragGhost, DragReturnFrame, DropLine, Grip},
        flow::Flow,
        hover_card::{
            HoverCard,
            intent::{HoverAnchor, use_hover_intent},
            parts::{FlagTone, HoverCardPart, HoverMessage, HoverStat, KeyHint},
            target::{HoverTarget, TargetElement},
        },
        panel::{Panel, PanelEdge, PanelScrim},
        popover::{Elevation, Popover},
        scrim::Scrim,
        scrim_strength::ScrimStrength,
        sheet::Sheet,
        sheet_placement::SheetPlacement,
        toast::use_toasts,
        tooltip::{Tooltip, TooltipKind},
    },
};
pub use crate::edit::{
    handle::{EditHandle, use_edit_handle},
    input::{Composition, EditInput, KeyInput, PreeditCursor},
    pointer::{EditFocus, EditPointer, Extend},
};
pub use crate::file_drop::{
    board::FileDropBoard,
    drag::{DropAcceptance, FileDrag, FileDragInput, FileDrop, Offer},
    hook::use_file_drop,
};
pub use crate::focus::{
    field::{FieldHandle, use_field_handle},
    request::{FocusRequest, use_focus_request},
    select::Select,
    selector::{FocusError, focus_by_selector},
    soon::focus_soon,
};
pub use crate::host::{
    captured::{CapturedPointer, PointerPhase},
    caret::{Caret, Collapsed, FieldSelection, InitialCaret, caret_at},
    document::{DocumentHost, use_document_host},
    drop_hit::DropHit,
    fallback::Fallback,
    focused::Focused,
    found::{Found, SameNode},
    hand_back::{HandBack, Record},
    ime::{ImeEvent, ImeListener, ImeSwitch},
    measure::{Anchor, Measured, MountedRef, RectProbe, use_rect},
    no_host::NoHost,
    parts::{CaretHost, ClickFocusHost, EditHost, FileDropHost, FocusHost, GeometryHost, ImeHost},
    pasted::Pasted,
    position::{EDIT_KIND_ATTR, EDIT_NODE_ATTR, EditKind, EditNode, TextPosition, TextRange},
    probe::Probe,
    reveal::{ScrollSpan, Scrolled, nearest_scroll},
    signals::HostSignals,
};
pub use crate::root::{
    chrome::{FrameTint, Ground, RootChrome},
    common::Common,
    extent::RootExtent,
    pass_through::{DataAttr, DataName, ExtraClass, PassThroughError},
    surface::Surface,
    typeface::use_typeface,
};
pub use crate::shell::{
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
    dock_parts::{DockFloor, RunningDot},
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
    now_playing::{
        NowPlayingTrack, kind::Playback, play_pause::PlayPauseButton, track_position::TrackPosition,
    },
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
        shell_scale::{SHELL_SCALE, ShellSize},
        shell_type::{FontWeight, ShellMetrics, ShellType},
        widget_paint::WidgetPaint,
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
pub use crate::spell::{
    lang::{Lang, Spell},
    marks::SpellReplace,
    service::{Learned, Paragraph, SpellFuture, SpellService},
    words::WordSpan,
};
pub use crate::stack::{
    host::{OverlayId, use_overlays},
    hover_hub::{HoverKey, HoverKind, use_hover_hub},
    layer_stack::{Dismissal, LayerId, LayerStack},
    menu_track::types::{
        Branch, ItemPath, MenuAnim, MenuDirection, MenuKey, MenuPhase, MenuTarget, MenuTiming,
        MenuTrack, MenuTrackEffect, MenuTrackEvent, Pickable, ShownBy,
    },
    pull_tab::{Pull, PullTab, TabArm},
    roving::{Rove, Roving, Wrap},
    toast_hub::{ToastState, UndoToken, use_toast_hub},
    typeahead::Typeahead,
};
pub use crate::window::{
    host::{HostWindow, WindowHost, use_window_host_provider},
    vocab::{
        Activation, Fullscreen, Maximized, ResizeEdge, Support, TileError, WindowState, WindowTile,
        Zoom,
    },
};
pub use ds_core::word::Word;
pub use ds_core::{
    colour::contrast::{Verdict, ratio},
    geometry::{
        placement::{Align, Flip, Placed, Placement, Side, place},
        scale::Scale,
        units::{Point, Px, Rect, Size},
    },
    press::{PointerButton, Press},
    spawner::Spawner,
    standard_action::{Reserved, SpaceNumber, StandardAction},
    text::clip::clip_chars,
    time::{
        FRAME_SLACK, FRAME_TICK,
        clock::{ClockGuard, VirtualClock, sleep},
    },
    vocab::{
        Activity, Availability, Check, Dismiss, DropState, Emphasis, FocusStyle, Fraction,
        InputModality, Muting, Percent, PressPhase, RowState, Selection, Shortcut, ShortcutKey,
        Shown,
    },
};
pub use ds_motion::{
    anim::Anim,
    drag::{DRAG_THRESHOLD, Drag, DragPhase, DragTracker, WINDOW_DRAG_THRESHOLD, use_drag},
    hover_intent::{HoverEvent, HoverIntent, HoverProfile, HoverWarmth, IntentEffect, IntentPhase},
    long_press::{LONG_PRESS_SLOP, LongPress, LongPressEffect, LongPressEvent},
    pane_slide::Pane,
    presence::{Exit, Presence},
    pulse_key::{PulseKey, PulsePhase},
    recipe::{Fill, Iteration},
    roster::{Heal, RosterState, RowPitch, StayError, Stayed},
    rubber::{RUBBER_SHARE, resist},
    settle::settle,
    swipe::{Speed, SwipeMetrics},
    timer::{TimerPhase, use_motion_timer},
    use_collapse::{Collapse, use_collapse},
    use_roster::{LeaveBy, Pitches, Roster, RosterSpec, use_roster},
    wake::WakeStamp,
};
pub use ds_style::{
    appearance::{
        accent::Accent,
        appearance::Appearance,
        blur::BlurState,
        material::Material,
        motion::{Motion, MotionLevel},
        peek::PeekMode,
        resolve::{Resolved, resolve},
        system::{Contrast, ReducedMotion, SystemPrefs},
        theme::{Scheme, Theme},
        typeface::Typeface,
    },
    fonts::{FACES, Face, FaceStyle, Subset},
    icon::{
        Icon,
        classify::ChromaLimit,
        family::PlateFamily,
        plate_tint::PlateTint,
        render::{Glyph, GlyphProps, IconPx, IconSize},
        shape::Shape,
        url::IconUrl,
    },
    kit::{Kit, KitRank, Kits, KnownNames, Section, Vocabulary},
    look::Look,
    material::{recipe::recipe, stack::MaterialStack},
    scale::use_scale,
    scope::{Scope, use_scope},
    space::{
        frame_vars::FrameVars,
        look::{CardAccent, Grain, SpaceLook},
        palette::{Capping, Dot, derive, readout::readout, swatch},
        presets::{PRESETS, default_look},
        store::{SpaceDefaults, SpaceStore, Workspace, WorkspaceId, WorkspaceIndex},
    },
    tokens::{
        accent_table::accent_of,
        colour::ColourToken,
        control_size::ControlSize,
        delay::DelayToken,
        easing::{Easing, EasingToken},
        elevation::Shadow,
        hex::{Alpha, Colour, Hex},
        label_hue::{HueColour, HueMember, LabelHue},
        layer::ZLayer,
        name::VarName,
        person::PersonSwatch,
        pixel::PixelToken,
        set::{Only, Place, TokenSet},
        shape::{Corner, Radius},
        size_scale::WholePx,
        spacing::SpacingToken,
        status::StatusMetrics,
        timing::DurationToken,
        token::{CssValue, Token, TokenKind, TokenScope},
        type_scale::{Family, FontSize},
        type_voice::VoiceToken,
    },
};

use futures_timer as _;
use serde_json as _;
use thiserror as _;
