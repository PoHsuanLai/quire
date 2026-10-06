//! A card `ThreadRow`'s height, computed from the same tokens `thread_row.css` reads, so a
//! `VirtualList` consumer never hard-codes it and a type-scale change cannot desync it.

use ds_core::geometry::{scale::Scale, units::Px};
use ds_style::tokens::pixel::PixelToken;
use ds_style::tokens::spacing::SpacingToken;
use ds_style::tokens::token::{Token, TokenScope};
use ds_style::tokens::type_scale::FontSize;

/// The root's line height (`.ds` in reset.css), which the name and the snippet inherit.
const ROOT_LINE: f32 = 1.55;
/// The subject's own line height (`.ds-thread-sub` in thread_row.css).
const SUBJECT_LINE: f32 = 1.35;

/// How many text lines a card row draws.
#[derive(Debug, Clone, Copy, PartialEq, Eq, Hash)]
pub enum ThreadLines {
    /// The name and the subject.
    Two,
    /// The name, the subject and the snippet.
    Three,
}

/// A token's length in logical pixels, read from the token's own CSS value (`13.5px`).
fn px(token: impl Token) -> f32 {
    let value = token.css_value(TokenScope::BASE);
    value
        .as_str()
        .strip_suffix("px")
        .and_then(|number| number.parse().ok())
        .unwrap_or_else(|| panic!("a px token, not {value}"))
}

/// A card row's own box height at scale 1: the text's lines, the row's vertical padding
/// (`--s-10` twice) and its transparent border (`--hair` twice).
///
/// The gap between rows is NOT included: it is [`thread_card_gap`], so a `VirtualList`'s
/// pitch is `thread_card_height(lines) + thread_card_gap()`. The height does not depend on the
/// row's slots (`more`, `strip`, `star`): none of them makes the row taller. At a fractional
/// device scale the border rounds to whole device pixels, and the height moves by under a pixel.
pub fn thread_card_height(lines: ThreadLines) -> Px {
    let name = px(FontSize::Body) * ROOT_LINE;
    let subject = px(FontSize::Body) * SUBJECT_LINE;
    let snippet = px(FontSize::Meta) * ROOT_LINE;
    let text = match lines {
        ThreadLines::Two => name + subject,
        ThreadLines::Three => name + subject + snippet,
    };
    let hair = PixelToken::Hair.css(Scale::ONE);
    let border: f32 = hair
        .strip_suffix("px")
        .and_then(|number| number.parse().ok())
        .expect("--hair is a px length");
    Px(text + 2.0 * px(SpacingToken::S10) + 2.0 * border)
}

/// The gap below every card row (its `margin-bottom`, `--s-5`), for a list's pitch.
pub fn thread_card_gap() -> Px {
    Px(px(SpacingToken::S5))
}
