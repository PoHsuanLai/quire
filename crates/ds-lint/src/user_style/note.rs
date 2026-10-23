//! What the report says about a person's stylesheet.

use super::limits;
use std::fmt;

/// Why a stylesheet is not well-formed CSS. The browser and Blitz skip what they cannot read, so
/// each fault costs the rules around it and nothing else.
#[derive(Debug, Clone, PartialEq, Eq)]
pub enum ParseFault {
    /// A string that runs into the end of its line.
    BadString,
    /// A `url(` whose contents are not a URL.
    BadUrl,
    /// A `}`, `)` or `]` that closes nothing.
    StrayCloser(char),
    /// A `{`, `(` or `[` never closed.
    Unclosed(char),
    /// A selector at the top level with no `{ ... }` after it.
    RuleWithoutBlock,
    /// Something in a rule's body that is not `property: value`.
    DeclarationWithoutColon,
}

impl fmt::Display for ParseFault {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        match self {
            ParseFault::BadString => write!(f, "a string is not closed before the end of its line"),
            ParseFault::BadUrl => write!(
                f,
                "a url( is not closed or holds something that is not a URL"
            ),
            ParseFault::StrayCloser(c) => write!(f, "`{c}` closes nothing"),
            ParseFault::Unclosed(c) => write!(f, "`{c}` is never closed"),
            ParseFault::RuleWithoutBlock => {
                write!(f, "a selector with no `{{ ... }}` block after it")
            }
            ParseFault::DeclarationWithoutColon => write!(f, "expected `property: value`"),
        }
    }
}

/// What one note is about.
#[derive(Debug, Clone, PartialEq, Eq)]
pub enum UserStyleNoteKind {
    /// The text is not well-formed CSS here.
    Parse(ParseFault),
    /// `var(--name)` where no design-system token and no rule in the file declares `--name`.
    UnknownVariable {
        /// The custom property, `--` included.
        name: String,
    },
    /// A selector part outside the public surface (`ds::selectors`): it may change without
    /// notice.
    InternalSelector {
        /// The class, id or attribute as written: `.ds-chip-label`, `[data-x]`.
        selector: String,
    },
    /// `!important`: a user rule is unlayered and beats every design-system rule, so it is never needed.
    Important,
    /// A `url()` that is not a local `file:` or `data:` URL.
    NonLocalUrl {
        /// The URL as written.
        url: String,
    },
    /// `backdrop-filter` or `filter: blur()` over [`limits::MAX_BLUR_RADIUS_PX`], or any blur on
    /// a surface root: it makes every frame slower.
    HeavyBlur {
        /// `backdrop-filter` or `filter`.
        property: String,
    },
    /// A `box-shadow` whose layers add up to more than [`limits::MAX_SHADOW_BLUR_TOTAL_PX`] of
    /// blur.
    LargeShadow,
    /// `animation-iteration-count: infinite` (or the shorthand) outside a spinner part.
    EndlessAnimation,
    /// `transition: all`: every property that changes animates.
    AnimatesEverything,
    /// A property this desktop's renderer does not draw (the `BlitzUnsupported` list).
    NoEffect {
        /// The property, as written lower-cased.
        property: String,
    },
}

impl UserStyleNoteKind {
    /// What Settings says about this note (DESIGN.md section 7.5): short, no developer words.
    pub fn plain_words(&self) -> &'static str {
        match self {
            UserStyleNoteKind::Parse(_) => "Could not be read",
            UserStyleNoteKind::UnknownVariable { .. } => "Uses a setting that does not exist",
            UserStyleNoteKind::InternalSelector { .. } => "This part may change in an update",
            UserStyleNoteKind::Important => "Does not need to force a rule",
            UserStyleNoteKind::NonLocalUrl { .. } => "Cannot load from outside this folder",
            UserStyleNoteKind::HeavyBlur { .. } => "Heavy blur may slow the desktop",
            UserStyleNoteKind::LargeShadow => "Large shadows may slow the desktop",
            UserStyleNoteKind::EndlessAnimation => "This animation never stops",
            UserStyleNoteKind::AnimatesEverything => "Animates everything",
            UserStyleNoteKind::NoEffect { .. } => "Has no effect on this desktop",
        }
    }
}

/// One thing worth telling the person about their stylesheet, with where.
#[derive(Debug, Clone, PartialEq, Eq)]
pub struct UserStyleNote {
    /// 1-based line.
    pub line: u32,
    /// 1-based column.
    pub column: u32,
    /// What it is.
    pub kind: UserStyleNoteKind,
}

impl UserStyleNote {
    /// Whether the text is not well-formed CSS here, the one kind a command exits non-zero on.
    pub fn is_parse_error(&self) -> bool {
        matches!(self.kind, UserStyleNoteKind::Parse(_))
    }
}

impl fmt::Display for UserStyleNote {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        let place = format!("{}:{}", self.line, self.column);
        match &self.kind {
            UserStyleNoteKind::Parse(fault) => write!(f, "{place}: error: {fault}"),
            UserStyleNoteKind::UnknownVariable { name } => {
                write!(
                    f,
                    "{place}: note: `{name}` is not a design-system token or declared here"
                )
            }
            UserStyleNoteKind::InternalSelector { selector } => write!(
                f,
                "{place}: note: `{selector}` is not on the public selector surface and may change without notice"
            ),
            UserStyleNoteKind::Important => {
                write!(
                    f,
                    "{place}: note: `!important` is never needed: your rules beat the design system by the cascade layer"
                )
            }
            UserStyleNoteKind::NonLocalUrl { url } => {
                write!(
                    f,
                    "{place}: note: `{url}` is not a local file: or data: URL"
                )
            }
            UserStyleNoteKind::HeavyBlur { property } => write!(
                f,
                "{place}: note: `{property}` blurs a lot ({}px is the limit, and any blur on a surface root); {}",
                limits::MAX_BLUR_RADIUS_PX,
                self.kind.plain_words()
            ),
            UserStyleNoteKind::LargeShadow => write!(
                f,
                "{place}: note: the shadows add up to more than {}px of blur; {}",
                limits::MAX_SHADOW_BLUR_TOTAL_PX,
                self.kind.plain_words()
            ),
            UserStyleNoteKind::EndlessAnimation => write!(
                f,
                "{place}: note: {} (`infinite` outside a spinner)",
                self.kind.plain_words()
            ),
            UserStyleNoteKind::AnimatesEverything => write!(
                f,
                "{place}: note: {} (`transition: all`; name the properties)",
                self.kind.plain_words()
            ),
            UserStyleNoteKind::NoEffect { property } => write!(
                f,
                "{place}: note: `{property}` has no effect on this desktop"
            ),
        }
    }
}
