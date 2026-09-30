//! `Word`: a closed vocabulary. Every enum whose variants are a fixed set of named things (a
//! theme, a size, a state) implements it by `#[derive(Word)]`, which gives the list of variants,
//! the attribute value or CSS word each one is written as, and the label a person reads.

#[cfg(test)]
mod tests;

pub use ds_core_derive::Word;

/// A closed set of words: an enum with no data in its variants.
///
/// `slug` is the kebab-case value an attribute or a stylesheet selector carries (the variant's
/// name unless `#[word(slug = "x")]` says otherwise, or the enum's `#[word(case = snake)]` when
/// it is stored and so matches its serde name); `label` is what a person reads (`Read only`).
pub trait Word: Copy + Eq + std::hash::Hash + 'static {
    /// Every variant, in declaration order.
    const ALL: &'static [Self];

    /// The attribute or stylesheet word for this variant.
    fn slug(self) -> &'static str;

    /// The words a person reads for this variant.
    fn label(self) -> &'static str;

    /// The variant written as `slug`, or `None` for a word outside the set.
    fn parse(slug: &str) -> Option<Self> {
        Self::ALL.iter().copied().find(|word| word.slug() == slug)
    }
}
