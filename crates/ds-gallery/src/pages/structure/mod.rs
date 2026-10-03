//! The structure page (design/30 sections 2.1 to 2.7, the P2 components): one section per
//! component that builds a window's content and its forms, every state it can express, live.

mod capsule;
mod date_picker;
mod drag_ghost;
mod field_row;
mod menu_bar;
mod sidebar;
mod sidebar_sections;
mod split_view;
mod stepper;
mod tab_view;
mod table;
mod titlebar;
mod toolbar;

use dioxus::prelude::*;

/// The Structure page.
#[component]
pub fn StructurePage() -> Element {
    rsx! {
        capsule::CapsuleSection {}
        stepper::StepperSection {}
        date_picker::DatePickerSection {}
        table::TableSection {}
        toolbar::ToolbarSection {}
        split_view::SplitViewSection {}
        split_view::SplitViewPeekSection {}
        sidebar::SidebarListSection {}
        sidebar_sections::SidebarSectionsSection {}
        tab_view::TabViewSection {}
        field_row::FieldRowSection {}
        menu_bar::MenuBarSection {}
        drag_ghost::DragGhostSection {}
        titlebar::TitlebarSection {}
    }
}
