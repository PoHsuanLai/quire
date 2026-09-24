//! Tuned tokens: a token whose value a settings key can move. The stylesheet declares the token
//! on `.ds` as its *input* property with the key's default as the fallback
//! (`--fs-shell-menu:var(--shell-menu-font,13px)`), and a consumer writes the input on any
//! element around its surface ([`crate::ShellMetrics`], [`crate::DockMetrics`]). Every nested
//! `.ds` re-declares the token from the inherited input, so one inline write reaches every
//! scope under it, where writing the token itself would be undone by the next nested scope.

use super::name::VarName;

/// One tuned token: the property components read, the input a consumer writes, and the default.
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub struct Tuned {
    /// The token components read: `--fs-shell-menu`.
    pub token: VarName,
    /// The input a consumer writes inline: `--shell-menu-font`.
    pub input: VarName,
    /// The settings key's default, as CSS: `13px`.
    pub default: &'static str,
}

impl Tuned {
    /// The stylesheet's declaration on `.ds`: `--fs-shell-menu:var(--shell-menu-font,13px);`.
    pub fn declaration(self) -> String {
        format!(
            "{}:var({},{});",
            self.token.as_str(),
            self.input.as_str(),
            self.default
        )
    }

    /// An inline override of the input: `--shell-menu-font:14px;`.
    pub fn write(self, value: &str) -> String {
        format!("{}:{value};", self.input.as_str())
    }
}

/// A length in whole logical pixels, as a settings key stores it.
pub(crate) fn px(value: u16) -> String {
    format!("{value}px")
}

#[cfg(test)]
mod tests {
    use super::Tuned;
    use crate::tokens::VarName;

    #[test]
    fn a_tuned_token_reads_its_input_with_the_default_behind_it() {
        let token = Tuned {
            token: VarName("--fs-shell-menu"),
            input: VarName("--shell-menu-font"),
            default: "13px",
        };
        assert_eq!(
            token.declaration(),
            "--fs-shell-menu:var(--shell-menu-font,13px);"
        );
        assert_eq!(token.write("14px"), "--shell-menu-font:14px;");
    }
}
