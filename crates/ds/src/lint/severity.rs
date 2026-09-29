//! How much an offence weighs. A new rule lands as a [`Severity::Warning`]: reported by
//! [`super::warnings`], [`super::markup_warnings`] and printed by [`super::assert_clean`], but
//! never failing a consumer's test, until the consumers have been swept and told
//! (design/27-HIG-PARITY.md section 7, H0). [`super::stylesheet`] and [`super::markup`] return
//! errors only, so a consumer's `assert!(offences.is_empty())` is unmoved by a warning.

use super::rule::{Offence, Rule};

/// Whether an offence fails a run.
#[derive(Debug, Clone, Copy, PartialEq, Eq, Hash)]
pub enum Severity {
    /// Fails [`super::assert_clean`] and is returned by [`super::stylesheet`] and
    /// [`super::markup`].
    Error,
    /// Reported, never failing: returned by [`super::warnings`] and [`super::markup_warnings`].
    Warning,
}

/// The rules that are warnings today: the HIG guardrails (design/27 section 7, H0), and the
/// `filter` functions a pinned backend does not paint.
pub const WARNINGS: [Rule; 6] = [
    Rule::FilterNotPainted,
    Rule::PointerCursor,
    Rule::MinFontSize,
    Rule::FocusRingShape,
    Rule::UnnamedControl,
    Rule::ThreeDots,
];

impl Rule {
    /// How much an offence of this rule weighs.
    pub fn severity(self) -> Severity {
        if WARNINGS.contains(&self) {
            Severity::Warning
        } else {
            Severity::Error
        }
    }
}

impl Offence {
    /// Its rule's severity.
    pub fn severity(&self) -> Severity {
        self.rule.severity()
    }
}

/// `offences` split into errors and warnings, order kept.
pub(super) fn split(offences: Vec<Offence>) -> (Vec<Offence>, Vec<Offence>) {
    offences
        .into_iter()
        .partition(|offence| offence.severity() == Severity::Error)
}
