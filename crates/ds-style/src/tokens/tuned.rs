//! Tuned tokens: a token whose value a look or a style sheet can move. The stylesheet declares
//! the token on `.ds` as `var(--input, default)` with the default as the fallback
//! (`--fs-shell-menu:var(--shell-menu-font,13px)`), and a consumer may write the input on any
//! element around its surface (`DockMetrics` in sill). Every nested `.ds` re-declares the
//! token from the inherited input, so one inline write reaches every scope under it, where
//! writing the token itself would be undone by the next nested scope. A family is tuned by
//! `#[token(kind = tuned)]`; its variants name the `input` and the default `value`.

#[cfg(test)]
mod tests {
    use crate::tokens::pixel::PixelToken;
    use crate::tokens::token::TokenScope;
    use ds_core::word::Word;

    #[test]
    fn a_tuned_token_reads_its_input_with_the_default_behind_it() {
        use crate::tokens::token::Token;
        let hair = PixelToken::Hair;
        assert_eq!(
            hair.css_value(TokenScope::BASE).as_str(),
            "var(--scale-hair,1px)"
        );
        assert_eq!(hair.write("2px"), "--scale-hair:2px;");
        assert!(PixelToken::ALL.contains(&hair));
    }
}
