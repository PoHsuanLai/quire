//! The words of the capability list that this crate owns (design: Extensions, section 4.2).
//!
//! Every capability word has exactly one owner, the crate that parses its entries. These are the
//! data words: an Extension that provides one carries pictures or values and no program.
//! Words owned elsewhere (`thumbnail`, `peek` and the rest of anyview-plugin's) are not here;
//! [`Provision::owns`](bayonet::manifest::Provides::owns) says no to them, and their owner reads
//! them itself from the raw entries of a scan.
//!
//! Room is left for the later data words (`wallpapers`, `sounds`, `cursors`): adding one is a
//! variant here, an entry type beside [`Look`](crate::Look), and a row in the capability table.

use ds_core::word::Word;

/// A capability word `ds-extensions` owns. The slug is the word a manifest writes in
/// `capability = "..."`.
#[derive(Debug, Clone, Copy, PartialEq, Eq, Hash, Word)]
#[word(case = snake)]
#[non_exhaustive]
pub enum Capability {
    /// A set of file-type pictures: one `dir` of them.
    FileIcons,
    /// A stylesheet that restyles every quire surface.
    Look,
}

#[cfg(test)]
mod tests {
    use super::Capability;
    use ds_core::word::Word;

    #[test]
    fn the_words_are_the_snake_case_spelling_a_manifest_writes() {
        let spelled: Vec<&str> = Capability::ALL.iter().map(|word| word.slug()).collect();
        assert_eq!(spelled, ["file_icons", "look"]);
        assert_eq!(Capability::parse("look"), Some(Capability::Look));
        assert_eq!(Capability::parse("thumbnail"), None);
    }
}
