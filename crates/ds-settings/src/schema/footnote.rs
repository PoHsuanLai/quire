//! A section's footnote: the grey paragraph a Mac settings pane puts under a group of rows,
//! once, where one row's note would not do (design/27 §5.12; detent's style pass, 2026-10-27).

use super::key::Section;
use super::page::Page;
use serde::{Deserialize, Serialize};

/// The words of one footnote.
#[derive(Debug, Clone, PartialEq, Eq, Serialize, Deserialize)]
#[serde(transparent)]
pub struct FootnoteText(pub String);

/// One `[[footnote]]` table: the text under the section of that name on that page.
#[derive(Debug, Clone, PartialEq, Eq, Serialize, Deserialize)]
pub struct Footnote {
    /// The page the section is on.
    pub page: Page,
    /// The section it goes under.
    pub section: Section,
    /// What it says.
    pub text: FootnoteText,
}
