//! Every component, one `<name>.rs` and `<name>.css` pair each (design/04-COMPONENTS.md).

pub mod account_tile;
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
pub mod clock_angles;
pub(crate) mod clock_dial;
pub mod clock_face;
pub mod clock_kind;
pub mod command_palette;
pub mod command_pill;
pub mod count;
pub mod dock_parts;
pub mod drag_ghost;
pub mod edge_strip;
pub mod edit_surface;
mod edit_surface_ctx;
mod edit_surface_focus;
mod edit_surface_keys;
mod edit_surface_pointer;
mod edit_surface_state;
pub mod flow;
pub mod group_header;
pub mod hover_card;
pub mod hover_strip;
pub mod icon_button;
pub mod icon_view;
pub mod image_source;
pub mod kbd;
pub mod level;
pub(crate) mod light_mark;
pub mod link_pill;
pub mod list_row;
pub mod lock_clock;
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
pub(crate) mod menu_surface;
pub(crate) mod menu_tracker;
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
pub mod osd;
pub(crate) mod osd_phase;
pub(crate) mod palette_host;
pub(crate) mod palette_lines;
pub(crate) mod palette_rows;
pub(crate) mod palette_select;
pub(crate) mod palette_shown;
pub mod pane_switcher;
pub mod panel;
pub mod pass_through;
pub mod peek;
pub mod persona;
pub mod polkit_prompt;
pub mod popover;
pub mod press;
pub mod provider_mark;
pub(crate) mod resize_edges;
pub mod rich_text;
pub mod row_action;
pub(crate) mod row_click;
pub mod row_hooks;
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
pub mod settings_row_trailing;
pub mod sheet;
pub mod sheet_placement;
pub(crate) mod sheet_presence;
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
pub mod switcher_fit;
pub mod sync_halo;
pub mod tabs;
pub mod text_input;
pub mod text_input_focus;
pub mod text_input_kind;
pub(crate) mod text_input_parts;
pub mod text_runs;
pub mod toast;
pub mod toggle;
pub mod tooltip;
pub(crate) mod track;
pub mod traffic_lights;
pub mod tree_item;
pub(crate) mod tree_item_parts;
pub mod vocab;
pub mod widget_frame;
pub mod widget_kind;
pub(crate) mod widget_scope;
pub mod window_frame;
pub mod workspace_pills;

