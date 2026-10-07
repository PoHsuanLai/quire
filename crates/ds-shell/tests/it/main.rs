//! The crate's integration tests: one executable, one module per former test file.
//! A separate test target needs a stated reason (CONVENTIONS.md, Tests).

mod accounts_ssr;
mod confirm_ssr;
mod confirm_words;
mod details_lint;
#[path = "../../../ds/tests/it/support/golden.rs"]
mod golden;
mod helpers_ssr;
mod self_lint;
mod stylesheet;
mod tokens;
