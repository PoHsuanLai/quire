//! The widget interface (design/23-WIDGETS.md section 9): every widget, quire's and other apps',
//! plugs in through one trait, [`Widget`], and is drawn only through [`WidgetCard`], which owns
//! the card. A provider hands a [`Timeline`] of dated entries and a refresh policy;
//! [`use_widget`] picks the entry for now on the design system's clock; [`WidgetRegistry`]
//! lists what a host can place. An app in another process sends the same timeline as a
//! [`WireTimeline`] (section 9.5; the transport is not built yet).
//!
//! A trait and not an enum, unlike the closed vocabularies elsewhere: the set of widgets is
//! open (every app may bring its own), and each kind has its own entry type.

pub(crate) mod battery;
pub(crate) mod calendar;
pub(crate) mod card;
pub(crate) mod clock;
pub(crate) mod contract;
pub(crate) mod gallery;
pub(crate) mod gallery_add;
pub(crate) mod gallery_book;
pub(crate) mod gallery_rows;
pub(crate) mod layout;
pub(crate) mod registry;
pub(crate) mod timeline;
pub(crate) mod use_widget;
pub(crate) mod wire;

pub use battery::{BatteryCell, BatteryEntry, BatteryWidget, MAX_RINGS};
pub use calendar::{
    EventLine, LARGE_EVENTS, MonthEntry, MonthFace, MonthIntent, MonthWidget, TodayLine,
};
pub use card::WidgetCard;
pub use clock::{ClockCity, ClockEntry, MAX_CITIES, WorldClockWidget};
pub use contract::{NoIntent, Widget, WidgetContext, WidgetKind, fit};
pub use gallery::{GalleryWords, WidgetGallery};
pub use layout::{
    DesktopGrid, GridCell, LayoutError, Order, WidgetAt, WidgetEdit, WidgetLayout, WidgetPlacement,
    apply, cells, first_free,
};
pub use registry::{
    TakenKind, UnsizedKind, WidgetInfo, WidgetRegistry, provide_widget_registry,
    use_widget_registry,
};
pub use timeline::{Dated, EntryDate, REFRESH_FLOOR, Refresh, RefreshAsk, Timeline, Wake};
pub use use_widget::use_widget;
pub use wire::{WireEntry, WireRefresh, WireTimeline};
