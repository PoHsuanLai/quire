//! A row's click, kept by value: `ListRow`'s `onclick` hands the consumer a `MouseData` it can
//! keep (to read Shift for a peek), while dioxus holds the original behind an `Rc`. Split from
//! `list_row` so the row's own file holds the row.

use dioxus::html::geometry::{ClientPoint, ElementPoint, PagePoint, ScreenPoint};
use dioxus::html::input_data::{MouseButton, MouseButtonSet};
use dioxus::html::{
    HasMouseData, InteractionElementOffset, InteractionLocation, Modifiers, ModifiersInteraction,
    PointerInteraction,
};
use dioxus::prelude::*;

/// A pointer event's data, copied: the handler owns a `MouseData` it can keep, as the
/// `EventHandler<MouseData>` prop asks, while dioxus holds the original behind an `Rc`.
pub(crate) fn snapshot(data: &MouseData) -> MouseData {
    MouseData::new(Pressed {
        client: data.client_coordinates(),
        page: data.page_coordinates(),
        screen: data.screen_coordinates(),
        element: data.element_coordinates(),
        modifiers: data.modifiers(),
        held: data.held_buttons(),
        trigger: data.trigger_button(),
    })
}

/// What a click carried, kept by value.
struct Pressed {
    client: ClientPoint,
    page: PagePoint,
    screen: ScreenPoint,
    element: ElementPoint,
    modifiers: Modifiers,
    held: MouseButtonSet,
    trigger: Option<MouseButton>,
}

impl InteractionLocation for Pressed {
    fn client_coordinates(&self) -> ClientPoint {
        self.client
    }

    fn screen_coordinates(&self) -> ScreenPoint {
        self.screen
    }

    fn page_coordinates(&self) -> PagePoint {
        self.page
    }
}

impl InteractionElementOffset for Pressed {
    fn element_coordinates(&self) -> ElementPoint {
        self.element
    }
}

impl ModifiersInteraction for Pressed {
    fn modifiers(&self) -> Modifiers {
        self.modifiers
    }
}

impl PointerInteraction for Pressed {
    fn trigger_button(&self) -> Option<MouseButton> {
        self.trigger
    }

    fn held_buttons(&self) -> MouseButtonSet {
        self.held
    }
}

impl HasMouseData for Pressed {
    fn as_any(&self) -> &dyn std::any::Any {
        self
    }
}
