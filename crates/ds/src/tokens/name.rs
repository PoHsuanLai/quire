//! The name of a CSS custom property, the one way a token is referred to from CSS.

/// A custom property name, `--` included: `--t-tap`, `--accent-soft`.
#[derive(Debug, Clone, Copy, PartialEq, Eq, Hash, PartialOrd, Ord)]
pub struct VarName(pub &'static str);

impl VarName {
    /// The name as written in a declaration: `--t-tap`.
    pub fn as_str(self) -> &'static str {
        self.0
    }

    /// A reference to it as a value: `var(--t-tap)`.
    pub fn reference(self) -> String {
        format!("var({})", self.0)
    }
}
