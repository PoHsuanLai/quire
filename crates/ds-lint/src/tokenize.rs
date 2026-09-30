//! Turning CSS into located tokens with `cssparser`, so every rule matches whole tokens.
//!
//! `cssparser::Parser` skips over the contents of a block or function once it has yielded the
//! token that opens it (`Function`, `ParenthesisBlock`, `SquareBracketBlock`,
//! `CurlyBracketBlock`); a caller who wants what is inside has to descend with
//! `parse_nested_block`. [`tokens`] does that descent itself so the result is a genuinely flat,
//! fully balanced token stream: every opening token is followed, eventually, by a matching
//! synthetic close token (`)`, `]` or `}`) at the same nesting depth it opened at. That is what
//! lets [`super::stylesheet`] and [`super::markup`] find rule bodies and declaration values by
//! walking the list once, tracking brace depth, rather than by re-parsing.

use cssparser::{Parser, ParserInput, Token};

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
    let mut input = ParserInput::new(css);
    let mut parser = Parser::new(&mut input);
    let mut out = Vec::new();
    collect(&mut parser, &mut out);
    out
}

/// One pass over `parser`'s current nesting level, descending into every block or function so
/// the result stays flat.
fn collect(parser: &mut Parser, out: &mut Vec<Located>) {
    loop {
        let location = parser.current_source_location();
        let start = parser.position();
        let token = match parser.next_including_whitespace_and_comments() {
            Ok(token) => token.clone(),
            // End of this level's input (real end of file, or the nested parser's delimiter).
            Err(_) => return,
        };
        let text = parser.slice_from(start).to_owned();
        // `cssparser`'s line is 0-based; ours is 1-based (matches `SourceLocation::line`'s own
        // doc, which counts from the second line onward from 1, and the first from 0).
        out.push(Located {
            text,
            line: location.line + 1,
            column: location.column,
        });
        let close = match token {
            Token::CurlyBracketBlock => Some('}'),
            Token::SquareBracketBlock => Some(']'),
            Token::ParenthesisBlock | Token::Function(_) => Some(')'),
            _ => None,
        };
        let Some(close) = close else { continue };
        // Descend: `parse_nested_block` always consumes through the matching close token
        // (or to the end of input, if it is never closed) once the closure returns.
        let _: Result<(), cssparser::ParseError<()>> = parser.parse_nested_block(|nested| {
            collect(nested, out);
            Ok(())
        });
        // The close token is single-byte ASCII and never itself a newline, so the position
        // right after it is on the same line, one column further, as where it started.
        let after = parser.current_source_location();
        out.push(Located {
            text: close.to_string(),
            line: after.line + 1,
            column: after.column.saturating_sub(1).max(1),
        });
    }
}