pub use account_tile::{AccountFace, AccountTile, AddAccountTile};
pub use animated_list::AnimatedList;
pub use app_switcher::{AppKey, AppSwitcher, SwitcherApp, TilePresence};
pub use appearance_picker::{AppearancePicker, PickerLayout};
pub use avatar::{
    Avatar, AvatarFace, AvatarMuting, AvatarShape, AvatarSize, AvatarTone, PersonHue, person_hue,
};
pub use banner_stack::{Banner, BannerEntry, BannerKey, BannerPosition, BannerStack};
pub use battery_figure::{BatteryFigure, use_battery_figure};
pub use battery_level::{
    BatteryLevel, FILL as BATTERY_FILL, LevelRing, RingMark, use_battery_fill,
};
pub use bump_on::{Bumped, use_bump_on};
pub use button::{Button, ButtonVariant};
pub use button_face::{ButtonFace, FaceMark, Leading, Trailing};
pub use button_size::ButtonSize;
pub use chip::{Chip, ChipVariant};
pub use clock_angles::{Hands, Tenths, hands};
pub use clock_face::ClockFace;
pub use clock_kind::{ClockLook, ClockTime, DayPhase, Seconds};
pub use command_palette::{CommandPalette, CommandPaletteHost, PaletteEntrance};
pub use command_pill::CommandPill;
pub use count::{Count, CountPlace};
pub use dock_parts::{DockFloor, RunningDot};
pub use drag_ghost::{DragGhost, DropLine, Grip};
pub use edge_strip::{EdgeStrip, SideState};
pub use edit_surface::EditSurface;
pub use flow::Flow;
pub use group_header::GroupHeader;
pub use hover_card::{
    FlagTone, HoverAnchor, HoverCard, HoverCardPart, HoverDriver, HoverMessage, HoverStat,
    HoverTarget, KeyHint, TargetElement, use_hover_intent,
};
pub use hover_strip::{ActionId, HoverStrip, StripAction, Titles};
pub use icon_button::{IconButton, IconButtonVariant, StatusMetrics};
pub use icon_view::IconView;
pub use image_source::ImageSize;
pub use kbd::{Kbd, KbdSize};
pub use level::{LevelControl, LevelGlyph, LevelLook, LevelMode, Muting, Tick};
pub use link_pill::{LinkPill, LinkTarget};
pub use list_row::ListRow;
pub use lock_clock::LockClock;
pub use lock_prompt::LockPrompt;
pub use lock_screen::LockScreen;
pub use lock_vocab::{CapsLock, LockLook, LockUser, PromptState};
pub use menu::{Menu, MenuEntrance, MenuKind};
pub use menu_bar_item::MenuBarItem;
pub use menu_cursor::Cursor;
pub use menu_entry::{MenuEntry, MenuRow, Tile, Trail};
pub use menu_filter::Filter;
pub use menu_pick::PickDismiss;
pub use module_grid::{GridColumns, GridMetrics, ModuleGrid};
pub use module_panel::{ModulePanel, PanelPlate};
pub use module_tile::ModuleTile;
pub use module_tile_kind::{Chevron, ModuleState, TileSpan};
pub use month_grid::MonthGrid;
pub use month_grid_data::{
    DayKey, DayMark, DayPlace, Eventful, IsoWeek, MonthDay, MonthGridData, MonthKey, MonthWeek,
    Step, WeekNumbers,
};
pub use month_grid_density::MonthDensity;
pub use notification_card::NotificationCard;
pub use notification_parts::{AppMark, CardAction, GroupCount, Hover, Layers};
pub use notification_swipe::Swipe;
pub use osd::{Level, Osd, OsdPosition};
pub use palette_shown::Retain;
pub use pane_switcher::PaneSwitcher;
pub use panel::{Panel, PanelEdge, PanelScrim};
pub use pass_through::{DataAttr, DataName, ExtraClass, PassThroughError};
pub use peek::Peek;
pub use persona::{
    Accessory, BLINK_MAX, BLINK_MIN, Backdrop, Brows, Cheeks, Creature, Eyes, HairTone, HeadShape,
    Mood, Mouth, Persona, PersonaFinish, PersonaSeed, PersonaSize, PersonaSpec, Tone, Top,
    UserPicture, UserPortrait, WakeStamp,
};
pub use polkit_prompt::PolkitPrompt;
pub use popover::{Dismiss, Elevation, Popover};
pub use press::{PointerButton, Press, Propagation};
pub use provider_mark::{ImageSource, MarkSize, MarkStyle, Provider, ProviderMark};
pub use rich_text::{Rich, RichRun, RichText};
pub use row_action::RowAction;
pub use row_hooks::PartHooks;
pub use scrim::Scrim;
pub use scrim_strength::ScrimStrength;
pub use search_field::SearchField;
pub use section_header::{HeaderKind, SectionHeader};
pub use segmented::{SegSize, SegmentedControl};
pub use selection_bubble::{BubbleAction, BubbleButton, BubbleMode, SelectionBubble};
pub use send_mood::SendMood;
pub use send_pill::{PillAction, SendPhase, SendPill, SendRing};
pub use settings_row::SettingsRow;
pub use settings_row_trailing::RowTrailing;
pub use sheet::{Sheet, SheetPlacement, SheetWidth};
pub use shot_ghost::ShotGhost;
pub use shot_press::DragStart;
pub use shot_thumbnail::{ShotThumbnail, ThumbAction};
pub use sidebar_item::{ItemKind, PlaceId, Preview, SidebarItem, TodayTrailing};
pub use slider::Slider;
pub use space_editor::{
    ActiveDot, DotIndex, MeasuredIn, MotionChoice, MotionLevels, SpaceDot, SpaceEditor,
};
pub use spinner::{Spinner, SpinnerKind};
pub use switcher_fit::{
    SWITCHER_MARGIN, SWITCHER_PADDING, SwitcherFit, SwitcherMetrics, fit as switcher_fit,
};
pub use sync_halo::{SyncHalo, SyncState};
pub use tabs::Tabs;
pub use text_input::{FieldFace, Focus, Grow, InputVariant, Rows, TextInput, TextInputKind};
pub use text_runs::{Run, RunTone, Text};
pub use toast::{ToastHost, use_toasts};
pub use toggle::Toggle;
pub use tooltip::{Shown, Tooltip, TooltipKind};
pub use traffic_lights::TilePose;
pub use tree_item::{Disclosure, TreeItem, TreeShape};
pub use vocab::{
    Availability, Check, DropState, Emphasis, Expanded, Fraction, Here, Key, PulseKey, PulsePhase,
    Selection, Shortcut, StaggerIndex, Switch,
};
pub use widget_frame::WidgetFrame;
pub use widget_kind::{CardTint, WidgetHost, WidgetSize, WidgetTitle};
pub use window_frame::{TrafficLights, WindowFrame, WindowTitlebar};
pub use workspace_pills::{WorkspacePill, WorkspacePills};

