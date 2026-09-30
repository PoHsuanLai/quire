//! What the linter rejects, and how strictly.

use ds_style::kit::{Kits, KnownNames};

/// One thing a consumer stylesheet or markup may not do.
#[derive(Debug, Clone, Copy, PartialEq, Eq, Hash, PartialOrd, Ord)]
pub enum Rule {
    /// A `#rgb`/`#rrggbb` colour: colours come from the token table.
    HexColour,
    /// `rgb()`, `rgba()`, `hsl()`, `oklch()`, `color-mix()` and the rest.
    ColourFunction,
    /// A named colour: `white`, `red`.
    NamedColour,
    /// `currentColor` anywhere but `stroke` and `fill`.
    CurrentColourOutsideStrokeFill,
    /// A raw `ms`/`s` duration: durations are `--t-*`.
    RawDuration,
    /// A raw `cubic-bezier()` or keyword easing: easings are `--e-*`.
    RawEasing,
    /// A `@keyframes` block: keyframes live in quire.
    Keyframes,
    /// An `animation-name` quire does not export.
    UnknownAnimation,
    /// A `font-family`: faces are `--font-*`.
    FontFamily,
    /// A raw `font-size`: sizes are `--fs-*`.
    RawFontSize,
    /// A raw `border-radius`: radii are `--r-*`.
    RawRadius,
    /// A raw `z-index`: layers are `--z-*`.
    RawZIndex,
    /// A literal `px` in a `margin`, `padding` or `gap`: steps are `--s-*` (design/01-LAYOUT.md
    /// section 2).
    RawSpacing,
    /// A literal hairline (`1px`, `.5px`) as a border or outline width, or as a box's whole
    /// `width`/`height`: at a fractional scale it blurs across two device pixels. Lines are
    /// `var(--hair)` and `var(--hairline)` (design/01-LAYOUT.md section 2.1).
    RawHairline,
    /// A `:root` selector.
    RootSelector,
    /// A selector that styles quire's own `.ds-*` classes or `[data-theme|accent|motion|material]`.
    DsInternals,
    /// A `var(--x)` that nothing declares.
    UndeclaredVar,
    /// `!important`.
    Important,
    /// A property or value Blitz does not paint, or that fights the host:
    /// `backdrop-filter`, `mix-blend-mode`, `position: sticky`, `text-overflow`, `line-clamp`,
    /// `text-shadow`, `scroll-behavior: smooth` (design/11-BEHAVIOUR-scroll.md#11-3-1-ownership).
    BlitzUnsupported,
    /// An attribute selector without the `*|` namespace: `[data-theme=dark]` never matches on
    /// Blitz, `[*|data-theme=dark]` matches there and in browsers (spike S2).
    UnprefixedAttributeSelector,
    /// `:focus-visible` or `:focus-within`, hard-coded false in blitz-dom (spike S12). Focus
    /// rings are `.ds[*|data-modality=keyboard] :focus`.
    FocusPseudoClass,
    /// A CSS `stroke` or `fill`: on Blitz SVG paint works only as attributes (spike S6), which
    /// `Glyph` writes.
    SvgPaintInCss,
    /// Markup: a class on a rendered element that no rule in scope styles (coherence rule 2).
    UnstyledClass,
    /// Markup: an element quire draws for you, written by hand: an `<svg>` that is not a
    /// `Glyph` (`.ds-ic`), a `<button>`, `<input>`, `<select>` or `<textarea>` outside a quire
    /// component (coherence rule 2).
    RawMarkup,
    /// `animation-iteration-count: infinite`, or `infinite` in the `animation` shorthand: a loop
    /// never lets the surface idle. A pending state is a bounded loop (`ds::detail::use_pending`)
    /// that holds still at its deadline (design/26-DETAILS.md R3, R4). Every profile.
    InfiniteLoop,
    /// Under [`Profile::Details`]: an `animation` or `transition` timed by a duration or easing
    /// token the details grammar does not play (`--t-ambient`, `--t-spin`, `--t-float`,
    /// `--t-awake`, `--t-sail`, `--t-boat-return`, `--t-send-ring`, `--t-flash`,
    /// `--t-count-step`, `--t-fill`): moments are timed by the grammar's tokens (`ds::detail::grammar`,
    /// design/26-DETAILS.md sections 3.1 and 3.4).
    OffGrammarTiming,
    /// `cursor: pointer` on a rule whose subject is not a link: controls use the arrow, and the
    /// pointing hand means only "this is a link" (design/27-HIG-PARITY.md section 6.3). A link is
    /// an `a`, a `[role=link]`, or a class with a `link` segment (`.ds-run-link`). A
    /// [`super::Severity::Warning`] under Strict until sill's sweep is clean.
    PointerCursor,
    /// A font size under the 10 px floor (design/27 section 3.16): a literal below 10 px, or a
    /// `var(--fs-*)` whose System value is below it. A [`super::Severity::Warning`] under Strict.
    MinFontSize,
    /// A `border-radius` in a `:focus` rule that is not `calc(var(--r-*) + var(--focus-gap))`:
    /// a ring follows its control's shape, one gap out (design/27 sections 3.9 and 6.4). A
    /// [`super::Severity::Warning`] under Strict.
    FocusRingShape,
    /// Markup: an interactive element (a button, a link, a form control, a `role` that takes
    /// input) with no accessible name: no text, no `aria-label`, `aria-labelledby`, `title` or
    /// `alt` (design/27 section 3.1). A [`super::Severity::Warning`] under Strict.
    UnnamedControl,
    /// Markup: "..." (three full stops) in text or a label attribute where "…" belongs
    /// (design/02-TYPE.md section 13). A [`super::Severity::Warning`] under Strict.
    ThreeDots,
}

