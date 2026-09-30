//! The token-level faults: what `cssparser` itself refuses to make a token of, and blocks that
//! open without closing (the flat token stream of `tokenize` closes every block it opens, so it
//! cannot say).

use super::note::{ParseFault, UserStyleNote, UserStyleNoteKind};
use cssparser::{ParseError, Parser, ParserInput, Token};

/// Every fault in `css`, in source order.
pub(super) fn faults(css: &str) -> Vec<UserStyleNote> {
    let mut input = ParserInput::new(css);
    let mut parser = Parser::new(&mut input);
    let mut out = Vec::new();
    level(&mut parser, &mut out);
    out
}

fn note(line: u32, column: u32, fault: ParseFault) -> UserStyleNote {
    UserStyleNote {
        line: line + 1,
        column,
        kind: UserStyleNoteKind::Parse(fault),
    }
}

fn level(parser: &mut Parser, out: &mut Vec<UserStyleNote>) {
    loop {
        let at = parser.current_source_location();
        let start = parser.position();
        let token = match parser.next_including_whitespace_and_comments() {
            Ok(token) => token.clone(),
            Err(_) => return,
        };
        let opened = match token {
            Token::BadString(_) => {
                out.push(note(at.line, at.column, ParseFault::BadString));
                continue;
            }
            Token::BadUrl(_) => {
                out.push(note(at.line, at.column, ParseFault::BadUrl));
                continue;
            }
            Token::CloseCurlyBracket => {
                out.push(note(at.line, at.column, ParseFault::StrayCloser('}')));
                continue;
            }
            Token::CloseSquareBracket => {
                out.push(note(at.line, at.column, ParseFault::StrayCloser(']')));
                continue;
            }
            Token::CloseParenthesis => {
                out.push(note(at.line, at.column, ParseFault::StrayCloser(')')));
                continue;
            }
            Token::CurlyBracketBlock => ('{', '}'),
            Token::SquareBracketBlock => ('[', ']'),
            Token::ParenthesisBlock | Token::Function(_) => ('(', ')'),
            _ => continue,
        };
        let _: Result<(), ParseError<()>> = parser.parse_nested_block(|nested| {
            level(nested, out);
            Ok(())
        });
        // The nested parser stops at its closer or at the end of the text; only a closer ends
        // the consumed text with the closing character.
        if !parser.slice_from(start).ends_with(opened.1) {
            out.push(note(at.line, at.column, ParseFault::Unclosed(opened.0)));
        }
    }
}