/// Every component stylesheet, in the cascade's fixed order: `(component, css)`.
pub const CSS: &[(&str, &str)] = &[
    ("account_tile", include_str!("account_tile.css")),
    ("animated_list", include_str!("animated_list.css")),
    ("appearance_picker", include_str!("appearance_picker.css")),
    ("app_switcher", include_str!("app_switcher.css")),
    ("avatar", include_str!("avatar.css")),
    ("banner_stack", include_str!("banner_stack.css")),
    ("battery_level", include_str!("battery_level.css")),
    ("button", include_str!("button.css")),
    ("chip", include_str!("chip.css")),
    ("clock_face", include_str!("clock_face.css")),
    ("command_palette", include_str!("command_palette.css")),
    ("command_pill", include_str!("command_pill.css")),
    ("count", include_str!("count.css")),
    ("dock_parts", include_str!("dock_parts.css")),
    ("drag_ghost", include_str!("drag_ghost.css")),
    ("edge_strip", include_str!("edge_strip.css")),
    ("edit_surface", include_str!("edit_surface.css")),
    ("group_header", include_str!("group_header.css")),
    ("hover_card", include_str!("hover_card.css")),
    ("hover_strip", include_str!("hover_strip.css")),
    ("icon_button", include_str!("icon_button.css")),
    ("icon_view", include_str!("icon_view.css")),
    ("kbd", include_str!("kbd.css")),
    ("level", include_str!("level.css")),
    ("link_pill", include_str!("link_pill.css")),
    ("list_row", include_str!("list_row.css")),
    ("lock_screen", include_str!("lock_screen.css")),
    ("lock_clock", include_str!("lock_clock.css")),
    ("lock_prompt", include_str!("lock_prompt.css")),
    ("menu", include_str!("menu.css")),
    ("menu_bar_item", include_str!("menu_bar_item.css")),
    ("menu_entry", include_str!("menu_entry.css")),
    ("notification_card", include_str!("notification_card.css")),
    ("osd", include_str!("osd.css")),
    ("module_tile", include_str!("module_tile.css")),
    ("module_panel", include_str!("module_panel.css")),
    ("month_grid", include_str!("month_grid.css")),
    ("pane_switcher", include_str!("pane_switcher.css")),
    ("panel", include_str!("panel.css")),
    ("peek", include_str!("peek.css")),
    ("persona", include_str!("persona.css")),
    ("popover", include_str!("popover.css")),
    ("polkit_prompt", include_str!("polkit_prompt.css")),
    ("provider_mark", include_str!("provider_mark.css")),
    ("scrim", include_str!("scrim.css")),
    ("search_field", include_str!("search_field.css")),
    ("section_header", include_str!("section_header.css")),
    ("segmented", include_str!("segmented.css")),
    ("selection_bubble", include_str!("selection_bubble.css")),
    ("send_pill", include_str!("send_pill.css")),
    ("settings_row", include_str!("settings_row.css")),
    ("sheet", include_str!("sheet.css")),
    ("shot_thumbnail", include_str!("shot_thumbnail.css")),
    ("sidebar_item", include_str!("sidebar_item.css")),
    ("slider", include_str!("slider.css")),
    ("space_editor", include_str!("space_editor.css")),
    ("spinner", include_str!("spinner.css")),
    ("sync_halo", include_str!("sync_halo.css")),
    ("tabs", include_str!("tabs.css")),
    ("text_input", include_str!("text_input.css")),
    ("text_runs", include_str!("text_runs.css")),
    ("toast", include_str!("toast.css")),
    ("toggle", include_str!("toggle.css")),
    ("tooltip", include_str!("tooltip.css")),
    ("tree_item", include_str!("tree_item.css")),
    ("widget_frame", include_str!("widget_frame.css")),
    ("window_frame", include_str!("window_frame.css")),
    ("workspace_pills", include_str!("workspace_pills.css")),
    // Last: the drop states SidebarItem and TreeItem share (mailo gaps 6) must win over either
    // item's hover and current rules, which have the same specificity.
    ("drop_place", include_str!("drop_place.css")),
];
