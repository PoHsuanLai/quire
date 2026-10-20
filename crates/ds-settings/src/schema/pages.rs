//! The pages Settings can show: the built-in table plus every `[[page]]` a schema declares.

use super::page::Page;
use super::page_decl::{PageDecl, PageFile, PageTitle};
use super::page_id::PageId;
use super::program::Schema;

/// The built-in pages, as data (`pages.toml`).
const BUILT_IN: &str = include_str!("pages.toml");

/// What Settings knows about a key's page.
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum PageStatus {
    /// The page is in the table: built in, or declared by a schema.
    Declared,
    /// Nothing declares the page. The key is kept; Settings shows it under a page named after the
    /// id, as it shows a third-party program's.
    Unknown,
}

/// Every page Settings can name, in sidebar order.
#[derive(Debug, Clone, PartialEq, Eq, Default)]
pub struct PageTable {
    pages: Vec<PageDecl>,
}

impl PageTable {
    /// The built-in pages alone.
    pub fn built_in() -> PageTable {
        let file: PageFile = toml::from_str(BUILT_IN).unwrap_or_default();
        PageTable::default().declaring(&file.page)
    }

    /// The built-in pages and the `[[page]]` tables of `schemas`.
    pub fn of_schemas(schemas: &[Schema]) -> PageTable {
        schemas.iter().fold(PageTable::built_in(), |table, schema| {
            table.declaring(&schema.pages)
        })
    }

    /// This table with `declared` added. A page whose id is already here replaces the earlier
    /// one, so a program can retitle a built-in page; the table stays in sidebar order.
    pub fn declaring(mut self, declared: &[PageDecl]) -> PageTable {
        for decl in declared {
            self.pages.retain(|page| page.id != decl.id);
            self.pages.push(decl.clone());
        }
        self.pages.sort_by_key(|page| page.order);
        self
    }

    /// The pages in sidebar order.
    pub fn pages(&self) -> &[PageDecl] {
        &self.pages
    }

    /// The page declared with `id`.
    pub fn get(&self, id: &PageId) -> Option<&PageDecl> {
        self.pages.iter().find(|page| page.id == *id)
    }

    /// The id `text` names: an id, one of a page's aliases, or an id behind the old `app:` prefix.
    /// `None` when no declared page answers to it.
    pub fn resolve(&self, text: &str) -> Option<PageId> {
        let text = text.trim();
        let text = text.strip_prefix("app:").unwrap_or(text);
        self.pages
            .iter()
            .find(|page| {
                page.id.as_str() == text || page.aliases.iter().any(|a| a.as_str() == text)
            })
            .map(|page| page.id.clone())
    }

    /// The declaration `page` names, through its aliases: `Page::App("Software Update")` is the
    /// `software-update` page.
    pub fn declared(&self, page: &Page) -> Option<&PageDecl> {
        self.resolve(page.id().as_str())
            .and_then(|id| self.get(&id))
    }

    /// Whether `page` is declared.
    pub fn status(&self, page: &Page) -> PageStatus {
        match self.declared(page) {
            Some(_) => PageStatus::Declared,
            None => PageStatus::Unknown,
        }
    }

    /// The title of `page`: the declared one, else its label.
    pub fn title_of(&self, page: &Page) -> PageTitle {
        self.declared(page).map_or_else(
            || PageTitle(page.label().to_owned()),
            |decl| decl.title.clone(),
        )
    }
}

#[cfg(test)]
mod tests {
    use super::{PageStatus, PageTable};
    use crate::schema::{Page, PageDecl, PageId, PageOrder, PageSymbol, PageTitle};

    fn wallpaper_retitled() -> PageDecl {
        PageDecl {
            id: PageId::new("wallpaper"),
            title: PageTitle("Backdrop".to_owned()),
            symbol: PageSymbol("image".to_owned()),
            order: PageOrder(85),
            aliases: Vec::new(),
        }
    }

    #[test]
    fn the_built_in_table_parses_and_declares_every_fixed_page_and_the_new_ones() {
        let table = PageTable::built_in();
        for page in Page::fixed() {
            assert_eq!(table.status(&page), PageStatus::Declared, "{page:?}");
            assert_eq!(table.title_of(&page).0, page.label(), "{page:?}");
        }
        for id in ["wallpaper", "wifi", "bluetooth", "displays"] {
            assert!(table.get(&PageId::new(id)).is_some(), "{id}");
        }
        let orders: Vec<_> = table.pages().iter().map(|page| page.order).collect();
        assert!(
            orders.windows(2).all(|pair| pair[0] < pair[1]),
            "{orders:?}"
        );
    }

    #[test]
    fn a_name_resolves_to_its_page_through_ids_aliases_and_the_old_prefix() {
        const CASES: &[(&str, Option<&str>)] = &[
            ("dock", Some("dock")),
            ("mouse-and-gestures", Some("mouse-and-gestures")),
            ("mouse_and_gestures", Some("mouse-and-gestures")),
            (" wallpaper ", Some("wallpaper")),
            ("app:wifi", Some("wifi")),
            ("Software Update", Some("software-update")),
            ("app:Software Update", Some("software-update")),
            ("Assistant Model", Some("assistant-model")),
            ("assistant_model", Some("assistant-model")),
            ("nowhere", None),
            ("", None),
        ];
        let table = PageTable::built_in();
        for (text, want) in CASES {
            assert_eq!(table.resolve(text), want.map(PageId::new), "{text:?}");
        }
    }

    #[test]
    fn a_declared_page_replaces_the_built_in_one_and_an_undeclared_page_is_unknown() {
        let table = PageTable::built_in().declaring(&[wallpaper_retitled()]);
        assert_eq!(table.title_of(&Page::declared("wallpaper")).0, "Backdrop");
        let ids: Vec<&str> = table.pages().iter().map(|page| page.id.as_str()).collect();
        let at = |id: &str| ids.iter().position(|found| *found == id);
        assert!(at("wallpaper") > at("displays") && at("wallpaper") < at("spaces"));
        assert_eq!(
            table.status(&Page::declared("not-declared")),
            PageStatus::Unknown
        );
        assert_eq!(
            table.title_of(&Page::declared("not-declared")).0,
            "not-declared"
        );
    }
}
