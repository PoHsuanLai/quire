//! Every component, one `<name>.rs` and `<name>.css` pair each (design/04-COMPONENTS.md).

pub mod account_tile;
pub mod alert;
pub mod alert_vocab;
pub mod animated_list;
pub mod app_switcher;
pub mod appearance_picker;
pub mod avatar;
pub(crate) mod banner_row;
pub mod banner_stack;
pub mod battery_figure;
pub mod battery_level;
pub(crate) mod battery_ring;
pub mod bump_on;
pub mod button;
pub mod button_face;
pub mod button_size;
pub mod chip;
pub mod chord;
pub mod clock_angles;
pub(crate) mod clock_dial;
pub mod clock_face;
pub mod clock_kind;
pub mod command_palette;
pub mod command_pill;
pub mod count;
pub mod device_battery;
pub(crate) mod device_forms;
pub mod device_glyph;
pub mod dock_parts;
pub mod drag_ghost;
pub mod edge_strip;
pub mod edit_surface;
pub(crate) mod edit_surface_ctx;
pub(crate) mod edit_surface_focus;
pub(crate) mod edit_surface_keys;
pub(crate) mod edit_surface_pointer;
pub(crate) mod edit_surface_spell;
pub(crate) mod edit_surface_spell_menu;
pub(crate) mod edit_surface_state;
pub mod emoji;
pub mod emoji_grid;
pub mod emoji_grid_nav;
pub mod flow;
pub mod group_header;
pub mod hover_card;
pub mod hover_strip;
pub mod icon_button;
pub mod icon_view;
pub mod idle_dim;
pub mod image_source;
pub mod kbd;
pub mod leaving_list;
pub(crate) mod leaving_row;
pub mod level;
pub(crate) mod light_mark;
pub mod link_pill;
pub mod list_row;
pub mod lock_clock;
pub(crate) mod lock_mood;
pub(crate) mod lock_picture;
pub mod lock_prompt;
pub mod lock_screen;
pub mod lock_vocab;
pub mod menu;
pub(crate) mod menu_active;
pub mod menu_bar_item;
pub mod menu_cursor;
pub mod menu_entry;
pub(crate) mod menu_filter;
pub(crate) mod menu_item;
pub(crate) mod menu_keys;
pub(crate) mod menu_kind;
pub(crate) mod menu_lines;
pub(crate) mod menu_match;
pub(crate) mod menu_panel;
pub mod menu_pick;
pub(crate) mod menu_return;
pub(crate) mod menu_rows;
pub(crate) mod menu_shape;
pub(crate) mod menu_surface;
pub(crate) mod menu_tracker;
pub(crate) mod module_disc;
pub mod module_grid;
pub mod module_panel;
pub mod module_tile;
pub mod module_tile_kind;
pub mod month_grid;
pub mod month_grid_data;
pub mod month_grid_density;
pub(crate) mod month_grid_header;
pub(crate) mod month_grid_weeks;
pub(crate) mod muted;
pub(crate) mod notification_body;
pub mod notification_card;
pub mod notification_parts;
pub mod notification_swipe;
pub mod now_playing;
pub mod now_playing_kind;
pub mod osd;
pub(crate) mod osd_phase;
pub(crate) mod palette_body;
pub mod palette_claim;
pub(crate) mod palette_expand;
pub mod palette_group;
pub(crate) mod palette_host;
pub(crate) mod palette_lines;
pub(crate) mod palette_motion;
pub(crate) mod palette_reveal;
pub(crate) mod palette_rows;
pub(crate) mod palette_select;
pub(crate) mod palette_shown;
pub(crate) mod palette_stops;
pub mod pane_switcher;
pub mod panel;
pub mod pass_through;
pub mod pdf_thumb;
pub(crate) mod pdf_thumb_grace;
pub mod peek;
pub mod play_pause;
pub mod polkit_prompt;
pub mod popover;
pub mod press;
pub mod preview_content;
pub mod preview_cue;
pub mod preview_pane;
pub mod provider_mark;
pub(crate) mod resize_edges;
pub mod rich_text;
pub mod row_action;
pub(crate) mod row_battery;
pub mod row_chord;
pub(crate) mod row_click;
pub mod row_hooks;
pub mod row_shape;
pub(crate) mod row_star;
pub mod scrim;
pub mod scrim_strength;
pub mod search_field;
pub(crate) mod secret_entry;
pub mod section_header;
pub mod segmented;
pub mod selection_bubble;
pub mod send_mood;
pub mod send_pill;
pub mod settings_row;
pub mod settings_row_phase;
pub mod settings_row_trailing;
pub mod sheet;
pub mod sheet_placement;
pub mod sheet_width;
pub(crate) mod shot_frame;
pub mod shot_ghost;
pub mod shot_press;
pub mod shot_thumbnail;
pub(crate) mod shown_phase;
pub mod sidebar_item;
pub mod slider;
pub mod space_editor;
pub mod spinner;
pub(crate) mod spring_presence;
pub mod standard_action;
pub mod status;
pub mod switcher_fit;
pub mod sync_halo;
pub mod tabs;
pub mod text_input;
pub mod text_input_focus;
pub mod text_input_kind;
pub(crate) mod text_input_mask;
pub(crate) mod text_input_parts;
pub mod text_runs;
pub mod toast;
pub mod toggle;
pub mod tooltip;
pub(crate) mod track;
pub mod track_position;
pub mod traffic_lights;
pub mod tree_item;
pub(crate) mod tree_item_parts;
pub mod user_picture;
pub mod vocab;
pub mod widget_exit;
pub mod widget_frame;
pub mod widget_kind;
pub(crate) mod widget_scope;
pub mod widget_slot;
pub mod window_frame;
pub mod workspace_pills;

