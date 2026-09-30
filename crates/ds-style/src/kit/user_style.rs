//! The person's own stylesheet as a value: the text of `style.css`, the last thing in the cascade
//! (ARCHITECTURE.md section 10).

use serde::{Deserialize, Serialize};

/// The text of the person's `style.css`. An empty text is no style at all; the text is never
/// parsed here, so a syntax error costs the rules after it and nothing else
/// (`ds_lint::user_stylesheet` reports them).
///
/// It lives beside [`super::KitRank::User`], which puts it last in the cascade, so `ds` can take
/// it as a prop while `ds-settings` stores it.
#[derive(Debug, Clone, PartialEq, Eq, Hash, Default, Serialize, Deserialize)]
#[serde(transparent)]
pub struct UserStyle(pub String);

impl UserStyle {
    /// Whether there is no text worth drawing: nothing or only whitespace.
    pub fn is_blank(&self) -> bool {
        self.0.trim().is_empty()
    }
}
