//! The structure components in every state they can express, as data: the table the golden test
//! walks. One `Case` is one component in one state; its golden is
//! `tests/snapshots/structure/<component>/<state>.html`.

use dioxus::prelude::*;
use ds::components::chrome::sidebar::Sidebar;
use ds::components::chrome::split_view::model::{PaneSpec, SplitPane};
use ds::components::chrome::split_view::view::SplitView;
use ds::components::chrome::tab_view::TabView;
use ds::components::chrome::titlebar_parts::{DocumentState, TitleParts};
use ds::components::chrome::toolbar::model::{ToolbarItem, ToolbarRoom};
use ds::components::chrome::toolbar::view::Toolbar;
use ds::components::chrome::window_frame::WindowTitlebar;
use ds::components::fields::field_row::{FieldGroup, FieldRow, RowLayout};
use ds::components::fields::stepper::model::{Readout, StepRange};
use ds::components::fields::stepper::view::Stepper;
use ds::components::lists::table::model::{CellAlign, Sort, SortDirection, TableColumn, TableRow};
use ds::components::lists::table::view::Table;
use ds::components::overlays::drag_ghost::DragCount;
use ds::prelude::*;
use ds_core::vocab::RowState;
use ds_style::tokens::control_size::{ControlSize, SidebarSize};

/// One component in one state.
pub struct Case {
    pub component: &'static str,
    pub state: &'static str,
    pub make: fn() -> Element,
}

fn range() -> StepRange {
    StepRange::new(0, 10, 1)
}

fn items() -> Vec<ToolbarItem<u8>> {
    vec![
        ToolbarItem::new(0, "Back", Icon::ChevronLeft),
        ToolbarItem::new(1, "Share", Icon::Upload),
        ToolbarItem::new(2, "Tag", Icon::Tag).toggle(Check::On),
        ToolbarItem::new(3, "Search", Icon::Search).with(Availability::Disabled),
    ]
}

fn columns() -> Vec<TableColumn<u8>> {
    vec![
        TableColumn::new(0, "Name", Px(160.0)),
        TableColumn::new(1, "Size", Px(80.0)).aligned(CellAlign::Trailing),
        TableColumn::new(2, "Kind", Px(90.0)),
    ]
}

fn table_rows() -> Vec<TableRow<u8>> {
    vec![
        TableRow::new(
            1,
            "Report",
            vec![rsx! { "Report" }, rsx! { "2 MB" }, rsx! { "PDF" }],
        ),
        TableRow::new(
            2,
            "Notes",
            vec![rsx! { "Notes" }, rsx! { "12 KB" }, rsx! { "Text" }],
        ),
    ]
}

fn places() -> Vec<ListItem<u8>> {
    let place = |key: u8, title: &'static str, here: bool| {
        ListItem::row(
            key,
            title,
            rsx! {
                Row {
                    title: TextLine::from(title),
                    leading: RowLeading::Icon(Icon::Inbox),
                    state: RowState {
                        selection: if here { Selection::Selected } else { Selection::Unselected },
                        ..RowState::default()
                    },
                }
            },
        )
    };
    vec![place(1, "Inbox", true), place(2, "Archive", false)]
}

fn tabs(count: u8) -> Vec<Choice<u8>> {
    (0..count)
        .map(|at| Choice::new(at, format!("Tab {}", at + 1)))
        .collect()
}

fn pane(shown: Shown) -> Vec<SplitPane> {
    vec![SplitPane::new(PaneSpec::SIDEBAR, rsx! { p { "Side" } }).shown(shown)]
}

