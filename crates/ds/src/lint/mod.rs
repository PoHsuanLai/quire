//! The stylesheet and markup linter every consumer runs in one test (ORCHESTRATION coherence
//! rules 1 and 2). It tokenizes with `cssparser`, never with substrings (CONVENTIONS
//! "Substrings are not tokens").
//!
//! A consumer's test, with one reviewed exception:
//!
//! ```
//! use ds::lint::{Exception, LintConfig, Rule, assert_clean};
//!
//! const OUR_CSS: &str = ".row { color: var(--ink); }\n.fade { mask-image: linear-gradient(#000, transparent); }";
//! const EXCEPTIONS: &[Exception] = &[Exception {
//!     rule: Rule::HexColour,
//!     selector: ".fade",
//!     reason: "a mask's alpha, never painted",
//! }];
//!
//! assert_clean(OUR_CSS, &LintConfig { exceptions: EXCEPTIONS, ..LintConfig::default() });
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
pub use rule::{Exception, LintConfig, Offence, Profile, Rule};
pub use stylesheet::stylesheet;
