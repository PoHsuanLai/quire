//! What a consumer's `use ds::prelude::*` brings in: the names an app draws with, one `pub use` per
//! name. Everything else is reached by its home path (ARCHITECTURE.md section 6).

// Root and appearance
pub use crate::assembly::app_style::AppStyle;
pub use crate::assembly::ds::Ds;
pub use crate::root::surface::Surface;
pub use ds_style::appearance::accent::Accent;
pub use ds_style::appearance::appearance::Appearance;
pub use ds_style::appearance::material::Material;
pub use ds_style::appearance::motion::Motion;
pub use ds_style::appearance::motion::MotionLevel;
pub use ds_style::appearance::system::SystemPrefs;
pub use ds_style::appearance::theme::Scheme;
pub use ds_style::appearance::theme::Theme;
pub use ds_style::appearance::typeface::Typeface;
pub use ds_style::scope::use_scope;
pub use ds_style::space::look::{Grain, SpaceLook};

// Vocabulary
pub use ds_core::vocab::Availability;
pub use ds_core::vocab::Check;
pub use ds_core::vocab::DropState;
pub use ds_core::vocab::Emphasis;
pub use ds_core::vocab::Fraction;
pub use ds_core::vocab::Percent;
pub use ds_core::vocab::Selection;
pub use ds_core::vocab::Severity;
pub use ds_core::vocab::Shortcut;
pub use ds_core::vocab::ShortcutKey;
pub use ds_core::vocab::Shown;
pub use ds_core::word::Word;

// Geometry
pub use ds_core::geometry::placement::Placement;
pub use ds_core::geometry::scale::Scale;
pub use ds_core::geometry::units::Point;
pub use ds_core::geometry::units::Px;
pub use ds_core::geometry::units::Rect;
pub use ds_core::geometry::units::Size;
pub use ds_style::tokens::hex::Alpha;

// Icons
pub use crate::components::content::icon_source::ExternalIcon;
pub use crate::components::content::icon_source::IconSource;
pub use crate::components::content::icon_view::IconView;
pub use crate::components::content::status::battery::BatteryGlyph;
pub use crate::components::content::status::bluetooth::BluetoothGlyph;
pub use crate::components::content::status::family::StatusState;
pub use crate::components::content::status::volume::VolumeGlyph;
pub use crate::components::content::status::wifi::WifiGlyph;
pub use ds_style::icon::Icon;
pub use ds_style::icon::render::IconSize;

// Progress and pending work
pub use crate::components::controls::progress::model::Progress;
pub use crate::components::controls::progress::model::ProgressStyle;
pub use crate::components::controls::progress::model::RingGap;
pub use crate::components::controls::progress::view::ProgressIndicator;
pub use ds_motion::detail::operation::Operation;
pub use ds_motion::detail::operation::PendingToken;

// Menus
pub use crate::components::menus::menu::cursor::MenuCursor;

// Fields
pub use crate::components::editor::surface::EditSurface;
pub use crate::components::fields::fact_list::{Fact, FactList};
pub use crate::components::fields::text_field::TextField;
pub use crate::components::fields::text_field_focus::FieldFocus;
pub use crate::components::fields::text_field_model::FieldBezel;
pub use crate::components::fields::text_field_model::Validity;
pub use crate::components::fields::text_field_model::{FieldKind, FieldText};
pub use crate::components::forms::form::Form;
pub use crate::components::forms::form_section::FormSection;
pub use crate::components::forms::icon_tile::{IconTile, TileFace};
pub use crate::components::forms::pane_stack::header::PageHeader;
pub use crate::components::forms::pane_stack::path::PanePath;
pub use crate::components::forms::pane_stack::stack::PaneStack;

// Overlays
pub use crate::components::overlays::alert::Alert;
pub use crate::components::overlays::drag_ghost::DragGhost;
pub use crate::components::overlays::hover_card::HoverCard;
pub use crate::components::overlays::inline_banner::InlineBanner;
pub use crate::components::overlays::loadable::Loadable;
pub use crate::components::overlays::loadable::Phase;
pub use crate::components::overlays::popover::Popover;
pub use crate::components::overlays::sheet::Sheet;
pub use crate::components::overlays::skeleton::Skeleton;
pub use crate::components::overlays::skeleton::SkeletonShape;
pub use crate::components::overlays::skeleton_row::SkeletonLines;
pub use crate::components::overlays::skeleton_row::SkeletonRow;
pub use crate::components::overlays::toast::use_toasts;
pub use crate::components::overlays::tooltip::Tooltip;
pub use crate::stack::host::use_overlays;

