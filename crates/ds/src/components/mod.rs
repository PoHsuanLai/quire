//! Every component, one `<name>.rs` and `<name>.css` pair each (design/04-COMPONENTS.md).

pub mod account_tile;
pub mod animated_list;
pub mod appearance_picker;
pub mod avatar;
pub mod button;
pub mod chip;
pub mod command_palette;
pub mod command_pill;
pub mod count;
pub mod dock_parts;
pub mod drag_ghost;
pub mod edge_strip;
pub mod hover_card;
pub mod hover_strip;
pub mod icon_button;
pub mod icon_view;
pub mod kbd;
pub mod link_pill;
pub mod list_row;
pub mod menu;
pub mod menu_bar_item;
pub mod menu_entry;
pub(crate) mod menu_keys;
pub(crate) mod menu_lines;
pub(crate) mod menu_match;
pub(crate) mod menu_panel;
pub(crate) mod menu_rows;
pub(crate) mod menu_tracker;
pub(crate) mod palette_lines;
pub(crate) mod palette_rows;
pub(crate) mod palette_select;
pub(crate) mod palette_shown;
pub mod peek;
pub mod popover;
pub mod press;
pub mod provider_mark;
pub mod scrim;
pub mod search_field;
pub mod section_header;
pub mod segmented;
pub mod selection_bubble;
pub mod send_pill;
pub mod sheet;
pub mod sidebar_item;
pub mod slider;
pub mod space_editor;
pub mod spinner;
pub mod sync_halo;
pub mod tabs;
pub mod text_input;
pub mod toast;
pub mod toggle;
pub mod tooltip;
pub mod vocab;
pub mod workspace_pills;

pub use account_tile::{AccountFace, AccountTile};
pub use animated_list::AnimatedList;
pub use appearance_picker::AppearancePicker;
pub use avatar::{Avatar, AvatarFace, AvatarShape, AvatarSize, AvatarTone, PersonHue, person_hue};
pub use button::{Button, ButtonVariant};
pub use chip::{Chip, ChipVariant};
pub use command_palette::{CommandPalette, CommandPaletteHost, PaletteEntrance};
pub use command_pill::CommandPill;
pub use count::{Count, CountPlace};
pub use dock_parts::{DockFloor, RunningDot};
pub use drag_ghost::{DragGhost, DropLine, Grip};
pub use edge_strip::{EdgeStrip, SideState};
pub use hover_card::{
    FlagTone, HoverCard, HoverCardPart, HoverMessage, HoverStat, HoverTarget, KeyHint,
};
pub use hover_strip::{ActionId, HoverStrip, StripAction};
pub use icon_button::{IconButton, IconButtonVariant, StatusMetrics};
pub use icon_view::IconView;
pub use kbd::{Kbd, KbdSize};
pub use link_pill::{LinkPill, LinkTarget};
pub use list_row::ListRow;
pub use menu::{Menu, MenuEntrance, MenuKind};
pub use menu_bar_item::MenuBarItem;
pub use menu_entry::{MenuEntry, Tile, Trail};
pub use menu_lines::Filter;
pub use palette_shown::Retain;
pub use peek::Peek;
pub use popover::{Dismiss, Elevation, Popover};
pub use press::{PointerButton, Press};
pub use provider_mark::{ImageSource, MarkSize, MarkStyle, Provider, ProviderMark};
pub use scrim::Scrim;
pub use search_field::SearchField;
pub use section_header::{HeaderKind, SectionHeader};
pub use segmented::{SegSize, SegmentedControl};
pub use selection_bubble::{BubbleAction, BubbleButton, BubbleMode, SelectionBubble};
pub use send_pill::{SendPhase, SendPill};
pub use sheet::Sheet;
pub use sidebar_item::{ItemKind, Preview, SidebarItem};
pub use slider::Slider;
pub use space_editor::{ActiveDot, DotIndex, SpaceDot, SpaceEditor};
pub use spinner::{Spinner, SpinnerKind};
pub use sync_halo::{SyncHalo, SyncState};
pub use tabs::Tabs;
pub use text_input::{Focus, InputVariant, TextInput, TextInputKind};
pub use toast::{ToastHost, use_toasts};
pub use toggle::Toggle;
pub use tooltip::{Shown, Tooltip, TooltipKind};
pub use vocab::{
    Availability, Check, DropState, Emphasis, Expanded, Fraction, Here, Key, PulseKey, PulsePhase,
    Selection, Shortcut, StaggerIndex, Switch,
};
pub use workspace_pills::{WorkspacePill, WorkspacePills};

/// Every component stylesheet, in the cascade's fixed order: `(component, css)`.
pub const CSS: &[(&str, &str)] = &[
    ("account_tile", include_str!("account_tile.css")),
    ("animated_list", include_str!("animated_list.css")),
    ("appearance_picker", include_str!("appearance_picker.css")),
    ("avatar", include_str!("avatar.css")),
    ("button", include_str!("button.css")),
    ("chip", include_str!("chip.css")),
    ("command_palette", include_str!("command_palette.css")),
    ("command_pill", include_str!("command_pill.css")),
    ("count", include_str!("count.css")),
    ("dock_parts", include_str!("dock_parts.css")),
    ("drag_ghost", include_str!("drag_ghost.css")),
    ("edge_strip", include_str!("edge_strip.css")),
    ("hover_card", include_str!("hover_card.css")),
    ("hover_strip", include_str!("hover_strip.css")),
    ("icon_button", include_str!("icon_button.css")),
    ("icon_view", include_str!("icon_view.css")),
    ("kbd", include_str!("kbd.css")),
    ("link_pill", include_str!("link_pill.css")),
    ("list_row", include_str!("list_row.css")),
    ("menu", include_str!("menu.css")),
    ("menu_bar_item", include_str!("menu_bar_item.css")),
    ("menu_entry", include_str!("menu_entry.css")),
    ("peek", include_str!("peek.css")),
    ("popover", include_str!("popover.css")),
    ("provider_mark", include_str!("provider_mark.css")),
    ("scrim", include_str!("scrim.css")),
    ("search_field", include_str!("search_field.css")),
    ("section_header", include_str!("section_header.css")),
    ("segmented", include_str!("segmented.css")),
    ("selection_bubble", include_str!("selection_bubble.css")),
    ("send_pill", include_str!("send_pill.css")),
    ("sheet", include_str!("sheet.css")),
    ("sidebar_item", include_str!("sidebar_item.css")),
    ("slider", include_str!("slider.css")),
    ("space_editor", include_str!("space_editor.css")),
    ("spinner", include_str!("spinner.css")),
    ("sync_halo", include_str!("sync_halo.css")),
    ("tabs", include_str!("tabs.css")),
    ("text_input", include_str!("text_input.css")),
    ("toast", include_str!("toast.css")),
    ("toggle", include_str!("toggle.css")),
    ("tooltip", include_str!("tooltip.css")),
    ("workspace_pills", include_str!("workspace_pills.css")),
];
