//! What the linter rejects, and how strictly.

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
    /// A `:root` selector.
    RootSelector,
    /// A selector that styles quire's own `.ds-*` classes or `[data-theme|accent|motion|material]`.
    DsInternals,
    /// A `var(--x)` that nothing declares.
    UndeclaredVar,
    /// `!important`.
    Important,
    /// A property or value Blitz does not paint: `filter`, `backdrop-filter`,
    /// `mix-blend-mode`, `position: sticky`, `text-overflow`, `line-clamp`, `text-shadow`.
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
}

impl Rule {
    /// Every rule, in declaration order.
    pub const ALL: [Rule; 20] = [
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
        Rule::RootSelector,
        Rule::DsInternals,
        Rule::UndeclaredVar,
        Rule::Important,
        Rule::BlitzUnsupported,
        Rule::UnprefixedAttributeSelector,
        Rule::FocusPseudoClass,
        Rule::SvgPaintInCss,
    ];
}

/// How strict a run is.
#[derive(Debug, Clone, Copy, PartialEq, Eq, Hash, Default)]
pub enum Profile {
    /// Every rule except the raw-geometry ones (`RawFontSize`, `RawRadius`, `RawZIndex`).
    #[default]
    Standard,
    /// Every rule.
    Strict,
}

/// A lint run's settings.
#[derive(Debug, Clone, PartialEq, Eq, Default)]
pub struct LintConfig {
    /// How strict.
    pub profile: Profile,
    /// Custom properties the consumer declares itself, beyond quire's own.
    pub own_vars: Vec<String>,
}

/// One violation, located so a failure names it.
#[derive(Debug, Clone, PartialEq, Eq)]
pub struct Offence {
    /// What was broken.
    pub rule: Rule,
    /// 1-based line in the input.
    pub line: u32,
    /// 1-based column in the input.
    pub column: u32,
    /// The offending text.
    pub text: String,
}
