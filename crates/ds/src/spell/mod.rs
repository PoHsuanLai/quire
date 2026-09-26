//! Spellchecking an [`EditSurface`](crate::EditSurface) (design/04-COMPONENTS.md section 50):
//! the prop that turns it on, the pure rules (which text is a word, which marks survive an
//! edit, which word the caret is typing), and the host seam a checker plugs into.

pub mod host;
pub mod lang;
pub mod marks;
pub mod script;
pub mod words;

pub use host::{HostSpell, Learned, Paragraph, SpellFuture, SpellService};
pub use lang::{Lang, Spell};
pub use marks::{Misspelt, SpellReplace, Typing};
pub use script::is_cjk;
pub use words::{Span, word_at, words};
