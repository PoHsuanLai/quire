//! Turning CSS into located tokens with `cssparser`, so every rule matches whole tokens.
#![allow(unused_variables)] // Freeze stubs: remove with the last todo!().

/// One CSS token with where it starts.
#[derive(Debug, Clone, PartialEq)]
pub struct Located {
    /// The token's text as written.
    pub text: String,
    /// 1-based line.
    pub line: u32,
    /// 1-based column.
    pub column: u32,
}

/// Every token in `css`, in order.
pub fn tokens(css: &str) -> Vec<Located> {
    todo!()
}