impl Rule {
    /// Every rule, in declaration order.
    pub const ALL: [Rule; 31] = [
        Rule::HexColour,
        Rule::ColourFunction,
        Rule::NamedColour,
        Rule::CurrentColourOutsideStrokeFill,
        Rule::RawDuration,
        Rule::RawEasing,
        Rule::Keyframes,
        Rule::UnknownAnimation,
        Rule::FontFamily,
        Rule::RawFontSize,
        Rule::RawRadius,
        Rule::RawZIndex,
        Rule::RawSpacing,
        Rule::RawHairline,
        Rule::RootSelector,
        Rule::DsInternals,
        Rule::UndeclaredVar,
        Rule::Important,
        Rule::BlitzUnsupported,
        Rule::UnprefixedAttributeSelector,
        Rule::FocusPseudoClass,
        Rule::SvgPaintInCss,
        Rule::UnstyledClass,
        Rule::RawMarkup,
        Rule::InfiniteLoop,
        Rule::OffGrammarTiming,
        Rule::PointerCursor,
        Rule::MinFontSize,
        Rule::FocusRingShape,
        Rule::UnnamedControl,
        Rule::ThreeDots,
    ];
}

/// How strict a run is.
#[derive(Debug, Clone, Copy, PartialEq, Eq, Hash, Default)]
pub enum Profile {
    /// Every rule but [`Rule::OffGrammarTiming`]; the HIG guardrails
    /// ([`super::Severity::Warning`] rules) report without failing.
    #[default]
    Strict,
    /// Every rule: `Strict` and the details grammar's timing (design/26-DETAILS.md). Off by
    /// default so a consumer opts in once its own sheets pass.
    Details,
}

/// One offence a consumer has decided to live with, and why: every offence of `rule` whose
/// selector is exactly `selector` is suppressed. The reason is for the reviewer; `assert_clean`
/// prints how many offences each exception swallowed, and fails on one that swallowed none: the
/// offence it excused has gone (or its selector was mistyped), and left in place it would
/// silently excuse the next offence on that selector.
#[derive(Debug, Clone, Copy, PartialEq, Eq, Hash)]
pub struct Exception {
    /// The rule it silences.
    pub rule: Rule,
    /// The selector it silences it on, as the offence reports it: a stylesheet rule's selector
    /// text (`.ds-truncate`), an at-rule's prelude (`@keyframes spin`), or a markup element as
    /// `tag.class.class` (`span.ds-avatar`). Compared whole, never as a prefix.
    pub selector: &'static str,
    /// Why this one is allowed.
    pub reason: &'static str,
}

impl Exception {
    /// Whether this exception silences `offence`.
    pub fn covers(&self, offence: &Offence) -> bool {
        self.rule == offence.rule && self.selector == offence.selector
    }
}

/// A lint run's settings.
#[derive(Debug, Clone, PartialEq, Eq)]
pub struct LintConfig {
    /// How strict.
    pub profile: Profile,
    /// Custom properties the consumer declares itself, beyond quire's own.
    pub own_vars: Vec<String>,
    /// Offences to suppress, each with its reason.
    pub exceptions: &'static [Exception],
    /// What the kits in play let a stylesheet name: custom properties, keyframes, the details
    /// grammar's timing tokens.
    pub known: KnownNames,
}

impl LintConfig {
    /// The default profile with no exceptions and no consumer variables, over the vocabulary of
    /// `kits`.
    pub fn new(kits: &Kits) -> LintConfig {
        LintConfig {
            profile: Profile::default(),
            own_vars: Vec::new(),
            exceptions: &[],
            known: kits.vocabulary(),
        }
    }

    /// Splits `offences` into those no exception covers and those one does.
    pub fn partition(&self, offences: Vec<Offence>) -> (Vec<Offence>, Vec<Offence>) {
        offences.into_iter().partition(|offence| {
            !self
                .exceptions
                .iter()
                .any(|exception| exception.covers(offence))
        })
    }
}

/// One violation, located so a failure names it.
#[derive(Debug, Clone, PartialEq, Eq)]
pub struct Offence {
    /// What was broken.
    pub rule: Rule,
    /// Where: the selector of the rule it sits in, an at-rule's prelude, or for markup the
    /// element as `tag.class.class`. What an [`Exception`] matches.
    pub selector: String,
    /// 1-based line in the input.
    pub line: u32,
    /// 1-based column in the input.
    pub column: u32,
    /// The offending text.
    pub text: String,
}
