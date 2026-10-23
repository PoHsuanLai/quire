//! The report on a person's stylesheet (ARCHITECTURE.md section 10): what is worth telling them,
//! never a reason to refuse it. User CSS is exempt from the design-system rules (raw colours,
//! durations and `font-family` are the point of it), so the notes are only these: text that is not
//! well-formed CSS, variables nothing declares, selectors off the public surface, `!important`,
//! URLs that are not local, and the cost notes (heavy blur, large shadows, an animation that never
//! stops, `transition: all`, a property with no effect here; their limits are in [`limits`]).

mod check;
mod cost;
mod items;
pub mod limits;
mod note;
mod scan;

use crate::tokenize;
use ds_style::kit::Kits;

pub use note::{ParseFault, UserStyleNote, UserStyleNoteKind};

/// The notes on `css`, in source order: parse errors first among equals, then the rest. `kits`
/// say which tokens, classes and attributes are the design system's public surface. Report only:
/// the caller loads the text whatever this returns.
pub fn user_stylesheet(css: &str, kits: &Kits) -> Vec<UserStyleNote> {
    user_stylesheet_with(css, kits, &[])
}

/// [`user_stylesheet`] with the parts apps publish: each table is one app's class names (from
/// `ds::selectors::AppSelectors::classes`), and a class in one is not an internal selector.
pub fn user_stylesheet_with(
    css: &str,
    kits: &Kits,
    app_tables: &[Vec<String>],
) -> Vec<UserStyleNote> {
    let mut known = kits.vocabulary();
    known
        .public_classes
        .extend(app_tables.iter().flatten().cloned());
    let tokens = tokenize::tokens(css);
    let (parts, mut notes) = items::parts(&tokens);
    notes.extend(scan::faults(css));
    notes.extend(check::notes(&parts, &known));
    notes.sort_by_key(|note| (note.line, note.column, !note.is_parse_error()));
    notes.dedup();
    notes
}

#[cfg(test)]
mod tests;
