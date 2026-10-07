//! What a `SearchField`'s suggestions panel lists: sections of menu items, each under an
//! optional header. Data, and the one flattening the menu draws from.

use crate::components::menus::item::item::MenuItem;

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
