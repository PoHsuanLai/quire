//! The stylesheet and markup linter every consumer runs in one test (the
//! coherence rules, ARCHITECTURE.md "Repo rules"). It tokenizes with `cssparser`, never with
//! substrings (CONVENTIONS §14 "Substrings are not tokens").
//!
//! A consumer's test, with one reviewed exception:
//!
//! ```
//! use ds_lint::{Exception, LintConfig, Rule, assert_clean};
//!
//! const OUR_CSS: &str = ".row { color: var(--ink); }\n.fade { mask-image: linear-gradient(#000, transparent); }";
//! const EXCEPTIONS: &[Exception] = &[Exception {
//!     rule: Rule::HexColour,
//!     selector: ".fade",
//!     reason: "a mask's alpha, never painted",
//! }];
//!
//! assert_clean(OUR_CSS, &LintConfig { exceptions: EXCEPTIONS, ..LintConfig::new(&ds::kits()) });
//! ```

pub(crate) mod animation;
pub(crate) mod assert;
pub(crate) mod blitz;
pub(crate) mod colours;
pub(crate) mod declaration;
pub(crate) mod details;
pub(crate) mod hairline;
pub(crate) mod hig;
pub(crate) mod inline_style;
pub(crate) mod kind;
pub(crate) mod markup;
pub(crate) mod markup_hig;
pub(crate) mod rule;
pub(crate) mod selector;
pub(crate) mod severity;
pub(crate) mod stylesheet;
pub(crate) mod text;
pub mod tokenize;
pub(crate) mod user_style;
pub mod walk;

pub use assert::assert_clean;
pub use markup::{markup, markup_warnings};
pub use rule::{Exception, LintConfig, Offence, Profile, Rule};
pub use severity::{Severity, WARNINGS};
pub use stylesheet::{stylesheet, warnings};
pub use user_style::{ParseFault, UserStyleNote, UserStyleNoteKind, user_stylesheet};
