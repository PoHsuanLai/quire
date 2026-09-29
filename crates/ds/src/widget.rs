//! The desktop widgets' layout: the grid, placing and moving a widget, and the wire form of a
//! widget's timeline.

pub use crate::shell::widget::{
    calendar::LARGE_EVENTS,
    contract::fit,
    layout::{
        DesktopGrid, GridCell, LayoutError, Order, WidgetAt, WidgetEdit, WidgetLayout,
        WidgetPlacement, apply, cells,
    },
    registry::{TakenKind, UnsizedKind},
    timeline::REFRESH_FLOOR,
    wire::{WireRefresh, WireTimeline},
};
