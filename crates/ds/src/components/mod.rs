//! Every component, one `<name>.rs` and `<name>.css` pair each (design/04-COMPONENTS.md).

pub(crate) mod app;
pub(crate) mod chrome;
pub(crate) mod content;
pub(crate) mod controls;
pub(crate) mod fields;
pub(crate) mod lists;
pub(crate) mod menus;
pub(crate) mod overlays;

pub use crate::components::app::account_tile::{AccountFace, AccountTile, AddAccountTile};
pub use crate::components::app::command_pill::CommandPill;
pub use crate::components::app::edge_strip::{EdgeStrip, SideState};
pub use crate::components::app::hover_strip::{ActionId, HoverStrip, StripAction, Titles};
pub use crate::components::app::link_pill::{LinkPill, LinkTarget};
pub use crate::components::app::peek::Peek;
pub use crate::components::app::send_mood::SendMood;
pub use crate::components::app::send_pill::{PillAction, SendPhase, SendPill, SendRing};
pub use crate::components::app::sidebar_item::{
    ItemKind, PlaceId, Preview, SidebarItem, TodayTrailing,
};
pub use crate::components::app::sync_halo::{SyncHalo, SyncState};
pub use crate::components::app::tree_item::{Disclosure, TreeItem, TreeShape};
pub use crate::components::chrome::traffic_lights::TilePose;
pub use crate::components::chrome::window_frame::{TrafficLights, WindowFrame, WindowTitlebar};
pub use crate::components::content::avatar::{
    Avatar, AvatarFace, AvatarMuting, AvatarShape, AvatarSize, AvatarTone, PersonHue, person_hue,
};
pub use crate::components::content::emoji_grid::{
    EMOJI_CELL, EMOJI_COLUMNS, EmojiCell, EmojiCells, EmojiGrid,
};
pub use crate::components::content::emoji_grid_nav::{GridEdge, GridMove, GridStep, grid_step};
pub use crate::components::content::icon_view::IconView;
pub use crate::components::content::image_source::ImageSize;
pub use crate::components::content::image_source::ImageSource;
pub use crate::components::content::pane_switcher::PaneSwitcher;
pub use crate::components::content::pdf_thumb::{
    PDF_DEFAULT_SHEET, PDF_THUMB_GRACE, PdfPage, PdfThumb, PdfTrouble, sheet_rect,
};
pub use crate::components::content::preview_content::{Mono, PANE_MEDIA, PaneContent};
pub use crate::components::content::preview_cue::PaneCue;
pub use crate::components::content::preview_pane::{PaneAction, PreviewPane};
pub use crate::components::content::provider_mark::{MarkSize, MarkStyle, Provider, ProviderMark};
pub use crate::components::content::rich_text::{Rich, RichRun, RichText};
pub use crate::components::content::status::{
    battery::BatteryGlyph,
    battery_state::{BatteryPower, BatteryState, LowAt},
    bluetooth::BluetoothGlyph,
    bluetooth_state::BluetoothState,
    family::{StatusGlyph, StatusState},
    volume::{VolumeGlyph, VolumeState, VolumeWaves},
    wifi::WifiGlyph,
    wifi_state::{WifiBars, WifiReach, WifiState},
};
pub use crate::components::content::text_runs::{Run, RunTone, Text};
pub use crate::components::controls::appearance_picker::{AppearancePicker, PickerLayout};
pub use crate::components::controls::bump_on::{Bumped, use_bump_on};
pub use crate::components::controls::button::{Button, ButtonVariant};
pub use crate::components::controls::button_face::{ButtonFace, FaceMark, Leading, Trailing};
pub use crate::components::controls::button_size::ButtonSize;
pub use crate::components::controls::chip::{Chip, ChipVariant};
pub use crate::components::controls::chord::Chord;
pub use crate::components::controls::count::{Count, CountPlace};
pub use crate::components::controls::icon_button::{IconButton, IconButtonVariant};
pub use crate::components::controls::kbd::{Kbd, KbdSize};
pub use crate::components::controls::level::{
    control::LevelControl,
    vocab::{LevelGlyph, LevelLook, LevelMode, LevelSource, Muting, Tick},
};
pub use crate::components::controls::pass_through::{
    DataAttr, DataName, ExtraClass, PassThroughError,
};
pub use crate::components::controls::press::Propagation;
pub use crate::components::controls::segmented::{SegSize, SegmentedControl};
pub use crate::components::controls::slider::Slider;
pub use crate::components::controls::spinner::{Spinner, SpinnerKind};
pub use crate::components::controls::tabs::Tabs;
pub use crate::components::controls::toggle::Toggle;
pub use crate::components::fields::edit_surface::EditSurface;
pub use crate::components::fields::edit_surface_spell_menu::{SPELL_SUGGESTIONS, SpellMarks};
pub use crate::components::fields::search_field::SearchField;
pub use crate::components::fields::selection_bubble::{
    BubbleAction, BubbleButton, BubbleMode, SelectionBubble,
};
pub use crate::components::fields::text_input::{InputVariant, TextInput};
pub use crate::components::fields::{
    text_input_focus::Focus,
    text_input_kind::{Grow, Rows, TextInputKind},
};
pub use crate::components::lists::animated_list::AnimatedList;
pub use crate::components::lists::leaving_list::{LeavingItem, LeavingList};
pub use crate::components::lists::list_row::ListRow;
pub use crate::components::lists::row_hooks::PartHooks;
pub use crate::components::lists::section_header::{HeaderKind, SectionHeader};
pub use crate::components::lists::settings_row::SettingsRow;
pub use crate::components::lists::settings_row_phase::{RowDisc, RowPhase, RowWork};
pub use crate::components::lists::settings_row_trailing::RowTrailing;
pub use crate::components::menus::menu::Menu;
pub use crate::components::menus::menu_cursor::Cursor;
pub use crate::components::menus::menu_entry::{MenuEntry, MenuRow, Tile, Trail};
pub use crate::components::menus::menu_filter::Filter;
pub use crate::components::menus::menu_kind::{MenuEntrance, MenuKind};
pub use crate::components::menus::menu_pick::PickDismiss;
pub use crate::components::menus::palette::command_palette::{ASIDE_WIDTH, CommandPalette};
pub use crate::components::menus::palette::palette_claim::{Claim, FieldKey};
pub use crate::components::menus::palette::palette_group::{
    GroupEntries, PaletteGroup, PaletteGroups,
};
pub use crate::components::menus::palette::palette_shown::Retain;
pub use crate::components::menus::palette::{
    palette_host::{CommandPaletteHost, PaletteEntrance},
    palette_motion::{PaletteHandle, use_palette_handle},
};
pub use crate::components::menus::row_action::RowAction;
pub use crate::components::menus::row_chord::{ChordShown, RowChord};
pub use crate::components::menus::row_shape::{ClipBody, RowShape};
pub use crate::components::menus::standard_action::{Reserved, SpaceNumber, StandardAction};
pub use crate::components::overlays::alert::Alert;
pub use crate::components::overlays::alert_vocab::{AlertButton, AlertEmphasis};
pub use crate::components::overlays::drag_ghost::{DragGhost, DragReturnFrame, DropLine, Grip};
pub use crate::components::overlays::flow::Flow;
pub use crate::components::overlays::hover_card::HoverCard;
pub use crate::components::overlays::hover_card::{
    intent::{HoverAnchor, HoverDriver, use_hover_intent},
    parts::{FlagTone, HoverCardPart, HoverMessage, HoverStat, KeyHint},
    target::{HoverTarget, TargetElement},
};
pub use crate::components::overlays::panel::{Panel, PanelEdge, PanelScrim};
pub use crate::components::overlays::popover::{Dismiss, Elevation, Popover};
pub use crate::components::overlays::scrim::Scrim;
pub use crate::components::overlays::scrim_strength::ScrimStrength;
pub use crate::components::overlays::sheet::Sheet;
pub use crate::components::overlays::toast::{ToastHost, use_toasts};
pub use crate::components::overlays::tooltip::{Shown, Tooltip, TooltipKind};
pub use crate::components::overlays::{sheet_placement::SheetPlacement, sheet_width::SheetWidth};
pub use crate::core::press::{PointerButton, Press};
pub use crate::core::vocab::{
    Availability, Check, DropState, Emphasis, Expanded, Fraction, Here, Key, Percent, Selection,
    Shortcut, StaggerIndex, Switch,
};
pub use crate::motion::pulse_key::{PulseKey, PulsePhase};
pub use crate::motion::wake::WakeStamp;
pub use crate::shell::bar::menu_bar_item::MenuBarItem;
pub use crate::shell::bar::workspace_pills::{WorkspacePill, WorkspacePills};
pub use crate::shell::battery::device_battery::DeviceBattery;
pub use crate::shell::battery::device_glyph::{Device, DeviceGlyph};
pub use crate::shell::battery::figure::{BatteryFigure, use_battery_figure};
pub use crate::shell::battery::level::{
    BatteryLevel, FILL as BATTERY_FILL, RingMark, use_battery_fill,
};
pub use crate::shell::clock::angles::{Hands, Tenths, hands};
pub use crate::shell::clock::face::ClockFace;
pub use crate::shell::clock::kind::{ClockLook, ClockTime, DayPhase, Seconds};
pub use crate::shell::control_center::module_grid::{GridColumns, GridMetrics, ModuleGrid};
pub use crate::shell::control_center::module_panel::{ModulePanel, PanelPlate};
pub use crate::shell::control_center::module_tile::ModuleTile;
pub use crate::shell::control_center::module_tile_kind::{
    Chevron, DiscMotion, ModuleState, TileSpan,
};
pub use crate::shell::dock_parts::{DockFloor, RunningDot};
pub use crate::shell::emoji::{AnimatedEmoji, EMOJI_ATTRIBUTION};
pub use crate::shell::emoji::{
    disc::{DiscHue, EmojiDisc, EmojiPlayback},
    id::EmojiId,
};
pub use crate::shell::idle_dim::IdleDim;
pub use crate::shell::lock::clock::LockClock;
pub use crate::shell::lock::polkit_prompt::PolkitPrompt;
pub use crate::shell::lock::prompt::LockPrompt;
pub use crate::shell::lock::screen::LockScreen;
pub use crate::shell::lock::vocab::{CapsLock, LockLook, LockUser, PromptState};
pub use crate::shell::month_grid::MonthGrid;
pub use crate::shell::month_grid::data::{
    DayKey, DayMark, DayPlace, Eventful, IsoWeek, MonthDay, MonthGridData, MonthKey, MonthWeek,
    Step, WeekNumbers,
};
pub use crate::shell::month_grid::density::MonthDensity;
pub use crate::shell::notifications::banner_stack::{
    Banner, BannerEntry, BannerKey, BannerPosition, BannerStack,
};
pub use crate::shell::notifications::card::NotificationCard;
pub use crate::shell::notifications::group_header::GroupHeader;
pub use crate::shell::notifications::parts::{AppMark, CardAction, GroupCount, Hover, Layers};
pub use crate::shell::notifications::swipe::Swipe;
pub use crate::shell::now_playing::NowPlayingTrack;
pub use crate::shell::now_playing::kind::Playback;
pub use crate::shell::now_playing::play_pause::PlayPauseButton;
pub use crate::shell::now_playing::track_position::TrackPosition;
pub use crate::shell::osd::{Level, Osd, OsdPosition};
pub use crate::shell::space_editor::{ActiveDot, DotIndex, SpaceEditor};
pub use crate::shell::space_editor::{
    dot::SpaceDot,
    rows::{MeasuredIn, MotionChoice, MotionLevels},
};
pub use crate::shell::switcher::app_switcher::{AppKey, AppSwitcher, SwitcherApp, TilePresence};
pub use crate::shell::switcher::switcher_fit::{
    SWITCHER_MARGIN, SWITCHER_PADDING, SwitcherFit, SwitcherMetrics, fit as switcher_fit,
};
pub use crate::shell::thumbs::shot_ghost::ShotGhost;
pub use crate::shell::thumbs::shot_press::DragStart;
pub use crate::shell::thumbs::shot_thumbnail::{ShotThumbnail, ThumbAction};
pub use crate::shell::user_picture::{
    choice::{FaceFile, PictureChoice, resolve_picture},
    mood::{Mood, PictureSize},
    picker::{PICTURE_CELL, PICTURE_COLUMNS, UserPicturePicker},
    picture::UserPicture,
    portrait::UserPortrait,
};
pub use crate::shell::widget::exit::CardPresence;
pub use crate::shell::widget::frame::WidgetFrame;
pub use crate::shell::widget::kind::{CardTint, Lift, WidgetHost, WidgetSize, WidgetTitle};
pub use crate::shell::widget::slot::WidgetSlotGuide;
pub use crate::style::tokens::status::StatusMetrics;
