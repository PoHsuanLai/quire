//! What a `SearchField`'s suggestions panel lists: sections of menu items, each under an
//! optional header. Data, and the one flattening the menu draws from.

use crate::components::menus::item::item::MenuItem;
use crate::host::measure::Anchor;
use ds_core::geometry::placement::Placement;

/// A group of suggestions under a header (Spotlight's "Top hit", "Mail", "People").
#[derive(Debug, Clone, PartialEq)]
pub struct SuggestionSection<T> {
    /// The group's title, drawn as a menu header; none for a group that needs none.
    pub header: Option<String>,
    /// The rows: any menu item, so a row can carry marks, an avatar and a second line.
    pub items: Vec<MenuItem<T>>,
}

impl<T> SuggestionSection<T> {
    /// A group of `items` under `header`.
    pub fn titled(header: impl Into<String>, items: Vec<MenuItem<T>>) -> Self {
        SuggestionSection {
            header: Some(header.into()),
            items,
        }
    }

    /// A group of `items` with no header.
    pub fn bare(items: Vec<MenuItem<T>>) -> Self {
        SuggestionSection {
            header: None,
            items,
        }
    }
}

/// Whether a new list of results highlights its first row.
#[derive(Debug, Clone, Copy, PartialEq, Eq, Hash, Default)]
pub enum InitialHighlight {
    /// Nothing is highlighted until a key or the pointer asks: Return submits the text.
    #[default]
    None,
    /// The first pickable row is highlighted whenever the results change, so Return picks it
    /// (Spotlight's top hit).
    TopHit,
}

/// What Escape does first when the field holds text and the suggestions are up.
#[derive(Debug, Clone, Copy, PartialEq, Eq, Hash, Default)]
pub enum EscapeOrder {
    /// The first Escape closes the panel, the second clears the text, the third goes on to the
    /// window (`NSSearchField`'s way).
    #[default]
    ClosePanelFirst,
    /// The first Escape clears the text (and with it the results), the second goes on to the
    /// window: Spotlight's way.
    ClearFirst,
}

/// How the suggestions are presented.
#[derive(Debug, Clone, Copy, PartialEq, Eq, Hash, Default)]
pub enum SuggestionsPresent {
    /// A pop-up menu under the field, a second surface.
    #[default]
    Popup,
    /// Spotlight's card: the field and its results are one surface, the field on top and the
    /// results under a hairline.
    Card,
}

/// Where a card floats: at the window level, hung from `anchor`. A card with no place is drawn
/// where it stands, and its host places it.
#[derive(Debug, Clone, PartialEq)]
pub struct CardPlace {
    /// What the card hangs from.
    pub anchor: Anchor,
    /// Which side of it, and how aligned.
    pub placement: Placement,
}

/// Who says which row is highlighted.
#[derive(Debug, Clone, PartialEq)]
pub enum SearchCursor<T> {
    /// The field's own: keys, the pointer and the initial highlight move it.
    Own,
    /// The caller's: this row's value (none highlights nothing). A key or the pointer only asks
    /// for a move through `on_highlight`.
    Is(Option<T>),
}

impl<T> Default for SearchCursor<T> {
    fn default() -> Self {
        SearchCursor::Own
    }
}

/// The sections as one menu: each non-empty section's header over its rows, a rule between
/// sections. An empty section draws nothing.
pub(crate) fn flatten<T: Clone>(sections: &[SuggestionSection<T>]) -> Vec<MenuItem<T>> {
    let mut rows = Vec::new();
    for section in sections.iter().filter(|section| !section.items.is_empty()) {
        if !rows.is_empty() {
            rows.push(MenuItem::Separator);
        }
        if let Some(header) = &section.header {
            rows.push(MenuItem::Header(header.clone()));
        }
        rows.extend(section.items.iter().cloned());
    }
    rows
}

#[cfg(test)]
mod tests {
    use super::{SuggestionSection, flatten};
    use crate::components::menus::item::item::MenuItem;

    #[test]
    fn sections_become_headers_rows_and_rules() {
        let item = |value: u8| MenuItem::new(value, format!("row {value}"));
        let sections = vec![
            SuggestionSection::titled("Top hit", vec![item(1)]),
            SuggestionSection::bare(Vec::new()),
            SuggestionSection::titled("People", vec![item(2), item(3)]),
            SuggestionSection::bare(vec![item(4)]),
        ];
        assert_eq!(
            flatten(&sections),
            vec![
                MenuItem::Header("Top hit".to_owned()),
                item(1),
                MenuItem::Separator,
                MenuItem::Header("People".to_owned()),
                item(2),
                item(3),
                MenuItem::Separator,
                item(4),
            ]
        );
    }
}
