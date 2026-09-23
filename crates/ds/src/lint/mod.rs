//! The stylesheet and markup linter every consumer runs in one test (ORCHESTRATION coherence
//! rules 1 and 2). It tokenizes with `cssparser`, never with substrings (CONVENTIONS
//! "Substrings are not tokens").
//!
//! ```ignore
//! #[test]
//! fn our_css_is_clean() {
//!     ds::lint::assert_clean(OUR_CSS, &ds::lint::LintConfig::default());
//! }
//! ```

pub mod assert;
pub mod blitz;
pub mod markup;
pub mod rule;
pub mod stylesheet;
pub mod tokenize;

mod colours;
mod declaration;
mod kind;
mod registry;
mod selector;
mod text;
mod walk;

pub use assert::assert_clean;
pub use markup::markup;
pub use rule::{LintConfig, Offence, Profile, Rule};
pub use stylesheet::stylesheet;