pub const CASES: &[Case] = &[
    Case {
        component: "stepper",
        state: "field",
        make: || rsx! { Stepper { label: "Copies", value: 4, range: range(), onchange: |_| {} } },
    },
    Case {
        component: "stepper",
        state: "bare",
        make: || rsx! { Stepper { label: "Copies", value: 4, range: range(), readout: Readout::Bare, onchange: |_| {} } },
    },
    Case {
        component: "stepper",
        state: "top",
        make: || rsx! { Stepper { label: "Copies", value: 10, range: range(), readout: Readout::Bare, onchange: |_| {} } },
    },
    Case {
        component: "stepper",
        state: "bottom",
        make: || rsx! { Stepper { label: "Copies", value: 0, range: range(), readout: Readout::Bare, onchange: |_| {} } },
    },
    Case {
        component: "stepper",
        state: "mini",
        make: || rsx! { Stepper { label: "Copies", value: 4, range: range(), size: ControlSize::Mini, onchange: |_| {} } },
    },
    Case {
        component: "stepper",
        state: "disabled",
        make: || rsx! { Stepper { label: "Copies", value: 4, range: range(), availability: Availability::Disabled, onchange: |_| {} } },
    },
    Case {
        component: "stepper",
        state: "busy",
        make: || rsx! { Stepper { label: "Copies", value: 4, range: range(), availability: Availability::Busy, onchange: |_| {} } },
    },
    Case {
        component: "table",
        state: "sorted-and-selected",
        make: || rsx! { Table::<u8, u8> { label: "Files", columns: columns(), rows: table_rows(), sort: Some(Sort { column: 0, direction: SortDirection::Ascending }), on_sort: |_| {}, selection: vec![2], cursor: Some(2), onselect: |_| {} } },
    },
    Case {
        component: "table",
        state: "descending",
        make: || rsx! { Table::<u8, u8> { label: "Files", columns: columns(), rows: table_rows(), sort: Some(Sort { column: 1, direction: SortDirection::Descending }), on_sort: |_| {}, onselect: |_| {} } },
    },
    Case {
        component: "toolbar",
        state: "fits",
        make: || rsx! { Toolbar::<u8> { leading: items()[..1].to_vec(), trailing: items()[1..].to_vec(), title: Some(TextLine::from("Downloads")), subtitle: Some(TextLine::from("14 items")), room: ToolbarRoom::Fixed(Px(800.0)), onpick: |_| {} } },
    },
    Case {
        component: "toolbar",
        state: "overflow",
        make: || rsx! { Toolbar::<u8> { leading: items()[..1].to_vec(), trailing: items()[1..].to_vec(), title: Some(TextLine::from("Downloads")), room: ToolbarRoom::Fixed(Px(250.0)), onpick: |_| {} } },
    },
    Case {
        component: "split_view",
        state: "open",
        make: || rsx! { SplitView { label: "Example", panes: pane(Shown::Visible), p { "Content" } } },
    },
    Case {
        component: "split_view",
        state: "folded",
        make: || rsx! { SplitView { label: "Example", panes: pane(Shown::Hidden), p { "Content" } } },
    },
    Case {
        component: "sidebar",
        state: "medium",
        make: || rsx! { Sidebar::<u8> { label: "Mail", items: places(), cursor: Some(1), onselect: |_| {}, header: rsx! { p { "Search" } } } },
    },
    Case {
        component: "sidebar",
        state: "large",
        make: || rsx! { Sidebar::<u8> { label: "Mail", items: places(), size: SidebarSize::Large, onselect: |_| {} } },
    },
    Case {
        component: "tab_view",
        state: "second",
        make: || rsx! { TabView::<u8> { label: "Settings", tabs: tabs(4), value: 1, onchange: |_| {}, p { "Body" } } },
    },
    Case {
        component: "tab_view",
        state: "six-at-most",
        make: || rsx! { TabView::<u8> { label: "Settings", tabs: tabs(8), value: 0, onchange: |_| {}, p { "Body" } } },
    },
    Case {
        component: "field_row",
        state: "setting",
        make: || rsx! { FieldRow { label: TextLine::from("Wi-Fi"), help: Some(TextLine::from("Join known networks.")), span { "control" } } },
    },
    Case {
        component: "field_row",
        state: "form",
        make: || rsx! { FieldRow { label: TextLine::from("Name"), layout: RowLayout::Form, span { "control" } } },
    },
    Case {
        component: "field_row",
        state: "disabled",
        make: || rsx! { FieldRow { label: TextLine::from("Airplane"), availability: Availability::Disabled, span { "control" } } },
    },
    Case {
        component: "field_row",
        state: "group",
        make: || rsx! { FieldGroup { title: "Network", FieldRow { label: TextLine::from("A"), span { "a" } } FieldRow { label: TextLine::from("B"), span { "b" } } } },
    },
    Case {
        component: "drag_ghost",
        state: "one",
        make: || rsx! { DragGhost { title: "Report", sub: "2 MB", at: Point { x: Px(452.0), y: Px(206.0) } } },
    },
    Case {
        component: "drag_ghost",
        state: "many",
        make: || rsx! { DragGhost { title: "5 items", sub: "Downloads", count: DragCount::new(5), at: Point { x: Px(452.0), y: Px(206.0) } } },
    },
    Case {
        component: "titlebar",
        state: "plain",
        make: || rsx! { WindowTitlebar { title: "Untitled" } },
    },
    Case {
        component: "titlebar",
        state: "parts",
        make: || rsx! { WindowTitlebar { title: "Report", parts: TitleParts { subtitle: Some("Edited today".to_owned()), proxy: Some(Icon::File), document: DocumentState::Edited }, lights: TrafficLights::Hidden } },
    },
];
