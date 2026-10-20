//! A page a schema declares: its id, its title, its glyph and where it sits in the sidebar
//! (design/22-SETTINGS.md section 9.3).

use serde::{Deserialize, Serialize};

use ds_core::settings::PageId;

/// What the sidebar calls a page.
#[derive(Debug, Clone, PartialEq, Eq, Serialize, Deserialize)]
#[serde(transparent)]
pub struct PageTitle(pub String);

/// The name of the glyph beside a page (`image`, `sun-moon`). The Settings app maps the name to
/// its own icon set and draws a plain window glyph for a name it does not know.
#[derive(Debug, Clone, PartialEq, Eq, Serialize, Deserialize)]
#[serde(transparent)]
pub struct PageSymbol(pub String);

/// Where a page sits in the sidebar: lower comes first. The built-in pages are spaced by ten, so
/// a program can put its page between two of them.
#[derive(Debug, Clone, Copy, PartialEq, Eq, PartialOrd, Ord, Serialize, Deserialize)]
#[serde(transparent)]
pub struct PageOrder(pub u16);

/// Whether the sidebar lists a page. A hidden page is still declared: a link, an intent or a
/// first-run step opens it by id, it just has no row of its own.
#[derive(Debug, Clone, Copy, Default, PartialEq, Eq, Serialize, Deserialize)]
#[serde(rename_all = "lowercase")]
pub enum Listing {
    /// A row in the sidebar, at the page's [`PageOrder`].
    #[default]
    Sidebar,
    /// Reachable by id alone.
    Hidden,
}

impl Listing {
    /// Whether this is the default, which a written declaration leaves out.
    fn is_default(&self) -> bool {
        *self == Listing::default()
    }
}

/// One `[[page]]` table.
#[derive(Debug, Clone, PartialEq, Eq, Serialize, Deserialize)]
pub struct PageDecl {
    pub id: PageId,
    pub title: PageTitle,
    pub symbol: PageSymbol,
    pub order: PageOrder,
    /// Whether the sidebar lists the page. Absent means it does.
    #[serde(default, skip_serializing_if = "Listing::is_default")]
    pub listing: Listing,
    /// Other names the page answers to: the names it had before it was declared, so an old link
    /// or an old schema still finds it. Absent when empty.
    #[serde(default, skip_serializing_if = "Vec::is_empty")]
    pub aliases: Vec<PageId>,
}

/// The `[[page]]` tables of a text that holds only pages: the shape of the built-in table.
#[derive(Debug, Default, Deserialize)]
pub(super) struct PageFile {
    #[serde(default)]
    pub page: Vec<PageDecl>,
}