// Lists and content
pub use crate::components::content::avatar::Avatar;
pub use crate::components::content::image_source::ImageSource;
pub use crate::components::content::pdf_thumb::PdfThumb;
pub use crate::components::content::provider_mark::ProviderMark;
pub use crate::components::content::text_runs::RunTone;
pub use crate::components::content::text_runs::TextLine;
pub use crate::components::content::text_runs::TextRun;
pub use crate::components::lists::emoji_grid::grid::EmojiGrid;
pub use crate::components::lists::list::list::List;
pub use crate::components::lists::list::model::ListItem;
pub use crate::components::lists::preview::pane::PreviewPane;
pub use crate::components::lists::row::accessory::Accessory;
pub use crate::components::lists::row::action::RowAction;
pub use crate::components::lists::row::chord::RowChord;
pub use crate::components::lists::row::leading::RowLeading;
pub use crate::components::lists::row::row::Row;
pub use crate::components::lists::section_header::HeaderAction;
pub use crate::components::lists::section_header::SectionHeader;

// Spaces
pub use crate::components::app::space_editor::SpaceEditor;
pub use crate::components::app::space_editor::colour::SpaceColour;
pub use crate::components::app::space_editor::dot::SpaceDot;

// Chrome
pub use crate::components::chrome::window_frame::TrafficLights;
pub use crate::components::chrome::window_frame::WindowFrame;
pub use crate::window::host::WindowHost;
pub use crate::window::vocab::ResizeEdge;
pub use crate::window::vocab::WindowState;

// Motion
pub use ds_motion::anim::Anim;
pub use ds_motion::detail::cue::Cue;
pub use ds_motion::detail::detailed::Detailed;
pub use ds_motion::detail::moment::Moment;
pub use ds_motion::detail::use_detail::use_detail;
pub use ds_motion::symbol::effect::{
    Activity, LoopEffect, OnceEffect, SymbolEffect, TransitionEffect, Trigger,
};
pub use ds_motion::symbol::view::Symbol;

// Host
pub use crate::host::document::DocumentHost;
pub use crate::host::document::use_document_host;
pub use crate::host::focused::Focused;
pub use crate::host::measure::Measured;
pub use crate::host::signals::HostSignals;
pub use crate::spell::service::SpellService;
pub use crate::window::host::HostWindow;

// Controls, menus and root pieces a consumer draws that the groups above omit
pub use crate::components::content::label::Label;
pub use crate::components::controls::button::Button;
pub use crate::components::controls::choice::Choice;
pub use crate::components::controls::radio_group::RadioGroup;
pub use crate::components::controls::segmented::SegmentedControl;
pub use crate::components::controls::slider::Slider;
pub use crate::components::controls::toggle::Toggle;
pub use crate::components::menus::item::item::{AfterPick, MenuImage, MenuItem};
pub use crate::components::menus::item::text::{ItemText, Marks};
pub use crate::components::menus::menu::hung::{Hung, PanelWidth};
pub use crate::components::menus::menu::menu::Menu;
pub use crate::components::menus::menu::placement::MenuPlacement;
pub use crate::components::menus::palette::command_palette::CommandPalette;
pub use crate::components::menus::palette::palette_host::CommandPaletteHost;
pub use crate::components::menus::pick_list::PickList;
pub use crate::components::menus::search::model::SuggestionSection;
pub use crate::components::menus::search::view::SearchField;
pub use crate::components::overlays::empty_state::EmptyState;
pub use crate::components::overlays::flow::Flow;
pub use crate::components::overlays::side_panel::SidePanel;
pub use crate::root::chrome::RootChrome;
pub use crate::root::common::Common;
pub use crate::root::extent::RootExtent;
pub use ds_motion::settle::settle;
pub use ds_motion::timer::use_motion_timer;
pub use ds_style::appearance::resolve::resolve;