pub use crate::components::emoji::{
    disc::{DiscHue, EmojiDisc, EmojiPlayback},
    id::EmojiId,
};
pub use crate::components::hover_card::{
    intent::{HoverAnchor, HoverDriver, use_hover_intent},
    parts::{FlagTone, HoverCardPart, HoverMessage, HoverStat, KeyHint},
    target::{HoverTarget, TargetElement},
};
pub use crate::components::image_source::ImageSource;
pub use crate::components::level::{
    control::LevelControl,
    vocab::{LevelGlyph, LevelLook, LevelMode, LevelSource, Muting, Tick},
};
pub use crate::components::menu_kind::{MenuEntrance, MenuKind};
pub use crate::components::space_editor::{
    dot::SpaceDot,
    rows::{MeasuredIn, MotionChoice, MotionLevels},
};
pub use crate::components::status::{
    battery::BatteryGlyph,
    battery_state::{BatteryPower, BatteryState, LowAt},
    bluetooth::BluetoothGlyph,
    bluetooth_state::BluetoothState,
    family::{StatusGlyph, StatusState},
    volume::{VolumeGlyph, VolumeState, VolumeWaves},
    wifi::WifiGlyph,
    wifi_state::{WifiBars, WifiReach, WifiState},
};
pub use crate::components::user_picture::{
    choice::{FaceFile, PictureChoice, resolve_picture},
    mood::{Mood, PictureSize},
    picker::{PICTURE_CELL, PICTURE_COLUMNS, UserPicturePicker},
    picture::UserPicture,
    portrait::UserPortrait,
};
pub use crate::components::{
    palette_host::{CommandPaletteHost, PaletteEntrance},
    palette_motion::{PaletteHandle, use_palette_handle},
};
pub use crate::components::{sheet_placement::SheetPlacement, sheet_width::SheetWidth};
pub use crate::components::{
    text_input_focus::Focus,
    text_input_kind::{Grow, Rows, TextInputKind},
};
pub use crate::core::press::{PointerButton, Press};
pub use crate::motion::pulse_key::{PulseKey, PulsePhase};
pub use crate::motion::wake::WakeStamp;
pub use crate::tokens::status::StatusMetrics;
pub use account_tile::{AccountFace, AccountTile, AddAccountTile};
pub use alert::Alert;
pub use alert_vocab::{AlertButton, AlertEmphasis};
pub use animated_list::AnimatedList;
pub use app_switcher::{AppKey, AppSwitcher, SwitcherApp, TilePresence};
pub use appearance_picker::{AppearancePicker, PickerLayout};
pub use avatar::{
    Avatar, AvatarFace, AvatarMuting, AvatarShape, AvatarSize, AvatarTone, PersonHue, person_hue,
};
pub use banner_stack::{Banner, BannerEntry, BannerKey, BannerPosition, BannerStack};
pub use battery_figure::{BatteryFigure, use_battery_figure};
pub use battery_level::{BatteryLevel, FILL as BATTERY_FILL, RingMark, use_battery_fill};
pub use bump_on::{Bumped, use_bump_on};
pub use button::{Button, ButtonVariant};
pub use button_face::{ButtonFace, FaceMark, Leading, Trailing};
pub use button_size::ButtonSize;
pub use chip::{Chip, ChipVariant};
pub use chord::Chord;
pub use clock_angles::{Hands, Tenths, hands};
pub use clock_face::ClockFace;
pub use clock_kind::{ClockLook, ClockTime, DayPhase, Seconds};
pub use command_palette::{ASIDE_WIDTH, CommandPalette};
pub use command_pill::CommandPill;
pub use count::{Count, CountPlace};
pub use device_battery::DeviceBattery;
pub use device_glyph::{Device, DeviceGlyph};
pub use dock_parts::{DockFloor, RunningDot};
pub use drag_ghost::{DragGhost, DragReturnFrame, DropLine, Grip};
pub use edge_strip::{EdgeStrip, SideState};
pub use edit_surface::EditSurface;
pub use edit_surface_spell_menu::{SPELL_SUGGESTIONS, SpellMarks};
pub use emoji::{AnimatedEmoji, EMOJI_ATTRIBUTION};
pub use emoji_grid::{EMOJI_CELL, EMOJI_COLUMNS, EmojiCell, EmojiCells, EmojiGrid};
pub use emoji_grid_nav::{GridEdge, GridMove, GridStep, grid_step};
pub use flow::Flow;
pub use group_header::GroupHeader;
pub use hover_card::HoverCard;
pub use hover_strip::{ActionId, HoverStrip, StripAction, Titles};
pub use icon_button::{IconButton, IconButtonVariant};
pub use icon_view::IconView;
pub use idle_dim::IdleDim;
pub use image_source::ImageSize;
pub use kbd::{Kbd, KbdSize};
pub use leaving_list::{LeavingItem, LeavingList};
pub use link_pill::{LinkPill, LinkTarget};
pub use list_row::ListRow;
pub use lock_clock::LockClock;
pub use lock_prompt::LockPrompt;
pub use lock_screen::LockScreen;
pub use lock_vocab::{CapsLock, LockLook, LockUser, PromptState};
pub use menu::Menu;
pub use menu_bar_item::MenuBarItem;
pub use menu_cursor::Cursor;
pub use menu_entry::{MenuEntry, MenuRow, Tile, Trail};
pub use menu_filter::Filter;
pub use menu_pick::PickDismiss;
pub use module_grid::{GridColumns, GridMetrics, ModuleGrid};
pub use module_panel::{ModulePanel, PanelPlate};
pub use module_tile::ModuleTile;
pub use module_tile_kind::{Chevron, DiscMotion, ModuleState, TileSpan};
pub use month_grid::MonthGrid;
pub use month_grid_data::{
    DayKey, DayMark, DayPlace, Eventful, IsoWeek, MonthDay, MonthGridData, MonthKey, MonthWeek,
    Step, WeekNumbers,
};
pub use month_grid_density::MonthDensity;
pub use notification_card::NotificationCard;
pub use notification_parts::{AppMark, CardAction, GroupCount, Hover, Layers};
pub use notification_swipe::Swipe;
pub use now_playing::NowPlayingTrack;
pub use now_playing_kind::Playback;
pub use osd::{Level, Osd, OsdPosition};
pub use palette_claim::{Claim, FieldKey};
pub use palette_group::{GroupEntries, PaletteGroup, PaletteGroups};
pub use palette_shown::Retain;
pub use pane_switcher::PaneSwitcher;
pub use panel::{Panel, PanelEdge, PanelScrim};
pub use pass_through::{DataAttr, DataName, ExtraClass, PassThroughError};
pub use pdf_thumb::{
    PDF_DEFAULT_SHEET, PDF_THUMB_GRACE, PdfPage, PdfThumb, PdfTrouble, sheet_rect,
};
pub use peek::Peek;
pub use play_pause::PlayPauseButton;
pub use polkit_prompt::PolkitPrompt;
pub use popover::{Dismiss, Elevation, Popover};
pub use press::Propagation;
pub use preview_content::{Mono, PANE_MEDIA, PaneContent};
pub use preview_cue::PaneCue;
pub use preview_pane::{PaneAction, PreviewPane};
pub use provider_mark::{MarkSize, MarkStyle, Provider, ProviderMark};
pub use rich_text::{Rich, RichRun, RichText};
pub use row_action::RowAction;
pub use row_chord::{ChordShown, RowChord};
pub use row_hooks::PartHooks;
pub use row_shape::{ClipBody, RowShape};
pub use scrim::Scrim;
pub use scrim_strength::ScrimStrength;
pub use search_field::SearchField;
pub use section_header::{HeaderKind, SectionHeader};
pub use segmented::{SegSize, SegmentedControl};
pub use selection_bubble::{BubbleAction, BubbleButton, BubbleMode, SelectionBubble};
pub use send_mood::SendMood;
pub use send_pill::{PillAction, SendPhase, SendPill, SendRing};
pub use settings_row::SettingsRow;
pub use settings_row_phase::{RowDisc, RowPhase, RowWork};
pub use settings_row_trailing::RowTrailing;
pub use sheet::Sheet;
pub use shot_ghost::ShotGhost;
pub use shot_press::DragStart;
pub use shot_thumbnail::{ShotThumbnail, ThumbAction};
pub use sidebar_item::{ItemKind, PlaceId, Preview, SidebarItem, TodayTrailing};
pub use slider::Slider;
pub use space_editor::{ActiveDot, DotIndex, SpaceEditor};
pub use spinner::{Spinner, SpinnerKind};
pub use standard_action::{Reserved, SpaceNumber, StandardAction};
pub use switcher_fit::{
    SWITCHER_MARGIN, SWITCHER_PADDING, SwitcherFit, SwitcherMetrics, fit as switcher_fit,
};
pub use sync_halo::{SyncHalo, SyncState};
pub use tabs::Tabs;
pub use text_input::{InputVariant, TextInput};
pub use text_runs::{Run, RunTone, Text};
pub use toast::{ToastHost, use_toasts};
pub use toggle::Toggle;
pub use tooltip::{Shown, Tooltip, TooltipKind};
pub use track_position::TrackPosition;
pub use traffic_lights::TilePose;
pub use tree_item::{Disclosure, TreeItem, TreeShape};
pub use vocab::{
    Availability, Check, DropState, Emphasis, Expanded, Fraction, Here, Key, Percent, Selection,
    Shortcut, StaggerIndex, Switch,
};
pub use widget_exit::CardPresence;
pub use widget_frame::WidgetFrame;
pub use widget_kind::{CardTint, Lift, WidgetHost, WidgetSize, WidgetTitle};
pub use widget_slot::WidgetSlotGuide;
pub use window_frame::{TrafficLights, WindowFrame, WindowTitlebar};
pub use workspace_pills::{WorkspacePill, WorkspacePills};
