//! The toolbar's search item (`NSSearchToolbarItem`): a search field while the band has room for
//! it, a magnifier button when it does not. Pressing the button opens the field where it was,
//! with the caret in it; the caret leaving an empty field, or Escape in one, folds it back.

use crate::components::chrome::toolbar::model::{Expansion, SearchFit};
use crate::components::content::icon_source::IconSource;
use crate::components::controls::button::Button;
use crate::components::controls::button_model::{Bezel, ImagePosition};
use crate::components::fields::text_field_focus::FieldFocus;
use crate::root::common::Common;
use dioxus::prelude::*;
use ds_core::geometry::units::Px;
use ds_core::press::Press;
use ds_core::vocab::Availability;
use ds_style::icon::Icon;
use ds_style::tokens::control_size::ControlSize;
use std::fmt;

/// A toolbar's search item.
#[derive(Clone, PartialEq)]
pub struct ToolbarSearch {
    /// What the field holds: the toolbar folds an open field back only while it is empty.
    pub value: String,
    /// The narrowest the field may be drawn; below that much room it collapses.
    pub min_width: Px,
    /// Draws the field for the seat it is given: a [`SearchField`](crate::components::menus::search::view::SearchField)
    /// with `focus: seat.focus()` and `onblur: seat.onblur()`.
    pub field: Callback<SearchSeat, Element>,
}

impl fmt::Debug for ToolbarSearch {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        f.debug_struct("ToolbarSearch")
            .field("value", &self.value)
            .field("min_width", &self.min_width)
            .finish_non_exhaustive()
    }
}

/// Where the toolbar draws the field.
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum SeatPlace {
    /// In the band, because it has room.
    Inline,
    /// Where the magnifier was, because it was pressed.
    Expanded,
}

/// The seat a [`ToolbarSearch::field`] draws its field for.
#[derive(Clone, Copy, PartialEq)]
pub struct SearchSeat {
    place: SeatPlace,
    fold: Callback<()>,
}

impl fmt::Debug for SearchSeat {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        f.debug_struct("SearchSeat")
            .field("place", &self.place)
            .finish_non_exhaustive()
    }
}

impl SearchSeat {
    /// Where the field is drawn.
    pub fn place(self) -> SeatPlace {
        self.place
    }

    /// How the field takes the keyboard: at once when it was just opened, never on its own
    /// otherwise.
    pub fn focus(self) -> FieldFocus {
        match self.place {
            SeatPlace::Inline => FieldFocus::Manual,
            SeatPlace::Expanded => FieldFocus::OnMount,
        }
    }

    /// What the field tells the toolbar when the caret leaves it.
    pub fn onblur(self) -> EventHandler<()> {
        self.fold
    }
}

/// The search item: the field in its seat, or the magnifier button.
pub(crate) fn search_item(
    search: &ToolbarSearch,
    fit: SearchFit,
    expansion: Signal<Expansion>,
) -> Element {
    let mut expansion = expansion;
    let place = match (fit, expansion()) {
        (SearchFit::Inline, _) => Some(SeatPlace::Inline),
        (SearchFit::Collapsed, Expansion::Open) => Some(SeatPlace::Expanded),
        (SearchFit::Collapsed, Expansion::Folded) => None,
    };
    let empty = search.value.is_empty();
    let fold = Callback::new(move |()| {
        if empty {
            expansion.set(Expansion::Folded);
        }
    });
    let Some(place) = place else {
        return rsx! {
            Button {
                label: "Search",
                title: Some("Search".to_owned()),
                bezel: Bezel::Toolbar,
                size: ControlSize::Large,
                image: ImagePosition::Only,
                icon: Some(IconSource::from(Icon::Search)),
                availability: Availability::Enabled,
                onclick: move |_: Press| expansion.set(Expansion::Open),
                common: Common::default(),
            }
        };
    };
    let seat = SearchSeat { place, fold };
    let width = search.min_width.0;
    let field = search.field.call(seat);
    rsx! {
        div {
            class: "ds-toolbar-search",
            "data-seat": if place == SeatPlace::Expanded { Some("expanded") } else { None },
            style: "width:{width}px",
            onkeydown: move |event: KeyboardEvent| {
                if event.key() == Key::Escape && place == SeatPlace::Expanded {
                    fold.call(());
                }
            },
            {field}
        }
    }
}
